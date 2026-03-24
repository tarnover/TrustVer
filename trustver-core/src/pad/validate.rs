// PAD validation logic

use crate::commit::{Severity, ValidationIssue};
use crate::pad::PadDocument;
use crate::version::TrustVersion;

/// Validate the structural integrity of a [`PadDocument`].
///
/// Returns a (possibly empty) list of [`ValidationIssue`]s. Callers should
/// treat any [`Severity::Error`] item as a blocking defect.
pub fn validate_pad(pad: &PadDocument) -> Vec<ValidationIssue> {
    let mut issues: Vec<ValidationIssue> = Vec::new();

    // -----------------------------------------------------------------------
    // Version parseable as TrustVersion
    // -----------------------------------------------------------------------
    let parsed_version = match pad.version.parse::<TrustVersion>() {
        Ok(v) => Some(v),
        Err(_) => {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "version field {:?} is not a valid TrustVersion string",
                    pad.version
                ),
            });
            None
        }
    };

    // -----------------------------------------------------------------------
    // Authorship tag must match parsed version's authorship
    // -----------------------------------------------------------------------
    if let Some(ref v) = parsed_version {
        if v.authorship != pad.authorship.tag {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "authorship tag mismatch: version implies {:?} but authorship.tag is {:?}",
                    v.authorship, pad.authorship.tag
                ),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Required non-empty string fields
    // -----------------------------------------------------------------------
    if pad.trustver_spec.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            message: "trustver_spec must not be empty".to_string(),
        });
    }
    if pad.package.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            message: "package must not be empty".to_string(),
        });
    }
    if pad.timestamp.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            message: "timestamp must not be empty".to_string(),
        });
    }
    if pad.identity.source.commit.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            message: "identity.source.commit must not be empty".to_string(),
        });
    }

    // -----------------------------------------------------------------------
    // Artifact hashes
    // -----------------------------------------------------------------------
    for (key, value) in &pad.identity.artifact_hashes {
        if key.starts_with("sha256") && value.len() != 64 {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "artifact hash key {:?} looks like sha256 but value has length {} (expected 64)",
                    key,
                    value.len()
                ),
            });
        }
        if !value.chars().all(|c| c.is_ascii_hexdigit()) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "artifact hash value for key {:?} is not valid hexadecimal",
                    key
                ),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Attestations — per-entry required fields
    // -----------------------------------------------------------------------
    for (idx, att) in pad.attestations.iter().enumerate() {
        if att.type_.is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!("attestation[{idx}]: type_ must not be empty"),
            });
        }
        if att.timestamp.is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!("attestation[{idx}]: timestamp must not be empty"),
            });
        }
        if att.attester.is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!("attestation[{idx}]: attester must not be empty"),
            });
        }
        // Warning: unsigned attestation
        if att.signature.is_none() {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!("attestation[{idx}] is unsigned (no signature present)"),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Warnings
    // -----------------------------------------------------------------------

    // Empty signatures array
    if pad.signatures.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: "signatures array is empty; the PAD has not been signed".to_string(),
        });
    }

    // Empty attestations array
    if pad.attestations.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: "attestations array is empty; consider adding attestations".to_string(),
        });
    }

    // ai_contribution_pct > 100 (u8 already caps at 255, but 101..=255 is invalid)
    if let Some(pct) = pad.authorship.detail.ai_contribution_pct {
        if pct > 100 {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!(
                    "ai_contribution_pct is {pct}, which exceeds 100 — check the contribution value"
                ),
            });
        }
    }

    // identity.source.repository is None
    if pad.identity.source.repository.is_none() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: "identity.source.repository is not set".to_string(),
        });
    }

    // identity.build is None
    if pad.identity.build.is_none() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: "identity.build is not set".to_string(),
        });
    }

    issues
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::Severity;
    use crate::pad::tests::sample_pad;

    #[test]
    fn valid_pad_passes() {
        let pad = sample_pad();
        let issues = validate_pad(&pad);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn invalid_version_string_is_error() {
        let mut pad = sample_pad();
        pad.version = "not-a-version".to_string();
        let issues = validate_pad(&pad);
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("version")));
    }

    #[test]
    fn authorship_tag_mismatch_is_error() {
        let mut pad = sample_pad();
        pad.version = "2.4.0+h".to_string();
        let issues = validate_pad(&pad);
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("mismatch")));
    }

    #[test]
    fn empty_signatures_is_warning() {
        let pad = sample_pad();
        let issues = validate_pad(&pad);
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("signature")));
    }

    #[test]
    fn unsigned_attestation_is_warning() {
        let mut pad = sample_pad();
        pad.attestations.push(crate::pad::Attestation {
            type_: "test-verified".to_string(),
            timestamp: "2026-03-23T14:00:00Z".to_string(),
            detail: serde_json::json!({}),
            attester: "ci@test.com".to_string(),
            signature: None,
        });
        let issues = validate_pad(&pad);
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("unsigned")));
    }

    #[test]
    fn contribution_pct_over_100_is_warning() {
        let mut pad = sample_pad();
        pad.authorship.detail.ai_contribution_pct = Some(150);
        let issues = validate_pad(&pad);
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("contribution")));
    }
}
