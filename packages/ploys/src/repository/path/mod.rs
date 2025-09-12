mod error;

use std::borrow::Cow;

use relative_path::{Component, RelativePath};

pub use self::error::Error;

/// Prepares a path for a repository.
pub(crate) fn prepare_path(
    mut path: Cow<'_, RelativePath>,
) -> Result<Cow<'_, RelativePath>, Error> {
    if !path.is_normalized() {
        path = Cow::Owned(path.normalize());
    }

    match path.components().next() {
        Some(Component::Normal(_)) => Ok(path),
        Some(Component::ParentDir) => Err(Error::Escape(path.into_owned())),
        Some(Component::CurDir) => unreachable!("path is normalized"),
        None => Err(Error::Empty),
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use relative_path::{RelativePath, RelativePathBuf};

    use super::{Error, prepare_path};

    #[test]
    fn test_prepare_valid() {
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("foo"))),
            Ok(Cow::Owned(RelativePathBuf::from("foo")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("/foo"))),
            Ok(Cow::Owned(RelativePathBuf::from("foo")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("./foo"))),
            Ok(Cow::Owned(RelativePathBuf::from("foo")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("./foo/../bar"))),
            Ok(Cow::Owned(RelativePathBuf::from("bar")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("foo/./bar"))),
            Ok(Cow::Owned(RelativePathBuf::from("foo/bar")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("foo/../bar/../baz"))),
            Ok(Cow::Owned(RelativePathBuf::from("baz")))
        );
    }

    #[test]
    fn test_prepare_invalid() {
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new(""))),
            Err(Error::Empty)
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("."))),
            Err(Error::Empty)
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("../foo"))),
            Err(Error::Escape(RelativePathBuf::from("../foo")))
        );
        assert_eq!(
            prepare_path(Cow::Borrowed(RelativePath::new("foo/../bar/../../baz"))),
            Err(Error::Escape(RelativePathBuf::from("../baz")))
        );
    }
}
