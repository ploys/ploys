mod changelog;
mod init;
mod release;

use anyhow::Error;
use clap::{Args, Subcommand};

use self::changelog::Changelog;
use self::init::Init;
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
            Command::Init(init) => init.exec(),
            Command::Release(release) => release.exec(),
            Command::Changelog(changelog) => changelog.exec(),
        }
    }
}

/// The inner package command.
#[derive(Subcommand)]
enum Command {
    /// Initializes a new package.
    Init(Init),
    /// Creates a new release.
    Release(Release),
    /// Queries the package changelog.
    Changelog(Changelog),
}
