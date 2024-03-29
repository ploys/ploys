mod project;

use anyhow::Error;
use clap::{Parser, Subcommand};

use self::project::Project;

/// Manage projects, packages, releases and deployments.
#[derive(Parser)]
#[command(name = "ploys", version, arg_required_else_help = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

impl Args {
    /// Executes the program.
    fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Project(project) => project.exec(),
        }
    }
}

/// The base command.
#[derive(Subcommand)]
enum Command {
    /// Manages the project.
    Project(Project),
}

fn main() -> Result<(), Error> {
    Args::parse().exec()
}
