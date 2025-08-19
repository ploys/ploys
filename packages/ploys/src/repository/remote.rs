use semver::Version;

use crate::changelog::Release;
use crate::package::BumpOrVersion;

use super::GitLike;

/// A remote repository.
///
/// This defines the shared API of a remote repository to simplify feature flag
/// handling.
pub trait Remote: GitLike {
    /// Requests a package release.
    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), Self::Error>;

    /// Gets the changelog release for the given package version.
    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Self::Error>;

    /// Creates a pull request.
    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Self::Error>;

    /// Creates a release.
    fn create_release(
        &self,
        tag: &str,
        sha: &str,
        name: &str,
        body: &str,
        prerelease: bool,
        latest: bool,
    ) -> Result<u64, Self::Error>;
}
