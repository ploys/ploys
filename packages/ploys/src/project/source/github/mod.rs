//! GitHub project inspection and management
//!
//! This module contains the utilities related to remote GitHub project
//! management.

mod error;
mod repo;

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

pub use self::error::Error;
pub use self::repo::Repository;

use super::revision::{Reference, Revision};
use super::Source;

/// The remote GitHub repository source.
#[derive(Clone, Debug)]
pub struct GitHub {
    repository: Repository,
    revision: Revision,
    token: Option<String>,
}

impl GitHub {
    /// Creates a GitHub source.
    pub(crate) fn new<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        Ok(Self {
            repository: repository.as_ref().parse::<Repository>()?,
            revision: Revision::head(),
            token: None,
        })
    }
}

impl GitHub {
    /// Gets the revision.
    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    /// Sets the revision.
    pub fn set_revision(&mut self, revision: impl Into<Revision>) {
        self.revision = revision.into();
    }

    /// Builds the source with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.revision = revision.into();
        self
    }
}

impl GitHub {
    /// Builds the source with the given authentication token.
    pub(crate) fn with_authentication_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Builds the source with validation to ensure it exists.
    pub(crate) fn validated(self) -> Result<Self, Error> {
        self.repository.validate(self.token.as_deref())?;

        Ok(self)
    }

    /// Gets the commit SHA.
    pub(crate) fn sha(&self) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct RefResponse {
            object: Object,
        }

        #[derive(serde::Deserialize)]
        struct Object {
            sha: String,
        }

        #[derive(serde::Deserialize)]
        struct TreeResponse {
            sha: String,
        }

        match &self.revision {
            Revision::Sha(sha) => Ok(sha.clone()),
            Revision::Head => {
                let sha = self
                    .repository
                    .get("git/trees/HEAD", self.token.as_deref())
                    .set("Accept", "application/vnd.github+json")
                    .set("X-GitHub-Api-Version", "2022-11-28")
                    .call()?
                    .into_json::<TreeResponse>()?
                    .sha;

                Ok(sha)
            }
            Revision::Reference(reference) => {
                let sha = self
                    .repository
                    .get(format!("git/ref/{reference}"), self.token.as_deref())
                    .set("Accept", "application/vnd.github+json")
                    .set("X-GitHub-Api-Version", "2022-11-28")
                    .call()?
                    .into_json::<RefResponse>()?
                    .object
                    .sha;

                Ok(sha)
            }
        }
    }

    /// Creates a new branch.
    pub(crate) fn create_branch(&self, branch_name: &str) -> Result<String, Error> {
        #[derive(serde::Serialize)]
        struct Body {
            r#ref: String,
            sha: String,
        }

        #[derive(serde::Deserialize)]
        struct RefResponse {
            object: Object,
        }

        #[derive(serde::Deserialize)]
        struct Object {
            sha: String,
        }

        let sha = self
            .repository
            .post("git/refs", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(Body {
                r#ref: format!("refs/heads/{}", branch_name),
                sha: self.sha()?,
            })?
            .into_json::<RefResponse>()?
            .object
            .sha;

        Ok(sha)
    }

    /// Commits the changes to the repository.
    pub(crate) fn commit(
        &self,
        message: impl AsRef<str>,
        files: impl Iterator<Item = (PathBuf, String)>,
    ) -> Result<String, Error> {
        #[derive(Serialize)]
        struct CreateBlob {
            content: String,
            encoding: String,
        }

        #[derive(Deserialize)]
        struct NewBlob {
            sha: String,
        }

        #[derive(Serialize)]
        struct CreateTree {
            tree: Vec<TreeObject>,
            base_tree: String,
        }

        #[derive(Serialize)]
        struct TreeObject {
            path: String,
            mode: String,
            r#type: String,
            sha: String,
        }

        #[derive(Deserialize)]
        struct NewTree {
            sha: String,
        }

        #[derive(Serialize)]
        struct CreateCommit {
            message: String,
            tree: String,
            parents: Vec<String>,
        }

        #[derive(Deserialize)]
        struct NewCommit {
            sha: String,
        }

        #[derive(Serialize)]
        struct UpdateRef {
            sha: String,
        }

        let base_sha = self.sha()?;

        let mut tree = CreateTree {
            tree: Vec::new(),
            base_tree: base_sha.clone(),
        };

        for (path, content) in files {
            let sha = self
                .repository
                .post("git/blobs", self.token.as_deref())
                .set("Accept", "application/vnd.github+json")
                .set("X-GitHub-Api-Version", "2022-11-28")
                .send_json(CreateBlob {
                    content,
                    encoding: String::from("utf-8"),
                })?
                .into_json::<NewBlob>()?
                .sha;

            tree.tree.push(TreeObject {
                path: path.to_string_lossy().into(),
                mode: String::from("100644"),
                r#type: String::from("blob"),
                sha,
            });
        }

        let tree_sha = self
            .repository
            .post("git/trees", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(tree)?
            .into_json::<NewTree>()?
            .sha;

        let commit_sha = self
            .repository
            .post("git/commits", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(CreateCommit {
                message: message.as_ref().to_owned(),
                tree: tree_sha,
                parents: vec![base_sha],
            })?
            .into_json::<NewCommit>()?
            .sha;

        if let Revision::Reference(Reference::Branch(branch)) = &self.revision {
            self.repository
                .patch(format!("git/refs/heads/{branch}"), self.token.as_deref())
                .set("Accept", "application/vnd.github+json")
                .set("X-GitHub-Api-Version", "2022-11-28")
                .send_json(UpdateRef {
                    sha: commit_sha.clone(),
                })?;
        }

        Ok(commit_sha)
    }
}

impl Source for GitHub {
    type Config = GitHubConfig;
    type Error = Error;

    fn open_with(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match config.token {
            Some(token) => Ok(Self::new(config.repo)?
                .with_revision(config.revision)
                .with_authentication_token(token)
                .validated()?),
            None => Ok(Self::new(config.repo)?.with_revision(config.revision)),
        }
    }

    fn get_name(&self) -> Result<String, Self::Error> {
        Ok(self.repository.name().to_owned())
    }

    fn get_url(&self) -> Result<Url, Self::Error> {
        Ok(format!("https://github.com/{}", self.repository)
            .parse()
            .unwrap())
    }

    fn get_files(&self) -> Result<Vec<PathBuf>, Self::Error> {
        let request = self
            .repository
            .get(
                format!("git/trees/{}", self.revision),
                self.token.as_deref(),
            )
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

    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Self::Error>
    where
        P: AsRef<Path>,
    {
        let request = self
            .repository
            .get(
                format!("contents/{}?ref={}", path.as_ref().display(), self.revision),
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

/// The GitHub source configuration.
pub struct GitHubConfig {
    repo: String,
    revision: Revision,
    token: Option<String>,
}

impl GitHubConfig {
    /// Creates a new GitHub source configuration.
    pub fn new<T>(repo: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            repo: repo.into(),
            revision: Revision::head(),
            token: None,
        }
    }

    /// Builds the configuration with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.revision = revision.into();
        self
    }

    /// Builds the configuration with the given authentication token.
    pub fn with_authentication_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
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
    use crate::project::source::Source;

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
