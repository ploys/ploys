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
use crate::repository::memory::Memory;
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
pub struct Project<T = Memory> {
    pub(crate) repository: T,
    config: Config,
}

impl Project {
    /// Creates a new project.
    pub fn new(name: impl Into<String>) -> Self {
        let config = Config::new(name);

        Self {
            repository: Memory::new().with_file("Ploys.toml", config.to_string().into_bytes()),
            config,
        }
    }
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

impl<T> Project<T>
where
    T: Repository,
{
    /// Reloads the project configuration.
    pub fn reload(&mut self) -> Result<&mut Self, Error<T::Error>> {
        let config = self
            .repository
            .get_file("Ploys.toml")
            .map_err(Error::Repository)?
            .ok_or(self::config::Error::Missing)?;

        self.config = Config::from_bytes(&config)?;

        Ok(self)
    }

    /// Builds the project with reloaded project configuration.
    pub fn reloaded(mut self) -> Result<Self, Error<T::Error>> {
        self.reload()?;

        Ok(self)
    }
}

#[cfg(feature = "fs")]
mod fs {
    use std::io::{Error as IoError, ErrorKind};
    use std::path::PathBuf;

    use crate::repository::fs::FileSystem;
    use crate::repository::memory::Memory;
    use crate::repository::Repository;

    use super::{Error, Project};

    /// The [`FileSystem`] repository constructors.
    impl Project<FileSystem> {
        /// Opens a project from a [`FileSystem`] repository.
        pub fn fs<P>(path: P) -> Result<Self, Error<IoError>>
        where
            P: Into<PathBuf>,
        {
            Self::open(FileSystem::open(path))
        }

        /// Opens a project from a [`FileSystem`] repository in the current
        /// directory.
        pub fn current_dir() -> Result<Self, Error<IoError>> {
            Self::open(FileSystem::current_dir()?)
        }
    }

    impl Project<Memory> {
        /// Writes the project to the file system.
        ///
        /// This method upgrades the project to use a [`FileSystem`] repository
        /// by writing the contents of the [`Memory`] repository to the file
        /// system. This includes the current project configuration stored in
        /// the project and not the original configuration from the repository.
        pub fn write<P>(self, path: P, force: bool) -> Result<Project<FileSystem>, Error<IoError>>
        where
            P: Into<PathBuf>,
        {
            let path = path.into();
            let meta = path.metadata()?;

            if !meta.is_dir() {
                return Err(Error::Repository(IoError::new(
                    ErrorKind::NotADirectory,
                    "Expected a directory",
                )));
            }

            let repository = FileSystem::open(&path);

            if !force && repository.get_index()?.count() > 0 {
                return Err(Error::Repository(IoError::new(
                    ErrorKind::DirectoryNotEmpty,
                    "Expected an empty directory",
                )));
            }

            std::fs::write(path.join("Ploys.toml"), self.config.to_string())?;

            for file in self.repository.get_index()? {
                if file.as_os_str() == "Ploys.toml" {
                    continue;
                }

                let path = path.join(&file);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let file = self.repository.get_file(&file)?.expect("indexed");

                std::fs::write(path, file)?;
            }

            Ok(Project {
                repository,
                config: self.config,
            })
        }
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
        self.config.project_description()
    }

    /// Sets the project description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.config.set_project_description(description);
        self
    }

    /// Builds the project with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    /// Gets the project repository.
    pub fn repository(&self) -> Option<RepoSpec> {
        self.config.project_repository()
    }

    /// Sets the project repository.
    pub fn set_repository(&mut self, repository: impl Into<RepoSpec>) -> &mut Self {
        self.config.set_project_repository(repository);
        self
    }

    /// Builds the project with the given repository.
    pub fn with_repository(mut self, repository: impl Into<RepoSpec>) -> Self {
        self.set_repository(repository);
        self
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
    use crate::repository::RepoSpec;

    use super::Project;

    #[test]
    fn test_builder() {
        let project = Project::new("example")
            .with_description("An example project.")
            .with_repository("ploys/example".parse::<RepoSpec>().unwrap());

        assert_eq!(project.name(), "example");
        assert_eq!(project.description().unwrap(), "An example project.");
        assert_eq!(
            project.repository().unwrap(),
            "ploys/example".parse::<RepoSpec>().unwrap()
        );

        let project = project.reloaded().unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);
        assert_eq!(project.repository(), None);
    }

    #[test]
    fn test_project_memory_repository() {
        let repository = Memory::new().with_file("Ploys.toml", b"[project]\nname = \"example\"");
        let mut project = Project::open(repository).unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);

        project.set_description("An example project.");

        assert_eq!(project.description(), Some("An example project."));

        let project = project.reloaded().unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);
    }
}
