use std::collections::BTreeMap;

use bytes::Bytes;
use relative_path::RelativePathBuf;

/// An iterator that drains a file map.
pub struct Drain<'a>(pub &'a mut BTreeMap<RelativePathBuf, Option<Bytes>>);

impl<'a> Iterator for Drain<'a> {
    type Item = (RelativePathBuf, Option<Bytes>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_first()
    }
}
