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

use std::borrow::Borrow;
use std::path::Path;

use semver::Version;
use url::Url;

use crate::file::Fileset;
use crate::package::{Lockfile, Package};
use crate::repository::{Remote, Repository};

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

    /// Gets a package with the given name.
    pub fn get_package(&self, name: impl AsRef<str>) -> Option<Package<'_>> {
        self.packages()
            .find(|package| package.name() == name.as_ref())
    }

    /// Gets the project packages.
    pub fn packages(&self) -> impl Iterator<Item = Package<'_>> {
        self.files
            .manifests()
            .filter_map(|(path, manifest)| Package::from_manifest(self, path, manifest))
    }

    // Gets the project lockfiles.
    pub fn lockfiles(&self) -> impl Iterator<Item = (&Path, &Lockfile)> {
        self.files.lockfiles()
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
}

impl Project {
    /// Requests the release of the specified package version.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    pub fn request_package_release(
        &self,
        package: impl AsRef<str>,
        version: impl Into<crate::package::BumpOrVersion>,
    ) -> Result<(), Error> {
        self.get_remote()
            .ok_or(Error::Unsupported)?
            .request_package_release(package.as_ref(), version.into())?;

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
        version: impl Borrow<Version>,
    ) -> Result<crate::changelog::Release, Error> {
        let release = self
            .get_remote()
            .ok_or(Error::Unsupported)?
            .get_changelog_release(
                package.as_ref(),
                version.borrow(),
                package.as_ref() == self.name(),
            )?;

        Ok(release)
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
}
