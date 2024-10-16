use std::fmt::{self, Debug};

use toml_edit::{Item, KeyMut, TableLike};

/// The cargo package dependency.
#[derive(Clone)]
pub struct Dependency<'a> {
    name: &'a str,
    version: Option<&'a str>,
    table: Option<&'a dyn TableLike>,
}

impl<'a> Dependency<'a> {
    /// Gets the dependency name.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the dependency version if it has been set.
    pub fn version(&self) -> Option<&'a str> {
        match self.version {
            Some(version) => Some(version),
            None => match self.table {
                Some(table) => match table.get("version") {
                    Some(version) => version.as_str(),
                    None => None,
                },
                None => None,
            },
        }
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
            .field("version", &self.version())
            .field("path", &self.path())
            .finish()
    }
}

impl<'a> From<(&'a str, &'a Item)> for Dependency<'a> {
    fn from((name, item): (&'a str, &'a Item)) -> Self {
        Self {
            name,
            version: item.as_str(),
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

/// The mutable cargo package dependency.
pub struct DependencyMut<'a> {
    name: KeyMut<'a>,
    item: &'a mut Item,
}

impl<'a> DependencyMut<'a> {
    /// Gets the dependency name.
    pub fn name(&self) -> &str {
        self.name.get()
    }

    /// Gets the dependency version if it has been set.
    pub fn version(&self) -> Option<&str> {
        match self.item.as_str() {
            Some(version) => Some(version),
            None => match self.item.as_table()?.get("version") {
                Some(version) => version.as_str(),
                None => None,
            },
        }
    }

    /// Sets the dependency version.
    pub fn set_version(&mut self, version: impl Into<String>) {
        if let Some(table) = self.item.as_table_like_mut() {
            if let Some(item) = table.get_mut("version") {
                if let Some(value) = item.as_value_mut() {
                    *value = version.into().into();

                    return;
                }
            }

            table.insert("version", Item::Value(version.into().into()));

            return;
        }

        if let Some(value) = self.item.as_value_mut() {
            *value = version.into().into();
        }
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&str> {
        match self.item.as_table()?.get("path") {
            Some(path) => path.as_str(),
            None => None,
        }
    }
}

impl<'a> Debug for DependencyMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DependencyMut")
            .field("name", &self.name())
            .field("version", &self.version())
            .field("path", &self.path())
            .finish()
    }
}

impl<'a> From<(KeyMut<'a>, &'a mut Item)> for DependencyMut<'a> {
    fn from((name, item): (KeyMut<'a>, &'a mut Item)) -> Self {
        Self { name, item }
    }
}

/// The mutable cargo package dependencies.
#[derive(Default)]
pub struct DependenciesMut<'a> {
    pub(super) table: Option<&'a mut dyn TableLike>,
}

impl<'a> DependenciesMut<'a> {
    /// Gets the mutable dependency with the given name.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.table
            .as_mut()?
            .iter_mut()
            .map(Into::<DependencyMut>::into)
            .find(|dependency| dependency.name() == name.as_ref())
    }
}

impl<'a> IntoIterator for DependenciesMut<'a> {
    type Item = DependencyMut<'a>;
    type IntoIter = Box<dyn Iterator<Item = DependencyMut<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.table {
            Some(table) => Box::new(table.iter_mut().map(Into::into)),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<'a> Debug for DependenciesMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.table {
            Some(table) => f
                .debug_list()
                .entries(Dependencies {
                    table: Some(&**table),
                })
                .finish(),
            None => f.debug_list().finish(),
        }
    }
}

impl<'a> From<&'a mut Item> for DependenciesMut<'a> {
    fn from(item: &'a mut Item) -> Self {
        Self {
            table: item.as_table_like_mut(),
        }
    }
}
