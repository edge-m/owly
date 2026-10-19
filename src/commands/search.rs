use std::path::PathBuf;

use clap::Args as ClapArgs;
use owly_cache::CacheDb;
use owly_types::FrontmatterValue;
use serde::Serialize;

use super::overview::Format;
use super::{context, index};

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
    index::update(args.config.as_deref(), None, false)?;
    let database = config_context.wiki_root().join(".owly/index.sqlite");
    let db =
        CacheDb::open(&database).map_err(|error| format!("cannot open database {}: {error}", database.display()))?;
    let paths =
        db.find_documents(&query.field, &query.value).map_err(|error| format!("cannot search database: {error}"))?;
    let results = paths
        .into_iter()
        .map(|path| {
            let values = db.frontmatter_values(&path).unwrap_or_default();
            let value = values
                .iter()
                .find(|value| value.field_path == query.field)
                .map(|value| display_value(&value.value))
                .unwrap_or_else(|| query.display_value());
            let document_type = values.iter().find_map(|value| {
                if value.field_path != "type" {
                    return None;
                }
                match &value.value {
                    FrontmatterValue::Text(value) => Some(value.clone()),
                    _ => None,
                }
            });
            SearchResult {
                page_id: path.strip_suffix(".md").unwrap_or(&path).to_string(),
                path,
                r#type: document_type,
                matched_field: query.field.clone(),
                value,
            }
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
    value: FrontmatterValue,
    raw_value: String,
}

impl Query {
    fn parse(query: &str) -> Result<Self, String> {
        let (field, value) = query.split_once(':').ok_or_else(|| "query must use the form field:value".to_string())?;
        if field.is_empty() || value.is_empty() {
            return Err("query must use the form field:value".to_string());
        }
        Ok(Self { field: field.to_string(), value: parse_value(value), raw_value: value.to_string() })
    }

    fn display_value(&self) -> String {
        self.raw_value.clone()
    }
}

fn parse_value(value: &str) -> FrontmatterValue {
    if value == "true" {
        FrontmatterValue::Boolean(true)
    } else if value == "false" {
        FrontmatterValue::Boolean(false)
    } else if let Ok(value) = value.parse::<f64>() {
        FrontmatterValue::Number(value)
    } else if value.len() == 10 && value.as_bytes().get(4) == Some(&b'-') && value.as_bytes().get(7) == Some(&b'-') {
        FrontmatterValue::Date(value.to_string())
    } else if value.len() > 10 && matches!(value.as_bytes().get(10), Some(b'T' | b' ')) {
        FrontmatterValue::Datetime(value.to_string())
    } else {
        FrontmatterValue::Text(value.to_string())
    }
}

fn display_value(value: &FrontmatterValue) -> String {
    match value {
        FrontmatterValue::Text(value) | FrontmatterValue::Date(value) | FrontmatterValue::Datetime(value) => {
            value.clone()
        }
        FrontmatterValue::Number(value) => value.to_string(),
        FrontmatterValue::Boolean(value) => value.to_string(),
        FrontmatterValue::Null => "null".to_string(),
    }
}
