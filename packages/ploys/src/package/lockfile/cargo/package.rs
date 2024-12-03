use semver::Version;
use toml_edit::{ArrayOfTables, Item, TableLike, Value};

/// The package table.
pub struct Package<'a>(&'a dyn TableLike);

impl Package<'_> {
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

/// The packages table array.
pub struct Packages<'a>(pub(super) Option<&'a ArrayOfTables>);

impl<'a> Packages<'a> {
    /// Gets a package with the given name.
    pub fn get(&'a self, package: &'a str) -> Option<Package<'a>> {
        match &self.0 {
            Some(arr) => arr
                .iter()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| Package(table)),
            None => None,
        }
    }
}

/// The mutable package table.
pub struct PackageMut<'a>(&'a mut dyn TableLike);

impl PackageMut<'_> {
    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        let item = self.0.entry("version").or_insert_with(Item::default);

        *item = Item::Value(Value::from(version.into().to_string()));

        self
    }
}

/// The mutable packages table array.
pub struct PackagesMut<'a>(pub(super) Option<&'a mut ArrayOfTables>);

impl<'a> PackagesMut<'a> {
    /// Gets a mutable package with the given name.
    pub fn get_mut(&'a mut self, package: &'a str) -> Option<PackageMut<'a>> {
        match &mut self.0 {
            Some(arr) => arr
                .iter_mut()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| PackageMut(table)),
            None => None,
        }
    }
}
