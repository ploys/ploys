//! Package inspection and management utilities
//!
//! This module includes utilities for inspecting and managing packages located
//! on the local file system or in a remote version control system.

mod bump;
mod error;
mod kind;
pub mod lockfile;
pub mod manifest;

use std::borrow::{Borrow, Cow};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use either::Either;
use semver::Version;
use tracing::info;
use url::Url;

use crate::changelog::Changelog;
use crate::project::Project;
use crate::repository::memory::Memory;
use crate::repository::{Remote, Repository};

pub use self::bump::{Bump, BumpOrVersion, Error as BumpError};
pub use self::error::Error;
pub use self::kind::PackageKind;
pub use self::lockfile::Lockfile;
pub use self::manifest::Manifest;
use self::manifest::{Dependencies, DependenciesMut, DependencyMut, DependencyRef};

/// A project package.
#[derive(Clone)]
pub struct Package<T = Memory> {
    pub(crate) repository: T,
    manifest: Manifest,
    path: PathBuf,
    primary: bool,
}

impl Package {
    /// Constructs a new cargo package.
    pub fn new_cargo(name: impl Into<String>) -> Self {
        Self {
            repository: Memory::new(),
            manifest: Manifest::new_cargo(name),
            path: PathBuf::new(),
            primary: false,
        }
    }
}

impl<T> Package<T> {
    /// Gets the package name.
    pub fn name(&self) -> &str {
        match self.manifest() {
            Manifest::Cargo(cargo) => cargo.package().expect("package").name(),
        }
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&str> {
        match self.manifest() {
            Manifest::Cargo(cargo) => cargo.package().expect("package").description(),
        }
    }

    /// Sets the package description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        match self.manifest_mut() {
            Manifest::Cargo(cargo) => {
                cargo
                    .package_mut()
                    .expect("package")
                    .set_description(description);
            }
        }

        self
    }

    /// Builds the package with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    /// Gets the package version.
    pub fn version(&self) -> Version {
        match self.manifest() {
            Manifest::Cargo(cargo) => cargo.package().expect("package").version(),
        }
    }

    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        match self.manifest_mut() {
            Manifest::Cargo(cargo) => cargo.package_mut().expect("package").set_version(version),
        };

        self
    }

    /// Bumps the package version.
    pub fn bump_version(&mut self, bump: impl Into<Bump>) -> Result<&mut Self, BumpError> {
        let mut version = self.version();

        bump.into().bump(&mut version)?;
        self.set_version(version);

        Ok(self)
    }

    /// Builds the package with the given version.
    pub fn with_version(mut self, version: impl Into<Version>) -> Self {
        self.set_version(version);
        self
    }

    /// Gets the package repository.
    pub fn repository(&self) -> Option<Url> {
        match self.manifest() {
            Manifest::Cargo(cargo) => cargo.package().expect("package").repository(),
        }
    }

    /// Sets the package repository.
    pub fn set_repository(&mut self, repository: impl Into<Url>) -> &mut Self {
        match self.manifest_mut() {
            Manifest::Cargo(cargo) => {
                cargo
                    .package_mut()
                    .expect("package")
                    .set_repository(repository);
            }
        }

        self
    }

    /// Builds the package with the given repository.
    pub fn with_repository(mut self, repository: impl Into<Url>) -> Self {
        self.set_repository(repository);
        self
    }

    /// Gets the package authors.
    pub fn authors(&self) -> Option<impl IntoIterator<Item = &str>> {
        match self.manifest() {
            Manifest::Cargo(cargo) => cargo.package().expect("package").authors(),
        }
    }

    /// Adds a package author.
    pub fn add_author(&mut self, author: impl Into<String>) -> &mut Self {
        match self.manifest_mut() {
            Manifest::Cargo(cargo) => {
                cargo.package_mut().expect("package").add_author(author);
            }
        }

        self
    }

    /// Builds the package with the given author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.add_author(author);
        self
    }

    /// Gets the package path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Gets the package manifest path.
    pub fn manifest_path(&self) -> PathBuf {
        self.path().join(self.kind().file_name())
    }

    /// Gets the package kind.
    pub fn kind(&self) -> PackageKind {
        self.manifest.package_kind()
    }

    /// Checks if this is the primary package.
    ///
    /// A primary package shares the same name as the project and all releases
    /// are tagged under the version number without the package name prefix.
    pub fn is_primary(&self) -> bool {
        self.primary
    }
}

impl<T> Package<T> {
    /// Gets the package manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Gets the mutable package manifest.
    ///
    /// Note that replacing the manifest with another kind is a logic error and
    /// the behavior is not specified. This will likely lead to incorrect
    /// results and panics.
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }
}

impl<T> Package<T>
where
    T: Repository,
{
    /// Gets the package changelog.
    pub fn changelog(&self) -> Option<Changelog> {
        self.get_file_as("CHANGELOG.md").ok().flatten()
    }
}

impl Package {
    /// Adds a file to the package.
    pub fn add_file(
        &mut self,
        path: impl AsRef<Path>,
        file: impl Into<Cow<'static, [u8]>>,
    ) -> &mut Self {
        self.repository.insert_file(self.path.join(path), file);
        self
    }

