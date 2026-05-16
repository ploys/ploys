use anyhow::Error;
use clap::Args;
use ploys::client::{Client, ServAddr, Token};

use super::init_keyring;

/// The `auth login` command.
#[derive(Args)]
pub struct Login {
    /// The project management server.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Login {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match self.token {
            Some(token) => {
                Client::build()
                    .with_server(self.server)
                    .with_access_token_flow(token)
                    .with_keyring_store(init_keyring()?)
                    .finished()?
                    .login()?;
            }
            None => {
                Client::build()
                    .with_server(self.server)
                    .with_device_code_flow()
                    .with_keyring_store(init_keyring()?)
                    .finished()?
                    .login()?;
            }
        }

        Ok(())
    }
}
