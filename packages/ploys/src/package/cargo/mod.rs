//! The `Cargo.toml` package for Rust.

mod error;
pub(super) mod manifest;

use std::path::{Path, PathBuf};

pub use self::error::Error;
use self::manifest::Manifest;

use super::{Bump, BumpError};

/// A `Cargo.toml` package for Rust.
#[derive(Clone, Debug)]
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
        self.manifest.package().expect("package").name()
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&str> {
        self.manifest.package().expect("package").description()
    }

    /// Gets the package version.
    pub fn version(&self) -> &str {
        self.manifest.package().expect("package").version()
    }

    /// Sets the package version.
    pub fn set_version<V>(&mut self, version: V) -> &mut Self
    where
        V: Into<String>,
    {
        self.manifest
            .package_mut()
            .expect("package")
            .set_version(version);
        self
    }

    /// Gets the package path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Bumps the package version.
    pub fn bump(&mut self, bump: Bump) -> Result<(), BumpError> {
        self.set_version(bump.bump_str(self.version())?.to_string());

        Ok(())
    }
}
