//! The `Cargo.toml` package manifest for Rust.

mod dependency;
mod package;
mod workspace;

use std::fmt::{self, Display};
use std::path::PathBuf;
use std::str::FromStr;

use globset::{Glob, GlobSetBuilder};
use toml_edit::{DocumentMut, Item, Table, Value};

use crate::package::manifest::Members;

pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::package::{Package, PackageMut};
pub use self::workspace::{Workspace, WorkspaceExclude, WorkspaceMembers};

use super::Error;

/// The cargo package manifest.
#[derive(Clone, Debug)]
pub struct CargoManifest(DocumentMut);

impl CargoManifest {
    /// Constructs a new cargo package manifest.
    pub fn new_package(name: impl Into<String>) -> Self {
        Self({
            let mut document = DocumentMut::new();

            document.insert(
                "package",
                Item::Table({
                    let mut table = Table::new();

                    table.insert("name", Item::Value(Value::from(name.into())));
                    table
                }),
            );
            document
        })
    }

    /// Gets the workspace table.
    pub fn workspace(&self) -> Option<Workspace<'_>> {
        match self.0.get("workspace") {
            Some(item) => item.as_table_like().map(Workspace),
            None => None,
        }
    }

    /// Gets the package table.
    pub fn package(&self) -> Option<Package<'_>> {
        match self.0.get("package") {
            Some(item) => item.as_table_like().map(Package),
            None => None,
        }
    }

    /// Gets the mutable package table.
    pub fn package_mut(&mut self) -> Option<PackageMut<'_>> {
        match self.0.get_mut("package") {
            Some(item) => item.as_table_like_mut().map(PackageMut),
            None => None,
        }
    }

    /// Gets the workspace members.
    ///
    /// This follows the [members and exclude fields](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields)
    /// documentation to build a list of included and excluded members.
    pub fn members(&self) -> Result<Members, Error> {
        let mut includes = GlobSetBuilder::new();
        let mut excludes = Vec::new();

        if let Some(workspace) = self.workspace() {
            for member in workspace.members() {
                includes.add(Glob::new(member.trim_start_matches("./"))?);
            }

            for path in workspace.exclude() {
                excludes.push(PathBuf::from(path));
            }
        }

        let dependencies = self
            .dependencies()
            .into_iter()
            .chain(self.dev_dependencies())
            .chain(self.build_dependencies());

        for dependency in dependencies {
            if let Some(path) = dependency.path() {
                includes.add(Glob::new(path.trim_start_matches("./"))?);
            }
        }

        Ok(Members::new(includes.build()?, excludes))
    }

    /// Creates a manifest from the given bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        std::str::from_utf8(bytes)?.parse()
    }
}

impl CargoManifest {
    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.dependencies().get(name)
    }

    /// Gets the mutable dependency with the given name.
    pub fn get_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.dependencies_mut().into_get_mut(name)
    }

    /// Gets the dependencies table.
    pub fn dependencies(&self) -> Dependencies<'_> {
        self.0
            .get("dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }

    /// Gets the mutable dependencies table.
    pub fn dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.0
            .get_mut("dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }
}

impl CargoManifest {
    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.dev_dependencies().get(name)
    }

    /// Gets the mutable dev dependency with the given name.
    pub fn get_dev_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.dev_dependencies_mut().into_get_mut(name)
    }

    /// Gets the dev dependencies table.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        self.0
            .get("dev-dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }

    /// Gets the mutable dev dependencies table.
    pub fn dev_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.0
            .get_mut("dev-dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }
}

impl CargoManifest {
    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.build_dependencies().get(name)
    }

    /// Gets the mutable build dependency with the given name.
    pub fn get_build_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.build_dependencies_mut().into_get_mut(name)
    }

    /// Gets the build dependencies table.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        self.0
            .get("build-dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }

    /// Gets the mutable build dependencies table.
    pub fn build_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.0
            .get_mut("build-dependencies")
            .map(Into::into)
            .unwrap_or_default()
    }
}

impl Display for CargoManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq for CargoManifest {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Eq for CargoManifest {}

impl FromStr for CargoManifest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use toml_edit::{value, DocumentMut};

    use super::CargoManifest;

    #[test]
    fn test_package_version() {
        let mut document = DocumentMut::new();

        document["package"]["name"] = value("example");

        let mut manifest = CargoManifest(document);

        assert_eq!(manifest.package().unwrap().version(), Version::new(0, 0, 0));

        manifest
            .package_mut()
            .unwrap()
            .set_version(Version::new(0, 1, 0));

        assert_eq!(manifest.package().unwrap().version(), Version::new(0, 1, 0));

        manifest
            .package_mut()
            .unwrap()
            .set_version(Version::new(0, 2, 0));

        assert_eq!(manifest.package().unwrap().version(), Version::new(0, 2, 0));
    }
}
