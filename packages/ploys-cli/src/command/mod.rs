use anyhow::Error;
use clap::Subcommand;

/// The base command.
#[derive(Subcommand)]
pub enum Command {}

impl Command {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match self {}
    }
}
