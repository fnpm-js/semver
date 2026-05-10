use std::{cmp::Ordering, fmt, str::FromStr};

use ecow::EcoVec;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Eq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: EcoVec<SmolStr>,
    build: EcoVec<SmolStr>,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: EcoVec::new(),
            build: EcoVec::new(),
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Error::InvalidVersion {
                input: input.to_string(),
            });
        }

        let (version_part, build) = match input.split_once('+') {
            Some((v, b)) => (v, parse_dot_separated(b)),
            None => (input, EcoVec::new()),
        };

        let (core_part, pre) = match version_part.split_once('-') {
            Some((c, p)) => (c, parse_dot_separated(p)),
            None => (version_part, EcoVec::new()),
        };

        let (major_str, rest) = core_part
            .split_once('.')
            .ok_or_else(|| Error::InvalidVersion {
                input: input.to_string(),
            })?;
        let (minor_str, patch_str) =
            rest.split_once('.')
                .ok_or_else(|| Error::InvalidVersion {
                    input: input.to_string(),
                })?;

        if patch_str.contains('.') {
            return Err(Error::InvalidVersion {
                input: input.to_string(),
            });
        }

        let major = parse_version_number(major_str, input)?;
        let minor = parse_version_number(minor_str, input)?;
        let patch = parse_version_number(patch_str, input)?;

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    pub fn major(&self) -> u64 {
        self.major
    }

    pub fn minor(&self) -> u64 {
        self.minor
    }

    pub fn patch(&self) -> u64 {
        self.patch
    }

    pub fn pre(&self) -> &EcoVec<SmolStr> {
        &self.pre
    }

    pub fn build(&self) -> &EcoVec<SmolStr> {
        &self.build
    }
}

fn parse_version_number(s: &str, input: &str) -> Result<u64> {
    if s.is_empty() {
        return Err(Error::InvalidVersion {
            input: input.to_string(),
        });
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(Error::InvalidVersion {
            input: input.to_string(),
        });
    }
    s.parse::<u64>().map_err(|_| Error::InvalidVersion {
        input: input.to_string(),
    })
}

fn parse_dot_separated(s: &str) -> EcoVec<SmolStr> {
    s.split('.').map(SmolStr::new).collect()
}

fn cmp_pre_identifier(a: &SmolStr, b: &SmolStr) -> Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(na), Ok(nb)) => na.cmp(&nb),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre == other.pre
    }
}

impl Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
        self.pre.hash(state);
    }
}

use std::hash::Hash;

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    let len = self.pre.len().min(other.pre.len());
                    for i in 0..len {
                        let ord = cmp_pre_identifier(&self.pre[i], &other.pre[i]);
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    }
                    self.pre.len().cmp(&other.pre.len())
                }
            })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-")?;
            for (i, id) in self.pre.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{id}")?;
            }
        }
        if !self.build.is_empty() {
            write!(f, "+")?;
            for (i, id) in self.build.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{id}")?;
            }
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major(), 1);
        assert_eq!(v.minor(), 2);
        assert_eq!(v.patch(), 3);
        assert!(v.pre().is_empty());
        assert!(v.build().is_empty());
    }

    #[test]
    fn test_parse_with_pre() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(v.major(), 1);
        assert_eq!(v.pre().len(), 2);
        assert_eq!(v.pre()[0].as_str(), "alpha");
        assert_eq!(v.pre()[1].as_str(), "1");
    }

    #[test]
    fn test_parse_with_build() {
        let v = Version::parse("1.2.3+build.123").unwrap();
        assert_eq!(v.build().len(), 2);
        assert_eq!(v.build()[0].as_str(), "build");
        assert_eq!(v.build()[1].as_str(), "123");
    }

    #[test]
    fn test_parse_with_pre_and_build() {
        let v = Version::parse("1.2.3-beta.2+build.456").unwrap();
        assert_eq!(v.pre().len(), 2);
        assert_eq!(v.build().len(), 2);
    }

    #[test]
    fn test_parse_errors() {
        assert!(Version::parse("").is_err());
        assert!(Version::parse("1").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("a.b.c").is_err());
        assert!(Version::parse("01.2.3").is_err());
        assert!(Version::parse("1.02.3").is_err());
        assert!(Version::parse("1.2.03").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(Version::parse("1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(
            Version::parse("1.2.3-alpha.1").unwrap().to_string(),
            "1.2.3-alpha.1"
        );
        assert_eq!(
            Version::parse("1.2.3+build").unwrap().to_string(),
            "1.2.3+build"
        );
        assert_eq!(
            Version::parse("1.2.3-rc.1+build").unwrap().to_string(),
            "1.2.3-rc.1+build"
        );
    }

    #[test]
    fn test_ordering() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("1.0.1").unwrap();
        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
    }

    #[test]
    fn test_pre_release_ordering() {
        let v1 = Version::parse("1.0.0-alpha").unwrap();
        let v2 = Version::parse("1.0.0-alpha.1").unwrap();
        let v3 = Version::parse("1.0.0-beta").unwrap();
        let v4 = Version::parse("1.0.0").unwrap();
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn test_pre_numeric_vs_alpha() {
        let v1 = Version::parse("1.0.0-1").unwrap();
        let v2 = Version::parse("1.0.0-alpha").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn test_pre_numeric_ordering() {
        let v1 = Version::parse("1.0.0-1").unwrap();
        let v2 = Version::parse("1.0.0-2").unwrap();
        let v3 = Version::parse("1.0.0-10").unwrap();
        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn test_build_ignored_in_eq() {
        let v1 = Version::parse("1.0.0+build1").unwrap();
        let v2 = Version::parse("1.0.0+build2").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let v = Version::parse("1.2.3-alpha.1+build").unwrap();
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"1.2.3-alpha.1+build\"");
        let deserialized: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.to_string(), "1.2.3-alpha.1+build");
    }

    #[test]
    fn test_from_str() {
        let v: Version = "1.2.3".parse().unwrap();
        assert_eq!(v.major(), 1);
    }
}
