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

    /// Query the specified version.
    #[clap(long, conflicts_with_all = ["latest", "unreleased"])]
    version: Option<Version>,

    /// Query only the latest changes.
    #[clap(long, conflicts_with_all = ["version", "unreleased"])]
    latest: bool,

    /// Query only the unreleased changes.
    #[clap(long, conflicts_with_all = ["version", "latest"])]
    unreleased: bool,

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

        match self.version {
            Some(version) => {
                let Some(changelog) = package.changelog() else {
                    bail!("Missing changelog");
                };

                let Some(release) = changelog.get_release(version.to_string()) else {
                    bail!("Invalid version {version}");
                };

                println!("{release:#}");
            }
            None if self.latest => {
                let Some(changelog) = package.changelog() else {
                    bail!("Missing changelog");
                };

                let Some(release) = changelog.releases().next() else {
                    bail!("Missing release");
                };

                println!("{release:#}");
            }
            None if self.unreleased => {
                let version = Version::new(package.version().major + 1, 0, 0);
                let release = package
                    .build_release_notes(&version)?
                    .with_version("Unreleased");

                println!("{release:#}");
            }
            None => {
                let Some(changelog) = package.changelog() else {
                    bail!("Missing changelog");
                };

                println!("{changelog:#}");
            }
        }

        Ok(())
    }
}
