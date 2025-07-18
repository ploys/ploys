//! The `Cargo.lock` package lockfile for Rust.

mod package;

use std::fmt::{self, Display};
use std::str::FromStr;

use semver::Version;
use toml_edit::{Decor, DocumentMut, Item, Key, value};

use crate::package::manifest::CargoManifest;

pub use self::package::{Packages, PackagesMut};

use super::Error;

/// The cargo package lockfile.
#[derive(Clone, Debug)]
pub struct CargoLockfile(DocumentMut);

impl CargoLockfile {
    /// Creates a new cargo lockfile.
    pub fn new() -> Self {
        let mut document = DocumentMut::new();

        let prefix = "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\n";
        let suffix = " ";
        let key = Key::new("version").with_leaf_decor(Decor::new(prefix, suffix));

        document.insert_formatted(&key, value(4));

        Self(document)
    }

    /// Adds a package to the lockfile.
    pub fn add_package(&mut self, manifest: &CargoManifest) -> &mut Self {
        self.packages_mut().insert(manifest);
        self
    }

    /// Builds the lockfile with a package.
    pub fn with_package(mut self, manifest: &CargoManifest) -> Self {
        self.add_package(manifest);
        self
    }

    /// Gets the package version.
    pub fn get_package_version(&self, package: impl AsRef<str>) -> Option<Version> {
        Some(self.packages().get(package.as_ref())?.version())
    }

    /// Sets the package version.
    pub fn set_package_version(&mut self, package: impl AsRef<str>, version: impl Into<Version>) {
        if let Some(mut package) = self.packages_mut().get_mut(package.as_ref()) {
            package.set_version(version);
        }
    }

    /// Gets the packages section.
    pub fn packages(&self) -> Packages<'_> {
        Packages(self.0.get("package").and_then(Item::as_array_of_tables))
    }

    /// Gets the mutable packages section.
    pub fn packages_mut(&mut self) -> PackagesMut<'_> {
        PackagesMut::new(self.0.entry("package"))
    }
}

impl CargoLockfile {
    /// Creates a manifest from the given bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        std::str::from_utf8(bytes)?.parse()
    }
}

impl Default for CargoLockfile {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for CargoLockfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq for CargoLockfile {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Eq for CargoLockfile {}

impl FromStr for CargoLockfile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

    use crate::package::manifest::CargoManifest;
    use crate::package::manifest::cargo::Dependency;

    use super::CargoLockfile;

    #[test]
    fn test_package_version() {
        let mut document = DocumentMut::new();

        document["version"] = value(3);
        document["package"] = {
            let mut tables = ArrayOfTables::new();
            let mut table = Table::new();

            table["name"] = value("example");
            table["version"] = value("0.0.0");

            tables.push(table);

            Item::ArrayOfTables(tables)
        };

        let mut lockfile = CargoLockfile(document);

        assert_eq!(
            lockfile.get_package_version("example"),
            Some(Version::new(0, 0, 0))
        );

        lockfile.set_package_version("example", Version::new(0, 1, 0));

        assert_eq!(
            lockfile.get_package_version("example"),
            Some(Version::new(0, 1, 0))
        );
    }

    #[test]
    fn test_builder() {
        let package_a = CargoManifest::new_package("package-a");
        let package_b = CargoManifest::new_package("package-b")
            .with_dependency(Dependency::new("example").with_version(Version::new(0, 1, 0)))
            .with_dependency(Dependency::new("package-a").with_path("../package-a"));
        let package_c = CargoManifest::new_package("a-package-c")
            .with_dependency(Dependency::new("package-a").with_path("../package-a"))
            .with_dependency(Dependency::new("package-b").with_path("../package-b"));

        let lockfile = CargoLockfile::new()
            .with_package(&package_a)
            .with_package(&package_b)
            .with_package(&package_c);

        assert!(lockfile.packages().get("package-a").is_some());
        assert!(lockfile.packages().get("package-b").is_some());
        assert!(lockfile.packages().get("a-package-c").is_some());
    }
}
