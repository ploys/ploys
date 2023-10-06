use std::path::PathBuf;

use globset::{Glob, GlobSetBuilder};
use toml_edit::{Array, Document, TableLike, Value};

use crate::package::members::Members;

use super::error::Error;
use super::Cargo;

/// The cargo package manifest.
#[derive(Debug)]
pub struct Manifest(Document);

impl Manifest {
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

    /// Gets the dependencies table.
    pub fn dependencies(&self) -> Dependencies<'_> {
        match self.0.get("dependencies") {
            Some(item) => Dependencies(item.as_table_like()),
            None => Dependencies(None),
        }
    }

    /// Gets the dev dependencies table.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        match self.0.get("dev-dependencies") {
            Some(item) => Dependencies(item.as_table_like()),
            None => Dependencies(None),
        }
    }

    /// Gets the build dependencies table.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        match self.0.get("build-dependencies") {
            Some(item) => Dependencies(item.as_table_like()),
            None => Dependencies(None),
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
        Ok(Self(std::str::from_utf8(bytes)?.parse()?))
    }

    /// Converts this manifest into a package with the given path.
    pub fn into_package(self, path: PathBuf) -> Option<Cargo> {
        match self.package().is_some() {
            true => Some(Cargo::new(self, path)),
            false => None,
        }
    }
}

/// The workspace table.
pub struct Workspace<'a>(&'a dyn TableLike);

impl<'a> Workspace<'a> {
    /// Gets the workspace members.
    pub fn members(&self) -> WorkspaceMembers<'a> {
        match self.0.get("members") {
            Some(item) => WorkspaceMembers(item.as_array()),
            None => WorkspaceMembers(None),
        }
    }

    /// Gets the workspace excludes.
    pub fn exclude(&self) -> WorkspaceExclude<'a> {
        match self.0.get("exclude") {
            Some(item) => WorkspaceExclude(item.as_array()),
            None => WorkspaceExclude(None),
        }
    }
}

/// The workspace members array.
pub struct WorkspaceMembers<'a>(Option<&'a Array>);

impl<'a> IntoIterator for WorkspaceMembers<'a> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(arr) => Box::new(arr.into_iter().flat_map(Value::as_str)),
            None => Box::new(std::iter::empty()),
        }
    }
}

/// The workspace excludes array.
pub struct WorkspaceExclude<'a>(Option<&'a Array>);

impl<'a> IntoIterator for WorkspaceExclude<'a> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(arr) => Box::new(arr.into_iter().flat_map(Value::as_str)),
            None => Box::new(std::iter::empty()),
        }
    }
}

/// The package table.
pub struct Package<'a>(&'a dyn TableLike);

impl<'a> Package<'a> {
    /// Gets the package name.
    pub fn name(&self) -> &'a str {
        self.0.get("name").expect("name").as_str().expect("name")
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&'a str> {
        match self.0.get("description") {
            Some(description) => Some(description.as_str().expect("description")),
            None => None,
        }
    }

    /// Gets the package version.
    pub fn version(&self) -> &'a str {
        self.0
            .get("version")
            .expect("version")
            .as_str()
            .expect("version")
    }
}

/// The dependencies table.
pub struct Dependencies<'a>(Option<&'a dyn TableLike>);

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = Dependency<'a>;
    type IntoIter = Box<dyn Iterator<Item = Dependency<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(table) => Box::new(
                table
                    .iter()
                    .map(|(name, item)| Dependency(name, item.as_table_like())),
            ),
            None => Box::new(std::iter::empty()),
        }
    }
}

/// The dependency item.
pub struct Dependency<'a>(&'a str, Option<&'a dyn TableLike>);

impl<'a> Dependency<'a> {
    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&'a str> {
        match self.1 {
            Some(table) => match table.get("path") {
                Some(path) => path.as_str(),
                None => None,
            },
            None => None,
        }
    }
}
