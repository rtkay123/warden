use std::sync::Arc;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct LocalConfig {
    pub nats: NatsConfig,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NatsConfig {
    #[serde(rename = "stream-name")]
    pub name: Arc<str>,
    pub subjects: Arc<[String]>,
    pub durable_name: Arc<str>,
}
