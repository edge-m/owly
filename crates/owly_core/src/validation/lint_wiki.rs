use chrono::{DateTime, NaiveDate};
use indexmap::IndexMap;
use serde_yaml::Value;

use crate::config::{Config, FieldRule, MandatoryFieldRule, ValueType};
use crate::diagnostic::Diagnostic;
use crate::document::{self, Document};
use crate::link_resolver::OutgoingTarget;
use crate::source;
use crate::workspace::ScanResult;

pub fn lint(scan: &ScanResult, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = scan.diagnostics.clone();
    diagnostics.extend(scan.documents.iter().flat_map(|document| lint_document(document, config)));
    diagnostics.extend(lint_links(scan));
    if config.source_tracking.enabled {
        diagnostics.extend(lint_sources(scan));
    }
    diagnostics
}

fn lint_sources(scan: &ScanResult) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for document in &scan.documents {
        let Some(items) = document.frontmatter.get(Value::String("sources".to_string())).and_then(Value::as_sequence)
        else {
            continue;
        };
        for item in items {
            let Some(mapping) = item.as_mapping() else {
                diagnostics.push(Diagnostic::error(
                    "invalid-source-record",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    "each source must be an object with path and sha256",
                ));
                continue;
            };
            let Some(path) = mapping.get(Value::String("path".to_string())).and_then(Value::as_str) else {
                diagnostics.push(Diagnostic::error(
                    "invalid-source-record",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    "source record must contain a string path",
                ));
                continue;
            };
            let Some(hash) = mapping.get(Value::String("sha256".to_string())).and_then(Value::as_str) else {
                diagnostics.push(Diagnostic::error(
                    "invalid-source-record",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    "source record must contain a string sha256",
                ));
                continue;
            };
            if let Err(message) = source::normalize_path(path) {
                diagnostics.push(Diagnostic::error(
                    "invalid-source-record",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    message,
                ));
            } else if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                diagnostics.push(Diagnostic::error(
                    "invalid-source-record",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    "source sha256 must be a 64-character hexadecimal hash",
                ));
            }
        }
    }
    diagnostics
}

/// Verify source existence and recorded SHA-256 values against the resolved wiki root.
pub fn lint_sources_at_root(scan: &ScanResult, wiki_root: &std::path::Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for document in &scan.documents {
        let Some(items) = document.frontmatter.get(Value::String("sources".to_string())).and_then(Value::as_sequence)
        else {
            continue;
        };
        for item in items {
            let Some(mapping) = item.as_mapping() else { continue };
            let Some(path) = mapping.get(Value::String("path".to_string())).and_then(Value::as_str) else { continue };
            let Some(recorded_hash) = mapping.get(Value::String("sha256".to_string())).and_then(Value::as_str) else {
                continue;
            };
            let Ok(normalized_path) = source::normalize_path(path) else { continue };
            let full_path = wiki_root.join(&normalized_path);
            if !full_path.is_file() {
                diagnostics.push(Diagnostic::error(
                    "missing-source",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    format!("source file does not exist: {normalized_path}"),
                ));
                continue;
            }
            match source::sha256_file(&full_path) {
                Ok(actual_hash) if actual_hash != recorded_hash => diagnostics.push(Diagnostic::error(
                    "source-drift",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    format!(
                        "source hash differs for {normalized_path} (recorded {recorded_hash}, current {actual_hash})"
                    ),
                )),
                Err(message) => diagnostics.push(Diagnostic::error(
                    "missing-source",
                    Some(document.relative_file_path_from_wiki_root.clone()),
                    message,
                )),
                _ => {}
            }
        }
    }
    diagnostics
}

fn lint_links(scan: &ScanResult) -> Vec<Diagnostic> {
    let pages = scan.documents.iter().map(|document| document.page_id()).collect::<std::collections::HashSet<_>>();
    let mut diagnostics = Vec::new();
    for document in &scan.documents {
        let links = document.outgoing_links();
        for link in links.links {
            if let OutgoingTarget::Page { id } = link.target
                && !pages.contains(&id)
            {
                diagnostics.push(Diagnostic {
                    code: "broken-link".to_string(),
                    severity: crate::diagnostic::Severity::Error,
                    path: Some(document.relative_file_path_from_wiki_root.clone()),
                    line: Some(link.line),
                    column: Some(link.column),
                    message: format!("page link points to missing page '{id}'"),
                });
            }
        }
    }
    diagnostics
}

fn lint_document(document: &Document, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    lint_mandatory_fields(
        &document.relative_file_path_from_wiki_root,
        &document.frontmatter,
        &config.mandatory_fields,
        config.source_tracking.enabled,
        &mut diagnostics,
    );

    if let Some(document_type) = document::string_value(&document.frontmatter, "type") {
        match config.types.get(&document_type) {
            Some(type_config) => lint_fields(
                &document.relative_file_path_from_wiki_root,
                &document.frontmatter,
                &type_config.fields,
                config.source_tracking.enabled,
                &mut diagnostics,
            ),
            None if !config.types.is_empty() => diagnostics.push(Diagnostic::error(
                "unknown-document-type",
                Some(document.relative_file_path_from_wiki_root.clone()),
                format!("unknown document type '{document_type}'"),
            )),
            None => {}
        }
    }

    diagnostics
}

