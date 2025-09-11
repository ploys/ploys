//! GitHub project inspection and management
//!
//! This module contains the utilities related to remote GitHub project
//! management.

mod changelog;
mod error;
mod repo;
mod spec;

use std::collections::BTreeSet;
use std::io::{self, Read};

use bytes::Bytes;
use relative_path::{RelativePath, RelativePathBuf};
use reqwest::header::CONTENT_TYPE;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::changelog::Release;
use crate::package::BumpOrVersion;
use crate::repository::cache::Cache;
use crate::repository::revision::Revision;
use crate::repository::{GitLike, Remote, Repository};

pub use self::error::Error;
pub use self::repo::Repo;
pub use self::spec::GitHubRepoSpec;

/// The remote GitHub repository.
#[derive(Clone, Debug)]
pub struct GitHub {
    repository: Repo,
    revision: Revision,
    token: Option<String>,
    cache: Cache,
}

impl GitHub {
    /// Opens a GitHub repository.
    ///
    /// Note that this does not validate the existence of the repository as it
    /// may require an authentication token. Call `validated` to ensure that a
    /// private repository exists after calling `with_authentication_token`.
    pub fn open<R>(repo: R) -> Result<Self, Error>
    where
        R: TryInto<GitHubRepoSpec, Error: Into<Error>>,
    {
        Ok(Self {
            repository: Repo::new(repo.try_into().map_err(Into::into)?)?,
            revision: Revision::head(),
            token: None,
            cache: Cache::new(),
        })
    }
}

impl GitHub {
    /// Builds the repository with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.revision = revision.into();
        self
    }

    /// Builds the repository with the given authentication token.
    pub fn with_authentication_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Builds the repository with validation to ensure it exists.
    pub fn validated(self) -> Result<Self, Error> {
        self.repository.validate(self.token.as_deref())?;

        Ok(self)
    }
}

impl GitHub {
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
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2022-11-28")
                    .send()?
                    .error_for_status()?
                    .json::<TreeResponse>()?
                    .sha;

                Ok(sha)
            }
            Revision::Reference(reference) => {
                let sha = self
                    .repository
                    .get(format!("git/ref/{reference}"), self.token.as_deref())
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2022-11-28")
                    .send()?
                    .error_for_status()?
                    .json::<RefResponse>()?
                    .object
                    .sha;

                Ok(sha)
            }
        }
    }
}

impl Repository for GitHub {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Some(self.get_file_uncached(path.as_ref())?));
        }

        self.cache.get_or_try_init(
            path,
            |path| self.get_file_uncached(path),
            || self.get_index_uncached(),
        )
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Box::new(self.get_index_uncached()?.into_iter())
                as Box<dyn Iterator<Item = RelativePathBuf>>);
        }

        Ok(Box::new(
            self.cache.get_or_try_index(|| self.get_index_uncached())?,
        ))
    }
}

impl GitHub {
    fn get_index_uncached(&self) -> Result<BTreeSet<RelativePathBuf>, Error> {
        let entries = self
            .repository
            .get(
                format!("git/trees/{}", self.revision),
                self.token.as_deref(),
            )
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .query(&[("recursive", "true")])
            .send()?
            .error_for_status()?
            .json::<TreeResponse>()?
            .tree
            .into_iter()
            .filter(|entry| entry.r#type == "blob")
            .map(|entry| entry.path)
            .collect::<BTreeSet<_>>();

        Ok(entries)
    }

    fn get_file_uncached(&self, path: impl AsRef<RelativePath>) -> Result<Bytes, Error> {
        let mut response = self
            .repository
            .get(
                format!("contents/{}?ref={}", path.as_ref(), self.revision),
                self.token.as_deref(),
            )
            .header("Accept", "application/vnd.github.raw")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?
            .error_for_status()?;

        match response.headers().get(CONTENT_TYPE) {
            Some(content_type) => match content_type.to_str() {
                Ok(content_type) if content_type.contains("application/vnd.github.raw") => {
                    let mut contents = Vec::new();

                    response.read_to_end(&mut contents)?;

                    Ok(contents.into())
                }
                _ => Err(io::Error::from(io::ErrorKind::NotFound))?,
            },
            _ => Err(io::Error::from(io::ErrorKind::NotFound))?,
        }
    }
}

impl GitLike for GitHub {
    fn sha(&self) -> Result<String, Self::Error> {
        self.sha()
    }

