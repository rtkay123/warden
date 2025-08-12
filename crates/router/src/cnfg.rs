use std::sync::Arc;

use serde::Deserialize;

pub const CACHE_KEY: i32 = 0;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    pub config_endpoint: Arc<str>,
    pub nats: Nats,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Nats {
    #[serde(rename = "stream-name")]
    pub name: Arc<str>,
    pub subjects: Arc<[String]>,
    pub destination_prefix: Arc<str>,
    pub max_messages: i64,
    pub durable_name: Arc<str>,
    pub config: ConfigNats,
}

#[derive(Deserialize, Clone)]
pub struct ConfigNats {
    #[serde(rename = "stream")]
    pub stream: Arc<str>,
    #[serde(rename = "reload-subject")]
    pub reload_subject: Arc<str>,
}
