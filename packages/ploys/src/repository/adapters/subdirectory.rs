use std::borrow::Cow;

use bytes::Bytes;
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::path::prepare_path;
use crate::repository::{Repository, Stage};

/// A repository adapter representing a subdirectory of another repository.
#[derive(Clone)]
pub struct Subdirectory<T> {
    repo: T,
    path: RelativePathBuf,
}

impl<T> Subdirectory<T>
where
    T: Repository,
    T::Error: From<crate::repository::path::Error>,
{
    /// Constructs a new subdirectory repository adapter.
    ///
    /// Note that this constructor rejects paths that navigate up the directory
    /// hierarchy and escape the repository but does not reject empty paths.
    pub fn new(repo: T, path: impl Into<RelativePathBuf>) -> Result<Self, T::Error> {
        let path = path.into();

        Ok(Self {
            repo,
            path: match path == RelativePath::new("") {
                true => path,
                false => prepare_path(Cow::Owned(path))?.into_owned(),
            },
        })
    }
}

impl<T> Subdirectory<T>
where
    T: Repository,
{
    /// Constructs a new subdirectory repository adapter at the root.
    pub fn new_root(repo: T) -> Self {
        Self {
            repo,
            path: RelativePathBuf::new(),
        }
    }

    /// Constructs a new subdirectory repository adapter without validation.
    pub(crate) fn new_unvalidated(repo: T, path: impl Into<RelativePathBuf>) -> Self {
        Self {
            repo,
            path: path.into(),
        }
    }
}

impl<T> Subdirectory<T> {
    /// Gets the subdirectory path.
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// Gets the inner repository.
    pub fn inner(&self) -> &T {
        &self.repo
    }

    /// Gets the inner repository as mutable.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.repo
    }
}

impl<T> Repository for Subdirectory<T>
where
    T: Repository,
{
    type Error = T::Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        self.repo.get_file(self.path.join(path))
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        Ok(self.repo.get_index()?.filter_map(|path| {
            path.strip_prefix(&self.path)
                .map(RelativePath::to_relative_path_buf)
                .ok()
        }))
    }
}

impl<T> Stage for Subdirectory<T>
where
    T: Stage,
{
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        self.repo.add_file(self.path.join(path.into()), file)?;

        Ok(self)
    }

    fn add_files(
        &mut self,
        files: impl IntoIterator<Item = (RelativePathBuf, Bytes)>,
    ) -> Result<&mut Self, Self::Error> {
        self.repo.add_files(
            files
                .into_iter()
                .map(|(path, file)| (self.path.join(path), file)),
        )?;

        Ok(self)
    }

    fn with_file(
        self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            repo: self.repo.with_file(path, file)?,
            path: self.path,
        })
    }

    fn with_files(
        self,
        files: impl IntoIterator<Item = (RelativePathBuf, Bytes)>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            repo: self.repo.with_files(
                files
                    .into_iter()
                    .map(|(path, file)| (self.path.join(path), file)),
            )?,
            path: self.path,
        })
    }

    fn remove_file(
        &mut self,
        path: impl AsRef<RelativePath>,
    ) -> Result<Option<Bytes>, Self::Error> {
        self.repo.remove_file(self.path.join(path))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use relative_path::RelativePathBuf;

    use crate::repository::types::staging::Staging;
    use crate::repository::{Repository, Stage};

    use super::Subdirectory;

    #[test]
    fn test_subdirectory_valid() {
        let staging = Staging::new()
            .with_file("hello.txt", "Hello World!")
            .unwrap()
            .with_file("foo/bar.txt", "Hello Foo!")
            .unwrap();

        let root = Subdirectory::new(staging.clone(), "").unwrap();

        assert_eq!(
            root.get_file("hello.txt"),
            Ok(Some(Bytes::from("Hello World!")))
        );
        assert_eq!(
            root.get_file("foo/bar.txt"),
            Ok(Some(Bytes::from("Hello Foo!")))
        );

        let root_index = root.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(root_index.len(), 2);

        assert!(root_index.contains(&RelativePathBuf::from("hello.txt")));
        assert!(root_index.contains(&RelativePathBuf::from("foo/bar.txt")));

        let foo = Subdirectory::new(staging, "foo").unwrap();

        assert_eq!(foo.get_file("hello.txt"), Ok(None));
        assert_eq!(foo.get_file("foo/bar.txt"), Ok(None));
        assert_eq!(foo.get_file("bar.txt"), Ok(Some(Bytes::from("Hello Foo!"))));

        let foo_index = foo.get_index().unwrap().collect::<Vec<_>>();

        assert_eq!(foo_index.len(), 1);

        assert!(foo_index.contains(&RelativePathBuf::from("bar.txt")));
    }

    #[test]
    fn test_subdirectory_invalid() {
        assert!(Subdirectory::new(Staging::new(), "..").is_err());
        assert!(Subdirectory::new(Staging::new(), "../foo/bar").is_err());
    }
}
