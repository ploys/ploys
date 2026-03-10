mod credentials;
mod error;

use reqwest::blocking::Client as HttpClient;

use crate::project::{Error as ProjError, Project};
use crate::repository::RepoAddr;
use crate::repository::types::github::{Error as RepoError, GitHub};

pub use self::credentials::{Credentials, Token};
pub use self::error::Error;

/// The project management client.
#[derive(Clone, Debug)]
pub struct Client {
    credentials: Option<Credentials>,
    http_client: HttpClient,
}

impl Client {
    /// Constructs a new project management client.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            credentials: None,
            http_client: HttpClient::builder()
                .user_agent(concat!("ploys/", env!("CARGO_PKG_VERSION")))
                .build()?,
        })
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
    pub(crate) fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Gets the authentication credentials access token.
    pub(crate) fn get_access_token(&self) -> Option<Token> {
        match &self.credentials {
            Some(credentials) => credentials.get_access_token(),
            None => None,
        }
    }
}
