//! Local project inspection and management
//!
//! This module contains the utilities related to local project management. The
//! [`Local`] type must be constructed via [`super::Project`].

mod error;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gix::remote::Direction;
use gix::traverse::tree::Recorder;
use gix::Repository;
use url::Url;

pub use self::error::{Error, GitError};

/// A project on the local file system.
#[derive(Clone, Debug)]
pub struct Local {
    repository: Repository,
}

impl Local {
    /// Creates a local project.
    pub(super) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            repository: gix::open(path.as_ref())?,
        })
    }
}

impl Local {
    /// Queries the project name.
    pub fn get_name(&self) -> Result<String, Error> {
        let path = self.repository.path().join("..").canonicalize()?;

        if let Ok(readme) = fs::read_to_string(path.join("README.md")) {
            if let Some(title) = readme.lines().find(|line| line.starts_with("# ")) {
                return Ok(title[2..].to_string());
            }
        }

        if let Some(file_stem) = path.file_stem() {
            return Ok(file_stem.to_string_lossy().to_string());
        }

        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "Invalid directory",
        )))
    }

    /// Queries the project URL.
    pub fn get_url(&self) -> Result<Url, Error> {
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

    /// Queries the project files.
    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
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

    /// Queries the contents of a project file.
    ///
    /// This method obtains the file contents from the local repository instead
    /// of the local file system.
    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
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
