//! warden-core
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]

/// Google well known types
#[allow(missing_docs)]
#[cfg(feature = "iso20022")]
pub mod google;

/// ISO20022 messages
#[allow(missing_docs)]
#[cfg(feature = "iso20022")]
pub mod iso20022;
