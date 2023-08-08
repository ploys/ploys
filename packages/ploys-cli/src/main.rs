mod command;

use anyhow::Error;
use clap::Parser;

use self::command::Command;

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
        self.command.exec()
    }
}

fn main() -> Result<(), Error> {
    Args::parse().exec()
}
