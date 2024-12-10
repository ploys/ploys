//! Project inspection and management utilities
//!
//! This module includes utilities for inspecting and managing projects located
//! in one of several supported formats including a local Git repository and a
//! remote GitHub repository.
//!
//! ## Git
//!
//! To open a local Git project use the [`Project::git`] constructor and pass in
//! a path to the project on the local file system. The target directory must be
//! initialized as a Git repository.
//!
//! ```no_run
//! use ploys::project::Project;
//!
//! let project = Project::git(".").unwrap();
//!
//! println!("Name:       {}", project.name());
//! println!("Repository: {}", project.get_url().unwrap());
//! ```
//!
//! ## GitHub
//!
//! To open a remote GitHub project use the [`Project::github`] constructor and
//! pass in a string in the `owner/repo` format. The target identifier must
//! match an existing GitHub repository.
//!
//! ```no_run
//! use ploys::project::Project;
//!
//! let project = Project::github("ploys/ploys").unwrap();
//!
//! println!("Name:       {}", project.name());
//! println!("Repository: {}", project.get_url().unwrap());
//! ```

mod error;
mod packages;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use url::Url;

use crate::file::File;
use crate::package::Package;
use crate::repository::{Remote, Repository};

pub use self::error::Error;
pub use self::packages::Packages;

/// A project from one of several supported repositories.
pub struct Project {
    repository: Repository,
    name: String,
}

impl Project {
    /// Opens an existing project from the given repository.
    pub fn open(repository: impl Into<Repository>) -> Result<Self, Error> {
        let repository = repository.into();
        let name = repository.get_name()?;

        Ok(Self { repository, name })
    }
}

#[cfg(feature = "git")]
mod git {
    use std::path::PathBuf;

    use crate::repository::git::Git;
    use crate::repository::revision::Revision;

    use super::{Error, Project};

    /// The [`Git`] repository constructors.
    impl Project {
        /// Opens a project from a [`Git`] repository.
        pub fn git<P>(path: P) -> Result<Self, Error>
        where
            P: Into<PathBuf>,
        {
            Self::open(Git::open(path)?)
        }

        /// Opens a project from a [`Git`] repository and revision.
        pub fn git_with_revision<P, V>(path: P, revision: V) -> Result<Self, Error>
        where
            P: Into<PathBuf>,
            V: Into<Revision>,
        {
            Self::open(Git::open(path)?.with_revision(revision))
        }
    }
}

#[cfg(feature = "github")]
mod github {
    use crate::repository::github::{Error as GitHubError, GitHub, GitHubRepoSpec};
    use crate::repository::revision::Revision;

    use super::{Error, Project};

    /// The [`GitHub`] repository constructors.
    impl Project {
        /// Opens a project from a [`GitHub`] repository.
        pub fn github<R>(repo: R) -> Result<Self, Error>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
        {
            Self::open(GitHub::open(repo)?)
        }

        /// Opens a project from a [`GitHub`] repository and revision.
        pub fn github_with_revision<R, V>(repo: R, revision: V) -> Result<Self, Error>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
            V: Into<Revision>,
        {
            Self::open(GitHub::open(repo)?.with_revision(revision).validated()?)
        }

        /// Opens a project from a [`GitHub`] repository and authentication
        /// token.
        pub fn github_with_authentication_token<R, T>(repo: R, token: T) -> Result<Self, Error>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
            T: Into<String>,
        {
            Self::open(
                GitHub::open(repo)?
                    .with_authentication_token(token)
                    .validated()?,
            )
        }

        /// Opens a project from a [`GitHub`] repository, revision, and
        /// authentication token.
        pub fn github_with_revision_and_authentication_token<R, V, T>(
            repo: R,
            revision: V,
            token: T,
        ) -> Result<Self, Error>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
            V: Into<Revision>,
            T: Into<String>,
        {
            Self::open(
                GitHub::open(repo)?
                    .with_revision(revision)
                    .with_authentication_token(token)
                    .validated()?,
            )
        }
    }
}

impl Project {
    /// Gets the project name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Queries the project URL.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_url(&self) -> Result<Url, Error> {
        Ok(self.repository.get_url()?)
    }

    /// Gets a package with the given name.
    pub fn get_package(&self, name: impl AsRef<str>) -> Option<Package<'_>> {
        self.packages()
            .find(|package| package.name() == name.as_ref())
    }

    /// Gets an iterator over the project packages.
    pub fn packages(&self) -> Packages<'_> {
        Packages::new(self)
    }
}

impl Project {
    /// Gets the remote repository.
    pub(crate) fn get_remote(&self) -> Option<&dyn Remote> {
        #[cfg(feature = "github")]
        #[allow(irrefutable_let_patterns)]
        if let Repository::GitHub(github) = &self.repository {
            return Some(github);
        }

        None
    }

    /// Gets a file at the given path.
    pub(crate) fn get_file(&self, path: impl AsRef<Path>) -> Option<&File> {
        self.repository.get_file(path)
    }

    /// Gets the file index.
    pub(crate) fn get_file_index(&self) -> &BTreeSet<PathBuf> {
        self.repository.get_file_index()
    }
}

impl TryFrom<Repository> for Project {
    type Error = Error;

    fn try_from(repository: Repository) -> Result<Self, Self::Error> {
        Self::open(repository)
    }
}

#[cfg(feature = "git")]
impl TryFrom<crate::repository::git::Git> for Project {
    type Error = Error;

    fn try_from(repository: crate::repository::git::Git) -> Result<Self, Self::Error> {
        Self::open(repository)
    }
}

#[cfg(feature = "github")]
impl TryFrom<crate::repository::github::GitHub> for Project {
    type Error = Error;

    fn try_from(repository: crate::repository::github::GitHub) -> Result<Self, Self::Error> {
        Self::open(repository)
    }
}
