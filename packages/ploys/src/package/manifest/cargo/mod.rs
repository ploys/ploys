//! The `Cargo.toml` package manifest for Rust.

mod dependency;
mod package;
mod workspace;

use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use globset::{Glob, GlobSetBuilder};
use toml_edit::{Array, DocumentMut, Item, Table, value};

use crate::package::manifest::Members;

pub use self::dependency::{
    Dependencies, DependenciesMut, Dependency, DependencyMut, DependencyRef,
};
pub use self::package::{Package, PackageMut};
pub use self::workspace::{Workspace, WorkspaceExclude, WorkspaceMembers, WorkspaceMut};

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

                    table.insert("name", value(name.into()));
                    table.insert("version", value("0.0.0"));
                    table.insert("edition", value("2024"));
                    table
                }),
            );
            document
        })
    }

    /// Constructs a new cargo workspace manifest.
    pub fn new_workspace() -> Self {
        Self({
            let mut document = DocumentMut::new();

            document.insert(
                "workspace",
                Item::Table({
                    let mut table = Table::new();

                    table.insert("resolver", value("3"));
                    table.insert("members", value(Array::new()));
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

    /// Gets the mutable workspace table.
    pub fn workspace_mut(&mut self) -> WorkspaceMut<'_> {
        WorkspaceMut::new(self.0.entry("workspace"))
    }

    /// Adds a new workspace member.
    pub fn add_workspace_member(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.workspace_mut().add_member(path);
        self
    }

    /// Builds the workspace with the given member.
    pub fn with_workspace_member(mut self, path: impl AsRef<Path>) -> Self {
        self.add_workspace_member(path);
        self
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
    /// Adds a new dependency to the manifest.
    pub fn add_dependency(&mut self, dependency: impl Into<Dependency>) -> &mut Self {
        self.dependencies_mut().insert(dependency);
        self
    }

    /// Builds the manifest with the given dependency.
    pub fn with_dependency(mut self, dependency: impl Into<Dependency>) -> Self {
        self.add_dependency(dependency);
        self
    }

    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
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
        DependenciesMut::new(self.0.entry("dependencies"))
    }
}

impl CargoManifest {
    /// Adds a new dev dependency to the manifest.
    pub fn add_dev_dependency(&mut self, dependency: impl Into<Dependency>) -> &mut Self {
        self.dev_dependencies_mut().insert(dependency);
        self
    }

    /// Builds the manifest with the given dev dependency.
    pub fn with_dev_dependency(mut self, dependency: impl Into<Dependency>) -> Self {
        self.add_dev_dependency(dependency);
        self
    }

    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
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
        DependenciesMut::new(self.0.entry("dev-dependencies"))
    }
}

impl CargoManifest {
    /// Adds a new build dependency to the manifest.
    pub fn add_build_dependency(&mut self, dependency: impl Into<Dependency>) -> &mut Self {
        self.build_dependencies_mut().insert(dependency);
        self
    }

    /// Builds the manifest with the given build dependency.
    pub fn with_build_dependency(mut self, dependency: impl Into<Dependency>) -> Self {
        self.add_build_dependency(dependency);
        self
    }

    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
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
        DependenciesMut::new(self.0.entry("build-dependencies"))
    }
}

impl Default for CargoManifest {
    fn default() -> Self {
        Self::new_workspace()
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
    use std::path::Path;

    use semver::Version;
    use toml_edit::{DocumentMut, value};

    use crate::package::manifest::cargo::Dependency;

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

    #[test]
    fn test_dependencies() {
        let mut manifest = CargoManifest::new_package("example");

        assert_eq!(manifest.dependencies().into_iter().count(), 0);
        assert_eq!(manifest.dependencies_mut().into_iter().count(), 0);

        assert!(manifest.0.get("dependencies").is_none());

        manifest.add_dependency(Dependency::new("example-one").with_version(Version::new(0, 1, 0)));

        assert_eq!(manifest.dependencies().into_iter().count(), 1);
        assert_eq!(manifest.dependencies_mut().into_iter().count(), 1);

        let one = manifest.get_dependency("example-one").unwrap();

        assert_eq!(one.name(), "example-one");
        assert_eq!(one.version(), Some("0.1.0"));
        assert_eq!(one.path(), None);

        manifest.add_dependency(Dependency::new("example-two").with_path("../example-two"));

        assert_eq!(manifest.dependencies().into_iter().count(), 2);
        assert_eq!(manifest.dependencies_mut().into_iter().count(), 2);

        let two = manifest.get_dependency("example-two").unwrap();

        assert_eq!(two.name(), "example-two");
        assert_eq!(two.version(), None);
        assert_eq!(two.path(), Some("../example-two"));

        manifest.add_dependency(
            Dependency::new("example-three")
                .with_version(Version::new(0, 2, 0))
                .with_path("../example-three"),
        );

        assert_eq!(manifest.dependencies().into_iter().count(), 3);
        assert_eq!(manifest.dependencies_mut().into_iter().count(), 3);

        let three = manifest.get_dependency("example-three").unwrap();

        assert_eq!(three.name(), "example-three");
        assert_eq!(three.version(), Some("0.2.0"));
        assert_eq!(three.path(), Some("../example-three"));
    }

    #[test]
    fn test_members() {
        let mut manifest = CargoManifest::new_workspace();

        manifest.add_workspace_member("packages/*");
        manifest.add_workspace_member("packages/example");
        manifest.add_workspace_member("examples/example");

        let members = manifest.members().unwrap();

        assert!(members.includes(Path::new("packages/example")));
        assert!(members.includes(Path::new("examples/example")));

        let expected = indoc::indoc! {r#"
            [workspace]
            resolver = "3"
            members = ["packages/*", "examples/example"]
        "#};

        assert_eq!(manifest.to_string(), expected);
    }
}
