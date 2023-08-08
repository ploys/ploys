use anyhow::Error;
use clap::{Parser, Subcommand};

/// Manage projects, packages, releases and deployments.
#[derive(Parser)]
#[command(name = "ploys", version, arg_required_else_help = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

impl Args {
    fn exec(self) -> Result<(), Error> {
        match self.command {}
    }
}

#[derive(Subcommand)]
enum Command {}

fn main() -> Result<(), Error> {
    Args::parse().exec()
}
