//! Repository inspection and management utilities
//!
//! This module includes utilities for inspecting and managing repositories
//! located on the local file system or in a remote version control system.

mod remote;
mod spec;
mod vcs;

pub mod adapters;
pub mod path;
pub mod revision;
pub mod types;

use bytes::Bytes;
use relative_path::{RelativePath, RelativePathBuf};

pub use self::remote::Remote;
pub use self::spec::{Error as RepoSpecError, RepoSpec, ShortRepoSpec};
pub use self::vcs::GitLike;

/// Defines a file repository.
///
/// # Paths
///
/// Repositories use [`RelativePath`] to address files to avoid portability
/// issues across operating systems. This uses a fixed `/` separator and is
/// guaranteed to be valid UTF-8.
pub trait Repository {
    type Error;

    /// Gets a file at the given path.
    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error>;

    /// Gets the index.
    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error>;
}

impl<T> Repository for &T
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        (**self).get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        (**self).get_index()
    }
}

impl<T> Repository for &mut T
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        (**self).get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        (**self).get_index()
    }
}

/// Defines the ability to stage files in a repository.
pub trait Stage: Repository {
    /// Adds the given file to the index.
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error>;

    /// Adds multiple staged files to the repository.
    fn add_files(
        &mut self,
        files: impl IntoIterator<Item = (RelativePathBuf, Bytes)>,
    ) -> Result<&mut Self, Self::Error> {
        for (path, file) in files {
            self.add_file(path, file)?;
        }

        Ok(self)
    }

    /// Builds the repository with the given file in the index.
    fn with_file(
        mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        self.add_file(path, file)?;

        Ok(self)
    }

    /// Builds the repository with the given staged files.
    fn with_files(
        mut self,
        files: impl IntoIterator<Item = (RelativePathBuf, Bytes)>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        self.add_files(files)?;

        Ok(self)
    }

    /// Removes a file from the index.
    fn remove_file(&mut self, path: impl AsRef<RelativePath>)
    -> Result<Option<Bytes>, Self::Error>;
}

impl<T> Stage for &mut T
where
    T: Stage,
{
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        (**self).add_file(path, file)?;

        Ok(self)
    }

    fn remove_file(
        &mut self,
        path: impl AsRef<RelativePath>,
    ) -> Result<Option<Bytes>, Self::Error> {
        (**self).remove_file(path)
    }
}

/// Defines the ability to commit files in a repository.
pub trait Commit: Stage {
    /// The associated commit parameters.
    type Params;

    /// Commits the staged file changes to the repository.
    fn commit(&mut self, params: impl Into<Self::Params>) -> Result<(), Self::Error>;

    /// Builds the repository with the staged file changes committed.
    fn committed(mut self, params: impl Into<Self::Params>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        self.commit(params)?;

        Ok(self)
    }
}

impl<T> Commit for &mut T
where
    T: Commit,
{
    type Params = T::Params;

    fn commit(&mut self, params: impl Into<Self::Params>) -> Result<(), Self::Error> {
        (**self).commit(params)
    }
}
