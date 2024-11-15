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

use std::path::{Path, PathBuf};

use semver::Version;
use url::Url;

use crate::file::Fileset;
use crate::package::{Bump, Lockfile, Package};
use crate::repository::Repository;

pub use self::error::Error;

/// A project from one of several supported repositories.
pub struct Project {
    repository: Repository,
    name: String,
    files: Fileset,
}

#[cfg(feature = "git")]
impl Project {
    /// Opens a project with the Git repository.
    pub fn git<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        use crate::repository::git::Git;

        let repository = Repository::Git(Git::new(path)?);
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
    }

    /// Opens a project with the Git repository and revision.
    pub fn git_with_revision<P, R>(path: P, revision: R) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        R: Into<crate::repository::revision::Revision>,
    {
        use crate::repository::git::Git;

        let repository = Repository::Git(Git::new(path)?.with_revision(revision));
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
    }
}

#[cfg(feature = "github")]
impl Project {
    /// Opens a project with the GitHub repository.
    pub fn github<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        use crate::repository::github::GitHub;

        let repository = Repository::GitHub(GitHub::new(repository)?.validated()?);
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
    }

    /// Opens a project with the GitHub repository and revision.
    pub fn github_with_revision<R, V>(repository: R, revision: V) -> Result<Self, Error>
    where
        R: AsRef<str>,
        V: Into<crate::repository::revision::Revision>,
    {
        use crate::repository::github::GitHub;

        let repository = Repository::GitHub(
            GitHub::new(repository)?
                .with_revision(revision)
                .validated()?,
        );
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
    }

    /// Opens a project with the GitHub repository and authentication token.
    pub fn github_with_authentication_token<R, T>(repository: R, token: T) -> Result<Self, Error>
    where
        R: AsRef<str>,
        T: Into<String>,
    {
        use crate::repository::github::GitHub;

        let repository = Repository::GitHub(
            GitHub::new(repository)?
                .with_authentication_token(token)
                .validated()?,
        );
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
    }

    /// Opens a project with the GitHub repository, revision, and authentication
    /// token.
    pub fn github_with_revision_and_authentication_token<R, V, T>(
        repository: R,
        revision: V,
        token: T,
    ) -> Result<Self, Error>
    where
        R: AsRef<str>,
        V: Into<crate::repository::revision::Revision>,
        T: Into<String>,
    {
        use crate::repository::github::GitHub;

        let repository = Repository::GitHub(
            GitHub::new(repository)?
                .with_revision(revision)
                .with_authentication_token(token)
                .validated()?,
        );
        let name = repository.get_name()?;
        let files = repository.get_fileset()?;

        Ok(Self {
            repository,
            name,
            files,
        })
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

    /// Gets the project packages.
    pub fn packages(&self) -> impl Iterator<Item = (&Path, &Package)> {
        self.files.packages()
    }

    // Gets the project lockfiles.
    pub fn lockfiles(&self) -> impl Iterator<Item = (&Path, &Lockfile)> {
        self.files.lockfiles()
    }

    /// Queries the project files.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        Ok(self.repository.get_files()?)
    }

    /// Queries the contents of a project file.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        Ok(self.repository.get_file_contents(path)?)
    }

    /// Sets the version of the target package.
    pub fn set_package_version<S>(&mut self, package: S, version: Version) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        match self.files.get_package_by_name_mut(package.as_ref()) {
            Some((_, pkg)) => {
                pkg.set_version(version.clone());

                let pkg = pkg.clone();

                if let Some(lockfile) = self.files.get_lockfile_by_kind_mut(pkg.kind()) {
                    lockfile.set_package_version(pkg.name(), pkg.version());
                }

                for (_, pkg) in self.files.packages_mut() {
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
        match self.files.get_package_by_name_mut(package.as_ref()) {
            Some((_, pkg)) => {
                pkg.bump(bump)?;

                let pkg = pkg.clone();

                if let Some(lockfile) = self.files.get_lockfile_by_kind_mut(pkg.kind()) {
                    lockfile.set_package_version(pkg.name(), pkg.version());
                }

                let version = pkg.version().to_owned();

                for (_, pkg) in self.files.packages_mut() {
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
        self.files
            .files()
            .filter(|(_, file)| file.is_changed())
            .map(|(path, file)| (path.to_owned(), file.get_contents()))
    }
}

impl Project {
    /// Commits the changes to the repository.
    ///
    /// This method takes a message and collection of files to include with the
    /// commit.
    #[allow(unused_variables)]
    pub fn commit(
        &mut self,
        message: impl AsRef<str>,
        files: impl IntoIterator<Item = (std::path::PathBuf, String)>,
    ) -> Result<String, Error> {
        let files = self.get_changed_files().chain(files).collect::<Vec<_>>();

        #[cfg(feature = "github")]
        #[allow(irrefutable_let_patterns)]
        if let Repository::GitHub(github) = &mut self.repository {
            use crate::repository::revision::{Reference, Revision};

            let sha = github.commit(message, files.into_iter())?;

            if !matches!(github.revision(), Revision::Reference(Reference::Branch(_))) {
                github.set_revision(Revision::sha(sha.clone()));
            }

            return Ok(sha);
        }

        Err(Error::Unsupported)
    }

    /// Initiates the release of the specified package version.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    #[allow(unused_variables)]
    pub fn initiate_package_release(
        &self,
        package: impl AsRef<str>,
        version: impl Into<crate::package::BumpOrVersion>,
    ) -> Result<(), Error> {
        #[cfg(feature = "github")]
        #[allow(irrefutable_let_patterns)]
        if let Repository::GitHub(github) = &self.repository {
            github.initiate_package_release(package.as_ref(), version.into())?;

            return Ok(());
        }

        Err(Error::Unsupported)
    }

    /// Gets the changelog release for the given package version.
    ///
    /// This method queries the GitHub API to generate new release information
    /// and may differ to the existing release information or changelogs. This
    /// includes information for new releases as well as existing ones.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    #[allow(unused_variables)]
    pub fn get_changelog_release(
        &self,
        package: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<crate::changelog::Release, Error> {
        #[cfg(feature = "github")]
        #[allow(irrefutable_let_patterns)]
        if let Repository::GitHub(github) = &self.repository {
            return Ok(github.get_changelog_release(
                package.as_ref(),
                version
                    .as_ref()
                    .parse::<Version>()
                    .map_err(super::package::BumpError::Semver)?,
                package.as_ref() == self.name(),
            )?);
        }

        Err(Error::Unsupported)
    }
}
