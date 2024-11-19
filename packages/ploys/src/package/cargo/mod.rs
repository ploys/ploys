//! The `Cargo.toml` package for Rust.

mod dependency;
mod error;
mod lockfile;
pub(super) mod manifest;

use std::fmt::{self, Display};

pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::error::Error;
pub use self::lockfile::CargoLockfile;
use self::manifest::Manifest;

use super::{Bump, BumpError};

/// A `Cargo.toml` package for Rust.
#[derive(Clone, Debug)]
pub struct Cargo {
    manifest: Manifest,
    changed: bool,
}

impl Cargo {
    /// Creates a new cargo package.
    fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
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

    /// Bumps the package version.
    pub fn bump(&mut self, bump: Bump) -> Result<(), BumpError> {
        self.set_version(bump.bump_str(self.version())?.to_string());

        Ok(())
    }

    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.dependencies().get(name)
    }

    /// Gets the mutable dependency with the given name.
    pub fn get_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.dependencies_mut()
            .into_iter()
            .find(|dependency| dependency.name() == name.as_ref())
    }

    /// Gets the dependencies.
    pub fn dependencies(&self) -> Dependencies<'_> {
        self.manifest.dependencies()
    }

    // Gets the mutable dependencies.
    pub fn dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.dependencies_mut()
    }

    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.dev_dependencies().get(name)
    }

    /// Gets the mutable dev dependency with the given name.
    pub fn get_dev_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.dev_dependencies_mut()
            .into_iter()
            .find(|dependency| dependency.name() == name.as_ref())
    }

    /// Gets the dev dependencies.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        self.manifest.dev_dependencies()
    }

    // Gets the mutable dev dependencies.
    pub fn dev_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.dev_dependencies_mut()
    }

    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.build_dependencies().get(name)
    }

    /// Gets the mutable build dependency with the given name.
    pub fn get_build_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.build_dependencies_mut()
            .into_iter()
            .find(|dependency| dependency.name() == name.as_ref())
    }

    /// Gets the build dependencies.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        self.manifest.build_dependencies()
    }

    // Gets the mutable build dependencies.
    pub fn build_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.build_dependencies_mut()
    }

    /// Checks if the package has been changed.
    pub fn is_changed(&self) -> bool {
        self.changed
    }

    /// Sets the package as changed.
    pub(crate) fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }
}

impl Display for Cargo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.manifest, f)
    }
}

impl PartialEq for Cargo {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
    }
}

impl Eq for Cargo {}
