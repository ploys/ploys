//! Lockfile inspection and management utilities
//!
//! This module includes utilities for inspecting and managing lockfiles across
//! different package managers.

pub mod cargo;
mod error;

use std::collections::HashMap;

use crate::package::PackageKind;
use crate::project::source::Source;

use self::cargo::CargoLockFile;
pub use self::error::Error;

/// A lockfile in one of several supported formats.
#[derive(Clone, Debug)]
pub enum LockFile {
    /// A `Cargo.lock` lockfile for Rust.
    Cargo(CargoLockFile),
}

impl LockFile {
    /// Sets the package version.
    pub fn set_package_version<P, V>(&mut self, package: P, version: V)
    where
        P: AsRef<str>,
        V: Into<String>,
    {
        match self {
            Self::Cargo(cargo) => cargo.set_package_version(package, version),
        }
    }

    /// Discovers project lockfiles.
    pub(super) fn discover_lockfiles<T>(
        source: &T,
    ) -> Result<HashMap<PackageKind, Self>, crate::project::Error>
    where
        T: Source,
        crate::project::Error: From<T::Error>,
    {
        let mut lockfiles = HashMap::new();

        for kind in PackageKind::variants() {
            if let Some(lockfile_name) = kind.lockfile_name() {
                if let Ok(bytes) = source.get_file_contents(lockfile_name) {
                    let lockfile = LockFile::from_bytes(*kind, &bytes)?;

                    lockfiles.insert(*kind, lockfile);
                }
            }
        }

        Ok(lockfiles)
    }

    /// Creates a lockfile from the given bytes.
    fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            PackageKind::Cargo => Ok(Self::Cargo(CargoLockFile::from_bytes(bytes)?)),
        }
    }
}
