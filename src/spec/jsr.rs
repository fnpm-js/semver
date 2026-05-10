use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use super::{TagSpec, VersionSpecKind};
use crate::{
    PackageType, VersionRangeKind,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JsrVersionSpec {
    raw: SmolStr,
    kind: VersionSpecKind,
}

impl JsrVersionSpec {
    pub fn parse(input: &str) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let kind = if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Jsr) {
            VersionSpecKind::Range(range)
        } else {
            VersionSpecKind::Tag(TagSpec::parse(raw)?)
        };

        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn kind(&self) -> &VersionSpecKind {
        &self.kind
    }

    pub fn as_version_range(&self) -> Option<&VersionRangeKind> {
        self.kind.as_version_range()
    }

    pub fn is_registry_range(&self) -> bool {
        self.kind.is_registry_range()
    }
}

impl super::VersionSpecTrait for JsrVersionSpec {
    fn matches(&self, version: &crate::Version) -> bool {
        self.kind.matches(version)
    }

    fn raw(&self) -> &str {
        &self.raw
    }

    fn package_type(&self) -> PackageType {
        PackageType::Jsr
    }
}

impl JsrVersionSpec {
    pub fn matches(&self, version: &crate::Version) -> bool {
        <Self as super::VersionSpecTrait>::matches(self, version)
    }

    pub fn package_type(&self) -> PackageType {
        <Self as super::VersionSpecTrait>::package_type(self)
    }
}

impl fmt::Display for JsrVersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for JsrVersionSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for JsrVersionSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for JsrVersionSpec {
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
    use crate::Version;

    #[test]
    fn test_parse_range() {
        let spec = JsrVersionSpec::parse("^1.2.3").unwrap();
        assert!(spec.is_registry_range());
        assert!(spec.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_parse_exact() {
        let spec = JsrVersionSpec::parse("1.2.3").unwrap();
        assert!(spec.is_registry_range());
        assert!(spec.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!spec.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_parse_tag() {
        let spec = JsrVersionSpec::parse("latest").unwrap();
        assert!(!spec.is_registry_range());
        assert!(!spec.matches(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_as_version_range() {
        let spec = JsrVersionSpec::parse("^1.0.0").unwrap();
        assert!(spec.as_version_range().is_some());

        let spec = JsrVersionSpec::parse("latest").unwrap();
        assert!(spec.as_version_range().is_none());
    }

    #[test]
    fn test_display() {
        let spec = JsrVersionSpec::parse("^1.2.3").unwrap();
        assert_eq!(spec.to_string(), "^1.2.3");
    }

    #[test]
    fn test_serde_roundtrip() {
        let spec = JsrVersionSpec::parse("^1.2.3").unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, "\"^1.2.3\"");
        let deserialized: JsrVersionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, spec);
    }

    #[test]
    fn test_package_type() {
        let spec = JsrVersionSpec::parse("^1.0.0").unwrap();
        assert_eq!(spec.package_type(), PackageType::Jsr);
    }
}
