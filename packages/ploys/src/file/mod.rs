mod fileset;

use std::fmt::{self, Display};

use strum::{EnumIs, EnumTryAs};

use crate::changelog::Changelog;
use crate::package::{Lockfile, Manifest};

pub use self::fileset::Fileset;

/// A file in one of a number of formats.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs, EnumTryAs)]
pub enum File {
    Manifest(Manifest),
    Lockfile(Lockfile),
    Changelog(Changelog),
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(manifest) => Display::fmt(manifest, f),
            Self::Lockfile(lockfile) => Display::fmt(lockfile, f),
            Self::Changelog(changelog) => Display::fmt(changelog, f),
        }
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
