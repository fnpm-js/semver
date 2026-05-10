use std::fmt;

use serde::{Deserialize, Serialize};

mod jsr;
mod npm;

pub use jsr::JsrPackageReq;
pub use npm::NpmPackageReq;

use crate::Version;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageType {
    #[default]
    Npm,
    Jsr,
}

impl PackageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Jsr => "jsr",
        }
    }
}

impl fmt::Display for PackageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[allow(dead_code)]
pub(crate) trait PackageReqTrait {
    fn name(&self) -> &str;
    fn version_spec(&self) -> &str;
    fn matches(&self, version: &Version) -> bool;
    fn package_type(&self) -> PackageType;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PackageReqKind {
    Npm(NpmPackageReq),
    Jsr(JsrPackageReq),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageReq {
    kind: PackageReqKind,
}

impl PackageReq {
    pub fn parse(input: &str) -> crate::Result<Self> {
        let input = input.trim();
        if let Some(rest) = input.strip_prefix("jsr:") {
            Self::parse_jsr(rest)
        } else if let Some(rest) = input.strip_prefix("npm:") {
            Self::parse_npm(rest)
        } else {
            Self::parse_npm(input)
        }
    }

    pub fn parse_npm(input: &str) -> crate::Result<Self> {
        Ok(Self {
            kind: PackageReqKind::Npm(NpmPackageReq::parse(input)?),
        })
    }

    pub fn parse_jsr(input: &str) -> crate::Result<Self> {
        Ok(Self {
            kind: PackageReqKind::Jsr(JsrPackageReq::parse(input)?),
        })
    }

    pub fn name(&self) -> &str {
        match &self.kind {
            PackageReqKind::Npm(r) => r.name(),
            PackageReqKind::Jsr(r) => r.name(),
        }
    }

    pub fn version_spec(&self) -> &str {
        match &self.kind {
            PackageReqKind::Npm(r) => r.version_spec(),
            PackageReqKind::Jsr(r) => r.as_str(),
        }
    }

    pub fn matches(&self, version: &Version) -> bool {
        match &self.kind {
            PackageReqKind::Npm(r) => r.matches(version),
            PackageReqKind::Jsr(r) => r.matches(version),
        }
    }

    pub fn package_type(&self) -> PackageType {
        match &self.kind {
            PackageReqKind::Npm(_) => PackageType::Npm,
            PackageReqKind::Jsr(_) => PackageType::Jsr,
        }
    }

    pub fn as_npm(&self) -> Option<&NpmPackageReq> {
        match &self.kind {
            PackageReqKind::Npm(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_jsr(&self) -> Option<&JsrPackageReq> {
        match &self.kind {
            PackageReqKind::Jsr(r) => Some(r),
            _ => None,
        }
    }
}

impl fmt::Display for PackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            PackageReqKind::Npm(r) => write!(f, "{r}"),
            PackageReqKind::Jsr(r) => write!(f, "{r}"),
        }
    }
}

impl From<NpmPackageReq> for PackageReq {
    fn from(req: NpmPackageReq) -> Self {
        Self {
            kind: PackageReqKind::Npm(req),
        }
    }
}

impl From<JsrPackageReq> for PackageReq {
    fn from(req: JsrPackageReq) -> Self {
        Self {
            kind: PackageReqKind::Jsr(req),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    #[test]
    fn test_parse_npm_default() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        assert_eq!(req.name(), "lodash");
        assert_eq!(req.version_spec(), "^4.0.0");
        assert_eq!(req.package_type(), PackageType::Npm);
    }

    #[test]
    fn test_parse_npm_protocol() {
        let req = PackageReq::parse("npm:@babel/core@^7.0.0").unwrap();
        assert_eq!(req.name(), "@babel/core");
        assert_eq!(req.version_spec(), "^7.0.0");
        assert_eq!(req.package_type(), PackageType::Npm);
        assert!(req.as_npm().is_some());
        assert!(req.as_jsr().is_none());
    }

    #[test]
    fn test_parse_jsr_protocol() {
        let req = PackageReq::parse("jsr:@std/path@^1.0.0").unwrap();
        assert_eq!(req.name(), "path");
        assert_eq!(req.version_spec(), "^1.0.0");
        assert_eq!(req.package_type(), PackageType::Jsr);
        assert!(req.as_jsr().is_some());
        assert!(req.as_npm().is_none());
    }

    #[test]
    fn test_matches_npm() {
        let req = PackageReq::parse("lodash@^4.0.0").unwrap();
        let v_match = Version::parse("4.17.21").unwrap();
        let v_no_match = Version::parse("5.0.0").unwrap();
        assert!(req.matches(&v_match));
        assert!(!req.matches(&v_no_match));
    }

    #[test]
    fn test_matches_jsr() {
        let req = PackageReq::parse("jsr:@std/path@^1.0.0").unwrap();
        let v_match = Version::parse("1.5.0").unwrap();
        let v_no_match = Version::parse("2.0.0").unwrap();
        assert!(req.matches(&v_match));
        assert!(!req.matches(&v_no_match));
    }

    #[test]
    fn test_display_npm() {
        let req = PackageReq::parse("react@^18.0.0").unwrap();
        assert_eq!(req.to_string(), "react@^18.0.0");
    }

    #[test]
    fn test_display_jsr() {
        let req = PackageReq::parse("jsr:@std/fs@~1.0.0").unwrap();
        assert_eq!(req.to_string(), "@std/fs@~1.0.0");
    }

    #[test]
    fn test_from_npm_package_req() {
        let npm_req = NpmPackageReq::parse("lodash@^4.0.0").unwrap();
        let req: PackageReq = npm_req.into();
        assert_eq!(req.package_type(), PackageType::Npm);
        assert_eq!(req.name(), "lodash");
    }

    #[test]
    fn test_from_jsr_package_req() {
        let jsr_req = JsrPackageReq::parse("@std/path@^1.0.0").unwrap();
        let req: PackageReq = jsr_req.into();
        assert_eq!(req.package_type(), PackageType::Jsr);
        assert_eq!(req.name(), "path");
    }

    #[test]
    fn test_as_npm() {
        let req = PackageReq::parse("npm:lodash@^4.0.0").unwrap();
        let npm = req.as_npm().unwrap();
        assert_eq!(npm.name(), "lodash");
    }

    #[test]
    fn test_as_jsr() {
        let req = PackageReq::parse("jsr:@std/path@^1.0.0").unwrap();
        let jsr = req.as_jsr().unwrap();
        assert_eq!(jsr.scope(), "std");
        assert_eq!(jsr.name(), "path");
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
        assert_eq!(req.name(), "oak");
        assert_eq!(req.package_type(), PackageType::Jsr);
    }
}
