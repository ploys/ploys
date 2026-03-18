mod credentials;
mod error;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use reqwest::blocking::Client as HttpClient;

use crate::project::{Error as ProjError, Project};
use crate::repository::RepoAddr;
use crate::repository::types::github::{Error as RepoError, GitHub};

pub use self::credentials::{Credentials, Token, TokenError};
pub use self::error::Error;

/// The project management client.
#[derive(Clone, Debug, Default)]
pub struct Client {
    credentials: Option<Credentials>,
    http_client: Arc<OnceCell<HttpClient>>,
}

impl Client {
    /// Constructs a new project management client.
    pub fn new() -> Self {
        Self {
            credentials: None,
            http_client: Arc::new(OnceCell::new()),
        }
    }

    /// Builds the client with the given authentication credentials.
    pub fn with_credentials(mut self, credentials: impl Into<Credentials>) -> Self {
        self.set_credentials(credentials);
        self
    }
}

impl Client {
    /// Gets a project with the given repository address.
    pub fn get_project<R>(&self, repo: R) -> Result<Project<GitHub>, Error>
    where
        R: TryInto<RepoAddr, Error: Into<RepoError>>,
    {
        let repo = GitHub::new(self.clone(), repo).map_err(ProjError::Repository)?;
        let proj = Project::open(repo)?;

        Ok(proj)
    }

    /// Sets the client authentication credentials.
    pub fn set_credentials(&mut self, credentials: impl Into<Credentials>) {
        self.credentials = Some(credentials.into());
    }
}

impl Client {
    /// Gets the HTTP client.
    pub(crate) fn http_client(&self) -> Result<&HttpClient, reqwest::Error> {
        self.http_client.get_or_try_init(|| {
            HttpClient::builder()
                .user_agent(concat!("ploys/", env!("CARGO_PKG_VERSION")))
                .build()
        })
    }

    /// Gets the authentication credentials access token.
    pub(crate) fn get_access_token(&self) -> Option<Token> {
        match &self.credentials {
            Some(credentials) => credentials.get_access_token(),
            None => None,
        }
    }
}
