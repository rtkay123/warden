use std::sync::Arc;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    pub config_endpoint: Arc<str>,
    pub nats: Nats,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Nats {
    pub name: Arc<str>,
    pub subjects: Arc<[String]>,
    pub durable_name: Arc<str>,
    pub destination_prefix: Arc<str>,
    pub config: ConfigNats,
}

#[derive(Deserialize, Clone)]
pub struct ConfigNats {
    pub stream: Arc<str>,
    pub reload_subject: Arc<str>,
}
