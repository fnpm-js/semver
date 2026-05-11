mod comparator;
mod kind;

pub(crate) use kind::VersionRangeKind;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::{Version, error::{Error, Result}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VersionRange {
    pub(crate) raw: SmolStr,
    pub(crate) kind: VersionRangeKind,
}

impl VersionRange {
    pub fn parse(input: &str) -> Result<Self> {
        let raw = input.trim();
        let kind = VersionRangeKind::parse(raw)?;
        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
        })
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.kind.matches(version)
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn canonical(&self) -> String {
        self.kind.canonical()
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl FromStr for VersionRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for VersionRange {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for VersionRange {
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
    fn test_basic() {
        let range = VersionRange::parse("^1.0.0").unwrap();
        assert_eq!(range.as_str(), "^1.0.0");
        assert!(range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_canonical() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        assert_eq!(range.canonical(), "^1.2.3");
    }

    #[test]
    fn test_any() {
        let range = VersionRange::parse("*").unwrap();
        assert_eq!(range.canonical(), "*");
        assert!(range.matches(&Version::parse("0.0.1").unwrap()));
        assert!(range.matches(&Version::parse("99.99.99").unwrap()));
    }

    #[test]
    fn test_exact() {
        let range = VersionRange::parse("1.2.3").unwrap();
        assert_eq!(range.as_str(), "1.2.3");
        assert_eq!(range.canonical(), "1.2.3");
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_exact_with_equals() {
        let range = VersionRange::parse("=1.2.3").unwrap();
        assert_eq!(range.as_str(), "=1.2.3");
        assert_eq!(range.canonical(), "1.2.3");
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
    }

    #[test]
    fn test_caret() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_tilde() {
        let range = VersionRange::parse("~1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_gte() {
        let range = VersionRange::parse(">=1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_wildcard_major() {
        let range = VersionRange::parse("1.x").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_wildcard_minor() {
        let range = VersionRange::parse("1.2.x").unwrap();
        assert!(range.matches(&Version::parse("1.2.0").unwrap()));
        assert!(range.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_multiple_comparators() {
        let range = VersionRange::parse(">=1.0.0 <2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_or() {
        let range = VersionRange::parse("^1.0.0 || ^2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(range.matches(&Version::parse("2.3.0").unwrap()));
        assert!(!range.matches(&Version::parse("3.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.9.0").unwrap()));
    }

    #[test]
    fn test_or_with_ranges() {
        let range = VersionRange::parse(">=1.0.0 <1.5.0 || >=2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.4.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(range.matches(&Version::parse("99.0.0").unwrap()));
    }

    #[test]
    fn test_or_exact() {
        let range = VersionRange::parse("1.0.0 || 2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("1.0.1").unwrap()));
        assert!(!range.matches(&Version::parse("3.0.0").unwrap()));
    }

    #[test]
    fn test_empty_error() {
        assert!(VersionRange::parse("").is_err());
        assert!(VersionRange::parse("   ").is_err());
    }

    #[test]
    fn test_unsupported_hyphen() {
        assert!(VersionRange::parse("1.0.0 - 2.0.0").is_err());
    }

    #[test]
    fn test_unsupported_v_prefix() {
        assert!(VersionRange::parse("v1.0.0").is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, "\"^1.2.3\"");

        let deserialized: VersionRange = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, range);
    }

    #[test]
    fn test_display() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        assert_eq!(range.to_string(), "^1.2.3");
    }

    #[test]
    fn test_from_str() {
        let range: VersionRange = "~1.2.3".parse().unwrap();
        assert_eq!(range.as_str(), "~1.2.3");
    }
}
