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
    any(feature = "postgres"),
    derive(bon::Builder),
    builder(state_mod(vis = "pub(crate)"))
)]
///
pub struct Services {
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Postgres connection pool handle
    pub postgres: sqlx::PgPool,
}

#[derive(Error, Debug)]
///
pub enum ServiceError {
    #[error(transparent)]
    #[cfg(feature = "postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
    /// Database Error
    Postgres(#[from] sqlx::Error),
}
