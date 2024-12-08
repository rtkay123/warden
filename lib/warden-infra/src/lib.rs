//! Infrastructure
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]

#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
/// Tracing utilities
pub mod tracing;

/// Services
pub mod services;

/// Config
pub mod config;

use thiserror::Error;

#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "postgres", feature = "nats"),
    derive(bon::Builder),
    builder(state_mod(vis = "pub(crate)"))
)]
///
pub struct Services {
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Postgres connection pool handle
    pub postgres: sqlx::PgPool,
    #[cfg(feature = "nats-core")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-core")))]
    /// NATS connection handle
    pub nats: async_nats::Client,
    #[cfg(feature = "nats-jetstream")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nats-jetstream")))]
    /// NATS-Jetstream connection handle
    pub jetstream: async_nats::jetstream::Context,
}

#[derive(Error, Debug)]
///
pub enum ServiceError {
    #[error(transparent)]
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Database Error
    Postgres(#[from] sqlx::Error),
    #[cfg(feature = "nats")]
    #[error(transparent)]
    /// NATS error
    Nats(#[from] async_nats::error::Error<async_nats::ConnectErrorKind>),
}
