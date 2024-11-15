mod fileset;

use crate::package::{Lockfile, Package};

pub use self::fileset::Fileset;

/// A file in one of a number of formats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum File {
    Package(Package),
    Lockfile(Lockfile),
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

    /// Gets the contents of the file.
    pub(crate) fn get_contents(&self) -> String {
        match self {
            Self::Package(package) => package.get_contents(),
            Self::Lockfile(lockfile) => lockfile.get_contents(),
        }
    }

    /// Checks if the file has been changed.
    pub(crate) fn is_changed(&self) -> bool {
        match self {
            Self::Package(package) => package.is_changed(),
            Self::Lockfile(lockfile) => lockfile.is_changed(),
        }
    }
}

impl From<Package> for File {
    fn from(value: Package) -> Self {
        Self::Package(value)
    }
}

impl From<Lockfile> for File {
    fn from(value: Lockfile) -> Self {
        Self::Lockfile(value)
    }
}
