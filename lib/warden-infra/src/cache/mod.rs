use log::{error, trace};
use redis::{aio::MultiplexedConnection, cluster::ClusterClient, cluster_async::ClusterConnection};

use crate::{
    ServiceError, ServicesBuilder,
    configuration::cache::CacheConfig,
    services_builder::{IsUnset, State},
};

#[derive(Clone)]
/// Cache Service
pub enum CacheService {
    /// Clustered connection
    Clustered(ClusterConnection),
    /// NonClustered connection
    NonClustered(MultiplexedConnection),
}

impl<S: State> ServicesBuilder<S> {
    /// create a Jetstream Context using the provided [NatsConfig]
    pub async fn with_cache(
        self,
        config: &CacheConfig,
    ) -> Result<ServicesBuilder<crate::services_builder::SetCache<S>>, ServiceError>
    where
        S::Cache: IsUnset,
    {
        let nodes = config.dsn.to_vec();

        match config.cluster {
            true => {
                trace!("setting up a clustered cache connection");
                let client = ClusterClient::new(nodes)?;
                let connection = client.get_async_connection().await?;

                Ok(self.cache(CacheService::Clustered(connection)))
            }
            false => {
                if let Some(node) = nodes.first() {
                    trace!("setting up cache connection");
                    let client = redis::Client::open(node.to_string())?;
                    let con = client.get_multiplexed_async_connection().await?;
                    Ok(self.cache(CacheService::NonClustered(con)))
                } else {
                    error!("a node is required for caching");
                    Err(ServiceError::Config)
                }
            }
        }
    }
}
