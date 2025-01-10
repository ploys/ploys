//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gix::traverse::tree::Recorder;
use gix::ThreadSafeRepository;

use crate::file::{File, FileCache};

pub use self::error::Error;

use super::revision::Revision;

/// The local Git repository.
#[derive(Clone)]
pub struct Git {
    repository: ThreadSafeRepository,
    revision: Revision,
    file_cache: Arc<FileCache>,
}

impl Git {
    /// Opens a Git repository.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Self {
            repository: ThreadSafeRepository::open(path)?,
            revision: Revision::Head,
            file_cache: Arc::new(FileCache::new()),
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
    pub fn get_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<Cow<'_, File>>, crate::project::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            let bytes = self.get_file_contents(path.as_ref())?;

            return Ok(Some(Cow::Owned(File::from_bytes(bytes, path.as_ref())?)));
        }

        self.file_cache
            .get_or_try_insert_with(path.as_ref(), |path| match self.get_file_contents(path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(err) => Err(err),
            })
            .map(|file| file.map(Cow::Borrowed))
    }

    /// Gets the file index.
    pub fn get_file_index(&self) -> impl Iterator<Item = Cow<'_, Path>> {
        self.file_cache
            .get_or_try_index_with(|| self.get_files())
            .iter()
            .map(|path| Cow::Borrowed(path.as_path()))
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
