use std::convert::Infallible;

use anyhow::{Context, Error};
use clap::Args;
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
    token: String,
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

        let project = Project::github_with_authentication_token(repo, self.token)?;

        project
            .get_package(&self.package)
            .ok_or(ploys::package::Error::<Infallible>::NotFound(self.package))?
            .request_release(self.version)?;

        Ok(())
    }
}
