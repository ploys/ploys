mod fileset;

use std::fmt::{self, Display};

use crate::changelog::Changelog;
use crate::package::{Lockfile, Manifest, Package};

pub use self::fileset::Fileset;

/// A file in one of a number of formats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum File {
    Package(Package),
    Manifest(Manifest),
    Lockfile(Lockfile),
    Changelog(Changelog),
}

impl File {
    /// Gets the file as a package.
    pub fn as_package(&self) -> Option<&Package> {
        match self {
            Self::Package(package) => Some(package),
            _ => None,
        }
    }

    /// Gets the file as a mutable package.
    pub fn as_package_mut(&mut self) -> Option<&mut Package> {
        match self {
            Self::Package(package) => Some(package),
            _ => None,
        }
    }

    /// Gets the file as a package manifest.
    pub fn as_manifest(&self) -> Option<&Manifest> {
        match self {
            Self::Manifest(manifest) => Some(manifest),
            _ => None,
        }
    }

    /// Gets the file as a mutable package manifest.
    pub fn as_manifest_mut(&mut self) -> Option<&mut Manifest> {
        match self {
            Self::Manifest(manifest) => Some(manifest),
            _ => None,
        }
    }

    /// Gets the file as a lockfile.
    pub fn as_lockfile(&self) -> Option<&Lockfile> {
        match self {
            Self::Lockfile(lockfile) => Some(lockfile),
            _ => None,
        }
    }

    /// Gets the file as a mutable lockfile.
    pub fn as_lockfile_mut(&mut self) -> Option<&mut Lockfile> {
        match self {
            Self::Lockfile(lockfile) => Some(lockfile),
            _ => None,
        }
    }

    /// Gets the file as a changelog.
    pub fn as_changelog(&self) -> Option<&Changelog> {
        match self {
            Self::Changelog(changelog) => Some(changelog),
            _ => None,
        }
    }

    /// Gets the file as a mutable changelog.
    pub fn as_changelog_mut(&mut self) -> Option<&mut Changelog> {
        match self {
            Self::Changelog(changelog) => Some(changelog),
            _ => None,
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Package(package) => Display::fmt(package, f),
            Self::Manifest(manifest) => Display::fmt(manifest, f),
            Self::Lockfile(lockfile) => Display::fmt(lockfile, f),
            Self::Changelog(changelog) => Display::fmt(changelog, f),
        }
    }
}

impl From<Package> for File {
    fn from(value: Package) -> Self {
        Self::Package(value)
    }
}

impl From<Manifest> for File {
    fn from(value: Manifest) -> Self {
        Self::Manifest(value)
    }
}

impl From<Lockfile> for File {
    fn from(value: Lockfile) -> Self {
        Self::Lockfile(value)
    }
}

impl From<Changelog> for File {
    fn from(value: Changelog) -> Self {
        Self::Changelog(value)
    }
}
