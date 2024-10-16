use std::fmt::{self, Debug};

use toml_edit::TableLike;

/// The dependency item.
pub struct Dependency<'a>(&'a str, Option<&'a dyn TableLike>);

impl<'a> Dependency<'a> {
    /// Gets the dependency name.
    pub fn name(&self) -> &'a str {
        self.0
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&'a str> {
        match self.1 {
            Some(table) => match table.get("path") {
                Some(path) => path.as_str(),
                None => None,
            },
            None => None,
        }
    }
}

impl<'a> Debug for Dependency<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dependency")
            .field("name", &self.name())
            .field("path", &self.path())
            .finish()
    }
}

/// The dependencies table.
pub struct Dependencies<'a>(pub(super) Option<&'a dyn TableLike>);

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = Dependency<'a>;
    type IntoIter = Box<dyn Iterator<Item = Dependency<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(table) => Box::new(
                table
                    .iter()
                    .map(|(name, item)| Dependency(name, item.as_table_like())),
            ),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<'a> Debug for Dependencies<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(Self(self.0)).finish()
    }
}
