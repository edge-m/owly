use owly_types::IndexedSourceReference;
use rusqlite::{Connection, Result, Row, Transaction, params};

pub fn delete_for_document(transaction: &Transaction<'_>, path: &str) -> Result<()> {
    transaction.execute("DELETE FROM source_references WHERE document_path = ?1", [path])?;
    Ok(())
}
pub fn insert(transaction: &Transaction<'_>, sources: &[IndexedSourceReference]) -> Result<()> {
    for source in sources {
        transaction.execute("INSERT INTO source_references(document_path, ordinal, source_path, sha256, origin) VALUES (?1, ?2, ?3, ?4, ?5)", params![source.document_path, source.ordinal, source.source_path, source.sha256, source.origin])?;
    }
    Ok(())
}
pub fn for_document(connection: &Connection, path: &str) -> Result<Vec<IndexedSourceReference>> {
    let mut statement = connection.prepare("SELECT document_path, ordinal, source_path, sha256, origin FROM source_references WHERE document_path = ?1 ORDER BY ordinal")?;
    statement.query_map([path], from_row)?.collect()
}
fn from_row(row: &Row<'_>) -> Result<IndexedSourceReference> {
    Ok(IndexedSourceReference {
        document_path: row.get(0)?,
        ordinal: row.get(1)?,
        source_path: row.get(2)?,
        sha256: row.get(3)?,
        origin: row.get(4)?,
    })
}
