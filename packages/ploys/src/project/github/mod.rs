//! GitHub project inspection and management
//!
//! This module contains the utilities related to remote GitHub project
//! management. The [`GitHub`] type must be constructed via [`super::Project`].

mod error;
mod repo;

use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use url::Url;

use crate::package::Package;

pub use self::error::Error;
pub use self::repo::Repository;

use super::git::Git;

/// A project in a remote GitHub repository.
#[derive(Clone, Debug)]
pub struct GitHub {
    repository: Repository,
    token: Option<String>,
}

impl GitHub {
    /// Creates a GitHub project.
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

impl GitHub {
    /// Queries the project name.
    pub fn get_name(&self) -> Result<String, Error> {
        Ok(self.repository.name().to_owned())
    }

    /// Queries the project URL.
    pub fn get_url(&self) -> Result<Url, Error> {
        Ok(format!("https://github.com/{}", self.repository)
            .parse()
            .unwrap())
    }

    /// Queries the project packages.
    pub fn get_packages(&self) -> Result<Vec<Package>, Error> {
        let files = self.get_files()?;

        Package::discover(&files, |path| self.get_file_contents(path))
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

    /// Queries the contents of a project file.
    ///
    /// This method makes a network request to query the file contents from the
    /// API.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        let request = self
            .repository
            .get(
                format!("contents/{}", path.as_ref().display()),
                self.token.as_deref(),
            )
            .set("Accept", "application/vnd.github.raw")
            .set("X-GitHub-Api-Version", "2022-11-28");

        let response = request.call()?;

        match response.header("content-type") {
            Some(content_type) if content_type.contains("application/vnd.github.raw") => {
                let mut contents = Vec::new();

                response.into_reader().read_to_end(&mut contents)?;

                Ok(contents)
            }
            _ => Err(io::Error::from(io::ErrorKind::NotFound))?,
        }
    }
}

impl TryFrom<Git> for GitHub {
    type Error = super::Error;

    fn try_from(git: Git) -> Result<Self, Self::Error> {
        Ok(Self::new(git.get_url()?)?)
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
    use super::{Error, GitHub};

    #[test]
    fn test_github_constructor() {
        assert!(GitHub::new("ploys/ploys").is_ok());
        assert!(GitHub::new("rust-lang/rust").is_ok());
        assert!(GitHub::new("one/two/three").is_err());
    }

    #[test]
    fn test_github_url() -> Result<(), Error> {
        assert_eq!(
            GitHub::new("ploys/ploys")?.get_url()?,
            "https://github.com/ploys/ploys".parse().unwrap()
        );

        Ok(())
    }
}
