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

impl<T> Remote for &T
where
    T: Remote,
{
    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), Self::Error> {
        (**self).request_package_release(package, version)
    }

    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Self::Error> {
        (**self).get_changelog_release(package, version, is_primary)
    }

    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Self::Error> {
        (**self).create_pull_request(head, base, title, body)
    }

    fn create_release(
        &self,
        tag: &str,
        sha: &str,
        name: &str,
        body: &str,
        prerelease: bool,
        latest: bool,
    ) -> Result<u64, Self::Error> {
        (**self).create_release(tag, sha, name, body, prerelease, latest)
    }
}

impl<T> Remote for &mut T
where
    T: Remote,
{
    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), Self::Error> {
        (**self).request_package_release(package, version)
    }

    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Self::Error> {
        (**self).get_changelog_release(package, version, is_primary)
    }

    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Self::Error> {
        (**self).create_pull_request(head, base, title, body)
    }

    fn create_release(
        &self,
        tag: &str,
        sha: &str,
        name: &str,
        body: &str,
        prerelease: bool,
        latest: bool,
    ) -> Result<u64, Self::Error> {
        (**self).create_release(tag, sha, name, body, prerelease, latest)
    }
}
