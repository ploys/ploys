use reqwest::Method;
use reqwest::blocking::RequestBuilder;

use crate::client::{Client, Credentials, Token};
use crate::repository::addr::RepoAddr;

use super::Error;

/// The GitHub repository information.
#[derive(Clone, Debug)]
pub struct Repo {
    addr: RepoAddr,
    client: Client,
}

impl Repo {
    /// Constructs a new repository.
    pub(crate) fn new(client: Client, addr: RepoAddr) -> Self {
        Self { addr, client }
    }

    /// Gets the repository owner.
    pub fn owner(&self) -> &str {
        self.addr.owner()
    }

    /// Gets the repository name.
    pub fn name(&self) -> &str {
        self.addr.name()
    }

    /// Sets the access token.
    pub(super) fn set_access_token(&mut self, token: impl Into<Token>) {
        self.client
            .set_credentials(Credentials::new().with_access_token(token));
    }

    /// Validates whether the remote repository exists.
    pub(super) fn validate(&self) -> Result<(), Error> {
        self.head("").send()?.error_for_status()?;

        Ok(())
    }
}

impl Repo {
    /// Gets the API endpoint.
    pub(super) fn endpoint<P>(&self, path: P) -> String
    where
        P: AsRef<str>,
    {
        match path.as_ref() {
            "" => format!("https://api.github.com/repos/{}", self.addr),
            path => format!(
                "https://api.github.com/repos/{}/{}",
                self.addr,
                path.trim_start_matches('/')
            ),
        }
    }

    /// Creates a HTTP request.
    pub(super) fn request<P>(&self, method: Method, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        let mut request = self
            .client
            .http_client()
            .request(method, self.endpoint(path));

        if let Some(token) = self.client.get_access_token() {
            request = request.bearer_auth(token);
        }

        request
    }

    /// Creates a HEAD request.
    pub(super) fn head<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::HEAD, path)
    }

    /// Creates a GET request.
    pub(super) fn get<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::GET, path)
    }

    /// Creates a POST request.
    pub(super) fn post<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::POST, path)
    }

    /// Creates a PATCH request.
    pub(super) fn patch<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::PATCH, path)
    }

    /// Creates a GraphQL HTTP request.
    pub(super) fn graphql(&self) -> RequestBuilder {
        let mut request = self
            .client
            .http_client()
            .post("https://api.github.com/graphql");

        if let Some(token) = self.client.get_access_token() {
            request = request.bearer_auth(token);
        }

        request
    }
}
