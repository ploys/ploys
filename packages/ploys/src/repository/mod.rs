//! Repository inspection and management utilities
//!
//! This module includes utilities for inspecting and managing repositories
//! located on the local file system or in a remote version control system.

mod cache;
mod error;
mod remote;
mod spec;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "github")]
pub mod github;

pub mod memory;
pub mod revision;

use std::borrow::Cow;
use std::path::Path;

pub use self::error::Error;
pub(crate) use self::remote::Remote;
pub use self::spec::{Error as RepoSpecError, RepoSpec, ShortRepoSpec};

/// A source code repository.
#[derive(Clone)]
pub enum Repository {
    Memory(self::memory::Memory),
    #[cfg(feature = "git")]
    Git(self::git::Git),
    #[cfg(feature = "github")]
    GitHub(self::github::GitHub),
}

impl Repository {
    /// Gets a file at the given path.
    pub(crate) fn get_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<Cow<'_, [u8]>>, crate::project::Error> {
        match self {
            Self::Memory(memory) => Ok(memory.get_file(path)?),
            #[cfg(feature = "git")]
            Self::Git(git) => Ok(git.get_file(path)?),
            #[cfg(feature = "github")]
            Self::GitHub(github) => Ok(github.get_file(path)?),
        }
    }

    /// Gets the file index.
    pub(crate) fn get_file_index(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Cow<'_, Path>> + '_>, crate::project::Error> {
        match self {
            Self::Memory(memory) => Ok(Box::new(memory.get_file_index()?)),
            #[cfg(feature = "git")]
            Self::Git(git) => git.get_file_index(),
            #[cfg(feature = "github")]
            Self::GitHub(github) => github.get_file_index(),
        }
    }
}

impl Repository {
    /// Gets the repository as a remote.
    pub(crate) fn as_remote(&self) -> Option<&dyn Remote> {
        match self {
            Self::Memory(_) => None,
            #[cfg(feature = "git")]
            Self::Git(_) => None,
            #[cfg(feature = "github")]
            Self::GitHub(github) => Some(github),
        }
    }
}

impl From<self::memory::Memory> for Repository {
    fn from(memory: self::memory::Memory) -> Self {
        Self::Memory(memory)
    }
}

#[cfg(feature = "git")]
impl From<self::git::Git> for Repository {
    fn from(git: self::git::Git) -> Self {
        Self::Git(git)
    }
}

#[cfg(feature = "github")]
impl From<self::github::GitHub> for Repository {
    fn from(github: self::github::GitHub) -> Self {
        Self::GitHub(github)
    }
}

#[cfg(feature = "github")]
impl From<self::github::GitHubRepoSpec> for Repository {
    fn from(github: self::github::GitHubRepoSpec) -> Self {
        Self::GitHub(github.into())
    }
}

impl TryFrom<RepoSpec> for Repository {
    type Error = RepoSpecError;

    fn try_from(spec: RepoSpec) -> Result<Self, Self::Error> {
        #[cfg(feature = "github")]
        if let Some(spec) = spec.to_github() {
            use self::github::GitHub;

            return Ok(Self::GitHub(GitHub::from(spec)));
        }

        Err(RepoSpecError::invalid(spec.to_string()))
    }
}
