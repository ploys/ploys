use std::fmt::{self, Display};
use std::io;

/// The local project error.
#[derive(Debug)]
pub enum Error {
    /// A Git error.
    Git(GitError),
    /// An I/O error.
    Io(io::Error),
    /// A package error.
    Package(crate::package::Error),
}

impl Error {
    /// Creates a remote not found error.
    pub(super) fn remote_not_found() -> Self {
        Self::Git(GitError::Remote(Box::new(
            gix::remote::find::existing::Error::NotFound {
                name: String::from("origin").into(),
            },
        )))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(error) => Display::fmt(error, f),
            Self::Io(error) => Display::fmt(error, f),
            Self::Package(error) => Display::fmt(error, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::package::Error> for Error {
    fn from(error: crate::package::Error) -> Self {
        Self::Package(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<gix::open::Error> for Error {
    fn from(error: gix::open::Error) -> Self {
        Self::Git(error.into())
    }
}

impl From<gix::remote::find::existing::Error> for Error {
    fn from(error: gix::remote::find::existing::Error) -> Self {
        Self::Git(error.into())
    }
}

impl From<gix::revision::spec::parse::single::Error> for Error {
    fn from(error: gix::revision::spec::parse::single::Error) -> Self {
        Self::Git(error.into())
    }
}

impl From<gix::odb::find::existing::Error<gix::odb::store::find::Error>> for Error {
    fn from(error: gix::odb::find::existing::Error<gix::odb::store::find::Error>) -> Self {
        Self::Git(error.into())
    }
}

impl From<gix::object::peel::to_kind::Error> for Error {
    fn from(error: gix::object::peel::to_kind::Error) -> Self {
        Self::Git(error.into())
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for Error {
    fn from(error: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Git(error.into())
    }
}

/// A Git error.
#[derive(Debug)]
pub enum GitError {
    /// An open error.
    Open(Box<gix::open::Error>),
    /// A remote lookup error.
    Remote(Box<gix::remote::find::existing::Error>),
    /// A revision parse error.
    Revision(Box<gix::revision::spec::parse::single::Error>),
    /// An object find error.
    ObjectFind(Box<gix::odb::find::existing::Error<gix::odb::store::find::Error>>),
    /// An object kind error.
    ObjectKind(Box<gix::object::peel::to_kind::Error>),
    /// A tree traversal error.
    Traverse(Box<gix::traverse::tree::breadthfirst::Error>),
}

impl Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(err) => Display::fmt(err, f),
            Self::Remote(err) => Display::fmt(err, f),
            Self::Revision(err) => Display::fmt(err, f),
            Self::ObjectFind(err) => Display::fmt(err, f),
            Self::ObjectKind(err) => Display::fmt(err, f),
            Self::Traverse(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for GitError {}

impl From<gix::open::Error> for GitError {
    fn from(error: gix::open::Error) -> Self {
        Self::Open(Box::new(error))
    }
}

impl From<gix::remote::find::existing::Error> for GitError {
    fn from(error: gix::remote::find::existing::Error) -> Self {
        Self::Remote(Box::new(error))
    }
}

impl From<gix::revision::spec::parse::single::Error> for GitError {
    fn from(error: gix::revision::spec::parse::single::Error) -> Self {
        Self::Revision(Box::new(error))
    }
}

impl From<gix::odb::find::existing::Error<gix::odb::store::find::Error>> for GitError {
    fn from(error: gix::odb::find::existing::Error<gix::odb::store::find::Error>) -> Self {
        Self::ObjectFind(Box::new(error))
    }
}

impl From<gix::object::peel::to_kind::Error> for GitError {
    fn from(error: gix::object::peel::to_kind::Error) -> Self {
        Self::ObjectKind(Box::new(error))
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for GitError {
    fn from(error: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Traverse(Box::new(error))
    }
}
