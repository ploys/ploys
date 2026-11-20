/// The `Git` commit parameters.
pub struct CommitParams {
    message: String,
    sign: Option<bool>,
}

impl CommitParams {
    /// Constructs new commit parameters with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            sign: None,
        }
    }

    /// Overrides the repository's `commit.gpgSign` setting for this commit.
    pub fn signed(mut self, sign: bool) -> Self {
        self.sign = Some(sign);
        self
    }
}

impl CommitParams {
    /// Gets the commit message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the signing override, or `None` to use `commit.gpgSign`.
    pub fn sign(&self) -> Option<bool> {
        self.sign
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
