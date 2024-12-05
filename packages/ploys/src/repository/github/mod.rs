//! GitHub project inspection and management
//!
//! This module contains the utilities related to remote GitHub project
//! management.

mod changelog;
mod error;
mod repo;
mod spec;

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::changelog::Release;
use crate::file::{File, FileCache};
use crate::package::BumpOrVersion;

pub use self::error::Error;
pub use self::repo::Repository;
pub use self::spec::GitHubRepoSpec;

use super::revision::Revision;
use super::Remote;

/// The remote GitHub repository.
#[derive(Clone, Debug)]
pub struct GitHub {
    repository: Repository,
    revision: Revision,
    token: Option<String>,
    file_cache: FileCache,
}

impl GitHub {
    /// Creates a GitHub repository.
    pub(crate) fn new(spec: impl Into<GitHubRepoSpec>) -> Self {
        Self {
            repository: Repository::new(spec),
            revision: Revision::head(),
            token: None,
            file_cache: FileCache::new(),
        }
    }
}

impl GitHub {
    /// Builds the repository with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.revision = revision.into();
        self
    }
}

impl GitHub {
    /// Builds the repository with the given authentication token.
    pub(crate) fn with_authentication_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Builds the repository with validation to ensure it exists.
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
}

impl GitHub {
    pub fn get_name(&self) -> Result<String, Error> {
        Ok(self.repository.name().to_owned())
    }

    pub fn get_url(&self) -> Result<Url, Error> {
        Ok(format!(
            "https://github.com/{}/{}",
            self.repository.owner(),
            self.repository.name()
        )
        .parse()
        .unwrap())
    }

    /// Gets the file at the given path.
    pub fn get_file(&self, path: impl AsRef<Path>) -> Option<&File> {
        self.file_cache
            .get_or_try_insert_with(path.as_ref(), |path| match self.get_file_contents(path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(err) => Err(err),
            })
            .inspect_err(|err| println!("Error loading file `{}`: {err}", path.as_ref().display()))
            .ok()
            .flatten()
    }

    /// Gets the file index.
    pub fn get_file_index(&self) -> &BTreeSet<PathBuf> {
        self.file_cache.get_or_try_index_with(|| self.get_files())
    }

    pub(crate) fn get_files(&self) -> Result<BTreeSet<PathBuf>, Error> {
        let request = self
            .repository
            .get(
                format!("git/trees/{}", self.revision),
                self.token.as_deref(),
            )
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .query("recursive", "true");

        let entries = request
            .call()?
            .into_json::<TreeResponse>()?
            .tree
            .into_iter()
            .filter(|entry| entry.r#type == "blob")
            .map(|entry| PathBuf::from(entry.path))
            .collect::<BTreeSet<_>>();

        Ok(entries)
    }

    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
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

impl Remote for GitHub {
    fn sha(&self) -> Result<String, super::Error> {
        Ok(self.sha()?)
    }

    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, super::Error> {
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
                })
                .map_err(Error::from)?
                .into_json::<NewBlob>()
                .map_err(Error::from)?
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
            .send_json(tree)
            .map_err(Error::from)?
            .into_json::<NewTree>()
            .map_err(Error::from)?
            .sha;

        let commit_sha = self
            .repository
            .post("git/commits", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(CreateCommit {
                message: message.to_owned(),
                tree: tree_sha,
                parents: vec![base_sha],
            })
            .map_err(Error::from)?
            .into_json::<NewCommit>()
            .map_err(Error::from)?
            .sha;

        Ok(commit_sha)
    }

    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), super::Error> {
        #[derive(Serialize)]
        struct ClientPayload {
            package: String,
            version: String,
        }

        self.repository
            .post("dispatches", self.token.as_deref())
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(RepositoryDispatchEvent {
                event_type: String::from("ploys-package-release-request"),
                client_payload: ClientPayload {
                    package: package.to_owned(),
                    version: version.to_string(),
                },
            })
            .map_err(Error::from)?;

        Ok(())
    }

    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, super::Error> {
        Ok(self::changelog::get_release(
            &self.repository,
            package,
            version,
            is_primary,
            self.token.as_deref(),
        )?)
    }

