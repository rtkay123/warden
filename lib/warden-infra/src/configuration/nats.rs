#![allow(unused_imports)]

use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use url::Url;

/// Represents the configuration for connecting to NATS servers.
#[derive(Debug, Deserialize, Clone)]
pub struct NatsConfig {
    /// List of host addresses for the NATS servers.
    hosts: Arc<[Url]>,
    /// Jetstream configuration. This field is included only if the `nats-jetstream` feature is enabled.
    /// It contains the configuration for Jetstream streams.
    #[serde(default)]
    #[cfg(feature = "nats-jetstream")]
    pub jetstream: Arc<[StreamConfig]>,
    #[serde(default)]
    /// Subjects to publish to
    pub pub_subjects: HashMap<String, String>,
}

#[cfg(feature = "nats-jetstream")]
/// Represents the configuration for a Jetstream stream in NATS.
#[derive(Debug, Deserialize, Default)]
pub struct StreamConfig {
    /// The name of the Jetstream stream.
    pub name: Arc<str>,

    /// List of subjects associated with the stream.
    pub subjects: Option<Arc<[String]>>,

    /// Maximum number of messages allowed in the stream. Default is 0 (no limit).
    #[serde(default)]
    pub max_msgs: i64,

    /// Maximum number of bytes allowed in the stream. Default is 0 (no limit).
    #[serde(default)]
    pub max_bytes: i64,

    /// List of consumers associated with the stream.
    #[serde(default)]
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
    pub fn hosts(&self) -> &[Url] {
        self.hosts.as_ref()
    }
}
