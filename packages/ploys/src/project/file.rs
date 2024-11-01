use std::collections::hash_map::IntoValues;
use std::collections::HashMap;
use std::iter::Flatten;
use std::path::{Path, PathBuf};

use crate::lockfile::LockFile;
use crate::package::{Package, PackageKind};

/// A collection of files.
#[derive(Clone, Debug, Default)]
pub struct Fileset {
    /// The internal file map.
    ///
    /// This maps the path to an optional file where the `Some` option variant
    /// represents a file that exists, `None` represents a file that does not
    /// exist, and no entry where the existence is not yet known.
    files: HashMap<PathBuf, Option<File>>,
}

impl Fileset {
    /// Constructs a new fileset.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Builds the fileset with the given files.
    pub fn with_files<I, T>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<File>,
    {
        self.extend(files);
        self
    }

    /// Inserts the given file.
    pub fn insert_file(&mut self, file: impl Into<File>) -> &mut Self {
        let file = file.into();

        self.files.insert(file.path().to_owned(), Some(file));
        self
    }

    /// Gets the file at the given path.
    pub fn get_file(&self, path: impl AsRef<Path>) -> Option<&File> {
        self.files.get(path.as_ref())?.as_ref()
    }

    /// Gets the mutable file at the given path.
    pub fn get_file_mut(&mut self, path: impl AsRef<Path>) -> Option<&mut File> {
        self.files.get_mut(path.as_ref())?.as_mut()
    }

    /// Gets a package with the given name.
    pub fn get_package_by_name(&self, name: impl AsRef<str>) -> Option<&Package> {
        self.packages()
            .find(|package| package.name() == name.as_ref())
    }

    /// Gets a mutable package with the given name.
    pub fn get_package_by_name_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Package> {
        self.packages_mut()
            .find(|package| package.name() == name.as_ref())
    }

    /// Gets a lockfile with the given kind.
    pub fn get_lockfile_by_kind(&self, kind: PackageKind) -> Option<&LockFile> {
        self.get_file(kind.lockfile_name()?)?.as_lockfile()
    }

    /// Gets a mutable lockfile with the given kind.
    pub fn get_lockfile_by_kind_mut(&mut self, kind: PackageKind) -> Option<&mut LockFile> {
        self.get_file_mut(kind.lockfile_name()?)?.as_lockfile_mut()
    }

    /// Gets an iterator over the files.
    pub fn files(&self) -> impl Iterator<Item = &File> {
        self.files.values().filter_map(Option::as_ref)
    }

    /// Gets an iterator over the mutable files.
    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut File> {
        self.files.values_mut().filter_map(Option::as_mut)
    }

    /// Gets an iterator over the packages.
    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.files().filter_map(File::as_package)
    }

    /// Gets an iterator over the mutable packages.
    pub fn packages_mut(&mut self) -> impl Iterator<Item = &mut Package> {
        self.files_mut().filter_map(File::as_package_mut)
    }

    /// Gets an iterator over the lockfiles.
    pub fn lockfiles(&self) -> impl Iterator<Item = &LockFile> {
        self.files().filter_map(File::as_lockfile)
    }

    /// Gets an iterator over the mutable lockfiles.
    pub fn lockfiles_mut(&mut self) -> impl Iterator<Item = &mut LockFile> {
        self.files_mut().filter_map(File::as_lockfile_mut)
    }

    /// Gets the number of files in the fileset.
    pub fn len(&self) -> usize {
        self.files.values().filter(|file| file.is_some()).count()
    }

    /// Checks whether the fileset is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Extend<T> for Fileset
where
    T: Into<File>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.files.extend(
            iter.into_iter()
                .map(Into::into)
                .map(|file| (file.path().to_owned(), Some(file))),
        );
    }
}

impl IntoIterator for Fileset {
    type Item = File;
    type IntoIter = Flatten<IntoValues<PathBuf, Option<File>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_values().flatten()
    }
}

impl<T> FromIterator<T> for Fileset
where
    T: Into<File>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            files: iter
                .into_iter()
                .map(Into::into)
                .map(|file| (file.path().to_owned(), Some(file)))
                .collect(),
        }
    }
}

/// A file in one of a number of formats.
#[derive(Clone, Debug)]
pub enum File {
    Package(Package),
    Lockfile(LockFile),
}

impl File {
    /// Gets the file path.
    pub fn path(&self) -> &Path {
        match self {
            Self::Package(package) => package.path(),
            Self::Lockfile(lockfile) => lockfile.kind().lockfile_name().expect("path"),
        }
    }

    /// Gets the file as a package.
    pub fn as_package(&self) -> Option<&Package> {
        match self {
            Self::Package(package) => Some(package),
            _ => None,
        }
    }

    /// Gets the file as a mutable package.
    pub fn as_package_mut(&mut self) -> Option<&mut Package> {
        match self {
            Self::Package(package) => Some(package),
            _ => None,
        }
    }

    /// Gets the file as a lockfile.
    pub fn as_lockfile(&self) -> Option<&LockFile> {
        match self {
            Self::Lockfile(lockfile) => Some(lockfile),
            _ => None,
        }
    }

    /// Gets the file as a mutable lockfile.
    pub fn as_lockfile_mut(&mut self) -> Option<&mut LockFile> {
        match self {
            Self::Lockfile(lockfile) => Some(lockfile),
            _ => None,
        }
    }
}

impl From<Package> for File {
    fn from(value: Package) -> Self {
        Self::Package(value)
    }
}

impl From<LockFile> for File {
    fn from(value: LockFile) -> Self {
        Self::Lockfile(value)
    }
}
