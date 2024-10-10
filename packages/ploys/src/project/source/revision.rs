use std::fmt::{self, Display};

/// The git revision.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Revision {
    #[default]
    Head,
    Sha(String),
    Reference(Reference),
}

impl Revision {
    /// Constructs a new HEAD revision.
    pub fn head() -> Self {
        Self::Head
    }

    /// Constructs a new SHA revision.
    pub fn sha(sha: impl Into<String>) -> Self {
        Self::Sha(sha.into())
    }

    /// Constructs a new branch revision.
    pub fn branch(branch: impl Into<String>) -> Self {
        Self::Reference(Reference::Branch(branch.into()))
    }

    /// Constructs a new tag revision.
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::Reference(Reference::Tag(tag.into()))
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Head => write!(f, "HEAD"),
            Self::Sha(sha) => write!(f, "{sha}"),
            Self::Reference(reference) => write!(f, "refs/{reference}"),
        }
    }
}

impl From<Reference> for Revision {
    fn from(reference: Reference) -> Self {
        Self::Reference(reference)
    }
}

/// The git reference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Reference {
    Branch(String),
    Tag(String),
}

impl Reference {
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
            Self::Branch(branch) => write!(f, "heads/{branch}"),
            Self::Tag(tag) => write!(f, "tags/{tag}"),
        }
    }
}
