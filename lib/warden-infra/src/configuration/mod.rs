#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
#[cfg(feature = "cache")]
/// Cache config
pub mod cache;

use serde::Deserialize;

use std::{fmt::Display, sync::Arc};

#[derive(Clone, Debug, Deserialize)]
/// Config Data
pub struct Configuration {
    /// General config
    pub application: App,
    #[cfg(feature = "cache")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
    /// Cache configuration
    pub cache: cache::CacheConfig,
    #[serde(skip)]
    /// Read from cargo
    pub metadata: Metadata,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Runtime env
pub enum Environment {
    /// Dev
    Development,
    ///
    Production,
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Environment::Development => "development",
            Environment::Production => "production",
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
/// App configuration
pub struct App {
    /// Env
    pub environment: Environment,
    #[cfg(feature = "api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    /// api port
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Default)]
/// App metadata
pub struct Metadata {
    #[serde(skip)]
    /// application name
    pub name: Arc<str>,
    #[serde(skip)]
    /// application version
    pub version: Arc<str>,
}
