use std::fmt::{self, Display};
use std::str::FromStr;

use semver::{BuildMetadata, Prerelease, Version};

/// The package semver bump target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bump {
    Major,
    Minor,
    Patch,
    Rc,
    Beta,
    Alpha,
}

impl Bump {
    /// Updates the given version.
    pub fn bump(self, version: &mut Version) -> Result<(), Error> {
        match self {
            Self::Major => {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
                version.pre = Prerelease::EMPTY;
                version.build = BuildMetadata::EMPTY;
            }
            Self::Minor => {
                version.minor += 1;
                version.patch = 0;
                version.pre = Prerelease::EMPTY;
                version.build = BuildMetadata::EMPTY;
            }
            Self::Patch => match version.pre.is_empty() {
                true => {
                    version.patch += 1;
                    version.pre = Prerelease::EMPTY;
                    version.build = BuildMetadata::EMPTY;
                }
                false => {
                    version.pre = Prerelease::EMPTY;
                }
            },
            Self::Rc => match parse_prelease(&version.pre) {
                Some((Tag::Rc, ver)) => {
                    version.pre = Prerelease::new(&format!("rc.{}", ver.unwrap_or(0) + 1))?;
                    version.build = BuildMetadata::EMPTY;
                }
                Some(_) => {
                    version.pre = Prerelease::new("rc.1")?;
                    version.build = BuildMetadata::EMPTY;
                }
                None => match version.major {
                    0 => {
                        version.minor += 1;
                        version.patch = 0;
                        version.pre = Prerelease::new("rc.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                    _ => {
                        version.major += 1;
                        version.minor = 0;
                        version.patch = 0;
                        version.pre = Prerelease::new("rc.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                },
            },
            Self::Beta => match parse_prelease(&version.pre) {
                Some((Tag::Rc, _)) => return Err(Error::Unsupported(self)),
                Some((Tag::Beta, ver)) => {
                    version.pre = Prerelease::new(&format!("beta.{}", ver.unwrap_or(0) + 1))?;
                    version.build = BuildMetadata::EMPTY;
                }
                Some(_) => {
                    version.pre = Prerelease::new("beta.1")?;
                    version.build = BuildMetadata::EMPTY;
                }
                None => match version.major {
                    0 => {
                        version.minor += 1;
                        version.patch = 0;
                        version.pre = Prerelease::new("beta.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                    _ => {
                        version.major += 1;
                        version.minor = 0;
                        version.patch = 0;
                        version.pre = Prerelease::new("beta.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                },
            },
            Self::Alpha => match parse_prelease(&version.pre) {
                Some((Tag::Rc, _)) => return Err(Error::Unsupported(self)),
                Some((Tag::Beta, _)) => return Err(Error::Unsupported(self)),
                Some((Tag::Alpha, ver)) => {
                    version.pre = Prerelease::new(&format!("alpha.{}", ver.unwrap_or(0) + 1))?;
                    version.build = BuildMetadata::EMPTY;
                }
                None => match version.major {
                    0 => {
                        version.minor += 1;
                        version.patch = 0;
                        version.pre = Prerelease::new("alpha.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                    _ => {
                        version.major += 1;
                        version.minor = 0;
                        version.patch = 0;
                        version.pre = Prerelease::new("alpha.1")?;
                        version.build = BuildMetadata::EMPTY;
                    }
                },
            },
        }

        Ok(())
    }

    /// Updates the given version string slice.
    pub fn bump_str(self, version: &str) -> Result<Version, Error> {
        let mut version = version.parse::<Version>()?;

        self.bump(&mut version)?;

        Ok(version)
    }
}

impl Display for Bump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bump::Major => write!(f, "major"),
            Bump::Minor => write!(f, "minor"),
            Bump::Patch => write!(f, "patch"),
            Bump::Rc => write!(f, "rc"),
            Bump::Beta => write!(f, "beta"),
            Bump::Alpha => write!(f, "alpha"),
        }
    }
}

impl FromStr for Bump {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "rc" => Ok(Self::Rc),
            "beta" => Ok(Self::Beta),
            "alpha" => Ok(Self::Alpha),
            _ => Err(Error::Invalid(s.to_string())),
        }
    }
}

/// The bump level or version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BumpOrVersion {
    Bump(Bump),
    Version(Version),
}

impl FromStr for BumpOrVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Bump>() {
            Ok(bump) => Ok(Self::Bump(bump)),
            Err(_) => match s.parse::<Version>() {
                Ok(version) => Ok(Self::Version(version)),
                Err(err) => Err(Error::Semver(err)),
            },
        }
    }
}

impl From<Bump> for BumpOrVersion {
    fn from(bump: Bump) -> Self {
        Self::Bump(bump)
    }
}

impl From<Version> for BumpOrVersion {
    fn from(version: Version) -> Self {
        Self::Version(version)
    }
}

/// The bump error.
#[derive(Debug)]
pub enum Error {
    /// A semver error.
    Semver(semver::Error),
    /// An invalid bump error.
    Invalid(String),
    /// An unsupported bump error.
    Unsupported(Bump),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Semver(err) => Display::fmt(err, f),
            Self::Invalid(bump) => write!(f, "Invalid bump: `{bump}`"),
            Self::Unsupported(bump) => write!(f, "Unsupported bump: `{bump}`"),
        }
    }
}

