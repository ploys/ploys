//! Project inspection and management utilities
//!
//! This module includes utilities for inspecting and managing projects located
//! on the local file system or in a remote version control system.
//!
//! ## Local
//!
//! To open a local project use the [`Project::local`] constructor and pass in
//! a path to the project on the local file system. The target directory must be
//! initialized as a Git repository.
//!
//! ```no_run
//! use ploys::project::Project;
//!
//! let project = Project::local(".").unwrap();
//!
//! println!("Name:       {}", project.get_name().unwrap());
//! println!("Repository: {}", project.get_url().unwrap());
//! ```
//!
//! ## Remote
//!
//! To open a remote project use the [`Project::remote`] constructor and pass in
//! a string in the `owner/repo` format. The target identifier must match an
//! existing GitHub repository.
//!
//! ```no_run
//! use ploys::project::Project;
//!
//! let project = Project::remote("ploys/ploys").unwrap();
//!
//! println!("Name:       {}", project.get_name().unwrap());
//! println!("Repository: {}", project.get_url().unwrap());
//! ```

mod error;
pub mod local;
pub mod remote;

use std::path::{Path, PathBuf};

use url::Url;

pub use self::error::Error;
use self::local::Local;
use self::remote::Remote;

/// A project that is either local or remote.
#[derive(Clone, Debug)]
pub enum Project {
    /// A project on the local file system.
    Local(Local),
    /// A project in a remote version control system.
    Remote(Remote),
}

impl Project {
    /// Creates a local project.
    pub fn local<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self::Local(Local::new(path)?))
    }

    /// Checks if the project is local.
    pub fn is_local(&self) -> bool {
        match self {
            Self::Local(_) => true,
            Self::Remote(_) => false,
        }
    }

    /// Gets the project as a local project.
    pub fn as_local(&self) -> Option<&Local> {
        match self {
            Self::Local(local) => Some(local),
            Self::Remote(_) => None,
        }
    }
}

impl Project {
    /// Creates a remote project.
    pub fn remote<R>(repository: R) -> Result<Self, Error>
    where
        R: AsRef<str>,
    {
        Ok(Self::Remote(Remote::new(repository)?.validated()?))
    }

    /// Creates a remote project with the given authentication token.
    pub fn remote_with_authentication_token<R, T>(repository: R, token: T) -> Result<Self, Error>
    where
        R: AsRef<str>,
        T: Into<String>,
    {
        Ok(Self::Remote(
            Remote::new(repository)?
                .with_authentication_token(token)
                .validated()?,
        ))
    }

    /// Checks if the project is remote.
    pub fn is_remote(&self) -> bool {
        match self {
            Self::Local(_) => false,
            Self::Remote(_) => true,
        }
    }

    /// Gets the project as a remote project.
    pub fn as_remote(&self) -> Option<&Remote> {
        match self {
            Self::Local(_) => None,
            Self::Remote(remote) => Some(remote),
        }
    }

    /// Converts the project into a remote.
    pub fn try_into_remote(self) -> Result<Self, Error> {
        match self {
            Self::Local(local) => Ok(Self::Remote(local.try_into()?)),
            Self::Remote(remote) => Ok(Self::Remote(remote)),
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
            Self::Local(local) => Ok(local.get_name()?),
            Self::Remote(remote) => Ok(remote.get_name()?),
        }
    }

    /// Queries the project URL.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_url(&self) -> Result<Url, Error> {
        match self {
            Self::Local(local) => Ok(local.get_url()?),
            Self::Remote(remote) => Ok(remote.get_url()?),
        }
    }

    /// Queries the project files.
    ///
    /// This method may perform file system operations or network requests to
    /// query the latest project information.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        match self {
            Self::Local(local) => Ok(local.get_files()?),
            Self::Remote(remote) => Ok(remote.get_files()?),
        }
    }
}
