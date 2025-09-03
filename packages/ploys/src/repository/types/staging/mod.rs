use std::collections::BTreeMap;
use std::collections::btree_map::IntoIter;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use bytes::Bytes;

use crate::repository::{Repository, Stage};

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
        self.files.insert(path.into(), file.into());

        Ok(self)
    }

    fn add_files(
        &mut self,
        files: impl IntoIterator<Item = (PathBuf, Bytes)>,
    ) -> Result<&mut Self, Self::Error> {
        self.files.extend(files);

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

impl IntoIterator for Staging {
    type Item = (PathBuf, Bytes);
    type IntoIter = IntoIter<PathBuf, Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}
