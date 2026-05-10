mod error;
mod package;
mod range;
mod spec;
mod version;

pub use error::{Error, Result};
pub use package::{JsrPackageReq, NpmPackageReq, PackageReq, PackageType};
pub use range::{JsrVersionRange, NpmVersionRange, PackageVersionRange, VersionRangeKind};
pub use spec::{
    AliasSpec, FileSpec, GitSpec, JsrVersionSpec, NpmVersionSpec, PackageVersionSpec, TagSpec,
    UrlSpec, VersionSpecKind, WorkspaceSpec,
};
pub use version::Version;
