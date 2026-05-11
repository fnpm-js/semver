use std::fmt;

use ecow::EcoVec;

use super::comparator::ComparatorSet;
use crate::{Version, error::{Error, Result}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VersionRangeKind {
    Any,
    Exact(Version),
    Range(EcoVec<ComparatorSet>),
}

impl VersionRangeKind {
    pub fn parse(input: &str) -> Result<Self> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(Error::EmptySpec);
        }
        if raw.contains(" - ") || raw.starts_with('v') {
            return Err(Error::UnsupportedRangeSyntax {
                input: input.to_string(),
            });
        }
        if raw == "*" {
            return Ok(Self::Any);
        }

        let exact = raw.strip_prefix('=').unwrap_or(raw);
        if !raw.contains("||") && is_full_version(exact) {
            let version = Version::parse(exact)?;
            return Ok(Self::Exact(version));
        }

        let mut sets = EcoVec::new();
        for segment in raw.split("||") {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }
            let set =
                ComparatorSet::parse(segment).ok_or_else(|| Error::InvalidVersionRange {
                    input: input.to_string(),
                })?;
            sets.push(set);
        }

        if sets.is_empty() {
            return Err(Error::InvalidVersionRange {
                input: input.to_string(),
            });
        }

        Ok(Self::Range(sets))
    }

    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(exact) => exact == version,
            Self::Range(sets) => sets.iter().any(|set| set.matches(version)),
        }
    }

    pub fn canonical(&self) -> String {
        match self {
            Self::Any => "*".to_string(),
            Self::Exact(version) => version.to_string(),
            Self::Range(sets) => {
                let mut result = String::new();
                for (i, set) in sets.iter().enumerate() {
                    if i > 0 {
                        result.push_str(" || ");
                    }
                    result.push_str(&set.to_string());
                }
                result
            }
        }
    }
}

impl fmt::Display for VersionRangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

fn is_full_version(input: &str) -> bool {
    let core = input.split_once('-').map_or(input, |(core, _)| core);
    let core = core.split_once('+').map_or(core, |(core, _)| core);
    let Some((major, rest)) = core.split_once('.') else {
        return false;
    };
    let Some((minor, patch)) = rest.split_once('.') else {
        return false;
    };
    !patch.contains('.')
        && is_numeric_identifier(major)
        && is_numeric_identifier(minor)
        && is_numeric_identifier(patch)
}

fn is_numeric_identifier(input: &str) -> bool {
    !input.is_empty() && input.chars().all(|ch| ch.is_ascii_digit())
}
