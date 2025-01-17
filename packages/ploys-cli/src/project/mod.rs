mod info;
mod init;

use anyhow::Error;
use clap::{Args, Subcommand};

use self::info::Info;
use self::init::Init;

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
            Command::Init(init) => init.exec(),
        }
    }
}

/// The inner project command.
#[derive(Subcommand)]
enum Command {
    /// Gets the project information.
    Info(Info),
    /// Initializes a new project.
    Init(Init),
}
