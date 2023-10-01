//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;

use std::io;
use std::path::{Path, PathBuf};

use gix::remote::Direction;
use gix::traverse::tree::Recorder;
use gix::Repository;
use url::Url;

use crate::package::Package;

pub use self::error::{Error, GitError};

use super::Source;

/// The local Git repository source.
#[derive(Clone, Debug)]
pub struct Git {
    repository: Repository,
}

impl Git {
    /// Creates a Git source.
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            repository: gix::open(path.as_ref())?,
        })
    }
}

impl Source for Git {
    type Config = GitConfig;
    type Error = Error;

    fn open_with(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::new(config.path)
    }

    fn get_name(&self) -> Result<String, Self::Error> {
        let path = self.repository.path().join("..").canonicalize()?;

        if let Some(file_stem) = path.file_stem() {
            return Ok(file_stem.to_string_lossy().to_string());
        }

        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "Invalid directory",
        )))
    }

    fn get_url(&self) -> Result<Url, Self::Error> {
        match self
            .repository
            .find_default_remote(Direction::Push)
            .transpose()?
        {
            Some(remote) => match remote.url(Direction::Push) {
                Some(url) => Ok(url
                    .to_bstring()
                    .to_string()
                    .parse()
                    .expect("A repository URL should be valid")),
                None => Err(Error::remote_not_found()),
            },
            None => Err(Error::remote_not_found()),
        }
    }

    fn get_packages(&self) -> Result<Vec<Package>, Self::Error> {
        let files = self.get_files()?;

        Package::discover(&files, |path| self.get_file_contents(path))
    }

    fn get_files(&self) -> Result<Vec<PathBuf>, Self::Error> {
        let tree = self
            .repository
            .rev_parse_single("HEAD")?
            .object()?
            .peel_to_tree()?;

        let mut recorder = Recorder::default();

        tree.traverse().breadthfirst(&mut recorder)?;

        let mut entries = recorder
            .records
            .into_iter()
            .filter(|entry| entry.mode.is_blob())
            .map(|entry| PathBuf::from(entry.filepath.to_string()))
            .collect::<Vec<_>>();

        entries.sort();

        Ok(entries)
    }

    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Self::Error>
    where
        P: AsRef<Path>,
    {
        let mut tree = self
            .repository
            .rev_parse_single("HEAD")?
            .object()?
            .peel_to_tree()?;

        let entry = tree
            .peel_to_entry_by_path(path)?
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

        if entry.mode().is_blob() {
            Ok(entry.object()?.detached().data)
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
    }
}

/// The Git source configuration.
pub struct GitConfig {
    path: PathBuf,
}

impl GitConfig {
    /// Creates a new Git source configuration.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            path: path.into(),
        }
    }
}
