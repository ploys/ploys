use std::convert::Infallible;

use anyhow::{Context, Error, bail};
use clap::Args;
use ploys::package::BumpOrVersion;
use ploys::project::Project;
use ploys::repository::RepoSpec;

/// The release command.
#[derive(Args)]
pub struct Release {
    /// The package identifier.
    package: String,

    /// The package version or level (major, minor, patch, rc, beta, alpha).
    version: BumpOrVersion,

    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoSpec>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: String,
}

impl Release {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let remote = match self.remote {
            Some(remote) => remote,
            None => Project::git(".")?
                .repository()
                .context("Missing remote repository")?,
        };

        let Some(github) = remote.to_github() else {
            bail!("Unsupported remote repository: {remote}");
        };

        let project = Project::github_with_authentication_token(github.to_string(), self.token)?;

        project
            .get_package(&self.package)
            .ok_or(ploys::package::Error::<Infallible>::NotFound(self.package))?
            .request_release(self.version)?;

        Ok(())
    }
}
