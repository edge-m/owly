use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Result, Row, Transaction, params, params_from_iter, types::Value};

use crate::model::{Condition, Document, FrontmatterValue, Link, Operator, SearchQuery, SourceReference, TypedValue};

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

    pub fn upsert_document(&self, document: &Document) -> Result<()> {
        self.connection.execute(
            "INSERT INTO documents(path, content_hash, frontmatter_hash, links_hash)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET
                 content_hash = excluded.content_hash,
                 frontmatter_hash = excluded.frontmatter_hash,
                 links_hash = excluded.links_hash",
            params![document.path, document.content_hash, document.frontmatter_hash, document.links_hash],
        )?;
        Ok(())
    }

    pub fn document(&self, path: &str) -> Result<Option<Document>> {
        self.connection
            .query_row(
                "SELECT path, content_hash, frontmatter_hash, links_hash FROM documents WHERE path = ?1",
                [path],
                document_from_row,
            )
            .optional()
    }

    pub fn delete_document(&self, path: &str) -> Result<bool> {
        Ok(self.connection.execute("DELETE FROM documents WHERE path = ?1", [path])? > 0)
    }

    pub fn replace_document_index(
        &mut self, document: &Document, values: &[FrontmatterValue], links: &[Link], sources: &[SourceReference],
    ) -> Result<()> {
        let transaction = self.connection.transaction()?;
        upsert_document_in(&transaction, document)?;
        transaction.execute("DELETE FROM frontmatter_values WHERE document_path = ?1", [&document.path])?;
        transaction.execute("DELETE FROM links WHERE source_path = ?1", [&document.path])?;
        transaction.execute("DELETE FROM source_references WHERE document_path = ?1", [&document.path])?;
        for value in values {
            transaction.execute(
                "INSERT INTO frontmatter_values(document_path, field_path, value_type, value_text, value_number,
                    value_boolean, value_date, value_datetime, ordinal)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    value.document_path,
                    value.field_path,
                    value.value_type,
                    value.value_text,
                    value.value_number,
                    value.value_boolean.map(i64::from),
                    value.value_date,
                    value.value_datetime,
                    value.ordinal,
                ],
            )?;
        }
        for link in links {
            transaction.execute(
                "INSERT INTO links(source_path, raw_target, resolved_target, target_kind, occurrence_count)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    link.source_path,
                    link.raw_target,
                    link.resolved_target,
                    link.target_kind,
                    link.occurrence_count
                ],
            )?;
        }
        for source in sources {
            transaction.execute(
                "INSERT INTO source_references(document_path, ordinal, source_path, sha256, origin)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![source.document_path, source.ordinal, source.source_path, source.sha256, source.origin],
            )?;
        }
        transaction.commit()
    }

    pub fn frontmatter_values(&self, path: &str) -> Result<Vec<FrontmatterValue>> {
        let mut statement = self.connection.prepare(
            "SELECT document_path, field_path, value_type, value_text, value_number, value_boolean,
                    value_date, value_datetime, ordinal
             FROM frontmatter_values WHERE document_path = ?1 ORDER BY field_path, ordinal",
        )?;
        statement.query_map([path], frontmatter_value_from_row)?.collect()
    }

    pub fn find_documents(&self, field_path: &str, value: &TypedValue) -> Result<Vec<String>> {
        self.search(&SearchQuery::new().and(field_path, Operator::Eq, value.clone()))
    }

    /// Find documents matching every condition in an AND query.
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<String>> {
        if query.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("search query must contain a condition".to_string()));
        }
        let mut sql = String::from("SELECT d.path FROM documents AS d WHERE ");
        let mut values = Vec::new();
        for (index, condition) in query.conditions.iter().enumerate() {
            if index > 0 {
                sql.push_str(" AND ");
            }
            let compiled = CompiledCondition::new(condition)?;
            sql.push_str(&format!(
                "EXISTS (SELECT 1 FROM frontmatter_values AS v{index}
                 WHERE v{index}.document_path = d.path
                   AND v{index}.field_path = ?
                   AND v{index}.value_type = ?
                   AND v{index}.{column} {operator} {comparison})",
                column = compiled.column,
                operator = compiled.operator,
                comparison = compiled.comparison,
            ));
            values.push(Value::Text(condition.field_path.clone()));
            values.push(Value::Text(compiled.value_type.to_string()));
            if let Some(value) = compiled.value {
                values.push(value);
            }
        }
        sql.push_str(" ORDER BY d.path");
        let mut statement = self.connection.prepare(&sql)?;
        statement.query_map(params_from_iter(values.iter()), |row| row.get(0))?.collect()
    }

    pub fn links(&self, source_path: &str) -> Result<Vec<Link>> {
        let mut statement = self.connection.prepare(
            "SELECT source_path, raw_target, resolved_target, target_kind, occurrence_count
             FROM links WHERE source_path = ?1 ORDER BY resolved_target, raw_target, target_kind",
        )?;
        statement.query_map([source_path], link_from_row)?.collect()
    }

    pub fn incoming_links(&self, target: &str) -> Result<Vec<Link>> {
        let mut statement = self.connection.prepare(
            "SELECT source_path, raw_target, resolved_target, target_kind, occurrence_count
             FROM links WHERE resolved_target = ?1 ORDER BY source_path, raw_target, target_kind",
        )?;
        statement.query_map([target], link_from_row)?.collect()
    }

    pub fn source_references(&self, document_path: &str) -> Result<Vec<SourceReference>> {
        let mut statement = self.connection.prepare(
            "SELECT document_path, ordinal, source_path, sha256, origin
             FROM source_references WHERE document_path = ?1 ORDER BY ordinal",
        )?;
        statement.query_map([document_path], source_reference_from_row)?.collect()
    }
}

