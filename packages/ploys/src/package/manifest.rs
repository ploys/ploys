use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use strum::{EnumIs, EnumTryAs};

use crate::repository::Repository;

use super::cargo::CargoManifest;
use super::error::Error;
use super::members::Members;
use super::{Package, PackageKind};

/// The package manifest.
#[derive(Clone, Debug, PartialEq, Eq, EnumIs, EnumTryAs)]
pub enum Manifest {
    /// A cargo package manifest.
    Cargo(CargoManifest),
}

impl Manifest {
    /// Gets the package kind.
    pub fn package_kind(&self) -> PackageKind {
        match self {
            Self::Cargo(_) => PackageKind::Cargo,
        }
    }

    /// Gets the file name for the manifest.
    pub(crate) fn file_name(&self) -> &'static Path {
        self.package_kind().file_name()
    }

    /// Gets the workspace members.
    pub fn members(&self) -> Result<Members, Error> {
        match self {
            Self::Cargo(cargo) => Ok(cargo.members()?),
        }
    }

    /// Produces an iterator of paths to treat as package directories.
    pub fn directories<'a>(&'a self, files: &'a [PathBuf]) -> impl Iterator<Item = &'a Path> {
        files
            .iter()
            .filter(|path| path.file_name() == Some(self.file_name().as_os_str()))
            .flat_map(|path| path.parent())
    }

    /// Finds member packages using the closure to query individual paths.
    pub(crate) fn discover_packages(
        self,
        files: &[PathBuf],
        repository: &Repository,
    ) -> Result<Vec<(PathBuf, Package)>, crate::project::Error> {
        let members = self.members()?;
        let file_name = self.file_name();

        let mut packages = Vec::new();

        if !members.is_empty() {
            for directory in self.directories(files) {
                if members.includes(directory) {
                    let path = directory.join(file_name);
                    let bytes = repository.get_file_contents(&path)?;
                    let package = Self::from_bytes(self.package_kind(), &bytes)?.into_package();

                    if let Some(package) = package {
                        packages.push((path, package));
                    }
                }
            }
        }

        if let Some(package) = self.into_package() {
            packages.push((file_name.to_owned(), package));
        }

        packages.sort_by_key(|(_, package)| package.name().to_owned());

        Ok(packages)
    }

    /// Creates a manifest from the given bytes.
    pub fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            PackageKind::Cargo => Ok(Self::Cargo(CargoManifest::from_bytes(bytes)?)),
        }
    }

    /// Converts this manifest into a package with the given path.
    pub fn into_package(self) -> Option<Package> {
        Some(match self {
            Self::Cargo(manifest) => Package::Cargo(manifest.into_package()?),
        })
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(cargo) => Display::fmt(cargo, f),
        }
    }
}
