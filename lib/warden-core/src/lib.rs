//! Core
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    // missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]
pub mod config;
pub mod google;
pub mod utils;

pub mod iso20022;
