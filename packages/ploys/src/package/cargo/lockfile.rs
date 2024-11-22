//! The `Cargo.lock` lockfile for Rust.

use std::fmt::{self, Display};

use toml_edit::{ArrayOfTables, DocumentMut, Item, TableLike, Value};

use crate::package::cargo::Error;

/// A `Cargo.lock` lockfile for Rust.
#[derive(Clone, Debug)]
pub struct CargoLockfile {
    manifest: DocumentMut,
}

impl CargoLockfile {
    /// Sets the package version.
    pub fn set_package_version<P, V>(&mut self, package: P, version: V)
    where
        P: AsRef<str>,
        V: Into<String>,
    {
        if let Some(mut package) = self.packages_mut().get_mut(package.as_ref()) {
            package.set_version(version);
        }
    }

    /// Creates a manifest from the given bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            manifest: std::str::from_utf8(bytes)?.parse()?,
        })
    }

    fn packages_mut(&mut self) -> PackagesMut<'_> {
        PackagesMut(
            self.manifest
                .get_mut("package")
                .and_then(Item::as_array_of_tables_mut),
        )
    }
}

impl Display for CargoLockfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.manifest, f)
    }
}

impl PartialEq for CargoLockfile {
    fn eq(&self, other: &Self) -> bool {
        self.manifest.to_string() == other.manifest.to_string()
    }
}

impl Eq for CargoLockfile {}

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
    fn set_version<V>(&mut self, version: V) -> &mut Self
    where
        V: Into<String>,
    {
        *self.0.get_mut("version").expect("version") = Item::Value(Value::from(version.into()));
        self
    }
}
