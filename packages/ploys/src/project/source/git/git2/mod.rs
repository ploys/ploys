use std::io;
use std::path::{Path, PathBuf};

use git2::{ObjectType, Repository, TreeWalkMode, TreeWalkResult};
use url::Url;

use super::{Error, GitConfig, Source};

/// The local Git repository source using `git2`.
pub struct Git2 {
    repository: Repository,
}

impl Git2 {
    /// Creates a Git2 source.
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            repository: Repository::open(path.as_ref())?,
        })
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
        let tree = self.repository.revparse_single("HEAD")?.peel_to_tree()?;
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
        let tree = self.repository.revparse_single("HEAD")?.peel_to_tree()?;
        let entry = tree.get_path(path.as_ref())?;
        let object = entry.to_object(&self.repository)?;

        if let Some(blob) = object.as_blob() {
            Ok(blob.content().to_vec())
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
    }
}
