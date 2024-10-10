//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;
mod git2;
mod gix;

use std::path::{Path, PathBuf};

use url::Url;

pub use self::error::{Error, GixError};
pub use self::git2::Git2;
pub use self::gix::Gix;

use super::revision::Revision;
use super::Source;

/// The local Git repository source.
pub enum Git {
    /// The `gix` source for basic `git` operations.
    Gix(Gix),
    /// The `git2` source for advanced `git` operations.
    Git2(Git2),
}

impl Git {
    /// Creates a Git source.
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self::Gix(Gix::new(path)?))
    }
}

impl Git {
    /// Gets the revision.
    pub fn revision(&self) -> &Revision {
        match self {
            Self::Gix(gix) => gix.revision(),
            Self::Git2(git2) => git2.revision(),
        }
    }

    /// Sets the revision.
    pub fn set_revision(&mut self, revision: impl Into<Revision>) {
        match self {
            Self::Gix(gix) => gix.set_revision(revision),
            Self::Git2(git2) => git2.set_revision(revision),
        }
    }

    /// Builds the source with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.set_revision(revision);
        self
    }
}

impl Git {
    /// Creates a new branch.
    pub(crate) fn create_branch(&mut self, branch_name: &str) -> Result<String, Error> {
        match self {
            Self::Gix(_) => unreachable!("upgrade called first"),
            Self::Git2(git2) => git2.create_branch(branch_name),
        }
    }
}

impl Source for Git {
    type Config = GitConfig;
    type Error = Error;

    fn open_with(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::new(config.path)
    }

    fn get_name(&self) -> Result<String, Self::Error> {
        match self {
            Self::Gix(gix) => gix.get_name(),
            Self::Git2(git2) => git2.get_name(),
        }
    }

    fn get_url(&self) -> Result<Url, Self::Error> {
        match self {
            Self::Gix(gix) => gix.get_url(),
            Self::Git2(git2) => git2.get_url(),
        }
    }

    fn get_files(&self) -> Result<Vec<PathBuf>, Self::Error> {
        match self {
            Self::Gix(gix) => gix.get_files(),
            Self::Git2(git2) => git2.get_files(),
        }
    }

    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Self::Error>
    where
        P: AsRef<Path>,
    {
        match self {
            Self::Gix(gix) => gix.get_file_contents(path),
            Self::Git2(git2) => git2.get_file_contents(path),
        }
    }
}

/// The Git source configuration.
pub struct GitConfig {
    path: PathBuf,
    revision: Revision,
}

impl GitConfig {
    /// Creates a new Git source configuration.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            path: path.into(),
            revision: Revision::Head,
        }
    }

    /// Builds the configuration with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.revision = revision.into();
        self
    }
}
