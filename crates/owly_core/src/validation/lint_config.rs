use crate::config::{Config, FieldRule, MandatoryFieldRule, ValueType};
use crate::diagnostic::Diagnostic;
use indexmap::IndexMap;

pub fn lint(config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    lint_mandatory_fields(
        "mandatory_fields",
        &config.mandatory_fields,
        config.source_tracking.enabled,
        &mut diagnostics,
    );

    for (document_type, type_config) in &config.types {
        lint_fields(
            &format!("types.{document_type}.fields"),
            &type_config.fields,
            config.source_tracking.enabled,
            &mut diagnostics,
        );
    }

    diagnostics
}

fn lint_mandatory_fields(
    scope: &str, fields: &IndexMap<String, MandatoryFieldRule>, ignore_sources: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (field, rule) in fields {
        if ignore_sources && field == "sources" {
            diagnostics.push(Diagnostic::warning(
                "source-definition-ignored",
                Some(format!("{scope}.{field}")),
                "source_tracking is enabled; the custom sources definition is ignored in favor of the standard source record",
            ));
            continue;
        }
        let field_scope = format!("{scope}.{field}");
        lint_mandatory_rule(&field_scope, rule, diagnostics);
    }
}

fn lint_mandatory_rule(scope: &str, rule: &MandatoryFieldRule, diagnostics: &mut Vec<Diagnostic>) {
    lint_rule_shape(scope, &rule.value_type, &rule.values, rule.items.is_some(), !rule.fields.is_empty(), diagnostics);
    if let Some(items) = &rule.items {
        lint_mandatory_rule(&format!("{scope}.items"), items, diagnostics);
    }
    lint_mandatory_fields(&format!("{scope}.fields"), &rule.fields, false, diagnostics);
}

fn lint_fields(
    scope: &str, fields: &IndexMap<String, FieldRule>, ignore_sources: bool, diagnostics: &mut Vec<Diagnostic>,
) {
    for (field, rule) in fields {
        if ignore_sources && field == "sources" {
            diagnostics.push(Diagnostic::warning(
                "source-definition-ignored",
                Some(format!("{scope}.{field}")),
                "source_tracking is enabled; the custom sources definition is ignored in favor of the standard source record",
            ));
            continue;
        }
        let field_scope = format!("{scope}.{field}");
        lint_rule(&field_scope, rule, diagnostics);
    }
}

fn lint_rule(scope: &str, rule: &FieldRule, diagnostics: &mut Vec<Diagnostic>) {
    lint_rule_shape(scope, &rule.value_type, &rule.values, rule.items.is_some(), !rule.fields.is_empty(), diagnostics);
    if let Some(items) = &rule.items {
        lint_rule(&format!("{scope}.items"), items, diagnostics);
    }
    lint_fields(&format!("{scope}.fields"), &rule.fields, false, diagnostics);
}

fn lint_rule_shape(
    scope: &str, value_type: &ValueType, values: &[String], has_items: bool, has_fields: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !values.is_empty() && !matches!(value_type, ValueType::String) {
        diagnostics.push(Diagnostic::error(
            "config-values-require-string",
            Some(scope.to_string()),
            "allowed values are currently supported only for string fields",
        ));
    }
    if has_items && !matches!(value_type, ValueType::Array) {
        diagnostics.push(Diagnostic::error(
            "config-items-require-array",
            Some(scope.to_string()),
            "items are supported only for array fields",
        ));
    }
    if has_fields && !matches!(value_type, ValueType::Object) {
        diagnostics.push(Diagnostic::error(
            "config-fields-require-object",
            Some(scope.to_string()),
            "fields are supported only for object fields",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_array_and_object_rules_are_linted() {
        let mut config = Config::default();
        config.mandatory_fields.insert(
            "tags".to_string(),
            MandatoryFieldRule {
                value_type: ValueType::Array,
                description: None,
                values: Vec::new(),
                items: Some(Box::new(MandatoryFieldRule {
                    value_type: ValueType::String,
                    description: None,
                    values: Vec::new(),
                    items: None,
                    fields: IndexMap::new(),
                })),
                fields: IndexMap::new(),
            },
        );

        assert!(lint(&config).is_empty());
    }
}
