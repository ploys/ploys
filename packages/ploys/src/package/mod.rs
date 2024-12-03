//! Package inspection and management utilities
//!
//! This module includes utilities for inspecting and managing packages located
//! on the local file system or in a remote version control system.

mod bump;
mod error;
mod kind;
pub mod lockfile;
pub mod manifest;
mod release;

use std::borrow::Cow;
use std::fmt::{self, Display};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use semver::Version;

use crate::changelog::Changelog;
use crate::file::File;
use crate::project::Project;

pub use self::bump::{Bump, BumpOrVersion, Error as BumpError};
pub use self::error::Error;
pub use self::kind::PackageKind;
pub use self::lockfile::Lockfile;
use self::manifest::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::manifest::{Manifest, Members};
pub use self::release::{ReleaseRequest, ReleaseRequestBuilder};

/// A project package.
#[derive(Clone)]
pub struct Package<'a> {
    manifest: Cow<'a, Manifest>,
    path: PathBuf,
    project: &'a Project,
}

impl Package<'_> {
    /// Gets the package name.
    pub fn name(&self) -> &str {
        match &*self.manifest {
            Manifest::Cargo(cargo) => cargo.package().expect("package").name(),
        }
    }

    /// Gets the package description.
    pub fn description(&self) -> Option<&str> {
        match &*self.manifest {
            Manifest::Cargo(cargo) => cargo.package().expect("package").description(),
        }
    }

    /// Gets the package version.
    pub fn version(&self) -> Version {
        match &*self.manifest {
            Manifest::Cargo(cargo) => cargo.package().expect("package").version(),
        }
    }

    /// Sets the package version.
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        match self.manifest.to_mut() {
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

    /// Gets the package path.
    pub fn path(&self) -> &Path {
        &self.path
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
        self.name() == self.project.name()
    }
}

impl<'a> Package<'a> {
    /// Gets the package changelog.
    pub fn changelog(&self) -> Option<&'a Changelog> {
        self.project
            .get_file(self.path().parent()?.join("CHANGELOG.md"))
            .and_then(File::try_as_changelog_ref)
    }
}

impl Package<'_> {
    /// Gets the dependency with the given name.
    pub fn get_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.manifest.get_dependency(name)
    }

    /// Gets the mutable dependency with the given name.
    pub fn get_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest.to_mut().get_dependency_mut(name)
    }

    /// Gets the dependencies.
    pub fn dependencies(&self) -> Dependencies<'_> {
        self.manifest.dependencies()
    }

    /// Gets the mutable dependencies.
    pub fn dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.to_mut().dependencies_mut()
    }
}

impl Package<'_> {
    /// Gets the dev dependency with the given name.
    pub fn get_dev_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.manifest.get_dev_dependency(name)
    }

    /// Gets the mutable dev dependency with the given name.
    pub fn get_dev_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest.to_mut().get_dev_dependency_mut(name)
    }

    /// Gets the dev dependencies.
    pub fn dev_dependencies(&self) -> Dependencies<'_> {
        self.manifest.dev_dependencies()
    }

    /// Gets the mutable dev dependencies.
    pub fn dev_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.to_mut().dev_dependencies_mut()
    }
}

impl Package<'_> {
    /// Gets the build dependency with the given name.
    pub fn get_build_dependency(&self, name: impl AsRef<str>) -> Option<Dependency<'_>> {
        self.manifest.get_build_dependency(name)
    }

    /// Gets the mutable build dependency with the given name.
    pub fn get_build_dependency_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        self.manifest.to_mut().get_build_dependency_mut(name)
    }

    /// Gets the build dependencies.
    pub fn build_dependencies(&self) -> Dependencies<'_> {
        self.manifest.build_dependencies()
    }

    /// Gets the mutable build dependencies.
    pub fn build_dependencies_mut(&mut self) -> DependenciesMut<'_> {
        self.manifest.to_mut().build_dependencies_mut()
    }
}

impl<'a> Package<'a> {
    /// Constructs a new release request builder.
    pub fn create_release_request(
        self,
        version: impl Into<BumpOrVersion>,
    ) -> ReleaseRequestBuilder<'a> {
        ReleaseRequestBuilder::new(self, version.into())
    }
}

impl<'a> Package<'a> {
    /// Constructs a package from a manifest.
    pub(super) fn from_manifest(
        project: &'a Project,
        path: impl Into<PathBuf>,
        manifest: &'a Manifest,
    ) -> Option<Self> {
        match manifest.package_kind() {
            PackageKind::Cargo => {
                manifest.try_as_cargo_ref()?.package()?;
            }
        }

        Some(Self {
            manifest: Cow::Borrowed(manifest),
            path: path.into(),
            project,
        })
    }
}

impl Deref for Package<'_> {
    type Target = Manifest;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

impl DerefMut for Package<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.manifest.to_mut()
    }
}

impl Display for Package<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.manifest, f)
    }
}
