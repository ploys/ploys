//! Repository inspection and management utilities
//!
//! This module includes utilities for inspecting and managing repositories
//! located on the local file system or in a remote version control system.

mod cache;
mod remote;
mod spec;
mod vcs;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "github")]
pub mod github;

pub mod memory;
pub mod revision;

use std::borrow::Cow;
use std::path::Path;

use bytes::Bytes;

pub use self::remote::Remote;
pub use self::spec::{Error as RepoSpecError, RepoSpec, ShortRepoSpec};
pub use self::vcs::GitLike;

/// Defines a file repository.
pub trait Repository {
    type Error;

    /// Gets a file at the given path.
    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error>;

    /// Gets the index.
    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, Path>>, Self::Error>;
}

impl<T> Repository for &T
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        (*self).get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, Path>>, Self::Error> {
        (*self).get_index()
    }
}