    /// Builds the package with the given file.
    pub fn with_file(
        mut self,
        path: impl AsRef<Path>,
        file: impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        self.add_file(path, file);
        self
    }
}

impl<T> Package<T>
where
    T: Repository,
{
    /// Gets a file at the given path.
    pub fn get_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<Cow<'_, [u8]>>, Error<T::Error>> {
        let path = self.path.join(path);

        if path == self.manifest_path() {
            return Ok(Some(Cow::Owned(self.manifest.to_string().into_bytes())));
        }

        self.repository.get_file(path).map_err(Error::Repository)
    }

    /// Gets a file at the given path in the specified format.
    #[allow(clippy::type_complexity)]
    pub fn get_file_as<U>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<U>, Either<Error<T::Error>, U::Err>>
    where
        U: FromStr,
    {
        match self.get_file(path).map_err(Either::Left)? {
            Some(bytes) => match std::str::from_utf8(&bytes) {
                Ok(str) => str.parse().map(Some).map_err(Either::Right),
                Err(err) => Err(Either::Left(Error::Utf8(err))),
            },
            None => Ok(None),
        }
    }
}

impl<T> Package<T> {
    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
        self.manifest().get_dependency(name)
    }

    /// Gets the mutable dependency with the given name.
    pub fn get_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest_mut().get_dependency_mut(name)
    }

    /// Gets the dependencies.
    pub fn dependencies(&self) -> Dependencies<'_> {
        self.manifest().dependencies()
    }

    /// Gets the mutable dependencies.
    pub fn dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest_mut().dependencies_mut()
    }
}

impl<T> Package<T> {
    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
        self.manifest().get_dev_dependency(name)
    }

    /// Gets the mutable dev dependency with the given name.
    pub fn get_dev_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest_mut().get_dev_dependency_mut(name)
    }

    /// Gets the dev dependencies.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        self.manifest().dev_dependencies()
    }

    /// Gets the mutable dev dependencies.
    pub fn dev_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest_mut().dev_dependencies_mut()
    }
}

impl<T> Package<T> {
    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<DependencyRef<'_>> {
        self.manifest().get_build_dependency(name)
    }

    /// Gets the mutable build dependency with the given name.
    pub fn get_build_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest_mut().get_build_dependency_mut(name)
    }

    /// Gets the build dependencies.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        self.manifest().build_dependencies()
    }

    /// Gets the mutable build dependencies.
    pub fn build_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest_mut().build_dependencies_mut()
    }
}

impl<T> Package<T>
where
    T: Remote,
{
    /// Requests the release of the specified package version.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    pub fn request_release(&self, version: impl Into<BumpOrVersion>) -> Result<(), T::Error> {
        let version = version.into();

        info!(
            package = self.name(),
            version = %self.version(),
            request = %version,
            "Requesting release"
        );

        self.repository
            .request_package_release(self.name(), version)?;

        Ok(())
    }

    /// Builds the changelog release for the given package version.
    ///
    /// This method queries the GitHub API to generate new release information
    /// and may differ to the existing release information or changelogs. This
    /// includes information for new releases as well as existing ones.
    ///
    /// It does not yet support parallel release or hotfix branches and expects
    /// all development to be on the default branch in the repository settings.
    pub fn build_release_notes(
        &self,
        version: impl Borrow<Version>,
    ) -> Result<crate::changelog::Release, T::Error> {
        self.repository
            .get_changelog_release(self.name(), version.borrow(), self.is_primary())
    }
}

impl<'a, T> Package<&'a T> {
    /// Constructs a package from a manifest.
    pub(super) fn from_manifest(
        project: &'a Project<T>,
        path: impl Into<PathBuf>,
        manifest: Manifest,
    ) -> Option<Self> {
        let kind = manifest.package_kind();
        let primary = match kind {
            PackageKind::Cargo => {
                let pkg = manifest.try_as_cargo_ref()?.package()?;

                pkg.name() == project.name()
            }
        };

        Some(Self {
            repository: &project.repository,
            manifest: manifest.clone(),
            path: path.into(),
            primary,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::{Package, PackageKind};

    #[test]
    fn test_package_builder() {
        let mut package = Package::new_cargo("example");

        assert_eq!(package.name(), "example");
        assert_eq!(package.description(), None);
        assert_eq!(package.version().to_string(), "0.0.0");
        assert_eq!(package.dependencies().into_iter().count(), 0);
        assert_eq!(package.dev_dependencies().into_iter().count(), 0);
        assert_eq!(package.build_dependencies().into_iter().count(), 0);
        assert_eq!(package.kind(), PackageKind::Cargo);
        assert_eq!(package.path(), Path::new(""));

        package.set_version("0.1.0".parse::<Version>().unwrap());

        assert_eq!(package.version().to_string(), "0.1.0");
        assert_eq!(package.changelog(), None);

        package.add_file("hello-world.txt", b"Hello World!");

        let txt = package.get_file("hello-world.txt").unwrap();

        assert_eq!(txt, Some(b"Hello World!".into()));
    }
}
