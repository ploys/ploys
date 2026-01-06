mod error;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::btree_map::IntoIter;

use bytes::Bytes;
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::path::prepare_path;
use crate::repository::{Repository, Stage};

pub use self::error::Error;

/// A staging repository.
#[derive(Clone)]
pub struct Staging {
    files: BTreeMap<RelativePathBuf, Bytes>,
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
    type Error = Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        let path = prepare_path(Cow::Borrowed(path.as_ref()))?;

        Ok(self.files.get(&*path).cloned())
    }

    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, RelativePath>>, Self::Error> {
        Ok(self
            .files
            .keys()
            .map(RelativePathBuf::as_ref)
            .map(Cow::Borrowed))
    }
}

impl Stage for Staging {
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        let path = prepare_path(Cow::Owned(path.into()))?;

        self.files.insert(path.into_owned(), file.into());

        Ok(self)
    }

    fn remove_file(
        &mut self,
        path: impl AsRef<RelativePath>,
    ) -> Result<Option<Bytes>, Self::Error> {
        let path = prepare_path(Cow::Borrowed(path.as_ref()))?;

        Ok(self.files.remove(&*path))
    }
}

impl Default for Staging {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Staging {
    type Item = (RelativePathBuf, Bytes);
    type IntoIter = IntoIter<RelativePathBuf, Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}
