//! The per-directory, machine-local file metadata cache.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

/// Metadata used to decide whether a source file should be read again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Nanoseconds since the Unix epoch.
    pub mtime: u128,
    pub size: u64,
}

/// A compact JSON cache for one wiki directory.
///
/// The cache contains no parsed content. It is safe to delete and regenerate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalFileCache {
    /// Metadata for the shared SQLite database, when one is configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<FileMetadata>,
    /// Metadata for the Markdown files in this directory, keyed by path.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub files: BTreeMap<String, FileMetadata>,
}

impl LocalFileCache {
    /// The conventional cache filename to add to a wiki's ignore rules.
    pub const FILE_NAME: &'static str = ".owly-cache.json";

    /// Read a cache. Missing or malformed cache files are treated as empty.
    pub fn load(path: impl AsRef<Path>) -> Self {
        fs::read(path).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
    }

    /// Write compact JSON using an atomic replacement.
    pub fn save_atomic(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let temporary = temporary_path(path);
        let bytes = serde_json::to_vec(self).map_err(io::Error::other)?;
        fs::write(&temporary, bytes)?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        Ok(())
    }

    /// Record a file's current metadata and return whether it changed.
    pub fn update_file(&mut self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let relative_path = path.to_string_lossy().replace('\\', "/");
        self.update_file_with_key(&relative_path, path)
    }

    /// Record a file using a caller-supplied key, useful for directory-local caches.
    pub fn update_file_with_key(&mut self, key: &str, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let metadata = file_metadata(path)?;
        let changed = self.files.get(key) != Some(&metadata);
        self.files.insert(key.replace('\\', "/"), metadata);
        Ok(changed)
    }

    /// Record the shared database's current metadata and return whether it changed.
    pub fn update_database(&mut self, path: impl AsRef<Path>) -> io::Result<bool> {
        let metadata = file_metadata(path.as_ref())?;
        let changed = self.database != Some(metadata);
        self.database = Some(metadata);
        Ok(changed)
    }

    /// Check a file against the cached metadata without reading its contents.
    pub fn is_file_current(&self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let key = path.to_string_lossy().replace('\\', "/");
        self.is_file_current_with_key(&key, path)
    }

    /// Check a file using a caller-supplied key, without reading its contents.
    pub fn is_file_current_with_key(&self, key: &str, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let key = key.replace('\\', "/");
        Ok(self.files.get(&key) == Some(&file_metadata(path)?))
    }
}

fn file_metadata(path: &Path) -> io::Result<FileMetadata> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata.modified()?.duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
    Ok(FileMetadata { mtime, size: metadata.len() })
}

fn temporary_path(path: &Path) -> PathBuf {
    let extension = format!("tmp-{}", std::process::id());
    path.with_extension(extension)
}
