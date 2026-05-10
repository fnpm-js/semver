use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use super::kind::VersionRangeKind;
use crate::{
    PackageType, Version,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JsrVersionRange {
    raw: SmolStr,
    kind: VersionRangeKind,
}

impl JsrVersionRange {
    pub fn parse(input: &str) -> Result<Self> {
        let raw = input.trim();
        let kind = VersionRangeKind::parse(raw, PackageType::Jsr)?;
        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl super::VersionRangeTrait for JsrVersionRange {
    fn matches(&self, version: &Version) -> bool {
        self.kind.matches(version)
    }

    fn raw(&self) -> &str {
        self.as_str()
    }

    fn canonical(&self) -> String {
        self.kind.canonical()
    }

    fn package_type(&self) -> PackageType {
        PackageType::Jsr
    }
}

impl JsrVersionRange {
    pub fn matches(&self, version: &Version) -> bool {
        <Self as super::VersionRangeTrait>::matches(self, version)
    }

    pub fn canonical(&self) -> String {
        <Self as super::VersionRangeTrait>::canonical(self)
    }

    pub fn package_type(&self) -> PackageType {
        <Self as super::VersionRangeTrait>::package_type(self)
    }
}

impl fmt::Display for JsrVersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for JsrVersionRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for JsrVersionRange {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for JsrVersionRange {
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
    fn test_parse_any() {
        let range = JsrVersionRange::parse("*").unwrap();
        assert_eq!(range.as_str(), "*");
        assert_eq!(range.canonical(), "*");
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("999.999.999").unwrap()));
    }

    #[test]
    fn test_parse_exact() {
        let range = JsrVersionRange::parse("1.2.3").unwrap();
        assert_eq!(range.canonical(), "1.2.3");
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_parse_caret() {
        let range = JsrVersionRange::parse("^1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_parse_tilde() {
        let range = JsrVersionRange::parse("~1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_parse_gte() {
        let range = JsrVersionRange::parse(">=1.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_parse_multiple_comparators() {
        let range = JsrVersionRange::parse(">=1.0.0 <2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_parse_wildcard() {
        let range = JsrVersionRange::parse("1.x").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_parse_empty_error() {
        assert!(JsrVersionRange::parse("").is_err());
        assert!(JsrVersionRange::parse("   ").is_err());
    }

    #[test]
    fn test_parse_or() {
        let range = JsrVersionRange::parse("^1.0.0 || ^2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(range.matches(&Version::parse("2.3.0").unwrap()));
        assert!(!range.matches(&Version::parse("3.0.0").unwrap()));
    }

    #[test]
    fn test_package_type() {
        let range = JsrVersionRange::parse("^1.0.0").unwrap();
        assert_eq!(range.package_type(), PackageType::Jsr);
    }

    #[test]
    fn test_serde_roundtrip() {
        let range = JsrVersionRange::parse("^1.2.3").unwrap();
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, "\"^1.2.3\"");
        let deserialized: JsrVersionRange = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, range);
    }

    #[test]
    fn test_display() {
        let range = JsrVersionRange::parse("^1.2.3").unwrap();
        assert_eq!(range.to_string(), "^1.2.3");
    }

    #[test]
    fn test_from_str() {
        let range: JsrVersionRange = "~1.2.3".parse().unwrap();
        assert_eq!(range.as_str(), "~1.2.3");
    }
}
