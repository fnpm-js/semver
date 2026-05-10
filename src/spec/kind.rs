use crate::{Version, VersionRangeKind};

use super::{AliasSpec, FileSpec, GitSpec, TagSpec, UrlSpec, WorkspaceSpec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VersionSpecKind {
    Range(VersionRangeKind),
    Tag(TagSpec),
    Workspace(WorkspaceSpec),
    File(FileSpec),
    Alias(AliasSpec),
    Git(GitSpec),
    Url(UrlSpec),
}

impl VersionSpecKind {
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Range(range) => range.matches(version),
            _ => false,
        }
    }

    pub fn is_registry_range(&self) -> bool {
        matches!(self, Self::Range(_))
    }

    pub fn as_version_range(&self) -> Option<&VersionRangeKind> {
        match self {
            Self::Range(range) => Some(range),
            _ => None,
        }
    }
}
