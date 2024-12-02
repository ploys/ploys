//! Repository inspection and management utilities
//!
//! This module includes utilities for inspecting and managing repositories
//! located on the local file system or in a remote version control system.

mod error;
mod remote;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "github")]
pub mod github;

pub mod revision;

use std::path::{Path, PathBuf};

use url::Url;

use crate::file::Fileset;
use crate::package::{Lockfile, Manifest};

pub use self::error::Error;
pub(crate) use self::remote::Remote;

/// A source code repository.
pub enum Repository {
    #[cfg(feature = "git")]
    Git(self::git::Git),
    #[cfg(feature = "github")]
    GitHub(self::github::GitHub),
}

impl Repository {
    /// Queries the project name.
    pub fn get_name(&self) -> Result<String, Error> {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_name()?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_name()?),
        }
    }

    /// Queries the project URL.
    pub fn get_url(&self) -> Result<Url, Error> {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_url()?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_url()?),
        }
    }

    /// Queries the project files.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_files()?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_files()?),
        }
    }

    /// Queries the contents of a project file.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_file_contents(path)?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_file_contents(path)?),
        }
    }

    /// Gets the repository fileset.
    pub(crate) fn get_fileset(&self) -> Result<Fileset, crate::project::Error> {
        Ok(Fileset::new()
            .with_files(Manifest::discover_manifests(self)?)
            .with_files(Lockfile::discover_lockfiles(self)?))
    }
}
