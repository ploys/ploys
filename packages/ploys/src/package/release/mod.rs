mod request;

pub use self::request::{ReleaseRequest, ReleaseRequestBuilder};

use super::Package;

/// The package release.
pub struct Release<'a> {
    #[allow(dead_code)]
    package: Package<'a>,
    id: u64,
    name: String,
    notes: crate::changelog::Release,
}

impl Release<'_> {
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
    package: Package<'a>,
}

impl<'a> ReleaseBuilder<'a> {
    /// Constructs a new release builder.
    pub(crate) fn new(package: Package<'a>) -> Self {
        Self { package }
    }

    /// Finishes the release.
    pub fn finish(self) -> Result<Release<'a>, crate::project::Error> {
        let Some(remote) = self.package.project.get_remote() else {
            return Err(crate::project::Error::Unsupported);
        };

        let sha = remote.sha()?;

        let version = self.package.version();
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

        let release = self
            .package
            .changelog()
            .and_then(|changelog| changelog.get_release(version.to_string()));

        let release = match release {
            Some(release) => release.to_owned(),
            None => self
                .package
                .project
                .get_changelog_release(self.package.name(), &version)?,
        };

        let body = format!("{release:#}")
            .lines()
            .skip(2)
            .collect::<Vec<_>>()
            .join("\n");

        let id = remote.create_release(&tag, &sha, &name, &body, prerelease, latest)?;

        Ok(Release {
            package: self.package,
            id,
            name,
            notes: release,
        })
    }
}
