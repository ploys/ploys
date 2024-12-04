use std::fmt::{self, Display};

use markdown::mdast::Node;
use markdown::ParseOptions;

use super::Text;

/// A changelog change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    message: String,
    url: Option<(String, String)>,
}

impl Change {
    /// Constructs a new change.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            url: None,
        }
    }

    /// Gets the change message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Gets the change URL.
    pub fn url(&self) -> Option<(&str, &str)> {
        self.url.as_ref().map(|(label, url)| (&**label, &**url))
    }

    /// Sets the change URL.
    pub fn set_url(&mut self, label: impl Into<String>, url: impl Into<String>) -> &mut Self {
        self.url = Some((label.into(), url.into()));
        self
    }

    /// Builds the change with the given URL.
    pub fn with_url(mut self, label: impl Into<String>, url: impl Into<String>) -> Self {
        self.set_url(label, url);
        self
    }
}

impl Change {
    /// Converts the change into markdown nodes.
    pub(super) fn into_nodes(self) -> Vec<Node> {
        let mut nodes = Vec::new();

        let md = markdown::to_mdast(&self.message, &ParseOptions::default()).expect("markdown");

        if let Node::Root(root) = md {
            nodes.extend(root.children);
        }

        if let Some(Node::Paragraph(paragraph)) = nodes.first_mut() {
            if let Some((label, url)) = self.url {
                if let Some(Node::Text(text)) = paragraph.children.last_mut() {
                    text.value.push_str(" (");
                } else {
                    paragraph.children.push(Node::Text(markdown::mdast::Text {
                        value: String::from(" ("),
                        position: None,
                    }));
                }

                paragraph.children.push(Node::Link(markdown::mdast::Link {
                    children: vec![Node::Text(markdown::mdast::Text {
                        value: label,
                        position: None,
                    })],
                    position: None,
                    url,
                    title: None,
                }));

                paragraph.children.push(Node::Text(markdown::mdast::Text {
                    value: String::from(")"),
                    position: None,
                }));
            }
        }

        nodes
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "- {}", self.message)?;

        if let Some((label, url)) = &self.url {
            if f.alternate() {
                write!(f, " ({label})")?;
            } else {
                write!(f, " ([{label}]({url}))")?;
            }
        }

        Ok(())
    }
}

/// A single change in a changelog.
#[derive(Clone, Debug)]
pub struct ChangeRef<'a> {
    text: Text<'a>,
}

impl ChangeRef<'_> {
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

    /// Creates an owned change.
    pub fn to_owned(&self) -> Change {
        fn inner(nodes: &[Node]) -> Option<Change> {
            let mut nodes = nodes.iter().rev();

            let Node::Text(close) = nodes.next()? else {
                return None;
            };

            if close.value != ")" {
                return None;
            }

            let Node::Link(link) = nodes.next()? else {
                return None;
            };

            let Node::Text(open) = nodes.next()? else {
                return None;
            };

            if !open.value.ends_with("(") {
                return None;
            }

            let nodes = nodes
                .rev()
                .cloned()
                .chain(
                    Some(open.clone())
                        .into_iter()
                        .map(|mut text| {
                            text.value = text.value.trim_end_matches("(").trim_end().to_owned();
                            text
                        })
                        .map(Node::Text),
                )
                .collect::<Vec<_>>();

            let message = Text { nodes: &nodes };
            let text = Text {
                nodes: &link.children,
            };

            Some(Change {
                message: message.to_string(),
                url: Some((text.to_string(), link.url.to_owned())),
            })
        }

        match inner(self.text.nodes) {
            Some(change) => change,
            None => Change {
                message: self.message().to_string(),
                url: None,
            },
        }
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

impl Display for ChangeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "- {:#}", self.text)
        } else {
            write!(f, "- {}", self.text)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Change;

    #[test]
    fn test_change() {
        let change = Change::new("Changed `something`")
            .with_url("#1", "https://github.com/ploys/example/pull/1");

        assert_eq!(change.message(), "Changed `something`");
        assert_eq!(
            change.url(),
            Some(("#1", "https://github.com/ploys/example/pull/1"))
        );
        assert_eq!(
            change.to_string(),
            "- Changed `something` ([#1](https://github.com/ploys/example/pull/1))"
        );
    }
}
