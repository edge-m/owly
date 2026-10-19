//! Shared domain types exchanged by Owly's source and cache crates.
//!
//! The types describe indexed documents and their metadata without depending
//! on filesystem or database implementations.

mod document;
mod frontmatter;
mod link;
mod source;

pub use document::{DocumentIndex, IndexedDocument};
pub use frontmatter::{FrontmatterValue, IndexedFrontmatterValue};
pub use link::IndexedLink;
pub use source::IndexedSourceReference;
