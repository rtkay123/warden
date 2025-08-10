// https://github.com/svix/svix-webhooks/tree/4ede01a3209658615bb8d3153965c5c3a2e1b7ff/server/svix-server/src/redis
pub mod cluster;
pub mod sentinel;

use std::{sync::Arc, time::Duration};

use bb8::{Pool, RunError};
use bb8_redis::RedisConnectionManager;
use redis::{
    AsyncConnectionConfig, ProtocolVersion, RedisConnectionInfo, RedisError, TlsMode,
    aio::ConnectionManagerConfig, sentinel::SentinelNodeConnectionInfo,
};
use sentinel::{RedisSentinelConnectionManager, SentinelConfig};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    ServiceError, ServicesBuilder,
    services_builder::{IsUnset, SetCache, State},
};

pub use self::cluster::RedisClusterConnectionManager;

pub const REDIS_CONN_TIMEOUT: Duration = Duration::from_secs(2);

impl<S: State> ServicesBuilder<S> {
    pub async fn cache(
        self,
        config: &CacheConfig,
    ) -> Result<ServicesBuilder<SetCache<S>>, crate::ServiceError>
    where
        S::Cache: IsUnset,
    {
        Ok(self.cache_internal(RedisManager::new(config).await?))
    }
}

fn default_max_conns() -> u16 {
    100
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct CacheConfig {
    #[serde(rename = "dsn")]
    redis_dsn: Arc<str>,
    #[serde(default)]
    pooled: bool,
    #[serde(rename = "type")]
    kind: RedisVariant,
    #[serde(default = "default_max_conns")]
    #[serde(rename = "max-connections")]
    max_connections: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum RedisVariant {
    Clustered,
    NonClustered,
    Sentinel(SentinelConfig),
}

#[derive(Clone)]
pub enum RedisManager {
    Clustered(Pool<RedisClusterConnectionManager>),
    NonClustered(Pool<RedisConnectionManager>),
    Sentinel(Pool<RedisSentinelConnectionManager>),
    ClusteredUnpooled(redis::cluster_async::ClusterConnection),
    NonClusteredUnpooled(redis::aio::ConnectionManager),
    SentinelUnpooled(Arc<Mutex<redis::sentinel::SentinelClient>>),
}

impl RedisManager {
    pub async fn new(config: &CacheConfig) -> Result<Self, ServiceError> {
        if config.pooled {
            Self::new_pooled(
                config.redis_dsn.as_ref(),
                &config.kind,
                config.max_connections,
            )
            .await
        } else {
            Self::new_unpooled(config.redis_dsn.as_ref(), &config.kind).await
        }
    }
    async fn new_pooled(
        dsn: &str,
        variant: &RedisVariant,
        max_conns: u16,
    ) -> Result<Self, ServiceError> {
        match variant {
            RedisVariant::Clustered => {
                let mgr = RedisClusterConnectionManager::new(dsn)?;
                let pool = bb8::Pool::builder()
                    .max_size(max_conns.into())
                    .build(mgr)
                    .await?;
                Ok(RedisManager::Clustered(pool))
            }
            RedisVariant::NonClustered => {
                let mgr = RedisConnectionManager::new(dsn)?;
                let pool = bb8::Pool::builder()
                    .max_size(max_conns.into())
                    .build(mgr)
                    .await?;
                Ok(RedisManager::NonClustered(pool))
            }
            RedisVariant::Sentinel(cfg) => {
                let tls_mode = cfg.redis_tls_mode_secure.then_some(TlsMode::Secure);
                let protocol = if cfg.redis_use_resp3 {
                    ProtocolVersion::RESP3
                } else {
                    ProtocolVersion::default()
                };
                let mgr = RedisSentinelConnectionManager::new(
                    vec![dsn],
                    cfg.service_name.clone(),
                    Some(SentinelNodeConnectionInfo {
                        tls_mode,
                        redis_connection_info: Some(RedisConnectionInfo {
                            db: cfg.redis_db.unwrap_or(0),
                            username: cfg.redis_username.clone(),
                            password: cfg.redis_password.clone(),
                            protocol,
                        }),
                    }),
                )?;
                let pool = bb8::Pool::builder()
                    .max_size(max_conns.into())
                    .build(mgr)
                    .await?;
                Ok(RedisManager::Sentinel(pool))
            }
        }
    }

    async fn new_unpooled(dsn: &str, variant: &RedisVariant) -> Result<Self, ServiceError> {
        match variant {
            RedisVariant::Clustered => {
                let cli = redis::cluster::ClusterClient::builder(vec![dsn])
                    .retries(1)
                    .connection_timeout(REDIS_CONN_TIMEOUT)
                    .build()?;
                let con = cli.get_async_connection().await?;
                Ok(RedisManager::ClusteredUnpooled(con))
            }
            RedisVariant::NonClustered => {
                let cli = redis::Client::open(dsn)?;
                let con = redis::aio::ConnectionManager::new_with_config(
                    cli,
                    ConnectionManagerConfig::new()
                        .set_number_of_retries(1)
                        .set_connection_timeout(REDIS_CONN_TIMEOUT),
                )
                .await?;
                Ok(RedisManager::NonClusteredUnpooled(con))
            }
            RedisVariant::Sentinel(cfg) => {
                let tls_mode = cfg.redis_tls_mode_secure.then_some(TlsMode::Secure);
                let protocol = if cfg.redis_use_resp3 {
                    ProtocolVersion::RESP3
                } else {
                    ProtocolVersion::default()
                };
                let cli = redis::sentinel::SentinelClient::build(
                    vec![dsn],
                    cfg.service_name.clone(),
                    Some(SentinelNodeConnectionInfo {
                        tls_mode,
                        redis_connection_info: Some(RedisConnectionInfo {
                            db: cfg.redis_db.unwrap_or(0),
                            username: cfg.redis_username.clone(),
                            password: cfg.redis_password.clone(),
                            protocol,
                        }),
                    }),
                    redis::sentinel::SentinelServerType::Master,
                )?;

                Ok(RedisManager::SentinelUnpooled(Arc::new(Mutex::new(cli))))
            }
        }
    }

    pub async fn get(&self) -> Result<RedisConnection<'_>, RunError<RedisError>> {
        match self {
            Self::Clustered(pool) => Ok(RedisConnection::Clustered(pool.get().await?)),
            Self::NonClustered(pool) => Ok(RedisConnection::NonClustered(pool.get().await?)),
            Self::Sentinel(pool) => Ok(RedisConnection::SentinelPooled(pool.get().await?)),
            Self::ClusteredUnpooled(conn) => Ok(RedisConnection::ClusteredUnpooled(conn.clone())),
            Self::NonClusteredUnpooled(conn) => {
                Ok(RedisConnection::NonClusteredUnpooled(conn.clone()))
            }
            Self::SentinelUnpooled(conn) => {
                let mut conn = conn.lock().await;
                let con = conn
                    .get_async_connection_with_config(
                        &AsyncConnectionConfig::new().set_response_timeout(REDIS_CONN_TIMEOUT),
                    )
                    .await?;
                Ok(RedisConnection::SentinelUnpooled(con))
            }
        }
    }
}

pub enum RedisConnection<'a> {
    Clustered(bb8::PooledConnection<'a, RedisClusterConnectionManager>),
    NonClustered(bb8::PooledConnection<'a, RedisConnectionManager>),
    SentinelPooled(bb8::PooledConnection<'a, RedisSentinelConnectionManager>),
    ClusteredUnpooled(redis::cluster_async::ClusterConnection),
    NonClusteredUnpooled(redis::aio::ConnectionManager),
    SentinelUnpooled(redis::aio::MultiplexedConnection),
}

impl redis::aio::ConnectionLike for RedisConnection<'_> {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        match self {
            RedisConnection::Clustered(conn) => conn.req_packed_command(cmd),
            RedisConnection::NonClustered(conn) => conn.req_packed_command(cmd),
            RedisConnection::ClusteredUnpooled(conn) => conn.req_packed_command(cmd),
            RedisConnection::NonClusteredUnpooled(conn) => conn.req_packed_command(cmd),
            RedisConnection::SentinelPooled(conn) => conn.req_packed_command(cmd),
            RedisConnection::SentinelUnpooled(conn) => conn.req_packed_command(cmd),
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        match self {
            RedisConnection::Clustered(conn) => conn.req_packed_commands(cmd, offset, count),
            RedisConnection::NonClustered(conn) => conn.req_packed_commands(cmd, offset, count),
            RedisConnection::ClusteredUnpooled(conn) => {
                conn.req_packed_commands(cmd, offset, count)
            }
            RedisConnection::NonClusteredUnpooled(conn) => {
                conn.req_packed_commands(cmd, offset, count)
            }
            RedisConnection::SentinelPooled(conn) => conn.req_packed_commands(cmd, offset, count),
            RedisConnection::SentinelUnpooled(conn) => conn.req_packed_commands(cmd, offset, count),
        }
    }

    fn get_db(&self) -> i64 {
        match self {
            RedisConnection::Clustered(conn) => conn.get_db(),
            RedisConnection::NonClustered(conn) => conn.get_db(),
            RedisConnection::ClusteredUnpooled(conn) => conn.get_db(),
            RedisConnection::NonClusteredUnpooled(conn) => conn.get_db(),
            RedisConnection::SentinelPooled(conn) => conn.get_db(),
            RedisConnection::SentinelUnpooled(conn) => conn.get_db(),
        }
    }
}

#[cfg(test)]
mod tests {
    use redis::AsyncCommands;

    use crate::cache::CacheConfig;

    use super::RedisManager;

    // Ensure basic set/get works -- should test sharding as well:
    #[tokio::test]
    // run with `cargo test -- --ignored redis` only when redis is up and configured
    #[ignore]
    async fn test_set_read_random_keys() {
        let config = CacheConfig {
            redis_dsn: "redis://localhost:6379".into(),
            pooled: false,
            kind: crate::cache::RedisVariant::NonClustered,
            max_connections: 10,
        };
        let mgr = RedisManager::new(&config).await.unwrap();
        let mut conn = mgr.get().await.unwrap();

        for (val, key) in "abcdefghijklmnopqrstuvwxyz".chars().enumerate() {
            let key = key.to_string();
            let _: () = conn.set(key.clone(), val).await.unwrap();
            assert_eq!(conn.get::<_, usize>(&key).await.unwrap(), val);
        }
    }
}
