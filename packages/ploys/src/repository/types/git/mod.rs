//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use gix::ThreadSafeRepository;
use gix::config::File;
use gix::create::{Kind, Options};
use gix::traverse::tree::Recorder;

use crate::repository::Repository;
use crate::repository::cache::Cache;
use crate::repository::revision::Revision;

pub use self::error::Error;

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

    /// Initializes a Git repository.
    pub fn init(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            repository: ThreadSafeRepository::init(path, Kind::WithWorktree, Options::default())?,
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
    /// Gets the author information.
    pub fn get_author() -> Option<String> {
        let globals = File::from_globals().ok();
        let overrides = File::from_environment_overrides().ok();

        let config = match (globals, overrides) {
            (Some(globals), Some(overrides)) => {
                let mut config = globals;
                config.append(overrides);
                config
            }
            (Some(config), None) => config,
            (None, Some(config)) => config,
            (None, None) => return None,
        };

        let name = config.string("user.name")?;
        let email = config.string("user.email")?;

        Some(format!("{name} <{email}>"))
    }
}

impl Repository for Git {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<Path>) -> Result<Option<Bytes>, Self::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Some(self.get_file_uncached(path.as_ref())?));
        }

        self.cache.get_or_try_init(
            path,
            |path| self.get_file_uncached(path),
            || self.get_index_uncached(),
        )
    }

    fn get_index(&self) -> Result<impl Iterator<Item = PathBuf>, Self::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Box::new(self.get_index_uncached()?.into_iter())
                as Box<dyn Iterator<Item = PathBuf>>);
        }

        Ok(Box::new(
            self.cache.get_or_try_index(|| self.get_index_uncached())?,
        ))
    }
}

impl Git {
    fn get_index_uncached(&self) -> Result<BTreeSet<PathBuf>, Error> {
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

    fn get_file_uncached(&self, path: impl AsRef<Path>) -> Result<Bytes, Error> {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let mut tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

        let entry = tree
            .peel_to_entry_by_path(path)?
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

        if entry.mode().is_blob() {
            Ok(entry.object()?.detached().data.into())
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
    }
}
