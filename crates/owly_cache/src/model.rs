/// A cached Markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub path: String,
    pub content_hash: String,
    pub frontmatter_hash: String,
    pub links_hash: String,
}

/// A typed frontmatter leaf value.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontmatterValue {
    pub document_path: String,
    pub field_path: String,
    pub value_type: String,
    pub value_text: Option<String>,
    pub value_number: Option<f64>,
    pub value_boolean: Option<bool>,
    pub value_date: Option<String>,
    pub value_datetime: Option<String>,
    pub ordinal: i64,
}

/// A cached link relation grouped by source, target, and raw spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub source_path: String,
    pub raw_target: String,
    pub resolved_target: String,
    pub target_kind: String,
    pub occurrence_count: i64,
}

/// A source recorded in a page's `sources` frontmatter field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReference {
    pub document_path: String,
    pub ordinal: i64,
    pub source_path: String,
    pub sha256: String,
    pub origin: Option<String>,
}

/// A value used by an exact frontmatter lookup.
#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Date(String),
    Datetime(String),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub field_path: String,
    pub operator: Operator,
    pub value: TypedValue,
}

impl Condition {
    pub fn new(field_path: impl Into<String>, operator: Operator, value: TypedValue) -> Self {
        Self { field_path: field_path.into(), operator, value }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchQuery {
    pub conditions: Vec<Condition>,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn and(mut self, field_path: impl Into<String>, operator: Operator, value: TypedValue) -> Self {
        self.conditions.push(Condition::new(field_path, operator, value));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}
