use std::fmt::{self, Display};
use std::str::FromStr;

use ureq::Request;

use super::Error;

/// The GitHub repository information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    owner: String,
    repo: String,
}

impl Repository {
    /// Gets the repository name.
    pub fn name(&self) -> &str {
        &self.repo
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
            "" => format!("https://api.github.com/repos/{}", self),
            path => format!(
                "https://api.github.com/repos/{}/{}",
                self,
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
}

impl Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)?;

        Ok(())
    }
}

impl FromStr for Repository {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((owner, repo)) => match repo.contains('/') {
                true => Err(Error::Parse(format!("Invalid repository format `{s}`"))),
                false => Ok(Self {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                }),
            },
            None => Err(Error::Parse(format!("Invalid repository format `{s}`"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;

    #[test]
    fn test_repository_parser() {
        assert!("ploys/ploys".parse::<Repository>().is_ok());
        assert!("rust-lang/rust".parse::<Repository>().is_ok());
        assert!("one/two/three".parse::<Repository>().is_err());
    }
}
