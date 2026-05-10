use thiserror::Error;

use crate::package::PackageType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("empty package version requirement")]
    EmptySpec,
    #[error("invalid version `{input}`")]
    InvalidVersion { input: String },
    #[error("invalid version range `{input}` for {package_type}")]
    InvalidVersionRange {
        input: String,
        package_type: PackageType,
    },
    #[error("unsupported range syntax `{input}` for {package_type}")]
    UnsupportedRangeSyntax {
        input: String,
        package_type: PackageType,
    },
    #[error("invalid tag `{input}`")]
    InvalidTag { input: String },
    #[error("invalid version spec `{input}`")]
    InvalidVersionSpec { input: String },
}
