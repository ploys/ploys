use toml_edit::{Array, TableLike, Value};

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
