use std::fmt;

use smol_str::SmolStr;

use super::PackageReqTrait;
use crate::{NpmVersionSpec, PackageType, Version, error::{Error, Result}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpmPackageReq {
    name: SmolStr,
    spec: NpmVersionSpec,
}

impl NpmPackageReq {
    pub fn new(name: impl Into<SmolStr>, spec: NpmVersionSpec) -> Self {
        Self {
            name: name.into(),
            spec,
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        let (name, spec_str) = split_npm_req(input)?;
        let spec = NpmVersionSpec::parse(spec_str)?;
        Ok(Self {
            name: SmolStr::new(name),
            spec,
        })
    }

    pub fn from_parts(name: &str, spec: &str) -> Result<Self> {
        let spec = NpmVersionSpec::parse(spec)?;
        Ok(Self {
            name: SmolStr::new(name),
            spec,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn spec(&self) -> &NpmVersionSpec {
        &self.spec
    }

    pub fn version_spec(&self) -> &str {
        self.spec.raw()
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.spec.matches(version)
    }
}

impl PackageReqTrait for NpmPackageReq {
    fn name(&self) -> &str {
        &self.name
    }

    fn version_spec(&self) -> &str {
        self.spec.raw()
    }

    fn matches(&self, version: &Version) -> bool {
        self.spec.matches(version)
    }

    fn package_type(&self) -> PackageType {
        PackageType::Npm
    }
}

impl fmt::Display for NpmPackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.spec)
    }
}

fn split_npm_req(input: &str) -> Result<(&str, &str)> {
    if input.is_empty() {
        return Err(Error::InvalidVersionSpec {
            input: input.to_string(),
        });
    }

    let at_index = if let Some(rest) = input.strip_prefix('@') {
        rest.find('@').map(|i| i + 1)
    } else {
        input.find('@')
    };

    match at_index {
        Some(i) => {
            let name = &input[..i];
            let spec = &input[i + 1..];
            if name.is_empty() || spec.is_empty() {
                return Err(Error::InvalidVersionSpec {
                    input: input.to_string(),
                });
            }
            Ok((name, spec))
        }
        None => Ok((input, "*")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    #[test]
    fn test_parse_simple() {
        let req = NpmPackageReq::parse("lodash@^4.0.0").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.version_spec(), "^4.0.0");
    }

    #[test]
    fn test_parse_scoped() {
        let req = NpmPackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.name(), "@babel/core");
        assert_eq!(req.version_spec(), "^7.0.0");
    }

    #[test]
    fn test_parse_no_version() {
        let req = NpmPackageReq::parse("lodash").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.version_spec(), "*");
    }

    #[test]
    fn test_parse_scoped_no_version() {
        let req = NpmPackageReq::parse("@types/node").unwrap();
        assert_eq!(req.name(), "@types/node");
        assert_eq!(req.version_spec(), "*");
    }

    #[test]
    fn test_parse_tilde() {
        let req = NpmPackageReq::parse("express@~4.17.0").unwrap();
        assert_eq!(req.name(), "express");
        assert_eq!(req.version_spec(), "~4.17.0");
    }

    #[test]
    fn test_parse_exact_version() {
        let req = NpmPackageReq::parse("react@18.2.0").unwrap();
        assert_eq!(req.name(), "react");
        assert_eq!(req.version_spec(), "18.2.0");
    }

    #[test]
    fn test_parse_range() {
        let req = NpmPackageReq::parse("typescript@>=4.0.0 <5.0.0").unwrap();
        assert_eq!(req.name(), "typescript");
        assert_eq!(req.version_spec(), ">=4.0.0 <5.0.0");
    }

    #[test]
    fn test_parse_empty_fails() {
        assert!(NpmPackageReq::parse("").is_err());
    }

    #[test]
    fn test_matches() {
        let req = NpmPackageReq::parse("lodash@^4.0.0").unwrap();
        let v_match = Version::parse("4.17.21").unwrap();
        let v_no_match = Version::parse("5.0.0").unwrap();
        assert!(req.matches(&v_match));
        assert!(!req.matches(&v_no_match));
    }

    #[test]
    fn test_display() {
        let req = NpmPackageReq::parse("@babel/core@^7.0.0").unwrap();
        assert_eq!(req.to_string(), "@babel/core@^7.0.0");
    }

    #[test]
    fn test_from_parts() {
        let req = NpmPackageReq::from_parts("lodash", "^4.0.0").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.version_spec(), "^4.0.0");
    }

    #[test]
    fn test_trim_whitespace() {
        let req = NpmPackageReq::parse("  lodash@^4.0.0  ").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.version_spec(), "^4.0.0");
    }
}
