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
pub mod source;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use semver::Version;
use url::Url;

use crate::lockfile::LockFile;
use crate::package::{Bump, Package, PackageKind};

pub use self::error::Error;
use self::source::Source;

/// A project from one of several supported sources.
#[derive(Clone, Debug)]
pub struct Project<T> {
    source: T,
    name: String,
    packages: Vec<Package>,
    lockfiles: HashMap<PackageKind, LockFile>,
}

impl<T> Project<T>
where
    T: Source,
    Error: From<T::Error>,
{
    /// Opens the project.
    pub fn open() -> Result<Self, Error>
    where
        T::Config: Default,
    {
        let source = T::open()?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }

    /// Opens the project with the given source configuration.
    pub fn open_with(config: T::Config) -> Result<Self, Error> {
        let source = T::open_with(config)?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }
}

#[cfg(feature = "git")]
impl Project<self::source::git::Git> {
    /// Opens a project with the Git source.
    pub fn git<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        use self::source::git::Git;

        let source = Git::new(path)?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }

    #[doc(hidden)]
    pub fn git2<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        use self::source::git::{Git, Git2};

        let source = Git::Git2(Git2::new(path)?);
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }
}

#[cfg(feature = "github")]
impl Project<self::source::github::GitHub> {
    /// Opens a project with the GitHub source.
    pub fn github<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?.validated()?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }

    /// Opens a project with the GitHub source and reference.
    pub fn github_with_reference<R, F>(repository: R, reference: F) -> Result<Self, Error>
    where
        R: AsRef<str>,
        F: Into<self::source::github::Reference>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?
            .with_reference(reference)
            .validated()?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }

    /// Opens a project with the GitHub source and authentication token.
    pub fn github_with_authentication_token<R, T>(repository: R, token: T) -> Result<Self, Error>
    where
        R: AsRef<str>,
        T: Into<String>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?
            .with_authentication_token(token)
            .validated()?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }

    /// Opens a project with the GitHub source, reference, and authentication
    /// token.
    pub fn github_with_reference_and_authentication_token<R, F, T>(
        repository: R,
        reference: F,
        token: T,
    ) -> Result<Self, Error>
    where
        R: AsRef<str>,
        F: Into<self::source::github::Reference>,
        T: Into<String>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?
            .with_reference(reference)
            .with_authentication_token(token)
            .validated()?;
        let name = source.get_name()?;
        let packages = Package::discover_packages(&source)?;
        let lockfiles = LockFile::discover_lockfiles(&source)?;

        Ok(Self {
            source,
            name,
            packages,
            lockfiles,
        })
    }
}

impl<T> Project<T>
where
    T: Source,
    Error: From<T::Error>,
{
    /// Gets the project name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Queries the project URL.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_url(&self) -> Result<Url, Error> {
        Ok(self.source.get_url()?)
    }

    /// Gets the project packages.
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    /// Queries the project files.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        Ok(self.source.get_files()?)
    }

    /// Queries the contents of a project file.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        Ok(self.source.get_file_contents(path)?)
    }

    /// Sets the version of the target package.
    pub fn set_package_version<S>(&mut self, package: S, version: Version) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        match self
            .packages
            .iter_mut()
            .find(|pkg| pkg.name() == package.as_ref())
        {
            Some(package) => {
                package.set_version(version);

                if let Some(lockfile) = self.lockfiles.get_mut(&package.kind()) {
                    lockfile.set_package_version(package.name(), package.version());
                }

                Ok(())
            }
            None => Err(Error::PackageNotFound(package.as_ref().to_owned())),
        }
    }

    /// Bumps the version of the target package.
    pub fn bump_package_version<S>(&mut self, package: S, bump: Bump) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        match self
            .packages
            .iter_mut()
            .find(|pkg| pkg.name() == package.as_ref())
        {
            Some(package) => {
                package.bump(bump)?;

                if let Some(lockfile) = self.lockfiles.get_mut(&package.kind()) {
                    lockfile.set_package_version(package.name(), package.version());
                }

                Ok(())
            }
            None => Err(Error::PackageNotFound(package.as_ref().to_owned())),
        }
    }
}

#[cfg(feature = "git")]
impl Project<self::source::git::Git> {
    /// Upgrades the interior source in place to use advanced `git` operations.
    ///
    /// The `Git` source uses two internal implementations of `git`, one that is
    /// pure Rust and another that uses C bindings. The former is not yet
    /// feature complete and so this swaps to the other implementation to
    /// support push operations.
    pub fn upgrade(&mut self) -> Result<(), Error> {
        use self::source::git::{Error, Git, Git2};

        if let Git::Gix(gix) = &self.source {
            let path = gix
                .repository
                .path()
                .join("..")
                .canonicalize()
                .map_err(Into::<Error>::into)?;

            self.source = Git::Git2(Git2::new(path)?);
        }

        Ok(())
    }

    /// Upgrades the interior source to use advanced `git` operations.
    pub fn upgraded(mut self) -> Result<Self, Error> {
        self.upgrade()?;

        Ok(self)
    }
}

#[cfg(feature = "git")]
impl Project<self::source::git::Git> {
    /// Releases the specified package.
    ///
    /// This triggers the release flow by creating a new remote branch. This
    /// acts as a form of authentication to ensure that the user has permission
    /// to create releases without authenticating with the API directly.
    pub fn release_package(
        &mut self,
        package: impl AsRef<str>,
        version: impl Into<crate::package::BumpOrVersion>,
    ) -> Result<(), Error> {
        use crate::package::BumpOrVersion;

        self.upgrade()?;

        let version = match version.into() {
            BumpOrVersion::Bump(bump) => {
                self.bump_package_version(package.as_ref(), bump)?;
                self.packages()
                    .iter()
                    .find(|pkg| pkg.name() == package.as_ref())
                    .expect("package")
                    .version()
                    .parse::<Version>()
                    .map_err(crate::package::BumpError::Semver)?
            }
            BumpOrVersion::Version(version) => {
                self.set_package_version(package.as_ref(), version.clone())?;

                version
            }
        };

        let branch_name = match package.as_ref() == self.name() {
            true => format!("release/{version}",),
            false => format!("release/{}-{version}", package.as_ref()),
        };

        self.source.create_branch(&branch_name)?;

        Ok(())
    }
}

#[cfg(feature = "github")]
impl Project<self::source::github::GitHub> {
    /// Releases the specified package.
    ///
    /// This triggers the release flow by creating a new remote branch. This
    /// acts as a form of authentication to ensure that the user has permission
    /// to create releases without authenticating with the API directly.
    pub fn release_package(
        &mut self,
        package: impl AsRef<str>,
        version: impl Into<crate::package::BumpOrVersion>,
    ) -> Result<(), Error> {
        use self::source::github::Reference;
        use crate::package::BumpOrVersion;

        let version = match version.into() {
            BumpOrVersion::Bump(bump) => {
                self.bump_package_version(package.as_ref(), bump)?;
                self.packages()
                    .iter()
                    .find(|pkg| pkg.name() == package.as_ref())
                    .expect("package")
                    .version()
                    .parse::<Version>()
                    .map_err(crate::package::BumpError::Semver)?
            }
            BumpOrVersion::Version(version) => {
                self.set_package_version(package.as_ref(), version.clone())?;

                version
            }
        };

        let branch_name = match package.as_ref() == self.name() {
            true => format!("release/{version}",),
            false => format!("release/{}-{version}", package.as_ref()),
        };

        let sha = self.source.create_branch(&branch_name)?;

        self.source.set_reference(Reference::Sha(sha));

        Ok(())
    }
}
