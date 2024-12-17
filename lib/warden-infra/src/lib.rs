//! Warden Infrastructure
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

/// Cache
#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
#[cfg(feature = "cache")]
pub mod cache;

/// NATS
#[cfg_attr(docsrs, doc(cfg(feature = "nats")))]
#[cfg(feature = "nats")]
pub mod nats;

/// Tracing
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
#[cfg(feature = "tracing")]
pub mod tracing;

/// Configuration for services
pub mod configuration;

/// Errors returned by services
#[derive(Error, Debug)]
pub enum ServiceError {
    #[cfg(feature = "cache")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
    #[error(transparent)]
    /// When creating the tracing layer
    Cache(#[from] redis::RedisError),
    #[error("bad config")]
    /// Config error
    Config,
    #[cfg(feature = "nats")]
    #[error(transparent)]
    /// NATS error
    Nats(#[from] async_nats::error::Error<async_nats::ConnectErrorKind>),
}

use thiserror::Error;
#[derive(Clone)]
#[cfg_attr(
    feature = "cache",
    derive(bon::Builder),
    builder(state_mod(vis = "pub(crate)"))
)]
///
pub struct Services {
    #[cfg(feature = "cache")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
    /// Cache connection handle
    pub cache: Option<cache::CacheService>,
    #[cfg(feature = "nats-core")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-core")))]
    /// NATS connection handle
    pub nats: Option<async_nats::Client>,
    #[cfg(feature = "nats-jetstream")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-jetstream")))]
    /// NATS-Jetstream connection handle
    pub jetstream: Option<async_nats::jetstream::Context>,
}
