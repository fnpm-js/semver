mod alias;
mod file;
mod git;
mod kind;
mod tag;
mod url;
mod workspace;

pub use alias::AliasSpec;
pub use file::FileSpec;
pub use git::GitSpec;
pub use tag::TagSpec;
pub use url::UrlSpec;
pub use workspace::WorkspaceSpec;

pub(crate) use kind::VersionSpecKind;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::{
    PackageType, Version, VersionRange,
    error::{Error, Result},
    range::VersionRangeKind,
};

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

    pub fn kind(&self) -> &VersionSpecKind {
        &self.kind
    }

    pub fn package_type(&self) -> PackageType {
        self.package_type
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.kind.matches(version)
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
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let package_type = PackageType::Npm;
        let err = || Error::InvalidVersionSpec {
            input: input.to_string(),
        };
        let kind = if let Some(rest) = raw.strip_prefix("workspace:") {
            if rest.is_empty() { return Err(err()); }
            VersionSpecKind::Workspace(WorkspaceSpec::new(rest))
        } else if let Some(rest) = raw.strip_prefix("file:") {
            if rest.is_empty() { return Err(err()); }
            VersionSpecKind::File(FileSpec::new(rest))
        } else if let Some(rest) = raw.strip_prefix("npm:") {
            if rest.is_empty() { return Err(err()); }
            VersionSpecKind::Alias(parse_alias(rest, input)?)
        } else if is_git_spec(raw) {
            VersionSpecKind::Git(GitSpec::new(raw))
        } else if is_url_spec(raw) {
            VersionSpecKind::Url(UrlSpec::parse(raw).map_err(|_| err())?)
        } else {
            try_parse_range_or_tag(raw, package_type)?
        };

        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
            package_type,
        })
    }

    pub(crate) fn parse_jsr(input: &str) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let package_type = PackageType::Jsr;
        let kind = try_parse_range_or_tag(raw, package_type)?;

        Ok(Self {
            raw: SmolStr::new(raw),
            kind,
            package_type,
        })
    }
}

fn try_parse_range_or_tag(raw: &str, package_type: PackageType) -> Result<VersionSpecKind> {
    if let Ok(range_kind) = VersionRangeKind::parse(raw, package_type) {
        Ok(VersionSpecKind::Range(VersionRange {
            raw: SmolStr::new(raw),
            kind: range_kind,
            package_type,
        }))
    } else {
        Ok(VersionSpecKind::Tag(TagSpec::parse(raw)?))
    }
}

fn parse_alias(value: &str, input: &str) -> Result<AliasSpec> {
    if value.is_empty() {
        return Err(Error::InvalidVersionSpec {
            input: input.to_string(),
        });
    }

    let at_index = if let Some(stripped) = value.strip_prefix('@') {
        stripped.find('@').map(|index| index + 1)
    } else {
        value.find('@')
    };

    let Some(at_index) = at_index else {
        return Err(Error::InvalidVersionSpec {
            input: input.to_string(),
        });
    };

    let (package, req) = value.split_at(at_index);
    let req = &req[1..];
    if package.is_empty() || req.is_empty() {
        return Err(Error::InvalidVersionSpec {
            input: input.to_string(),
        });
    }

    let req = VersionSpec::parse_npm(req)?;
    Ok(AliasSpec::new(package, req))
}

fn is_git_spec(input: &str) -> bool {
    input.starts_with("git:")
        || input.starts_with("git+ssh:")
        || input.starts_with("git+https:")
        || input.starts_with("ssh:")
        || input.starts_with("github:")
        || input.starts_with("gitlab:")
        || input.starts_with("bitbucket:")
}

fn is_url_spec(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
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
}