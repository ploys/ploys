use relative_path::RelativePathBuf;
use semver::Version;
use tracing::{info, info_span};

use crate::changelog::Release;
use crate::package::{BumpOrVersion, Lockfile, Package};
use crate::project::Project;

use super::Remote;

/// The release request.
pub struct ReleaseRequest {
    id: u64,
    title: String,
    notes: Release,
    version: Version,
}

impl ReleaseRequest {
    /// Gets the release request id.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the release request title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Gets the release request notes.
    pub fn notes(&self) -> &Release {
        &self.notes
    }

    /// Gets the release request version.
    pub fn version(&self) -> &Version {
        &self.version
    }
}

/// The release request builder.
///
/// This configures the release request that will be generated on the remote
/// repository.
pub struct ReleaseRequestBuilder<'a, T> {
    project: &'a Project<T>,
    package: Package<T>,
    version: BumpOrVersion,
    options: Options,
}

impl<'a, T> ReleaseRequestBuilder<'a, T> {
    /// Constructs a new release request builder.
    pub(crate) fn new(
        project: &'a Project<T>,
        package: Package<T>,
        version: BumpOrVersion,
    ) -> Self {
        Self {
            project,
            package,
            version,
            options: Options::default(),
        }
    }
}

impl<T> ReleaseRequestBuilder<'_, T> {
    /// Update the package manifest.
    pub fn update_package_manifest(mut self, enable: bool) -> Self {
        self.options.update_package_manifest = enable;
        self
    }

    /// Update the dependent package manifests.
    pub fn update_dependent_package_manifests(mut self, enable: bool) -> Self {
        self.options.update_dependent_package_manifests = enable;
        self
    }

    /// Update the workspace lockfile.
    pub fn update_lockfile(mut self, enable: bool) -> Self {
        self.options.update_lockfile = enable;
        self
    }

    /// Update the package changelog.
    pub fn update_changelog(mut self, enable: bool) -> Self {
        self.options.update_changelog = enable;
        self
    }
}

impl<T> ReleaseRequestBuilder<'_, T>
where
    T: Remote,
{
    /// Finishes the release request.
    pub fn finish(mut self) -> Result<ReleaseRequest, crate::project::Error<T::Error>> {
        let mut files = Vec::new();

        let version = match self.version {
            BumpOrVersion::Bump(bump) => {
                self.package.bump_version(bump)?;
                self.package.version()
            }
            BumpOrVersion::Version(version) => {
                self.package.set_version(version.clone());
                version
            }
        };

        let span = info_span!("release_request", package = self.package.name(), %version);
        let _enter = span.enter();

        info!("Creating release request");

        if self.options.update_package_manifest {
            files.push((
                self.package.manifest_path().to_owned(),
                self.package.manifest().to_string(),
            ));
        }

        if self.options.update_dependent_package_manifests {
            for mut package in self.project.packages() {
                if package.name() == self.package.name() {
                    continue;
                }

                let mut changed = false;

                if let Some(mut dependency) = package.get_dependency_mut(self.package.name()) {
                    dependency.set_version(version.clone());
                    changed = true;
                }

                if let Some(mut dependency) = package.get_dev_dependency_mut(self.package.name()) {
                    dependency.set_version(version.clone());
                    changed = true;
                }

                if let Some(mut dependency) = package.get_build_dependency_mut(self.package.name())
                {
                    dependency.set_version(version.clone());
                    changed = true;
                }

                if changed {
                    files.push((
                        package.manifest_path().to_owned(),
                        package.manifest().to_string(),
                    ));
                }
            }
        }

        if self.options.update_lockfile
            && let Some(path) = self.package.kind().lockfile_name()
        {
            let lockfile = self
                .project
                .repository
                .get_file(path)
                .ok()
                .flatten()
                .and_then(|bytes| Lockfile::from_bytes(self.package.kind(), &bytes).ok());

            if let Some(mut lockfile) = lockfile {
                lockfile.set_package_version(self.package.name(), version.clone());
                files.push((RelativePathBuf::from(path), lockfile.to_string()));
            }
        }

        let mut release = self
            .package
            .build_release_notes(&version)
            .map_err(crate::project::Error::Repository)?;

        if self.options.update_changelog {
            let path = self.package.path().join("CHANGELOG.md");

            let mut changelog = self.package.changelog().unwrap_or_default();

            changelog.add_release(release.clone());
            files.push((path, changelog.to_string()));
        }

        release.set_description(format!(
            "Releasing package `{}` version `{version}`.",
            self.package.name()
        ));

        if let Some(url) = release.url() {
            release.add_reference(version.to_string(), url.to_string());
        }

        let body = release.to_string();
        let title = match self.package.is_primary() {
            true => format!("Release `{version}`"),
            false => format!("Release `{}@{version}`", self.package.name()),
        };
        let branch = match self.package.is_primary() {
            true => format!("release/{version}"),
            false => format!("release/{}-{version}", self.package.name()),
        };

        let default_branch = self
            .project
            .repository
            .get_default_branch()
            .map_err(crate::project::Error::Repository)?;

        self.project
            .repository
            .create_branch(&branch)
            .map_err(crate::project::Error::Repository)?;

        let sha = self
            .project
            .repository
            .commit(&title, files)
            .map_err(crate::project::Error::Repository)?;

        self.project
            .repository
            .update_branch(&branch, &sha)
            .map_err(crate::project::Error::Repository)?;

        let id = self
            .project
            .repository
            .create_pull_request(&branch, &default_branch, &title, &body)
            .map_err(crate::project::Error::Repository)?;

        info!(id, "Created release request");

        Ok(ReleaseRequest {
            id,
            title,
            notes: release,
            version,
        })
    }
}

/// The release request options.
struct Options {
    update_package_manifest: bool,
    update_dependent_package_manifests: bool,
    update_lockfile: bool,
    update_changelog: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            update_package_manifest: true,
            update_dependent_package_manifests: true,
            update_lockfile: true,
            update_changelog: true,
        }
    }
}
