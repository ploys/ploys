use std::convert::Infallible;

use anyhow::{Error, bail};
use clap::Args;
use ploys::client::{Client, ServAddr, Token};
use ploys::repository::RepoAddr;
use semver::Version;

use crate::auth::init_keyring;

/// The changelog command.
#[derive(Args)]
pub struct Changelog {
    /// The repository address (owner/name) or GitHub URL.
    repo: RepoAddr,

    /// The package identifier.
    package: String,

    /// Query the specified version.
    #[arg(long, conflicts_with_all = ["latest", "unreleased"])]
    version: Option<Version>,

    /// Query only the latest changes.
    #[arg(long, conflicts_with_all = ["version", "unreleased"])]
    latest: bool,

    /// Query only the unreleased changes.
    #[arg(long, conflicts_with_all = ["version", "latest"])]
    unreleased: bool,

    /// The management server address.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Changelog {
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
