use std::hash::Hash;

use once_map::{Equivalent, OnceMap};

/// A key-value cache supporting writes via shared references.
pub struct Cache<K, V> {
    inner: OnceMap<K, Box<Option<V>>>,
}

impl<K, V> Cache<K, V> {
    /// Constructs a new cache.
    pub fn new() -> Self {
        Self {
            inner: OnceMap::new(),
        }
    }
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    /// Gets the value for the given key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Eq + Equivalent<K> + ?Sized,
    {
        self.inner.get(key)?.as_ref()
    }

    /// Gets the mutable value for the given key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Eq + Equivalent<K> + ?Sized,
    {
        self.inner
            .iter_mut()
            .find(|(k, _)| key.equivalent(*k))
            .map(|(_, val)| (**val).as_mut())?
    }

    /// Inserts the given key-value pair.
    pub fn insert(&mut self, key: impl Into<K>, val: impl Into<V>) {
        let key = key.into();

        self.inner.remove(&key);
        self.inner.insert(key, |_| Box::new(Some(val.into())));
    }
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for Cache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .read_only_view()
                .iter()
                .map(|(key, val)| (key.clone(), val.clone()))
                .collect(),
        }
    }
}

impl<K, V, K2, V2> FromIterator<(K2, V2)> for Cache<K, V>
where
    K: Eq + Hash,
    K2: Into<K>,
    V2: Into<V>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K2, V2)>,
    {
        Self {
            inner: iter
                .into_iter()
                .map(|(key, val)| (key.into(), Box::new(Some(val.into()))))
                .collect(),
        }
    }
}
