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
        name: String,
        path: PathBuf,
    },
    Prune,
    Sanity,
    Cp {
        name: String,
        dest: PathBuf,
    },
    Rm {
        name: String,
    },
    #[command(visible_alias = "list")]
    Ls {
        name: Option<String>,
    },
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCommand {
    Cp { name: String, dest: Option<PathBuf> },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Add { name, path } => commands::add(&name, &path),
        Command::Prune => commands::prune(),
        Command::Sanity => commands::sanity(),
        Command::Cp { name, dest } => commands::cp(&name, &dest),
        Command::Rm { name } => commands::rm(&name),
        Command::Ls { name } => commands::ls(name.as_deref()),
        Command::Skill { command } => match command {
            SkillCommand::Cp { name, dest } => commands::skill_cp(&name, dest.as_deref()),
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
