use std::path::PathBuf;

use clap::{Args as ClapArgs, ValueEnum};

use super::context;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "YAML configuration file")]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let context = context::load(args.config.as_deref())?;
    let config = context.config();
    let diagnostics = owly_core::validation::lint_config::lint(config);
    let has_errors = diagnostics.iter().any(|diagnostic| diagnostic.is_error());
    match args.format {
        Format::Json => {
            let value = serde_json::json!({
                "name": config.name,
                "wiki_root": context.wiki_root(),
                "source_tracking": {
                    "enabled": config.source_tracking.enabled,
                    "source_item": {
                        "required_fields": ["path", "sha256"],
                        "optional_fields": ["origin"],
                        "path": "wiki-root-relative path, normally under raw/",
                        "sha256": "SHA-256 of the source file",
                    }
                },
                "directories": &config.directories,
                "types": &config.types,
                "diagnostics": &diagnostics,
            });
            println!("{}", serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?);
        }
        Format::Text => {
            println!("name: {}", config.name);
            println!("wiki_root: {}", context.wiki_root().display());
            println!("types:");
            for (name, type_config) in &config.types {
                println!("  - {name}: {} field(s)", type_config.fields.len());
            }
            if diagnostics.is_empty() {
                println!("diagnostics: none");
            } else {
                println!("diagnostics: {}", diagnostics.len());
                for diagnostic in &diagnostics {
                    println!("  - [{}] {}", diagnostic.code, diagnostic.message);
                }
            }
        }
    }
    Ok(if has_errors { 1 } else { 0 })
}
