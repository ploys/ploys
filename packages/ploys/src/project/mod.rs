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

use url::Url;

use crate::lockfile::LockFile;
use crate::package::{Bump, Package, PackageKind};

pub use self::error::Error;
use self::source::git::Git;
use self::source::github::{GitHub, Reference};
use self::source::Source;

/// A project from one of several supported sources.
#[derive(Clone, Debug)]
pub struct Project<T = Git> {
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

impl Project<Git> {
    /// Opens a project with the Git source.
    pub fn git<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
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
}

impl Project<GitHub> {
    /// Opens a project with the GitHub source.
    pub fn github<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
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
        F: Into<Reference>,
    {
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
        F: Into<Reference>,
        T: Into<String>,
    {
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
