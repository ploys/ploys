//! Project Configuration
//!
//! This module includes the project configuration file format that is based on
//! the TOML file format.

mod error;
mod project;

use std::fmt::{self, Display};
use std::str::FromStr;

use toml_edit::{value, DocumentMut, Item, Table};

use crate::repository::RepoSpec;

pub use self::error::Error;
pub use self::project::{Project, ProjectMut};

/// The project configuration.
#[derive(Clone, Debug)]
pub struct Config(DocumentMut);

impl Config {
    /// Creates a new project config.
    pub fn new(name: impl Into<String>) -> Self {
        let mut document = DocumentMut::new();

        document.insert(
            "project",
            Item::Table({
                let mut table = Table::new();

                table.insert("name", value(name.into()));
                table
            }),
        );

        Self(document)
    }

    /// Gets the project name.
    pub fn name(&self) -> &str {
        self.project().name()
    }

    /// Gets the project description.
    pub fn description(&self) -> Option<&str> {
        self.project().description()
    }

    /// Sets the project description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.project_mut().set_description(description);
        self
    }

    /// Builds the config with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    /// Gets the project repository.
    pub fn repository(&self) -> Option<RepoSpec> {
        self.project().repository()
    }

    /// Sets the project repository.
    pub fn set_repository(&mut self, repository: impl Into<RepoSpec>) -> &mut Self {
        self.project_mut().set_repository(repository);
        self
    }

    /// Builds the config with the given repository.
    pub fn with_repository(mut self, repository: impl Into<RepoSpec>) -> Self {
        self.set_repository(repository);
        self
    }

    /// The project section.
    pub fn project(&self) -> Project<'_> {
        Project::from_table(
            self.0
                .get("project")
                .and_then(Item::as_table_like)
                .expect("project"),
        )
        .expect("project table")
    }

    /// The mutable project section.
    pub fn project_mut(&mut self) -> ProjectMut<'_> {
        ProjectMut::from_table(
            self.0
                .get_mut("project")
                .and_then(Item::as_table_like_mut)
                .expect("project"),
        )
        .expect("project table")
    }
}

impl Config {
    /// Constructs config from the given bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        std::str::from_utf8(bytes)?.parse()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Eq for Config {}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let document: DocumentMut = s.parse()?;

        Project::from_table(
            document
                .get("project")
                .and_then(Item::as_table_like)
                .ok_or(Error::Invalid)?,
        )
        .ok_or(Error::Invalid)?;

        Ok(Self(document))
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::RepoSpec;

    use super::Config;

    #[test]
    fn test_builder() {
        let config = Config::new("example")
            .with_description("An example repository.")
            .with_repository("ploys/example".parse::<RepoSpec>().unwrap());

        assert_eq!(config.name(), "example");
        assert_eq!(config.description().unwrap(), "An example repository.");
        assert_eq!(
            config.repository().unwrap(),
            "ploys/example".parse::<RepoSpec>().unwrap()
        );

        let expected = indoc::indoc! {r#"
            [project]
            name = "example"
            description = "An example repository."
            repository = "ploys/example"
        "#};

        assert_eq!(config.to_string(), expected);
    }
}
