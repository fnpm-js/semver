mod tag;

pub use tag::TagSpec;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::{Version, VersionRange, error::{Error, Result}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VersionSpecKind {
    Range(VersionRange),
    Tag(TagSpec),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VersionSpec {
    raw: SmolStr,
    kind: VersionSpecKind,
}

impl VersionSpec {
    pub fn parse(input: &str) -> Result<Self> {
        Self::parse_range_or_tag(input)
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn matches(&self, version: &Version) -> bool {
        match &self.kind {
            VersionSpecKind::Range(range) => range.matches(version),
            VersionSpecKind::Tag(_) => false,
        }
    }

    pub fn as_tag(&self) -> Option<&TagSpec> {
        match &self.kind {
            VersionSpecKind::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    pub fn as_range(&self) -> Option<&VersionRange> {
        match &self.kind {
            VersionSpecKind::Range(range) => Some(range),
            _ => None,
        }
    }

    pub(crate) fn parse_range_or_tag(input: &str) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let kind = VersionRange::parse(raw)
            .map(VersionSpecKind::Range)
            .or_else(|_| TagSpec::parse(raw).map(VersionSpecKind::Tag))?;

        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
        })
    }
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl FromStr for VersionSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for VersionSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for VersionSpec {
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
    fn test_range() {
        let spec = VersionSpec::parse("^1.2.3").unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_exact() {
        let spec = VersionSpec::parse("1.2.3").unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!spec.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_tag() {
        let spec = VersionSpec::parse("latest").unwrap();
        assert!(spec.as_tag().is_some());
        assert_eq!(spec.as_tag().unwrap().as_str(), "latest");
        assert!(!spec.matches(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_as_range() {
        let spec = VersionSpec::parse("^1.0.0").unwrap();
        assert!(spec.as_range().is_some());

        let spec = VersionSpec::parse("latest").unwrap();
        assert!(spec.as_range().is_none());
    }

    #[test]
    fn test_display() {
        let spec = VersionSpec::parse("^1.2.3").unwrap();
        assert_eq!(spec.to_string(), "^1.2.3");
    }

    #[test]
    fn test_serde_roundtrip() {
        let spec = VersionSpec::parse("^1.2.3").unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, "\"^1.2.3\"");
        let deserialized: VersionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, spec);
    }

    #[test]
    fn test_empty_error() {
        assert!(VersionSpec::parse("").is_err());
    }
}