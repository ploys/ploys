use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::OnceCell;

/// The repository cache.
///
/// This stores a map of file paths to optional file contents and is intended to
/// be used for repositories that are locked to a specific commit. This means
/// that the index and contents will not change.
///
/// The implementation is not particularly efficient as not every file will need
/// to be loaded but this design greatly simplifies the handling of files. It
/// avoids the need to use `stable_deref_trait` and append-only data structures
/// at the cost of extra memory per file path. It also requires that the index
/// be loaded before any files can be loaded.
///
/// This uses the `once_cell` crate as the `OnceLock::get_or_try_init` method in
/// the standard library is currently [experimental][109737]. The design should
/// allow for the `OnceCell` type in the `async-lock` crate to be substituted
/// for the one from `once_cell` using `get_or_try_init_blocking` and enable the
/// ability to load files asynchronously using `get_or_try_init`.
///
/// [109737]: https://github.com/rust-lang/rust/issues/109737
#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub struct Cache {
    cache: Arc<OnceCell<BTreeMap<PathBuf, OnceCell<Box<[u8]>>>>>,
}

impl Cache {
    /// Constructs a new repository cache.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(OnceCell::new()),
        }
    }

    /// Gets or inserts the file with the given path.
    pub fn get_or_try_insert<E, F>(
        &self,
        path: impl AsRef<Path>,
        with: F,
    ) -> Result<Option<&[u8]>, E>
    where
        F: FnOnce(&Path) -> Result<Vec<u8>, E>,
    {
        match self.cache.get() {
            Some(cache) => match cache.get(path.as_ref()) {
                Some(cell) => cell
                    .get_or_try_init(|| with(path.as_ref()).map(Into::into))
                    .map(AsRef::as_ref)
                    .map(Some),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// Gets or inserts the index.
    pub fn get_or_try_index<E, F>(&self, with: F) -> Result<impl Iterator<Item = &Path>, E>
    where
        F: FnOnce() -> Result<BTreeSet<PathBuf>, E>,
    {
        self.cache
            .get_or_try_init(|| {
                with().map(|index| {
                    index
                        .into_iter()
                        .map(|index| (index, OnceCell::new()))
                        .collect()
                })
            })
            .map(|index| index.keys().map(PathBuf::as_path))
    }

    /// Gets or inserts the index and file with the given path.
    pub fn get_or_try_init<E, F, G>(
        &self,
        path: impl AsRef<Path>,
        insert: F,
        index: G,
    ) -> Result<Option<&[u8]>, E>
    where
        F: FnOnce(&Path) -> Result<Vec<u8>, E>,
        G: FnOnce() -> Result<BTreeSet<PathBuf>, E>,
    {
        let _ = self.get_or_try_index(index)?;

        self.get_or_try_insert(path, insert)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
