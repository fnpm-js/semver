use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use super::{AliasSpec, FileSpec, GitSpec, TagSpec, UrlSpec, VersionSpecKind, WorkspaceSpec};
use crate::{
    PackageType, VersionRangeKind,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpmVersionSpec {
    raw: SmolStr,
    kind: VersionSpecKind,
}

impl NpmVersionSpec {
    pub fn new(raw: impl Into<SmolStr>, kind: VersionSpecKind) -> Self {
        Self {
            raw: raw.into(),
            kind,
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        Self::parse_npm(input)
    }

    pub fn raw(&self) -> &str {
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

impl super::VersionSpecTrait for NpmVersionSpec {
    fn matches(&self, version: &crate::Version) -> bool {
        self.kind.matches(version)
    }

    fn raw(&self) -> &str {
        &self.raw
    }

    fn package_type(&self) -> PackageType {
        PackageType::Npm
    }
}

impl NpmVersionSpec {
    pub fn matches(&self, version: &crate::Version) -> bool {
        <Self as super::VersionSpecTrait>::matches(self, version)
    }

    pub fn package_type(&self) -> PackageType {
        <Self as super::VersionSpecTrait>::package_type(self)
    }
}

impl NpmVersionSpec {
    pub(crate) fn parse_npm(input: &str) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }

        let kind = match raw.as_bytes()[0] {
            b'^' | b'~' | b'>' | b'<' | b'=' | b'0'..=b'9' | b'*' => {
                if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'w' => {
                if let Some(value) = raw.strip_prefix("workspace:") {
                    if value.is_empty() {
                        return Err(Error::InvalidVersionSpec {
                            input: input.to_string(),
                        });
                    }
                    VersionSpecKind::Workspace(WorkspaceSpec::new(value))
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'f' => {
                if let Some(value) = raw.strip_prefix("file:") {
                    if value.is_empty() {
                        return Err(Error::InvalidVersionSpec {
                            input: input.to_string(),
                        });
                    }
                    VersionSpecKind::File(FileSpec::new(value))
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'n' => {
                if let Some(value) = raw.strip_prefix("npm:") {
                    VersionSpecKind::Alias(parse_alias(value, input)?)
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'g' | b's' => {
                if is_git_spec(raw) {
                    VersionSpecKind::Git(GitSpec::new(raw))
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'b' => {
                if raw.starts_with("bitbucket:") {
                    VersionSpecKind::Git(GitSpec::new(raw))
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            b'h' => {
                if is_url_spec(raw) {
                    VersionSpecKind::Url(UrlSpec::parse(raw).map_err(|_| {
                        Error::InvalidVersionSpec {
                            input: input.to_string(),
                        }
                    })?)
                } else if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
            _ => {
                if let Ok(range) = VersionRangeKind::parse(raw, PackageType::Npm) {
                    VersionSpecKind::Range(range)
                } else {
                    VersionSpecKind::Tag(TagSpec::parse(raw)?)
                }
            }
        };

        Ok(Self::new(raw, kind))
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

    let req = NpmVersionSpec::parse_npm(req)?;
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

impl fmt::Display for NpmVersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.raw())
    }
}

impl FromStr for NpmVersionSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for NpmVersionSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.raw())
    }
}

impl<'de> Deserialize<'de> for NpmVersionSpec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}
