#[derive(Clone)]
pub struct WebhookSecret {
    pub value: String,
}

impl WebhookSecret {
    /// Constructs a new webhook secret.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}
