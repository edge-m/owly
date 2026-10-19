use clap::{CommandFactory, Parser, Subcommand};

use crate::commands::{config, graph, index, init, inspect, onboard, overview, schema, search, skill, validate};

#[derive(Debug, Parser)]
#[command(name = "owly", about = "Owly wiki validator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Validate the entire wiki and print a summary")]
    Validate(validate::Args),
    #[command(subcommand, about = "Inspect wiki structure and types")]
    Overview(OverviewCommand),
    #[command(about = "Create a starter configuration")]
    Init(init::Args),
    #[command(about = "Build or update the SQLite search index")]
    Index(index::Args),
    #[command(about = "Inspect the complete wiki state")]
    Inspect(inspect::Args),
    #[command(subcommand, about = "Inspect configuration")]
    Config(ConfigCommand),
    #[command(about = "Export the wiki knowledge graph")]
    Graph(graph::Args),
    #[command(about = "Describe the wiki schema")]
    Schema(schema::Args),
    #[command(about = "Explain how to start using Owly with an Agent")]
    Onboard(onboard::Args),
    #[command(about = "Write the Owly Agent Skill")]
    Skill(skill::Args),
    #[command(about = "Search structured frontmatter")]
    Search(search::Args),
}

#[derive(Debug, Subcommand)]
enum OverviewCommand {
    #[command(about = "Show configured directory structure")]
    Directories(overview::DirectoryArgs),
    #[command(about = "Show configured document types")]
    Types(overview::TypeArgs),
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    #[command(about = "Validate the configuration file")]
    Validate(config::ValidateArgs),
}

pub fn run(args: Vec<String>) -> Result<u8, String> {
    let cli = match Cli::try_parse_from(std::iter::once("owly".to_string()).chain(args)) {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = error.exit_code();
            error.print().map_err(|error| format!("cannot print CLI error: {error}"))?;
            return Ok(exit_code as u8);
        }
    };

    match cli.command {
        Some(Command::Validate(args)) => validate::run(&args),
        Some(Command::Overview(OverviewCommand::Directories(args))) => overview::directories(&args),
        Some(Command::Overview(OverviewCommand::Types(args))) => overview::types(&args),
        Some(Command::Init(args)) => init::run(&args),
        Some(Command::Index(args)) => index::run(&args),
        Some(Command::Inspect(args)) => inspect::run(&args),
        Some(Command::Config(ConfigCommand::Validate(args))) => config::validate(&args),
        Some(Command::Graph(args)) => graph::run(&args),
        Some(Command::Schema(args)) => schema::run(&args),
        Some(Command::Onboard(args)) => onboard::run(&args),
        Some(Command::Skill(args)) => skill::run(&args),
        Some(Command::Search(args)) => search::run(&args),
        None => {
            print_help();
            Ok(0)
        }
    }
}

fn print_help() {
    println!("{}", Cli::command().render_help());
}
