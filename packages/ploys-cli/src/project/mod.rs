mod info;

use anyhow::Error;
use clap::{Args, Subcommand};

use self::info::Info;

/// The project command.
#[derive(Args)]
pub struct Project {
    #[clap(subcommand)]
    command: Command,
}

impl Project {
    /// Executes the project command.
    pub fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Info(info) => info.exec(),
        }
    }
}

/// The inner project command.
#[derive(Subcommand)]
enum Command {
    /// Gets the project information.
    Info(Info),
}
