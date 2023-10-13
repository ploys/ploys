use std::fmt::{self, Display};
use std::io;

/// The Git source error.
#[derive(Debug)]
pub enum Error {
    /// A Git error.
    Git(GitError),
    /// An I/O error.
    Io(io::Error),
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
            Self::Git(err) => Display::fmt(err, f),
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<gix::open::Error> for Error {
    fn from(err: gix::open::Error) -> Self {
        Self::Git(err.into())
    }
}

impl From<gix::remote::find::existing::Error> for Error {
    fn from(err: gix::remote::find::existing::Error) -> Self {
        Self::Git(err.into())
    }
}

impl From<gix::revision::spec::parse::single::Error> for Error {
    fn from(err: gix::revision::spec::parse::single::Error) -> Self {
        Self::Git(err.into())
    }
}

impl From<gix::odb::find::existing::Error<gix::odb::store::find::Error>> for Error {
    fn from(err: gix::odb::find::existing::Error<gix::odb::store::find::Error>) -> Self {
        Self::Git(err.into())
    }
}

impl From<gix::object::peel::to_kind::Error> for Error {
    fn from(err: gix::object::peel::to_kind::Error) -> Self {
        Self::Git(err.into())
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for Error {
    fn from(err: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Git(err.into())
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
    fn from(err: gix::open::Error) -> Self {
        Self::Open(Box::new(err))
    }
}

impl From<gix::remote::find::existing::Error> for GitError {
    fn from(err: gix::remote::find::existing::Error) -> Self {
        Self::Remote(Box::new(err))
    }
}

impl From<gix::revision::spec::parse::single::Error> for GitError {
    fn from(err: gix::revision::spec::parse::single::Error) -> Self {
        Self::Revision(Box::new(err))
    }
}

impl From<gix::odb::find::existing::Error<gix::odb::store::find::Error>> for GitError {
    fn from(err: gix::odb::find::existing::Error<gix::odb::store::find::Error>) -> Self {
        Self::ObjectFind(Box::new(err))
    }
}

impl From<gix::object::peel::to_kind::Error> for GitError {
    fn from(err: gix::object::peel::to_kind::Error) -> Self {
        Self::ObjectKind(Box::new(err))
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for GitError {
    fn from(err: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Traverse(Box::new(err))
    }
}
