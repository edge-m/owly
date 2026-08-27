//! Markdown document parsing and front matter extraction.

use std::collections::BTreeMap;

use serde_yaml::{Mapping, Value};

use crate::diagnostic::Diagnostic;

#[derive(Debug)]
/// A parsed Markdown document without retaining its body text.
pub struct Document {
    /// Relative file path from the wiki root, including the `.md` extension.
    pub relative_file_path_from_wiki_root: String,
    /// YAML front matter represented as a mapping of field names to values.
    pub frontmatter: Mapping,
    /// Markdown links extracted from the body before semantic resolution.
    pub raw_links: RawLinks,
}

impl Document {
    /// Return the stable page ID derived from this document's wiki-relative path.
    pub fn page_id(&self) -> String {
        page_id(&self.relative_file_path_from_wiki_root)
    }

    /// Resolve links found in this document against its wiki-relative path.
    pub fn outgoing_links(&self) -> crate::link_resolver::OutgoingLinks {
        crate::link_resolver::resolve(&self.raw_links, &self.relative_file_path_from_wiki_root)
    }
}

#[derive(Debug, Default)]
/// Links found in a document while preserving their original Markdown data.
pub struct RawLinks {
    /// Links in source order.
    pub links: Vec<RawLink>,
}

#[derive(Debug, PartialEq, Eq)]
/// A single Markdown inline link before it is classified as a page, resource, or external URL.
pub struct RawLink {
    /// The text between the link's square brackets.
    pub label: String,
    /// The original link destination, without an optional Markdown link title.
    pub target: String,
    /// One-based line of the opening `[` in the Markdown link.
    pub line: usize,
    /// One-based column of the opening `[` in the Markdown link.
    pub column: usize,
}

/// Parse a Markdown document's YAML front matter and extract raw inline links.
///
/// The document body is inspected for links but is not retained in the returned
/// [`Document`]. The path is expected to be relative to the wiki root and is
/// preserved as provided.
///
/// Returns an `invalid-frontmatter` diagnostic when the document does not start
/// with a matching pair of `---` delimiters or when the front matter is not a
/// YAML mapping.
pub fn parse(path: String, content: &str) -> Result<Document, Diagnostic> {
    let (frontmatter, body) = split_frontmatter(content).ok_or_else(|| {
        Diagnostic::error(
            "invalid-frontmatter",
            Some(path.clone()),
            "frontmatter must start with '---' and end with a matching delimiter",
        )
    })?;

    let frontmatter: Value = serde_yaml::from_str(frontmatter).map_err(|error| {
        Diagnostic::error("invalid-frontmatter", Some(path.clone()), format!("cannot parse YAML frontmatter: {error}"))
    })?;
    let frontmatter = match frontmatter {
        Value::Mapping(mapping) => mapping,
        _ => {
            return Err(Diagnostic::error(
                "invalid-frontmatter",
                Some(path),
                "frontmatter must contain a YAML mapping",
            ));
        }
    };

    let body_start = content.len() - body.len();
    let body_line_offset = content[..body_start].bytes().filter(|byte| *byte == b'\n').count();
    Ok(Document {
        relative_file_path_from_wiki_root: path.clone(),
        frontmatter,
        raw_links: extract_raw_links(body, body_line_offset),
    })
}

fn extract_raw_links(body: &str, body_line_offset: usize) -> RawLinks {
    let mut links = Vec::new();
    let mut offset = 0;

    while let Some(open_offset) = body[offset..].find('[') {
        let open_offset = offset + open_offset;
        if open_offset > 0 && body.as_bytes()[open_offset - 1] == b'!' {
            offset = open_offset + 1;
            continue;
        }

        let Some(label_end) = body[open_offset + 1..].find(']') else {
            break;
        };
        let label_end = open_offset + 1 + label_end;
        let destination_start = label_end + 1;
        if body.as_bytes().get(destination_start) != Some(&b'(') {
            offset = label_end + 1;
            continue;
        }

        let Some(destination_end) = body[destination_start + 1..].find(')') else {
            break;
        };
        let destination_end = destination_start + 1 + destination_end;
        let label = body[open_offset + 1..label_end].to_string();
        let destination = body[destination_start + 1..destination_end].trim();
        let raw_target = destination.split_once(char::is_whitespace).map_or(destination, |(target, _)| target);
        let raw_target = raw_target.trim_matches('<').trim_matches('>');
        if !raw_target.is_empty() {
            let (line, column) = line_and_column(body, open_offset, body_line_offset);
            links.push(RawLink { label, target: raw_target.to_string(), line, column });
        }
        offset = destination_end + 1;
    }

    RawLinks { links }
}

fn line_and_column(content: &str, offset: usize, line_offset: usize) -> (usize, usize) {
    let line = content[..offset].bytes().filter(|byte| *byte == b'\n').count() + line_offset + 1;
    let column = content[..offset].rsplit('\n').next().map_or(1, |line| line.chars().count() + 1);
    (line, column)
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
            let content_end = line_start;
            let body_start = offset;
            return Some((&content[frontmatter_start..content_end], &content[body_start..]));
        }
    }
    None
}

/// Return a string-valued front matter field.
pub fn string_value(frontmatter: &Mapping, field: &str) -> Option<String> {
    frontmatter.get(Value::String(field.to_string())).and_then(Value::as_str).map(ToOwned::to_owned)
}

/// Check whether a front matter field is present, regardless of its value type.
pub fn has_field(frontmatter: &Mapping, field: &str) -> bool {
    frontmatter.contains_key(Value::String(field.to_string()))
}

/// Return the stable page ID derived from a Markdown path relative to the wiki root.
pub fn page_id(path: &str) -> String {
    let normalized = normalize_relative_path(path);
    normalized.strip_suffix(".md").unwrap_or(&normalized).to_string()
}

/// Normalize a wiki-relative path to its platform-independent representation.
pub fn normalize_relative_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Return string-keyed front matter entries in deterministic key order.
pub fn values(frontmatter: &Mapping) -> BTreeMap<String, &Value> {
    frontmatter.iter().filter_map(|(key, value)| Some((key.as_str()?.to_string(), value))).collect()
}

#[cfg(test)]
mod tests {
    use super::{normalize_relative_path, page_id, parse};

    #[test]
    fn extracts_markdown_inline_links_without_retaining_body() {
        let document = parse(
            "note.md".to_string(),
            "---\nid: note\n---\nSee [other](other.md) and [external](https://example.com).\n![image](image.png)\n",
        )
        .expect("document should parse");

        assert_eq!(
            document.raw_links.links,
            [
                super::RawLink { label: "other".to_string(), target: "other.md".to_string(), line: 4, column: 5 },
                super::RawLink {
                    label: "external".to_string(),
                    target: "https://example.com".to_string(),
                    line: 4,
                    column: 27,
                },
            ]
        );
        assert_eq!(document.page_id(), "note");
        assert_eq!(document.outgoing_links().links.len(), 2);
        assert_eq!(page_id("docs\\guide.md"), "docs/guide");
        assert_eq!(normalize_relative_path("docs\\guide.md"), "docs/guide.md");
    }
}
