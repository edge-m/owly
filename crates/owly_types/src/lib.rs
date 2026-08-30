//! Shared domain types exchanged by Owly's source and cache crates.

mod document;
mod frontmatter;
mod link;
mod source;

pub use document::{DocumentIndex, IndexedDocument};
pub use frontmatter::{FrontmatterValue, IndexedFrontmatterValue};
pub use link::IndexedLink;
pub use source::IndexedSourceReference;
