use ureq::Request;

use super::{Error, GitHubRepoSpec};

/// The GitHub repository information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    spec: GitHubRepoSpec,
}

impl Repository {
    /// Constructs a new repository.
    pub(crate) fn new(spec: impl Into<GitHubRepoSpec>) -> Self {
        Self { spec: spec.into() }
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
        self.head("", token).call()?;

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
    pub(super) fn request<P>(&self, method: &str, path: P, token: Option<&str>) -> Request
    where
        P: AsRef<str>,
    {
        let mut request =
            ureq::request(method, &self.endpoint(path)).set("User-Agent", "ploys/ploys");

        if let Some(token) = &token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }

        request
    }

    /// Creates a HEAD request.
    pub(super) fn head<P>(&self, path: P, token: Option<&str>) -> Request
    where
        P: AsRef<str>,
    {
        self.request("HEAD", path, token)
    }

    /// Creates a GET request.
    pub(super) fn get<P>(&self, path: P, token: Option<&str>) -> Request
    where
        P: AsRef<str>,
    {
        self.request("GET", path, token)
    }

    /// Creates a POST request.
    pub(super) fn post<P>(&self, path: P, token: Option<&str>) -> Request
    where
        P: AsRef<str>,
    {
        self.request("POST", path, token)
    }

    /// Creates a PATCH request.
    pub(super) fn patch<P>(&self, path: P, token: Option<&str>) -> Request
    where
        P: AsRef<str>,
    {
        self.request("PATCH", path, token)
    }

    /// Creates a GraphQL HTTP request.
    pub(super) fn graphql(&self, token: Option<&str>) -> Request {
        let mut request =
            ureq::post("https://api.github.com/graphql").set("User-Agent", "ploys/ploys");

        if let Some(token) = &token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }

        request
    }
}
