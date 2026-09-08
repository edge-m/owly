use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: i64 = 1;

pub fn initialize(connection: &Connection) -> Result<()> {
    connection.execute_batch(&format!(
        "PRAGMA user_version = {SCHEMA_VERSION};

        CREATE TABLE IF NOT EXISTS documents (
            path TEXT PRIMARY KEY NOT NULL,
            frontmatter_hash TEXT NOT NULL,
            links_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS frontmatter_values (
            document_path TEXT NOT NULL REFERENCES documents(path) ON DELETE CASCADE,
            field_path TEXT NOT NULL,
            value_type TEXT NOT NULL,
            value_text TEXT,
            value_number REAL,
            value_boolean INTEGER CHECK (value_boolean IS NULL OR value_boolean IN (0, 1)),
            value_date TEXT,
            value_datetime TEXT,
            ordinal INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (document_path, field_path, ordinal)
        );

        CREATE INDEX IF NOT EXISTS idx_frontmatter_text
            ON frontmatter_values(field_path, value_type, value_text);
        CREATE INDEX IF NOT EXISTS idx_frontmatter_number
            ON frontmatter_values(field_path, value_type, value_number);
        CREATE INDEX IF NOT EXISTS idx_frontmatter_boolean
            ON frontmatter_values(field_path, value_type, value_boolean);
        CREATE INDEX IF NOT EXISTS idx_frontmatter_date
            ON frontmatter_values(field_path, value_type, value_date);
        CREATE INDEX IF NOT EXISTS idx_frontmatter_datetime
            ON frontmatter_values(field_path, value_type, value_datetime);

        CREATE TABLE IF NOT EXISTS links (
            source_path TEXT NOT NULL REFERENCES documents(path) ON DELETE CASCADE,
            raw_target TEXT NOT NULL,
            resolved_target TEXT NOT NULL,
            target_kind TEXT NOT NULL,
            occurrence_count INTEGER NOT NULL CHECK (occurrence_count > 0),
            PRIMARY KEY (source_path, raw_target, resolved_target, target_kind)
        );

        CREATE INDEX IF NOT EXISTS idx_links_target
            ON links(resolved_target, target_kind);

        CREATE TABLE IF NOT EXISTS source_references (
            document_path TEXT NOT NULL REFERENCES documents(path) ON DELETE CASCADE,
            ordinal INTEGER NOT NULL DEFAULT 0,
            source_path TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            origin TEXT,
            PRIMARY KEY (document_path, ordinal)
        );

        CREATE INDEX IF NOT EXISTS idx_source_references_path
            ON source_references(source_path);
        CREATE INDEX IF NOT EXISTS idx_source_references_sha256
            ON source_references(sha256);"
    ))
}
