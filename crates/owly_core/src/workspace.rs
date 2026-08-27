use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::matches_glob;
use crate::diagnostic::Diagnostic;
use crate::document::{self, Document};

pub struct ScanResult {
    pub documents: Vec<Document>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Workspace {
    root: PathBuf,
    excludes: Vec<String>,
    walk_errors: Vec<walkdir::Error>,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self { root, excludes: Vec::new(), walk_errors: Vec::new() }
    }

    pub fn with_excludes(mut self, excludes: &[String]) -> Self {
        self.excludes = excludes.to_vec();
        self
    }

    pub fn scan(&mut self) -> Result<ScanResult, String> {
        if !self.root.is_dir() {
            return Err(format!("wiki path is not a directory: {}", self.root.display()));
        }

        let mut documents = Vec::new();
        let mut diagnostics = Vec::new();
        for result in WalkDir::new(&self.root) {
            let entry = match result {
                Ok(entry) => entry,
                Err(error) => {
                    self.walk_errors.push(error);
                    continue;
                }
            };
            if !entry.file_type().is_file() || !is_markdown(entry.path()) {
                continue;
            }
            let relative = relative_path(&self.root, entry.path())?;
            if self.excludes.iter().any(|pattern| matches_glob(pattern, &relative)) {
                continue;
            }
            let content = fs::read_to_string(entry.path())
                .map_err(|error| format!("cannot read {}: {error}", entry.path().display()))?;
            match document::parse(relative, &content) {
                Ok(document) => documents.push(document),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }

        Ok(ScanResult { documents, diagnostics })
    }

    pub fn walk_errors(&self) -> &[walkdir::Error] {
        &self.walk_errors
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("md")
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map_err(|error| format!("cannot calculate relative path: {error}"))
        .map(|path| document::normalize_relative_path(&path.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Workspace;

    #[test]
    fn excluded_markdown_is_not_scanned() {
        let root = std::env::temp_dir().join(format!("owly-workspace-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("raw")).expect("test directory should be created");
        fs::write(root.join("index.md"), "---\n{}\n---\n").expect("index should be written");
        fs::write(root.join("raw/source.md"), "not frontmatter").expect("raw file should be written");

        let scan =
            Workspace::new(root.clone()).with_excludes(&["raw/**".to_string()]).scan().expect("scan should succeed");
        assert_eq!(scan.documents.len(), 1);
        assert_eq!(scan.documents[0].relative_file_path_from_wiki_root, "index.md");
        assert!(scan.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("test directory should be removed");
    }
}
