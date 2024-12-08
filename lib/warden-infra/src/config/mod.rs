use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// Config Data
pub struct Configuration {
    /// Environment
    pub environment: Environment,
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Database configuration
    pub database: crate::services::postgres::PgConfig,
    #[cfg(feature = "api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    ///
    pub port: u16,
    #[cfg(feature = "nats")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats")))]
    /// Nats config
    pub nats: crate::services::nats::NatsConfig,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
///
pub enum Environment {
    /// Dev
    Development,
    ///
    Production,
}
