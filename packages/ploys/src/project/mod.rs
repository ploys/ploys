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

    /// Opens a project with the Git source and revision.
    pub fn git_with_revision<P, R>(path: P, revision: R) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        R: Into<self::source::revision::Revision>,
    {
        use self::source::git::Git;

        let source = Git::new(path)?.with_revision(revision);
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

    /// Opens a project with the GitHub source and revision.
    pub fn github_with_revision<R, V>(repository: R, revision: V) -> Result<Self, Error>
    where
        R: AsRef<str>,
        V: Into<self::source::revision::Revision>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?
            .with_revision(revision)
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

    /// Opens a project with the GitHub source, revision, and authentication
    /// token.
    pub fn github_with_revision_and_authentication_token<R, V, T>(
        repository: R,
        revision: V,
        token: T,
    ) -> Result<Self, Error>
    where
        R: AsRef<str>,
        V: Into<self::source::revision::Revision>,
        T: Into<String>,
    {
        use self::source::github::GitHub;

        let source = GitHub::new(repository)?
            .with_revision(revision)
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
            Some(pkg) => {
                pkg.set_version(version.clone());

                if let Some(lockfile) = self.lockfiles.get_mut(&pkg.kind()) {
                    lockfile.set_package_version(pkg.name(), pkg.version());
                }

                for pkg in self.packages.iter_mut() {
                    if let Some(mut dependency) = pkg.get_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.to_string());
                        pkg.set_changed(true);
                    }

                    if let Some(mut dependency) = pkg.get_dev_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.to_string());
                        pkg.set_changed(true);
                    }

                    if let Some(mut dependency) = pkg.get_build_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.to_string());
                        pkg.set_changed(true);
                    }
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
            Some(pkg) => {
                pkg.bump(bump)?;

                if let Some(lockfile) = self.lockfiles.get_mut(&pkg.kind()) {
                    lockfile.set_package_version(pkg.name(), pkg.version());
                }

                let version = pkg.version().to_owned();

                for pkg in self.packages.iter_mut() {
                    if let Some(mut dependency) = pkg.get_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.clone());
                        pkg.set_changed(true);
                    }

                    if let Some(mut dependency) = pkg.get_dev_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.clone());
                        pkg.set_changed(true);
                    }

                    if let Some(mut dependency) = pkg.get_build_dependency_mut(package.as_ref()) {
                        dependency.set_version(version.clone());
                        pkg.set_changed(true);
                    }
                }

                Ok(())
            }
            None => Err(Error::PackageNotFound(package.as_ref().to_owned())),
        }
    }

    /// Gets the changed files.
    pub fn get_changed_files(&self) -> impl Iterator<Item = (PathBuf, String)> + '_ {
        self.packages()
            .iter()
            .filter(|package| package.is_changed())
            .map(|package| (package.path().to_owned(), package.get_contents()))
            .chain(
                self.lockfiles
                    .iter()
                    .filter(|(_, lockfile)| lockfile.is_changed())
                    .filter_map(|(kind, lockfile)| {
                        kind.lockfile_name()
                            .map(|name| (name.to_owned(), lockfile.get_contents()))
                    }),
            )
    }
}

#[cfg(feature = "github")]
impl Project<self::source::github::GitHub> {
    /// Commits the changes to the repository.
    ///
    /// This method takes a message and collection of files to include with the
    /// commit.
    pub fn commit(
        &mut self,
        message: impl AsRef<str>,
        files: impl IntoIterator<Item = (std::path::PathBuf, String)>,
    ) -> Result<String, Error> {
        use self::source::revision::{Reference, Revision};

        let files = self.get_changed_files().chain(files);
        let sha = self.source.commit(message, files)?;

        if !matches!(
            self.source.revision(),
            Revision::Reference(Reference::Branch(_))
        ) {
            self.source.set_revision(Revision::sha(sha.clone()));
        }

        Ok(sha)
    }

    /// Initiates the release of the specified package version.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    pub fn initiate_package_release(
        &self,
        package: impl AsRef<str>,
        version: impl Into<crate::package::BumpOrVersion>,
    ) -> Result<(), Error> {
        self.source
            .initiate_package_release(package.as_ref(), version.into())?;

        Ok(())
    }

    /// Gets the changelog release for the given package version.
    ///
    /// This method queries the GitHub API to generate new release information
    /// and may differ to the existing release information or changelogs. This
    /// includes information for new releases as well as existing ones.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    pub fn get_changelog_release(
        &self,
        package: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<crate::changelog::Release, Error> {
        Ok(self.source.get_changelog_release(
            package.as_ref(),
            version
                .as_ref()
                .parse::<Version>()
                .map_err(super::package::BumpError::Semver)?,
            package.as_ref() == self.name(),
        )?)
    }
}
