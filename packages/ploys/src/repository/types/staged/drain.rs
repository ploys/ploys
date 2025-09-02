use std::collections::BTreeMap;
use std::path::PathBuf;

use bytes::Bytes;

/// An iterator that drains a file map.
pub struct Drain<'a>(pub &'a mut BTreeMap<PathBuf, Option<Bytes>>);

impl<'a> Iterator for Drain<'a> {
    type Item = (PathBuf, Option<Bytes>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_first()
    }
}
