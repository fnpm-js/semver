mod error;
mod package;
mod range;
mod spec;
mod version;

pub use error::{Error, Result};
pub use package::{PackageReq, PackageType};
pub use range::VersionRange;
pub use spec::{
    AliasSpec, FileSpec, GitSpec, TagSpec, UrlSpec, VersionSpec, WorkspaceSpec,
};
pub use version::Version;