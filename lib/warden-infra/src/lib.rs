//! Warden Infrastructure
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
)]

/// Cache
#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
#[cfg(feature = "cache")]
pub mod cache;

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
}
