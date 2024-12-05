use std::fmt::{self, Display};
use std::str::FromStr;

use url::Url;

use crate::repository::spec::Error;

/// The GitHub repository specification.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GitHubRepoSpec {
    owner: String,
    repo: String,
}

impl GitHubRepoSpec {
    /// Constructs a new GitHub repository specification.
    ///
    /// This requires that both `owner` and `repo` match the following regular
    /// expression: `^[a-zA-Z0-9\-_\.]+$`
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Result<Self, Error> {
        let owner = owner.into();
        let repo = repo.into();

        if owner.is_empty() || !owner.chars().all(is_valid_char) {
            return Err(Error::invalid(format!("{owner}/{repo}")));
        }

        if repo.is_empty() || !repo.chars().all(is_valid_char) {
            return Err(Error::invalid(format!("{owner}/{repo}")));
        }

        Ok(Self { owner, repo })
    }

    /// Gets the owner of the repository.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Gets the name of the repository.
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Gets the repository URL.
    pub fn to_url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}", self))
            .expect("constructor ensures valid path components")
    }
}

impl Display for GitHubRepoSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

impl FromStr for GitHubRepoSpec {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        match spec.split_once('/') {
            Some((owner, repo)) => Self::new(owner, repo),
            None => Err(Error::invalid(spec)),
        }
    }
}

/// A helper utility to define valid `owner` and `repo` name characters.
fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-' || c == '_' || c == '.'
}

#[cfg(test)]
mod tests {
    use super::GitHubRepoSpec;

    #[test]
    fn test_parse() {
        assert!("ploys/ploys".parse::<GitHubRepoSpec>().is_ok());
        assert!("rust-lang/rust".parse::<GitHubRepoSpec>().is_ok());
        assert!("one/two/three".parse::<GitHubRepoSpec>().is_err());
    }
}
