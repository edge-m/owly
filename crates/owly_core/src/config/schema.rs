use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct WikiLintConfig {
    /// Markdown paths to exclude, relative to the wiki root.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::WikiLintConfig;

    #[test]
    fn exclude_patterns_match_relative_markdown_paths() {
        let config = WikiLintConfig {
            exclude: vec!["raw/**".to_string(), "**/SKILL.md".to_string(), "draft-?.md".to_string()],
        };

        assert!(config.is_excluded("raw/source.md"));
        assert!(config.is_excluded("nested/SKILL.md"));
        assert!(config.is_excluded("SKILL.md"));
        assert!(config.is_excluded("draft-1.md"));
        assert!(!config.is_excluded("notes/draft-1.md"));
    }
}

impl WikiLintConfig {
    pub fn is_excluded(&self, path: &str) -> bool {
        self.exclude.iter().any(|pattern| matches_glob(pattern, path))
    }
}

pub fn matches_glob(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_start_matches("./").replace('\\', "/");
    let path = path.replace('\\', "/");
    matches_glob_bytes(pattern.as_bytes(), path.as_bytes())
}

fn matches_glob_bytes(pattern: &[u8], path: &[u8]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    if pattern.starts_with(b"**") {
        if pattern.starts_with(b"**/") && matches_glob_bytes(&pattern[3..], path) {
            return true;
        }
        let rest = &pattern[2..];
        return matches_glob_bytes(rest, path) || (!path.is_empty() && matches_glob_bytes(pattern, &path[1..]));
    }
    if path.is_empty() {
        return false;
    }
    match pattern[0] {
        b'*' => {
            matches_glob_bytes(&pattern[1..], path) || (path[0] != b'/' && matches_glob_bytes(pattern, &path[1..]))
        }
        b'?' => path[0] != b'/' && matches_glob_bytes(&pattern[1..], &path[1..]),
        byte => byte == path[0] && matches_glob_bytes(&pattern[1..], &path[1..]),
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ConfigLintConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SourceTrackingConfig {
    /// Whether formal `sources` references are validated.
    pub enabled: bool,
}

impl Default for SourceTrackingConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct DirectoryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub directories: IndexMap<String, DirectoryConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct TypeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub fields: IndexMap<String, FieldRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MandatoryFieldRule {
    #[serde(rename = "type")]
    pub value_type: ValueType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<MandatoryFieldRule>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: IndexMap<String, MandatoryFieldRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FieldRule {
    #[serde(rename = "type")]
    pub value_type: ValueType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<FieldRule>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: IndexMap<String, FieldRule>,
}

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Date,
    Datetime,
    Boolean,
    Number,
    Array,
    Object,
}
