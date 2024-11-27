//! Lockfile inspection and management utilities
//!
//! This module includes utilities for inspecting and managing lockfiles across
//! different package managers.

use std::fmt::{self, Display};
use std::path::PathBuf;

use semver::Version;
use strum::{EnumIs, EnumTryAs, IntoEnumIterator};

use crate::package::{Error, PackageKind};
use crate::repository::Repository;

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

    /// Discovers project lockfiles.
    pub(crate) fn discover_lockfiles(
        repository: &Repository,
    ) -> Result<Vec<(PathBuf, Self)>, crate::project::Error> {
        let mut lockfiles = Vec::new();

        for kind in PackageKind::iter() {
            if let Some(lockfile_name) = kind.lockfile_name() {
                if let Ok(bytes) = repository.get_file_contents(lockfile_name) {
                    let lockfile = Lockfile::from_bytes(kind, &bytes)?;

                    lockfiles.push((lockfile_name.to_owned(), lockfile));
                }
            }
        }

        Ok(lockfiles)
    }

    /// Creates a lockfile from the given bytes.
    fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
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
