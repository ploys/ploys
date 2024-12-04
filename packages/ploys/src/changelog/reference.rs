use std::fmt::{self, Display};

use markdown::mdast::Definition;

/// A changelog reference.
pub struct ReferenceRef<'a> {
    definition: &'a Definition,
}

impl ReferenceRef<'_> {
    /// Gets the reference ID.
    pub fn id(&self) -> &str {
        &self.definition.identifier
    }

    /// Gets the reference URL.
    pub fn url(&self) -> &str {
        &self.definition.url
    }

    /// Creates an owned reference.
    pub fn to_owned(&self) -> (String, String) {
        (self.id().to_owned(), self.url().to_owned())
    }
}

impl<'a> ReferenceRef<'a> {
    /// Constructs the release reference from a definition.
    pub(super) fn from_definition(definition: &'a Definition) -> Self {
        Self { definition }
    }
}

impl Display for ReferenceRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]: {}",
            self.definition.identifier, self.definition.url
        )
    }
}
