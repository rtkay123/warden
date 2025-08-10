use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    pub cache_ttl: u64,
    #[serde(rename = "pseudonyms-endpoint")]
    pub pseudonyms_endpoint: std::sync::Arc<str>,
}
