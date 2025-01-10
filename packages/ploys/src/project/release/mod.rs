mod request;

use tracing::{info, info_span};

pub use self::request::{ReleaseRequest, ReleaseRequestBuilder};

use super::{Package, Project};

/// The package release.
pub struct Release {
    #[allow(dead_code)]
    package: Package,
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
pub struct ReleaseBuilder<'a> {
    project: &'a Project,
    package: Package,
}

impl<'a> ReleaseBuilder<'a> {
    /// Constructs a new release builder.
    pub(crate) fn new(project: &'a Project, package: Package) -> Self {
        Self { project, package }
    }

    /// Finishes the release.
    pub fn finish(self) -> Result<Release, crate::project::Error> {
        let Some(remote) = self.project.repository.as_remote() else {
            return Err(crate::project::Error::Unsupported);
        };

        let sha = remote.sha()?;

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

        let id = remote.create_release(&tag, &sha, &name, &body, prerelease, latest)?;

        info!(id, "Created release");

        Ok(Release {
            package: self.package,
            id,
            name,
            notes: release,
        })
    }
}
