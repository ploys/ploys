use std::path::Path;

use either::Either;
use globset::Glob;
use toml_edit::{Array, Entry, Item, Table, TableLike, Value};

/// The workspace table.
pub struct Workspace<'a>(pub(super) &'a dyn TableLike);

impl<'a> Workspace<'a> {
    /// Gets the workspace members.
    pub fn members(&self) -> WorkspaceMembers<'a> {
        match self.0.get("members") {
            Some(item) => WorkspaceMembers(item.as_array()),
            None => WorkspaceMembers(None),
        }
    }

    /// Gets the workspace excludes.
    pub fn exclude(&self) -> WorkspaceExclude<'a> {
        match self.0.get("exclude") {
            Some(item) => WorkspaceExclude(item.as_array()),
            None => WorkspaceExclude(None),
        }
    }
}

/// The mutable workspace table.
pub struct WorkspaceMut<'a> {
    table: Either<&'a mut dyn TableLike, Option<Entry<'a>>>,
}

impl<'a> WorkspaceMut<'a> {
    pub(super) fn new(entry: Entry<'a>) -> Self {
        Self {
            table: Either::Right(Some(entry)),
        }
    }
}

impl WorkspaceMut<'_> {
    /// Adds a member to the workspace.
    pub fn add_member(&mut self, path: impl AsRef<Path>) {
        let table = self.table(true).expect("table");
        let members = table
            .entry("members")
            .or_insert_with(|| Item::Value(Value::Array(Array::new())));

        if let Some(members) = members.as_array_mut() {
            for member in members.iter() {
                if let Some(member) = member.as_str() {
                    if path.as_ref() == Path::new(member) {
                        return;
                    }

                    if let Ok(glob) = Glob::new(member.trim_start_matches("./")) {
                        if glob.compile_matcher().is_match(path.as_ref()) {
                            return;
                        }
                    }
                }
            }

            members.push(path.as_ref().to_string_lossy().to_string());
        }
    }
}

impl WorkspaceMut<'_> {
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
}

/// The workspace members array.
pub struct WorkspaceMembers<'a>(Option<&'a Array>);

impl<'a> IntoIterator for WorkspaceMembers<'a> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(arr) => Box::new(arr.into_iter().flat_map(Value::as_str)),
            None => Box::new(std::iter::empty()),
        }
    }
}

/// The workspace excludes array.
pub struct WorkspaceExclude<'a>(Option<&'a Array>);

impl<'a> IntoIterator for WorkspaceExclude<'a> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(arr) => Box::new(arr.into_iter().flat_map(Value::as_str)),
            None => Box::new(std::iter::empty()),
        }
    }
}
