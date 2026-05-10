use crate::{Version, VersionRange};

use super::{AliasSpec, FileSpec, GitSpec, TagSpec, UrlSpec, WorkspaceSpec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VersionSpecKind {
    Range(VersionRange),
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
}