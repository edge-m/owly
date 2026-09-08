use std::fs;

use super::{LocalFileCache, discover, load, sha256};

#[test]
fn discovers_and_loads_markdown() {
    let root = std::env::temp_dir().join(format!("owly-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/a.md"), "---\ntags: [rust, sqlite]\npriority: 10\nsources:\n  - path: raw/a.json\n    sha256: abc\n---\nSee [b](b.md) and [b](b.md).").unwrap();

    let paths = discover(&root).unwrap();
    assert_eq!(paths, [std::path::PathBuf::from("docs/a.md")]);
    let index = load(&root, "docs/a.md").unwrap();
    assert_eq!(index.document.path, "docs/a.md");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sha256_matches_standard_vector() {
    assert_eq!(sha256(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn normalizes_datetime_offsets_to_utc() {
    let root = std::env::temp_dir().join(format!("owly-source-datetime-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("note.md"), "---\ngenerated: 2026-08-12T19:30:00+09:00\n---\n").unwrap();

    let index = load(&root, "note.md").unwrap();
    assert_eq!(index.frontmatter[0].value, owly_types::FrontmatterValue::Datetime("2026-08-12T10:30:00Z".to_string()));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn loads_large_markdown_through_buffered_reader() {
    let root = std::env::temp_dir().join(format!("owly-source-large-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let mut content = String::from("---\ntype: note\n---\n");
    content.push_str(&"body ".repeat((256 * 1024 / 5) + 1));
    fs::write(root.join("large.md"), content).unwrap();

    let index = load(&root, "large.md").unwrap();
    assert_eq!(index.document.path, "large.md");
    assert!(index.frontmatter.iter().any(|value| value.field_path == "type"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_cache_round_trips_compact_json_and_detects_changes() {
    let root = std::env::temp_dir().join(format!("owly-source-cache-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let document = root.join("note.md");
    let cache_path = root.join(LocalFileCache::FILE_NAME);
    fs::write(&document, "one").unwrap();

    let mut cache = LocalFileCache::default();
    assert!(cache.update_file(&document).unwrap());
    assert!(cache.is_file_current(&document).unwrap());
    cache.save_atomic(&cache_path).unwrap();
    let bytes = fs::read(&cache_path).unwrap();
    assert!(!bytes.contains(&b'\n'));
    assert_eq!(LocalFileCache::load(&cache_path), cache);

    fs::write(&document, "two").unwrap();
    assert!(!cache.is_file_current(&document).unwrap());
    assert!(LocalFileCache::load(root.join("missing.json")).files.is_empty());
    fs::remove_dir_all(root).unwrap();
}
