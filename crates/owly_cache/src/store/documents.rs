use owly_types::IndexedDocument;
use rusqlite::{Connection, OptionalExtension, Result, Row, Transaction, params};

pub fn upsert(connection: &Connection, document: &IndexedDocument) -> Result<()> {
    connection.execute(
        "INSERT INTO documents(path, frontmatter_hash, links_hash) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO UPDATE SET frontmatter_hash = excluded.frontmatter_hash,
         links_hash = excluded.links_hash",
        params![document.path, document.frontmatter_hash, document.links_hash],
    )?;
    Ok(())
}
pub fn upsert_in(transaction: &Transaction<'_>, document: &IndexedDocument) -> Result<()> {
    transaction.execute(
        "INSERT INTO documents(path, frontmatter_hash, links_hash) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO UPDATE SET frontmatter_hash = excluded.frontmatter_hash,
         links_hash = excluded.links_hash",
        params![document.path, document.frontmatter_hash, document.links_hash],
    )?;
    Ok(())
}
pub fn get(connection: &Connection, path: &str) -> Result<Option<IndexedDocument>> {
    connection
        .query_row("SELECT path, frontmatter_hash, links_hash FROM documents WHERE path = ?1", [path], from_row)
        .optional()
}
pub fn delete(connection: &Connection, path: &str) -> Result<bool> {
    Ok(connection.execute("DELETE FROM documents WHERE path = ?1", [path])? > 0)
}
pub fn paths(connection: &Connection) -> Result<Vec<String>> {
    let mut statement = connection.prepare("SELECT path FROM documents ORDER BY path")?;
    statement.query_map([], |row| row.get(0))?.collect()
}
pub fn clear(connection: &Connection) -> Result<()> {
    connection.execute("DELETE FROM documents", [])?;
    Ok(())
}
fn from_row(row: &Row<'_>) -> Result<IndexedDocument> {
    Ok(IndexedDocument { path: row.get(0)?, frontmatter_hash: row.get(1)?, links_hash: row.get(2)? })
}
