use std::path::PathBuf;

use semver::Version;

use crate::changelog::Release;
use crate::package::BumpOrVersion;

use super::revision::Revision;
use super::Error;

/// A remote repository.
///
/// This defines the shared API of a remote repository to simplify feature flag
/// handling.
pub trait Remote {
    /// Gets the revision.
    fn revision(&self) -> &Revision;

    /// Sets the revision.
    fn set_revision(&mut self, revision: Revision);

    /// Commits the changes to the repository.
    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, Error>;

    /// Requests a package release.
    fn request_package_release(&self, package: &str, version: BumpOrVersion) -> Result<(), Error>;

    /// Gets the changelog release for the given package version.
    fn get_changelog_release(
        &self,
        package: &str,
        version: Version,
        is_primary: bool,
    ) -> Result<Release, Error>;
}
