mod schema;

use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub use schema::{
    ConfigLintConfig, DirectoryConfig, FieldRule, MandatoryFieldRule, SourceTrackingConfig, TypeConfig, ValueType,
    WikiLintConfig, matches_glob,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Version of owly that generated this configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owly_version: Option<String>,
    /// Wiki root, relative to the configuration file when it is not absolute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wiki_root: Option<PathBuf>,
    /// Directory roles keyed by their path relative to the wiki root.
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub directories: IndexMap<String, DirectoryConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub mandatory_fields: IndexMap<String, MandatoryFieldRule>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub types: IndexMap<String, TypeConfig>,
    pub wiki_lint: WikiLintConfig,
    pub config_lint: ConfigLintConfig,
    /// Rules for validating page references to files in the raw source tree.
    pub source_tracking: SourceTrackingConfig,
}

/// A configuration together with the path context needed to resolve relative paths.
#[derive(Debug, Clone)]
pub struct ConfigContext {
    config: Config,
    base_dir: PathBuf,
    wiki_root: PathBuf,
}

impl ConfigContext {
    /// Build a context from an in-memory configuration and its path base directory.
    pub fn new(config: Config, base_dir: impl Into<PathBuf>) -> Result<Self, String> {
        let base_dir = base_dir.into();
        let wiki_root = config
            .wiki_root
            .as_ref()
            .map(|wiki_root| if wiki_root.is_absolute() { wiki_root.clone() } else { base_dir.join(wiki_root) })
            .ok_or_else(|| "configuration does not define wiki_root".to_string())?;
        Ok(Self { config, base_dir, wiki_root })
    }

    /// Return the underlying configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Return the directory used to resolve relative configuration paths.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Return the resolved wiki root.
    pub fn wiki_root(&self) -> &Path {
        &self.wiki_root
    }

    /// Return the configured document type by name.
    pub fn type_config(&self, name: &str) -> Option<&TypeConfig> {
        self.config.types.get(name)
    }

    /// Return the configured directory by its wiki-relative path.
    pub fn directory_config(&self, path: &Path) -> Option<&DirectoryConfig> {
        let mut directories = &self.config.directories;
        let mut directory = None;
        for component in path.components() {
            let name = match component {
                std::path::Component::Normal(name) => name.to_str()?,
                std::path::Component::CurDir => continue,
                _ => return None,
            };
            directory = directories.get(name);
            directories = &directory?.directories;
        }
        directory
    }
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|error| format!("cannot read config {}: {error}", path.display()))?;
        serde_yaml::from_str(&content).map_err(|error| {
            let rendered = error.to_string();
            let is_unknown_field = rendered.contains("unknown field");
            let message = rendered.split_once(" at line ").map_or(rendered.as_str(), |(message, _)| message);
            let location = error.location().map_or_else(
                || "location: unavailable".to_string(),
                |location| format!("location: line {}, column {}", location.line(), location.column()),
            );
            let title = if is_unknown_field { "invalid configuration" } else { "invalid YAML" };
            let help = if is_unknown_field {
                "check the setting name against the supported configuration keys"
            } else {
                "check indentation and `key: value` syntax"
            };
            format!("{title} in {}\n  {location}\n  message: {message}\n  help: {help}", path.display())
        })
    }

    pub fn wiki_root_path(&self, config_path: &Path) -> Option<PathBuf> {
        self.wiki_root.as_ref().map(|wiki_root| {
            if wiki_root.is_absolute() {
                wiki_root.clone()
            } else {
                config_path.parent().unwrap_or_else(|| Path::new(".")).join(wiki_root)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_is_rejected_in_mandatory_fields() {
        let result: Result<Config, _> =
            serde_yaml::from_str("mandatory_fields:\n  title:\n    type: string\n    optional: true\n");

        assert!(result.is_err());
    }

    #[test]
    fn context_resolves_paths_and_nested_directory_configurations() {
        let config: Config = serde_yaml::from_str(
            "wiki_root: wiki\ndirectories:\n  docs:\n    directories:\n      guides:\n        description: Guides\ntypes:\n  guide:\n    description: A guide\n",
        )
        .expect("config should parse");
        let context = ConfigContext::new(config, "/project/config").expect("context should resolve");

        assert_eq!(context.wiki_root(), Path::new("/project/config/wiki"));
        assert_eq!(context.directory_config(Path::new("docs/guides")).unwrap().description.as_deref(), Some("Guides"));
        assert_eq!(context.type_config("guide").unwrap().description.as_deref(), Some("A guide"));
    }
}
