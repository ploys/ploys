mod auth;
mod package;
mod project;

use anyhow::Error;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

use self::auth::Auth;
use self::package::Package;
use self::project::Project;

/// Manage projects, packages, releases and deployments.
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
    fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Project(project) => project.exec(),
            Command::Package(package) => package.exec(),
            Command::Auth(auth) => auth.exec(),
        }
    }
}

/// The base command.
#[derive(Subcommand)]
enum Command {
    /// Manages the project.
    Project(Project),
    /// Manages the packages.
    Package(Package),
    /// Authenticate with GitHub.
    Auth(Auth),
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.verbose.tracing_level_filter())
        .pretty()
        .init();

    args.exec()
}
