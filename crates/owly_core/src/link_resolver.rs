use std::path::Path;

use crate::document::{RawLink, RawLinks};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct OutgoingLinks {
    pub links: Vec<OutgoingLink>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutgoingLink {
    pub label: String,
    pub raw_target: String,
    pub target: OutgoingTarget,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutgoingTarget {
    Page { id: String },
    Resource { path: String },
    External { url: String },
}

pub fn resolve(raw_links: &RawLinks, source_path: &str) -> OutgoingLinks {
    OutgoingLinks { links: raw_links.links.iter().filter_map(|link| resolve_link(link, source_path)).collect() }
}

fn resolve_link(link: &RawLink, source_path: &str) -> Option<OutgoingLink> {
    let target = classify_target(&link.target, source_path)?;
    Some(OutgoingLink {
        label: link.label.clone(),
        raw_target: link.target.clone(),
        target,
        line: link.line,
        column: link.column,
    })
}

fn classify_target(raw_target: &str, source_path: &str) -> Option<OutgoingTarget> {
    if raw_target.starts_with('#') {
        return None;
    }
    if raw_target.contains("://") || raw_target.starts_with("mailto:") {
        return Some(OutgoingTarget::External { url: raw_target.to_string() });
    }

    let path = raw_target.split_once('#').map_or(raw_target, |(path, _)| path);
    if path.is_empty() {
        return None;
    }
    let path = normalize_internal_path(source_path, path);
    if Path::new(&path).extension().and_then(|extension| extension.to_str()) == Some("md") {
        let id = path.strip_suffix(".md").unwrap_or(&path).to_string();
        Some(OutgoingTarget::Page { id })
    } else {
        Some(OutgoingTarget::Resource { path })
    }
}

fn normalize_internal_path(source_path: &str, target: &str) -> String {
    let source_parent = Path::new(source_path).parent().unwrap_or_else(|| Path::new(""));
    let combined = if target.starts_with('/') {
        target.to_string()
    } else {
        source_parent.join(target).to_string_lossy().into_owned()
    };
    let mut components = Vec::new();
    let combined = combined.replace('\\', "/");
    for component in combined.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            component => components.push(component),
        }
    }
    components.join("/")
}

#[cfg(test)]
mod tests {
    use super::{OutgoingTarget, resolve};
    use crate::document::{RawLink, RawLinks};

    #[test]
    fn resolves_pages_resources_and_external_urls() {
        let raw_links = RawLinks {
            links: vec![
                RawLink { label: "page".to_string(), target: "../spec.md".to_string(), line: 4, column: 5 },
                RawLink {
                    label: "resource".to_string(),
                    target: "../assets/logo.png".to_string(),
                    line: 5,
                    column: 5,
                },
                RawLink {
                    label: "external".to_string(),
                    target: "https://example.com".to_string(),
                    line: 6,
                    column: 5,
                },
            ],
        };

        let resolved = resolve(&raw_links, "docs/guide.md");
        assert!(matches!(resolved.links[0].target, OutgoingTarget::Page { ref id } if id == "spec"));
        assert!(
            matches!(resolved.links[1].target, OutgoingTarget::Resource { ref path } if path == "assets/logo.png")
        );
        assert!(
            matches!(resolved.links[2].target, OutgoingTarget::External { ref url } if url == "https://example.com")
        );
    }
}