impl std::error::Error for Error {}

impl From<semver::Error> for Error {
    fn from(err: semver::Error) -> Self {
        Self::Semver(err)
    }
}

enum Tag {
    Rc,
    Beta,
    Alpha,
}

impl FromStr for Tag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rc" => Ok(Self::Rc),
            "beta" => Ok(Self::Beta),
            "alpha" => Ok(Self::Alpha),
            _ => Err(()),
        }
    }
}

fn parse_prelease(prerelease: &Prerelease) -> Option<(Tag, Option<u64>)> {
    match prerelease.is_empty() {
        true => None,
        false => match prerelease.as_str().split_once('.') {
            Some((lhs, rhs)) => Some((lhs.parse::<Tag>().ok()?, rhs.parse::<u64>().ok())),
            None => Some((prerelease.as_str().parse::<Tag>().ok()?, None)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Bump, Error};

    #[test]
    fn test_bump_major() -> Result<(), Error> {
        let a = Bump::Major.bump_str("0.0.0")?;
        let b = Bump::Major.bump_str("1.0.0")?;
        let c = Bump::Major.bump_str("1.2.3")?;

        assert_eq!(a, "1.0.0".parse().unwrap());
        assert_eq!(b, "2.0.0".parse().unwrap());
        assert_eq!(c, "2.0.0".parse().unwrap());

        Ok(())
    }

    #[test]
    fn test_bump_minor() -> Result<(), Error> {
        let a = Bump::Minor.bump_str("0.0.0")?;
        let b = Bump::Minor.bump_str("0.1.0")?;
        let c = Bump::Minor.bump_str("1.2.3")?;

        assert_eq!(a, "0.1.0".parse().unwrap());
        assert_eq!(b, "0.2.0".parse().unwrap());
        assert_eq!(c, "1.3.0".parse().unwrap());

        Ok(())
    }

    #[test]
    fn test_bump_patch() -> Result<(), Error> {
        let a = Bump::Patch.bump_str("0.0.0")?;
        let b = Bump::Patch.bump_str("1.0.0")?;
        let c = Bump::Patch.bump_str("1.2.3")?;

        assert_eq!(a, "0.0.1".parse().unwrap());
        assert_eq!(b, "1.0.1".parse().unwrap());
        assert_eq!(c, "1.2.4".parse().unwrap());

        Ok(())
    }

    #[test]
    fn test_bump_rc() -> Result<(), Error> {
        let a = Bump::Rc.bump_str("0.1.0")?;
        let b = Bump::Rc.bump_str("1.0.0")?;
        let c = Bump::Rc.bump_str("1.0.0-alpha.1")?;
        let d = Bump::Rc.bump_str("1.0.0-beta.1")?;
        let e = Bump::Rc.bump_str("1.0.0-rc.1")?;

        assert_eq!(a, "0.2.0-rc.1".parse().unwrap());
        assert_eq!(b, "2.0.0-rc.1".parse().unwrap());
        assert_eq!(c, "1.0.0-rc.1".parse().unwrap());
        assert_eq!(d, "1.0.0-rc.1".parse().unwrap());
        assert_eq!(e, "1.0.0-rc.2".parse().unwrap());

        Ok(())
    }

    #[test]
    fn test_bump_beta() -> Result<(), Error> {
        let a = Bump::Beta.bump_str("0.1.0")?;
        let b = Bump::Beta.bump_str("1.0.0")?;
        let c = Bump::Beta.bump_str("1.0.0-alpha.1")?;
        let d = Bump::Beta.bump_str("1.0.0-beta.1")?;
        let e = Bump::Beta.bump_str("1.0.0-rc.1");

        assert_eq!(a, "0.2.0-beta.1".parse().unwrap());
        assert_eq!(b, "2.0.0-beta.1".parse().unwrap());
        assert_eq!(c, "1.0.0-beta.1".parse().unwrap());
        assert_eq!(d, "1.0.0-beta.2".parse().unwrap());

        assert!(e.is_err());

        Ok(())
    }

    #[test]
    fn test_bump_alpha() -> Result<(), Error> {
        let a = Bump::Alpha.bump_str("0.1.0")?;
        let b = Bump::Alpha.bump_str("1.0.0")?;
        let c = Bump::Alpha.bump_str("0.1.0-alpha.1")?;
        let d = Bump::Alpha.bump_str("1.0.0-alpha.1")?;
        let e = Bump::Alpha.bump_str("1.0.0-beta.1");
        let f = Bump::Alpha.bump_str("1.0.0-rc.1");

        assert_eq!(a, "0.2.0-alpha.1".parse().unwrap());
        assert_eq!(b, "2.0.0-alpha.1".parse().unwrap());
        assert_eq!(c, "0.1.0-alpha.2".parse().unwrap());
        assert_eq!(d, "1.0.0-alpha.2".parse().unwrap());

        assert!(e.is_err());
        assert!(f.is_err());

        Ok(())
    }
}
