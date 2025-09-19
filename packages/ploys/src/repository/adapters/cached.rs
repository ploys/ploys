use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;
use once_cell::sync::OnceCell;
use once_map::OnceMap;
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::Repository;

/// A repository adapter for caching files.
#[derive(Clone)]
pub struct Cached<T> {
    inner: T,
    index: Arc<OnceCell<BTreeSet<RelativePathBuf>>>,
    cache: Arc<OnceMap<RelativePathBuf, Box<OnceCell<Option<Bytes>>>>>,
}

impl<T> Cached<T> {
    /// Constructs a new cached repository adapter.
    pub fn new(repo: T) -> Self {
        Self {
            inner: repo,
            index: Arc::new(OnceCell::new()),
            cache: Arc::new(OnceMap::new()),
        }
    }

    /// Gets the inner repository.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Gets the inner repository as mutable.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.index = Arc::new(OnceCell::new());
        self.cache = Arc::new(OnceMap::new());
    }
}

impl<T> Repository for Cached<T>
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        self.cache.map_try_insert(
            path.as_ref().to_owned(),
            |path| {
                self.inner
                    .get_file(path)
                    .map(OnceCell::with_value)
                    .map(Box::new)
            },
            |path, item| item.get_or_try_init(|| self.inner.get_file(path)).cloned(),
        )?
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        Ok(self
            .index
            .get_or_try_init(|| self.inner.get_index().map(Iterator::collect))?
            .iter()
            .map(Clone::clone))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::convert::Infallible;

    use bytes::Bytes;
    use relative_path::{RelativePath, RelativePathBuf};

    use crate::repository::Repository;

    use super::Cached;

    #[derive(Clone)]
    struct Inner {
        a: RefCell<Option<Bytes>>,
        b: RefCell<Option<Bytes>>,
        c: RefCell<Option<Bytes>>,
    }

    impl Repository for Inner {
        type Error = Infallible;

        fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
            match path.as_ref().as_str() {
                "a" => Ok(self.a.take()),
                "b" => Ok(self.b.take()),
                "c" => Ok(self.c.take()),
                _ => Ok(None),
            }
        }

        fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
            Ok(vec![
                self.a.borrow().is_some().then_some("a"),
                self.b.borrow().is_some().then_some("b"),
                self.c.borrow().is_some().then_some("c"),
            ]
            .into_iter()
            .flatten()
            .map(RelativePathBuf::from))
        }
    }

    impl Default for Inner {
        fn default() -> Self {
            Self {
                a: RefCell::new(Some(Bytes::from("A!"))),
                b: RefCell::new(Some(Bytes::from("B?"))),
                c: RefCell::new(Some(Bytes::from("C."))),
            }
        }
    }

    #[test]
    fn test_cached_repository() {
        let repo = Cached::new(Inner::default());

        assert_eq!(
            repo.get_index().unwrap().collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(repo.get_file("a").unwrap(), Some(Bytes::from("A!")));
        assert_eq!(repo.get_file("b").unwrap(), Some(Bytes::from("B?")));
        assert_eq!(repo.get_file("c").unwrap(), Some(Bytes::from("C.")));

        assert_eq!(repo.inner().get_index().unwrap().count(), 0);
        assert_eq!(repo.inner().get_file("a").unwrap(), None);
        assert_eq!(repo.inner().get_file("b").unwrap(), None);
        assert_eq!(repo.inner().get_file("c").unwrap(), None);

        assert_eq!(
            repo.get_index().unwrap().collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(repo.get_file("a").unwrap(), Some(Bytes::from("A!")));
        assert_eq!(repo.get_file("b").unwrap(), Some(Bytes::from("B?")));
        assert_eq!(repo.get_file("c").unwrap(), Some(Bytes::from("C.")));
    }
}
