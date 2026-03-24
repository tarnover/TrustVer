pub mod generate;
pub mod sign;
pub mod validate;
pub mod attest;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::version::AuthorshipTag;

// ---------------------------------------------------------------------------
// PadError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum PadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("signing error: {0}")]
    Signing(String),
    #[error("verification error: {0}")]
    Verification(String),
    #[error("invalid PAD: {0}")]
    Invalid(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("cosign error: {0}")]
    Cosign(String),
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Stable,
    Rc,
    Preview,
    Experimental,
    Sandbox,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Scope::Stable => "stable",
            Scope::Rc => "rc",
            Scope::Preview => "preview",
            Scope::Experimental => "experimental",
            Scope::Sandbox => "sandbox",
        };
        f.write_str(s)
    }
}

impl FromStr for Scope {
    type Err = PadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stable" => Ok(Scope::Stable),
            "rc" => Ok(Scope::Rc),
            "preview" => Ok(Scope::Preview),
            "experimental" => Ok(Scope::Experimental),
            "sandbox" => Ok(Scope::Sandbox),
            other => Err(PadError::Invalid(format!("unknown scope: {other}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// Identity types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIdentity {
    pub system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub artifact_hashes: HashMap<String, String>,
    pub source: SourceIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildIdentity>,
}

// ---------------------------------------------------------------------------
// Authorship types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorshipDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_contribution_pct: Option<u8>,
    #[serde(default)]
    pub human_reviewers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorship {
    pub tag: AuthorshipTag,
    pub detail: AuthorshipDetail,
}

// ---------------------------------------------------------------------------
// Attestation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    #[serde(rename = "type")]
    pub type_: String,
    pub timestamp: String,
    #[serde(default)]
    pub detail: Value,
    pub attester: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

// ---------------------------------------------------------------------------
// Signature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signer: String,
    pub algorithm: String,
    pub key_id: String,
    pub signature: String,
}

// ---------------------------------------------------------------------------
// PadDocument
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadDocument {
    pub trustver_spec: String,
    pub version: String,
    pub package: String,
    pub timestamp: String,
    pub identity: Identity,
    pub authorship: Authorship,
    pub scope: Scope,
    #[serde(default)]
    pub attestations: Vec<Attestation>,
    #[serde(default)]
    pub signatures: Vec<Signature>,
}

impl PadDocument {
    /// Produce canonical JSON (sorted keys, no whitespace) of this document
    /// with the `signatures` array excluded.  Used as the content to sign.
    pub fn signable_content(&self) -> Result<String, PadError> {
        // Serialize the whole document to a Value first.
        let mut value: Value = serde_json::to_value(self)?;

        // Remove the signatures field.
        if let Value::Object(ref mut map) = value {
            map.remove("signatures");
        }

        // Produce canonical JSON (sorted keys, no whitespace).
        Ok(canonical_json(&value))
    }

    /// Deserialize a `PadDocument` from a JSON file.
    pub fn load(path: &Path) -> Result<Self, PadError> {
        let content = std::fs::read_to_string(path)?;
        let doc: PadDocument = serde_json::from_str(&content)?;
        Ok(doc)
    }

    /// Pretty-print this document to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), PadError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// canonical_json — recursive helper
// ---------------------------------------------------------------------------

/// Serialize a `serde_json::Value` as canonical JSON:
/// - object keys sorted lexicographically at every nesting level
/// - no extra whitespace
pub(crate) fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            let pairs: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let v = canonical_json(&map[k]);
                    format!("\"{}\":{}", k, v)
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", items.join(","))
        }
        // For all scalar types, the default serde_json compact output is
        // already canonical (numbers have no trailing zeros for integers,
        // strings use standard escape sequences, booleans and null are
        // unambiguous).
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::version::AuthorshipTag;
    use std::collections::HashMap;

    pub(crate) fn sample_pad() -> PadDocument {
        let mut artifact_hashes = HashMap::new();
        artifact_hashes.insert(
            "sha256".to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        );

        PadDocument {
            trustver_spec: "0.3.0".to_string(),
            version: "2.4.0+hrai".to_string(),
            package: "mylib".to_string(),
            timestamp: "2026-03-23T14:22:00Z".to_string(),
            identity: Identity {
                artifact_hashes,
                source: SourceIdentity {
                    repository: Some("https://github.com/example/mylib".to_string()),
                    commit: "abc123def456789".to_string(),
                    branch: Some("main".to_string()),
                },
                build: Some(BuildIdentity {
                    system: "github-actions".to_string(),
                    build_id: Some("run-98765".to_string()),
                    reproducible: true,
                }),
            },
            authorship: Authorship {
                tag: AuthorshipTag::Hrai,
                detail: AuthorshipDetail {
                    ai_model: Some("claude-opus-4-6".to_string()),
                    ai_contribution_pct: Some(72),
                    human_reviewers: vec!["jascha@tarnover.com".to_string()],
                    review_timestamp: Some("2026-03-23T13:45:00Z".to_string()),
                },
            },
            scope: Scope::Stable,
            attestations: vec![],
            signatures: vec![],
        }
    }

    #[test]
    fn pad_serde_roundtrip() {
        let pad = sample_pad();
        let json = serde_json::to_string_pretty(&pad).unwrap();
        let deserialized: PadDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "2.4.0+hrai");
        assert_eq!(deserialized.package, "mylib");
        assert_eq!(deserialized.authorship.tag, AuthorshipTag::Hrai);
        assert_eq!(deserialized.scope, Scope::Stable);
    }

    #[test]
    fn scope_serializes_as_lowercase() {
        let json = serde_json::to_string(&Scope::Experimental).unwrap();
        assert_eq!(json, "\"experimental\"");
        let parsed: Scope = serde_json::from_str("\"rc\"").unwrap();
        assert_eq!(parsed, Scope::Rc);
    }

    #[test]
    fn attestation_type_serializes_correctly() {
        let att = Attestation {
            type_: "test-verified".to_string(),
            timestamp: "2026-03-23T14:00:00Z".to_string(),
            detail: serde_json::json!({"suite": "pytest", "coverage_pct": 94}),
            attester: "ci@github.com".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&att).unwrap();
        assert!(json.contains("\"type\""));
        assert!(!json.contains("\"type_\""));
    }

    #[test]
    fn pad_signable_content_excludes_signatures() {
        let mut pad = sample_pad();
        pad.signatures.push(Signature {
            signer: "test@test.com".to_string(),
            algorithm: "ECDSA-P256".to_string(),
            key_id: "abc123".to_string(),
            signature: "fakesig".to_string(),
        });
        let content = pad.signable_content().unwrap();
        assert!(!content.contains("fakesig"));
        assert!(content.contains("mylib"));
        // Canonical: no pretty-printing whitespace
        assert!(!content.contains('\n'));
    }
}
