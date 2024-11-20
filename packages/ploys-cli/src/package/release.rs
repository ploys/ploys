use anyhow::Error;
use clap::Args;
use ploys::package::BumpOrVersion;
use ploys::project::Project;

use crate::util::repo_or_url::RepoOrUrl;

/// The release command.
#[derive(Args)]
pub struct Release {
    /// The package identifier.
    package: String,

    /// The package version or level (major, minor, patch, rc, beta, alpha).
    version: BumpOrVersion,

    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoOrUrl>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: String,
}

impl Release {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let remote = match self.remote {
            Some(remote) => remote,
            None => {
                let project = Project::git(".")?;
                let url = project.get_url()?;

                RepoOrUrl::Url(url)
            }
        };

        let project = Project::github_with_authentication_token(
            remote.try_into_repo()?.to_string(),
            self.token,
        )?;

        project.request_package_release(self.package, self.version)?;

        Ok(())
    }
}
