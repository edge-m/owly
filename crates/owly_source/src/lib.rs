//! File discovery and Markdown loading for Owly wikis.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use owly_types::{
    DocumentIndex, FrontmatterValue, IndexedDocument, IndexedFrontmatterValue, IndexedLink, IndexedSourceReference,
};
use serde_yaml::Value;
use walkdir::WalkDir;

mod hash;
mod local_cache;

pub use hash::sha256;
pub use local_cache::{FileMetadata, LocalFileCache};

/// An error produced while discovering or loading a source document.
#[derive(Debug)]
pub enum SourceError {
    InvalidRoot(PathBuf),
    InvalidPath(String),
    Walk(walkdir::Error),
    Io { path: PathBuf, source: std::io::Error },
    InvalidUtf8 { path: PathBuf, source: std::string::FromUtf8Error },
    InvalidFrontmatter { path: String, message: String },
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(path) => write!(formatter, "wiki path is not a directory: {}", path.display()),
            Self::InvalidPath(path) => write!(formatter, "invalid wiki-relative path: {path}"),
            Self::Walk(error) => write!(formatter, "cannot walk wiki: {error}"),
            Self::Io { path, source } => write!(formatter, "cannot read {}: {source}", path.display()),
            Self::InvalidUtf8 { path, source } => write!(formatter, "file is not UTF-8: {}: {source}", path.display()),
            Self::InvalidFrontmatter { path, message } => {
                write!(formatter, "invalid frontmatter in {path}: {message}")
            }
        }
    }
}

impl std::error::Error for SourceError {}

/// Discover Markdown files below `root` as normalized relative paths in deterministic order.
pub fn discover(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, SourceError> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(SourceError::InvalidRoot(root.to_path_buf()));
    }

    let mut paths = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(SourceError::Walk)?;
        if entry.file_type().is_file() && is_markdown(entry.path()) {
            let relative =
                entry.path().strip_prefix(root).map_err(|error| SourceError::InvalidPath(error.to_string()))?;
            let relative = normalize_relative_path(relative)?;
            paths.push(PathBuf::from(relative));
        }
    }
    paths.sort();
    Ok(paths)
}

/// Load and index one Markdown file relative to the wiki root.
pub fn load(root: impl AsRef<Path>, relative_path: impl AsRef<Path>) -> Result<DocumentIndex, SourceError> {
    let relative_path = relative_path.as_ref();
    let relative = normalize_relative_path(relative_path)?;
    let path = root.as_ref().join(relative_path);
    let bytes = fs::read(&path).map_err(|source| SourceError::Io { path: path.clone(), source })?;
    let content = String::from_utf8(bytes.clone()).map_err(|source| SourceError::InvalidUtf8 { path, source })?;
    let (frontmatter, body) = split_frontmatter(&content).ok_or_else(|| SourceError::InvalidFrontmatter {
        path: relative.clone(),
        message: "frontmatter must start with '---' and end with a matching delimiter".to_string(),
    })?;
    let yaml: Value = serde_yaml::from_str(frontmatter)
        .map_err(|error| SourceError::InvalidFrontmatter { path: relative.clone(), message: error.to_string() })?;
    if !matches!(yaml, Value::Mapping(_)) {
        return Err(SourceError::InvalidFrontmatter {
            path: relative,
            message: "frontmatter must contain a YAML mapping".to_string(),
        });
    }

    let links = extract_links(body, &relative);
    let links_hash = sha256(serialize_links(&links).as_bytes());
    let mut values = flatten_frontmatter(&yaml);
    let mut sources = source_references(&yaml, &relative);
    for value in &mut values {
        value.document_path = relative.clone();
    }
    for source in &mut sources {
        source.document_path = relative.clone();
    }
    let frontmatter_hash = sha256(serialize_frontmatter(&values).as_bytes());
    Ok(DocumentIndex {
        document: IndexedDocument {
            path: relative.clone(),
            content_hash: sha256(&bytes),
            frontmatter_hash,
            links_hash,
        },
        frontmatter: values,
        links,
        sources,
    })
}

fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("md")
}

fn normalize_relative_path(path: &Path) -> Result<String, SourceError> {
    if path.is_absolute() {
        return Err(SourceError::InvalidPath(path.display().to_string()));
    }
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(component) => components.push(component.to_string_lossy().into_owned()),
            _ => return Err(SourceError::InvalidPath(path.display().to_string())),
        }
    }
    if components.is_empty() {
        return Err(SourceError::InvalidPath(path.display().to_string()));
    }
    Ok(components.join("/"))
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let mut lines = content.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim() != "---" {
        return None;
    }
    let frontmatter_start = first.len();
    let mut offset = frontmatter_start;
    for line in lines {
        let line_start = offset;
        offset += line.len();
        if line.trim() == "---" {
            return Some((&content[frontmatter_start..line_start], &content[offset..]));
        }
    }
    None
}

fn flatten_frontmatter(document: &Value) -> Vec<IndexedFrontmatterValue> {
    let mut values = Vec::new();
    if let Value::Mapping(mapping) = document {
        for (key, value) in mapping {
            if let Some(key) = key.as_str() {
                flatten_value(key.to_string(), value, &mut values, 0);
            }
        }
    }
    values
}

