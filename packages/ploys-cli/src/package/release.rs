use std::convert::Infallible;

use anyhow::Error;
use clap::Args;
use ploys::client::{Client, ServAddr, Token};
use ploys::package::BumpOrVersion;
use ploys::repository::RepoAddr;

use crate::auth::init_keyring;

/// The release command.
#[derive(Args)]
pub struct Release {
    /// The repository address (owner/name) or GitHub URL.
    repo: RepoAddr,

    /// The package identifier.
    package: String,

    /// The package version or level (major, minor, patch, rc, beta, alpha).
    version: BumpOrVersion,

    /// The management server address.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Release {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let client = match self.token {
            Some(token) => Client::build()
                .with_server(self.server)
                .with_access_token_flow(token)
                .finished()?,
            None => Client::build()
                .with_server(self.server)
                .with_refresh_token_flow()
                .with_keyring_store(init_keyring()?)
                .finished()?,
        };

        let project = client.get_project(self.repo)?;

        project
            .get_package(&self.package)
            .ok_or(ploys::package::Error::<Infallible>::NotFound(self.package))?
            .request_release(self.version)?;

        Ok(())
    }
}
