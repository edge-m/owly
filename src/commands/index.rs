use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, ValueEnum};
use owly_cache::CacheDb;
use owly_source::LocalFileCache;
use owly_types::{DocumentIndex, IndexedDocument};
use serde::Serialize;

use super::context;

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
    #[arg(long, help = "SQLite index path, relative to the wiki root")]
    pub database: Option<PathBuf>,
    #[arg(long, help = "Rebuild the complete index")]
    pub force: bool,
    #[arg(long, value_enum, default_value_t = Format::Human)]
    pub format: Format,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub database: String,
    pub rebuilt: bool,
    pub indexed: usize,
    pub reused: usize,
    pub removed: usize,
    pub warnings: Vec<String>,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let summary = update(args.config.as_deref(), args.database.as_deref(), args.force)?;
    print_summary(&summary, args.format)?;
    Ok(0)
}

pub fn update(config_path: Option<&Path>, database_path: Option<&Path>, force: bool) -> Result<Summary, String> {
    let context = context::load(config_path)?;
    let root = context.wiki_root();
    if !root.is_dir() {
        return Err(format!("wiki path is not a directory: {}", root.display()));
    }

    let database = database_path.map_or_else(|| root.join(DEFAULT_DATABASE), |path| root.join(path));
    if let Some(parent) = database.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create database directory {}: {error}", parent.display()))?;
    }
    let database_existed = database.is_file();
    let root_cache_path = root.join(LocalFileCache::FILE_NAME);
    let mut root_cache = LocalFileCache::load(&root_cache_path);
    let cached_database = root_cache.database;
    let database_changed = database_existed
        && root_cache
            .update_database(&database)
            .map_err(|error| format!("cannot inspect database {}: {error}", database.display()))?;
    let rebuild = force || !database_existed || cached_database.is_none() || database_changed;

    let discovered = owly_source::discover(root).map_err(|error| error.to_string())?;
    let excluded = context.config().wiki_lint.exclude.as_slice();
    let paths = discovered
        .into_iter()
        .filter(|path| {
            let path_string = path.to_string_lossy().replace('\\', "/");
            !excluded.iter().any(|pattern| owly_core::config::matches_glob(pattern, &path_string))
        })
        .collect::<Vec<_>>();
    let grouped = group_by_directory(root, &paths);
    let warnings = grouped
        .iter()
        .filter(|(_, files)| files.len() > 250)
        .map(|(directory, files)| {
            format!("directory {} contains {} Markdown files; consider splitting it", directory.display(), files.len())
        })
        .collect::<Vec<_>>();

    let working_database = if rebuild {
        let temporary = database.with_extension(format!("sqlite.tmp-{}", std::process::id()));
        if temporary.exists() {
            fs::remove_file(&temporary)
                .map_err(|error| format!("cannot remove temporary database {}: {error}", temporary.display()))?;
        }
        temporary
    } else {
        database.clone()
    };
    let mut db = CacheDb::open(&working_database)
        .map_err(|error| format!("cannot open database {}: {error}", working_database.display()))?;

    let current_paths = paths.iter().map(|path| path.to_string_lossy().replace('\\', "/")).collect::<BTreeSet<_>>();
    let mut indexed = 0;
    let mut reused = 0;
    for (directory, files) in &grouped {
        let cache_path = directory.join(LocalFileCache::FILE_NAME);
        let mut cache = LocalFileCache::load(&cache_path);
        for (relative, path) in files {
            let key = relative.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            let changed_by_metadata = cache
                .update_file_with_key(key, path)
                .map_err(|error| format!("cannot inspect source {}: {error}", path.display()))?;
            let changed = rebuild || changed_by_metadata;
            if changed {
                let index = owly_source::load(root, relative).map_err(|error| error.to_string())?;
                let existing = db
                    .document(&index.document.path)
                    .map_err(|error| format!("cannot read {} from index: {error}", relative.display()))?;
                let replace = derived_index_changed(existing.as_ref(), &index);
                if replace {
                    db.replace_index(&index)
                        .map_err(|error| format!("cannot write {} to index: {error}", relative.display()))?;
                    indexed += 1;
                } else {
                    reused += 1;
                }
            } else {
                reused += 1;
            }
        }
        cache.files.retain(|key, _| {
            files.iter().any(|(path, _)| path.file_name().and_then(|name| name.to_str()) == Some(key))
        });
        cache
            .save_atomic(&cache_path)
            .map_err(|error| format!("cannot write cache {}: {error}", cache_path.display()))?;
    }

    let mut removed = 0;
    for path in db.document_paths().map_err(|error| format!("cannot read indexed paths: {error}"))? {
        if !current_paths.contains(&path) {
            removed +=
                usize::from(db.delete_document(&path).map_err(|error| format!("cannot remove {path}: {error}"))?);
        }
    }
    drop(db);
    if rebuild {
        fs::rename(&working_database, &database).map_err(|error| {
            format!("cannot replace database {} with {}: {error}", database.display(), working_database.display())
        })?;
    }
    let mut root_cache = LocalFileCache::load(&root_cache_path);
    root_cache
        .update_database(&database)
        .map_err(|error| format!("cannot inspect database {}: {error}", database.display()))?;
    root_cache
        .save_atomic(&root_cache_path)
        .map_err(|error| format!("cannot write cache {}: {error}", root_cache_path.display()))?;

    Ok(Summary { database: database.display().to_string(), rebuilt: rebuild, indexed, reused, removed, warnings })
}

