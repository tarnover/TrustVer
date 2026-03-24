use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// VersionError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("invalid authorship tag: {0}")]
    InvalidAuthorship(String),
    #[error("invalid version string: {0}")]
    InvalidVersion(String),
}

// ---------------------------------------------------------------------------
// AuthorshipTag
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorshipTag {
    H,
    Ai,
    Hrai,
    Aih,
    Auto,
    Mix,
}

impl FromStr for AuthorshipTag {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "h" => Ok(AuthorshipTag::H),
            "ai" => Ok(AuthorshipTag::Ai),
            "hrai" => Ok(AuthorshipTag::Hrai),
            "aih" => Ok(AuthorshipTag::Aih),
            "auto" => Ok(AuthorshipTag::Auto),
            "mix" => Ok(AuthorshipTag::Mix),
            other => Err(VersionError::InvalidAuthorship(other.to_string())),
        }
    }
}

impl fmt::Display for AuthorshipTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AuthorshipTag::H => "h",
            AuthorshipTag::Ai => "ai",
            AuthorshipTag::Hrai => "hrai",
            AuthorshipTag::Aih => "aih",
            AuthorshipTag::Auto => "auto",
            AuthorshipTag::Mix => "mix",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// BumpLevel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpLevel {
    Macro,
    Meso,
    Micro,
}

// ---------------------------------------------------------------------------
// TrustVersion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustVersion {
    pub macro_ver: u64,
    pub meso: u64,
    pub micro: u64,
    pub pre_release: Option<String>,
    pub authorship: AuthorshipTag,
}

impl TrustVersion {
    /// Bump to the next version at the given level, resetting lower segments
    /// and clearing any pre-release label.
    pub fn bump(&self, level: BumpLevel, authorship: AuthorshipTag) -> TrustVersion {
        let (macro_ver, meso, micro) = match level {
            BumpLevel::Macro => (self.macro_ver + 1, 0, 0),
            BumpLevel::Meso => (self.macro_ver, self.meso + 1, 0),
            BumpLevel::Micro => (self.macro_ver, self.meso, self.micro + 1),
        };
        TrustVersion {
            macro_ver,
            meso,
            micro,
            pre_release: None,
            authorship,
        }
    }
}

impl FromStr for TrustVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Must contain exactly one `+` separator
        let plus_pos = s
            .rfind('+')
            .ok_or_else(|| VersionError::InvalidVersion(s.to_string()))?;

        let version_part = &s[..plus_pos];
        let auth_part = &s[plus_pos + 1..];

        if version_part.is_empty() {
            return Err(VersionError::InvalidVersion(s.to_string()));
        }

        // Parse authorship
        let authorship = auth_part
            .parse::<AuthorshipTag>()
            .map_err(|_| VersionError::InvalidVersion(s.to_string()))?;

        // Split version_part into numeric part and optional pre-release
        // Pre-release starts at the first `-` after the third numeric component.
        // Grammar: MACRO.MESO.MICRO[-PRE_RELEASE]
        let (numeric_part, pre_release) = split_pre_release(version_part)
            .ok_or_else(|| VersionError::InvalidVersion(s.to_string()))?;

        // Parse numeric segments
        let segments: Vec<&str> = numeric_part.splitn(4, '.').collect();
        if segments.len() != 3 {
            return Err(VersionError::InvalidVersion(s.to_string()));
        }

        let macro_ver = parse_segment(segments[0])
            .ok_or_else(|| VersionError::InvalidVersion(s.to_string()))?;
        let meso = parse_segment(segments[1])
            .ok_or_else(|| VersionError::InvalidVersion(s.to_string()))?;
        let micro = parse_segment(segments[2])
            .ok_or_else(|| VersionError::InvalidVersion(s.to_string()))?;

        Ok(TrustVersion {
            macro_ver,
            meso,
            micro,
            pre_release,
            authorship,
        })
    }
}

impl fmt::Display for TrustVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.macro_ver, self.meso, self.micro)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        write!(f, "+{}", self.authorship)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a single numeric segment, rejecting leading zeros and non-numeric input.
fn parse_segment(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    // Reject leading zeros on multi-digit numbers
    if s.len() > 1 && s.starts_with('0') {
        return None;
    }
    s.parse::<u64>().ok()
}