struct CompiledCondition {
    value_type: &'static str,
    column: &'static str,
    operator: &'static str,
    comparison: &'static str,
    value: Option<Value>,
}

impl CompiledCondition {
    fn new(condition: &Condition) -> Result<Self> {
        let (value_type, column, value) = match &condition.value {
            TypedValue::Text(value) => ("string", "value_text", Some(Value::Text(value.clone()))),
            TypedValue::Number(value) => ("number", "value_number", Some(Value::Real(*value))),
            TypedValue::Boolean(value) => ("boolean", "value_boolean", Some(Value::Integer(i64::from(*value)))),
            TypedValue::Date(value) => ("date", "value_date", Some(Value::Text(value.clone()))),
            TypedValue::Datetime(value) => ("datetime", "value_datetime", Some(Value::Text(value.clone()))),
            TypedValue::Null => ("null", "value_text", None),
        };
        let (operator, comparison) = match (&condition.operator, &condition.value) {
            (Operator::Eq, TypedValue::Null) => ("IS", "NULL"),
            (Operator::Ne, TypedValue::Null) => ("IS NOT", "NULL"),
            (Operator::Eq, _) => ("=", "?"),
            (Operator::Ne, _) => ("<>", "?"),
            (Operator::Lt, _) => ("<", "?"),
            (Operator::Lte, _) => ("<=", "?"),
            (Operator::Gt, _) => (">", "?"),
            (Operator::Gte, _) => (">=", "?"),
        };
        if matches!(condition.value, TypedValue::Null) && !matches!(condition.operator, Operator::Eq | Operator::Ne) {
            return Err(rusqlite::Error::InvalidParameterName("null only supports Eq or Ne".to_string()));
        }
        Ok(Self { value_type, column, operator, comparison, value })
    }
}

fn upsert_document_in(transaction: &Transaction<'_>, document: &Document) -> Result<()> {
    transaction.execute(
        "INSERT INTO documents(path, content_hash, frontmatter_hash, links_hash)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET content_hash = excluded.content_hash,
             frontmatter_hash = excluded.frontmatter_hash, links_hash = excluded.links_hash",
        params![document.path, document.content_hash, document.frontmatter_hash, document.links_hash],
    )?;
    Ok(())
}

fn document_from_row(row: &Row<'_>) -> Result<Document> {
    Ok(Document {
        path: row.get(0)?,
        content_hash: row.get(1)?,
        frontmatter_hash: row.get(2)?,
        links_hash: row.get(3)?,
    })
}

fn frontmatter_value_from_row(row: &Row<'_>) -> Result<FrontmatterValue> {
    Ok(FrontmatterValue {
        document_path: row.get(0)?,
        field_path: row.get(1)?,
        value_type: row.get(2)?,
        value_text: row.get(3)?,
        value_number: row.get(4)?,
        value_boolean: row.get::<_, Option<i64>>(5)?.map(|value| value != 0),
        value_date: row.get(6)?,
        value_datetime: row.get(7)?,
        ordinal: row.get(8)?,
    })
}

fn link_from_row(row: &Row<'_>) -> Result<Link> {
    Ok(Link {
        source_path: row.get(0)?,
        raw_target: row.get(1)?,
        resolved_target: row.get(2)?,
        target_kind: row.get(3)?,
        occurrence_count: row.get(4)?,
    })
}

fn source_reference_from_row(row: &Row<'_>) -> Result<SourceReference> {
    Ok(SourceReference {
        document_path: row.get(0)?,
        ordinal: row.get(1)?,
        source_path: row.get(2)?,
        sha256: row.get(3)?,
        origin: row.get(4)?,
    })
}
