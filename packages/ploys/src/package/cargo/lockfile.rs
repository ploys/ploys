//! The `Cargo.lock` lockfile for Rust.

use std::fmt::{self, Display};

use semver::Version;
use toml_edit::{ArrayOfTables, DocumentMut, Item, TableLike, Value};

use crate::package::cargo::Error;

/// The cargo package lockfile.
#[derive(Clone, Debug)]
pub struct CargoLockfile(DocumentMut);

impl CargoLockfile {
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

    /// Creates a manifest from the given bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self(std::str::from_utf8(bytes)?.parse()?))
    }

    fn packages(&self) -> Packages<'_> {
        Packages(self.0.get("package").and_then(Item::as_array_of_tables))
    }

    fn packages_mut(&mut self) -> PackagesMut<'_> {
        PackagesMut(
            self.0
                .get_mut("package")
                .and_then(Item::as_array_of_tables_mut),
        )
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

/// The packages table.
struct Packages<'a>(Option<&'a ArrayOfTables>);

impl<'a> Packages<'a> {
    fn get(&'a self, package: &'a str) -> Option<Package<'a>> {
        match &self.0 {
            Some(arr) => arr
                .iter()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| Package(table)),
            None => None,
        }
    }
}

/// The package table.
struct Package<'a>(&'a dyn TableLike);

impl<'a> Package<'a> {
    /// Gets the package version.
    pub fn version(&self) -> Version {
        self.0
            .get("version")
            .and_then(Item::as_str)
            .unwrap_or("0.0.0")
            .parse()
            .expect("version should be valid semver")
    }
}

/// The mutable packages table.
struct PackagesMut<'a>(Option<&'a mut ArrayOfTables>);

impl<'a> PackagesMut<'a> {
    fn get_mut(&'a mut self, package: &'a str) -> Option<PackageMut<'a>> {
        match &mut self.0 {
            Some(arr) => arr
                .iter_mut()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| PackageMut(table)),
            None => None,
        }
    }
}

/// The mutable package table.
struct PackageMut<'a>(&'a mut dyn TableLike);

impl<'a> PackageMut<'a> {
    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        let item = self.0.entry("version").or_insert_with(Item::default);

        *item = Item::Value(Value::from(version.into().to_string()));

        self
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

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
}
