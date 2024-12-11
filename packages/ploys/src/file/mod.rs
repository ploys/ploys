mod cache;
mod error;

use std::fmt::{self, Display};
use std::path::Path;

use strum::{EnumIs, EnumTryAs};

use crate::changelog::Changelog;
use crate::package::{Lockfile, Manifest, PackageKind};
use crate::project::Config;

pub use self::cache::FileCache;
pub use self::error::ParseError;

/// A file in one of a number of formats.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs, EnumTryAs)]
pub enum File {
    Config(Config),
    Manifest(Manifest),
    Lockfile(Lockfile),
    Changelog(Changelog),
}

impl File {
    /// Constructs a new file from the given bytes and path.
    pub(crate) fn from_bytes(bytes: Vec<u8>, path: &Path) -> Option<Self> {
        match path.file_name() {
            Some(name) if name == "Ploys.toml" => Config::from_bytes(&bytes).ok().map(Self::Config),
            Some(name) if name == "Cargo.toml" => Manifest::from_bytes(PackageKind::Cargo, &bytes)
                .ok()
                .map(Self::Manifest),
            Some(name) if name == "Cargo.lock" => Lockfile::from_bytes(PackageKind::Cargo, &bytes)
                .ok()
                .map(Self::Lockfile),
            Some(name) if name == "CHANGELOG.md" => Some(Self::Changelog(
                String::from_utf8(bytes).ok()?.parse().ok()?,
            )),
            _ => None,
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(config) => Display::fmt(config, f),
            Self::Manifest(manifest) => Display::fmt(manifest, f),
            Self::Lockfile(lockfile) => Display::fmt(lockfile, f),
            Self::Changelog(changelog) => Display::fmt(changelog, f),
        }
    }
}

impl From<Config> for File {
    fn from(value: Config) -> Self {
        Self::Config(value)
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
