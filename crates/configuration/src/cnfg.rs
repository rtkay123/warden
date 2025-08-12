use std::sync::Arc;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct LocalConfig {
    pub nats: JetstreamConfig,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct JetstreamConfig {
    pub stream: Arc<str>,
    pub max_messages: i64,
    pub subject: Arc<str>,
}
