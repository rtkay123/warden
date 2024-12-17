use std::collections::HashMap;
use std::sync::Arc;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{ServiceError, ServicesBuilder};

#[derive(Deserialize, Debug, Clone, Hash, Eq, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Database {
    TransactionHistory,
}

#[derive(Debug, Deserialize, Clone)]
/// Postgres configuration
pub struct PgConfig {
    pool_size: u32,
    port: u16,
    name: Arc<str>,
    host: Arc<str>,
    user: Arc<str>,
    password: SecretString,
}

impl PgConfig {
    /// Getter for size
    pub fn pool_size(&self) -> u32 {
        self.pool_size
    }

    /// Getter for port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Getter for name
    pub fn name(&self) -> &str {
        &self.name.as_ref()
    }

    /// Getter for host
    pub fn host(&self) -> &str {
        &self.host.as_ref()
    }

    /// Getter for username
    pub fn username(&self) -> &str {
        &self.user.as_ref()
    }

    /// Getter for password (you may want to return a reference or handle it differently)
    pub fn password(&self) -> &SecretString {
        &self.password
    }

    pub(crate) fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        )
    }
}
use crate::services_builder::{IsUnset, SetPostgres, State};

impl<S: State> ServicesBuilder<S> {
    /// create a [sqlx::PgPool] using the provided [PgConfig]
    pub async fn with_postgres(
        self,
        config: &HashMap<Database, PgConfig>,
    ) -> Result<ServicesBuilder<SetPostgres<S>>, ServiceError>
    where
        S::Postgres: IsUnset,
    {
        let mut items = HashMap::with_capacity(config.len());

        for (db, config) in config {
            let db_pool = sqlx::postgres::PgPoolOptions::new()
                // The default connection limit for a Postgres server is 100 connections, with 3 reserved for superusers.
                //
                // If you're deploying your application with multiple replicas, then the total
                // across all replicas should not exceed the Postgres connection limit.
                .max_connections(config.pool_size())
                .connect(&config.connection_string())
                .await?;
            items.insert(*db, db_pool);
        }
        Ok(self.postgres(items))
    }
}
