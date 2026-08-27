use std::collections::{HashMap, HashSet};

use crate::diagnostic::Diagnostic;
use crate::link_resolver::OutgoingTarget;
use crate::workspace::ScanResult;

/// Find Markdown pages that cannot be reached from the wiki-root `index.md`.
pub fn find(scan: &ScanResult) -> Vec<Diagnostic> {
    let pages: HashSet<String> = scan.documents.iter().map(|document| document.page_id()).collect();
    let mut links = HashMap::<String, Vec<String>>::new();
    for document in &scan.documents {
        let source = document.page_id();
        let targets = document
            .outgoing_links()
            .links
            .into_iter()
            .filter_map(|link| match link.target {
                OutgoingTarget::Page { id } if pages.contains(&id) => Some(id),
                _ => None,
            })
            .collect();
        links.insert(source, targets);
    }

    let mut visited = HashSet::new();
    let mut stack = vec!["index".to_string()];
    while let Some(page) = stack.pop() {
        if !visited.insert(page.clone()) {
            continue;
        }
        if let Some(targets) = links.get(&page) {
            stack.extend(targets.iter().cloned());
        }
    }

    scan.documents
        .iter()
        .filter(|document| {
            let page = document.page_id();
            !visited.contains(&page)
        })
        .map(|document| {
            Diagnostic::error(
                "orphan-page",
                Some(document.relative_file_path_from_wiki_root.clone()),
                "page is not reachable from index.md",
            )
        })
        .collect()
}
