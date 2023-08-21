//! Remote project inspection and management
//!
//! This module contains the utilities related to remote project management. The
//! [`Remote`] type must be constructed via [`super::Project`].

mod error;
mod repo;

use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

pub use self::error::Error;
pub use self::repo::Repository;

use super::local::Local;

/// A project in a remote version control system.
#[derive(Clone, Debug)]
pub struct Remote {
    repository: Repository,
    token: Option<String>,
}

impl Remote {
    /// Creates a remote project.
    pub(super) fn new<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        Ok(Self {
            repository: repository.as_ref().parse::<Repository>()?,
            token: None,
        })
    }

    /// Builds the project with the given authentication token.
    pub(super) fn with_authentication_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Builds the project with validation to ensure it exists.
    pub(super) fn validated(self) -> Result<Self, Error> {
        self.repository.validate(self.token.as_deref())?;

        Ok(self)
    }
}

impl Remote {
    /// Queries the project name.
    pub fn get_name(&self) -> Result<String, Error> {
        let request = self
            .repository
            .get("/readme", self.token.as_deref())
            .set("Accept", "application/vnd.github.raw");

        if let Ok(response) = request.call() {
            if let Ok(readme) = response.into_string() {
                if let Some(title) = readme.lines().find(|line| line.starts_with("# ")) {
                    return Ok(title[2..].to_string());
                }
            }
        }

        Ok(self.repository.to_string())
    }

    /// Queries the project URL.
    pub fn get_url(&self) -> Result<Url, Error> {
        Ok(format!("https://github.com/{}", self.repository)
            .parse()
            .unwrap())
    }

    /// Queries the project files.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        let request = self
            .repository
            .get("git/trees/HEAD", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .query("recursive", "true");

        let mut entries = request
            .call()?
            .into_json::<TreeResponse>()?
            .tree
            .into_iter()
            .filter(|entry| entry.r#type == "blob")
            .map(|entry| PathBuf::from(entry.path))
            .collect::<Vec<_>>();

        entries.sort();

        Ok(entries)
    }
}

impl TryFrom<Local> for Remote {
    type Error = super::Error;

    fn try_from(local: Local) -> Result<Self, Self::Error> {
        Ok(Self::new(local.get_url()?)?)
    }
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeResponseEntry>,
}

#[derive(Deserialize)]
struct TreeResponseEntry {
    path: String,
    r#type: String,
}

#[cfg(test)]
mod tests {
    use super::{Error, Remote};

    #[test]
    fn test_remote_constructor() {
        assert!(Remote::new("ploys/ploys").is_ok());
        assert!(Remote::new("rust-lang/rust").is_ok());
        assert!(Remote::new("one/two/three").is_err());
    }

    #[test]
    fn test_remote_url() -> Result<(), Error> {
        assert_eq!(
            Remote::new("ploys/ploys")?.get_url()?,
            "https://github.com/ploys/ploys".parse().unwrap()
        );

        Ok(())
    }
}
