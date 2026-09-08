/// A document and the hashes used to synchronize its derived index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDocument {
    pub path: String,
    pub frontmatter_hash: String,
    pub links_hash: String,
}

/// The complete parsed index data for one document.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentIndex {
    pub document: IndexedDocument,
    pub frontmatter: Vec<crate::IndexedFrontmatterValue>,
    pub links: Vec<crate::IndexedLink>,
    pub sources: Vec<crate::IndexedSourceReference>,
}
