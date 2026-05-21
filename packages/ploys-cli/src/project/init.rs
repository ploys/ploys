use std::io::IsTerminal;

use anyhow::Error;
use clap::Args;
use dialoguer::Input;
use ploys::client::{Client, ServAddr, Token};
use ploys::repository::RepoAddr;
use ploys::repository::types::git::Git;

use crate::auth::init_keyring;

/// Initializes a new project.
#[derive(Args)]
pub struct Init {
    /// The repository address (owner/name) or GitHub URL.
    repo: RepoAddr,

    /// The project description.
    #[arg(long)]
    description: Option<String>,

    /// The project author.
    #[arg(long)]
    author: Vec<String>,

    /// The project visibility.
    #[arg(long)]
    private: bool,

    /// The management server address.
    #[arg(long, default_value = "api.ploys.dev")]
    server: ServAddr,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<Token>,
}

impl Init {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let is_terminal = std::io::stdin().is_terminal();
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

        let description = match self.description {
            Some(description) => Some(description),
            None if !is_terminal => None,
            None => {
                let description = Input::<String>::new()
                    .with_prompt("Description")
                    .allow_empty(true)
                    .interact_text()?;

                match description.is_empty() {
                    true => None,
                    false => Some(description),
                }
            }
        };

        let authors = match self.author.is_empty() {
            false => self.author,
            true if !is_terminal => Vec::new(),
            true => {
                let mut author = format!("The {} Project Developers", self.repo.name());

                if let Some(git_author) = Git::get_author() {
                    author = git_author;
                };

                let author = Input::<String>::new()
                    .with_prompt("Author")
                    .with_initial_text(author)
                    .allow_empty(true)
                    .interact_text()?;

                match author.is_empty() {
                    true => Vec::new(),
                    false => vec![author],
                }
            }
        };

        let mut builder = client.create_project(self.repo)?;

        if let Some(description) = description {
            builder = builder.with_description(description);
        }

        if !authors.is_empty() {
            builder = builder.with_authors(authors);
        }

        if self.private {
            builder = builder.with_private_visibility();
        }

        builder.finished()?;

        Ok(())
    }
}
