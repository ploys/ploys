use std::path::PathBuf;

use super::Repository;

/// Defines a git-like version control system.
pub trait GitLike: Repository {
    /// Gets the commit SHA.
    fn sha(&self) -> Result<String, Self::Error>;

    /// Commits the changes to the repository.
    fn commit(&self, message: &str, files: Vec<(PathBuf, String)>) -> Result<String, Self::Error>;

    /// Gets the default branch.
    fn get_default_branch(&self) -> Result<String, Self::Error>;

    /// Creates a new branch.
    fn create_branch(&self, name: &str) -> Result<(), Self::Error>;

    /// Updates the branch to point to the given SHA.
    fn update_branch(&self, name: &str, sha: &str) -> Result<(), Self::Error>;
}
