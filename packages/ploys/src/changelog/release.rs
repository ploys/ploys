use std::fmt::{self, Display};

use markdown::mdast::Node;

use super::{ChangesetRef, MultilineText, ReferenceRef};

/// A changelog release entry.
#[derive(Clone, Debug)]
pub struct ReleaseRef<'a> {
    version: &'a str,
    date: Option<&'a str>,
    nodes: &'a [Node],
}

impl<'a> ReleaseRef<'a> {
    /// Gets the release version.
    pub fn version(&self) -> &str {
        self.version
    }

    /// Gets the release date.
    pub fn date(&self) -> Option<&str> {
        self.date
    }

    /// Gets the release description.
    pub fn description(&self) -> Option<MultilineText<'_>> {
        MultilineText::from_nodes(self.nodes)
    }

    /// Gets the `Added` changeset.
    pub fn added(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Added")
    }

    /// Gets the `Changed` changeset.
    pub fn changed(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Changed")
    }

    /// Gets the `Deprecated` changeset.
    pub fn deprecated(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Deprecated")
    }

    /// Gets the `Removed` changeset.
    pub fn removed(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Removed")
    }

    /// Gets the `Fixed` changeset.
    pub fn fixed(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Fixed")
    }

    /// Gets the `Security` changeset.
    pub fn security(&self) -> Option<ChangesetRef<'a>> {
        self.get_changeset("Security")
    }

    /// Gets the changeset for the given label.
    pub fn get_changeset(&self, label: impl AsRef<str>) -> Option<ChangesetRef<'a>> {
        self.changesets()
            .find(|changeset| changeset.label() == label.as_ref())
    }

    /// Gets an iterator over the changesets.
    pub fn changesets(&self) -> impl Iterator<Item = ChangesetRef<'a>> {
        self.get_sections().filter_map(ChangesetRef::from_nodes)
    }

    /// Gets an iterator over the references.
    pub fn references(&self) -> impl Iterator<Item = ReferenceRef<'a>> {
        self.nodes
            .chunk_by(|node, _| matches!(node, Node::Definition(_)))
            .last()
            .into_iter()
            .flat_map(|nodes| {
                nodes.iter().filter_map(|node| match node {
                    Node::Definition(definition) => Some(ReferenceRef::from_definition(definition)),
                    _ => None,
                })
            })
    }
}

impl<'a> ReleaseRef<'a> {
    /// Constructs the release reference from a slice of nodes.
    pub(super) fn from_nodes(nodes: &'a [Node]) -> Option<Self> {
        let Node::Heading(heading) = nodes.first()? else {
            return None;
        };

        if heading.depth != 2 {
            return None;
        }

        let version = heading.children.iter().find_map(|node| match node {
            Node::LinkReference(link) => Some(&link.identifier),
            _ => None,
        })?;

        let date = heading.children.iter().find_map(|node| match node {
            Node::Text(text) => Some(text.value.trim().trim_start_matches('-').trim()),
            _ => None,
        });

        Some(Self {
            version,
            date,
            nodes: &nodes[1..],
        })
    }

    /// Gets the sections separated by a third-level heading.
    fn get_sections(&self) -> impl Iterator<Item = &'a [Node]> {
        self.nodes
            .chunk_by(|_, node| !matches!(node, Node::Heading(heading) if heading.depth == 3))
    }
}

impl<'a> Display for ReleaseRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.date {
            Some(date) => write!(f, "## [{}] - {date}", self.version)?,
            None => write!(f, "## [{}]", self.version)?,
        }

        if let Some(description) = self.description() {
            if f.alternate() {
                write!(f, "\n\n{description:#}")?;
            } else {
                write!(f, "\n\n{description}")?;
            }
        }

        let mut changesets = self.changesets().peekable();

        if changesets.peek().is_some() {
            write!(f, "\n\n")?;

            while let Some(changeset) = changesets.next() {
                if f.alternate() {
                    write!(f, "{changeset:#}")?;
                } else {
                    write!(f, "{changeset}")?;
                }

                if changesets.peek().is_some() {
                    write!(f, "\n\n")?;
                }
            }
        }

        let mut references = self.references().peekable();

        if references.peek().is_some() {
            write!(f, "\n\n")?;

            while let Some(reference) = references.next() {
                if f.alternate() {
                    write!(f, "{reference:#}")?;
                } else {
                    write!(f, "{reference}")?;
                }

                if references.peek().is_some() {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}
