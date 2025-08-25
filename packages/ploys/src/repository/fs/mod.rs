mod error;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use walkdir::WalkDir;

pub use self::error::Error;

use super::{Repository, Stage, Staged};

/// A file system repository.
#[derive(Clone)]
pub struct FileSystem {
    inner: Staged<Inner>,
}

impl FileSystem {
    /// Opens a file system repository.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self {
            inner: Staged::new(Inner { path: path.into() }),
        }
    }

    /// Opens a file system repository in the current directory.
    pub fn current_dir() -> Result<Self, Error> {
        Ok(Self::open(std::env::current_dir()?))
    }

    /// Gets the file system path.
    pub fn path(&self) -> &Path {
        &self.inner.inner.path
    }
}

impl Repository for FileSystem {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        self.inner.get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        self.inner.get_index()
    }
}

impl Stage for FileSystem {
    fn add_file(
        &mut self,
        path: impl Into<PathBuf>,
        file: impl Into<Bytes>,
    ) -> Result<&mut Self, Self::Error> {
        self.inner.add_file(path, file)?;

        Ok(self)
    }

    fn remove_file(&mut self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        self.inner.remove_file(path)
    }
}

#[derive(Clone)]
struct Inner {
    path: PathBuf,
}

impl Repository for Inner {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        match std::fs::read(self.path.join(path.as_ref())) {
            Ok(bytes) => Ok(Some(bytes.into())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        let index = WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_name() != ".git"
                    && entry.path().strip_prefix(&self.path).ok() != Some(Path::new("target"))
            })
            .flat_map(|res| match res {
                Ok(entry) => match entry.file_type().is_dir() {
                    true => None,
                    false => Some(Ok(entry
                        .path()
                        .strip_prefix(&self.path)
                        .expect("prefixed")
                        .to_owned())),
                },
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(index.into_iter())
    }
}
