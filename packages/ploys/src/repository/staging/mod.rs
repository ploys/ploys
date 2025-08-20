use std::collections::BTreeMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bytes::Bytes;
use parking_lot::Mutex;

use super::{Repository, Stage};

/// A staging repository.
#[derive(Clone)]
pub struct Staging {
    files: Arc<Mutex<BTreeMap<PathBuf, Bytes>>>,
}

impl Staging {
    /// Creates a new staging repository.
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Adds the given file to the index.
    pub fn add_file(&mut self, path: impl Into<PathBuf>, file: impl Into<Bytes>) -> &mut Self {
        self.files.lock().insert(path.into(), file.into());
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
        Ok(self.files.lock().get(path.as_ref()).cloned())
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        Ok(self
            .files
            .lock()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .into_iter())
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
}

impl Default for Staging {
    fn default() -> Self {
        Self::new()
    }
}
