use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Args as ClapArgs, ValueEnum};
use serde::Serialize;

use crate::output;
use owly_core::{validation, workspace::Workspace};

use super::context;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "YAML configuration file")]
    pub config: Option<PathBuf>,
    #[arg(long, help = "Single wiki file to validate")]
    pub file: Option<PathBuf>,
    #[arg(long, help = "Include each diagnostic after the summary")]
    pub details: bool,
    #[arg(long, value_enum, default_value_t = Format::Human)]
    pub format: Format,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub by_code: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct JsonOutput<'a> {
    summary: Summary,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<&'a [owly_core::diagnostic::Diagnostic]>,
}

pub fn run(args: &Args) -> Result<u8, String> {
    run_internal(args, args.details)
}

fn run_internal(args: &Args, details: bool) -> Result<u8, String> {
    let config_context = context::load(args.config.as_deref())?;
    let config_diagnostics = validation::lint_config::lint(config_context.config());
    if config_diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        output::print_validation(&config_diagnostics, details, matches!(args.format, Format::Json));
        return Ok(2);
    }

    let mut diagnostics = config_diagnostics;
    let scanned = Workspace::new(config_context.wiki_root().to_path_buf())
        .with_excludes(&config_context.config().wiki_lint.exclude)
        .scan()?;
    diagnostics.extend(validation::lint_wiki::lint(&scanned, config_context.config()));
    if config_context.config().source_tracking.enabled {
        diagnostics.extend(validation::lint_wiki::lint_sources_at_root(&scanned, config_context.wiki_root()));
    }
    diagnostics.extend(validation::orphans::find(&scanned));

    if let Some(file) = &args.file {
        let target = file.to_string_lossy().replace('\\', "/");
        if !config_context.config().wiki_lint.is_excluded(&target)
            && !scanned.documents.iter().any(|document| document.relative_file_path_from_wiki_root == target)
            && !scanned.diagnostics.iter().any(|diagnostic| diagnostic.path.as_deref() == Some(target.as_str()))
        {
            diagnostics.push(owly_core::diagnostic::Diagnostic::error(
                "file-not-found",
                Some(target.clone()),
                "file does not exist or is not a Markdown document",
            ));
        }
        diagnostics.retain(|diagnostic| diagnostic.path.as_deref() == Some(target.as_str()));
    }

    output::print_validation(&diagnostics, details, matches!(args.format, Format::Json));
    Ok(if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) { 1 } else { 0 })
}

pub fn summary(diagnostics: &[owly_core::diagnostic::Diagnostic]) -> Summary {
    let mut by_code = BTreeMap::new();
    for diagnostic in diagnostics {
        *by_code.entry(diagnostic.code.clone()).or_insert(0) += 1;
    }
    Summary {
        errors: diagnostics.iter().filter(|diagnostic| diagnostic.is_error()).count(),
        warnings: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, owly_core::diagnostic::Severity::Warning))
            .count(),
        infos: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, owly_core::diagnostic::Severity::Info))
            .count(),
        by_code,
    }
}

pub fn json_output<'a>(diagnostics: &'a [owly_core::diagnostic::Diagnostic], details: bool) -> JsonOutput<'a> {
    JsonOutput { summary: summary(diagnostics), diagnostics: details.then_some(diagnostics) }
}
