#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
#[cfg(feature = "cache")]
/// Cache config
pub mod cache;

#[cfg_attr(docsrs, doc(cfg(feature = "nats")))]
#[cfg(feature = "nats")]
/// Cache config
pub mod nats;

use serde::Deserialize;

use std::{collections::HashMap, fmt::Display, sync::Arc};

#[derive(Clone, Debug, Deserialize)]
/// Config Data
pub struct Configuration {
    /// General config
    pub application: App,
    #[cfg(feature = "cache")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
    /// Cache configuration
    pub cache: cache::CacheConfig,
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Databases
    pub database: HashMap<crate::postgres::Database, crate::postgres::PgConfig>,
    #[cfg(feature = "nats")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats")))]
    /// Cache configuration
    pub nats: nats::NatsConfig,
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
    #[cfg(feature = "tracing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
    /// Log level
    #[serde(default)]
    pub log_level: LogLevel,
}

#[derive(Copy, Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[cfg(feature = "tracing")]
/// Log Level
pub enum LogLevel {
    /// trace
    Trace,
    /// debug
    Debug,
    /// info
    #[default]
    Info,
    /// warn
    Warn,
    /// error
    Error,
}

/// Log Level
#[cfg(feature = "tracing")]
impl From<LogLevel> for tracing_subscriber::EnvFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
        .into()
    }
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
