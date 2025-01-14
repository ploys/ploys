use std::path::PathBuf;

use semver::Version;

use crate::changelog::Release;
use crate::package::BumpOrVersion;

use super::Repository;

/// A remote repository.
///
/// This defines the shared API of a remote repository to simplify feature flag
/// handling.
pub trait Remote: Repository {
    /// Gets the commit SHA.
    fn sha(&self) -> Result<String, Self::Error>;

    /// Commits the changes to the repository.
    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, Self::Error>;

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

    /// Gets the default branch.
    fn get_default_branch(&self) -> Result<String, Self::Error>;

    /// Creates a new branch.
    fn create_branch(&self, name: &str) -> Result<(), Self::Error>;

    /// Updates the branch to point to the given SHA.
    fn update_branch(&self, name: &str, sha: &str) -> Result<(), Self::Error>;

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
    fn sha(&self) -> Result<String, Self::Error> {
        (*self).sha()
    }

    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, Self::Error> {
        (*self).commit(message, files)
    }

    fn request_package_release(
        &self,
        package: &str,
        version: BumpOrVersion,
    ) -> Result<(), Self::Error> {
        (*self).request_package_release(package, version)
    }

    fn get_changelog_release(
        &self,
        package: &str,
        version: &Version,
        is_primary: bool,
    ) -> Result<Release, Self::Error> {
        (*self).get_changelog_release(package, version, is_primary)
    }

    fn get_default_branch(&self) -> Result<String, Self::Error> {
        (*self).get_default_branch()
    }

    fn create_branch(&self, name: &str) -> Result<(), Self::Error> {
        (*self).create_branch(name)
    }

    fn update_branch(&self, name: &str, sha: &str) -> Result<(), Self::Error> {
        (*self).update_branch(name, sha)
    }

    fn create_pull_request(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<u64, Self::Error> {
        (*self).create_pull_request(head, base, title, body)
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
        (*self).create_release(tag, sha, name, body, prerelease, latest)
    }
}
