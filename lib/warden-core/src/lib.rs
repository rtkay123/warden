//! warden-core
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]

/// Type file descriptor
#[cfg(any(feature = "message", feature = "pseudonyms", feature = "configuration"))]
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("warden_descriptor");

/// Google well known types
#[allow(missing_docs)]
#[cfg(any(feature = "message", feature = "pseudonyms", feature = "configuration"))]
pub mod google;

/// ISO20022 messages
#[allow(missing_docs)]
#[cfg(feature = "message")]
pub mod iso20022;

/// Message in transit
#[allow(missing_docs)]
#[allow(clippy::large_enum_variant)]
#[cfg(feature = "message")]
pub mod message;

/// Pseudonyms
#[allow(missing_docs)]
#[cfg(feature = "pseudonyms")]
pub mod pseudonyms;

#[allow(missing_docs)]
#[cfg(feature = "configuration")]
pub mod configuration;
