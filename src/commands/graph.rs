use std::path::PathBuf;

use clap::Args as ClapArgs;

use super::context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "YAML configuration file")]
    pub config: Option<PathBuf>,
}

pub fn run(args: &Args) -> Result<u8, String> {
    let config_context = context::load(args.config.as_deref())?;
    let scanned = owly_core::workspace::Workspace::new(config_context.wiki_root().to_path_buf())
        .with_excludes(&config_context.config().wiki_lint.exclude)
        .scan()?;
    let graph = owly_core::graph::WikiGraph::from_scan(&scanned);
    let json = serde_json::to_string_pretty(&graph).map_err(|error| format!("cannot serialize graph: {error}"))?;
    println!("{json}");
    Ok(0)
}
