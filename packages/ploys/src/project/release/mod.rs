mod request;

use tracing::{info, info_span};

use crate::repository::Remote;

pub use self::request::{ReleaseRequest, ReleaseRequestBuilder};

use super::{Package, Project};

/// The package release.
pub struct Release {
    id: u64,
    name: String,
    notes: crate::changelog::Release,
}

impl Release {
    /// Gets the release id.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the release name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the release notes.
    pub fn notes(&self) -> &crate::changelog::Release {
        &self.notes
    }
}

/// The package release builder.
pub struct ReleaseBuilder<'a, T> {
    project: &'a Project<T>,
    package: Package<&'a T>,
}

impl<'a, T> ReleaseBuilder<'a, T> {
    /// Constructs a new release builder.
    pub(crate) fn new(project: &'a Project<T>, package: Package<&'a T>) -> Self {
        Self { project, package }
    }
}

impl<T> ReleaseBuilder<'_, T>
where
    T: Remote,
{
    /// Finishes the release.
    pub fn finish(self) -> Result<Release, T::Error> {
        let sha = self.project.repository.sha()?;

        let version = self.package.version();

        let span = info_span!("release", package = self.package.name(), %version);
        let _enter = span.enter();

        info!("Creating release");

        let prerelease = !version.pre.is_empty();
        let latest = self.package.is_primary() && !prerelease;

        let tag = match self.package.is_primary() {
            true => version.to_string(),
            false => format!("{}-{version}", self.package.name()),
        };

        let name = match self.package.is_primary() {
            true => version.to_string(),
            false => format!("{} {version}", self.package.name()),
        };

        let changelog = self.package.changelog();
        let release = changelog
            .as_ref()
            .and_then(|changelog| changelog.get_release(version.to_string()));

        let release = match release {
            Some(release) => release.to_owned(),
            None => self.package.build_release_notes(&version)?,
        };

        let body = format!("{release:#}")
            .lines()
            .skip(2)
            .collect::<Vec<_>>()
            .join("\n");

        let id = self
            .project
            .repository
            .create_release(&tag, &sha, &name, &body, prerelease, latest)?;

        info!(id, "Created release");

        Ok(Release {
            id,
            name,
            notes: release,
        })
    }
}
