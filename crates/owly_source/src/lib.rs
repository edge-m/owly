//! File discovery and Markdown loading for Owly wikis.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, SecondsFormat, Utc};
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
    let file = fs::File::open(&path).map_err(|source| SourceError::Io { path: path.clone(), source })?;
    let mut reader = BufReader::new(file);
    let yaml = read_frontmatter(&mut reader, &path, &relative)?;
    let links = extract_links_from_reader(&mut reader, &relative, &path)?;
    let links_hash = sha256(serialize_links(&links).as_bytes());
    if !matches!(yaml, Value::Mapping(_)) {
        return Err(SourceError::InvalidFrontmatter {
            path: relative,
            message: "frontmatter must contain a YAML mapping".to_string(),
        });
    }

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
        document: IndexedDocument { path: relative.clone(), frontmatter_hash, links_hash },
        frontmatter: values,
        links,
        sources,
    })
}

fn read_frontmatter(reader: &mut impl BufRead, path: &Path, relative: &str) -> Result<Value, SourceError> {
    let first = next_line(reader, path)?.ok_or_else(|| invalid_frontmatter(relative))?;
    if first.trim() != "---" {
        return Err(invalid_frontmatter(relative));
    }
    let mut frontmatter = String::new();
    loop {
        let Some(line) = next_line(reader, path)? else { return Err(invalid_frontmatter(relative)) };
        if line.trim() == "---" {
            break;
        }
        frontmatter.push_str(&line);
    }
    let yaml = serde_yaml::from_str(&frontmatter)
        .map_err(|error| SourceError::InvalidFrontmatter { path: relative.to_string(), message: error.to_string() })?;
    Ok(yaml)
}

fn next_line(reader: &mut impl BufRead, path: &Path) -> Result<Option<String>, SourceError> {
    let mut bytes = Vec::new();
    let length =
        reader.read_until(b'\n', &mut bytes).map_err(|source| SourceError::Io { path: path.to_path_buf(), source })?;
    if length == 0 {
        return Ok(None);
    }
    String::from_utf8(bytes).map(Some).map_err(|source| SourceError::InvalidUtf8 { path: path.to_path_buf(), source })
}

fn invalid_frontmatter(path: &str) -> SourceError {
    SourceError::InvalidFrontmatter {
        path: path.to_string(),
        message: "frontmatter must start with '---' and end with a matching delimiter".to_string(),
    }
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
        Value::String(value) if is_datetime(value) => {
            FrontmatterValue::Datetime(normalize_datetime(value).unwrap_or_else(|| value.clone()))
        }
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

fn normalize_datetime(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|datetime| datetime.with_timezone(&Utc).to_rfc3339_opts(SecondsFormat::AutoSi, true))
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

fn extract_links_from_reader(
    reader: &mut impl BufRead, source_path: &str, path: &Path,
) -> Result<Vec<IndexedLink>, SourceError> {
    let mut extractor = LinkExtractor::new(source_path);
    while let Some(line) = next_line(reader, path)? {
        extractor.push(&line);
    }
    Ok(extractor.finish())
}

struct LinkExtractor<'a> {
    source_path: &'a str,
    links: BTreeMap<(String, String, String), i64>,
    state: LinkState,
    previous_was_bang: bool,
}

enum LinkState {
    Searching,
    Label { image: bool },
    AfterLabel { image: bool },
    Destination(String),
}

impl<'a> LinkExtractor<'a> {
    fn new(source_path: &'a str) -> Self {
        Self { source_path, links: BTreeMap::new(), state: LinkState::Searching, previous_was_bang: false }
    }
    fn push(&mut self, input: &str) {
        for character in input.chars() {
            match &mut self.state {
                LinkState::Searching => {
                    if character == '[' {
                        self.state = LinkState::Label { image: self.previous_was_bang };
                    }
                    self.previous_was_bang = character == '!';
                }
                LinkState::Label { image } => {
                    if character == ']' {
                        self.state = LinkState::AfterLabel { image: *image };
                    }
                }
                LinkState::AfterLabel { image } => {
                    self.state = if character == '(' && !*image {
                        LinkState::Destination(String::new())
                    } else {
                        LinkState::Searching
                    };
                    self.previous_was_bang = character == '!';
                }
                LinkState::Destination(destination) => {
                    if character == ')' {
                        let target = destination
                            .trim()
                            .split_once(char::is_whitespace)
                            .map_or(destination.trim(), |(target, _)| target)
                            .trim_matches(['<', '>']);
                        if !target.is_empty() && !target.starts_with('#') {
                            let (resolved, kind) = resolve_target(target, self.source_path);
                            *self.links.entry((target.to_string(), resolved, kind)).or_default() += 1;
                        }
                        self.state = LinkState::Searching;
                        self.previous_was_bang = false;
                    } else {
                        destination.push(character);
                    }
                }
            }
        }
    }
    fn finish(self) -> Vec<IndexedLink> {
        self.links
            .into_iter()
            .map(|((raw_target, resolved_target, target_kind), occurrence_count)| IndexedLink {
                source_path: self.source_path.to_string(),
                raw_target,
                resolved_target,
                target_kind,
                occurrence_count,
            })
            .collect()
    }
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
    let mut escaped_components = 0;
    let normalized = joined.replace('\\', "/");
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    escaped_components += 1;
                }
            }
            component => components.push(component),
        }
    }
    let resolved = format!("{}{}", "../".repeat(escaped_components), components.join("/"));
    let kind = if escaped_components > 0
        && Path::new(&resolved).file_name().and_then(|name| name.to_str()) == Some("entrance.md")
    {
        "external_graph"
    } else if resolved.ends_with(".md") {
        "page"
    } else {
        "resource"
    };
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
