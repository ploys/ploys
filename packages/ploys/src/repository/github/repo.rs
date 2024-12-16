use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Method;

use super::{Error, GitHubRepoSpec};

/// The GitHub repository information.
#[derive(Clone, Debug)]
pub struct Repository {
    spec: GitHubRepoSpec,
    client: Client,
}

impl Repository {
    /// Constructs a new repository.
    pub(crate) fn new(spec: impl Into<GitHubRepoSpec>) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static("ploys/ploys"));

        Ok(Self {
            spec: spec.into(),
            client: Client::builder().default_headers(headers).build()?,
        })
    }

    /// Gets the repository owner.
    pub fn owner(&self) -> &str {
        self.spec.owner()
    }

    /// Gets the repository name.
    pub fn name(&self) -> &str {
        self.spec.repo()
    }

    /// Validates whether the remote repository exists.
    pub(super) fn validate(&self, token: Option<&str>) -> Result<(), Error> {
        self.head("", token).send()?.error_for_status()?;

        Ok(())
    }
}

impl Repository {
    /// Gets the API endpoint.
    pub(super) fn endpoint<P>(&self, path: P) -> String
    where
        P: AsRef<str>,
    {
        match path.as_ref() {
            "" => format!("https://api.github.com/repos/{}", self.spec),
            path => format!(
                "https://api.github.com/repos/{}/{}",
                self.spec,
                path.trim_start_matches('/')
            ),
        }
    }

    /// Creates a HTTP request.
    pub(super) fn request<P>(&self, method: Method, path: P, token: Option<&str>) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        let mut request = self.client.request(method, self.endpoint(path));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        request
    }

    /// Creates a HEAD request.
    pub(super) fn head<P>(&self, path: P, token: Option<&str>) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::HEAD, path, token)
    }

    /// Creates a GET request.
    pub(super) fn get<P>(&self, path: P, token: Option<&str>) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::GET, path, token)
    }

    /// Creates a POST request.
    pub(super) fn post<P>(&self, path: P, token: Option<&str>) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::POST, path, token)
    }

    /// Creates a PATCH request.
    pub(super) fn patch<P>(&self, path: P, token: Option<&str>) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::PATCH, path, token)
    }

    /// Creates a GraphQL HTTP request.
    pub(super) fn graphql(&self, token: Option<&str>) -> RequestBuilder {
        let mut request = self.client.post("https://api.github.com/graphql");

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        request
    }
}
