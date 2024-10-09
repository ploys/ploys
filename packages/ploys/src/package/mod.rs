//! Package inspection and management utilities
//!
//! This module includes utilities for inspecting and managing packages located
//! on the local file system or in a remote version control system.

mod bump;
pub mod cargo;
mod error;
mod manifest;
mod members;

use std::path::Path;

use semver::Version;

use crate::project::source::Source;

pub use self::bump::{Bump, BumpOrVersion, Error as BumpError};
use self::cargo::Cargo;
pub use self::error::Error;
use self::manifest::Manifest;

/// A package in one of several supported formats.
#[derive(Clone, Debug)]
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

    /// Gets the package manifest file path.
    pub fn path(&self) -> &Path {
        match self {
            Self::Cargo(cargo) => cargo.path(),
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

    /// Gets the package contents.
    pub fn get_contents(&self) -> String {
        match self {
            Self::Cargo(cargo) => cargo.get_contents(),
        }
    }

    /// Checks if the package has been changed.
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Cargo(cargo) => cargo.is_changed(),
        }
    }
}

impl Package {
    /// Discovers project packages.
    pub(super) fn discover_packages<T>(source: &T) -> Result<Vec<Package>, crate::project::Error>
    where
        T: Source,
        crate::project::Error: From<T::Error>,
    {
        let files = source.get_files()?;
        let mut packages = Vec::new();

        for kind in PackageKind::variants() {
            if let Ok(bytes) = source.get_file_contents(kind.file_name()) {
                let manifest = Manifest::from_bytes(*kind, &bytes)?;

                packages.extend(manifest.discover_packages(&files, source)?);
            }
        }

        Ok(packages)
    }
}

impl From<Cargo> for Package {
    fn from(value: Cargo) -> Self {
        Self::Cargo(value)
    }
}

/// The package kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackageKind {
    /// The cargo package kind.
    Cargo,
}

impl PackageKind {
    /// Gets the package variants.
    pub(super) fn variants() -> &'static [Self] {
        &[Self::Cargo]
    }

    /// Gets the package file name.
    pub fn file_name(&self) -> &'static Path {
        match self {
            Self::Cargo => Path::new("Cargo.toml"),
        }
    }

    /// Gets the lockfile name.
    pub(super) fn lockfile_name(&self) -> Option<&'static Path> {
        match self {
            Self::Cargo => Some(Path::new("Cargo.lock")),
        }
    }
}
