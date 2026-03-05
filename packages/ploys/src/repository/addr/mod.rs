mod error;

use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

pub use self::error::Error;

/// A repository address.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RepoAddr(Url);

impl RepoAddr {
    /// Constructs a new repository address.
    ///
    /// This requires that both `owner` and `name` match the following regular
    /// expression: `^[a-zA-Z0-9\-_\.]+$`
    pub fn new(owner: impl AsRef<str>, name: impl AsRef<str>) -> Result<Self, Error> {
        let owner = owner.as_ref();
        let name = name.as_ref();

        if owner.is_empty() || !owner.chars().all(is_valid_char) {
            return Err(Error::invalid(format!("{owner}/{name}")));
        }

        if name.is_empty() || !name.chars().all(is_valid_char) {
            return Err(Error::invalid(format!("{owner}/{name}")));
        }

        let url = format!("https://github.com/{owner}/{name}");

        Ok(Self(Url::parse(&url).expect("valid url")))
    }

    /// Gets the repository owner.
    pub fn owner(&self) -> &str {
        self.0
            .path()
            .trim_start_matches("/")
            .split('/')
            .next()
            .expect("valid owner")
    }

    /// Gets the repository name.
    pub fn name(&self) -> &str {
        self.0
            .path()
            .trim_start_matches("/")
            .split('/')
            .nth(1)
            .expect("valid name")
    }

    /// Gets the repository full name in the `owner/name` format.
    pub fn full_name(&self) -> &str {
        self.0.path().trim_start_matches("/")
    }

    /// Gets the repository URL.
    pub fn url(&self) -> &Url {
        &self.0
    }
}

impl Debug for RepoAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RepoAddr").field(&self.full_name()).finish()
    }
}

impl Display for RepoAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match f.alternate() {
            true => write!(f, "{}", self.url()),
            false => write!(f, "{}", self.full_name()),
        }
    }
}

impl Serialize for RepoAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for RepoAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = RepoAddr;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<Self::Value>()
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl FromStr for RepoAddr {
    type Err = Error;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        match addr
            .trim_start_matches("https://github.com/")
            .split_once('/')
        {
            Some((owner, name)) => Self::new(owner, name),
            None => Err(Error::invalid(addr)),
        }
    }
}

impl TryFrom<&str> for RepoAddr {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&String> for RepoAddr {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for RepoAddr {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<Url> for RepoAddr {
    type Error = Error;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        match url.as_str().starts_with("https://github.com/") {
            true => {
                if url.query().is_none()
                    && url.fragment().is_none()
                    && let Some((owner, name)) = url.path().trim_start_matches("/").split_once('/')
                    && !owner.is_empty()
                    && owner.chars().all(is_valid_char)
                    && !name.is_empty()
                    && name.chars().all(is_valid_char)
                {
                    Ok(Self(url))
                } else {
                    Err(Error::invalid(url))
                }
            }
            false => Err(Error::invalid(url)),
        }
    }
}

/// A helper utility to define valid repository `owner` and `name` characters.
fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-' || c == '_' || c == '.'
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::RepoAddr;

    #[test]
    fn test_parse() {
        assert!("ploys/ploys".parse::<RepoAddr>().is_ok());
        assert!("rust-lang/rust".parse::<RepoAddr>().is_ok());
        assert!("one/two/three".parse::<RepoAddr>().is_err());
        assert!("https://github.com/ploys/ploys".parse::<RepoAddr>().is_ok());
    }

    #[test]
    fn test_methods() {
        let repo_addr = RepoAddr::new("owner", "name").unwrap();

        assert_eq!(repo_addr.owner(), "owner");
        assert_eq!(repo_addr.name(), "name");
        assert_eq!(repo_addr.full_name(), "owner/name");
        assert_eq!(
            repo_addr.url(),
            &Url::parse("https://github.com/owner/name").unwrap()
        );
    }
}
