#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub mod tracing;

#[cfg(feature = "cache")]
#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
pub mod cache;

#[cfg(feature = "cache")]
pub use redis;

#[cfg(feature = "postgres")]
pub use sqlx;

#[cfg(any(feature = "nats-core", feature = "nats-jetstream"))]
pub use async_nats;

#[cfg(feature = "opentelemetry")]
mod otel {
    pub use opentelemetry;
    pub use opentelemetry_http;
    pub use opentelemetry_otlp;
    pub use opentelemetry_sdk;
    pub use opentelemetry_semantic_conventions;
    pub use tracing_opentelemetry;
}

#[cfg(feature = "opentelemetry")]
pub use otel::*;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;

#[cfg(any(feature = "nats-core", feature = "nats-jetstream"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nats-core", feature = "nats-jetstream")))
)]
pub mod nats;

mod config;
pub use config::*;

#[derive(Clone, bon::Builder)]
pub struct Services {
    #[cfg(feature = "postgres")]
    #[builder(setters(vis = "", name = pg_internal))]
    pub postgres: Option<sqlx::PgPool>,
    #[cfg(feature = "cache")]
    #[builder(setters(vis = "", name = cache_internal))]
    pub cache: Option<cache::RedisManager>,
    #[cfg(feature = "nats-core")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-core")))]
    #[builder(setters(vis = "", name = nats_internal))]
    /// NATS connection handle
    pub nats: Option<async_nats::Client>,
    #[cfg(feature = "nats-jetstream")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-jetstream")))]
    #[builder(setters(vis = "", name = jetstream_internal))]
    /// NATS-Jetstream connection handle
    pub jetstream: Option<async_nats::jetstream::Context>,
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("service was not initialised")]
    NotInitialised,
    #[error("unknown data store error")]
    Unknown,
    #[error("invalid config `{0}`")]
    Configuration(String),
    #[cfg(feature = "postgres")]
    #[error(transparent)]
    /// Postgres error
    Postgres(#[from] sqlx::Error),
    #[cfg(feature = "cache")]
    #[error(transparent)]
    /// Redis error
    Cache(#[from] redis::RedisError),
    #[cfg(feature = "opentelemetry")]
    #[error(transparent)]
    /// When creating the tracing layer
    Opentelemetry(#[from] opentelemetry_sdk::trace::TraceError),
    #[cfg(any(feature = "nats-core", feature = "nats-jetstream"))]
    #[error(transparent)]
    /// NATS error
    Nats(#[from] async_nats::error::Error<async_nats::ConnectErrorKind>),
    #[cfg(feature = "tracing-loki")]
    #[error(transparent)]
    /// When creating the tracing layer
    Loki(#[from] tracing_loki::Error),
}