/// Split `version_part` (everything before `+`) into the dot-numeric portion
/// and an optional pre-release string (without the leading `-`).
///
/// The pre-release begins at the first `-` that follows the third dot-separated
/// numeric segment.  Returns `None` if the string is structurally invalid.
fn split_pre_release(version_part: &str) -> Option<(String, Option<String>)> {
    // Find the third dot to know where MICRO ends
    let mut dot_count = 0usize;
    let mut micro_end: Option<usize> = None;
    for (i, ch) in version_part.char_indices() {
        if ch == '.' {
            dot_count += 1;
            if dot_count == 2 {
                // The MICRO segment starts after this dot; find where it ends
                // (at `-` or end-of-string)
                let rest = &version_part[i + 1..];
                if let Some(dash_pos) = rest.find('-') {
                    micro_end = Some(i + 1 + dash_pos);
                } else {
                    // No pre-release
                    micro_end = Some(version_part.len());
                }
                break;
            }
        }
    }

    let micro_end = micro_end?;

    if micro_end == version_part.len() {
        // No pre-release
        Some((version_part.to_string(), None))
    } else {
        let numeric = version_part[..micro_end].to_string();
        let pre = version_part[micro_end + 1..].to_string(); // skip the `-`
        if pre.is_empty() {
            None
        } else {
            Some((numeric, Some(pre)))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_authorship_tags() {
        assert_eq!("h".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::H);
        assert_eq!("ai".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::Ai);
        assert_eq!("hrai".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::Hrai);
        assert_eq!("aih".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::Aih);
        assert_eq!("auto".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::Auto);
        assert_eq!("mix".parse::<AuthorshipTag>().unwrap(), AuthorshipTag::Mix);
    }

    #[test]
    fn authorship_tag_display_roundtrip() {
        for tag in [AuthorshipTag::H, AuthorshipTag::Ai, AuthorshipTag::Hrai,
                     AuthorshipTag::Aih, AuthorshipTag::Auto, AuthorshipTag::Mix] {
            let s = tag.to_string();
            assert_eq!(s.parse::<AuthorshipTag>().unwrap(), tag);
        }
    }

    #[test]
    fn reject_invalid_authorship_tag() {
        assert!("unknown".parse::<AuthorshipTag>().is_err());
        assert!("".parse::<AuthorshipTag>().is_err());
        assert!("H".parse::<AuthorshipTag>().is_err());
    }

    #[test]
    fn parse_valid_versions() {
        let v: TrustVersion = "2.4.0+hrai".parse().unwrap();
        assert_eq!(v.macro_ver, 2);
        assert_eq!(v.meso, 4);
        assert_eq!(v.micro, 0);
        assert_eq!(v.pre_release, None);
        assert_eq!(v.authorship, AuthorshipTag::Hrai);

        let v: TrustVersion = "0.0.0+h".parse().unwrap();
        assert_eq!(v.macro_ver, 0);

        let v: TrustVersion = "1.0.0+auto".parse().unwrap();
        assert_eq!(v.authorship, AuthorshipTag::Auto);
    }

    #[test]
    fn parse_version_with_prerelease() {
        let v: TrustVersion = "2.4.0-rc.1+hrai".parse().unwrap();
        assert_eq!(v.macro_ver, 2);
        assert_eq!(v.meso, 4);
        assert_eq!(v.micro, 0);
        assert_eq!(v.pre_release.as_deref(), Some("rc.1"));
        assert_eq!(v.authorship, AuthorshipTag::Hrai);
    }

    #[test]
    fn version_display_roundtrip() {
        for input in ["2.4.0+hrai", "0.0.0+h", "1.0.0+auto", "3.1.1-beta.2+mix"] {
            let v: TrustVersion = input.parse().unwrap();
            assert_eq!(v.to_string(), input);
        }
    }

    #[test]
    fn reject_invalid_versions() {
        assert!("2.4.0".parse::<TrustVersion>().is_err());          // missing authorship
        assert!("01.4.0+h".parse::<TrustVersion>().is_err());       // leading zero
        assert!("2.04.0+h".parse::<TrustVersion>().is_err());       // leading zero
        assert!("2.4.0+unknown".parse::<TrustVersion>().is_err());  // bad tag
        assert!("2.4+h".parse::<TrustVersion>().is_err());          // missing micro
        assert!("+h".parse::<TrustVersion>().is_err());              // no version
        assert!("".parse::<TrustVersion>().is_err());                // empty
        assert!("abc+h".parse::<TrustVersion>().is_err());          // non-numeric
    }

    #[test]
    fn bump_macro() {
        let v: TrustVersion = "1.2.3+h".parse().unwrap();
        let bumped = v.bump(BumpLevel::Macro, AuthorshipTag::Hrai);
        assert_eq!(bumped.to_string(), "2.0.0+hrai");
    }

    #[test]
    fn bump_meso() {
        let v: TrustVersion = "1.2.3+h".parse().unwrap();
        let bumped = v.bump(BumpLevel::Meso, AuthorshipTag::Ai);
        assert_eq!(bumped.to_string(), "1.3.0+ai");
    }

    #[test]
    fn bump_micro() {
        let v: TrustVersion = "1.2.3+h".parse().unwrap();
        let bumped = v.bump(BumpLevel::Micro, AuthorshipTag::Mix);
        assert_eq!(bumped.to_string(), "1.2.4+mix");
    }

    #[test]
    fn bump_clears_prerelease() {
        let v: TrustVersion = "1.2.3-rc.1+h".parse().unwrap();
        let bumped = v.bump(BumpLevel::Micro, AuthorshipTag::H);
        assert_eq!(bumped.to_string(), "1.2.4+h");
    }
}
