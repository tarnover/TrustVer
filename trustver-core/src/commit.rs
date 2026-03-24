use std::collections::HashMap;
use std::str::FromStr;

use thiserror::Error;

use crate::version::AuthorshipTag;

// ---------------------------------------------------------------------------
// CommitParseError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CommitParseError {
    #[error("invalid subject line: {0}")]
    InvalidSubject(String),
}

// ---------------------------------------------------------------------------
// Severity / ValidationIssue
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}

// ---------------------------------------------------------------------------
// CommitMessage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CommitMessage {
    pub type_: String,
    pub scope: Option<String>,
    pub description: String,
    pub authorship_tag: Option<AuthorshipTag>,
    pub body: Option<String>,
    pub trailers: HashMap<String, String>,
}

impl CommitMessage {
    /// Parse a raw commit message string into a `CommitMessage`.
    ///
    /// Subject format: `type[(scope)]: description[ [tag]]`
    /// After the subject, an optional blank line separates the body.
    /// The final paragraph where every line is a `Key: Value` pair is
    /// treated as the trailer block.
    pub fn parse(raw: &str) -> Result<Self, CommitParseError> {
        let mut lines = raw.lines();

        // --- Subject line ---
        let subject = lines.next().unwrap_or("").trim().to_string();
        if subject.is_empty() {
            return Err(CommitParseError::InvalidSubject(
                "empty subject line".to_string(),
            ));
        }

        // Extract optional [tag] from end of subject
        let (subject_without_tag, authorship_tag) = extract_tag(&subject);

        // Parse conventional commit format
        let (type_, scope, description) = parse_conventional(&subject_without_tag)?;

        // --- Body and trailers ---
        // Collect remaining lines into paragraphs (split on blank lines)
        let remaining: Vec<&str> = lines.collect();
        let full_rest = remaining.join("\n");

        let (body, trailers) = parse_body_and_trailers(&full_rest);

        Ok(CommitMessage {
            type_,
            scope,
            description,
            authorship_tag,
            body,
            trailers,
        })
    }

    /// Validate the commit message against TrustVer rules.
    /// Returns a list of `ValidationIssue`s (errors and warnings).
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues: Vec<ValidationIssue> = Vec::new();

        let has_subject_tag = self.authorship_tag.is_some();
        let authorship_trailer = self.trailers.get("Authorship");
        let has_authorship_trailer = authorship_trailer.is_some();

        // --- Error: both missing ---
        if !has_subject_tag && !has_authorship_trailer {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "both subject [tag] and Authorship trailer are missing".to_string(),
            });
            return issues; // early return per spec
        }

        // --- Error: subject tag present but no Authorship trailer ---
        if has_subject_tag && !has_authorship_trailer {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "subject [tag] present but Authorship trailer is missing".to_string(),
            });
        }

        // --- Error: Authorship trailer present but no subject tag ---
        if has_authorship_trailer && !has_subject_tag {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "Authorship trailer present but missing [tag] in subject line".to_string(),
            });
        }

        // --- Error: tag mismatch ---
        if let (Some(tag), Some(trailer_val)) = (self.authorship_tag, authorship_trailer) {
            let tag_str = tag.to_string();
            if tag_str != trailer_val.as_str() {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!(
                        "subject tag [{tag_str}] and Authorship trailer value \"{trailer_val}\" mismatch"
                    ),
                });
            }
        }

        // --- Determine effective tag for further checks ---
        let effective_tag: Option<AuthorshipTag> = self
            .authorship_tag
            .or_else(|| authorship_trailer.and_then(|v| AuthorshipTag::from_str(v).ok()));

        let ai_involved = matches!(
            effective_tag,
            Some(AuthorshipTag::Ai)
                | Some(AuthorshipTag::Hrai)
                | Some(AuthorshipTag::Aih)
                | Some(AuthorshipTag::Auto)
        );

        // --- Error: hrai requires Reviewer ---
        if effective_tag == Some(AuthorshipTag::Hrai) && !self.trailers.contains_key("Reviewer") {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "tag [hrai] requires a Reviewer trailer".to_string(),
            });
        }

        // --- Warnings ---
        if ai_involved {
            if !self.trailers.contains_key("Model") {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: "AI-involved tag but no Model trailer present".to_string(),
                });
            }

            if !self.trailers.contains_key("Contribution") {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: "AI-involved tag but no Contribution trailer present".to_string(),
                });
            }

            if effective_tag == Some(AuthorshipTag::Auto) && !self.trailers.contains_key("Agent-Id")
            {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: "tag [auto] but no Agent-Id trailer present".to_string(),
                });
            }
        }

        issues
    }

    /// Returns `true` if validation produces no `Error`-level issues.
    pub fn is_valid(&self) -> bool {
        self.validate()
            .iter()
            .all(|i| i.severity != Severity::Error)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract an optional `[tag]` from the end of the subject line.
/// Returns `(subject_without_tag, Option<AuthorshipTag>)`.
fn extract_tag(subject: &str) -> (String, Option<AuthorshipTag>) {
    if let Some(bracket_pos) = subject.rfind('[') {
        let rest = &subject[bracket_pos..];
        if rest.ends_with(']') {
            let inner = &rest[1..rest.len() - 1];
            if let Ok(tag) = inner.parse::<AuthorshipTag>() {
                let without_tag = subject[..bracket_pos].trim_end().to_string();
                return (without_tag, Some(tag));
            }
        }
    }
    (subject.to_string(), None)
}

/// Parse `type[(scope)]: description` from the subject (after tag removal).
fn parse_conventional(subject: &str) -> Result<(String, Option<String>, String), CommitParseError> {
    // Split on first ": "
    let colon_space = subject.find(": ").ok_or_else(|| {
        CommitParseError::InvalidSubject(format!("missing ': ' separator in subject: {subject}"))
    })?;

    let prefix = &subject[..colon_space];
    let description = subject[colon_space + 2..].trim().to_string();

    if description.is_empty() {
        return Err(CommitParseError::InvalidSubject(
            "empty description".to_string(),
        ));
    }

    // prefix is either `type` or `type(scope)`
    if let Some(paren_open) = prefix.find('(') {
        let type_ = prefix[..paren_open].trim().to_string();
        if !prefix.ends_with(')') {
            return Err(CommitParseError::InvalidSubject(format!(
                "unclosed scope parenthesis: {prefix}"
            )));
        }
        let scope = prefix[paren_open + 1..prefix.len() - 1].trim().to_string();
        Ok((type_, Some(scope), description))
    } else {
        Ok((prefix.trim().to_string(), None, description))
    }
}

/// Split the rest of the commit message (after the subject line) into an
/// optional body and a map of trailers.
///
/// The trailer block is the last paragraph where **every** line matches
/// `Key: Value`. Everything before the trailer block (excluding the leading
/// blank line) is the body.
fn parse_body_and_trailers(rest: &str) -> (Option<String>, HashMap<String, String>) {
    // Split into paragraphs on blank lines
    let paragraphs: Vec<&str> = rest
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        return (None, HashMap::new());
    }

    // Check if the last paragraph is a trailer block
    let last = paragraphs.last().unwrap();
    let trailers = try_parse_trailers(last);

    if let Some(trailer_map) = trailers {
        // Body is everything except the last (trailer) paragraph
        let body_paragraphs = &paragraphs[..paragraphs.len() - 1];
        let body = if body_paragraphs.is_empty() {
            None
        } else {
            Some(body_paragraphs.join("\n\n"))
        };
        (body, trailer_map)
    } else {
        // No trailer block — everything is body
        let body = Some(paragraphs.join("\n\n"));
        (body, HashMap::new())
    }
}

