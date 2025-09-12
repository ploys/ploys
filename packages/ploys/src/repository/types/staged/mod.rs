mod drain;
mod error;

use std::borrow::Cow;
use std::collections::BTreeMap;

use bytes::Bytes;
use itertools::Itertools;
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::path::prepare_path;
use crate::repository::{Repository, Stage};

use self::drain::Drain;

pub use self::error::Error;

/// A repository adapter for staging changes.
#[derive(Clone)]
pub struct Staged<T> {
    pub(crate) inner: T,
    files: BTreeMap<RelativePathBuf, Option<Bytes>>,
}

impl<T> Staged<T> {
    /// Constructs a new staged repository adapter.
    pub fn new(repo: T) -> Self {
        Self {
            inner: repo,
            files: BTreeMap::new(),
        }
    }

    /// Builds the adapter with the given repository.
    pub fn with_repository<U>(self, repo: U) -> Staged<U> {
        Staged {
            inner: repo,
            files: self.files,
        }
    }

    /// Drains the staged files.
    pub(crate) fn drain(&mut self) -> impl Iterator<Item = (RelativePathBuf, Option<Bytes>)> {
        Drain(&mut self.files).fuse()
    }
}

impl<T> Repository for Staged<T>
where
    T: Repository,
{
    type Error = Error<T::Error>;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        let path = prepare_path(Cow::Borrowed(path.as_ref()))?;

        match self.files.get(&*path) {
            Some(Some(file)) => Ok(Some(file.clone())),
            Some(None) => Ok(None),
            None => self.inner.get_file(path).map_err(Error::Repo),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        Ok(self
            .files
            .keys()
            .cloned()
            .merge(self.inner.get_index().map_err(Error::Repo)?)
            .unique()
            .filter(|path| self.files.get(path).is_none_or(Option::is_some)))
    }
}

impl<T> Stage for Staged<T>
where
    T: Repository,
{
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        let path = prepare_path(Cow::Owned(path.into()))?;

        self.files.insert(path.into_owned(), Some(file.into()));

        Ok(self)
    }

    fn remove_file(
        &mut self,
        path: impl AsRef<RelativePath>,
    ) -> Result<Option<Bytes>, Self::Error> {
        let path = prepare_path(Cow::Borrowed(path.as_ref()))?;

        Ok(self.files.insert(path.into_owned(), None).flatten())
    }
}

#[cfg(test)]
mod tests {
    use relative_path::RelativePathBuf;

    use crate::repository::types::staging::Staging;
    use crate::repository::{Repository, Stage};

    use super::Staged;

    #[test]
    fn test_staged_repository() {
        let inner = Staging::new()
            .with_file("a", "A")
            .unwrap()
            .with_file("b", "B")
            .unwrap();
        let mut outer = Staged::new(inner);

        assert_eq!(outer.get_file("a"), Ok(Some("A".into())));
        assert_eq!(outer.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.get_file("c"), Ok(None));

        let index = outer.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(index.len(), 2);
        assert!(index.contains(&RelativePathBuf::from("a")));
        assert!(index.contains(&RelativePathBuf::from("b")));

        outer.add_file("c", "C").unwrap();
        outer.remove_file("a").unwrap();

        assert_eq!(outer.get_file("a"), Ok(None));
        assert_eq!(outer.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.get_file("c"), Ok(Some("C".into())));

        let index = outer.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(index.len(), 2);
        assert!(index.contains(&RelativePathBuf::from("b")));
        assert!(index.contains(&RelativePathBuf::from("c")));

        assert_eq!(outer.inner.get_file("a"), Ok(Some("A".into())));
        assert_eq!(outer.inner.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.inner.get_file("c"), Ok(None));
    }
}
