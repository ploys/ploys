//! Lockfile inspection and management utilities
//!
//! This module includes utilities for inspecting and managing lockfiles across
//! different package managers.

use std::fmt::{self, Display};

use semver::Version;
use strum::{EnumIs, EnumTryAs};

use crate::package::{Error, PackageKind};

use super::cargo::CargoLockfile;

/// The package lockfile.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs, EnumTryAs)]
pub enum Lockfile {
    /// A cargo package lockfile.
    Cargo(CargoLockfile),
}

impl Lockfile {
    /// Gets the lockfile kind.
    pub fn kind(&self) -> PackageKind {
        match self {
            Self::Cargo(_) => PackageKind::Cargo,
        }
    }

    /// Sets the package version.
    pub fn set_package_version(&mut self, package: impl AsRef<str>, version: impl Into<Version>) {
        match self {
            Self::Cargo(cargo) => cargo.set_package_version(package, version),
        }
    }
}

impl Lockfile {
    /// Constructs a lockfile from the given bytes.
    pub(crate) fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            PackageKind::Cargo => Ok(Self::Cargo(CargoLockfile::from_bytes(bytes)?)),
        }
    }
}

impl Display for Lockfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(cargo) => Display::fmt(cargo, f),
        }
    }
}
