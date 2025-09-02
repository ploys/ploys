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

use std::path::{Path, PathBuf};
use std::str::FromStr;

use bytes::Bytes;
use either::Either;

use crate::package::lockfile::CargoLockfile;
use crate::package::manifest::CargoManifest;
use crate::package::{BumpOrVersion, Package, PackageKind};
use crate::repository::types::staging::Staging;
use crate::repository::{Remote, RepoSpec, Repository, Stage};

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
pub struct Project<T = Staging> {
    pub(crate) repository: T,
    config: Config,
}

impl Project {
    /// Creates a new project.
    pub fn new(name: impl Into<String>) -> Self {
        let config = Config::new(name);

        Self {
            repository: Staging::new().with_file("Ploys.toml", config.to_string().into_bytes()),
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
    T: Stage,
{
    /// Adds a file to the project.
    pub fn add_file(
        &mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Error<T::Error>> {
        self.repository
            .add_file(path, file)
            .map_err(Error::Repository)?;

        Ok(self)
    }

    /// Builds the project with the given file.
    pub fn with_file(
        mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<Self, Error<T::Error>> {
        self.add_file(path, file)?;

        Ok(self)
    }
}

impl<T> Project<T>
where
    T: Repository,
{
    /// Gets a file at the given path.
    pub fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Error<T::Error>> {
        if path.as_ref() == Path::new("Ploys.toml") {
            return Ok(Some(self.config.to_string().into()));
        }

        self.repository.get_file(path).map_err(Error::Repository)
    }

    /// Gets a file at the given path in the specified format.
    #[allow(clippy::type_complexity)]
    pub fn get_file_as<U>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<U>, Either<Error<T::Error>, U::Err>>
    where
        U: FromStr,
    {
        match self.get_file(path).map_err(Either::Left)? {
            Some(bytes) => match std::str::from_utf8(&bytes) {
                Ok(str) => str.parse().map(Some).map_err(Either::Right),
                Err(err) => Err(Either::Left(Error::Utf8(err))),
            },
            None => Ok(None),
        }
    }
}

impl<T> Project<T>
where
    T: Stage,
{
    /// Adds the given package to the project.
    pub fn add_package(
        &mut self,
        package: impl Into<Package>,
    ) -> Result<&mut Self, Error<T::Error>> {
        let package = package.into();
        let base_path = Path::new("packages").join(package.name());

        for path in package.repository.get_index()? {
            if path == package.manifest_path() {
                continue;
            }

            if let Some(file) = package.repository.get_file(&path)? {
                self.add_file(base_path.join(path), file)?;
            }
        }

        self.add_file(
            base_path.join(package.manifest_path()),
            package.manifest().to_string().into_bytes(),
        )?;

        match package.kind() {
            PackageKind::Cargo => {
                let mut manifest = self
                    .get_file_as::<CargoManifest>("Cargo.toml")
                    .map_err(|err| {
                        err.map_right(crate::package::Error::Manifest)
                            .map_right(Error::Package)
                            .into_inner()
                    })?
                    .unwrap_or_default();

                manifest.add_workspace_member("packages/*");
                manifest.add_workspace_member(base_path.join(package.path()));

                let mut lockfile = self
                    .get_file_as::<CargoLockfile>("Cargo.lock")
                    .map_err(|err| {
                        err.map_right(crate::package::Error::Lockfile)
                            .map_right(Error::Package)
                            .into_inner()
                    })?
                    .unwrap_or_default();

                lockfile.add_package(package.manifest().try_as_cargo_ref().expect("cargo"));

                self.add_file("Cargo.toml", manifest.to_string().into_bytes())?;
                self.add_file("Cargo.lock", lockfile.to_string().into_bytes())?;
            }
        }

        Ok(self)
    }

    /// Builds the project with the given package.
    pub fn with_package(mut self, package: impl Into<Package>) -> Result<Self, Error<T::Error>> {
        self.add_package(package)?;

        Ok(self)
    }
}

impl<T> Project<T>
where
    T: Repository,
{
    /// Gets a package with the given name.
    pub fn get_package(&self, name: impl AsRef<str>) -> Option<Package<T>> {
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

#[cfg(feature = "fs")]
mod fs {
    use std::io::{Error as IoError, ErrorKind};
    use std::path::PathBuf;

    use crate::repository::types::fs::{Error as FsError, FileSystem};
    use crate::repository::types::staging::Staging;
    use crate::repository::{Commit, Repository, Stage};

    use super::{Error, Project};

    /// The [`FileSystem`] repository constructors.
    impl Project<FileSystem> {
        /// Opens a project from a [`FileSystem`] repository.
        pub fn fs<P>(path: P) -> Result<Self, Error<FsError>>
        where
            P: Into<PathBuf>,
        {
            Self::open(FileSystem::open(path)?)
        }

        /// Opens a project from a [`FileSystem`] repository in the current
        /// directory.
        pub fn current_dir() -> Result<Self, Error<FsError>> {
            Self::open(FileSystem::current_dir()?)
        }

        /// Writes the project to the file system.
        ///
        /// This method writes any staged file changes in this project to the
        /// file system, including the addition and removal of files. This
        /// includes the current project configuration, overriding any changes
        /// on disk.
        pub fn write(&mut self) -> Result<&mut Self, Error<FsError>> {
            self.repository.commit(())?;

            Ok(self)
        }
    }

    impl Project<Staging> {
        /// Writes the project to the file system.
        ///
        /// This method upgrades the project to use a [`FileSystem`] repository
        /// by writing the contents of the [`Staging`] repository to the file
        /// system. This includes the current project configuration stored in
        /// the project and not the original configuration from the repository.
        pub fn write<P>(self, path: P, force: bool) -> Result<Project<FileSystem>, Error<FsError>>
        where
            P: Into<PathBuf>,
        {
            let path = path.into();
            let repository = FileSystem::open(&path)?;

            if !force && repository.get_index()?.count() > 0 {
                return Err(Error::Repository(FsError::Io(IoError::new(
                    ErrorKind::DirectoryNotEmpty,
                    "Expected an empty directory",
                ))));
            }

            Ok(Project {
                repository: repository
                    .with_files(self.repository)?
                    .with_file(path.join("Ploys.toml"), self.config.to_string())?
                    .committed(())?,
                config: self.config,
            })
        }
    }
}

#[cfg(feature = "git")]
mod git {
    use std::path::PathBuf;

    use crate::repository::revision::Revision;
    use crate::repository::types::git::{Error as GitError, Git};

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
    use crate::repository::revision::Revision;
    use crate::repository::types::github::{Error as GitHubError, GitHub, GitHubRepoSpec};

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

#[cfg(all(feature = "fs", feature = "git"))]
mod fs_git {
    use crate::repository::revision::Revision;
    use crate::repository::types::fs::FileSystem;
    use crate::repository::types::git::{Error as GitError, Git};

    use super::{Error, Project};

    impl Project<FileSystem> {
        /// Upgrades to a [`Git`] repository without changing configuration.
        ///
        /// This method opens a [`Git`] repository at the same path as the
        /// [`FileSystem`] repository, keeping the project configuration from
        /// the current repository. Any changes to files that have not been
        /// committed to the target revision will not be accessible. This could
        /// lead to an invalid configuration state so the [`Project::reload`]
        /// and [`Project::reloaded`] methods are available to reload the
        /// configuration.
        pub fn into_git(
            self,
            revision: impl Into<Revision>,
        ) -> Result<Project<Git>, Error<GitError>> {
            Ok(Project {
                repository: Git::open(self.repository.path())?.with_revision(revision),
                config: self.config,
            })
        }

        /// Initializes a new [`Git`] repository.
        ///
        /// This method creates a new a [`Git`] repository at the same path as
        /// the [`FileSystem`] repository, keeping the project configuration
        /// from the current repository. This does not stage or commit changes
        /// so the new repository will appear to be empty.
        pub fn init_git(self) -> Result<Project<Git>, Error<GitError>> {
            Ok(Project {
                repository: Git::init(self.repository.path())?,
                config: self.config,
            })
        }
    }

    impl TryFrom<Project<FileSystem>> for Project<Git> {
        type Error = Error<GitError>;

        fn try_from(project: Project<FileSystem>) -> Result<Self, Self::Error> {
            project.into_git(Revision::head())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use crate::changelog::Changelog;
    use crate::package::Package;
    use crate::package::lockfile::CargoLockfile;
    use crate::package::manifest::CargoManifest;
    use crate::repository::RepoSpec;
    use crate::repository::types::staging::Staging;

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

        let mut project = project.reloaded().unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);
        assert_eq!(project.repository(), None);

        let package_a = Package::new_cargo("example-one");
        let package_b = Package::new_cargo("example-two")
            .with_version(Version::new(0, 1, 0))
            .with_file("CHANGELOG.md", Changelog::new().to_string().into_bytes())
            .unwrap();

        project.add_package(package_a).unwrap();
        project.add_package(package_b).unwrap();

        let package_a = project.get_package("example-one").unwrap();
        let package_b = project.get_package("example-two").unwrap();

        assert_eq!(package_a.name(), "example-one");
        assert_eq!(package_a.version(), Version::new(0, 0, 0));
        assert_eq!(package_a.get_file("CHANGELOG.md").unwrap(), None);

        assert_eq!(package_b.name(), "example-two");
        assert_eq!(package_b.version(), Version::new(0, 1, 0));
        assert_eq!(
            package_b.get_file_as("CHANGELOG.md").unwrap(),
            Some(Changelog::new())
        );

        let manifest = project
            .get_file_as::<CargoManifest>("Cargo.toml")
            .unwrap()
            .unwrap();

        let members = manifest.members().unwrap();

        assert!(members.includes(Path::new("packages/example-one")));
        assert!(members.includes(Path::new("packages/example-two")));

        let lockfile = project
            .get_file_as::<CargoLockfile>("Cargo.lock")
            .unwrap()
            .unwrap();

        assert_eq!(
            lockfile.get_package_version("example-one"),
            Some(Version::new(0, 0, 0))
        );
        assert_eq!(
            lockfile.get_package_version("example-two"),
            Some(Version::new(0, 1, 0))
        );
    }

    #[test]
    fn test_project_staging_repository() {
        let repository = Staging::new().with_file("Ploys.toml", "[project]\nname = \"example\"");
        let mut project = Project::open(repository).unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);

        project.set_description("An example project.");

        assert_eq!(project.description(), Some("An example project."));

        let mut project = project.reloaded().unwrap();

        assert_eq!(project.name(), "example");
        assert_eq!(project.description(), None);

        project.add_file("hello-world.txt", "Hello World!").unwrap();

        let txt = project.get_file("hello-world.txt").unwrap();

        assert_eq!(txt, Some("Hello World!".into()));
    }
}
