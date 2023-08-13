use std::fmt::{self, Display};

/// The remote project error.
#[derive(Debug)]
pub enum Error {
    /// An HTTP status response.
    Response(u16),
    /// A transport error.
    Transport(Box<ureq::Transport>),
    /// A parse error.
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Response(status_code) => match status_code {
                401 => write!(f, "401 Unauthorized"),
                403 => write!(f, "403 Forbidden"),
                404 => write!(f, "404 Not Found"),
                429 => write!(f, "429 Too Many Requests"),
                status_code => write!(f, "Response error: {status_code}"),
            },
            Error::Transport(transport) => Display::fmt(transport, f),
            Error::Parse(message) => write!(f, "Parse error: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Status(status_code, _) => Self::Response(status_code),
            ureq::Error::Transport(transport) => Self::Transport(Box::new(transport)),
        }
    }
}
