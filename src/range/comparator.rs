use std::fmt;

use ecow::EcoVec;
use smol_str::SmolStr;

use crate::Version;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Op {
    Exact,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Caret,
    Tilde,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Comparator {
    pub op: Op,
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre: EcoVec<SmolStr>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComparatorSet {
    pub(crate) comparators: EcoVec<Comparator>,
}

impl Comparator {
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let (op, rest) = if let Some(r) = input.strip_prefix(">=") {
            (Op::GtEq, r)
        } else if let Some(r) = input.strip_prefix("<=") {
            (Op::LtEq, r)
        } else if let Some(r) = input.strip_prefix('>') {
            (Op::Gt, r)
        } else if let Some(r) = input.strip_prefix('<') {
            (Op::Lt, r)
        } else if let Some(r) = input.strip_prefix('^') {
            (Op::Caret, r)
        } else if let Some(r) = input.strip_prefix('~') {
            (Op::Tilde, r)
        } else if let Some(r) = input.strip_prefix('=') {
            (Op::Exact, r)
        } else {
            (Op::Exact, input)
        };

        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }

        let (version_part, pre) = match rest.split_once('-') {
            Some((v, p)) => {
                let v = v.split_once('+').map_or(v, |(v, _)| v);
                (v, p.split('.').map(SmolStr::new).collect())
            }
            None => {
                let v = rest.split_once('+').map_or(rest, |(v, _)| v);
                (v, EcoVec::new())
            }
        };

        let mut parts: [&str; 3] = [""; 3];
        let mut count = 0;
        for part in version_part.split('.') {
            if count >= 3 {
                break;
            }
            parts[count] = part;
            count += 1;
        }
        if count == 0 {
            return None;
        }

        let major = parts[0].parse::<u64>().ok()?;
        let minor = if count > 1 {
            parse_version_part(parts[1])
        } else {
            None
        };
        let patch = if count > 2 {
            parse_version_part(parts[2])
        } else {
            None
        };

        Some(Self {
            op,
            major,
            minor,
            patch,
            pre,
        })
    }

    pub fn matches(&self, version: &Version) -> bool {
        if !version.pre().is_empty() && !self.matches_pre_policy(version) {
            return false;
        }

        match self.op {
            Op::Exact => self.matches_exact(version),
            Op::Gt => self.matches_gt(version),
            Op::GtEq => self.matches_gte(version),
            Op::Lt => self.matches_lt(version),
            Op::LtEq => self.matches_lte(version),
            Op::Caret => self.matches_caret(version),
            Op::Tilde => self.matches_tilde(version),
        }
    }

    fn matches_pre_policy(&self, version: &Version) -> bool {
        if self.pre.is_empty() {
            return false;
        }
        let minor = self.minor.unwrap_or(0);
        let patch = self.patch.unwrap_or(0);
        version.major() == self.major && version.minor() == minor && version.patch() == patch
    }

    fn matches_exact(&self, version: &Version) -> bool {
        if version.major() != self.major {
            return false;
        }
        if let Some(minor) = self.minor
            && version.minor() != minor
        {
            return false;
        }
        if let Some(patch) = self.patch
            && version.patch() != patch
        {
            return false;
        }
        self.cmp_pre(version) == std::cmp::Ordering::Equal
    }

    fn matches_gt(&self, version: &Version) -> bool {
        self.cmp_version(version) == std::cmp::Ordering::Greater
    }

    fn matches_gte(&self, version: &Version) -> bool {
        self.cmp_version(version) != std::cmp::Ordering::Less
    }

    fn matches_lt(&self, version: &Version) -> bool {
        self.cmp_version(version) == std::cmp::Ordering::Less
    }

    fn matches_lte(&self, version: &Version) -> bool {
        self.cmp_version(version) != std::cmp::Ordering::Greater
    }

    fn matches_caret(&self, version: &Version) -> bool {
        if !self.matches_gte(version) {
            return false;
        }

        let minor = self.minor.unwrap_or(0);
        let patch = self.patch.unwrap_or(0);

        if self.major != 0 {
            version.major() < self.major + 1
        } else if minor != 0 {
            version.major() == 0 && version.minor() < minor + 1
        } else {
            version.major() == 0 && version.minor() == 0 && version.patch() < patch + 1
        }
    }

    fn matches_tilde(&self, version: &Version) -> bool {
        if !self.matches_gte(version) {
            return false;
        }

        if version.major() != self.major {
            return false;
        }

        let minor = self.minor.unwrap_or(0);
        if self.minor.is_some() {
            version.minor() == minor
        } else {
            true
        }
    }

    fn cmp_version(&self, version: &Version) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        let major_cmp = version.major().cmp(&self.major);
        if major_cmp != Ordering::Equal {
            return major_cmp;
        }

        let minor = self.minor.unwrap_or(0);
        let minor_cmp = version.minor().cmp(&minor);
        if minor_cmp != Ordering::Equal {
            return minor_cmp;
        }

        let patch = self.patch.unwrap_or(0);
        let patch_cmp = version.patch().cmp(&patch);
        if patch_cmp != Ordering::Equal {
            return patch_cmp;
        }

        self.cmp_pre(version)
    }

    fn cmp_pre(&self, version: &Version) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (version.pre().is_empty(), self.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                let len = version.pre().len().min(self.pre.len());
                for i in 0..len {
                    let ord = cmp_pre_identifier(&version.pre()[i], &self.pre[i]);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                version.pre().len().cmp(&self.pre.len())
            }
        }
    }
}

fn cmp_pre_identifier(a: &SmolStr, b: &SmolStr) -> std::cmp::Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(na), Ok(nb)) => na.cmp(&nb),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

