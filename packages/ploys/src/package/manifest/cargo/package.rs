use semver::Version;
use toml_edit::{Item, TableLike, Value};

/// The package table.
pub struct Package<'a>(pub(super) &'a dyn TableLike);

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
    ///
    /// This adheres to the [manifest format reference][1] and defaults to
    /// `0.0.0` if the `version` field has not been set.
    ///
    /// [1]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field
    pub fn version(&self) -> Version {
        self.0
            .get("version")
            .and_then(Item::as_str)
            .unwrap_or("0.0.0")
            .parse()
            .expect("version should be valid semver")
    }
}

/// The mutable package table.
pub struct PackageMut<'a>(pub(super) &'a mut dyn TableLike);

impl PackageMut<'_> {
    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        let item = self.0.entry("version").or_insert_with(Item::default);

        *item = Item::Value(Value::from(version.into().to_string()));

        self
    }
}
