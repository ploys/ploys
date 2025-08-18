use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use bytes::Bytes;

use super::Repository;

/// An in-memory repository.
#[derive(Clone)]
pub struct Memory {
    files: BTreeMap<PathBuf, Bytes>,
}

impl Memory {
    /// Creates a new in-memory repository.
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    /// Inserts a file into the repository.
    pub fn insert_file(&mut self, path: impl Into<PathBuf>, file: impl Into<Bytes>) -> &mut Self {
        self.files.insert(path.into(), file.into());
        self
    }

    /// Builds the repository with the given file.
    pub fn with_file(mut self, path: impl Into<PathBuf>, file: impl Into<Bytes>) -> Self {
        self.insert_file(path, file);
        self
    }
}

impl Repository for Memory {
    type Error = Infallible;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.files.get(path.as_ref()).cloned())
    }

    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, Path>>, Self::Error> {
        Ok(self.files.keys().map(|path| Cow::Borrowed(path.as_path())))
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
