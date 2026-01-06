use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;
use either::Either;
use once_cell::sync::OnceCell;
use once_map::OnceMap;
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::Repository;

/// A repository adapter for caching files.
#[derive(Clone)]
pub struct Cached<T> {
    inner: T,
    index: Arc<OnceCell<BTreeSet<RelativePathBuf>>>,
    files: Arc<OnceMap<RelativePathBuf, Box<Option<Bytes>>>>,
    enabled: bool,
}

impl<T> Cached<T> {
    /// Constructs a new cached repository adapter.
    pub fn new(repo: T) -> Self {
        Self {
            inner: repo,
            index: Arc::new(OnceCell::new()),
            files: Arc::new(OnceMap::new()),
            enabled: true,
        }
    }

    /// Enables or disables the cache.
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Builds the cached repository adapter as enabled or disabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enable(enabled);
        self
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
        self.files = Arc::new(OnceMap::new());
    }
}

impl<T> Repository for Cached<T>
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        if !self.enabled {
            return self.inner.get_file(path);
        }

        self.files
            .try_insert(path.as_ref().to_owned(), |path| {
                self.inner.get_file(path).map(Box::new)
            })
            .cloned()
    }

    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, RelativePath>>, Self::Error> {
        if !self.enabled {
            return self.inner.get_index().map(Either::Left);
        }

        Ok(Either::Right(
            self.index
                .get_or_try_init(|| {
                    self.inner
                        .get_index()
                        .map(|iter| iter.map(Cow::into_owned))
                        .map(Iterator::collect)
                })?
                .iter()
                .map(RelativePathBuf::as_relative_path)
                .map(Cow::Borrowed),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::convert::Infallible;

    use bytes::Bytes;
    use relative_path::RelativePath;

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

        fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, RelativePath>>, Self::Error> {
            Ok(vec![
                self.a.borrow().is_some().then_some("a"),
                self.b.borrow().is_some().then_some("b"),
                self.c.borrow().is_some().then_some("c"),
            ]
            .into_iter()
            .flatten()
            .map(RelativePath::new)
            .map(Cow::Borrowed))
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
    fn test_cached_repository_enabled() {
        let repo = Cached::new(Inner::default());

        assert_eq!(
            repo.get_index().unwrap().collect::<Vec<_>>(),
            vec![
                Cow::Borrowed(RelativePath::new("a")),
                Cow::Borrowed(RelativePath::new("b")),
                Cow::Borrowed(RelativePath::new("c"))
            ]
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
            vec![
                Cow::Borrowed(RelativePath::new("a")),
                Cow::Borrowed(RelativePath::new("b")),
                Cow::Borrowed(RelativePath::new("c"))
            ]
        );
        assert_eq!(repo.get_file("a").unwrap(), Some(Bytes::from("A!")));
        assert_eq!(repo.get_file("b").unwrap(), Some(Bytes::from("B?")));
        assert_eq!(repo.get_file("c").unwrap(), Some(Bytes::from("C.")));
    }

    #[test]
    fn test_cached_repository_disabled() {
        let repo = Cached::new(Inner::default()).enabled(false);

        assert_eq!(
            repo.get_index().unwrap().collect::<Vec<_>>(),
            vec![
                Cow::Borrowed(RelativePath::new("a")),
                Cow::Borrowed(RelativePath::new("b")),
                Cow::Borrowed(RelativePath::new("c"))
            ]
        );
        assert_eq!(repo.get_file("a").unwrap(), Some(Bytes::from("A!")));
        assert_eq!(repo.get_file("b").unwrap(), Some(Bytes::from("B?")));
        assert_eq!(repo.get_file("c").unwrap(), Some(Bytes::from("C.")));

        assert_eq!(repo.inner().get_index().unwrap().count(), 0);
        assert_eq!(repo.inner().get_file("a").unwrap(), None);
        assert_eq!(repo.inner().get_file("b").unwrap(), None);
        assert_eq!(repo.inner().get_file("c").unwrap(), None);

        assert_eq!(repo.get_index().unwrap().count(), 0);
        assert_eq!(repo.get_file("a").unwrap(), None);
        assert_eq!(repo.get_file("b").unwrap(), None);
        assert_eq!(repo.get_file("c").unwrap(), None);
    }
}
