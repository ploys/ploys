use std::path::Path;

use strum::EnumIter;

/// The package kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum PackageKind {
    /// The cargo package kind.
    Cargo,
}

impl PackageKind {
    /// Gets the package file name.
    pub fn file_name(&self) -> &'static Path {
        match self {
            Self::Cargo => Path::new("Cargo.toml"),
        }
    }

    /// Gets the lockfile name.
    pub(crate) fn lockfile_name(&self) -> Option<&'static Path> {
        match self {
            Self::Cargo => Some(Path::new("Cargo.lock")),
        }
    }
}
