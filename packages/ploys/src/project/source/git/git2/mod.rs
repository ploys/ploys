use std::io;
use std::path::{Path, PathBuf};

use auth_git2::GitAuthenticator;
use git2::build::TreeUpdateBuilder;
use git2::{
    FileMode, ObjectType, PushOptions, RemoteCallbacks, Repository, TreeWalkMode, TreeWalkResult,
};
use url::Url;

use crate::project::source::revision::{Reference, Revision};

use super::{Error, GitConfig, Source};

/// The local Git repository source using `git2`.
pub struct Git2 {
    repository: Repository,
    revision: Revision,
}

impl Git2 {
    /// Creates a Git2 source.
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            repository: Repository::open(path.as_ref())?,
            revision: Revision::Head,
        })
    }
}

impl Git2 {
    /// Gets the revision.
    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    /// Sets the revision.
    pub fn set_revision(&mut self, revision: impl Into<Revision>) {
        self.revision = revision.into();
    }

    /// Builds the source with the given revision.
    pub fn with_revision(mut self, revision: impl Into<Revision>) -> Self {
        self.set_revision(revision);
        self
    }
}

impl Git2 {
    /// Creates a new branch.
    pub(crate) fn create_branch(&self, branch_name: &str) -> Result<String, Error> {
        let spec = self.revision.to_string();
        let commit = self.repository.revparse_single(&spec)?.peel_to_commit()?;
        let sha = commit.id().to_string();
        let remote_name = self
            .get_default_remote_name()?
            .ok_or_else(Error::remote_not_found)?;

        let mut remote = self.repository.find_remote(&remote_name)?;
        let mut branch = self.repository.branch(branch_name, &commit, false)?;

        let config = self.repository.config()?;
        let auth = GitAuthenticator::new_empty()
            .try_cred_helper(true)
            .try_ssh_agent(true)
            .add_default_ssh_keys();

        let mut remote_callbacks = RemoteCallbacks::new();
        let mut options = PushOptions::default();

        remote_callbacks.credentials(auth.credentials(&config));
        options.remote_callbacks(remote_callbacks);

        let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");

        branch.set_upstream(Some(branch_name))?;
        remote.push(&[refspec], Some(&mut options))?;

        Ok(sha)
    }

    /// Commits the changes to the repository.
    pub(crate) fn commit(
        &self,
        message: impl AsRef<str>,
        files: impl Iterator<Item = (PathBuf, String)>,
    ) -> Result<String, Error> {
        let spec = self.revision.to_string();
        let tree = self.repository.revparse_single(&spec)?.peel_to_tree()?;
        let commit = self.repository.revparse_single(&spec)?.peel_to_commit()?;
        let signature = self.repository.signature()?;
        let mut tree_builder = TreeUpdateBuilder::new();

        for (path, content) in files {
            let blob = self.repository.blob(content.as_bytes())?;

            tree_builder.upsert(path, blob, FileMode::Blob);
        }

        let tree_id = tree_builder.create_updated(&self.repository, &tree)?;
        let tree = self.repository.find_tree(tree_id)?;
        let update_ref = match &self.revision {
            Revision::Head | Revision::Reference(Reference::Branch(_)) => {
                Some(self.revision.to_string())
            }
            _ => None,
        };

        let sha = self.repository.commit(
            update_ref.as_deref(),
            &signature,
            &signature,
            message.as_ref(),
            &tree,
            &[&commit],
        )?;

        if let Revision::Reference(Reference::Branch(_)) = &self.revision {
            let config = self.repository.config()?;
            let auth = GitAuthenticator::new_empty()
                .try_cred_helper(true)
                .try_ssh_agent(true)
                .add_default_ssh_keys();
            let remote_name = self
                .get_default_remote_name()?
                .ok_or_else(Error::remote_not_found)?;

            let mut remote = self.repository.find_remote(&remote_name)?;
            let mut remote_callbacks = RemoteCallbacks::new();
            let mut options = PushOptions::default();

            remote_callbacks.credentials(auth.credentials(&config));
            options.remote_callbacks(remote_callbacks);

            let refspec = format!("{}:{}", self.revision, self.revision);

            remote.push(&[refspec], Some(&mut options))?;
        }

        Ok(sha.to_string())
    }

    /// Gets the default remote name.
    ///
    /// This replicates the logic from `remote_default_name` in `gix`.
    fn get_default_remote_name(&self) -> Result<Option<String>, Error> {
        let remote_name = self
            .repository
            .config()?
            .get_str("remote.pushDefault")
            .map(ToOwned::to_owned)
            .map(Some)
            .or_else(|_| {
                let remotes = self.repository.remotes()?;

                Ok::<_, Error>(match remotes.len() {
                    0 => None,
                    1 => remotes.get(0).map(ToOwned::to_owned),
                    _ => remotes
                        .iter()
                        .any(|remote| remote == Some("origin"))
                        .then_some("origin")
                        .map(ToOwned::to_owned),
                })
            })?;

        Ok(remote_name)
    }
}

impl Source for Git2 {
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
        let remote_name = self
            .get_default_remote_name()?
            .ok_or_else(Error::remote_not_found)?;

        let remote = self.repository.find_remote(&remote_name)?;

        match remote.url() {
            Some(url) => Ok(Url::parse(url).expect("url")),
            None => Err(Error::remote_not_found()),
        }
    }

    fn get_files(&self) -> Result<Vec<PathBuf>, Self::Error> {
        let spec = self.revision.to_string();
        let tree = self.repository.revparse_single(&spec)?.peel_to_tree()?;
        let mut entries = Vec::new();

        tree.walk(TreeWalkMode::PreOrder, |path, entry| {
            if let Some(ObjectType::Blob) = entry.kind() {
                if let Some(name) = entry.name() {
                    entries.push(Path::new(path).join(name));
                }
            }

            TreeWalkResult::Ok
        })?;

        entries.sort();

        Ok(entries)
    }

    fn get_file_contents<P>(&self, path: P) -> Result<Vec<u8>, Self::Error>
    where
        P: AsRef<Path>,
    {
        let spec = self.revision.to_string();
        let tree = self.repository.revparse_single(&spec)?.peel_to_tree()?;
        let entry = tree.get_path(path.as_ref())?;
        let object = entry.to_object(&self.repository)?;

        if let Some(blob) = object.as_blob() {
            Ok(blob.content().to_vec())
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
    }
}
