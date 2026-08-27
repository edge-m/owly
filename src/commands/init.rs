use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use indexmap::IndexMap;
use owly_core::config::{
    Config, ConfigLintConfig, DirectoryConfig, MandatoryFieldRule, TypeConfig, ValueType, WikiLintConfig,
};

const CONFIG_NAME: &str = "owly.yml";

const TOP_LEVEL_COMMENTS: &[(&str, &str)] = &[
    ("owly_version:", "Owly version that generated this configuration."),
    ("wiki_root:", "Wiki root path used by Owly commands."),
    ("directories:", "Directory structure and descriptions."),
    ("mandatory_fields:", "Fields required on every document."),
    ("types:", "Fields specific to each document type."),
    ("wiki_lint:", "Wiki validation settings."),
    ("source_tracking:", "Validation settings for page references to raw sources."),
    ("config_lint:", "Configuration schema validation settings."),
];

#[derive(Debug, ClapArgs)]
pub struct Args {}

fn mandatory_field(value_type: ValueType, description: &str) -> MandatoryFieldRule {
    MandatoryFieldRule {
        value_type,
        description: Some(description.to_string()),
        values: Vec::new(),
        items: None,
        fields: IndexMap::new(),
    }
}

fn add_top_level_comments(yaml: &str) -> String {
    let mut output = String::with_capacity(yaml.len() + TOP_LEVEL_COMMENTS.len() * 50);
    for line in yaml.lines() {
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && let Some((_, comment)) = TOP_LEVEL_COMMENTS.iter().find(|(key, _)| line.starts_with(key))
        {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
            output.push_str("# ");
            output.push_str(comment);
            output.push('\n');
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

pub fn run(_args: &Args) -> Result<u8, String> {
    let config_path = PathBuf::from(CONFIG_NAME);
    if config_path.exists() {
        return Err(format!("configuration file already exists: {}", config_path.display()));
    }

    let config = Config {
        owly_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        wiki_root: Some(PathBuf::from(".")),
        directories: IndexMap::from([(
            "raw".to_string(),
            DirectoryConfig {
                description: Some("Raw data source; files can be added here freely".to_string()),
                directories: IndexMap::from([(
                    "inbox".to_string(),
                    DirectoryConfig { description: Some("Incoming raw files".to_string()), ..Default::default() },
                )]),
            },
        )]),
        mandatory_fields: IndexMap::from([
            ("type".to_string(), mandatory_field(ValueType::String, "Concept type used for routing and filtering.")),
            ("title".to_string(), mandatory_field(ValueType::String, "Human-readable display name.")),
            ("description".to_string(), mandatory_field(ValueType::String, "Short summary of the concept.")),
            (
                "tags".to_string(),
                MandatoryFieldRule {
                    items: Some(Box::new(mandatory_field(ValueType::String, "A short tag."))),
                    ..mandatory_field(ValueType::Array, "Short labels used for categorization and search.")
                },
            ),
            (
                "generated".to_string(),
                MandatoryFieldRule {
                    fields: IndexMap::from([
                        (
                            "at".to_string(),
                            mandatory_field(ValueType::Datetime, "UTC timestamp of the last meaningful change."),
                        ),
                        ("by".to_string(), mandatory_field(ValueType::String, "Actor that generated the content.")),
                    ]),
                    ..mandatory_field(ValueType::Object, "How and when the content was generated.")
                },
            ),
            (
                "stale_after".to_string(),
                mandatory_field(
                    ValueType::Date,
                    "Date on or after which the content is considered stale. Agents should usually set it to 14 days after creation, adjusting for how quickly the information changes.",
                ),
            ),
        ]),
        types: IndexMap::from([(
            "note".to_string(),
            TypeConfig { description: Some("A general-purpose note".to_string()), fields: IndexMap::new() },
        )]),
        wiki_lint: WikiLintConfig {
            exclude: vec!["**/.*".to_string(), "**/README.md".to_string(), "raw/**".to_string()],
        },
        config_lint: ConfigLintConfig { max_nesting_depth: Some(1) },
        source_tracking: owly_core::config::SourceTrackingConfig { enabled: true },
    };
    let yaml =
        serde_yaml::to_string(&config).map_err(|error| format!("cannot serialize default configuration: {error}"))?;
    let content = add_top_level_comments(&yaml);

    fs::write(&config_path, content)
        .map_err(|error| format!("cannot write configuration {}: {error}", config_path.display()))?;
    println!("wrote {}", config_path.display());

    Ok(0)
}
