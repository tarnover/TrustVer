use std::collections::HashMap;

use thiserror::Error;

use crate::version::AuthorshipTag;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub tag: Option<AuthorshipTag>,
    pub lines_changed: u64,
    pub has_reviewer: bool,
}

#[derive(Debug, Clone)]
pub struct DerivationSummary {
    pub total_commits: usize,
    pub untagged_commits: usize,
    pub total_lines: u64,
    pub tag_weights: HashMap<AuthorshipTag, f64>,
}

#[derive(Debug, Clone)]
pub struct DerivationResult {
    pub tag: AuthorshipTag,
    pub summary: DerivationSummary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DeriveError {
    #[error("strict mode: {count} untagged commit(s) found")]
    UntaggedCommits { count: usize },
}

// ---------------------------------------------------------------------------
// derive_authorship
// ---------------------------------------------------------------------------

pub fn derive_authorship(
    commits: &[CommitInfo],
    strict: bool,
) -> Result<DerivationResult, DeriveError> {
    let mut warnings = Vec::new();

    // 1. Empty commits → Mix with warning
    if commits.is_empty() {
        warnings.push("No commits provided; defaulting to Mix.".to_string());
        return Ok(DerivationResult {
            tag: AuthorshipTag::Mix,
            summary: DerivationSummary {
                total_commits: 0,
                untagged_commits: 0,
                total_lines: 0,
                tag_weights: HashMap::new(),
            },
            warnings,
        });
    }

    // 2. Count untagged
    let untagged_commits = commits.iter().filter(|c| c.tag.is_none()).count();

    if strict && untagged_commits > 0 {
        return Err(DeriveError::UntaggedCommits {
            count: untagged_commits,
        });
    }

    // 3. Warn about untagged in lenient mode
    if untagged_commits > 0 {
        warnings.push(format!(
            "{} untagged commit(s) treated as Mix.",
            untagged_commits
        ));
    }

    // 4. Calculate weighted totals
    let total_lines: u64 = commits.iter().map(|c| c.lines_changed).sum();

    // 5. Track all_ai_have_reviewer (only for Ai and Hrai, NOT Auto)
    let mut all_ai_have_reviewer = true;
    let mut has_ai_or_hrai = false;

    let mut weighted: HashMap<AuthorshipTag, u64> = HashMap::new();
    for commit in commits {
        let effective_tag = commit.tag.unwrap_or(AuthorshipTag::Mix);
        *weighted.entry(effective_tag).or_insert(0) += commit.lines_changed;

        match effective_tag {
            AuthorshipTag::Ai | AuthorshipTag::Hrai => {
                has_ai_or_hrai = true;
                if !commit.has_reviewer {
                    all_ai_have_reviewer = false;
                }
            }
            _ => {}
        }
    }

    // If there are no Ai/Hrai commits at all, the reviewer condition is vacuously
    // false (nothing qualifies for Hrai derivation via that path).
    if !has_ai_or_hrai {
        all_ai_have_reviewer = false;
    }

    // 6. If total_lines == 0 → Mix
    if total_lines == 0 {
        let tag_weights = weighted.keys().map(|k| (*k, 0.0_f64)).collect();
        return Ok(DerivationResult {
            tag: AuthorshipTag::Mix,
            summary: DerivationSummary {
                total_commits: commits.len(),
                untagged_commits,
                total_lines: 0,
                tag_weights,
            },
            warnings,
        });
    }

    // 7. Calculate percentages
    let total = total_lines as f64;
    let pct = |tag: AuthorshipTag| -> f64 {
        weighted.get(&tag).copied().unwrap_or(0) as f64 / total * 100.0
    };

    let h_pct = pct(AuthorshipTag::H);
    let ai_pct = pct(AuthorshipTag::Ai);
    let hrai_pct = pct(AuthorshipTag::Hrai);
    let aih_pct = pct(AuthorshipTag::Aih);
    let auto_pct = pct(AuthorshipTag::Auto);

    let tag_weights: HashMap<AuthorshipTag, f64> = [
        AuthorshipTag::H,
        AuthorshipTag::Ai,
        AuthorshipTag::Hrai,
        AuthorshipTag::Aih,
        AuthorshipTag::Auto,
        AuthorshipTag::Mix,
    ]
    .iter()
    .map(|&t| (t, pct(t)))
    .collect();

    // 8. Apply threshold rules in order
    let derived = if h_pct >= 95.0 {
        AuthorshipTag::H
    } else if (ai_pct + hrai_pct) >= 80.0 && all_ai_have_reviewer {
        AuthorshipTag::Hrai
    } else if aih_pct >= 80.0 {
        AuthorshipTag::Aih
    } else if auto_pct >= 80.0 {
        AuthorshipTag::Auto
    } else if (ai_pct + auto_pct) >= 80.0 {
        AuthorshipTag::Ai
    } else {
        AuthorshipTag::Mix
    };

    Ok(DerivationResult {
        tag: derived,
        summary: DerivationSummary {
            total_commits: commits.len(),
            untagged_commits,
            total_lines,
            tag_weights,
        },
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::AuthorshipTag;

    fn commit(tag: Option<AuthorshipTag>, lines: u64, has_reviewer: bool) -> CommitInfo {
        CommitInfo {
            tag,
            lines_changed: lines,
            has_reviewer,
        }
    }

    #[test]
    fn all_human_derives_h() {
        let commits = vec![
            commit(Some(AuthorshipTag::H), 100, false),
            commit(Some(AuthorshipTag::H), 200, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::H);
    }

    #[test]
    fn ai_with_review_derives_hrai() {
        let commits = vec![
            commit(Some(AuthorshipTag::Hrai), 400, true),
            commit(Some(AuthorshipTag::H), 50, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Hrai);
    }

    #[test]
    fn ai_without_review_derives_ai() {
        let commits = vec![
            commit(Some(AuthorshipTag::Ai), 400, false),
            commit(Some(AuthorshipTag::H), 50, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Ai);
    }

    #[test]
    fn auto_derives_auto() {
        let commits = vec![
            commit(Some(AuthorshipTag::Auto), 400, false),
            commit(Some(AuthorshipTag::H), 50, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Auto);
    }

    #[test]
    fn aih_derives_aih() {
        let commits = vec![
            commit(Some(AuthorshipTag::Aih), 400, false),
            commit(Some(AuthorshipTag::H), 50, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Aih);
    }

    #[test]
    fn mixed_derives_mix() {
        let commits = vec![
            commit(Some(AuthorshipTag::H), 100, false),
            commit(Some(AuthorshipTag::Ai), 100, false),
            commit(Some(AuthorshipTag::Aih), 100, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Mix);
    }

    #[test]
    fn strict_rejects_untagged() {
        let commits = vec![
            commit(Some(AuthorshipTag::H), 100, false),
            commit(None, 50, false),
        ];
        assert!(derive_authorship(&commits, true).is_err());
    }

    #[test]
    fn lenient_treats_untagged_as_mix() {
        let commits = vec![
            commit(Some(AuthorshipTag::H), 100, false),
            commit(None, 50, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Mix);
    }

    #[test]
    fn empty_commits_derives_mix() {
        let result = derive_authorship(&[], false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Mix);
    }

    #[test]
    fn auto_commits_dont_block_hrai_derivation() {
        // 85% hrai (with reviewer) + 5% auto (no reviewer) + 10% h
        // The auto commit should NOT prevent hrai derivation
        let commits = vec![
            commit(Some(AuthorshipTag::Hrai), 850, true),
            commit(Some(AuthorshipTag::Auto), 50, false),
            commit(Some(AuthorshipTag::H), 100, false),
        ];
        let result = derive_authorship(&commits, false).unwrap();
        assert_eq!(result.tag, AuthorshipTag::Hrai);
    }
}
