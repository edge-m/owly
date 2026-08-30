/// A typed frontmatter value independent of its persistence representation.
#[derive(Debug, Clone, PartialEq)]
pub enum FrontmatterValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Date(String),
    Datetime(String),
    Null,
}

/// A frontmatter value associated with its document and field path.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedFrontmatterValue {
    pub document_path: String,
    pub field_path: String,
    pub value: FrontmatterValue,
    pub ordinal: i64,
}
