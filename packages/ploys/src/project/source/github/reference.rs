use std::fmt::{self, Display};

/// The git reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Head,
    Sha(String),
    Branch(String),
    Tag(String),
}

impl Reference {
    /// Constructs a new head reference.
    pub fn head() -> Self {
        Self::Head
    }

    /// Constructs a new SHA reference.
    pub fn sha(sha: impl Into<String>) -> Self {
        Self::Sha(sha.into())
    }

    /// Constructs a new branch reference.
    pub fn branch(branch: impl Into<String>) -> Self {
        Self::Branch(branch.into())
    }

    /// Constructs a new tag reference.
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::Tag(tag.into())
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Head => write!(f, "HEAD"),
            Self::Sha(sha) => Display::fmt(sha, f),
            Self::Branch(branch) => Display::fmt(branch, f),
            Self::Tag(tag) => Display::fmt(tag, f),
        }
    }
}
