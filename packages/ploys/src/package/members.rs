use std::path::{Path, PathBuf};

use globset::GlobSet;

/// The package members.
pub struct Members {
    includes: GlobSet,
    excludes: Vec<PathBuf>,
}

impl Members {
    /// Creates new members with the given includes and excludes.
    pub fn new(includes: GlobSet, excludes: Vec<PathBuf>) -> Self {
        Self { includes, excludes }
    }

    /// Checks whether the members includes the given path.
    pub fn includes(&self, path: &Path) -> bool {
        self.includes.is_match(path) && !self.excludes(path)
    }

    /// Checks whether the members excludes the given path.
    pub fn excludes(&self, path: &Path) -> bool {
        self.excludes
            .iter()
            .any(|exclude| path.starts_with(exclude))
    }

    /// Checks whether the members is empty.
    pub fn is_empty(&self) -> bool {
        self.includes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use globset::{Glob, GlobSetBuilder};

    use super::Members;

    #[test]
    fn test_member_includes() {
        let mut includes = GlobSetBuilder::new();
        let mut excludes = Vec::new();

        includes.add(Glob::new("Cargo.toml").unwrap());
        includes.add(Glob::new("crates/foo").unwrap());
        includes.add(Glob::new("packages/foo").unwrap());
        includes.add(Glob::new("packages/*").unwrap());
        excludes.push(PathBuf::from("packages/baz"));

        let members = Members::new(includes.build().unwrap(), excludes);

        assert!(!members.is_empty());
        assert!(members.includes(Path::new("crates/foo")));
        assert!(members.includes(Path::new("packages/foo")));
        assert!(members.includes(Path::new("packages/bar")));
        assert!(members.excludes(Path::new("packages/baz")));
    }
}
