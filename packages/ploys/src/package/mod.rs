//! Package inspection and management utilities
//!
//! This module includes utilities for inspecting and managing packages located
//! on the local file system or in a remote version control system.

mod bump;
pub mod cargo;
mod dependency;
mod error;
mod kind;
mod lockfile;
mod manifest;
mod members;
mod release;

use std::fmt::{self, Display};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use semver::Version;

use crate::project::Project;
use crate::repository::Repository;

pub use self::bump::{Bump, BumpOrVersion, Error as BumpError};
use self::cargo::Cargo;
pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::error::Error;
pub use self::kind::PackageKind;
pub use self::lockfile::Lockfile;
use self::manifest::Manifest;
pub use self::release::{ReleaseRequest, ReleaseRequestBuilder};

/// An immutable package reference.
#[derive(Clone, Copy)]
pub struct PackageRef<'a> {
    pub(crate) package: &'a Package,
    pub(crate) path: &'a Path,
    pub(crate) project: &'a Project,
}

impl<'a> PackageRef<'a> {
    /// Gets the package name.
    pub fn name(&self) -> &'a str {
        self.package.name()
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&'a str> {
        self.package.description()
    }

    /// Gets the package version.
    pub fn version(&self) -> Version {
        self.package.version().parse().expect("version")
    }

    /// Gets the package path.
    pub fn path(&self) -> &'a Path {
        self.path
    }

    /// Checks if this is the primary package.
    ///
    /// A primary package shares the same name as the project and all releases
    /// are tagged under the version number without the package name prefix.
    pub fn is_primary(&self) -> bool {
        self.package.name() == self.project.name()
    }
}

impl<'a> PackageRef<'a> {
    /// Constructs a new release request builder.
    pub fn create_release_request(
        self,
        version: impl Into<BumpOrVersion>,
    ) -> ReleaseRequestBuilder<'a> {
        ReleaseRequestBuilder::new(self, version.into())
    }
}

impl<'a> Deref for PackageRef<'a> {
    type Target = Package;

    fn deref(&self) -> &Self::Target {
        self.package
    }
}

/// A package in one of several supported formats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Package {
    /// A `Cargo.toml` package for Rust.
    Cargo(Cargo),
}

impl Package {
    /// Gets the package name.
    pub fn name(&self) -> &str {
        match self {
            Self::Cargo(cargo) => cargo.name(),
        }
    }

    /// Gets the package description, if it exists.
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Cargo(cargo) => cargo.description(),
        }
    }

    /// Gets the package version.
    pub fn version(&self) -> &str {
        match self {
            Self::Cargo(cargo) => cargo.version(),
        }
    }

    /// Gets the package kind.
    pub fn kind(&self) -> PackageKind {
        match self {
            Self::Cargo(_) => PackageKind::Cargo,
        }
    }

    /// Sets the package version.
    pub fn set_version(&mut self, version: Version) {
        match self {
            Self::Cargo(cargo) => cargo.set_version(version.to_string()),
        };
    }

    /// Bumps the package version.
    pub fn bump(&mut self, bump: Bump) -> Result<(), BumpError> {
        match self {
            Self::Cargo(cargo) => Ok(cargo.bump(bump)?),
        }
    }

    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        match self {
            Self::Cargo(cargo) => cargo.get_dependency(name).map(Dependency::Cargo),
        }
    }

    /// Gets the mutable dependency with the given name.
    pub fn get_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        match self {
            Self::Cargo(cargo) => cargo.get_dependency_mut(name).map(DependencyMut::Cargo),
        }
    }

    /// Gets the dependencies.
    pub fn dependencies(&self) -> Dependencies<'_> {
        match self {
            Self::Cargo(cargo) => Dependencies::Cargo(cargo.dependencies()),
        }
    }

    /// Gets the mutable dependencies.
    pub fn dependencies_mut(&mut self) -> DependenciesMut<'_> {
        match self {
            Self::Cargo(cargo) => DependenciesMut::Cargo(cargo.dependencies_mut()),
        }
    }

    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        match self {
            Self::Cargo(cargo) => cargo.get_dev_dependency(name).map(Dependency::Cargo),
        }
    }

    /// Gets the mutable dev dependency with the given name.
    pub fn get_dev_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        match self {
            Self::Cargo(cargo) => cargo.get_dev_dependency_mut(name).map(DependencyMut::Cargo),
        }
    }

    /// Gets the dev dependencies.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        match self {
            Self::Cargo(cargo) => Dependencies::Cargo(cargo.dev_dependencies()),
        }
    }

    /// Gets the mutable dev dependencies.
    pub fn dev_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        match self {
            Self::Cargo(cargo) => DependenciesMut::Cargo(cargo.dev_dependencies_mut()),
        }
    }

    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        match self {
            Self::Cargo(cargo) => cargo.get_build_dependency(name).map(Dependency::Cargo),
        }
    }

    /// Gets the mutable build dependency with the given name.
    pub fn get_build_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        match self {
            Self::Cargo(cargo) => cargo
                .get_build_dependency_mut(name)
                .map(DependencyMut::Cargo),
        }
    }

    /// Gets the build dependencies.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        match self {
            Self::Cargo(cargo) => Dependencies::Cargo(cargo.build_dependencies()),
        }
    }

    /// Gets the mutable build dependencies.
    pub fn build_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        match self {
            Self::Cargo(cargo) => DependenciesMut::Cargo(cargo.build_dependencies_mut()),
        }
    }

    /// Checks if the package has been changed.
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Cargo(cargo) => cargo.is_changed(),
        }
    }

    /// Sets the package as changed.
    pub(crate) fn set_changed(&mut self, changed: bool) {
        match self {
            Self::Cargo(cargo) => cargo.set_changed(changed),
        }
    }
}

impl Package {
    /// Discovers project packages.
    pub(super) fn discover_packages(
        repository: &Repository,
    ) -> Result<Vec<(PathBuf, Package)>, crate::project::Error> {
        let files = repository.get_files()?;
        let mut packages = Vec::new();

        for kind in PackageKind::variants() {
            if let Ok(bytes) = repository.get_file_contents(kind.file_name()) {
                let manifest = Manifest::from_bytes(*kind, &bytes)?;

                packages.extend(manifest.discover_packages(&files, repository)?);
            }
        }

        Ok(packages)
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(cargo) => Display::fmt(cargo, f),
        }
    }
}

impl From<Cargo> for Package {
    fn from(value: Cargo) -> Self {
        Self::Cargo(value)
    }
}
