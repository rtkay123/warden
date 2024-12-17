use std::sync::Arc;

use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone, Debug)]
/// Cache configuration
pub struct CacheConfig {
    /// Cache dsn
    pub dsn: Arc<[Url]>,
    /// Is this a cluster
    pub cluster: bool,
}
