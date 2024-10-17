mod change;
mod changeset;
mod reference;
mod release;
mod text;

use std::convert::Infallible;
use std::fmt::{self, Display};
use std::str::FromStr;

use markdown::mdast::{Node, Root};
use markdown::ParseOptions;

pub use self::change::ChangeRef;
pub use self::changeset::ChangesetRef;
pub use self::reference::ReferenceRef;
pub use self::release::ReleaseRef;
pub use self::text::{MultilineText, Text};

/// Represents a changelog file.
///
/// This uses the [keep a changelog](https://keepachangelog.com) format to parse
/// and generate changelogs. There is very limited support for deviation from
/// this format so the changelog should not yet be manually edited.
#[derive(Clone, Debug)]
pub struct Changelog(Node);

impl Changelog {
    /// Gets the changelog title.
    pub fn title(&self) -> Option<Text<'_>> {
        self.0
            .children()?
            .iter()
            .find(|node| matches!(node, Node::Heading(heading) if heading.depth == 1))
            .and_then(|node| Some(Text::from_nodes(node.children()?)))
    }

    /// Gets the changelog description.
    pub fn description(&self) -> Option<MultilineText<'_>> {
        self.get_sections()
            .next()
            .and_then(|nodes| match nodes.first() {
                Some(Node::Heading(heading)) if heading.depth == 1 => {
                    MultilineText::from_nodes(&nodes[1..])
                }
                Some(Node::Heading(heading)) if heading.depth == 2 => None,
                _ => MultilineText::from_nodes(nodes),
            })
    }

    /// Gets a release for the given version.
    pub fn get_release(&self, version: impl AsRef<str>) -> Option<ReleaseRef<'_>> {
        self.releases()
            .find(|release| release.version() == version.as_ref())
    }

    /// Gets an iterator over the releases.
    pub fn releases(&self) -> impl Iterator<Item = ReleaseRef<'_>> {
        self.get_sections().filter_map(ReleaseRef::from_nodes)
    }

    /// Gets an iterator over the references.
    pub fn references(&self) -> impl Iterator<Item = ReferenceRef<'_>> {
        self.releases()
            .last()
            .into_iter()
            .flat_map(|release| release.references())
    }
}

impl Changelog {
    /// Gets the sections separated by a second-level heading.
    fn get_sections(&self) -> impl Iterator<Item = &[Node]> {
        self.0.children().into_iter().flat_map(|nodes| {
            nodes.chunk_by(|_, node| !matches!(node, Node::Heading(heading) if heading.depth == 2))
        })
    }
}

impl Default for Changelog {
    fn default() -> Self {
        Self(Node::Root(Root {
            children: Vec::new(),
            position: None,
        }))
    }
}

impl Display for Changelog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(title) = self.title() {
            if f.alternate() {
                write!(f, "# {title:#}")?;
            } else {
                write!(f, "# {title}")?;
            }
        } else {
            write!(f, "# Changelog")?;
        }

        if let Some(description) = self.description() {
            if f.alternate() {
                write!(f, "\n\n{description:#}")?;
            } else {
                write!(f, "\n\n{description}")?;
            }
        }

        let mut releases = self.releases().peekable();

        if releases.peek().is_some() {
            write!(f, "\n\n")?;

            while let Some(release) = releases.next() {
                if f.alternate() {
                    write!(f, "{release:#}")?;
                } else {
                    write!(f, "{release}")?;
                }

                if releases.peek().is_some() {
                    write!(f, "\n\n")?;
                }
            }
        }

        writeln!(f)?;

        Ok(())
    }
}

