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
//! println!("Repository: {}", project.repository().unwrap());
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
//! println!("Repository: {}", project.repository().unwrap());
//! ```

pub mod config;
mod error;
mod packages;
mod release;

use crate::package::{BumpOrVersion, Package};
use crate::repository::{Remote, RepoSpec, Repository};

pub use self::config::Config;
pub use self::error::Error;
pub use self::packages::Packages;
pub use self::release::{ReleaseBuilder, ReleaseRequest, ReleaseRequestBuilder};

/// A project from one of several supported repositories.
///
/// A valid project contains a `Ploys.toml` configuration file in the following
/// format. Note that only the `project.name` field is required.
///
/// ```toml
/// [project]
/// name = "{project-name}"
/// description = "{project-description}"
/// repository = "https://github.com/{project-owner}/{project-name}"
/// ```
pub struct Project<T> {
    pub(crate) repository: T,
    config: Config,
}

impl<T> Project<T>
where
    T: Repository,
{
    /// Opens an existing project from the given repository.
    pub fn open(repository: T) -> Result<Self, Error<T::Error>> {
        let config = repository
            .get_file("Ploys.toml")
            .map_err(Error::Repository)?
            .ok_or(self::config::Error::Missing)?;

        Ok(Self {
            config: Config::from_bytes(&config)?,
            repository,
        })
    }
}

#[cfg(feature = "git")]
mod git {
    use std::path::PathBuf;

    use crate::repository::git::{Error as GitError, Git};
    use crate::repository::revision::Revision;

    use super::{Error, Project};

    /// The [`Git`] repository constructors.
    impl Project<Git> {
        /// Opens a project from a [`Git`] repository.
        pub fn git<P>(path: P) -> Result<Self, Error<GitError>>
        where
            P: Into<PathBuf>,
        {
            Self::open(Git::open(path).map_err(Error::Repository)?)
        }

        /// Opens a project from a [`Git`] repository and revision.
        pub fn git_with_revision<P, V>(path: P, revision: V) -> Result<Self, Error<GitError>>
        where
            P: Into<PathBuf>,
            V: Into<Revision>,
        {
            Self::open(Git::open(path)?.with_revision(revision))
        }
    }

    impl TryFrom<Git> for Project<Git> {
        type Error = Error<GitError>;

        fn try_from(repository: Git) -> Result<Self, Self::Error> {
            Self::open(repository)
        }
    }
}

#[cfg(feature = "github")]
mod github {
    use crate::repository::github::{Error as GitHubError, GitHub, GitHubRepoSpec};
    use crate::repository::revision::Revision;

    use super::{Error, Project};

    /// The [`GitHub`] repository constructors.
    impl Project<GitHub> {
        /// Opens a project from a [`GitHub`] repository.
        pub fn github<R>(repo: R) -> Result<Self, Error<GitHubError>>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
        {
            Self::open(GitHub::open(repo)?.validated()?)
        }

        /// Opens a project from a [`GitHub`] repository and revision.
        pub fn github_with_revision<R, V>(repo: R, revision: V) -> Result<Self, Error<GitHubError>>
        where
            R: TryInto<GitHubRepoSpec, Error: Into<GitHubError>>,
            V: Into<Revision>,
        {
            Self::open(GitHub::open(repo)?.with_revision(revision).validated()?)
        }

        /// Opens a project from a [`GitHub`] repository and authentication
        /// token.
        pub fn github_with_authentication_token<R, T>(
            repo: R,
            token: T,
        ) -> Result<Self, Error<GitHubError>>
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
        ) -> Result<Self, Error<GitHubError>>
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

    impl TryFrom<GitHub> for Project<GitHub> {
        type Error = Error<GitHubError>;

        fn try_from(repository: GitHub) -> Result<Self, Self::Error> {
            Self::open(repository)
        }
    }
}

impl<T> Project<T> {
    /// Gets the project name.
    pub fn name(&self) -> &str {
        self.config.project().name()
    }

    /// Gets the project description.
    pub fn description(&self) -> Option<&str> {
        self.config.project().description()
    }

    /// Gets the project repository.
    pub fn repository(&self) -> Option<RepoSpec> {
        self.config.project().repository()
    }
}

impl<T> Project<T>
where
    T: Repository,
{
    /// Gets a package with the given name.
    pub fn get_package(&self, name: impl AsRef<str>) -> Option<Package<&'_ T>> {
        self.packages()
            .find(|package| package.name() == name.as_ref())
    }

    /// Gets an iterator over the project packages.
    pub fn packages(&self) -> Packages<'_, T> {
        Packages::new(self)
    }
}

impl<T> Project<T>
where
    T: Remote,
{
    /// Constructs a new package release request builder.
    pub fn create_package_release_request(
        &self,
        package: impl AsRef<str>,
        version: impl Into<BumpOrVersion>,
    ) -> Result<ReleaseRequestBuilder<'_, T>, Error<T::Error>> {
        let package = self.get_package(package.as_ref()).ok_or_else(|| {
            Error::Package(crate::package::Error::NotFound(
                package.as_ref().to_string(),
            ))
        })?;

        Ok(ReleaseRequestBuilder::new(self, package, version.into()))
    }

    /// Constructs a new package release builder.
    pub fn create_package_release(
        &self,
        package: impl AsRef<str>,
    ) -> Result<ReleaseBuilder<'_, T>, Error<T::Error>> {
        let package = self.get_package(package.as_ref()).ok_or_else(|| {
            Error::Package(crate::package::Error::NotFound(
                package.as_ref().to_string(),
            ))
        })?;

        Ok(ReleaseBuilder::new(self, package))
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::memory::Memory;

    use super::Project;

    #[test]
    fn test_project_memory_repository() {
        let repository = Memory::new().with_file("Ploys.toml", b"[project]\nname = \"example\"");
        let project = Project::open(repository).unwrap();

        assert_eq!(project.name(), "example");
    }
}
