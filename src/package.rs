use std::fmt;

use smol_str::SmolStr;

use crate::{
    Version, VersionSpec,
    error::{Error, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageReq {
    scope: Option<SmolStr>,
    name: SmolStr,
    spec: VersionSpec,
}

impl PackageReq {
    #[inline]
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        let (scope, name, spec_str) = split_req(input)?;
        let spec = VersionSpec::parse(spec_str)?;
        Ok(Self {
            scope: scope.map(SmolStr::new),
            name: SmolStr::new(name),
            spec,
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
    }

    #[test]
    fn test_parse_npm_scoped() {
        let req = PackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.scope(), Some("babel"));
        assert_eq!(req.name(), "core");
        assert_eq!(req.full_name().as_str(), "@babel/core");
        assert_eq!(req.spec().as_str(), "^7.0.0");
    }

    #[test]
    fn test_parse_no_version() {
        let req = PackageReq::parse("lodash").unwrap();
        assert_eq!(req.scope(), None);
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.spec().as_str(), "*");
    }

    #[test]
    fn test_parse_scoped_no_version() {
        let req = PackageReq::parse("@types/node").unwrap();
        assert_eq!(req.scope(), Some("types"));
        assert_eq!(req.name(), "node");
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
    fn test_matches() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        assert!(req.matches(&Version::parse("4.17.21").unwrap()));
        assert!(!req.matches(&Version::parse("5.0.0").unwrap()));
    }

    #[test]
    fn test_display() {
        let req = PackageReq::parse("react@^18.0.0").unwrap();
        assert_eq!(req.to_string(), "react@^18.0.0");
    }

    #[test]
    fn test_display_scoped() {
        let req = PackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.to_string(), "@babel/core@^7.0.0");
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