use semver::Version;

use crate::changelog::{Changelog, Release};
use crate::file::File;

use super::{BumpOrVersion, Package};

/// The release request.
pub struct ReleaseRequest<'a> {
    #[allow(dead_code)]
    package: Package<'a>,
    id: u64,
    title: String,
    notes: Release,
    version: Version,
}

impl ReleaseRequest<'_> {
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
pub struct ReleaseRequestBuilder<'a> {
    package: Package<'a>,
    version: BumpOrVersion,
    options: Options,
}

impl<'a> ReleaseRequestBuilder<'a> {
    /// Constructs a new release request builder.
    pub(super) fn new(package: Package<'a>, version: BumpOrVersion) -> Self {
        Self {
            package,
            version,
            options: Options::default(),
        }
    }

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

    /// Finishes the release request.
    pub fn finish(mut self) -> Result<ReleaseRequest<'a>, crate::project::Error> {
        let Some(remote) = self.package.project.get_remote() else {
            return Err(crate::project::Error::Unsupported);
        };

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

        if self.options.update_package_manifest {
            files.push((self.package.path().to_owned(), self.package.to_string()));
        }

        if self.options.update_dependent_package_manifests {
            for mut package in self.package.project.packages() {
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
                    files.push((package.path().to_owned(), package.to_string()));
                }
            }
        }

        if self.options.update_lockfile {
            if let Some(path) = self.package.kind().lockfile_name() {
                if let Some(File::Lockfile(lockfile)) = self.package.project.get_file(path) {
                    let mut lockfile = lockfile.clone();

                    lockfile.set_package_version(self.package.name(), version.clone());
                    files.push((path.to_owned(), lockfile.to_string()));
                }
            }
        }

        let mut release = self
            .package
            .project
            .get_changelog_release(self.package.name(), &version)?;

        if self.options.update_changelog {
            let path = self
                .package
                .path()
                .parent()
                .expect("parent")
                .join("CHANGELOG.md");

            let mut changelog = match self.package.project.get_file_contents(&path).ok() {
                Some(bytes) => match String::from_utf8(bytes).ok() {
                    Some(string) => string.parse::<Changelog>().expect("changelog"),
                    None => Changelog::new(),
                },
                None => Changelog::new(),
            };

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

        let default_branch = remote.get_default_branch()?;

        remote.create_branch(&branch)?;

        let sha = remote.commit(&title, files)?;

        remote.update_branch(&branch, &sha)?;

        let id = remote.create_pull_request(&branch, &default_branch, &title, &body)?;

        Ok(ReleaseRequest {
            package: self.package,
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