    fn get_default_branch(&self) -> Result<String, super::Error> {
        #[derive(Deserialize)]
        struct RepoResponse {
            default_branch: String,
        }

        let default_branch = self
            .repository
            .get("", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .call()
            .map_err(Error::from)?
            .into_json::<RepoResponse>()
            .map_err(Error::from)?
            .default_branch;

        Ok(default_branch)
    }

    fn create_branch(&self, name: &str) -> Result<(), super::Error> {
        #[derive(Serialize)]
        struct NewBranch {
            r#ref: String,
            sha: String,
        }

        let sha = self.sha()?;

        self.repository
            .post("git/refs", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(NewBranch {
                r#ref: format!("refs/heads/{}", name.trim_start_matches('/')),
                sha,
            })
            .map_err(Error::from)?;

        Ok(())
    }

    fn update_branch(&self, name: &str, sha: &str) -> Result<(), super::Error> {
        #[derive(Serialize)]
        struct UpdateRef {
            sha: String,
        }

        self.repository
            .patch(format!("git/refs/heads/{name}"), self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(UpdateRef {
                sha: sha.to_owned(),
            })
            .map_err(Error::from)?;

        Ok(())
    }

    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, super::Error> {
        #[derive(Serialize)]
        struct NewPullRequest {
            title: String,
            head: String,
            base: String,
            body: String,
        }

        #[derive(Deserialize)]
        struct PullRequestResponse {
            number: u64,
        }

        let number = self
            .repository
            .post("pulls", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(NewPullRequest {
                title: title.to_owned(),
                head: head.to_owned(),
                base: base.to_owned(),
                body: body.to_owned(),
            })
            .map_err(Error::from)?
            .into_json::<PullRequestResponse>()
            .map_err(Error::from)?
            .number;

        Ok(number)
    }

    fn create_release(
        &self,
        tag: &str,
        sha: &str,
        name: &str,
        body: &str,
        prerelease: bool,
        latest: bool,
    ) -> Result<u64, super::Error> {
        #[derive(Serialize)]
        struct NewRelease {
            tag_name: String,
            target_commitish: String,
            name: String,
            body: String,
            draft: bool,
            prerelease: bool,
            generate_release_notes: bool,
            make_latest: MakeLatest,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "lowercase")]
        enum MakeLatest {
            True,
            False,
        }

        impl From<bool> for MakeLatest {
            fn from(value: bool) -> Self {
                match value {
                    true => Self::True,
                    false => Self::False,
                }
            }
        }

        #[derive(Deserialize)]
        struct ReleaseResponse {
            id: u64,
        }

        let id = self
            .repository
            .post("releases", self.token.as_deref())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .send_json(NewRelease {
                tag_name: tag.to_owned(),
                target_commitish: sha.to_owned(),
                name: name.to_owned(),
                body: body.to_owned(),
                draft: false,
                prerelease,
                generate_release_notes: false,
                make_latest: latest.into(),
            })
            .map_err(Error::from)?
            .into_json::<ReleaseResponse>()
            .map_err(Error::from)?
            .id;

        Ok(id)
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

#[derive(Serialize)]
struct RepositoryDispatchEvent<T> {
    event_type: String,
    client_payload: T,
}

#[cfg(test)]
mod tests {
    use super::{Error, GitHub, GitHubRepoSpec};

    #[test]
    fn test_github_url() -> Result<(), Error> {
        assert_eq!(
            GitHub::new("ploys/ploys".parse::<GitHubRepoSpec>()?).get_url()?,
            "https://github.com/ploys/ploys".parse().unwrap()
        );

        Ok(())
    }
}
