use either::Either;
use semver::Version;
use toml_edit::{Array, ArrayOfTables, Entry, Formatted, Item, Table, TableLike, Value, value};

use crate::package::manifest::CargoManifest;

/// The package table.
pub struct Package<'a>(&'a dyn TableLike);

impl Package<'_> {
    /// Gets the package version.
    pub fn version(&self) -> Version {
        self.0
            .get("version")
            .and_then(Item::as_str)
            .unwrap_or("0.0.0")
            .parse()
            .expect("version should be valid semver")
    }
}

/// The packages table array.
pub struct Packages<'a>(pub(super) Option<&'a ArrayOfTables>);

impl<'a> Packages<'a> {
    /// Gets a package with the given name.
    pub fn get(&'a self, package: &'a str) -> Option<Package<'a>> {
        match &self.0 {
            Some(arr) => arr
                .iter()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| Package(table)),
            None => None,
        }
    }
}

/// The mutable package table.
pub struct PackageMut<'a>(&'a mut dyn TableLike);

impl PackageMut<'_> {
    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        let item = self.0.entry("version").or_insert_with(Item::default);

        *item = Item::Value(Value::from(version.into().to_string()));

        self
    }
}

/// The mutable packages table array.
pub struct PackagesMut<'a> {
    array: Either<&'a mut ArrayOfTables, Option<Entry<'a>>>,
}

impl<'a> PackagesMut<'a> {
    pub(super) fn new(entry: Entry<'a>) -> Self {
        Self {
            array: Either::Right(Some(entry)),
        }
    }
}

impl<'a> PackagesMut<'a> {
    /// Gets a mutable package with the given name.
    pub fn get_mut(&'a mut self, package: &'a str) -> Option<PackageMut<'a>> {
        match self.array(false) {
            Some(arr) => arr
                .iter_mut()
                .find(|table| table.get("name").and_then(Item::as_str) == Some(package))
                .map(|table| PackageMut(table)),
            None => None,
        }
    }

    /// Inserts a package.
    pub fn insert(&mut self, manifest: &CargoManifest) {
        let packages = self.array(true).expect("array");

        if let Some(package) = manifest.package() {
            let mut table = Table::new();

            table.insert("name", value(package.name()));
            table.insert("version", value(package.version().to_string()));

            let deps = manifest
                .dependencies()
                .into_iter()
                .chain(manifest.dev_dependencies())
                .map(|dependency| {
                    Value::String(Formatted::new(dependency.name().to_string()))
                        .decorated("\n ", "")
                })
                .collect::<Vec<_>>();

            if !deps.is_empty() {
                let mut array = Array::from_iter(deps);

                array.set_trailing("\n");
                table.insert("dependencies", Item::Value(Value::Array(array)));
            }

            packages.push(table);

            bubble_sort(&mut packages.iter_mut().collect::<Vec<_>>());
        }
    }
}

impl PackagesMut<'_> {
    fn init_array(&mut self, overwrite: bool) -> Option<()> {
        if let Either::Right(option) = &mut self.array {
            match option.take().expect("some") {
                Entry::Occupied(entry) if entry.get().as_array_of_tables().is_some() => {
                    self.array =
                        Either::Left(entry.into_mut().as_array_of_tables_mut().expect("array"));
                }
                Entry::Occupied(mut entry) if overwrite => {
                    *entry.get_mut() = Item::Table(Table::new());

                    self.array =
                        Either::Left(entry.into_mut().as_array_of_tables_mut().expect("array"));
                }
                Entry::Vacant(entry) if overwrite => {
                    self.array = Either::Left(
                        entry
                            .insert(Item::ArrayOfTables(ArrayOfTables::new()))
                            .as_array_of_tables_mut()
                            .expect("array"),
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

    fn array(&mut self, overwrite: bool) -> Option<&mut ArrayOfTables> {
        self.init_array(overwrite)?;

        match self.array.as_mut().left() {
            Some(array) => Some(*array),
            None => None,
        }
    }
}

fn bubble_sort(arr: &mut [&mut Table]) {
    let mut n = arr.len();

    while n > 0 {
        let mut t = 0;

        for i in 1..n {
            let a = arr[i - 1].get("name").expect("name").as_str().expect("str");
            let b = arr[i].get("name").expect("name").as_str().expect("str");

            if a > b {
                let (lhs, rhs) = arr.split_at_mut(i);

                std::mem::swap(lhs[i - 1], rhs[0]);

                t = i;
            }
        }

        n = t;
    }
}
