use std::fmt;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
    Version, VersionSpec,
    error::{Error, Result},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageType {
    #[default]
    Npm,
    Jsr,
}

impl PackageType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Jsr => "jsr",
        }
    }
    pub(crate) const fn prefix(&self) -> &'static str {
        match self {
            Self::Npm => "npm:",
            Self::Jsr => "jsr:",
        }
    }
}

impl fmt::Display for PackageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageReq {
    scope: Option<SmolStr>,
    name: SmolStr,
    spec: VersionSpec,
    package_type: PackageType,
}

impl PackageReq {
    #[inline]
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        if let Some(rest) = input.strip_prefix(PackageType::Jsr.prefix()) {
            Self::parse_jsr(rest)
        } else if let Some(rest) = input.strip_prefix(PackageType::Npm.prefix()) {
            Self::parse_npm(rest)
        } else {
            Self::parse_npm(input)
        }
    }

    pub fn parse_npm(input: &str) -> Result<Self> {
        let input = input.trim();
        let (scope, name, spec_str) = split_req(input)?;
        let spec = VersionSpec::parse_npm(spec_str)?;
        Ok(Self {
            scope: scope.map(SmolStr::new),
            name: SmolStr::new(name),
            spec,
            package_type: PackageType::Npm,
        })
    }

    pub fn parse_jsr(input: &str) -> Result<Self> {
        let input = input.trim();
        let (scope, name, spec_str) = split_req(input)?;
        if scope.is_none() {
            return Err(Error::InvalidVersionSpec {
                input: input.to_string(),
            });
        }
        let spec = VersionSpec::parse_jsr(spec_str)?;
        Ok(Self {
            scope: scope.map(SmolStr::new),
            name: SmolStr::new(name),
            spec,
            package_type: PackageType::Jsr,
        })
    }

    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_name(&self) -> SmolStr {
        match &self.scope {
            Some(scope) => SmolStr::new(format!("@{scope}/{}", self.name)),
            None => self.name.clone(),
        }
    }

    pub fn spec(&self) -> &VersionSpec {
        &self.spec
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.spec.matches(version)
    }

    pub fn package_type(&self) -> PackageType {
        self.package_type
    }
}

fn split_req(input: &str) -> Result<(Option<&str>, &str, &str)> {
    if input.is_empty() {
        return Err(Error::InvalidVersionSpec {
            input: input.to_string(),
        });
    }

    if let Some(rest) = input.strip_prefix('@') {
        let slash_index = rest.find('/').ok_or_else(|| Error::InvalidVersionSpec {
            input: input.to_string(),
        })?;
        let scope = &rest[..slash_index];
        let after_slash = &rest[slash_index + 1..];

        if scope.is_empty() || after_slash.is_empty() {
            return Err(Error::InvalidVersionSpec {
                input: input.to_string(),
            });
        }

        match after_slash.find('@') {
            Some(at_index) => {
                let name = &after_slash[..at_index];
                let spec = &after_slash[at_index + 1..];
                if name.is_empty() || spec.is_empty() {
                    return Err(Error::InvalidVersionSpec {
                        input: input.to_string(),
                    });
                }
                Ok((Some(scope), name, spec))
            }
            None => Ok((Some(scope), after_slash, "*")),
        }
    } else {
        match input.find('@') {
            Some(at_index) => {
                let name = &input[..at_index];
                let spec = &input[at_index + 1..];
                if name.is_empty() || spec.is_empty() {
                    return Err(Error::InvalidVersionSpec {
                        input: input.to_string(),
                    });
                }
                Ok((None, name, spec))
            }
            None => Ok((None, input, "*")),
        }
    }
}

