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
//! println!("Name:       {}", project.get_name().unwrap());
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
//! println!("Name:       {}", project.get_name().unwrap());
//! println!("Repository: {}", project.get_url().unwrap());
//! ```

mod error;
pub mod git;
pub mod github;

use std::path::{Path, PathBuf};

use url::Url;

use crate::package::Package;

pub use self::error::Error;
use self::git::Git;
use self::github::GitHub;

/// A project from one of several supported sources.
#[derive(Clone, Debug)]
pub enum Project {
    /// A project in a local Git repository.
    Git(Git),
    /// A project in a remote GitHub repository.
    GitHub(GitHub),
}

impl Project {
    /// Creates a Git project.
    pub fn git<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self::Git(Git::new(path)?))
    }

    /// Checks if the project is Git.
    pub fn is_git(&self) -> bool {
        match self {
            Self::Git(_) => true,
            Self::GitHub(_) => false,
        }
    }

    /// Gets the project as Git.
    pub fn as_git(&self) -> Option<&Git> {
        match self {
            Self::Git(git) => Some(git),
            Self::GitHub(_) => None,
        }
    }
}

impl Project {
    /// Creates a GitHub project.
    pub fn github<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        Ok(Self::GitHub(GitHub::new(repository)?.validated()?))
    }

    /// Creates a GitHub project with the given authentication token.
    pub fn github_with_authentication_token<R, T>(repository: R, token: T) -> Result<Self, Error>
    where
        R: AsRef<str>,
        T: Into<String>,
    {
        Ok(Self::GitHub(
            GitHub::new(repository)?
                .with_authentication_token(token)
                .validated()?,
        ))
    }

    /// Checks if the project is GitHub.
    pub fn is_github(&self) -> bool {
        match self {
            Self::Git(_) => false,
            Self::GitHub(_) => true,
        }
    }

    /// Gets the project as GitHub.
    pub fn as_github(&self) -> Option<&GitHub> {
        match self {
            Self::Git(_) => None,
            Self::GitHub(github) => Some(github),
        }
    }

    /// Converts the project into GitHub.
    pub fn try_into_github(self) -> Result<Self, Error> {
        match self {
            Self::Git(git) => Ok(Self::GitHub(git.try_into()?)),
            Self::GitHub(github) => Ok(Self::GitHub(github)),
        }
    }
}

impl Project {
    /// Queries the project name.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_name(&self) -> Result<String, Error> {
        match self {
            Self::Git(git) => Ok(git.get_name()?),
            Self::GitHub(github) => Ok(github.get_name()?),
        }
    }

    /// Queries the project URL.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_url(&self) -> Result<Url, Error> {
        match self {
            Self::Git(git) => Ok(git.get_url()?),
            Self::GitHub(github) => Ok(github.get_url()?),
        }
    }

    /// Queries the project packages.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_packages(&self) -> Result<Vec<Package>, Error> {
        match self {
            Self::Git(git) => Ok(git.get_packages()?),
            Self::GitHub(github) => Ok(github.get_packages()?),
        }
    }

    /// Queries the project files.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        match self {
            Self::Git(git) => Ok(git.get_files()?),
            Self::GitHub(github) => Ok(github.get_files()?),
        }
    }

    /// Queries the contents of a project file.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        match self {
            Self::Git(git) => Ok(git.get_file_contents(path)?),
            Self::GitHub(github) => Ok(github.get_file_contents(path)?),
        }
    }
}
