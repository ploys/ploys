//! The `Cargo.toml` package for Rust.

mod dependency;
mod error;
mod lockfile;
mod manifest;

use std::fmt::{self, Display};

use semver::Version;

pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::error::Error;
pub use self::lockfile::CargoLockfile;
pub use self::manifest::CargoManifest;

use super::{Bump, BumpError};

/// A `Cargo.toml` package for Rust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cargo {
    manifest: CargoManifest,
}

impl Cargo {
    /// Creates a new cargo package.
    fn new(manifest: CargoManifest) -> Self {
        Self { manifest }
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
    pub fn version(&self) -> Version {
        self.manifest.package().expect("package").version()
    }

    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        self.manifest
            .package_mut()
            .expect("package")
            .set_version(version);
        self
    }

    /// Bumps the package version.
    pub fn bump(&mut self, bump: Bump) -> Result<(), BumpError> {
        let mut version = self.version();

        bump.bump(&mut version)?;
        self.set_version(version);

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
}

impl Display for Cargo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.manifest, f)
    }
}
