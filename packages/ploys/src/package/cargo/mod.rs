//! The `Cargo.toml` package for Rust.

mod dependency;
mod error;
pub(super) mod manifest;

use std::path::{Path, PathBuf};

pub use self::dependency::{Dependencies, Dependency};
pub use self::error::Error;
use self::manifest::Manifest;

use super::{Bump, BumpError};

/// A `Cargo.toml` package for Rust.
#[derive(Clone, Debug)]
pub struct Cargo {
    manifest: Manifest,
    path: PathBuf,
    changed: bool,
}

impl Cargo {
    /// Creates a new cargo package.
    fn new(manifest: Manifest, path: PathBuf) -> Self {
        Self {
            manifest,
            path,
            changed: false,
        }
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
        self.changed = true;
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

    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.dependencies().get(name)
    }

    /// Gets the dependencies.
    pub fn dependencies(&self) -> Dependencies<'_> {
        self.manifest.dependencies()
    }

    /// Gets the package contents.
    pub fn get_contents(&self) -> String {
        self.manifest.0.to_string()
    }

    /// Checks if the package has been changed.
    pub fn is_changed(&self) -> bool {
        self.changed
    }
}
