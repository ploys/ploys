use std::fmt::{self, Display};

use markdown::mdast::Node;

use super::{ChangeRef, MultilineText};

/// A changelog release labelled changeset.
#[derive(Clone, Debug)]
pub struct ChangesetRef<'a> {
    label: &'a str,
    nodes: &'a [Node],
}

impl<'a> ChangesetRef<'a> {
    /// Gets the changeset label.
    pub fn label(&self) -> &str {
        self.label
    }

    /// Gets the changeset description.
    pub fn description(&self) -> Option<MultilineText<'_>> {
        MultilineText::from_nodes(self.nodes)
    }

    /// Gets an iterator over the changes.
    pub fn changes(&self) -> impl Iterator<Item = ChangeRef<'a>> {
        self.nodes
            .iter()
            .filter_map(|node| match node {
                Node::List(list) => Some(list),
                _ => None,
            })
            .flat_map(|list| {
                list.children.iter().filter_map(|node| match node {
                    Node::ListItem(item) => ChangeRef::from_nodes(&item.children),
                    _ => None,
                })
            })
    }
}

impl<'a> ChangesetRef<'a> {
    /// Constructs the changeset reference from a slice of nodes.
    pub(super) fn from_nodes(nodes: &'a [Node]) -> Option<Self> {
        let Node::Heading(heading) = nodes.first()? else {
            return None;
        };

        if heading.depth != 3 {
            return None;
        }

        let label = heading.children.iter().find_map(|node| match node {
            Node::Text(text) => Some(&*text.value),
            _ => None,
        })?;

        Some(Self {
            label,
            nodes: &nodes[1..],
        })
    }
}

impl<'a> Display for ChangesetRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "### {}", self.label())?;

        if let Some(description) = self.description() {
            if f.alternate() {
                write!(f, "\n\n{description:#}")?;
            } else {
                write!(f, "\n\n{description}")?;
            }
        }

        let mut changes = self.changes().peekable();

        if changes.peek().is_some() {
            write!(f, "\n\n")?;

            while let Some(change) = changes.next() {
                if f.alternate() {
                    write!(f, "{change:#}")?;
                } else {
                    write!(f, "{change}")?;
                }

                if changes.peek().is_some() {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}
