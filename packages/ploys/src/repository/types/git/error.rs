use std::fmt::{self, Display};
use std::io;

/// The Git repository error.
#[derive(Debug)]
pub enum Error {
    /// An invalid path error.
    Path(crate::repository::path::Error),
    /// A `gix` error.
    Gix(GixError),
    /// An I/O error.
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(err) => Display::fmt(err, f),
            Self::Gix(err) => Display::fmt(err, f),
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::repository::path::Error> for Error {
    fn from(err: crate::repository::path::Error) -> Self {
        Self::Path(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<gix::open::Error> for Error {
    fn from(err: gix::open::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::init::Error> for Error {
    fn from(err: gix::init::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::remote::find::existing::Error> for Error {
    fn from(err: gix::remote::find::existing::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::revision::spec::parse::single::Error> for Error {
    fn from(err: gix::revision::spec::parse::single::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::object::find::existing::Error> for Error {
    fn from(err: gix::object::find::existing::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::object::peel::to_kind::Error> for Error {
    fn from(err: gix::object::peel::to_kind::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for Error {
    fn from(err: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::object::tree::editor::init::Error> for Error {
    fn from(err: gix::object::tree::editor::init::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::object::tree::editor::write::Error> for Error {
    fn from(err: gix::object::tree::editor::write::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::objs::tree::editor::Error> for Error {
    fn from(err: gix::objs::tree::editor::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::object::write::Error> for Error {
    fn from(err: gix::object::write::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::commit::Error> for Error {
    fn from(err: gix::commit::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::reference::find::existing::Error> for Error {
    fn from(err: gix::reference::find::existing::Error) -> Self {
        Self::Gix(err.into())
    }
}

impl From<gix::repository::edit_tree::Error> for Error {
    fn from(err: gix::repository::edit_tree::Error) -> Self {
        Self::Gix(err.into())
    }
}

/// A Git error.
#[derive(Debug)]
pub enum GixError {
    /// An open error.
    Open(Box<gix::open::Error>),
    /// An init error.
    Init(Box<gix::init::Error>),
    /// A remote lookup error.
    Remote(Box<gix::remote::find::existing::Error>),
    /// A revision parse error.
    Revision(Box<gix::revision::spec::parse::single::Error>),
    /// An object find error.
    ObjectFind(Box<gix::object::find::existing::Error>),
    /// An object kind error.
    ObjectKind(Box<gix::object::peel::to_kind::Error>),
    /// A tree traversal error.
    Traverse(Box<gix::traverse::tree::breadthfirst::Error>),
    /// A tree editor error.
    TreeEditor(Box<gix::objs::tree::editor::Error>),
    /// A tree editor initialization error.
    TreeEditorInit(Box<gix::object::tree::editor::init::Error>),
    /// A tree editor write error.
    TreeEditorWrite(Box<gix::object::tree::editor::write::Error>),
    /// An object write error.
    ObjectWrite(Box<gix::object::write::Error>),
    /// A commit error.
    Commit(Box<gix::commit::Error>),
    /// A find reference error.
    FindReference(Box<gix::reference::find::existing::Error>),
    /// An edit tree error.
    EditTree(Box<gix::repository::edit_tree::Error>),
}

impl Display for GixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(err) => Display::fmt(err, f),
            Self::Init(err) => Display::fmt(err, f),
            Self::Remote(err) => Display::fmt(err, f),
            Self::Revision(err) => Display::fmt(err, f),
            Self::ObjectFind(err) => Display::fmt(err, f),
            Self::ObjectKind(err) => Display::fmt(err, f),
            Self::Traverse(err) => Display::fmt(err, f),
            Self::TreeEditor(err) => Display::fmt(err, f),
            Self::TreeEditorInit(err) => Display::fmt(err, f),
            Self::TreeEditorWrite(err) => Display::fmt(err, f),
            Self::ObjectWrite(err) => Display::fmt(err, f),
            Self::Commit(err) => Display::fmt(err, f),
            Self::FindReference(err) => Display::fmt(err, f),
            Self::EditTree(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for GixError {}

impl From<gix::open::Error> for GixError {
    fn from(err: gix::open::Error) -> Self {
        Self::Open(Box::new(err))
    }
}

impl From<gix::init::Error> for GixError {
    fn from(err: gix::init::Error) -> Self {
        Self::Init(Box::new(err))
    }
}

impl From<gix::remote::find::existing::Error> for GixError {
    fn from(err: gix::remote::find::existing::Error) -> Self {
        Self::Remote(Box::new(err))
    }
}

impl From<gix::revision::spec::parse::single::Error> for GixError {
    fn from(err: gix::revision::spec::parse::single::Error) -> Self {
        Self::Revision(Box::new(err))
    }
}

impl From<gix::object::find::existing::Error> for GixError {
    fn from(err: gix::object::find::existing::Error) -> Self {
        Self::ObjectFind(Box::new(err))
    }
}

impl From<gix::object::peel::to_kind::Error> for GixError {
    fn from(err: gix::object::peel::to_kind::Error) -> Self {
        Self::ObjectKind(Box::new(err))
    }
}

impl From<gix::traverse::tree::breadthfirst::Error> for GixError {
    fn from(err: gix::traverse::tree::breadthfirst::Error) -> Self {
        Self::Traverse(Box::new(err))
    }
}

impl From<gix::object::tree::editor::init::Error> for GixError {
    fn from(err: gix::object::tree::editor::init::Error) -> Self {
        Self::TreeEditorInit(Box::new(err))
    }
}

impl From<gix::object::tree::editor::write::Error> for GixError {
    fn from(err: gix::object::tree::editor::write::Error) -> Self {
        Self::TreeEditorWrite(Box::new(err))
    }
}

impl From<gix::objs::tree::editor::Error> for GixError {
    fn from(err: gix::objs::tree::editor::Error) -> Self {
        Self::TreeEditor(Box::new(err))
    }
}

impl From<gix::object::write::Error> for GixError {
    fn from(err: gix::object::write::Error) -> Self {
        Self::ObjectWrite(Box::new(err))
    }
}

impl From<gix::commit::Error> for GixError {
    fn from(err: gix::commit::Error) -> Self {
        Self::Commit(Box::new(err))
    }
}

impl From<gix::reference::find::existing::Error> for GixError {
    fn from(err: gix::reference::find::existing::Error) -> Self {
        Self::FindReference(Box::new(err))
    }
}

impl From<gix::repository::edit_tree::Error> for GixError {
    fn from(err: gix::repository::edit_tree::Error) -> Self {
        Self::EditTree(Box::new(err))
    }
}
