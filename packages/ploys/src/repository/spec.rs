use std::fmt::{self, Display};
use std::str::FromStr;

use strum::{EnumIs, EnumTryAs};
use url::Url;

/// The repository specification.
///
/// A repository can be specified in one of two formats:
/// - `Url`, e.g. `https://github.com/ploys/ploys`
/// - `Short`, e.g. `github:ploys/ploys` or `ploys/ploys`
///
/// The above formats are all equivalent and refer to a GitHub repository where
/// the owner is named `ploys` and the repository is also named `ploys`. The
/// short prefix `github` can be ommitted as it is the default.
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumIs, EnumTryAs)]
pub enum RepoSpec {
    /// A URL repository specification.
    Url(Url),
    /// A short repository specification.
    Short(ShortRepoSpec),
}

impl RepoSpec {
    /// Constructs a URL repository specification.
    pub fn url(url: impl Into<Url>) -> Self {
        Self::Url(url.into())
    }

    /// Constructs a short repository specification.
    pub fn short(short: impl Into<ShortRepoSpec>) -> Self {
        Self::Short(short.into())
    }

    /// Gets the repository URL.
    pub fn to_url(&self) -> Url {
        match self {
            Self::Url(url) => url.clone(),
            Self::Short(short) => short.to_url(),
        }
    }

    /// Gets a GitHub repository specification.
    #[cfg(feature = "github")]
    pub fn to_github(&self) -> Option<super::types::github::GitHubRepoSpec> {
        match self {
            Self::Url(url) => match url.host()? {
                url::Host::Domain("github.com") => url
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches(".git")
                    .parse()
                    .ok(),
                _ => None,
            },
            Self::Short(short) => match short {
                ShortRepoSpec::Default(github) | ShortRepoSpec::GitHub(github) => {
                    Some(github.clone())
                }
            },
        }
    }
}

impl Display for RepoSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(url) => Display::fmt(url, f),
            Self::Short(short) => Display::fmt(short, f),
        }
    }
}

impl FromStr for RepoSpec {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        match spec.parse::<ShortRepoSpec>() {
            Ok(short) => Ok(Self::Short(short)),
            Err(err @ Error::Unsupported(_))
                if !spec.starts_with("http:") && !spec.starts_with("https:") =>
            {
                Err(err)
            }
            Err(_) => match spec.parse::<Url>() {
                Ok(url) => Ok(Self::Url(url)),
                Err(_) => Err(Error::invalid(spec)),
            },
        }
    }
}

impl From<Url> for RepoSpec {
    fn from(url: Url) -> Self {
        Self::Url(url)
    }
}

impl From<ShortRepoSpec> for RepoSpec {
    fn from(short: ShortRepoSpec) -> Self {
        Self::Short(short)
    }
}

#[cfg(feature = "github")]
impl From<super::types::github::GitHubRepoSpec> for RepoSpec {
    fn from(github: super::types::github::GitHubRepoSpec) -> Self {
        Self::Short(ShortRepoSpec::GitHub(github))
    }
}

/// The short repository specification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumIs, EnumTryAs)]
pub enum ShortRepoSpec {
    /// A default (GitHub) repository specification.
    #[cfg(feature = "github")]
    Default(super::types::github::GitHubRepoSpec),
    /// A GitHub repository specification.
    #[cfg(feature = "github")]
    GitHub(super::types::github::GitHubRepoSpec),
}

impl ShortRepoSpec {
    /// Gets the repository URL.
    pub fn to_url(&self) -> Url {
        match self {
            #[cfg(feature = "github")]
            Self::Default(spec) | Self::GitHub(spec) => spec.to_url(),
            #[cfg(not(feature = "github"))]
            _ => unreachable!(),
        }
    }
}

impl Display for ShortRepoSpec {
    #[cfg_attr(not(feature = "github"), allow(unused_variables))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "github")]
            Self::Default(spec) => write!(f, "{spec}"),
            #[cfg(feature = "github")]
            Self::GitHub(spec) => write!(f, "github:{spec}"),
            #[cfg(not(feature = "github"))]
            _ => unreachable!(),
        }
    }
}

impl FromStr for ShortRepoSpec {
    type Err = Error;

    #[cfg_attr(not(feature = "github"), allow(unused_variables))]
    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        match spec.split_once(':') {
            Some((kind, rest)) => match kind {
                #[cfg(feature = "github")]
                "github" => Ok(Self::GitHub(rest.parse()?)),
                _ => Err(Error::unsupported(spec)),
            },
            #[cfg(feature = "github")]
            None => Ok(Self::Default(spec.parse()?)),
            #[cfg(not(feature = "github"))]
            None => Err(Error::unsupported(spec)),
        }
    }
}

#[cfg(feature = "github")]
impl From<super::types::github::GitHubRepoSpec> for ShortRepoSpec {
    fn from(github: super::types::github::GitHubRepoSpec) -> Self {
        Self::GitHub(github)
    }
}

/// A repository specification error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid repository specification.
    Invalid(String),
    /// An unsupported repository specification.
    Unsupported(String),
}

impl Error {
    /// Constructs a new invalid error variant.
    pub fn invalid(input: impl Into<String>) -> Self {
        Self::Invalid(input.into())
    }

    /// Constructs a new unsupported error variant.
    pub fn unsupported(input: impl Into<String>) -> Self {
        Self::Unsupported(input.into())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(input) => write!(f, "Invalid repository specification: {input}"),
            Self::Unsupported(input) => write!(f, "Unsupported repository specification: {input}"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg_attr(feature = "github", test)]
    fn test_parse() {
        use super::RepoSpec;

        assert!("rust-lang/rust".parse::<RepoSpec>().is_ok());
        assert!("ploys/ploys".parse::<RepoSpec>().is_ok());
        assert!("github:ploys/ploys".parse::<RepoSpec>().is_ok());
        assert!("https://github.com/ploys/ploys".parse::<RepoSpec>().is_ok());
        assert!("ploys/ploys/ploys".parse::<RepoSpec>().is_err());
        assert!("unknown:ploys/ploys".parse::<RepoSpec>().is_err());
    }
}
