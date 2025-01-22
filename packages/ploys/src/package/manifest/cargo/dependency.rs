use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};

use either::Either;
use semver::Version;
use toml_edit::{value, Entry, InlineTable, Item, KeyMut, Table, TableLike, Value};

/// A *Cargo* package dependency.
pub struct Dependency {
    name: String,
    version: Option<Version>,
    path: Option<PathBuf>,
}

impl Dependency {
    /// Creates a new dependency.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            path: None,
        }
    }

    /// Gets the dependency name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the dependency version.
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// Sets the dependency version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        self.version = Some(version.into());
        self
    }

    /// Builds the dependency with the given version.
    pub fn with_version(mut self, version: impl Into<Version>) -> Self {
        self.set_version(version);
        self
    }

    /// Gets the dependency path.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Sets the dependency path.
    pub fn set_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    /// Builds the dependency with the given path.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.set_path(path);
        self
    }
}

/// The cargo package dependency.
#[derive(Clone)]
pub struct DependencyRef<'a> {
    name: &'a str,
    version: Option<&'a str>,
    table: Option<&'a dyn TableLike>,
}

impl<'a> DependencyRef<'a> {
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

impl Debug for DependencyRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DependencyRef")
            .field("name", &self.name())
            .field("version", &self.version())
            .field("path", &self.path())
            .finish()
    }
}

impl<'a> From<(&'a str, &'a Item)> for DependencyRef<'a> {
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
    pub fn get(&self, name: impl AsRef<str>) -> Option<DependencyRef<'a>> {
        self.clone()
            .into_iter()
            .find(|dependency| dependency.name() == name.as_ref())
    }
}

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = DependencyRef<'a>;
    type IntoIter = Box<dyn Iterator<Item = DependencyRef<'a>> + 'a>;

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
pub struct DependenciesMut<'a> {
    table: Either<&'a mut dyn TableLike, Option<Entry<'a>>>,
}

impl<'a> DependenciesMut<'a> {
    pub(super) fn new(entry: Entry<'a>) -> Self {
        Self {
            table: Either::Right(Some(entry)),
        }
    }
}

impl<'a> DependenciesMut<'a> {
    /// Gets the mutable dependency with the given name.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.table(false)?
            .iter_mut()
            .map(Into::<DependencyMut>::into)
            .find(|dependency| dependency.name() == name.as_ref())
    }

    /// Gets the mutable dependency with the given name.
    pub fn into_get_mut(self, name: impl AsRef<str>) -> Option<DependencyMut<'a>> {
        self.into_table(false)?
            .iter_mut()
            .map(Into::<DependencyMut>::into)
            .find(|dependency| dependency.name() == name.as_ref())
    }

    /// Inserts a dependency.
    pub fn insert(&mut self, dependency: impl Into<Dependency>) {
        let dependency = dependency.into();
        let dependencies = self.table(true).expect("table");
        let is_table = dependency.path().is_some();

        dependencies.remove(dependency.name());

        if is_table {
            let mut table = InlineTable::new();

            if let Some(version) = dependency.version() {
                table.insert("version", Value::from(version.to_string()));
            }

            if let Some(path) = dependency.path() {
                table.insert("path", Value::from(path.display().to_string()));
            }

            dependencies.insert(dependency.name(), Item::Value(Value::InlineTable(table)));
        } else if let Some(version) = dependency.version() {
            dependencies.insert(dependency.name(), value(version.to_string()));
        }
    }
}

impl<'a> DependenciesMut<'a> {
    fn init_table(&mut self, overwrite: bool) -> Option<()> {
        if let Either::Right(option) = &mut self.table {
            match option.take().expect("some") {
                Entry::Occupied(entry) if entry.get().as_table_like().is_some() => {
                    self.table = Either::Left(entry.into_mut().as_table_like_mut().expect("table"));
                }
                Entry::Occupied(mut entry) if overwrite => {
                    *entry.get_mut() = Item::Table(Table::new());

                    self.table = Either::Left(entry.into_mut().as_table_like_mut().expect("table"));
                }
                Entry::Vacant(entry) if overwrite => {
                    self.table = Either::Left(
                        entry
                            .insert(Item::Table(Table::new()))
                            .as_table_like_mut()
                            .expect("table"),
                    );
                }
                entry => {
                    option.replace(entry);

                    return None;
                }
            }
        }

        Some(())
    }

    fn table(&mut self, overwrite: bool) -> Option<&mut dyn TableLike> {
        self.init_table(overwrite)?;

        match self.table.as_mut().left() {
            Some(table) => Some(*table),
            None => None,
        }
    }

    fn into_table(mut self, overwrite: bool) -> Option<&'a mut dyn TableLike> {
        self.init_table(overwrite)?;
        self.table.left()
    }
}

impl<'a> IntoIterator for DependenciesMut<'a> {
    type Item = DependencyMut<'a>;
    type IntoIter = Box<dyn Iterator<Item = DependencyMut<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.into_table(false) {
            Some(table) => Box::new(table.iter_mut().map(Into::into)),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl Debug for DependenciesMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.table {
            Either::Left(table) => f
                .debug_list()
                .entries(Dependencies {
                    table: Some(&**table),
                })
                .finish(),
            Either::Right(_) => f.debug_list().finish(),
        }
    }
}
