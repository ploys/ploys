use strum::EnumIter;

/// The package kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum PackageKind {
    /// The cargo package kind.
    Cargo,
}

impl PackageKind {
    /// Gets the package file name.
    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Cargo => "Cargo.toml",
        }
    }

    /// Gets the lockfile name.
    pub(crate) fn lockfile_name(&self) -> Option<&'static str> {
        match self {
            Self::Cargo => Some("Cargo.lock"),
        }
    }
}
