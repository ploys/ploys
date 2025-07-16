use semver::Version;
use toml_edit::{Array, Entry, Item, TableLike, Value, value};
use url::Url;

/// The package table.
pub struct Package<'a>(pub(super) &'a dyn TableLike);

impl<'a> Package<'a> {
    /// Gets the package name.
    pub fn name(&self) -> &'a str {
        self.0.get("name").expect("name").as_str().expect("name")
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&'a str> {
        self.0.get("description").and_then(Item::as_str)
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

    /// Gets the package repository.
    pub fn repository(&self) -> Option<Url> {
        self.0.get("repository")?.as_str()?.parse().ok()
    }

    /// Gets the package authors.
    pub fn authors(&self) -> Option<impl IntoIterator<Item = &'a str> + use<'a>> {
        Some(
            self.0
                .get("authors")?
                .as_array()?
                .iter()
                .flat_map(Value::as_str),
        )
    }
}

/// The mutable package table.
pub struct PackageMut<'a>(pub(super) &'a mut dyn TableLike);

impl PackageMut<'_> {
    /// Gets the package name.
    pub fn name(&self) -> &str {
        self.0.get("name").expect("name").as_str().expect("name")
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&str> {
        self.0.get("description").and_then(Item::as_str)
    }

    /// Sets the package description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        let item = self.0.entry("description").or_insert_with(Item::default);

        *item = Item::Value(Value::from(description.into()));

        self
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

    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        let item = self.0.entry("version").or_insert_with(Item::default);

        *item = Item::Value(Value::from(version.into().to_string()));

        self
    }

    /// Gets the package repository.
    pub fn repository(&self) -> Option<Url> {
        self.0.get("repository")?.as_str()?.parse().ok()
    }

    /// Sets the package repository.
    pub fn set_repository(&mut self, repository: impl Into<Url>) -> &mut Self {
        let item = self.0.entry("repository").or_insert_with(Item::default);

        *item = Item::Value(Value::from(repository.into().to_string()));

        self
    }

    /// Gets the package authors.
    pub fn authors(&self) -> Option<impl IntoIterator<Item = &str>> {
        Some(
            self.0
                .get("authors")?
                .as_array()?
                .iter()
                .flat_map(Value::as_str),
        )
    }

    /// Adds an author to the package.
    pub fn add_author(&mut self, author: impl Into<String>) -> &mut Self {
        match self.0.entry("authors") {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Item::Value(Value::Array(array)) => {
                    array.push(author.into());
                }
                item => {
                    *item = value(Array::from_iter([author.into()]));
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(value(Array::from_iter([author.into()])));
            }
        }

        self
    }
}
