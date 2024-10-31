//! Project source configuration
//!
//! This module contains the common functionality that is shared by different
//! project sources.

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "github")]
pub mod github;

#[cfg(any(feature = "git", feature = "github"))]
pub mod revision;

use std::path::{Path, PathBuf};

use url::Url;

/// A project source.
pub trait Source {
    /// The source error.
    type Error;

    /// Queries the source name.
    fn get_name(&self) -> Result<String, Self::Error>;

    /// Queries the source URL.
    fn get_url(&self) -> Result<Url, Self::Error>;

    /// Queries the project files.
    fn get_files(&self) -> Result<Vec<PathBuf>, Self::Error>;

    /// Queries the contents of a project file.
    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Self::Error>
    where
        P: AsRef<Path>;
}
