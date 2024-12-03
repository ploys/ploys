use std::fmt::{self, Display};

use strum::{EnumIs, EnumTryAs};

use super::cargo::CargoManifest;
use super::error::Error;
use super::members::Members;
use super::{Dependencies, DependenciesMut, Dependency, DependencyMut, PackageKind};

/// The package manifest.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs, EnumTryAs)]
pub enum Manifest {
    /// A cargo package manifest.
    Cargo(CargoManifest),
}

impl Manifest {
    /// Gets the package kind.
    pub fn package_kind(&self) -> PackageKind {
        match self {
            Self::Cargo(_) => PackageKind::Cargo,
        }
    }

    /// Gets the workspace members.
    pub fn members(&self) -> Result<Members, Error> {
        match self {
            Self::Cargo(cargo) => Ok(cargo.members()?),
        }
    }

    /// Creates a manifest from the given bytes.
    pub fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            PackageKind::Cargo => Ok(Self::Cargo(CargoManifest::from_bytes(bytes)?)),
        }
    }
}

impl Manifest {
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
}

impl Manifest {
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
}

impl Manifest {
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
}

impl Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(cargo) => Display::fmt(cargo, f),
        }
    }
}
