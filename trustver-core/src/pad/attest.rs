// PAD attestation logic

use crate::pad::{Attestation, PadDocument, PadError};

/// Append an attestation to a PAD document, optionally signing it.
///
/// If `sign_key_pem` is `Some`, the attestation content is serialized as
/// canonical JSON and signed with the given ECDSA-P256 private key via
/// SchemaPin.  If `None`, `signature` is left empty.
///
/// Returns a clone of `pad` with the new `Attestation` appended.
pub fn append_attestation(
    pad: &PadDocument,
    attestation_type: &str,
    attester: &str,
    detail: serde_json::Value,
    sign_key_pem: Option<&str>,
) -> Result<PadDocument, PadError> {
    let timestamp =
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let signature = if let Some(key_pem) = sign_key_pem {
        let content = serde_json::json!({
            "type": attestation_type,
            "timestamp": timestamp,
            "detail": detail,
            "attester": attester,
        });
        let canonical = super::canonical_json(&content);
        let sig = schemapin::crypto::sign_data(key_pem, canonical.as_bytes())
            .map_err(|e| PadError::Signing(e.to_string()))?;
        Some(sig)
    } else {
        None
    };

    let attestation = Attestation {
        type_: attestation_type.to_string(),
        timestamp,
        detail,
        attester: attester.to_string(),
        signature,
    };

    let mut new_pad = pad.clone();
    new_pad.attestations.push(attestation);
    Ok(new_pad)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pad::tests::sample_pad;

    #[test]
    fn append_unsigned_attestation() {
        let pad = sample_pad();
        let result = append_attestation(
            &pad,
            "test-verified",
            "ci@test.com",
            serde_json::json!({"suite": "cargo test", "passed": 52}),
            None,
        )
        .unwrap();
        assert_eq!(result.attestations.len(), 1);
        assert_eq!(result.attestations[0].type_, "test-verified");
        assert_eq!(result.attestations[0].attester, "ci@test.com");
        assert!(result.attestations[0].signature.is_none());
    }

    #[test]
    fn append_signed_attestation() {
        let pad = sample_pad();
        let keypair = schemapin::crypto::generate_key_pair().unwrap();
        let result = append_attestation(
            &pad,
            "manual-audit",
            "auditor@test.com",
            serde_json::json!({"scope": "security"}),
            Some(&keypair.private_key_pem),
        )
        .unwrap();
        assert_eq!(result.attestations.len(), 1);
        assert!(result.attestations[0].signature.is_some());
    }

    #[test]
    fn append_preserves_existing() {
        let pad = sample_pad();
        let pad2 = append_attestation(
            &pad,
            "ci-passed",
            "ci@test.com",
            serde_json::json!({}),
            None,
        )
        .unwrap();
        let pad3 = append_attestation(
            &pad2,
            "code-review",
            "dev@test.com",
            serde_json::json!({}),
            None,
        )
        .unwrap();
        assert_eq!(pad3.attestations.len(), 2);
    }
}
