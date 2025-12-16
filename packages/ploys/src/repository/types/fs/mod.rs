mod error;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use walkdir::WalkDir;

pub use self::error::Error;

use crate::repository::adapters::staged::Staged;
use crate::repository::{Commit, Open, Repository, Stage};

/// A file system repository.
#[derive(Clone)]
pub struct FileSystem {
    inner: Staged<Inner>,
}

impl FileSystem {
    /// Opens a file system repository in the current directory.
    pub fn current_dir() -> Result<Self, Error> {
        Self::open(std::env::current_dir()?)
    }

    /// Gets the file system path.
    pub fn path(&self) -> &Path {
        &self.inner.inner.path
    }
}

impl Repository for FileSystem {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        self.inner.get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        self.inner.get_index()
    }
}

impl Stage for FileSystem {
    fn add_file(
        &mut self,
        path: impl Into<RelativePathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        self.inner.add_file(path, file)?;

        Ok(self)
    }

    fn remove_file(
        &mut self,
        path: impl AsRef<RelativePath>,
    ) -> Result<Option<Bytes>, Self::Error> {
        self.inner.remove_file(path)
    }
}

impl Commit for FileSystem {
    type Params = ();

    fn commit(&mut self, _: impl Into<Self::Params>) -> Result<(), Self::Error> {
        let base_path = self.path().to_owned();

        for (path, file) in self.inner.drain() {
            let path = path.to_logical_path(&base_path);

            match file {
                Some(file) => {
                    if let Some(parent) = path.parent()
                        && parent != base_path
                    {
                        std::fs::create_dir_all(parent)?;
                    }

                    std::fs::write(path, file)?;
                }
                None => {
                    if let Err(err) = std::fs::remove_file(&path)
                        && err.kind() != ErrorKind::NotFound
                    {
                        return Err(err.into());
                    }

                    let mut parent = path.parent();

                    while let Some(path) = parent
                        && path != base_path
                        && path.read_dir()?.next().is_none()
                    {
                        std::fs::remove_dir(path)?;

                        parent = path.parent();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Open for FileSystem {
    type Context = PathBuf;

    fn open<T, E>(ctx: T) -> Result<Self, Self::Error>
    where
        T: TryInto<Self::Context, Error = E>,
        E: Into<Self::Error>,
    {
        let path = ctx.try_into().map_err(Into::into)?;

        let Ok(meta) = path.metadata() else {
            return Err(Error::Directory(path));
        };

        if !meta.is_dir() {
            return Err(Error::Directory(path));
        }

        Ok(Self {
            inner: Staged::new(Inner {
                path: path.canonicalize()?,
            }),
        })
    }
}

#[derive(Clone)]
struct Inner {
    path: PathBuf,
}

impl Repository for Inner {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        match std::fs::read(path.as_ref().to_logical_path(&self.path)) {
            Ok(bytes) => Ok(Some(bytes.into())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        let index = WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_name() != ".git"
                    && entry.path().strip_prefix(&self.path).ok() != Some(Path::new("target"))
            })
            .flat_map(|res| match res {
                Ok(entry) => match entry.file_type().is_dir() {
                    true => None,
                    false => Some(Ok(entry.path().relative_to(&self.path).expect("prefixed"))),
                },
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(index.into_iter())
    }
}
