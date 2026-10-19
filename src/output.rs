use owly_core::diagnostic::Diagnostic;

use crate::commands::validate;

pub fn print_diagnostics(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        println!("OK: no issues found");
    } else {
        println!("Found {} diagnostic(s):", diagnostics.len());
        println!();
        for diagnostic in diagnostics {
            println!("{}: [{}]", severity_name(diagnostic), diagnostic.code);
            if let Some(path) = diagnostic.path.as_deref() {
                println!("  path: {path}");
            }
            println!("  message: {}", diagnostic.message);
            if let Some(help) = diagnostic_help(diagnostic) {
                println!("  help: {help}");
            }
            println!();
        }
    }
}

pub fn print_validation(diagnostics: &[Diagnostic], details: bool, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&validate::json_output(diagnostics, details))
                .expect("validation output should be serializable")
        );
        return;
    }

    let summary = validate::summary(diagnostics);
    println!("Validation summary");
    println!("  errors: {}", summary.errors);
    println!("  warnings: {}", summary.warnings);
    println!("  infos: {}", summary.infos);
    if summary.by_code.is_empty() {
        println!("  diagnostics: none");
    } else {
        println!("  diagnostics by code:");
        for (code, count) in &summary.by_code {
            println!("    {code}: {count}");
        }
    }

    if details && !diagnostics.is_empty() {
        println!();
        print_diagnostics(diagnostics);
    }
}

fn severity_name(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic.severity {
        owly_core::diagnostic::Severity::Error => "error",
        owly_core::diagnostic::Severity::Warning => "warning",
        owly_core::diagnostic::Severity::Info => "info",
    }
}

fn diagnostic_help(diagnostic: &Diagnostic) -> Option<&'static str> {
    match diagnostic.code.as_str() {
        "config-values-require-string" => Some("use 'values' only with type: string"),
        "config-items-require-array" => Some("use 'items' only with type: array"),
        "config-fields-require-object" => Some("use 'fields' only with type: object"),
        "config-owly-version-missing" => Some("add owly_version: 0.1.1 to the configuration"),
        "config-owly-version-incompatible" => Some("set owly_version to a version supported by this binary"),
        "source-definition-ignored" => Some("set source_tracking.enabled to false to use a custom sources definition"),
        _ => None,
    }
}
