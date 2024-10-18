use std::fmt::{self, Display};

use markdown::mdast::Node;
use markdown::ParseOptions;

use super::{Change, ChangeRef, MultilineText};

/// A changelog changeset.
#[derive(Clone, Debug)]
pub struct Changeset {
    label: String,
    description: Option<String>,
    changes: Vec<Change>,
}

impl Changeset {
    /// Constructs a new changeset.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            changes: Vec::new(),
        }
    }

    /// Constructs a new `Added` changeset.
    pub fn added() -> Self {
        Self::new("Added")
    }

    /// Constructs a new `Changed` changeset.
    pub fn changed() -> Self {
        Self::new("Changed")
    }

    /// Constructs a new `Deprecated` changeset.
    pub fn deprecated() -> Self {
        Self::new("Deprecated")
    }

    /// Constructs a new `Removed` changeset.
    pub fn removed() -> Self {
        Self::new("Removed")
    }

    /// Constructs a new `Fixed` changeset.
    pub fn fixed() -> Self {
        Self::new("Fixed")
    }

    /// Constructs a new `Security` changeset.
    pub fn security() -> Self {
        Self::new("Security")
    }

    /// Gets the changeset label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Gets the changeset description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the changeset description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Builds the changeset with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    /// Adds a change to the changeset.
    pub fn add_change(&mut self, change: impl Into<Change>) -> &mut Self {
        self.changes.push(change.into());
        self
    }

    /// Builds the changeset with the given change.
    pub fn with_change(mut self, change: impl Into<Change>) -> Self {
        self.add_change(change);
        self
    }

    /// Gets an iterator over the changes.
    pub fn changes(&self) -> impl Iterator<Item = &Change> {
        self.changes.iter()
    }
}

impl Changeset {
    /// Converts the changeset into markdown nodes.
    pub(super) fn into_nodes(self) -> Vec<Node> {
        let mut nodes = Vec::new();

        let heading = Node::Heading(markdown::mdast::Heading {
            children: vec![Node::Text(markdown::mdast::Text {
                value: self.label,
                position: None,
            })],
            position: None,
            depth: 3,
        });

        nodes.push(heading);

        if let Some(description) = self.description {
            let md = markdown::to_mdast(&description, &ParseOptions::default()).expect("markdown");

            if let Node::Root(root) = md {
                nodes.extend(root.children);
            }
        }

        if !self.changes.is_empty() {
            let list = Node::List(markdown::mdast::List {
                children: self
                    .changes
                    .into_iter()
                    .map(|change| {
                        Node::ListItem(markdown::mdast::ListItem {
                            children: change.into_nodes(),
                            position: None,
                            spread: false,
                            checked: None,
                        })
                    })
                    .collect(),
                position: None,
                ordered: false,
                start: None,
                spread: false,
            });

            nodes.push(list);
        }

        nodes
    }
}

impl Display for Changeset {
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::changelog::Change;

    use super::Changeset;

    #[test]
    fn test_changeset() {
        let changeset = Changeset::fixed()
            .with_description("Fixed some things.")
            .with_change(
                Change::new("Fixed `one`")
                    .with_url("#1", "https://github.com/ploys/example/pull/1"),
            )
            .with_change(
                Change::new("Fixed `two`")
                    .with_url("#2", "https://github.com/ploys/example/pull/2"),
            );

        let output = indoc! {"
            ### Fixed

            Fixed some things.

            - Fixed `one` ([#1](https://github.com/ploys/example/pull/1))
            - Fixed `two` ([#2](https://github.com/ploys/example/pull/2))\
        "};

        assert_eq!(changeset.description(), Some("Fixed some things."));
        assert_eq!(changeset.changes().count(), 2);
        assert_eq!(changeset.to_string(), output);
    }
}
