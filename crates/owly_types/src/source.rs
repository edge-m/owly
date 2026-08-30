/// A source recorded in an indexed document's `sources` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSourceReference {
    pub document_path: String,
    pub ordinal: i64,
    pub source_path: String,
    pub sha256: String,
    pub origin: Option<String>,
}
