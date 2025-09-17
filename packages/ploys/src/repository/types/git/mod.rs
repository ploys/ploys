//! Git project inspection and management
//!
//! This module contains the utilities related to local Git project management.

mod error;
mod params;

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use gix::config::File;
use gix::create::{Kind, Options};
use gix::traverse::tree::Recorder;
use gix::{ObjectId, ThreadSafeRepository};
use relative_path::{RelativePath, RelativePathBuf};

use crate::repository::adapters::staged::Staged;
use crate::repository::cache::Cache;
use crate::repository::revision::{Reference, Revision};
use crate::repository::{Commit, Repository, Stage};

pub use self::error::Error;
pub use self::params::CommitParams;

/// The local Git repository.
#[derive(Clone)]
pub struct Git {
    inner: Staged<Inner>,
}

impl Git {
    /// Opens a Git repository.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Self {
            inner: Staged::new(Inner {
                repository: ThreadSafeRepository::open(path)?,
                revision: Revision::Head,
                cache: Cache::new(),
            }),
        })
    }

    /// Initializes a Git repository.
    pub fn init(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            inner: Staged::new(Inner {
                repository: ThreadSafeRepository::init(
                    path,
                    Kind::WithWorktree,
                    Options::default(),
                )?,
                revision: Revision::Head,
                cache: Cache::new(),
            }),
        })
    }
}

impl Git {
    /// Gets the revision.
    pub fn revision(&self) -> &Revision {
        &self.inner.inner.revision
    }

    /// Sets the revision.
    pub fn set_revision(&mut self, revision: impl Into<Revision>) {
        self.inner.inner.revision = revision.into();
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

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        self.inner.get_file(path)
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        self.inner.get_index()
    }
}

impl Stage for Git {
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

impl Commit for Git {
    type Context = CommitParams;

    fn commit(&mut self, context: impl Into<Self::Context>) -> Result<(), Self::Error> {
        let context = context.into();
        let repo = self.inner.inner.repository.to_thread_local();
        let revision = self.inner.inner.revision.to_string();

        let (mut editor, parent) = match &self.inner.inner.revision {
            Revision::Head if repo.head()?.is_unborn() => (
                repo.edit_tree(ObjectId::empty_tree(repo.object_hash()))?,
                None,
            ),
            _ => {
                let object_id = repo.rev_parse_single(&*revision)?;
                let commit_id = object_id.object()?.peel_to_commit()?.id();
                let editor = object_id.object()?.peel_to_tree()?.edit()?;

                (editor, Some(commit_id))
            }
        };

        for (path, file) in self.inner.drain() {
            match file {
                Some(bytes) => {
                    let blob_id = repo.write_blob(&bytes)?;

                    editor.upsert(path.as_str(), gix::object::tree::EntryKind::Blob, blob_id)?;
                }
                None => {
                    editor.remove(path.as_str())?;
                }
            }
        }

        let tree_id = editor.write()?;

        match &self.inner.inner.revision {
            Revision::Head | Revision::Reference(Reference::Branch(_)) => {
                repo.commit(revision, context.message(), tree_id, parent)?;
            }
            _ => {
                let author = repo
                    .author()
                    .ok_or(gix::commit::Error::AuthorMissing)?
                    .map_err(gix::commit::Error::from)?;
                let committer = repo
                    .committer()
                    .ok_or(gix::commit::Error::CommitterMissing)?
                    .map_err(gix::commit::Error::from)?;

                let commit = gix::objs::Commit {
                    message: context.message().into(),
                    tree: tree_id.into(),
                    author: author.into(),
                    committer: committer.into(),
                    encoding: None,
                    parents: parent.into_iter().map(Into::into).collect(),
                    extra_headers: Default::default(),
                };

                let commit_id = repo.write_object(&commit)?;

                self.inner.inner.revision = Revision::Sha(commit_id.to_string());
                self.inner.inner.cache = Cache::new();
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct Inner {
    repository: ThreadSafeRepository,
    revision: Revision,
    cache: Cache,
}

impl Repository for Inner {
    type Error = Error;

    fn get_file(&self, path: impl AsRef<RelativePath>) -> Result<Option<Bytes>, Self::Error> {
        let res = if !matches!(self.revision, Revision::Sha(_)) {
            self.get_file_uncached(path.as_ref()).map(Some)
        } else {
            self.cache.get_or_try_init(
                path,
                |path| self.get_file_uncached(path),
                || self.get_index_uncached(),
            )
        };

        match res {
            Ok(file) => Ok(file),
            Err(Error::Io(err)) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn get_index(&self) -> Result<impl Iterator<Item = RelativePathBuf>, Self::Error> {
        if !matches!(&self.revision, Revision::Sha(_)) {
            return Ok(Box::new(self.get_index_uncached()?.into_iter())
                as Box<dyn Iterator<Item = RelativePathBuf>>);
        }

        Ok(Box::new(
            self.cache.get_or_try_index(|| self.get_index_uncached())?,
        ))
    }
}

impl Inner {
    fn get_index_uncached(&self) -> Result<BTreeSet<RelativePathBuf>, Error> {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

        let mut recorder = Recorder::default();

        tree.traverse().breadthfirst(&mut recorder)?;

        let entries = recorder
            .records
            .into_iter()
            .filter(|entry| entry.mode.is_blob())
            .map(|entry| RelativePathBuf::from(entry.filepath.to_string()))
            .collect::<BTreeSet<_>>();

        Ok(entries)
    }

    fn get_file_uncached(&self, path: impl AsRef<RelativePath>) -> Result<Bytes, Error> {
        let spec = self.revision.to_string();
        let repo = self.repository.to_thread_local();
        let mut tree = repo.rev_parse_single(&*spec)?.object()?.peel_to_tree()?;

        let entry = tree
            .peel_to_entry_by_path(path.as_ref().as_str())?
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

        if entry.mode().is_blob() {
            Ok(entry.object()?.detached().data.into())
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
    }
}
