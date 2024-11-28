use std::fmt::{self, Display};

use markdown::mdast::Node;
use markdown::ParseOptions;

use super::{Changeset, ChangesetRef, MultilineText, ReferenceRef};

/// A changelog release.
#[derive(Clone, Debug)]
pub struct Release {
    version: String,
    date: Option<String>,
    url: Option<String>,
    description: Option<String>,
    changesets: Vec<Changeset>,
    references: Vec<(String, String)>,
}

impl Release {
    /// Constructs a new changelog release.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            date: None,
            url: None,
            description: None,
            changesets: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Gets the release version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Gets the release date.
    pub fn date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    /// Sets the release date.
    pub fn set_date(&mut self, date: impl Into<String>) -> &mut Self {
        self.date = Some(date.into());
        self
    }

    /// Builds the release with the given date.
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.set_date(date);
        self
    }

    /// Gets the release URL.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Sets the release URL.
    pub fn set_url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    /// Builds the release with the given URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.set_url(url);
        self
    }

    /// Gets the release description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the release description.
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Builds the release with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    /// Adds a changeset to the release.
    pub fn add_changeset(&mut self, changeset: impl Into<Changeset>) -> &mut Self {
        self.changesets.push(changeset.into());
        self
    }

    /// Builds the release with the given changeset.
    pub fn with_changeset(mut self, changeset: impl Into<Changeset>) -> Self {
        self.add_changeset(changeset);
        self
    }

    /// Gets an iterator over the changesets.
    pub fn changesets(&self) -> impl Iterator<Item = &Changeset> {
        self.changesets.iter()
    }

    /// Adds a reference to the release.
    pub fn add_reference(&mut self, id: impl Into<String>, url: impl Into<String>) -> &mut Self {
        self.references.push((id.into(), url.into()));
        self
    }

    /// Builds the release with the given reference.
    pub fn with_reference(mut self, id: impl Into<String>, url: impl Into<String>) -> Self {
        self.add_reference(id, url);
        self
    }

    /// Gets an iterator over the references.
    pub fn references(&self) -> impl Iterator<Item = (&str, &str)> {
        self.references.iter().map(|(id, url)| (&**id, &**url))
    }
}

impl Release {
    /// Converts the release into markdown nodes.
    pub(super) fn into_nodes(self) -> Vec<Node> {
        let mut nodes = Vec::new();

        let version = Node::LinkReference(markdown::mdast::LinkReference {
            children: vec![Node::Text(markdown::mdast::Text {
                value: self.version.clone(),
                position: None,
            })],
            position: None,
            reference_kind: markdown::mdast::ReferenceKind::Shortcut,
            identifier: self.version,
            label: None,
        });

        let date = self.date.map(|date| {
            Node::Text(markdown::mdast::Text {
                value: format!(" - {date}"),
                position: None,
            })
        });

        let heading = Node::Heading(markdown::mdast::Heading {
            children: Some(version).into_iter().chain(date).collect(),
            position: None,
            depth: 2,
        });

        nodes.push(heading);

        if let Some(description) = self.description {
            let md = markdown::to_mdast(&description, &ParseOptions::default()).expect("markdown");

            if let Node::Root(root) = md {
                nodes.extend(root.children);
            }
        }

        nodes.extend(self.changesets.into_iter().flat_map(Changeset::into_nodes));
        nodes
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.date {
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

            while let Some((id, url)) = references.next() {
                write!(f, "[{id}]: {url}")?;

                if references.peek().is_some() {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

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

impl Display for ReleaseRef<'_> {
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::changelog::{Change, Changeset};

    use super::Release;

    #[test]
    fn test_release() {
        let release = Release::new("0.1.0")
            .with_date("2024-01-01")
            .with_changeset(
                Changeset::fixed()
                    .with_description("Fixed some things.")
                    .with_change(
                        Change::new("Fixed `one`")
                            .with_url("#1", "https://github.com/ploys/example/pull/1"),
                    )
                    .with_change(
                        Change::new("Fixed `two`")
                            .with_url("#2", "https://github.com/ploys/example/pull/2"),
                    ),
            )
            .with_reference(
                "0.1.0",
                "https://github.com/ploys/example/releases/tag/0.1.0",
            );

        let output = indoc! {"
            ## [0.1.0] - 2024-01-01

            ### Fixed

            Fixed some things.

            - Fixed `one` ([#1](https://github.com/ploys/example/pull/1))
            - Fixed `two` ([#2](https://github.com/ploys/example/pull/2))

            [0.1.0]: https://github.com/ploys/example/releases/tag/0.1.0\
        "};

        assert_eq!(release.version(), "0.1.0");
        assert_eq!(release.date(), Some("2024-01-01"));
        assert_eq!(release.changesets().count(), 1);
        assert_eq!(release.references().count(), 1);
        assert_eq!(release.to_string(), output);
    }
}
