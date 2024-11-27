use std::path::PathBuf;

use semver::Version;

use crate::changelog::Release;
use crate::package::BumpOrVersion;

use super::Error;

/// A remote repository.
///
/// This defines the shared API of a remote repository to simplify feature flag
/// handling.
pub trait Remote {
    /// Commits the changes to the repository.
    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, Error>;

    /// Requests a package release.
    fn request_package_release(&self, package: &str, version: BumpOrVersion) -> Result<(), Error>;

    /// Gets the changelog release for the given package version.
    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Error>;

    /// Gets the default branch.
    fn get_default_branch(&self) -> Result<String, Error>;

    /// Creates a new branch.
    fn create_branch(&self, name: &str) -> Result<(), Error>;

    /// Updates the branch to point to the given SHA.
    fn update_branch(&self, name: &str, sha: &str) -> Result<(), Error>;

    /// Creates a pull request.
    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Error>;
}
