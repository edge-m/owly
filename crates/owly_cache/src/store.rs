use std::path::Path;

use owly_types::{
    DocumentIndex, FrontmatterValue, IndexedDocument, IndexedFrontmatterValue, IndexedLink, IndexedSourceReference,
};
use rusqlite::{Connection, Result};

mod documents;
mod frontmatter_values;
mod links;
mod source_references;

/// Connection and CRUD operations for the derived cache index.
pub struct CacheDb {
    connection: Connection,
}

impl CacheDb {
    /// Open or create a cache database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }
    /// Create an in-memory cache database.
    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }
    fn from_connection(connection: Connection) -> Result<Self> {
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        let database = Self { connection };
        database.initialize()?;
        Ok(database)
    }
    pub fn schema_version(&self) -> Result<i64> {
        self.connection.query_row("PRAGMA user_version", [], |row| row.get(0))
    }
    pub fn initialize(&self) -> Result<()> {
        crate::schema::initialize(&self.connection)
    }
    pub fn upsert_document(&self, document: &IndexedDocument) -> Result<()> {
        documents::upsert(&self.connection, document)
    }
    /// Replace every derived row belonging to one parsed source document.
    pub fn replace_index(&mut self, index: &DocumentIndex) -> Result<()> {
        self.replace_document_index(&index.document, &index.frontmatter, &index.links, &index.sources)
    }
    pub fn document(&self, path: &str) -> Result<Option<IndexedDocument>> {
        documents::get(&self.connection, path)
    }
    pub fn delete_document(&self, path: &str) -> Result<bool> {
        documents::delete(&self.connection, path)
    }
    /// Return all indexed document paths in deterministic order.
    pub fn document_paths(&self) -> Result<Vec<String>> {
        documents::paths(&self.connection)
    }
    /// Remove all derived rows so the index can be rebuilt from the source of truth.
    pub fn clear(&self) -> Result<()> {
        documents::clear(&self.connection)
    }
    pub fn replace_document_index(
        &mut self, document: &IndexedDocument, values: &[IndexedFrontmatterValue], links: &[IndexedLink],
        sources: &[IndexedSourceReference],
    ) -> Result<()> {
        let transaction = self.connection.transaction()?;
        documents::upsert_in(&transaction, document)?;
        frontmatter_values::delete_for_document(&transaction, &document.path)?;
        links::delete_for_document(&transaction, &document.path)?;
        source_references::delete_for_document(&transaction, &document.path)?;
        frontmatter_values::insert(&transaction, values)?;
        links::insert(&transaction, links)?;
        source_references::insert(&transaction, sources)?;
        transaction.commit()
    }
    pub fn frontmatter_values(&self, path: &str) -> Result<Vec<IndexedFrontmatterValue>> {
        frontmatter_values::for_document(&self.connection, path)
    }
    pub fn find_documents(&self, field_path: &str, value: &FrontmatterValue) -> Result<Vec<String>> {
        self.search(&crate::query::SearchQuery::new().and(field_path, crate::query::Operator::Eq, value.clone()))
    }
    /// Find documents matching every condition in an AND query.
    pub fn search(&self, query: &crate::query::SearchQuery) -> Result<Vec<String>> {
        frontmatter_values::search(&self.connection, query)
    }
    pub fn links(&self, source_path: &str) -> Result<Vec<IndexedLink>> {
        links::for_source(&self.connection, source_path)
    }
    pub fn incoming_links(&self, target: &str) -> Result<Vec<IndexedLink>> {
        links::for_target(&self.connection, target)
    }
    pub fn source_references(&self, document_path: &str) -> Result<Vec<IndexedSourceReference>> {
        source_references::for_document(&self.connection, document_path)
    }
}
