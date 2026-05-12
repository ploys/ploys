use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::{Host, ParseError};

/// The server address.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServAddr(Host);

impl Display for ServAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Default for ServAddr {
    fn default() -> Self {
        Self(Host::Domain(String::from("api.ploys.dev")))
    }
}

impl FromStr for ServAddr {
    type Err = ParseError;

    fn from_str(host: &str) -> Result<Self, Self::Err> {
        Ok(Self(Host::parse(host)?))
    }
}
