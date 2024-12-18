//! Core
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]
/// Protobuf types
pub mod google;
/// Type conversion utils
pub mod utils;

/// ISO20022 messages
#[allow(missing_docs)]
pub mod iso20022;

/// Pseudonyms entities
#[allow(missing_docs)]
pub mod pseudonyms;
