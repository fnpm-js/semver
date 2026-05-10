use std::fmt;

use smol_str::SmolStr;

use super::PackageReqTrait;
use crate::{JsrVersionSpec, PackageType, Version, error::{Error, Result}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JsrPackageReq {
    scope: SmolStr,
    name: SmolStr,
    spec: JsrVersionSpec,
}

impl JsrPackageReq {
    pub fn new(scope: impl Into<SmolStr>, name: impl Into<SmolStr>, spec: JsrVersionSpec) -> Self {
        Self {
            scope: scope.into(),
            name: name.into(),
            spec,
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        let (scope, name, spec_str) = split_jsr_req(input)?;
        let spec = JsrVersionSpec::parse(spec_str)?;
        Ok(Self {
            scope: SmolStr::new(scope),
            name: SmolStr::new(name),
            spec,
        })
    }

    pub fn from_parts(scope: &str, name: &str, spec: &str) -> Result<Self> {
        let spec = JsrVersionSpec::parse(spec)?;
        Ok(Self {
            scope: SmolStr::new(scope),
            name: SmolStr::new(name),
            spec,
        })
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn spec(&self) -> &JsrVersionSpec {
        &self.spec
    }

    pub fn as_str(&self) -> &str {
        self.spec.as_str()
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.spec.matches(version)
    }

    pub fn full_name(&self) -> String {
        format!("@{}/{}", self.scope, self.name)
    }
}

impl PackageReqTrait for JsrPackageReq {
    fn name(&self) -> &str {
        &self.name
    }

    fn version_spec(&self) -> &str {
        self.spec.as_str()
    }

    fn matches(&self, version: &Version) -> bool {
        self.spec.matches(version)
    }

    fn package_type(&self) -> PackageType {
        PackageType::Jsr
    }
}

impl fmt::Display for JsrPackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}/{}@{}", self.scope, self.name, self.spec)
    }
}

fn split_jsr_req(input: &str) -> Result<(&str, &str, &str)> {
    let err = || Error::InvalidVersionSpec {
        input: input.to_string(),
    };

    let rest = input.strip_prefix('@').ok_or_else(err)?;

    let slash_index = rest.find('/').ok_or_else(err)?;
    let scope = &rest[..slash_index];
    let after_slash = &rest[slash_index + 1..];

    if scope.is_empty() || after_slash.is_empty() {
        return Err(err());
    }

    match after_slash.find('@') {
        Some(at_index) => {
            let name = &after_slash[..at_index];
            let spec = &after_slash[at_index + 1..];
            if name.is_empty() || spec.is_empty() {
                return Err(err());
            }
            Ok((scope, name, spec))
        }
        None => Ok((scope, after_slash, "*")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    #[test]
    fn test_parse_basic() {
        let req = JsrPackageReq::parse("@std/path@^1.0.0").unwrap();
        assert_eq!(req.scope(), "std");
        assert_eq!(req.name(), "path");
        assert_eq!(req.as_str(), "^1.0.0");
    }

    #[test]
    fn test_parse_no_version() {
        let req = JsrPackageReq::parse("@std/fs").unwrap();
        assert_eq!(req.scope(), "std");
        assert_eq!(req.name(), "fs");
        assert_eq!(req.as_str(), "*");
    }

    #[test]
    fn test_parse_tilde() {
        let req = JsrPackageReq::parse("@oak/oak@~12.0.0").unwrap();
        assert_eq!(req.scope(), "oak");
        assert_eq!(req.name(), "oak");
        assert_eq!(req.as_str(), "~12.0.0");
    }

    #[test]
    fn test_parse_exact() {
        let req = JsrPackageReq::parse("@std/assert@1.0.0").unwrap();
        assert_eq!(req.scope(), "std");
        assert_eq!(req.name(), "assert");
        assert_eq!(req.as_str(), "1.0.0");
    }

    #[test]
    fn test_parse_missing_at_prefix_fails() {
        assert!(JsrPackageReq::parse("std/path@^1.0.0").is_err());
    }

    #[test]
    fn test_parse_missing_slash_fails() {
        assert!(JsrPackageReq::parse("@stdpath@^1.0.0").is_err());
    }

    #[test]
    fn test_parse_empty_scope_fails() {
        assert!(JsrPackageReq::parse("@/path@^1.0.0").is_err());
    }

    #[test]
    fn test_parse_empty_name_fails() {
        assert!(JsrPackageReq::parse("@std/@^1.0.0").is_err());
    }

    #[test]
    fn test_matches() {
        let req = JsrPackageReq::parse("@std/path@^1.0.0").unwrap();
        let v_match = Version::parse("1.2.3").unwrap();
        let v_no_match = Version::parse("2.0.0").unwrap();
        assert!(req.matches(&v_match));
        assert!(!req.matches(&v_no_match));
    }

    #[test]
    fn test_display() {
        let req = JsrPackageReq::parse("@std/path@^1.0.0").unwrap();
        assert_eq!(req.to_string(), "@std/path@^1.0.0");
    }

    #[test]
    fn test_full_name() {
        let req = JsrPackageReq::parse("@std/path@^1.0.0").unwrap();
        assert_eq!(req.full_name(), "@std/path");
    }

    #[test]
    fn test_from_parts() {
        let req = JsrPackageReq::from_parts("std", "path", "^1.0.0").unwrap();
        assert_eq!(req.scope(), "std");
        assert_eq!(req.name(), "path");
        assert_eq!(req.as_str(), "^1.0.0");
    }

    #[test]
    fn test_trim_whitespace() {
        let req = JsrPackageReq::parse("  @std/path@^1.0.0  ").unwrap();
        assert_eq!(req.scope(), "std");
        assert_eq!(req.name(), "path");
    }
}
