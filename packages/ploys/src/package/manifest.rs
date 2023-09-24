use std::path::{Path, PathBuf};

use super::cargo::manifest::Manifest as CargoManifest;
use super::error::Error;
use super::members::Members;
use super::{Package, PackageKind};

/// The package manifest.
#[derive(Debug)]
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
    pub fn file_name(&self) -> &'static Path {
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
    pub fn packages<F, E>(self, files: &[PathBuf], find: F) -> Result<Vec<Package>, E>
    where
        F: Fn(&Path) -> Result<Vec<u8>, E>,
        E: From<Error>,
    {
        let members = self.members()?;
        let file_name = self.file_name();

        let mut packages = Vec::new();

        if !members.is_empty() {
            for directory in self.directories(files) {
                if members.includes(directory) {
                    let path = directory.join(file_name);
                    let bytes = find(&path)?;
                    let package = Self::from_bytes(self.package_kind(), &bytes)?.into_package(path);

                    if let Some(package) = package {
                        packages.push(package);
                    }
                }
            }
        }

        if let Some(package) = self.into_package(file_name.to_owned()) {
            packages.push(package);
        }

        packages.sort_by_key(|package| package.name().to_owned());

        Ok(packages)
    }

    /// Creates a manifest from the given bytes.
    pub fn from_bytes(kind: PackageKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            PackageKind::Cargo => Ok(Self::Cargo(CargoManifest::from_bytes(bytes)?)),
        }
    }

    /// Converts this manifest into a package with the given path.
    pub fn into_package(self, path: PathBuf) -> Option<Package> {
        Some(match self {
            Self::Cargo(manifest) => Package::Cargo(manifest.into_package(path)?),
        })
    }
}
