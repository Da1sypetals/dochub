mod commands;
mod config;
mod paths;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "dochub")]
#[command(about = "Named local hubs from ~/.dochub/hub.toml")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Register a directory as a named hub in hub.toml
    Add {
        #[arg(help = "Label stored in hub.toml")]
        skill_name: String,
        #[arg(help = "Directory to register (must exist)")]
        path: PathBuf,
    },
    /// Drop hub entries whose paths are gone
    Prune,
    /// Report hubs larger than sane-size (MB) from hub.toml
    Sanity,
    /// Copy hub tree to <dest>/<skill_name>/content/
    Cp {
        #[arg(help = "Hub label to copy")]
        skill_name: String,
        #[arg(help = "Destination parent directory")]
        dest: PathBuf,
    },
    /// Remove a hub label (prompts for confirmation)
    Rm {
        #[arg(help = "Hub label to remove")]
        skill_name: String,
    },
    /// List hub labels and paths
    #[command(visible_alias = "list")]
    Ls {
        #[arg(help = "If set, show only this label")]
        skill_name: Option<String>,
    },
    /// Copy hub into each skill-dir under <dest>/<skill-dir>/skills/<skill_name> (dest defaults to .)
    #[command(name = "use")]
    Use {
        #[arg(help = "Hub label to copy")]
        skill_name: String,
        #[arg(help = "Project root; default: current directory")]
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
