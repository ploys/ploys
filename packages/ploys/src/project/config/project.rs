use either::Either;
use toml_edit::{Array, Item, TableLike, Value};

use crate::repository::RepoSpec;

/// The project table.
pub struct Project<'a>(&'a dyn TableLike);

impl<'a> Project<'a> {
    /// Gets the project name.
    pub fn name(&self) -> &'a str {
        self.0.get("name").and_then(Item::as_str).expect("name")
    }

    /// Gets the project description.
    pub fn description(&self) -> Option<&'a str> {
        self.0.get("description").and_then(Item::as_str)
    }

    /// Gets the project repository.
    pub fn repository(&self) -> Option<RepoSpec> {
        self.0.get("repository")?.as_str()?.parse().ok()
    }

    /// Gets the project authors.
    pub fn authors(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        match self.0.get("authors") {
            Some(item) => match item.as_array() {
                Some(arr) => Either::Right(arr.iter().filter_map(|value| value.as_str())),
                None => Either::Left(std::iter::empty()),
            },
            None => Either::Left(std::iter::empty()),
        }
    }
}

impl<'a> Project<'a> {
    /// Constructs the project section from a table.
    pub(super) fn from_table(table: &'a dyn TableLike) -> Option<Self> {
        table.get("name").and_then(Item::as_str)?;

        Some(Self(table))
    }
}

/// The mutable project table.
pub struct ProjectMut<'a>(&'a mut dyn TableLike);

impl ProjectMut<'_> {
    /// Gets the project name.
    pub fn name(&self) -> &str {
        self.0.get("name").and_then(Item::as_str).expect("name")
    }

    /// Sets the project name.
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        let item = self.0.entry("name").or_insert_with(Item::default);

        *item = Item::Value(Value::from(name.into()));

        self
    }

    /// Gets the project description.
    pub fn description(&self) -> Option<&str> {
        self.0.get("description").and_then(Item::as_str)
    }

    /// Sets the project description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        let item = self.0.entry("description").or_insert_with(Item::default);

        *item = Item::Value(Value::from(description.into()));

        self
    }

    /// Gets the project repository.
    pub fn repository(&self) -> Option<RepoSpec> {
        self.0.get("repository")?.as_str()?.parse().ok()
    }

    /// Sets the project repository.
    pub fn set_repository(&mut self, repo: impl Into<RepoSpec>) -> &mut Self {
        let item = self.0.entry("repository").or_insert_with(Item::default);

        *item = Item::Value(Value::from(repo.into().to_string()));

        self
    }

    /// Gets the project authors.
    pub fn authors(&self) -> impl Iterator<Item = &str> {
        match self.0.get("authors") {
            Some(item) => match item.as_array() {
                Some(arr) => Either::Right(arr.iter().filter_map(|value| value.as_str())),
                None => Either::Left(std::iter::empty()),
            },
            None => Either::Left(std::iter::empty()),
        }
    }

    /// Sets the project authors.
    pub fn set_authors(&mut self, authors: impl IntoIterator<Item = String>) -> &mut Self {
        let item = self.0.entry("authors").or_insert_with(Item::default);

        *item = Item::Value(Value::Array(Array::from_iter(authors)));

        self
    }
}

impl<'a> ProjectMut<'a> {
    /// Constructs the mutable project section from a mutable table.
    pub(super) fn from_table(table: &'a mut dyn TableLike) -> Option<Self> {
        table.get("name").and_then(Item::as_str)?;

        Some(Self(table))
    }
}
