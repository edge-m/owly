use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Args as ClapArgs, ValueEnum};
use owly_core::config::{
    Config, ConfigContext, DirectoryConfig, FieldRule, MandatoryFieldRule, TypeConfig, ValueType,
};
use owly_core::document;
use owly_core::workspace::{ScanResult, Workspace};
use serde::Serialize;

use super::context;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, ClapArgs)]
pub struct DirectoryArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub statistics: bool,
    #[arg(long, value_enum, default_value_t = Format::Json)]
    pub format: Format,
}

#[derive(Debug, ClapArgs)]
pub struct TypeArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub statistics: bool,
    #[arg(long)]
    pub r#type: Option<String>,
    #[arg(long, value_enum, default_value_t = Format::Json)]
    pub format: Format,
}

#[derive(Debug, Serialize)]
struct DirectoryView {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statistics: Option<DirectoryStatistics>,
    directories: Vec<DirectoryView>,
}

#[derive(Debug, Serialize)]
struct DirectoryStatistics {
    total: usize,
    types: BTreeMap<String, usize>,
    #[serde(skip_serializing_if = "is_zero")]
    invalid_frontmatter: usize,
}

#[derive(Debug, Serialize)]
struct TypeView {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statistics: Option<TypeStatistics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<TypeSchema>,
}

#[derive(Debug, Serialize)]
struct TypeStatistics {
    count: usize,
}

#[derive(Debug, Serialize)]
struct TypeSchema {
    required_fields: BTreeMap<String, FieldView>,
    optional_fields: BTreeMap<String, FieldView>,
}

#[derive(Debug, Serialize)]
struct FieldView {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<FieldView>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    fields: BTreeMap<String, FieldView>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

pub fn directories(args: &DirectoryArgs) -> Result<u8, String> {
    let context = context::load(args.config.as_deref())?;
    let config = context.config();
    let scan = if args.statistics { Some(scan(&context)?) } else { None };
    let directories = config
        .directories
        .iter()
        .map(|(name, directory)| directory_view(directory, name, scan.as_ref()))
        .collect::<Vec<_>>();
    print_value(&directories, args.format, |directory| format_directory(directory, 0))
}

pub fn types(args: &TypeArgs) -> Result<u8, String> {
    let context = context::load(args.config.as_deref())?;
    let config = context.config();
    let scan = if args.statistics || args.r#type.is_some() { Some(scan(&context)?) } else { None };
    let selected = args.r#type.as_deref();
    if let Some(name) = selected
        && !config.types.contains_key(name)
    {
        return Err(format!("unknown document type '{name}'"));
    }
    let views = config
        .types
        .iter()
        .filter(|(name, _)| selected.is_none_or(|selected| selected == name.as_str()))
        .map(|(name, type_config)| {
            type_view(name, type_config, config, args.statistics, selected.is_some(), scan.as_ref())
        })
        .collect::<Vec<_>>();
    print_value(&views, args.format, |view| {
        view.iter()
            .map(|view| {
                let mut line = view.name.clone();
                if let Some(description) = &view.description {
                    line.push_str(&format!(": {description}"));
                }
                if let Some(statistics) = &view.statistics {
                    line.push_str(&format!(" ({})", statistics.count));
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn scan(context: &ConfigContext) -> Result<ScanResult, String> {
    Workspace::new(context.wiki_root().to_path_buf()).with_excludes(&context.config().wiki_lint.exclude).scan()
}

fn directory_view(config: &DirectoryConfig, path: &str, scan: Option<&ScanResult>) -> DirectoryView {
    DirectoryView {
        path: path.to_string(),
        description: config.description.clone(),
        statistics: scan.map(|scan| directory_statistics(path, scan)),
        directories: config
            .directories
            .iter()
            .map(|(child_name, child)| {
                let child_path = format!("{path}/{child_name}");
                directory_view(child, &child_path, scan)
            })
            .collect(),
    }
}

fn directory_statistics(path: &str, scan: &ScanResult) -> DirectoryStatistics {
    let prefix = format!("{path}/");
    let documents = scan.documents.iter().filter(|document| {
        let file = document.relative_file_path_from_wiki_root.as_str();
        file.starts_with(&prefix)
    });
    let mut statistics = DirectoryStatistics { total: 0, types: BTreeMap::new(), invalid_frontmatter: 0 };
    for document in documents {
        statistics.total += 1;
        if let Some(document_type) = document::string_value(&document.frontmatter, "type") {
            *statistics.types.entry(document_type).or_default() += 1;
        }
    }
    statistics.invalid_frontmatter = scan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path.as_deref().is_some_and(|file| file.starts_with(&prefix)))
        .count();
    statistics
}

fn type_view(
    name: &str, config: &TypeConfig, root_config: &Config, statistics: bool, include_schema: bool,
    scan: Option<&ScanResult>,
) -> TypeView {
    TypeView {
        name: name.to_string(),
        description: config.description.clone(),
        statistics: statistics.then(|| TypeStatistics {
            count: scan
                .into_iter()
                .flat_map(|scan| scan.documents.iter())
                .filter(|document| document::string_value(&document.frontmatter, "type").as_deref() == Some(name))
                .count(),
        }),
        schema: include_schema.then(|| type_schema(config, root_config)),
    }
}

fn type_schema(config: &TypeConfig, root_config: &Config) -> TypeSchema {
    let mut required_fields = root_config
        .mandatory_fields
        .iter()
        .map(|(field, rule)| (field.clone(), mandatory_field_view(rule)))
        .collect::<BTreeMap<_, _>>();
    let mut optional_fields = BTreeMap::new();
    for (field, rule) in &config.fields {
        if rule.optional {
            optional_fields.insert(field.clone(), field_view(rule));
        } else {
            required_fields.insert(field.clone(), field_view(rule));
        }
    }
    TypeSchema { required_fields, optional_fields }
}

fn print_value<T: Serialize>(value: &T, format: Format, human: impl Fn(&T) -> String) -> Result<u8, String> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(value).map_err(|error| error.to_string())?),
        Format::Human => println!("{}", human(value)),
    }
    Ok(0)
}

fn format_directory(directory: &[DirectoryView], _depth: usize) -> String {
    directory
        .iter()
        .map(|directory| {
            let statistics = directory
                .statistics
                .as_ref()
                .map_or(String::new(), |statistics| format!(" ({} articles)", statistics.total));
            format!("{}{}", directory.path, statistics)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn field_view(rule: &FieldRule) -> FieldView {
    FieldView {
        r#type: value_type_name(&rule.value_type).to_string(),
        description: rule.description.clone(),
        values: rule.values.clone(),
        items: rule.items.as_deref().map(field_view).map(Box::new),
        fields: rule.fields.iter().map(|(field, rule)| (field.clone(), field_view(rule))).collect(),
    }
}

#[allow(dead_code)]
fn mandatory_field_view(rule: &MandatoryFieldRule) -> FieldView {
    FieldView {
        r#type: value_type_name(&rule.value_type).to_string(),
        description: rule.description.clone(),
        values: rule.values.clone(),
        items: rule.items.as_deref().map(mandatory_field_view).map(Box::new),
        fields: rule.fields.iter().map(|(field, rule)| (field.clone(), mandatory_field_view(rule))).collect(),
    }
}

fn value_type_name(value_type: &ValueType) -> &'static str {
    match value_type {
        ValueType::String => "string",
        ValueType::Date => "date",
        ValueType::Datetime => "datetime",
        ValueType::Boolean => "boolean",
        ValueType::Number => "number",
        ValueType::Array => "array",
        ValueType::Object => "object",
    }
}
