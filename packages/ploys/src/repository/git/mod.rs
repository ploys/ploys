//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use gix::traverse::tree::Recorder;
use gix::ThreadSafeRepository;

pub use self::error::Error;

use super::cache::Cache;
use super::revision::Revision;

/// The local Git repository.
#[derive(Clone)]
pub struct Git {
    repository: ThreadSafeRepository,
    revision: Revision,
    cache: Cache,
}

impl Git {
    /// Opens a Git repository.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Self {
            repository: ThreadSafeRepository::open(path)?,
            revision: Revision::Head,
            cache: Cache::new(),
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
    /// Gets the file at the given path.
    pub fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Cow<'_, [u8]>>, Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Some(Cow::Owned(self.get_file_contents(path.as_ref())?)));
        }

        Ok(self
            .cache
            .get_or_try_init(
                path,
                |path| self.get_file_contents(path),
                || self.get_files(),
            )?
            .map(Cow::Borrowed))
    }

    /// Gets the index.
    pub fn get_index(&self) -> Result<Box<dyn Iterator<Item = Cow<'_, Path>> + '_>, Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Box::new(self.get_files()?.into_iter().map(Cow::Owned)));
        }

        Ok(Box::new(
            self.cache
                .get_or_try_index(|| self.get_files())?
                .map(Cow::Borrowed),
        ))
    }

    pub(crate) fn get_files(&self) -> Result<BTreeSet<PathBuf>, Error> {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

        let mut recorder = Recorder::default();

        tree.traverse().breadthfirst(&mut recorder)?;

        let entries = recorder
            .records
            .into_iter()
            .filter(|entry| entry.mode.is_blob())
            .map(|entry| PathBuf::from(entry.filepath.to_string()))
            .collect::<BTreeSet<_>>();

        Ok(entries)
    }

    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Error>
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
