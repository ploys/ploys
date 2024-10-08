use anyhow::Error;
use clap::Args;
use ploys::package::BumpOrVersion;
use ploys::project::source::github::GitHub;
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
    #[clap(long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Release {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match &self.remote {
            Some(remote) => match &self.token {
                Some(token) => {
                    let mut project = Project::<GitHub>::github_with_authentication_token(
                        remote.clone().try_into_repo()?.to_string(),
                        token,
                    )?;

                    project.release_package(self.package, self.version)?;

                    Ok(())
                }
                None => {
                    let mut project = Project::github(remote.clone().try_into_repo()?.to_string())?;

                    project.release_package(self.package, self.version)?;

                    Ok(())
                }
            },
            None => {
                let mut project = Project::git(".")?;

                project.release_package(self.package, self.version)?;

                Ok(())
            }
        }
    }
}
