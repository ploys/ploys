use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use itertools::Itertools;

use super::{Repository, Stage};

/// A repository adapter for staging changes.
#[derive(Clone)]
pub struct Staged<T> {
    inner: T,
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
            .iter()
            .filter_map(|(key, val)| val.as_ref().map(|_| key.clone()))
            .merge(self.inner.get_index()?)
            .unique())
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

    fn remove_file(&mut self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.files.insert(path.as_ref().to_owned(), None).flatten())
    }
}

#[cfg(test)]
mod tests {
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

        outer.add_file("c", "C").unwrap();
        outer.remove_file("a").unwrap();

        assert_eq!(outer.get_file("a"), Ok(None));
        assert_eq!(outer.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.get_file("c"), Ok(Some("C".into())));

        assert_eq!(outer.inner.get_file("a"), Ok(Some("A".into())));
        assert_eq!(outer.inner.get_file("b"), Ok(Some("B".into())));
        assert_eq!(outer.inner.get_file("c"), Ok(None));
    }
}
