//! The `Cargo.toml` package for Rust.

mod error;
pub(super) mod manifest;

use std::path::{Path, PathBuf};

pub use self::error::Error;
use self::manifest::Manifest;

/// A `Cargo.toml` package for Rust.
pub struct Cargo {
    manifest: Manifest,
    path: PathBuf,
}

impl Cargo {
    /// Creates a new cargo package.
    fn new(manifest: Manifest, path: PathBuf) -> Self {
        Self { manifest, path }
    }

    /// Gets the package name.
    pub fn name(&self) -> &str {
        self.manifest.inner().package().name()
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&str> {
        self.manifest.inner().package().description()
    }

    /// Gets the package version.
    pub fn version(&self) -> &str {
        self.manifest.inner().package().version()
    }

    /// Gets the package path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
