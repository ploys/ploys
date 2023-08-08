mod inspect;

use anyhow::Error;
use clap::Subcommand;

use self::inspect::Inspect;

/// The base command.
#[derive(Subcommand)]
pub enum Command {
    /// Inspects the project.
    Inspect(Inspect),
}

impl Command {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Self::Inspect(inspect) => inspect.exec(),
        }
    }
}
