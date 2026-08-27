use std::path::PathBuf;

use clap::Args as ClapArgs;
use owly_core::document;
use owly_core::workspace::Workspace;
use serde::Serialize;
use serde_yaml::Value;

use super::context;
use super::overview::Format;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub query: String,
    #[arg(long, value_enum, default_value_t = Format::Json)]
    pub format: Format,
}

#[derive(Debug, Serialize)]
struct SearchResult {
    page_id: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    matched_field: String,
    value: String,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let config_context = context::load(args.config.as_deref())?;
    let query = Query::parse(&args.query)?;
    let scan = Workspace::new(config_context.wiki_root().to_path_buf())
        .with_excludes(&config_context.config().wiki_lint.exclude)
        .scan()?;
    let results = scan
        .documents
        .iter()
        .filter_map(|document| {
            let value = document.frontmatter.get(Value::String(query.field.clone()))?;
            query.matches(value).then(|| SearchResult {
                page_id: document.page_id(),
                path: document.relative_file_path_from_wiki_root.clone(),
                r#type: document::string_value(&document.frontmatter, "type"),
                matched_field: query.field.clone(),
                value: display_value(value),
            })
        })
        .collect::<Vec<_>>();

    match args.format {
        Format::Json => println!("{}", serde_json::to_string_pretty(&results).map_err(|error| error.to_string())?),
        Format::Human => {
            for result in &results {
                println!("{}\t{}\t{}", result.page_id, result.matched_field, result.value);
            }
        }
    }
    Ok(0)
}

struct Query {
    field: String,
    value: String,
}

impl Query {
    fn parse(query: &str) -> Result<Self, String> {
        let (field, value) = query.split_once(':').ok_or_else(|| "query must use the form field:value".to_string())?;
        if field.is_empty() || value.is_empty() {
            return Err("query must use the form field:value".to_string());
        }
        Ok(Self { field: field.to_string(), value: value.to_string() })
    }

    fn matches(&self, value: &Value) -> bool {
        match value {
            Value::String(candidate) => candidate == &self.value,
            Value::Bool(candidate) => candidate.to_string() == self.value,
            Value::Number(candidate) => candidate.to_string() == self.value,
            Value::Sequence(values) => values.iter().any(|value| self.matches(value)),
            _ => false,
        }
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<unserializable>".to_string()),
    }
}
