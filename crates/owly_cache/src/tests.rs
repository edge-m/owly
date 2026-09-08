use owly_types::{
    DocumentIndex, FrontmatterValue, IndexedDocument, IndexedFrontmatterValue, IndexedLink, IndexedSourceReference,
};

use super::{CacheDb, Operator, SearchQuery};

fn document() -> IndexedDocument {
    IndexedDocument {
        path: "notes/example.md".to_string(),
        frontmatter_hash: "frontmatter".to_string(),
        links_hash: "links".to_string(),
    }
}

#[test]
fn initializes_schema_and_crud_is_atomic() {
    let mut db = CacheDb::open_in_memory().expect("database should open");
    assert_eq!(db.schema_version().unwrap(), 1);
    let document = document();
    let values = [
        IndexedFrontmatterValue {
            document_path: document.path.clone(),
            field_path: "priority".to_string(),
            value: FrontmatterValue::Number(10.0),
            ordinal: 0,
        },
        IndexedFrontmatterValue {
            document_path: document.path.clone(),
            field_path: "tags".to_string(),
            value: FrontmatterValue::Text("sqlite".to_string()),
            ordinal: 0,
        },
    ];
    let links = [IndexedLink {
        source_path: document.path.clone(),
        raw_target: "../other.md".to_string(),
        resolved_target: "notes/other.md".to_string(),
        target_kind: "page".to_string(),
        occurrence_count: 2,
    }];
    let sources = [IndexedSourceReference {
        document_path: document.path.clone(),
        ordinal: 0,
        source_path: "raw/example.md".to_string(),
        sha256: "sha".to_string(),
        origin: Some("https://example.com".to_string()),
    }];

    db.replace_document_index(&document, &values, &links, &sources).unwrap();
    let index = DocumentIndex {
        document: document.clone(),
        frontmatter: values.to_vec(),
        links: links.to_vec(),
        sources: sources.to_vec(),
    };
    db.replace_index(&index).unwrap();
    assert_eq!(db.document(&document.path).unwrap(), Some(document.clone()));
    assert_eq!(db.frontmatter_values(&document.path).unwrap(), values);
    assert_eq!(db.links(&document.path).unwrap(), links);
    assert_eq!(db.incoming_links("notes/other.md").unwrap(), links);
    assert_eq!(db.source_references(&document.path).unwrap(), sources);
    assert_eq!(
        db.find_documents("tags", &FrontmatterValue::Text("sqlite".to_string())).unwrap(),
        ["notes/example.md".to_string()]
    );
    assert_eq!(
        db.find_documents("priority", &FrontmatterValue::Number(10.0)).unwrap(),
        ["notes/example.md".to_string()]
    );
    assert_eq!(
        db.search(&SearchQuery::new().and("priority", Operator::Gte, FrontmatterValue::Number(10.0)).and(
            "tags",
            Operator::Eq,
            FrontmatterValue::Text("sqlite".to_string())
        ),)
            .unwrap(),
        ["notes/example.md".to_string()]
    );
    assert!(
        db.search(&SearchQuery::new().and("priority", Operator::Gt, FrontmatterValue::Number(10.0)))
            .unwrap()
            .is_empty()
    );
    assert!(
        db.search(&SearchQuery::new().and("unknown", Operator::Eq, FrontmatterValue::Text("value".to_string())))
            .unwrap()
            .is_empty()
    );
    assert!(db.search(&SearchQuery::new()).is_err());
    db.upsert_document(&IndexedDocument { links_hash: "changed".to_string(), ..document.clone() }).unwrap();
    assert_eq!(db.document(&document.path).unwrap().unwrap().links_hash, "changed");
    assert!(db.delete_document(&document.path).unwrap());
    assert!(db.document(&document.path).unwrap().is_none());
    assert!(db.frontmatter_values(&document.path).unwrap().is_empty());
}
