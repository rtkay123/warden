//! Core
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]
/// Protobuf types
pub mod google;

/// ISO20022 messages
#[allow(missing_docs)]
pub mod iso20022;
