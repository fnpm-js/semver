mod alias;
mod file;
mod git;
mod jsr;
mod kind;
mod npm;
mod tag;
mod url;
mod workspace;

pub use alias::AliasSpec;
pub use file::FileSpec;
pub use git::GitSpec;
pub use jsr::JsrVersionSpec;
pub use kind::VersionSpecKind;
pub use npm::NpmVersionSpec;
pub use tag::TagSpec;
pub use url::UrlSpec;
pub use workspace::WorkspaceSpec;

use crate::{PackageType, Version};

#[allow(dead_code)]
pub(crate) trait VersionSpecTrait {
    fn matches(&self, version: &Version) -> bool;
    fn raw(&self) -> &str;
    fn package_type(&self) -> PackageType;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PackageVersionSpec {
    Npm(NpmVersionSpec),
    Jsr(JsrVersionSpec),
}

impl PackageVersionSpec {
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Npm(s) => s.matches(version),
            Self::Jsr(s) => s.matches(version),
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            Self::Npm(s) => s.raw(),
            Self::Jsr(s) => s.as_str(),
        }
    }

    pub fn package_type(&self) -> PackageType {
        match self {
            Self::Npm(_) => PackageType::Npm,
            Self::Jsr(_) => PackageType::Jsr,
        }
    }
}

impl From<NpmVersionSpec> for PackageVersionSpec {
    fn from(spec: NpmVersionSpec) -> Self {
        Self::Npm(spec)
    }
}

impl From<JsrVersionSpec> for PackageVersionSpec {
    fn from(spec: JsrVersionSpec) -> Self {
        Self::Jsr(spec)
    }
}
