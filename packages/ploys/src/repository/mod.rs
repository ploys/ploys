//! Project source configuration
//!
//! This module contains the common functionality that is shared by different
//! project sources.

mod error;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "github")]
pub mod github;

#[cfg(any(feature = "git", feature = "github"))]
pub mod revision;

use std::path::{Path, PathBuf};

use url::Url;

pub use self::error::Error;

/// A project source.
pub enum Source {
    #[cfg(feature = "git")]
    Git(self::git::Git),
    #[cfg(feature = "github")]
    GitHub(self::github::GitHub),
}

impl Source {
    /// Queries the source name.
    pub fn get_name(&self) -> Result<String, Error> {
        match self {
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_name()?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_name()?),
        }
    }

    /// Queries the source URL.
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
}
