use anyhow::Error;
use clap::Args;
use ploys::client::{Client, ServAddr, Token};
use time::format_description::well_known::Rfc2822;

use super::init_keyring;

/// The `auth status` command.
#[derive(Args)]
pub struct Status {
    /// The project management server.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Status {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let client = match &self.token {
            Some(token) => Client::build()
                .with_server(self.server)
                .with_access_token_flow(token.clone())
                .finished()?,
            None => Client::build()
                .with_server(self.server)
                .with_refresh_token_flow()
                .with_keyring_store(init_keyring()?)
                .finished()?,
        };

        let credentials = client.login()?;

        print!("Logged in as {}", credentials.user());

        if let Some(expiry) = credentials.expiry() {
            print!(" until {}", expiry.format(&Rfc2822)?);
        }

        if self.token.is_some() {
            println!(" via token");
        } else {
            println!(" via keyring");
        }

        Ok(())
    }
}
