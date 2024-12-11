//! Project Configuration
//!
//! This module includes the project configuration file format that is based on
//! the TOML file format.

mod error;
mod project;

use std::fmt::{self, Display};

use toml_edit::{DocumentMut, Item};

pub use self::error::Error;
pub use self::project::{Project, ProjectMut};

/// The project configuration.
#[derive(Clone, Debug)]
pub struct Config(DocumentMut);

impl Config {
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
        let document: DocumentMut = std::str::from_utf8(bytes)?.parse()?;

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
