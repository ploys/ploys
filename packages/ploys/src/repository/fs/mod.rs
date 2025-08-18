use std::borrow::Cow;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use bytes::Bytes;
use walkdir::WalkDir;

use super::Repository;

/// A file system repository.
#[derive(Clone)]
pub struct FileSystem {
    path: PathBuf,
}

impl FileSystem {
    /// Opens a file system repository.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Opens a file system repository in the current directory.
    pub fn current_dir() -> Result<Self, Error> {
        Ok(Self {
            path: std::env::current_dir()?,
        })
    }

    /// Gets the file system path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Repository for FileSystem {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        match std::fs::read(self.path.join(path.as_ref())) {
            Ok(bytes) => Ok(Some(bytes.into())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = Cow<'_, Path>>, Self::Error> {
        let index = WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_name() != ".git"
                    && entry.path().strip_prefix(&self.path).ok() != Some(Path::new("target"))
            })
            .flat_map(|res| match res {
                Ok(entry) => match entry.file_type().is_dir() {
                    true => None,
                    false => Some(Ok(Cow::Owned(
                        entry
                            .path()
                            .strip_prefix(&self.path)
                            .expect("prefixed")
                            .to_owned(),
                    ))),
                },
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(index.into_iter())
    }
}
