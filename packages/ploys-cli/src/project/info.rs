use anyhow::Error;
use clap::Args;
use console::style;
use ploys::client::{Client, Credentials, Token};
use ploys::repository::RepoAddr;

/// Gets the project information.
#[derive(Args)]
pub struct Info {
    /// The repository address (owner/name) or GitHub URL.
    repo: RepoAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Info {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let client = match self.token {
            Some(token) => Client::build()
                .with_credentials(Credentials::new().with_access_token(token))
                .finished()?,
            None => Client::build().finished()?,
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
