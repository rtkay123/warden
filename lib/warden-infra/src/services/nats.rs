#![allow(unused_imports)]

use serde::Deserialize;
use std::sync::Arc;

/// Represents the configuration for connecting to NATS servers.
#[derive(Debug, Deserialize, Clone)]
pub struct NatsConfig {
    /// List of host addresses for the NATS servers.
    hosts: Arc<[String]>,

    /// Jetstream configuration. This field is included only if the `nats-jetstream` feature is enabled.
    /// It contains the configuration for Jetstream streams.
    #[serde(default)]
    #[cfg(feature = "nats-jetstream")]
    pub jetstream: Arc<[StreamConfig]>,
}

#[cfg(feature = "nats-jetstream")]
/// Represents the configuration for a Jetstream stream in NATS.
#[derive(Debug, Deserialize, Default)]
pub struct StreamConfig {
    /// The name of the Jetstream stream.
    pub name: Arc<str>,

    /// List of subjects associated with the stream.
    pub subjects: Arc<[String]>,

    /// Maximum number of messages allowed in the stream. Default is 0 (no limit).
    #[serde(default)]
    pub max_msgs: i64,

    /// Maximum number of bytes allowed in the stream. Default is 0 (no limit).
    #[serde(default)]
    pub max_bytes: i64,

    /// List of consumers associated with the stream.
    pub consumers: Arc<[ConsumerConfig]>,
}

#[cfg(feature = "nats-jetstream")]
/// Represents the configuration for a consumer in NATS Jetstream.
#[derive(Debug, Deserialize, Default)]
pub struct ConsumerConfig {
    /// The name of the consumer.
    pub name: Arc<str>,

    /// The durable name for the consumer. This is optional.
    pub durable: Option<Arc<str>>,

    /// The subject to deliver messages to. This is optional.
    pub deliver_subject: Option<Arc<str>>,
}

impl NatsConfig {
    /// Returns the list of host addresses for the NATS servers.
    pub fn hosts(&self) -> &[String] {
        self.hosts.as_ref()
    }
}

#[cfg(feature = "nats")]
use crate::{
    ServiceError, ServicesBuilder,
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

        let jetstream = async_nats::jetstream::new(client);

        Ok(self.jetstream(jetstream))
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
