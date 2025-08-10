use std::sync::Arc;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
/// Nats configuration
pub struct NatsConfig {
    /// Hosts dsn
    #[serde(default = "nats")]
    pub hosts: Arc<[String]>,
}

pub(crate) fn nats() -> Arc<[String]> {
    let hosts = vec!["nats://localhost:4222".to_string()];
    hosts.into()
}

impl NatsConfig {
    fn hosts(&self) -> Vec<String> {
        self.hosts.iter().map(ToString::to_string).collect()
    }
}

use crate::{
    ServiceError, ServicesBuilder,
    services_builder::{IsUnset, State},
};

#[cfg(feature = "nats-jetstream")]
impl<S: State> ServicesBuilder<S> {
    /// create a Jetstream Context using the provided [NatsConfig]
    pub async fn nats_jetstream(
        self,
        config: &NatsConfig,
    ) -> Result<ServicesBuilder<crate::services_builder::SetJetstream<S>>, ServiceError>
    where
        S::Jetstream: IsUnset,
    {
        let hosts = config.hosts();
        let client = async_nats::connect(hosts).await?;

        Ok(self.jetstream_internal(async_nats::jetstream::new(client)))
    }
}

#[cfg(feature = "nats-core")]
impl<S: State> ServicesBuilder<S> {
    /// create a NATS connection using the provided [NatsConfig]
    pub async fn nats(
        self,
        config: &NatsConfig,
    ) -> Result<ServicesBuilder<crate::services_builder::SetNats<S>>, ServiceError>
    where
        S::Nats: IsUnset,
    {
        let hosts = config.hosts();
        let client = async_nats::connect(hosts).await?;

        Ok(self.nats_internal(client))
    }
}
