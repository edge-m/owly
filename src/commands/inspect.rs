use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, ValueEnum};
use rusqlite::Connection;
use serde::Serialize;

use super::{context, validate};
use owly_core::document;
use owly_core::workspace::{ScanResult, Workspace};

const DEFAULT_DATABASE: &str = ".owly/index.sqlite";

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "YAML configuration file")]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Human)]
    pub format: Format,
}

#[derive(Debug, Serialize)]
struct InspectOutput {
    name: String,
    wiki_root: String,
    page_count: usize,
    invalid_frontmatter: usize,
    source_tracking: bool,
    directories: Vec<String>,
    types: BTreeMap<String, usize>,
    graph: GraphState,
    validation: validate::Summary,
    index: IndexState,
}

#[derive(Debug, Serialize)]
struct GraphState {
    entrance: Option<String>,
    broken_references: usize,
    orphan_pages: Vec<String>,
    unreachable_pages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct IndexState {
    path: String,
    exists: bool,
    schema_version: Option<i64>,
    document_count: usize,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let context = context::load(args.config.as_deref())?;
    let config = context.config();
    let config_diagnostics = owly_core::validation::lint_config::lint(config);
    let scan = if config_diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        None
    } else {
        Some(Workspace::new(context.wiki_root().to_path_buf()).with_excludes(&config.wiki_lint.exclude).scan()?)
    };

    let (page_count, invalid_frontmatter, types, graph, wiki_diagnostics) = match scan {
        Some(scan) => inspect_scan(&scan, config, context.wiki_root()),
        None => (0, 0, BTreeMap::new(), empty_graph(), Vec::new()),
    };
    let mut diagnostics = config_diagnostics;
    diagnostics.extend(wiki_diagnostics);

    let database = context.wiki_root().join(DEFAULT_DATABASE);
    let index = inspect_index(&database)?;
    let output = InspectOutput {
        name: config.name.clone(),
        wiki_root: context.wiki_root().display().to_string(),
        page_count,
        invalid_frontmatter,
        source_tracking: config.source_tracking.enabled,
        directories: configured_directories(config),
        types,
        graph,
        validation: validate::summary(&diagnostics),
        index,
    };

    match args.format {
        Format::Json => println!("{}", serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?),
        Format::Human => print_human(&output),
    }
    Ok(if output.validation.errors > 0 { 1 } else { 0 })
}

fn inspect_scan(
    scan: &ScanResult, config: &owly_core::config::Config, wiki_root: &Path,
) -> (usize, usize, BTreeMap<String, usize>, GraphState, Vec<owly_core::diagnostic::Diagnostic>) {
    let mut types = BTreeMap::new();
    for document in &scan.documents {
        if let Some(document_type) = document::string_value(&document.frontmatter, "type") {
            *types.entry(document_type).or_default() += 1;
        }
    }

    let mut diagnostics = owly_core::validation::lint_wiki::lint_at_root(scan, config, wiki_root);
    if config.source_tracking.enabled {
        diagnostics.extend(owly_core::validation::lint_wiki::lint_sources_at_root(scan, wiki_root));
    }
    diagnostics.extend(owly_core::validation::orphans::find(scan));
    let graph = owly_core::graph::WikiGraph::from_scan(scan);
    let entrance = scan
        .documents
        .iter()
        .find(|document| document.page_id() == owly_core::graph::ENTRANCE_PAGE_ID)
        .map(|document| document.page_id());
    (
        scan.documents.len(),
        scan.diagnostics.iter().filter(|diagnostic| diagnostic.code == "invalid-frontmatter").count(),
        types,
        GraphState {
            entrance,
            broken_references: graph.broken_references.len(),
            orphan_pages: graph.orphan_pages,
            unreachable_pages: graph.unreachable_pages,
        },
        diagnostics,
    )
}

fn empty_graph() -> GraphState {
    GraphState { entrance: None, broken_references: 0, orphan_pages: Vec::new(), unreachable_pages: Vec::new() }
}

fn configured_directories(config: &owly_core::config::Config) -> Vec<String> {
    fn collect(
        prefix: &str, directories: &indexmap::IndexMap<String, owly_core::config::DirectoryConfig>,
        result: &mut Vec<String>,
    ) {
        for (name, directory) in directories {
            let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            result.push(path.clone());
            collect(&path, &directory.directories, result);
        }
    }

    let mut result = Vec::new();
    collect("", &config.directories, &mut result);
    result
}

fn inspect_index(database: &Path) -> Result<IndexState, String> {
    if !database.is_file() {
        return Ok(IndexState {
            path: database.display().to_string(),
            exists: false,
            schema_version: None,
            document_count: 0,
        });
    }
    let connection =
        Connection::open(database).map_err(|error| format!("cannot open index {}: {error}", database.display()))?;
    let schema_version = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| format!("cannot inspect index {}: {error}", database.display()))?;
    let document_count = connection
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get::<_, i64>(0))
        .map_err(|error| format!("cannot inspect index {}: {error}", database.display()))?;
    Ok(IndexState {
        path: database.display().to_string(),
        exists: true,
        schema_version: Some(schema_version),
        document_count: document_count as usize,
    })
}

fn print_human(output: &InspectOutput) {
    println!("name: {}", output.name);
    println!("wiki_root: {}", output.wiki_root);
    println!("pages: {}", output.page_count);
    println!("invalid_frontmatter: {}", output.invalid_frontmatter);
    println!("source_tracking: {}", output.source_tracking);
    println!("directories:");
    for directory in &output.directories {
        println!("  - {directory}");
    }
    println!("types:");
    for (name, count) in &output.types {
        println!("  - {name}: {count}");
    }
    println!("graph:");
    println!("  entrance: {}", output.graph.entrance.as_deref().unwrap_or("missing"));
    println!("  broken_references: {}", output.graph.broken_references);
    println!("  orphan_pages: {}", output.graph.orphan_pages.len());
    println!("  unreachable_pages: {}", output.graph.unreachable_pages.len());
    println!("validation:");
    println!("  errors: {}", output.validation.errors);
    println!("  warnings: {}", output.validation.warnings);
    println!("index:");
    println!("  path: {}", output.index.path);
    println!("  exists: {}", output.index.exists);
    println!("  documents: {}", output.index.document_count);
}