fn print_summary(summary: &Summary, format: Format) -> Result<(), String> {
    match format {
        Format::Human => {
            println!("{}", if summary.rebuilt { "rebuilt index" } else { "updated index" });
            println!("indexed: {}, reused: {}, removed: {}", summary.indexed, summary.reused, summary.removed);
            for warning in &summary.warnings {
                eprintln!("warning: {warning}");
            }
        }
        Format::Json => println!("{}", serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?),
    }
    Ok(())
}

fn derived_index_changed(existing: Option<&IndexedDocument>, index: &DocumentIndex) -> bool {
    existing.is_none_or(|document| {
        document.frontmatter_hash != index.document.frontmatter_hash
            || document.links_hash != index.document.links_hash
    })
}

fn group_by_directory(root: &Path, paths: &[PathBuf]) -> BTreeMap<PathBuf, Vec<(PathBuf, PathBuf)>> {
    let mut grouped = BTreeMap::new();
    for relative in paths {
        let directory = relative.parent().map_or_else(|| root.to_path_buf(), |parent| root.join(parent));
        grouped.entry(directory).or_insert_with(Vec::new).push((relative.clone(), root.join(relative)));
    }
    grouped
}

#[cfg(test)]
mod tests {
    use std::fs;

    use owly_cache::CacheDb;
    use owly_types::{DocumentIndex, IndexedDocument};

    use super::{Args, Format, derived_index_changed, run};

    fn index_with_hashes(frontmatter_hash: &str, links_hash: &str) -> DocumentIndex {
        DocumentIndex {
            document: IndexedDocument {
                path: "note.md".to_string(),
                frontmatter_hash: frontmatter_hash.to_string(),
                links_hash: links_hash.to_string(),
            },
            frontmatter: Vec::new(),
            links: Vec::new(),
            sources: Vec::new(),
        }
    }

    #[test]
    fn detects_derived_index_changes_by_hash() {
        let existing = index_with_hashes("frontmatter", "links").document;
        assert!(!derived_index_changed(Some(&existing), &index_with_hashes("frontmatter", "links")));
        assert!(derived_index_changed(Some(&existing), &index_with_hashes("changed", "links")));
        assert!(derived_index_changed(Some(&existing), &index_with_hashes("frontmatter", "changed")));
        assert!(derived_index_changed(None, &index_with_hashes("frontmatter", "links")));
    }

    #[test]
    fn builds_and_reuses_an_index() {
        let root = std::env::temp_dir().join(format!("owly-index-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let wiki = root.join("wiki");
        fs::create_dir_all(&wiki).unwrap();
        fs::write(
            root.join("owly.yml"),
            "wiki_root: wiki\nwiki_lint:\n  exclude: []\nsource_tracking:\n  enabled: false\n",
        )
        .unwrap();
        fs::write(wiki.join("note.md"), "---\ntags: [rust]\n---\n").unwrap();
        let args = Args { config: Some(root.join("owly.yml")), database: None, force: false, format: Format::Json };

        run(&args).unwrap();
        let database = wiki.join(".owly/index.sqlite");
        let db = CacheDb::open(&database).unwrap();
        assert_eq!(db.document_paths().unwrap(), ["note.md".to_string()]);
        drop(db);
        run(&args).unwrap();
        assert!(wiki.join(".owly-cache.json").is_file());
        assert!(wiki.join(".owly/index.sqlite").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_existing_database_when_rebuild_fails() {
        let root = std::env::temp_dir().join(format!("owly-index-failure-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let wiki = root.join("wiki");
        fs::create_dir_all(&wiki).unwrap();
        fs::write(
            root.join("owly.yml"),
            "wiki_root: wiki\nwiki_lint:\n  exclude: []\nsource_tracking:\n  enabled: false\n",
        )
        .unwrap();
        fs::write(wiki.join("valid.md"), "---\ntags: [rust]\n---\n").unwrap();
        let args = Args { config: Some(root.join("owly.yml")), database: None, force: false, format: Format::Json };

        run(&args).unwrap();
        let database = wiki.join(".owly/index.sqlite");
        fs::write(wiki.join("invalid.md"), "---\ntags: [\n---\n").unwrap();
        let rebuild_args = Args { force: true, ..args };
        assert!(run(&rebuild_args).is_err());

        let db = CacheDb::open(&database).unwrap();
        assert_eq!(db.document_paths().unwrap(), ["valid.md".to_string()]);
        fs::remove_dir_all(root).unwrap();
    }
}
