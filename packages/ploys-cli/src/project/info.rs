use anyhow::Error;
use clap::Args;
use console::style;
use ploys::client::{Client, ServAddr, Token};
use ploys::repository::RepoAddr;

use crate::auth::init_keyring;

/// Gets the project information.
#[derive(Args)]
pub struct Info {
    /// The repository address (owner/name) or GitHub URL.
    repo: RepoAddr,

    /// The management server address.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Info {
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

        println!("{}:\n", style("Project").underlined().bold());
        println!("Name:        {}", project.name());

        if let Some(description) = project.description() {
            println!("Description: {description}");
        }

        if let Some(repository) = project.repository() {
            println!("Repository:  {repository:#}");
        }

        println!("\n{}:\n", style("Packages").underlined().bold());

        let packages = project.packages().collect::<Vec<_>>();

        let max_name_len = packages
            .iter()
            .map(|pkg| pkg.name().len())
            .max()
            .unwrap_or_default();
        let max_version_len = packages
            .iter()
            .map(|pkg| pkg.version().to_string().len())
            .max()
            .unwrap_or_default();

        for package in packages.iter() {
            println!(
                "{:<max_name_len$}  {:>max_version_len$}  {}",
                package.name(),
                package.version(),
                package.description().unwrap_or_default()
            );
        }

        Ok(())
    }
}