impl FromStr for Changelog {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            markdown::to_mdast(value, &ParseOptions::default()).expect("markdown"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::Changelog;

    #[test]
    fn test_changelog_parser() {
        let changelog_text = indoc! {"
            # Changelog

            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [0.2.0] - 2024-01-04

            ### Added

            - Added three ([#3](https://github.com/ploys/example/pull/3))

            ### Removed

            - Removed four ([#4](https://github.com/ploys/example/pull/4))

            ### Fixed

            - Fixed five ([#5](https://github.com/ploys/example/pull/5))

            ### Changed

            - Changed six ([#6](https://github.com/ploys/example/pull/6))
            - Changed seven ([#7](https://github.com/ploys/example/pull/7))
            - Changed `eight` ([#8](https://github.com/ploys/example/pull/8))

            ## [0.1.2] - 2024-01-03

            ### Fixed

            This changeset has a description.

            - Fixed two ([#2](https://github.com/ploys/example/pull/2))

            ## [0.1.1] - 2024-01-02

            ### Fixed

            - Fixed one ([#1](https://github.com/ploys/example/pull/1))

            ## [0.1.0] - 2024-01-01

            This is the initial release.

            [0.2.0]: https://github.com/ploys/example/releases/tag/0.2.0
            [0.1.2]: https://github.com/ploys/example/releases/tag/0.1.2
            [0.1.1]: https://github.com/ploys/example/releases/tag/0.1.1
            [0.1.0]: https://github.com/ploys/example/releases/tag/0.1.0
        "};

        let changelog = changelog_text.parse::<Changelog>().unwrap();

        assert_eq!(changelog.title().unwrap().to_string(), "Changelog");

        let mut references = changelog.references();

        let t0 = references.next().unwrap();
        let t1 = references.next().unwrap();
        let t2 = references.next().unwrap();
        let t3 = references.next().unwrap();

        assert_eq!(t0.id(), "0.2.0");
        assert_eq!(t1.id(), "0.1.2");
        assert_eq!(t2.id(), "0.1.1");
        assert_eq!(t3.id(), "0.1.0");

        assert_eq!(
            t0.url(),
            "https://github.com/ploys/example/releases/tag/0.2.0"
        );
        assert_eq!(
            t1.url(),
            "https://github.com/ploys/example/releases/tag/0.1.2"
        );
        assert_eq!(
            t2.url(),
            "https://github.com/ploys/example/releases/tag/0.1.1"
        );
        assert_eq!(
            t3.url(),
            "https://github.com/ploys/example/releases/tag/0.1.0"
        );

        let r0 = changelog.get_release("0.1.0").unwrap();
        let r1 = changelog.get_release("0.1.1").unwrap();
        let r2 = changelog.get_release("0.1.2").unwrap();
        let r3 = changelog.get_release("0.2.0").unwrap();

        assert_eq!(r0.version(), "0.1.0");
        assert_eq!(r1.version(), "0.1.1");
        assert_eq!(r2.version(), "0.1.2");
        assert_eq!(r3.version(), "0.2.0");

        assert_eq!(r0.date(), Some("2024-01-01"));
        assert_eq!(r1.date(), Some("2024-01-02"));
        assert_eq!(r2.date(), Some("2024-01-03"));
        assert_eq!(r3.date(), Some("2024-01-04"));

        assert_eq!(
            r0.description().unwrap().to_string(),
            "This is the initial release.",
        );

        assert_eq!(r0.changesets().count(), 0);
        assert_eq!(r1.changesets().count(), 1);
        assert_eq!(r2.changesets().count(), 1);
        assert_eq!(r3.changesets().count(), 4);

        let s0 = r1.fixed().unwrap();
        let s1 = r2.fixed().unwrap();
        let s2 = r3.added().unwrap();
        let s3 = r3.removed().unwrap();
        let s4 = r3.fixed().unwrap();
        let s5 = r3.changed().unwrap();

        assert_eq!(s0.changes().count(), 1);
        assert_eq!(s1.changes().count(), 1);
        assert_eq!(s2.changes().count(), 1);
        assert_eq!(s3.changes().count(), 1);
        assert_eq!(s4.changes().count(), 1);
        assert_eq!(s5.changes().count(), 3);

        assert_eq!(s0.label(), "Fixed");
        assert_eq!(s1.label(), "Fixed");
        assert_eq!(s2.label(), "Added");
        assert_eq!(s3.label(), "Removed");
        assert_eq!(s4.label(), "Fixed");
        assert_eq!(s5.label(), "Changed");

        assert_eq!(
            s1.description().unwrap().to_string(),
            "This changeset has a description."
        );

        let mut changes = s5.changes();

        let c0 = changes.next().unwrap();
        let c1 = changes.next().unwrap();
        let c2 = changes.next().unwrap();

        assert_eq!(c0.message(), "Changed six (#6)");
        assert_eq!(c1.message(), "Changed seven (#7)");
        assert_eq!(c2.message(), "Changed `eight` (#8)");

        assert_eq!(c0.url(), Some("https://github.com/ploys/example/pull/6"));
        assert_eq!(c1.url(), Some("https://github.com/ploys/example/pull/7"));
        assert_eq!(c2.url(), Some("https://github.com/ploys/example/pull/8"));

        assert_eq!(changelog.to_string(), changelog_text);
    }
}
