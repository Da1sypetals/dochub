mod commands;
mod config;
mod paths;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "dochub")]
#[command(about = "Manage local document hubs.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Add {
        skill_name: String,
        path: PathBuf,
    },
    Prune,
    Sanity,
    Cp {
        skill_name: String,
        dest: PathBuf,
    },
    Rm {
        skill_name: String,
    },
    #[command(visible_alias = "list")]
    Ls {
        skill_name: Option<String>,
    },
    /// Copy hub content into each configured skill-dir under dest (see skill-dir in hub.toml).
    #[command(name = "use")]
    Use {
        skill_name: String,
        dest: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Add { skill_name, path } => commands::add(&skill_name, &path),
        Command::Prune => commands::prune(),
        Command::Sanity => commands::sanity(),
        Command::Cp { skill_name, dest } => commands::cp(&skill_name, &dest),
        Command::Rm { skill_name } => commands::rm(&skill_name),
        Command::Ls { skill_name } => commands::ls(skill_name.as_deref()),
        Command::Use { skill_name, dest } => commands::hub_use(&skill_name, dest.as_deref()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