fn parse_version_part(s: &str) -> Option<u64> {
    match s {
        "x" | "X" | "*" => None,
        _ => s.parse::<u64>().ok(),
    }
}

impl ComparatorSet {
    pub fn parse(input: &str) -> Option<Self> {
        let mut comparators = EcoVec::new();
        for token in input.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            comparators.push(Comparator::parse(token)?);
        }
        if comparators.is_empty() {
            return None;
        }
        Some(Self { comparators })
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.comparators.iter().all(|c| c.matches(version))
    }
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            Op::Exact => {}
            Op::Gt => write!(f, ">")?,
            Op::GtEq => write!(f, ">=")?,
            Op::Lt => write!(f, "<")?,
            Op::LtEq => write!(f, "<=")?,
            Op::Caret => write!(f, "^")?,
            Op::Tilde => write!(f, "~")?,
        }
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
            if let Some(patch) = self.patch {
                write!(f, ".{patch}")?;
            }
        }
        if !self.pre.is_empty() {
            write!(f, "-")?;
            for (i, id) in self.pre.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{id}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for ComparatorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, c) in self.comparators.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn test_parse_comparator() {
        let c = Comparator::parse("^1.2.3").unwrap();
        assert_eq!(c.op, Op::Caret);
        assert_eq!(c.major, 1);
        assert_eq!(c.minor, Some(2));
        assert_eq!(c.patch, Some(3));
    }

    #[test]
    fn test_parse_comparator_gte() {
        let c = Comparator::parse(">=1.0.0").unwrap();
        assert_eq!(c.op, Op::GtEq);
        assert_eq!(c.major, 1);
        assert_eq!(c.minor, Some(0));
        assert_eq!(c.patch, Some(0));
    }

    #[test]
    fn test_parse_partial() {
        let c = Comparator::parse(">=1.2").unwrap();
        assert_eq!(c.major, 1);
        assert_eq!(c.minor, Some(2));
        assert_eq!(c.patch, None);
    }

    #[test]
    fn test_parse_with_pre() {
        let c = Comparator::parse(">=1.0.0-alpha.1").unwrap();
        assert_eq!(c.pre.len(), 2);
        assert_eq!(c.pre[0].as_str(), "alpha");
        assert_eq!(c.pre[1].as_str(), "1");
    }

    #[test]
    fn test_caret_major() {
        let c = Comparator::parse("^1.2.3").unwrap();
        assert!(c.matches(&v("1.2.3")));
        assert!(c.matches(&v("1.9.9")));
        assert!(!c.matches(&v("2.0.0")));
        assert!(!c.matches(&v("1.2.2")));
    }

    #[test]
    fn test_caret_zero_major() {
        let c = Comparator::parse("^0.2.3").unwrap();
        assert!(c.matches(&v("0.2.3")));
        assert!(c.matches(&v("0.2.9")));
        assert!(!c.matches(&v("0.3.0")));
        assert!(!c.matches(&v("1.0.0")));
    }

    #[test]
    fn test_caret_zero_minor() {
        let c = Comparator::parse("^0.0.3").unwrap();
        assert!(c.matches(&v("0.0.3")));
        assert!(!c.matches(&v("0.0.4")));
        assert!(!c.matches(&v("0.1.0")));
    }

    #[test]
    fn test_tilde() {
        let c = Comparator::parse("~1.2.3").unwrap();
        assert!(c.matches(&v("1.2.3")));
        assert!(c.matches(&v("1.2.9")));
        assert!(!c.matches(&v("1.3.0")));
        assert!(!c.matches(&v("1.2.2")));
    }

    #[test]
    fn test_gte() {
        let c = Comparator::parse(">=1.2.3").unwrap();
        assert!(c.matches(&v("1.2.3")));
        assert!(c.matches(&v("2.0.0")));
        assert!(!c.matches(&v("1.2.2")));
    }

    #[test]
    fn test_lt() {
        let c = Comparator::parse("<2.0.0").unwrap();
        assert!(c.matches(&v("1.9.9")));
        assert!(!c.matches(&v("2.0.0")));
        assert!(!c.matches(&v("2.0.1")));
    }

    #[test]
    fn test_comparator_set() {
        let set = ComparatorSet::parse(">=1.0.0 <2.0.0").unwrap();
        assert!(set.matches(&v("1.0.0")));
        assert!(set.matches(&v("1.9.9")));
        assert!(!set.matches(&v("2.0.0")));
        assert!(!set.matches(&v("0.9.9")));
    }

    #[test]
    fn test_pre_release_policy() {
        let c = Comparator::parse(">=1.0.0-alpha").unwrap();
        assert!(c.matches(&v("1.0.0-alpha")));
        assert!(c.matches(&v("1.0.0-beta")));
        assert!(c.matches(&v("1.0.0")));
        assert!(!c.matches(&v("1.0.1-alpha")));
    }

    #[test]
    fn test_pre_release_no_pre_in_range() {
        let c = Comparator::parse(">=1.0.0").unwrap();
        assert!(c.matches(&v("1.0.0")));
        assert!(c.matches(&v("2.0.0")));
        assert!(!c.matches(&v("1.0.1-alpha")));
    }

    #[test]
    fn test_display_comparator() {
        assert_eq!(Comparator::parse("^1.2.3").unwrap().to_string(), "^1.2.3");
        assert_eq!(
            Comparator::parse(">=1.0.0").unwrap().to_string(),
            ">=1.0.0"
        );
        assert_eq!(Comparator::parse("~1.2").unwrap().to_string(), "~1.2");
    }

    #[test]
    fn test_display_set() {
        let set = ComparatorSet::parse(">=1.0.0 <2.0.0").unwrap();
        assert_eq!(set.to_string(), ">=1.0.0 <2.0.0");
    }
}
