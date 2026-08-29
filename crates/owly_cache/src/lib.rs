//! SQLite-backed derived index for Owly wikis.

mod model;
mod schema;
mod store;

pub use model::{Condition, Document, FrontmatterValue, Link, Operator, SearchQuery, SourceReference, TypedValue};
pub use store::CacheDb;

#[cfg(test)]
mod tests;
