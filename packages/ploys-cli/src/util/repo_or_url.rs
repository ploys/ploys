use std::str::FromStr;

use anyhow::{anyhow, Error};
use ploys::project::repository::github::Repository;
use url::Url;

/// The repository string or remote URL.
#[derive(Clone)]
pub enum RepoOrUrl {
    Repo(Repository),
    Url(Url),
}

impl RepoOrUrl {
    /// Attempts to construct a repository from a URL.
    pub fn try_into_repo(self) -> Result<Repository, Error> {
        self.try_into()
    }
}

impl FromStr for RepoOrUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Repository>() {
            Ok(repo) => Ok(Self::Repo(repo)),
            Err(_) => match s.parse::<Url>() {
                Ok(url) => Ok(Self::Url(url)),
                Err(_) => Err(anyhow!("Expected owner/repo or URL, found: {}", s)),
            },
        }
    }
}

impl TryFrom<RepoOrUrl> for Repository {
    type Error = Error;

    fn try_from(value: RepoOrUrl) -> Result<Self, Self::Error> {
        match value {
            RepoOrUrl::Repo(repo) => Ok(repo),
            RepoOrUrl::Url(url) => {
                if url.domain() != Some("github.com") {
                    return Err(anyhow!(
                        "Unsupported remote repository: Only GitHub is supported"
                    ));
                }

                Ok(url
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches(".git")
                    .parse::<Repository>()?)
            }
        }
    }
}
