mod github;
mod serve;
mod state;

use anyhow::Error;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

use self::serve::Serve;

/// Controls the API for managing projects, packages, releases and deployments.
#[derive(Parser)]
#[command(name = "ploys", version, arg_required_else_help = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    #[command(flatten)]
    verbose: Verbosity,
}

impl Args {
    /// Executes the program.
    async fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Serve(command) => command.exec().await,
        }
    }
}

/// The base command.
#[derive(Subcommand)]
enum Command {
    /// Serves the API.
    Serve(Serve),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.verbose.tracing_level_filter())
        .init();

    args.exec().await
}
