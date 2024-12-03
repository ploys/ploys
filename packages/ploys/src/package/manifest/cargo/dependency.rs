use std::fmt::{self, Debug};

use semver::Version;
use toml_edit::{Item, KeyMut, TableLike, Value};

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

impl Debug for Dependency<'_> {
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

impl Debug for Dependencies<'_> {
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

impl DependencyMut<'_> {
    /// Gets the dependency name.
    pub fn name(&self) -> &str {
        self.name.get()
    }

    /// Gets the dependency version if it has been set.
    pub fn version(&self) -> Option<Version> {
        match self.item.as_str() {
            Some(version) => version.parse().ok(),
            None => match self.item.as_table()?.get("version") {
                Some(version) => version.as_str()?.parse().ok(),
                None => None,
            },
        }
    }

    /// Sets the dependency version.
    pub fn set_version(&mut self, version: impl Into<Version>) {
        if let Some(table) = self.item.as_table_like_mut() {
            let item = table.entry("version").or_insert_with(Item::default);

            *item = Item::Value(Value::from(version.into().to_string()));
        } else if let Some(value) = self.item.as_value_mut() {
            *value = Value::from(version.into().to_string());
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

impl Debug for DependencyMut<'_> {
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

    /// Gets the mutable dependency with the given name.
    pub fn into_get_mut(self, name: impl AsRef<str>) -> Option<DependencyMut<'a>> {
        self.table?
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

impl Debug for DependenciesMut<'_> {
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
