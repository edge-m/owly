/// A link extracted from an indexed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedLink {
    pub source_path: String,
    pub raw_target: String,
    pub resolved_target: String,
    pub target_kind: String,
    pub occurrence_count: i64,
}