fn lint_fields(
    path: &str, frontmatter: &serde_yaml::Mapping, rules: &IndexMap<String, FieldRule>, ignore_sources: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (field, rule) in rules {
        if ignore_sources && field == "sources" {
            continue;
        }
        match frontmatter.get(Value::String(field.clone())) {
            None if !rule.optional => diagnostics.push(Diagnostic::error(
                "missing-required-field",
                Some(path.to_string()),
                format!("required field '{field}' is missing"),
            )),
            Some(value) => lint_value(path, field, value, rule, diagnostics),
            None => {}
        }
    }
}

fn lint_mandatory_fields(
    path: &str, frontmatter: &serde_yaml::Mapping, rules: &IndexMap<String, MandatoryFieldRule>, ignore_sources: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (field, rule) in rules {
        if ignore_sources && field == "sources" {
            continue;
        }
        match frontmatter.get(Value::String(field.clone())) {
            None => diagnostics.push(Diagnostic::error(
                "missing-required-field",
                Some(path.to_string()),
                format!("required field '{field}' is missing"),
            )),
            Some(value) => lint_mandatory_value(path, field, value, rule, diagnostics),
        }
    }
}

fn lint_value(path: &str, field: &str, value: &Value, rule: &FieldRule, diagnostics: &mut Vec<Diagnostic>) {
    if !matches_type(value, &rule.value_type) {
        diagnostics.push(Diagnostic::error(
            "invalid-field-type",
            Some(path.to_string()),
            format!("field '{field}' has an invalid value type"),
        ));
        return;
    }
    lint_allowed_values(path, field, value, &rule.values, diagnostics);
    if let Some(item_rule) = &rule.items {
        for item in value.as_sequence().into_iter().flatten() {
            lint_value(path, field, item, item_rule, diagnostics);
        }
    }
    if let Some(mapping) = value.as_mapping() {
        lint_fields(&format!("{path}.{field}"), mapping, &rule.fields, false, diagnostics);
    }
}

fn lint_mandatory_value(
    path: &str, field: &str, value: &Value, rule: &MandatoryFieldRule, diagnostics: &mut Vec<Diagnostic>,
) {
    if !matches_type(value, &rule.value_type) {
        diagnostics.push(Diagnostic::error(
            "invalid-field-type",
            Some(path.to_string()),
            format!("field '{field}' has an invalid value type"),
        ));
        return;
    }
    lint_allowed_values(path, field, value, &rule.values, diagnostics);
    if let Some(item_rule) = &rule.items {
        for item in value.as_sequence().into_iter().flatten() {
            lint_mandatory_value(path, field, item, item_rule, diagnostics);
        }
    }
    if let Some(mapping) = value.as_mapping() {
        lint_mandatory_fields(&format!("{path}.{field}"), mapping, &rule.fields, false, diagnostics);
    }
}

fn lint_allowed_values(path: &str, field: &str, value: &Value, allowed: &[String], diagnostics: &mut Vec<Diagnostic>) {
    if !allowed.is_empty() {
        let valid = value.as_str().is_some_and(|value| allowed.iter().any(|allowed| allowed == value));
        if !valid {
            diagnostics.push(Diagnostic::error(
                "invalid-field-value",
                Some(path.to_string()),
                format!("field '{field}' has a value outside the configured values"),
            ));
        }
    }
}

fn matches_type(value: &Value, expected: &ValueType) -> bool {
    match expected {
        ValueType::String => value.is_string(),
        ValueType::Date => value.as_str().is_some_and(is_valid_date),
        ValueType::Datetime => value.as_str().is_some_and(is_valid_datetime),
        ValueType::Boolean => value.is_bool(),
        ValueType::Number => value.is_number(),
        ValueType::Array => value.is_sequence(),
        ValueType::Object => value.is_mapping(),
    }
}

fn is_valid_date(value: &str) -> bool {
    value.len() == 10
        && NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|date| date.format("%Y-%m-%d").to_string() == value)
            .unwrap_or(false)
}

fn is_valid_datetime(value: &str) -> bool {
    value.ends_with('Z') && DateTime::parse_from_rfc3339(value).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_and_datetime_require_the_expected_formats() {
        assert!(matches_type(&Value::String("2026-08-12".into()), &ValueType::Date));
        assert!(!matches_type(&Value::String("2026-8-12".into()), &ValueType::Date));
        assert!(matches_type(&Value::String("2026-08-12T10:30:00Z".into()), &ValueType::Datetime));
        assert!(matches_type(&Value::String("2026-08-12T10:30:00.123Z".into()), &ValueType::Datetime));
        assert!(!matches_type(&Value::String("2026-08-12T19:30:00+09:00".into()), &ValueType::Datetime));
    }
}