/// Attempt to parse every line in `paragraph` as `Key: Value`.
/// Returns `Some(map)` only if **all** lines are valid trailer lines.
fn try_parse_trailers(paragraph: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    for line in paragraph.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once(": ")?;
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::AuthorshipTag;

    #[test]
    fn parse_full_commit_message() {
        let msg = "feat(auth): add OAuth2 PKCE flow [hrai]\n\nAI-generated implementation.\n\nAuthorship: hrai\nModel: claude-opus-4-6\nContribution: ~85% AI-generated\nReviewer: jascha@tarnover.com";
        let parsed = CommitMessage::parse(msg).unwrap();
        assert_eq!(parsed.type_, "feat");
        assert_eq!(parsed.scope.as_deref(), Some("auth"));
        assert_eq!(parsed.description, "add OAuth2 PKCE flow");
        assert_eq!(parsed.authorship_tag, Some(AuthorshipTag::Hrai));
        assert_eq!(parsed.trailers.get("Authorship"), Some(&"hrai".to_string()));
        assert_eq!(
            parsed.trailers.get("Model"),
            Some(&"claude-opus-4-6".to_string())
        );
        assert_eq!(
            parsed.trailers.get("Reviewer"),
            Some(&"jascha@tarnover.com".to_string())
        );
    }

    #[test]
    fn parse_minimal_commit() {
        let msg = "fix: handle edge case [h]\n\nAuthorship: h";
        let parsed = CommitMessage::parse(msg).unwrap();
        assert_eq!(parsed.type_, "fix");
        assert_eq!(parsed.scope, None);
        assert_eq!(parsed.description, "handle edge case");
        assert_eq!(parsed.authorship_tag, Some(AuthorshipTag::H));
    }

    #[test]
    fn parse_commit_no_tag() {
        let msg = "fix: handle edge case";
        let parsed = CommitMessage::parse(msg).unwrap();
        assert_eq!(parsed.authorship_tag, None);
    }

    #[test]
    fn valid_commit_no_issues() {
        let msg = "feat(auth): add PKCE [hrai]\n\nBody text.\n\nAuthorship: hrai\nModel: claude-opus-4-6\nReviewer: jascha@tarnover.com";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn error_missing_subject_tag_and_trailer() {
        let msg = "feat: some change";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(!errors.is_empty());
    }

    #[test]
    fn error_trailer_present_but_no_subject_tag() {
        let msg = "feat: some change\n\nAuthorship: hrai\nReviewer: test@test.com";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors
            .iter()
            .any(|i| i.message.contains("missing [tag] in subject")));
    }

    #[test]
    fn error_tag_mismatch() {
        let msg = "feat: change [h]\n\nAuthorship: ai";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("mismatch")));
    }

    #[test]
    fn error_hrai_missing_reviewer() {
        let msg = "feat: change [hrai]\n\nAuthorship: hrai\nModel: claude-opus-4-6";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.iter().any(|i| i.message.contains("Reviewer")));
    }

    #[test]
    fn warning_missing_model_for_ai_tag() {
        let msg = "feat: change [ai]\n\nAuthorship: ai";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect();
        assert!(warnings.iter().any(|i| i.message.contains("Model")));
    }

    #[test]
    fn warning_missing_agent_id_for_auto() {
        let msg = "feat: change [auto]\n\nAuthorship: auto\nModel: claude-opus-4-6";
        let commit = CommitMessage::parse(msg).unwrap();
        let issues = commit.validate();
        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect();
        assert!(warnings.iter().any(|i| i.message.contains("Agent-Id")));
    }
}
