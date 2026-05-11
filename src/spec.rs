mod tag;

pub use tag::TagSpec;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::{
    PackageType, Version, VersionRange,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum VersionSpecKind {
    Range(VersionRange),
    Tag(TagSpec),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VersionSpec {
    raw: SmolStr,
    kind: VersionSpecKind,
    package_type: PackageType,
}

impl VersionSpec {
    #[inline]
    pub fn parse(input: &str, package_type: PackageType) -> Result<Self> {
        match package_type {
            PackageType::Npm => Self::parse_npm(input),
            PackageType::Jsr => Self::parse_jsr(input),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn package_type(&self) -> PackageType {
        self.package_type
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
}

impl VersionSpec {
    pub(crate) fn parse_npm(input: &str) -> Result<Self> {
        Self::parse_range_or_tag(input, PackageType::Npm)
    }

    pub(crate) fn parse_jsr(input: &str) -> Result<Self> {
        Self::parse_range_or_tag(input, PackageType::Jsr)
    }

    fn parse_range_or_tag(input: &str, package_type: PackageType) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let kind = VersionRange::parse(raw, package_type)
            .map(VersionSpecKind::Range)
            .or_else(|_| TagSpec::parse(raw).map(VersionSpecKind::Tag))?;

        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
            package_type,
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
        Self::parse(s, PackageType::Npm)
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
        Self::parse(&value, PackageType::Npm).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_range() {
        let spec = VersionSpec::parse("^1.2.3", PackageType::Npm).unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_npm_exact() {
        let spec = VersionSpec::parse("1.2.3", PackageType::Npm).unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!spec.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_npm_tag() {
        let spec = VersionSpec::parse("latest", PackageType::Npm).unwrap();
        assert!(spec.as_tag().is_some());
        assert_eq!(spec.as_tag().unwrap().as_str(), "latest");
        assert!(!spec.matches(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_npm_as_range() {
        let spec = VersionSpec::parse("^1.0.0", PackageType::Npm).unwrap();
        assert!(spec.as_range().is_some());

        let spec = VersionSpec::parse("latest", PackageType::Npm).unwrap();
        assert!(spec.as_range().is_none());
    }

    #[test]
    fn test_jsr_range() {
        let spec = VersionSpec::parse("^1.2.3", PackageType::Jsr).unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_jsr_exact() {
        let spec = VersionSpec::parse("1.2.3", PackageType::Jsr).unwrap();
        assert!(spec.as_range().is_some());
        assert!(spec.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!spec.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_jsr_tag() {
        let spec = VersionSpec::parse("latest", PackageType::Jsr).unwrap();
        assert!(spec.as_tag().is_some());
    }

    #[test]
    fn test_npm_package_type() {
        let spec = VersionSpec::parse("^1.0.0", PackageType::Npm).unwrap();
        assert_eq!(spec.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_jsr_package_type() {
        let spec = VersionSpec::parse("^1.0.0", PackageType::Jsr).unwrap();
        assert_eq!(spec.package_type(), PackageType::Jsr);
    }

    #[test]
    fn test_display() {
        let spec = VersionSpec::parse("^1.2.3", PackageType::Npm).unwrap();
        assert_eq!(spec.to_string(), "^1.2.3");
    }

    #[test]
    fn test_serde_roundtrip() {
        let spec = VersionSpec::parse("^1.2.3", PackageType::Npm).unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, "\"^1.2.3\"");
        let deserialized: VersionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, spec);
    }

    #[test]
    fn test_npm_empty_error() {
        assert!(VersionSpec::parse("", PackageType::Npm).is_err());
    }

    #[test]
    fn test_jsr_empty_error() {
        assert!(VersionSpec::parse("", PackageType::Jsr).is_err());
    }

    #[test]
    fn test_npm_non_semver_sources_fail() {
        assert!(VersionSpec::parse("workspace:*", PackageType::Npm).is_err());
        assert!(VersionSpec::parse("file:../pkg", PackageType::Npm).is_err());
        assert!(VersionSpec::parse("https://example.com/pkg.tgz", PackageType::Npm).is_err());
    }
}