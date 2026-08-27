use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use crate::{link_resolver::OutgoingTarget, workspace::ScanResult};

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrokenReference {
    pub source: String,
    pub target: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct WikiGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub broken_references: Vec<BrokenReference>,
    pub orphan_pages: Vec<String>,
    pub unreachable_pages: Vec<String>,
}

impl WikiGraph {
    pub fn from_scan(scan: &ScanResult) -> Self {
        let mut nodes = scan
            .documents
            .iter()
            .map(|document| GraphNode {
                id: document.page_id(),
                path: document.relative_file_path_from_wiki_root.clone(),
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));

        let page_ids = nodes.iter().map(|node| node.id.as_str()).collect::<HashSet<_>>();
        let mut edges = Vec::new();
        let mut broken_references = Vec::new();
        for document in &scan.documents {
            let source = document.page_id();
            for link in document.outgoing_links().links {
                if let OutgoingTarget::Page { id } = link.target {
                    if page_ids.contains(id.as_str()) {
                        edges.push(GraphEdge {
                            source: source.clone(),
                            target: id,
                            label: link.label,
                            line: link.line,
                            column: link.column,
                        });
                    } else {
                        broken_references.push(BrokenReference {
                            source: source.clone(),
                            target: id,
                            line: link.line,
                            column: link.column,
                        });
                    }
                }
            }
        }
        edges.sort_by(|left, right| {
            (&left.source, &left.target, left.line).cmp(&(&right.source, &right.target, right.line))
        });
        broken_references.sort_by(|left, right| {
            (&left.source, &left.target, left.line).cmp(&(&right.source, &right.target, right.line))
        });

        let mut outgoing = HashMap::<&str, Vec<&str>>::new();
        let mut incoming = HashMap::<&str, usize>::new();
        for edge in &edges {
            outgoing.entry(edge.source.as_str()).or_default().push(edge.target.as_str());
            *incoming.entry(edge.target.as_str()).or_default() += 1;
        }

        let mut reachable = HashSet::new();
        let mut queue = VecDeque::from(["index"]);
        while let Some(id) = queue.pop_front() {
            if !reachable.insert(id) {
                continue;
            }
            if let Some(targets) = outgoing.get(id) {
                queue.extend(targets.iter().copied());
            }
        }

        let mut orphan_pages = nodes
            .iter()
            .filter(|node| !incoming.contains_key(node.id.as_str()) && node.id != "index")
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        let mut unreachable_pages = nodes
            .iter()
            .filter(|node| !reachable.contains(node.id.as_str()))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        orphan_pages.sort();
        unreachable_pages.sort();

        Self { nodes, edges, broken_references, orphan_pages, unreachable_pages }
    }
}

#[cfg(test)]
mod tests {
    use super::WikiGraph;
    use crate::{document::parse, workspace::ScanResult};

    #[test]
    fn builds_edges_and_reachability() {
        let index = parse("index.md".into(), "---\n{}\n---\n[Note](note.md)\n").unwrap();
        let note = parse("note.md".into(), "---\n{}\n---\n[Missing](missing.md)\n").unwrap();
        let orphan = parse("orphan.md".into(), "---\n{}\n---\n").unwrap();
        let graph = WikiGraph::from_scan(&ScanResult { documents: vec![index, note, orphan], diagnostics: vec![] });

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.broken_references[0].target, "missing");
        assert_eq!(graph.orphan_pages, vec!["orphan"]);
        assert_eq!(graph.unreachable_pages, vec!["orphan"]);
    }
}
