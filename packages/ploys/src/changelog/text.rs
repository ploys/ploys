use std::fmt::{self, Display};

use markdown::mdast::Node;

/// A changelog text section.
#[derive(Clone, Debug)]
pub struct Text<'a> {
    pub(super) nodes: &'a [Node],
}

impl<'a> Text<'a> {
    /// Constructs the text from a slice of nodes.
    pub(super) fn from_nodes(nodes: &'a [Node]) -> Self {
        Self { nodes }
    }
}

impl Display for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.nodes {
            write_node(f, node)?;
        }

        Ok(())
    }
}

/// A changelog multiline text section.
#[derive(Clone, Debug)]
pub struct MultilineText<'a> {
    nodes: &'a [Node],
}

impl<'a> MultilineText<'a> {
    /// Constructs the multiline text from a slice of nodes.
    pub(super) fn from_nodes(nodes: &'a [Node]) -> Option<Self> {
        nodes
            .split(|node| matches!(node, Node::Heading(_) | Node::List(_) | Node::Definition(_)))
            .next()
            .and_then(|nodes| match nodes.is_empty() {
                true => None,
                false => Some(Self { nodes }),
            })
    }
}

impl Display for MultilineText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nodes = self.nodes.iter().peekable();

        while let Some(node) = nodes.next() {
            if write_node(f, node)? && nodes.peek().is_some() {
                write!(f, "\n\n")?;
            }
        }

        Ok(())
    }
}

/// Writes a markdown node.
///
/// Unfortunately, the `ToString` implementation does not render markdown and
/// markdown generation from an AST is not yet supported by the `markdown`
/// crate. This is a temporary solution until a better implementation can be
/// found.
pub(super) fn write_node(f: &mut fmt::Formatter<'_>, node: &Node) -> Result<bool, fmt::Error> {
    let mut printed = false;

    match node {
        Node::Heading(heading) => {
            write!(f, "{:#<width$} ", "", width = heading.depth as usize)?;

            for node in &heading.children {
                printed |= write_node(f, node)?;
            }

            Ok(printed)
        }
        Node::Paragraph(paragraph) => {
            for node in &paragraph.children {
                printed |= write_node(f, node)?;
            }

            Ok(printed)
        }
        Node::Text(text) => {
            write!(f, "{}", text.value)?;

            Ok(true)
        }
        Node::Link(link) => match f.alternate() {
            true => {
                for node in &link.children {
                    printed |= write_node(f, node)?;
                }

                Ok(printed)
            }
            false => {
                write!(f, "[")?;

                for node in &link.children {
                    printed |= write_node(f, node)?;
                }

                write!(f, "]({})", link.url)?;

                Ok(printed)
            }
        },
        Node::InlineCode(code) => {
            write!(f, "`{}`", code.value)?;

            Ok(true)
        }
        _ => Ok(printed),
    }
}
