//! Lockfile inspection and management utilities
//!
//! This module includes utilities for inspecting and managing lockfiles across
//! different package managers.

pub mod cargo;
mod error;

use crate::package::PackageKind;
use crate::repository::Source;

use self::cargo::CargoLockFile;
pub use self::error::Error;

/// A lockfile in one of several supported formats.
#[derive(Clone, Debug)]
pub enum LockFile {
    /// A `Cargo.lock` lockfile for Rust.
    Cargo(CargoLockFile),
}

impl LockFile {
    /// Gets the lockfile kind.
    pub fn kind(&self) -> PackageKind {
        match self {
            Self::Cargo(_) => PackageKind::Cargo,
        }
    }

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

    /// Gets the contents of the lockfile.
    pub fn get_contents(&self) -> String {
        match self {
            Self::Cargo(cargo) => cargo.get_contents(),
        }
    }

    /// Checks if the lockfile has been changed.
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Cargo(cargo) => cargo.is_changed(),
        }
    }

    /// Discovers project lockfiles.
    pub(super) fn discover_lockfiles(source: &Source) -> Result<Vec<Self>, crate::project::Error> {
        let mut lockfiles = Vec::new();

        for kind in PackageKind::variants() {
            if let Some(lockfile_name) = kind.lockfile_name() {
                if let Ok(bytes) = source.get_file_contents(lockfile_name) {
                    let lockfile = LockFile::from_bytes(*kind, &bytes)?;

                    lockfiles.push(lockfile);
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
