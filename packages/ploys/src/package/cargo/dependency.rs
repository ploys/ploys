use std::fmt::{self, Debug};

use toml_edit::{Item, TableLike};

/// The cargo package dependency.
#[derive(Clone)]
pub struct Dependency<'a> {
    name: &'a str,
    table: Option<&'a dyn TableLike>,
}

impl<'a> Dependency<'a> {
    /// Gets the dependency name.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&'a str> {
        match self.table {
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

impl<'a> From<(&'a str, &'a Item)> for Dependency<'a> {
    fn from((name, item): (&'a str, &'a Item)) -> Self {
        Self {
            name,
            table: item.as_table_like(),
        }
    }
}

/// The cargo package dependencies.
#[derive(Clone, Default)]
pub struct Dependencies<'a> {
    pub(super) table: Option<&'a dyn TableLike>,
}

impl<'a> Dependencies<'a> {
    /// Gets the dependency with the given name.
    pub fn get(&self, name: impl AsRef<str>) -> Option<Dependency<'a>> {
        self.clone()
            .into_iter()
            .find(|dependency| dependency.name() == name.as_ref())
    }
}

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = Dependency<'a>;
    type IntoIter = Box<dyn Iterator<Item = Dependency<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.table {
            Some(table) => Box::new(table.iter().map(Into::into)),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<'a> Debug for Dependencies<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a> From<&'a Item> for Dependencies<'a> {
    fn from(item: &'a Item) -> Self {
        Self {
            table: item.as_table_like(),
        }
    }
}
