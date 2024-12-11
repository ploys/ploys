use toml_edit::{Item, TableLike, Value};

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
    pub fn repository(&self) -> RepoSpec {
        self.0
            .get("repository")
            .and_then(Item::as_str)
            .expect("repository")
            .parse()
            .expect("repository specification")
    }
}

impl<'a> Project<'a> {
    /// Constructs the project section from a table.
    pub(super) fn from_table(table: &'a dyn TableLike) -> Option<Self> {
        table.get("name").and_then(Item::as_str)?;
        table
            .get("repository")
            .and_then(Item::as_str)?
            .parse::<RepoSpec>()
            .ok()?;

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
    pub fn repository(&self) -> RepoSpec {
        self.0
            .get("repository")
            .and_then(Item::as_str)
            .expect("repository")
            .parse()
            .expect("repository specification")
    }

    /// Sets the project repository.
    pub fn set_repository(&mut self, repo: impl Into<RepoSpec>) -> &mut Self {
        let item = self.0.entry("repository").or_insert_with(Item::default);

        *item = Item::Value(Value::from(repo.into().to_string()));

        self
    }
}

impl<'a> ProjectMut<'a> {
    /// Constructs the mutable project section from a mutable table.
    pub(super) fn from_table(table: &'a mut dyn TableLike) -> Option<Self> {
        table.get("name").and_then(Item::as_str)?;
        table
            .get("repository")
            .and_then(Item::as_str)?
            .parse::<RepoSpec>()
            .ok()?;

        Some(Self(table))
    }
}