impl fmt::Display for PackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            Some(scope) => write!(f, "@{}/{}@{}", scope, self.name, self.spec),
            None => write!(f, "{}@{}", self.name, self.spec),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npm_default() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        assert_eq!(req.scope(), None);
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.spec().as_str(), "^4.0.0");
        assert_eq!(req.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_parse_npm_scoped() {
        let req = PackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.scope(), Some("babel"));
        assert_eq!(req.name(), "core");
        assert_eq!(req.full_name().as_str(), "@babel/core");
        assert_eq!(req.spec().as_str(), "^7.0.0");
        assert_eq!(req.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_parse_npm_protocol() {
        let req = PackageReq::parse("npm:@babel/core@^7.0.0").unwrap();
        assert_eq!(req.scope(), Some("babel"));
        assert_eq!(req.name(), "core");
        assert_eq!(req.spec().as_str(), "^7.0.0");
        assert_eq!(req.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_parse_jsr_protocol() {
        let req = PackageReq::parse("jsr:@std/path@^1.0.0").unwrap();
        assert_eq!(req.scope(), Some("std"));
        assert_eq!(req.name(), "path");
        assert_eq!(req.full_name().as_str(), "@std/path");
        assert_eq!(req.spec().as_str(), "^1.0.0");
        assert_eq!(req.package_type(), PackageType::Jsr);
    }

    #[test]
    fn test_parse_npm_no_version() {
        let req = PackageReq::parse("lodash").unwrap();
        assert_eq!(req.scope(), None);
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.spec().as_str(), "*");
    }

    #[test]
    fn test_parse_npm_scoped_no_version() {
        let req = PackageReq::parse("@types/node").unwrap();
        assert_eq!(req.scope(), Some("types"));
        assert_eq!(req.name(), "node");
        assert_eq!(req.spec().as_str(), "*");
    }

    #[test]
    fn test_parse_jsr_no_version() {
        let req = PackageReq::parse("jsr:@std/fs").unwrap();
        assert_eq!(req.scope(), Some("std"));
        assert_eq!(req.name(), "fs");
        assert_eq!(req.spec().as_str(), "*");
    }

    #[test]
    fn test_parse_npm_tilde() {
        let req = PackageReq::parse("express@~4.17.0").unwrap();
        assert_eq!(req.name(), "express");
        assert_eq!(req.spec().as_str(), "~4.17.0");
    }

    #[test]
    fn test_parse_npm_exact() {
        let req = PackageReq::parse("react@18.2.0").unwrap();
        assert_eq!(req.name(), "react");
        assert_eq!(req.spec().as_str(), "18.2.0");
    }

    #[test]
    fn test_parse_npm_range() {
        let req = PackageReq::parse("typescript@>=4.0.0 <5.0.0").unwrap();
        assert_eq!(req.name(), "typescript");
        assert_eq!(req.spec().as_str(), ">=4.0.0 <5.0.0");
    }

    #[test]
    fn test_parse_empty_fails() {
        assert!(PackageReq::parse("").is_err());
    }

    #[test]
    fn test_parse_jsr_no_scope_fails() {
        assert!(PackageReq::parse_jsr("path@^1.0.0").is_err());
    }

    #[test]
    fn test_parse_jsr_empty_scope_fails() {
        assert!(PackageReq::parse_jsr("@/path@^1.0.0").is_err());
    }

    #[test]
    fn test_matches_npm() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        assert!(req.matches(&Version::parse("4.17.21").unwrap()));
        assert!(!req.matches(&Version::parse("5.0.0").unwrap()));
    }

    #[test]
    fn test_matches_jsr() {
        let req = PackageReq::parse("jsr:@std/path@^1.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_display_npm() {
        let req = PackageReq::parse("react@^18.0.0").unwrap();
        assert_eq!(req.to_string(), "react@^18.0.0");
    }

    #[test]
    fn test_display_npm_scoped() {
        let req = PackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.to_string(), "@babel/core@^7.0.0");
    }

    #[test]
    fn test_display_jsr() {
        let req = PackageReq::parse("jsr:@std/fs@~1.0.0").unwrap();
        assert_eq!(req.to_string(), "@std/fs@~1.0.0");
    }

    #[test]
    fn test_full_name_no_scope() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        assert_eq!(req.full_name().as_str(), "lodash");
    }

    #[test]
    fn test_full_name_with_scope() {
        let req = PackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.full_name().as_str(), "@babel/core");
    }

    #[test]
    fn test_parse_npm_explicit() {
        let req = PackageReq::parse_npm("express@~4.0.0").unwrap();
        assert_eq!(req.name(), "express");
        assert_eq!(req.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_parse_jsr_explicit() {
        let req = PackageReq::parse_jsr("@oak/oak@^12.0.0").unwrap();
        assert_eq!(req.scope(), Some("oak"));
        assert_eq!(req.name(), "oak");
        assert_eq!(req.package_type(), PackageType::Jsr);
    }

    #[test]
    fn test_trim_whitespace() {
        let req = PackageReq::parse("  lodash@^4.0.0  ").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.spec().as_str(), "^4.0.0");
    }

    #[test]
    fn test_as_tag() {
        let req = PackageReq::parse("lodash@latest").unwrap();
        assert!(req.spec().as_tag().is_some());
        assert_eq!(req.spec().as_tag().unwrap().as_str(), "latest");
    }
}
