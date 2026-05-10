use crate::{PackageType, Version};

mod comparator;
mod jsr;
mod kind;
mod npm;

pub use jsr::JsrVersionRange;
pub use kind::VersionRangeKind;
pub use npm::NpmVersionRange;

#[allow(dead_code)]
pub(crate) trait VersionRangeTrait {
    fn matches(&self, version: &Version) -> bool;
    fn raw(&self) -> &str;
    fn canonical(&self) -> String;
    fn package_type(&self) -> PackageType;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PackageVersionRange {
    Npm(NpmVersionRange),
    Jsr(JsrVersionRange),
}

impl PackageVersionRange {
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Npm(r) => r.matches(version),
            Self::Jsr(r) => r.matches(version),
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            Self::Npm(r) => r.as_str(),
            Self::Jsr(r) => r.as_str(),
        }
    }

    pub fn canonical(&self) -> String {
        match self {
            Self::Npm(r) => r.canonical(),
            Self::Jsr(r) => r.canonical(),
        }
    }

    pub fn package_type(&self) -> PackageType {
        match self {
            Self::Npm(_) => PackageType::Npm,
            Self::Jsr(_) => PackageType::Jsr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_version_range_npm() {
        let npm_range = NpmVersionRange::parse("^1.0.0").unwrap();
        let pkg_range = PackageVersionRange::Npm(npm_range);

        assert_eq!(pkg_range.raw(), "^1.0.0");
        assert_eq!(pkg_range.package_type(), PackageType::Npm);

        let v1 = Version::parse("1.5.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        assert!(pkg_range.matches(&v1));
        assert!(!pkg_range.matches(&v2));
    }

    #[test]
    fn test_package_version_range_npm_canonical() {
        let range = NpmVersionRange::parse("^1.2.3").unwrap();
        let pkg_range = PackageVersionRange::Npm(range);
        assert_eq!(pkg_range.canonical(), "^1.2.3");
    }

    #[test]
    fn test_package_version_range_npm_any() {
        let range = NpmVersionRange::parse("*").unwrap();
        let pkg_range = PackageVersionRange::Npm(range);

        assert_eq!(pkg_range.canonical(), "*");
        assert!(pkg_range.matches(&Version::parse("0.0.1").unwrap()));
        assert!(pkg_range.matches(&Version::parse("99.99.99").unwrap()));
    }

    #[test]
    fn test_package_version_range_npm_exact() {
        let range = NpmVersionRange::parse("1.2.3").unwrap();
        let pkg_range = PackageVersionRange::Npm(range);

        assert_eq!(pkg_range.raw(), "1.2.3");
        assert_eq!(pkg_range.canonical(), "1.2.3");
        assert!(pkg_range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!pkg_range.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_package_version_range_jsr() {
        let jsr_range = JsrVersionRange::parse("^1.0.0").unwrap();
        let pkg_range = PackageVersionRange::Jsr(jsr_range);

        assert_eq!(pkg_range.raw(), "^1.0.0");
        assert_eq!(pkg_range.package_type(), PackageType::Jsr);

        assert!(pkg_range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!pkg_range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_package_version_range_jsr_canonical() {
        let range = JsrVersionRange::parse("^1.2.3").unwrap();
        let pkg_range = PackageVersionRange::Jsr(range);
        assert_eq!(pkg_range.canonical(), "^1.2.3");
    }

    #[test]
    fn test_package_version_range_jsr_any() {
        let range = JsrVersionRange::parse("*").unwrap();
        let pkg_range = PackageVersionRange::Jsr(range);

        assert_eq!(pkg_range.canonical(), "*");
        assert!(pkg_range.matches(&Version::parse("0.0.1").unwrap()));
        assert!(pkg_range.matches(&Version::parse("99.99.99").unwrap()));
    }
}
