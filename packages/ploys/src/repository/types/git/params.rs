/// The `Git` commit parameters.
pub struct CommitParams {
    message: String,
}

impl CommitParams {
    /// Constructs new commit parameters with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Gets the commit message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<&str> for CommitParams {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<String> for CommitParams {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}
