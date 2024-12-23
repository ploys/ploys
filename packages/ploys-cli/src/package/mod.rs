mod release;

use anyhow::Error;
use clap::{Args, Subcommand};

use self::release::Release;

/// The package command.
#[derive(Args)]
pub struct Package {
    #[clap(subcommand)]
    command: Command,
}

impl Package {
    /// Executes the package command.
    pub fn exec(self) -> Result<(), Error> {
        match self.command {
            Command::Release(release) => release.exec(),
        }
    }
}

/// The inner package command.
#[derive(Subcommand)]
enum Command {
    /// Creates a new release.
    Release(Release),
}