fn flatten_value(path: String, value: &Value, values: &mut Vec<IndexedFrontmatterValue>, ordinal: i64) {
    match value {
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                if let Some(key) = key.as_str() {
                    flatten_value(format!("{path}.{key}"), value, values, ordinal);
                }
            }
        }
        Value::Sequence(sequence) => {
            for (ordinal, value) in sequence.iter().enumerate() {
                flatten_value(path.clone(), value, values, ordinal as i64);
            }
        }
        value => values.push(IndexedFrontmatterValue {
            document_path: String::new(),
            field_path: path,
            value: typed_value(value),
            ordinal,
        }),
    }
}

fn typed_value(value: &Value) -> FrontmatterValue {
    match value {
        Value::Null => FrontmatterValue::Null,
        Value::Bool(value) => FrontmatterValue::Boolean(*value),
        Value::Number(value) => FrontmatterValue::Number(value.as_f64().unwrap_or_default()),
        Value::String(value) if is_date(value) => FrontmatterValue::Date(value.clone()),
        Value::String(value) if is_datetime(value) => FrontmatterValue::Datetime(value.clone()),
        Value::String(value) => FrontmatterValue::Text(value.clone()),
        _ => FrontmatterValue::Null,
    }
}

fn serialize_frontmatter(values: &[IndexedFrontmatterValue]) -> String {
    let mut entries = values
        .iter()
        .map(|value| format!("{}\0{}\0{:?}", value.field_path, value.ordinal, value.value))
        .collect::<Vec<_>>();
    entries.sort();
    entries.join("\n")
}

fn is_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value.bytes().enumerate().all(|(i, byte)| i == 4 || i == 7 || byte.is_ascii_digit())
}

fn is_datetime(value: &str) -> bool {
    value.len() > 10 && value.as_bytes().get(10).is_some_and(|byte| *byte == b'T' || *byte == b' ')
}

fn source_references(document: &Value, _path: &str) -> Vec<IndexedSourceReference> {
    let Some(items) = document.get("sources").and_then(Value::as_sequence) else { return Vec::new() };
    items
        .iter()
        .enumerate()
        .filter_map(|(ordinal, item)| {
            let mapping = item.as_mapping()?;
            Some(IndexedSourceReference {
                document_path: String::new(),
                ordinal: ordinal as i64,
                source_path: mapping.get("path")?.as_str()?.to_string(),
                sha256: mapping.get("sha256")?.as_str()?.to_string(),
                origin: mapping.get("origin").and_then(Value::as_str).map(ToOwned::to_owned),
            })
        })
        .collect()
}

fn extract_links(body: &str, source_path: &str) -> Vec<IndexedLink> {
    let mut links = BTreeMap::<(String, String, String), i64>::new();
    let mut offset = 0;
    while let Some(open) = body[offset..].find('[') {
        let open = offset + open;
        if open > 0 && body.as_bytes()[open - 1] == b'!' {
            offset = open + 1;
            continue;
        }
        let Some(label_end) = body[open + 1..].find(']') else { break };
        let label_end = open + label_end + 1;
        if body.as_bytes().get(label_end + 1) != Some(&b'(') {
            offset = label_end + 1;
            continue;
        }
        let Some(close) = body[label_end + 2..].find(')') else { break };
        let close = label_end + close + 2;
        let destination = body[label_end + 2..close].trim();
        let target = destination
            .split_once(char::is_whitespace)
            .map_or(destination, |(target, _)| target)
            .trim_matches(['<', '>']);
        if !target.is_empty() && !target.starts_with('#') {
            let (resolved, kind) = resolve_target(target, source_path);
            *links.entry((target.to_string(), resolved, kind)).or_default() += 1;
        }
        offset = close + 1;
    }
    links
        .into_iter()
        .map(|((raw_target, resolved_target, target_kind), occurrence_count)| IndexedLink {
            source_path: source_path.to_string(),
            raw_target,
            resolved_target,
            target_kind,
            occurrence_count,
        })
        .collect()
}

fn resolve_target(target: &str, source_path: &str) -> (String, String) {
    if target.contains("://") || target.starts_with("mailto:") {
        return (target.to_string(), "external".to_string());
    }
    let target = target.split_once('#').map_or(target, |(path, _)| path);
    let parent = Path::new(source_path).parent().unwrap_or_else(|| Path::new(""));
    let joined = if target.starts_with('/') {
        target.trim_start_matches('/').to_string()
    } else {
        parent.join(target).to_string_lossy().into_owned()
    };
    let mut components = Vec::new();
    let normalized = joined.replace('\\', "/");
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            component => components.push(component),
        }
    }
    let resolved = components.join("/");
    let kind = if resolved.ends_with(".md") { "page" } else { "resource" };
    (resolved, kind.to_string())
}

fn serialize_links(links: &[IndexedLink]) -> String {
    links
        .iter()
        .map(|link| {
            format!("{}\0{}\0{}\0{}", link.raw_target, link.resolved_target, link.target_kind, link.occurrence_count)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