    fn commit(
        &self,
        message: &str,
        files: Vec<(RelativePathBuf, String)>,
    ) -> Result<String, Self::Error> {
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
            path: RelativePathBuf,
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
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .json(&CreateBlob {
                    content,
                    encoding: String::from("utf-8"),
                })
                .send()
                .map_err(Error::from)?
                .error_for_status()
                .map_err(Error::from)?
                .json::<NewBlob>()
                .map_err(Error::from)?
                .sha;

            tree.tree.push(TreeObject {
                path,
                mode: String::from("100644"),
                r#type: String::from("blob"),
                sha,
            });
        }

        let tree_sha = self
            .repository
            .post("git/trees", self.token.as_deref())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&tree)
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?
            .json::<NewTree>()
            .map_err(Error::from)?
            .sha;

        let commit_sha = self
            .repository
            .post("git/commits", self.token.as_deref())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&CreateCommit {
                message: message.to_owned(),
                tree: tree_sha,
                parents: vec![base_sha],
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?
            .json::<NewCommit>()
            .map_err(Error::from)?
            .sha;

        Ok(commit_sha)
    }

    fn get_default_branch(&self) -> Result<String, Self::Error> {
        #[derive(Deserialize)]
        struct RepoResponse {
            default_branch: String,
        }

        let default_branch = self
            .repository
            .get("", self.token.as_deref())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?
            .json::<RepoResponse>()
            .map_err(Error::from)?
            .default_branch;

        Ok(default_branch)
    }

    fn create_branch(&self, name: &str) -> Result<(), Self::Error> {
        #[derive(Serialize)]
        struct NewBranch {
            r#ref: String,
            sha: String,
        }

        let sha = self.sha()?;

        self.repository
            .post("git/refs", self.token.as_deref())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&NewBranch {
                r#ref: format!("refs/heads/{}", name.trim_start_matches('/')),
                sha,
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?;

        Ok(())
    }

    fn update_branch(&self, name: &str, sha: &str) -> Result<(), Self::Error> {
        #[derive(Serialize)]
        struct UpdateRef {
            sha: String,
        }

        self.repository
            .patch(format!("git/refs/heads/{name}"), self.token.as_deref())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&UpdateRef {
                sha: sha.to_owned(),
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?;

        Ok(())
    }
}

impl Remote for GitHub {
    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), Self::Error> {
        #[derive(Serialize)]
        struct ClientPayload {
            package: String,
            version: String,
        }

        self.repository
            .post("dispatches", self.token.as_deref())
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&RepositoryDispatchEvent {
                event_type: String::from("ploys-package-release-request"),
                client_payload: ClientPayload {
                    package: package.to_owned(),
                    version: version.to_string(),
                },
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?;

        Ok(())
    }

    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Self::Error> {
        self::changelog::get_release(
            &self.repository,
            package,
            version,
            is_primary,
            self.token.as_deref(),
        )
    }

    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Self::Error> {
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
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&NewPullRequest {
                title: title.to_owned(),
                head: head.to_owned(),
                base: base.to_owned(),
                body: body.to_owned(),
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?
            .json::<PullRequestResponse>()
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
    ) -> Result<u64, Self::Error> {
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
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&NewRelease {
                tag_name: tag.to_owned(),
                target_commitish: sha.to_owned(),
                name: name.to_owned(),
                body: body.to_owned(),
                draft: false,
                prerelease,
                generate_release_notes: false,
                make_latest: latest.into(),
            })
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)?
            .json::<ReleaseResponse>()
            .map_err(Error::from)?
            .id;

        Ok(id)
    }
}

impl From<GitHubRepoSpec> for GitHub {
    fn from(repo: GitHubRepoSpec) -> Self {
        Self::open(repo).expect("valid repo specification")
    }
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeResponseEntry>,
}

#[derive(Deserialize)]
struct TreeResponseEntry {
    path: RelativePathBuf,
    r#type: String,
}

#[derive(Serialize)]
struct RepositoryDispatchEvent<T> {
    event_type: String,
    client_payload: T,
}
