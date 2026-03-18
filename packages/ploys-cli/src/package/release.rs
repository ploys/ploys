use std::convert::Infallible;

use anyhow::{Context, Error};
use clap::Args;
use ploys::client::{Client, Credentials, Token};
use ploys::package::BumpOrVersion;
use ploys::project::Project;
use ploys::repository::RepoAddr;

/// The release command.
#[derive(Args)]
pub struct Release {
    /// The package identifier.
    package: String,

    /// The package version or level (major, minor, patch, rc, beta, alpha).
    version: BumpOrVersion,

    /// The remote GitHub repository owner/name or URL.
    #[clap(long)]
    remote: Option<RepoAddr>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Token,
}

impl Release {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let repo = match self.remote {
            Some(repo) => repo,
            None => Project::git(".")?
                .repository()
                .context("Missing remote repository")?,
        };

        let credentials = Credentials::new().with_access_token(self.token);
        let client = Client::new().with_credentials(credentials);
        let project = client.get_project(repo)?;

        project
            .get_package(&self.package)
            .ok_or(ploys::package::Error::<Infallible>::NotFound(self.package))?
            .request_release(self.version)?;

        Ok(())
    }
}
