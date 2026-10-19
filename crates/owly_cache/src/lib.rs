//! SQLite-backed derived index for Owly wikis.
//!
//! [`CacheDb`] owns the rebuildable database used for indexed document
//! metadata, frontmatter queries, links, and tracked source references.

mod query;
mod schema;
mod store;

pub use owly_types::{
    FrontmatterValue, IndexedDocument, IndexedFrontmatterValue, IndexedLink, IndexedSourceReference,
};
pub use query::{Condition, Operator, SearchQuery};
pub use store::CacheDb;

#[cfg(test)]
mod tests;
