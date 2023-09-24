use std::path::PathBuf;

use cargo_toml::{Dependency, Manifest as CargoToml};
use globset::{Glob, GlobSetBuilder};

use crate::package::members::Members;

use super::error::Error;
use super::Cargo;

/// The cargo package manifest.
#[derive(Debug)]
pub struct Manifest(CargoToml);

impl Manifest {
    /// Gets the inner manifest.
    pub fn inner(&self) -> &CargoToml {
        &self.0
    }

    /// Gets the workspace members.
    ///
    /// This follows the [members and exclude fields](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields)
    /// documentation to build a list of included and excluded members.
    pub fn members(&self) -> Result<Members, Error> {
        let mut includes = GlobSetBuilder::new();
        let mut excludes = Vec::new();

        if let Some(workspace) = &self.0.workspace {
            for member in &workspace.members {
                includes.add(Glob::new(member.trim_start_matches("./"))?);
            }

            for path in &workspace.exclude {
                excludes.push(PathBuf::from(path));
            }
        }

        let dependencies = self
            .0
            .dependencies
            .values()
            .chain(self.0.dev_dependencies.values())
            .chain(self.0.build_dependencies.values());

        for dependency in dependencies {
            if let Dependency::Detailed(dependency) = dependency {
                if let Some(path) = &dependency.path {
                    includes.add(Glob::new(path.trim_start_matches("./"))?);
                }
            }
        }

        Ok(Members::new(includes.build()?, excludes))
    }

    /// Creates a manifest from the given bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self(CargoToml::from_slice(bytes)?))
    }

    /// Converts this manifest into a package with the given path.
    pub fn into_package(self, path: PathBuf) -> Option<Cargo> {
        match self.0.package.is_some() {
            true => Some(Cargo::new(self, path)),
            false => None,
        }
    }
}
