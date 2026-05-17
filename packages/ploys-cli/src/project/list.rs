use anyhow::Error;
use clap::Args;
use console::style;
use ploys::client::{Client, ServAddr, Token};

use crate::auth::init_keyring;

/// The `project list` command.
#[derive(Args)]
pub struct List {
    /// The project management server.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl List {
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

        client.login()?;

        println!("{}:\n", style("Projects").underlined().bold());

        let projects = client.projects().flatten().collect::<Vec<_>>();

        let max_name_len = projects
            .iter()
            .map(|project| project.name().len())
            .max()
            .unwrap_or_default();

        for project in projects {
            println!(
                "{:<max_name_len$}  {}",
                project.name(),
                project.description().unwrap_or_default()
            );
        }

        Ok(())
    }
}
