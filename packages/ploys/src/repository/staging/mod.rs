use std::collections::BTreeMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use bytes::Bytes;

use super::{Repository, Stage};

/// A staging repository.
#[derive(Clone)]
pub struct Staging {
    files: BTreeMap<PathBuf, Bytes>,
}

impl Staging {
    /// Creates a new staging repository.
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    /// Adds the given file to the index.
    pub fn add_file(&mut self, path: impl Into<PathBuf>, file: impl Into<Bytes>) -> &mut Self {
        self.files.insert(path.into(), file.into());
        self
    }

    /// Builds the repository with the given file in the index.
    pub fn with_file(mut self, path: impl Into<PathBuf>, file: impl Into<Bytes>) -> Self {
        self.add_file(path, file);
        self
    }
}

impl Repository for Staging {
    type Error = Infallible;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.files.get(path.as_ref()).cloned())
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        Ok(self.files.keys().cloned())
    }
}

impl Stage for Staging {
    fn add_file(
        &mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        self.add_file(path, file);

        Ok(self)
    }

    fn remove_file(&mut self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.files.remove(path.as_ref()))
    }
}

impl Default for Staging {
    fn default() -> Self {
        Self::new()
    }
}
