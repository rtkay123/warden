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
