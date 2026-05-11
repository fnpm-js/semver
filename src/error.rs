use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("empty package version requirement")]
    EmptySpec,
    #[error("invalid version `{input}`")]
    InvalidVersion { input: String },
    #[error("invalid version range `{input}`")]
    InvalidVersionRange { input: String },
    #[error("unsupported range syntax `{input}`")]
    UnsupportedRangeSyntax { input: String },
    #[error("invalid tag `{input}`")]
    InvalidTag { input: String },
    #[error("invalid version spec `{input}`")]
    InvalidVersionSpec { input: String },
}
