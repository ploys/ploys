//! The `Cargo.lock` package lockfile for Rust.

mod package;

use std::fmt::{self, Display};
use std::str::FromStr;

use semver::Version;
use toml_edit::{DocumentMut, Item};

pub use self::package::{Packages, PackagesMut};

use super::Error;

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

    /// Gets the packages section.
    pub fn packages(&self) -> Packages<'_> {
        Packages(self.0.get("package").and_then(Item::as_array_of_tables))
    }

    /// Gets the mutable packages section.
    pub fn packages_mut(&mut self) -> PackagesMut<'_> {
        PackagesMut(
            self.0
                .get_mut("package")
                .and_then(Item::as_array_of_tables_mut),
        )
    }
}

impl CargoLockfile {
    /// Creates a manifest from the given bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        std::str::from_utf8(bytes)?.parse()
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
