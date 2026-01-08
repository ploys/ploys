use std::convert::Infallible;

use anyhow::{Context, Error, bail};
use clap::Args;
use ploys::project::Project;
use ploys::repository::RepoSpec;
use semver::Version;

/// The changelog command.
#[derive(Args)]
pub struct Changelog {
    /// The package identifier.
    package: String,

    /// The package version.
    #[clap(long)]
    version: Option<Version>,

    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoSpec>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: String,
}

impl Changelog {
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
        let package = project
            .get_package(&self.package)
            .ok_or(ploys::package::Error::<Infallible>::NotFound(self.package))?;

        let release = match self.version {
            Some(version) => match package.changelog() {
                Some(changelog) => match changelog.get_release(version.to_string()) {
                    Some(release) => release.to_owned(),
                    None => package.build_release_notes(&version)?,
                },
                None => package.build_release_notes(&version)?,
            },
            None => {
                let version = Version::new(package.version().major + 1, 0, 0);

                package
                    .build_release_notes(&version)?
                    .with_version("Unreleased")
            }
        };

        println!("{release:#}");

        Ok(())
    }
}
