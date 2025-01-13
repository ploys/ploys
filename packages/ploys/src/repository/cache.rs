use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use once_map::OnceMap;

use super::File;

/// The repository cache.
#[derive(Clone, Debug)]
pub struct Cache {
    index: Arc<OnceLock<BTreeSet<PathBuf>>>,
    inner: Arc<OnceMap<PathBuf, Box<Option<File>>>>,
}

impl Cache {
    /// Constructs a new repository cache.
    pub fn new() -> Self {
        Self {
            index: Arc::new(OnceLock::new()),
            inner: Arc::new(OnceMap::new()),
        }
    }

    /// Gets or inserts the file with the given path.
    pub fn get_or_try_insert_with<E, F>(
        &self,
        path: impl AsRef<Path>,
        with: F,
    ) -> Result<Option<&File>, crate::project::Error>
    where
        F: FnOnce(&Path) -> Result<Option<Vec<u8>>, E>,
        E: Into<crate::project::Error>,
    {
        self.inner
            .try_insert(path.as_ref().to_owned(), |path| {
                match with(path).map_err(Into::into)? {
                    Some(bytes) => Ok(Box::new(Some(File::from_bytes(bytes, path)?))),
                    None => Ok(Box::new(None)),
                }
            })
            .map(Option::as_ref)
    }

    /// Gets or inserts the index.
    pub fn get_or_try_index_with<E, F>(&self, with: F) -> &BTreeSet<PathBuf>
    where
        F: FnOnce() -> Result<BTreeSet<PathBuf>, E>,
    {
        self.index.get_or_init(|| with().unwrap_or_default())
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
