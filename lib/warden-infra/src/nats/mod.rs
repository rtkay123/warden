use crate::{
    ServiceError, ServicesBuilder,
    configuration::nats::NatsConfig,
    services_builder::{IsUnset, State},
};

#[cfg(feature = "nats-jetstream")]
impl<S: State> ServicesBuilder<S> {
    /// create a Jetstream Context using the provided [NatsConfig]
    pub async fn with_nats_jetstream(
        self,
        config: &NatsConfig,
    ) -> Result<ServicesBuilder<crate::services_builder::SetJetstream<S>>, ServiceError>
    where
        S::Jetstream: IsUnset,
    {
        let hosts = config.hosts();
        let client = async_nats::connect(hosts).await?;

        Ok(self.jetstream(async_nats::jetstream::new(client)))
    }
}

#[cfg(feature = "nats-core")]
impl<S: State> ServicesBuilder<S> {
    /// create a NATS connection using the provided [NatsConfig]
    pub async fn with_nats(
        self,
        config: &NatsConfig,
    ) -> Result<ServicesBuilder<crate::services_builder::SetNats<S>>, ServiceError>
    where
        S::Nats: IsUnset,
    {
        let hosts = config.hosts();
        let client = async_nats::connect(hosts).await?;

        Ok(self.nats(client))
    }
}
