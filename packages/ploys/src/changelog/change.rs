use std::fmt::{self, Display};

use markdown::mdast::Node;

use super::Text;

/// A single change in a changelog.
#[derive(Clone, Debug)]
pub struct ChangeRef<'a> {
    text: Text<'a>,
}

impl<'a> ChangeRef<'a> {
    /// Gets the change message.
    pub fn message(&self) -> String {
        format!("{:#}", self.text)
    }

    /// Gets the url if set.
    pub fn url(&self) -> Option<&str> {
        self.text.nodes.iter().rev().find_map(|node| match node {
            Node::Link(link) => Some(&*link.url),
            _ => None,
        })
    }
}

impl<'a> ChangeRef<'a> {
    /// Constructs the change reference from a slice of nodes.
    pub(super) fn from_nodes(nodes: &'a [Node]) -> Option<Self> {
        let Node::Paragraph(paragraph) = nodes.first()? else {
            return None;
        };

        Some(Self {
            text: Text::from_nodes(&paragraph.children),
        })
    }
}

impl<'a> Display for ChangeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "- {:#}", self.text)
        } else {
            write!(f, "- {}", self.text)
        }
    }
}
