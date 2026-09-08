use owly_types::IndexedLink;
use rusqlite::{Connection, Result, Row, Transaction, params};

pub fn delete_for_document(transaction: &Transaction<'_>, path: &str) -> Result<()> {
    transaction.execute("DELETE FROM links WHERE source_path = ?1", [path])?;
    Ok(())
}
pub fn insert(transaction: &Transaction<'_>, links: &[IndexedLink]) -> Result<()> {
    for link in links {
        transaction.execute("INSERT INTO links(source_path, raw_target, resolved_target, target_kind, occurrence_count) VALUES (?1, ?2, ?3, ?4, ?5)", params![link.source_path, link.raw_target, link.resolved_target, link.target_kind, link.occurrence_count])?;
    }
    Ok(())
}
pub fn for_source(connection: &Connection, path: &str) -> Result<Vec<IndexedLink>> {
    query(connection, "source_path", path, "resolved_target, raw_target, target_kind")
}
pub fn for_target(connection: &Connection, target: &str) -> Result<Vec<IndexedLink>> {
    query(connection, "resolved_target", target, "source_path, raw_target, target_kind")
}
fn query(connection: &Connection, column: &str, value: &str, order: &str) -> Result<Vec<IndexedLink>> {
    let mut statement = connection.prepare(&format!("SELECT source_path, raw_target, resolved_target, target_kind, occurrence_count FROM links WHERE {column} = ?1 ORDER BY {order}"))?;
    statement.query_map([value], from_row)?.collect()
}
fn from_row(row: &Row<'_>) -> Result<IndexedLink> {
    Ok(IndexedLink {
        source_path: row.get(0)?,
        raw_target: row.get(1)?,
        resolved_target: row.get(2)?,
        target_kind: row.get(3)?,
        occurrence_count: row.get(4)?,
    })
}
