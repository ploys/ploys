use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// An in-memory repository.
#[derive(Clone)]
pub struct Memory {
    files: BTreeMap<PathBuf, Box<[u8]>>,
}

impl Memory {
    /// Creates a new in-memory repository.
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    /// Inserts a file into the repository.
    pub fn insert_file(
        &mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Cow<'static, [u8]>>,
    ) -> &mut Self {
        self.files.insert(path.into(), file.into().into());
        self
    }

    /// Builds the repository with the given file.
    pub fn with_file(
        mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        self.insert_file(path, file);
        self
    }
}

impl Memory {
    /// Gets the file at the given path.
    pub fn get_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<Cow<'_, [u8]>>, crate::project::Error> {
        Ok(self
            .files
            .get(path.as_ref())
            .map(AsRef::as_ref)
            .map(Cow::Borrowed))
    }

    /// Gets the file index.
    pub fn get_file_index(
        &self,
    ) -> Result<impl Iterator<Item = Cow<'_, Path>>, crate::project::Error> {
        Ok(self.files.keys().map(|path| Cow::Borrowed(path.as_path())))
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
