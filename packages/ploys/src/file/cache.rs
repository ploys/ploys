use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use once_map::OnceMap;

use super::File;

/// The file cache.
#[derive(Debug)]
pub struct FileCache {
    index: OnceLock<BTreeSet<PathBuf>>,
    inner: OnceMap<PathBuf, Box<Option<File>>>,
}

impl FileCache {
    /// Constructs a new file cache.
    pub fn new() -> Self {
        Self {
            index: OnceLock::new(),
            inner: OnceMap::new(),
        }
    }

    /// Gets or inserts the file with the given path.
    pub fn get_or_try_insert_with<E, F>(
        &self,
        path: impl AsRef<Path>,
        with: F,
    ) -> Result<Option<&File>, E>
    where
        F: FnOnce(&Path) -> Result<Option<Vec<u8>>, E>,
    {
        self.inner
            .try_insert(path.as_ref().to_owned(), |path| {
                with(path).map(|bytes| match bytes {
                    Some(bytes) => Box::new(File::from_bytes(bytes, path)),
                    None => Box::new(None),
                })
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

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FileCache {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            inner: self
                .inner
                .read_only_view()
                .iter()
                .map(|(key, val)| (key.clone(), val.clone()))
                .collect(),
        }
    }
}
