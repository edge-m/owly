use std::fs;

use super::{discover, load, sha256};

#[test]
fn discovers_and_loads_markdown() {
    let root = std::env::temp_dir().join(format!("owly-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/a.md"), "---\ntags: [rust, sqlite]\npriority: 10\nsources:\n  - path: raw/a.json\n    sha256: abc\n---\nSee [b](b.md) and [b](b.md).").unwrap();

    let paths = discover(&root).unwrap();
    assert_eq!(paths.len(), 1);
    let index = load(&root, "docs/a.md").unwrap();
    assert_eq!(index.document.path, "docs/a.md");
    assert!(!index.document.content_hash.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sha256_matches_standard_vector() {
    assert_eq!(sha256(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}
