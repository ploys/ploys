use std::collections::hash_map::IntoValues;
use std::collections::HashMap;
use std::iter::Flatten;
use std::path::{Path, PathBuf};

use crate::package::{Lockfile, Package, PackageKind};

use super::File;

/// A collection of files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    pub fn with_files<I, P, T>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<PathBuf>,
        T: Into<File>,
    {
        self.extend(files);
        self
    }

    /// Inserts the given file.
    pub fn insert_file(&mut self, path: impl Into<PathBuf>, file: impl Into<File>) -> &mut Self {
        let file = file.into();

        self.files.insert(path.into(), Some(file));
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
    pub fn get_package_by_name(&self, name: impl AsRef<str>) -> Option<(&Path, &Package)> {
        self.packages()
            .find(|(_, package)| package.name() == name.as_ref())
    }

    /// Gets a mutable package with the given name.
    pub fn get_package_by_name_mut(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<(&Path, &mut Package)> {
        self.packages_mut()
            .find(|(_, package)| package.name() == name.as_ref())
    }

    /// Gets a lockfile with the given kind.
    pub fn get_lockfile_by_kind(&self, kind: PackageKind) -> Option<&Lockfile> {
        self.get_file(kind.lockfile_name()?)?.as_lockfile()
    }

    /// Gets a mutable lockfile with the given kind.
    pub fn get_lockfile_by_kind_mut(&mut self, kind: PackageKind) -> Option<&mut Lockfile> {
        self.get_file_mut(kind.lockfile_name()?)?.as_lockfile_mut()
    }

    /// Gets an iterator over the files.
    pub fn files(&self) -> impl Iterator<Item = (&Path, &File)> {
        self.files
            .iter()
            .filter_map(|(path, file)| Some((path.as_path(), file.as_ref()?)))
    }

    /// Gets an iterator over the mutable files.
    pub fn files_mut(&mut self) -> impl Iterator<Item = (&Path, &mut File)> {
        self.files
            .iter_mut()
            .filter_map(|(path, file)| Some((path.as_path(), file.as_mut()?)))
    }

    /// Gets an iterator over the packages.
    pub fn packages(&self) -> impl Iterator<Item = (&Path, &Package)> {
        self.files()
            .filter_map(|(path, file)| Some((path, file.as_package()?)))
    }

    /// Gets an iterator over the mutable packages.
    pub fn packages_mut(&mut self) -> impl Iterator<Item = (&Path, &mut Package)> {
        self.files_mut()
            .filter_map(|(path, file)| Some((path, file.as_package_mut()?)))
    }

    /// Gets an iterator over the lockfiles.
    pub fn lockfiles(&self) -> impl Iterator<Item = (&Path, &Lockfile)> {
        self.files()
            .filter_map(|(path, file)| Some((path, file.as_lockfile()?)))
    }

    /// Gets an iterator over the mutable lockfiles.
    pub fn lockfiles_mut(&mut self) -> impl Iterator<Item = (&Path, &mut Lockfile)> {
        self.files_mut()
            .filter_map(|(path, file)| Some((path, file.as_lockfile_mut()?)))
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

impl<P, T> Extend<(P, T)> for Fileset
where
    P: Into<PathBuf>,
    T: Into<File>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (P, T)>,
    {
        self.files.extend(
            iter.into_iter()
                .map(|(path, file)| (path.into(), Some(file.into()))),
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

impl<P, T> FromIterator<(P, T)> for Fileset
where
    P: Into<PathBuf>,
    T: Into<File>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (P, T)>,
    {
        Self {
            files: iter
                .into_iter()
                .map(|(path, file)| (path.into(), Some(file.into())))
                .collect(),
        }
    }
}
