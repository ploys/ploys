use std::path::Path;

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
    pub(crate) fn lockfile_name(&self) -> Option<&'static Path> {
        match self {
            Self::Cargo => Some(Path::new("Cargo.lock")),
        }
    }
}
