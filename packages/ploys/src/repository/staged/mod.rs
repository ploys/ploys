mod drain;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use itertools::Itertools;

use self::drain::Drain;

use super::{Repository, Stage};

/// A repository adapter for staging changes.
#[derive(Clone)]
pub struct Staged<T> {
    pub(crate) inner: T,
    files: BTreeMap<PathBuf, Option<Bytes>>,
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
    pub(crate) fn drain(&mut self) -> impl Iterator<Item = (PathBuf, Option<Bytes>)> {
        Drain(&mut self.files).fuse()
    }
}

impl<T> Repository for Staged<T>
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        match self.files.get(path.as_ref()) {
            Some(Some(file)) => Ok(Some(file.clone())),
            Some(None) => Ok(None),
            None => self.inner.get_file(path),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        Ok(self
            .files
            .keys()
            .cloned()
            .merge(self.inner.get_index()?)
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
        path: impl Into<PathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        self.files.insert(path.into(), Some(file.into()));

        Ok(self)
    }

    fn add_files(
        &mut self,
        files: impl IntoIterator<Item = (PathBuf, Bytes)>,
    ) -> Result<&mut Self, Self::Error> {
        self.files
            .extend(files.into_iter().map(|(path, file)| (path, Some(file))));

        Ok(self)
    }

    fn remove_file(&mut self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.files.insert(path.as_ref().to_owned(), None).flatten())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::repository::staging::Staging;
    use crate::repository::{Repository, Stage};

    use super::Staged;

    #[test]
    fn test_staged_repository() {
        let inner = Staging::new().with_file("a", "A").with_file("b", "B");
        let mut outer = Staged::new(inner);

        assert_eq!(outer.get_file("a"), Ok(Some("A".into())));
        assert_eq!(outer.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.get_file("c"), Ok(None));

        let index = outer.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(index.len(), 2);
        assert!(index.contains(&PathBuf::from("a")));
        assert!(index.contains(&PathBuf::from("b")));

        outer.add_file("c", "C").unwrap();
        outer.remove_file("a").unwrap();

        assert_eq!(outer.get_file("a"), Ok(None));
        assert_eq!(outer.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.get_file("c"), Ok(Some("C".into())));

        let index = outer.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(index.len(), 2);
        assert!(index.contains(&PathBuf::from("b")));
        assert!(index.contains(&PathBuf::from("c")));

        assert_eq!(outer.inner.get_file("a"), Ok(Some("A".into())));
        assert_eq!(outer.inner.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.inner.get_file("c"), Ok(None));
    }
}
