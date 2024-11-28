//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;

use std::io;
use std::path::{Path, PathBuf};

use gix::remote::Direction;
use gix::traverse::tree::Recorder;
use gix::ThreadSafeRepository;
use url::Url;

pub use self::error::Error;

use super::revision::Revision;

/// The local Git repository.
pub struct Git {
    repository: ThreadSafeRepository,
    revision: Revision,
}

impl Git {
    /// Creates a Git repository.
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            repository: ThreadSafeRepository::open(path.as_ref())?,
            revision: Revision::Head,
        })
    }
}

impl Git {
    /// Gets the revision.
    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    /// Sets the revision.
    pub fn set_revision(&mut self, revision: impl Into<Revision>) {
        self.revision = revision.into();
    }

    /// Builds the repository with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.set_revision(revision);
        self
    }
}

impl Git {
    pub fn get_name(&self) -> Result<String, Error> {
        let path = self.repository.path().join("..").canonicalize()?;

        if let Some(file_stem) = path.file_stem() {
            return Ok(file_stem.to_string_lossy().to_string());
        }

        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "Invalid directory",
        )))
    }

    pub fn get_url(&self) -> Result<Url, Error> {
        let repo = self.repository.to_thread_local();

        match repo.find_default_remote(Direction::Push).transpose()? {
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

    pub fn get_files(&self) -> Result<Vec<PathBuf>, Error> {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

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

    pub fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: AsRef<Path>,
    {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let mut tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

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
