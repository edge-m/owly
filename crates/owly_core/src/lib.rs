//! Core validation and wiki-model APIs used by [`owly`](https://crates.io/crates/owly).
//!
//! This crate contains configuration handling, diagnostics, document and link
//! models, workspace scanning, graph construction, and wiki validation. It is
//! the reusable library layer behind the Owly command-line application.

pub mod config;
pub mod diagnostic;
pub mod document;
pub mod graph;
pub mod link_resolver;
pub mod source;
pub mod validation;
pub mod workspace;
