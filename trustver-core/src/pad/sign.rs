// PAD signing and verification logic

use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::pad::{PadDocument, PadError, Signature};

// ---------------------------------------------------------------------------
// ECDSA-P256 signing
// ---------------------------------------------------------------------------

/// Sign a PAD document with an ECDSA-P256 private key.
///
/// Returns a clone of `pad` with the new `Signature` appended.  The signable
/// content is the canonical JSON of the document with the `signatures` array
/// excluded (see `PadDocument::signable_content`).
pub fn sign_pad(
    pad: &PadDocument,
    private_key_pem: &str,
    key_id: &str,
    signer: &str,
) -> Result<PadDocument, PadError> {
    let canonical = pad.signable_content()?;

    let signature = schemapin::crypto::sign_data(private_key_pem, canonical.as_bytes())
        .map_err(|e| PadError::Signing(e.to_string()))?;

    let mut signed = pad.clone();
    signed.signatures.push(Signature {
        signer: signer.to_string(),
        algorithm: "ECDSA-P256".to_string(),
        key_id: key_id.to_string(),
        signature,
    });
    Ok(signed)
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Verify a single `Signature` entry against `pad`.
///
/// - `ECDSA-P256`: requires `public_key_pem`; verifies via schemapin.
/// - `sigstore-cosign`: `public_key_pem` is ignored; shells out to `cosign`.
/// - anything else: returns `PadError::Verification`.
pub fn verify_pad_signature(
    pad: &PadDocument,
    sig: &Signature,
    public_key_pem: Option<&str>,
) -> Result<bool, PadError> {
    match sig.algorithm.as_str() {
        "ECDSA-P256" => {
            let pubkey = public_key_pem.ok_or_else(|| {
                PadError::Verification(
                    "public_key_pem is required for ECDSA-P256 verification".to_string(),
                )
            })?;
            let canonical = pad.signable_content()?;
            schemapin::crypto::verify_signature(pubkey, canonical.as_bytes(), &sig.signature)
                .map_err(|e| PadError::Verification(e.to_string()))
        }
        "sigstore-cosign" => verify_cosign_signature(pad, sig),
        other => Err(PadError::Verification(format!(
            "unknown algorithm: {other}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// cosign helpers
// ---------------------------------------------------------------------------

/// Verify a `sigstore-cosign` signature by shelling out to `cosign verify-blob`.
fn verify_cosign_signature(pad: &PadDocument, sig: &Signature) -> Result<bool, PadError> {
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Decode the base64-encoded bundle stored in sig.signature.
    let bundle_bytes = STANDARD
        .decode(&sig.signature)
        .map_err(|e| PadError::Verification(format!("base64 decode error: {e}")))?;

    // Write bundle to a named temp file.
    let mut bundle_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut bundle_file, &bundle_bytes)?;

    // Write the signable content to another temp file.
    let canonical = pad.signable_content()?;
    let mut content_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut content_file, canonical.as_bytes())?;

    let status = Command::new("cosign")
        .args([
            "verify-blob",
            "--bundle",
            bundle_file
                .path()
                .to_str()
                .ok_or_else(|| PadError::Cosign("invalid bundle path".to_string()))?,
            content_file
                .path()
                .to_str()
                .ok_or_else(|| PadError::Cosign("invalid content path".to_string()))?,
        ])
        .status()
        .map_err(|e| PadError::Cosign(format!("failed to run cosign: {e}")))?;

    Ok(status.success())
}

/// Sign a PAD document using `cosign sign-blob` (keyless / Sigstore).
///
/// The resulting bundle file is written to `{pad_path}.cosign.bundle`.
/// The bundle content is base64-encoded and stored in the returned
/// `Signature.signature` field.
pub fn sign_pad_cosign(
    pad: &PadDocument,
    signer: &str,
    pad_path: &Path,
) -> Result<PadDocument, PadError> {
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Write signable content to a temp file for cosign to read.
    let canonical = pad.signable_content()?;
    let mut content_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut content_file, canonical.as_bytes())?;

    // Construct the bundle output path.
    let bundle_path = {
        let mut p = pad_path.as_os_str().to_owned();
        p.push(".cosign.bundle");
        std::path::PathBuf::from(p)
    };

    let status = Command::new("cosign")
        .args([
            "sign-blob",
            "--yes",
            "--bundle",
            bundle_path
                .to_str()
                .ok_or_else(|| PadError::Cosign("invalid bundle path".to_string()))?,
            content_file
                .path()
                .to_str()
                .ok_or_else(|| PadError::Cosign("invalid content path".to_string()))?,
        ])
        .status()
        .map_err(|e| PadError::Cosign(format!("failed to run cosign: {e}")))?;

    if !status.success() {
        return Err(PadError::Cosign(format!(
            "cosign sign-blob exited with status {status}"
        )));
    }

    // Read and base64-encode the bundle.
    let bundle_bytes = std::fs::read(&bundle_path)?;
    let signature = STANDARD.encode(&bundle_bytes);

    let mut signed = pad.clone();
    signed.signatures.push(Signature {
        signer: signer.to_string(),
        algorithm: "sigstore-cosign".to_string(),
        key_id: "sigstore".to_string(),
        signature,
    });
    Ok(signed)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pad::tests::sample_pad;

    #[test]
    fn sign_and_verify_roundtrip() {
        let pad = sample_pad();
        let keypair = schemapin::crypto::generate_key_pair().unwrap();
        let key_id = schemapin::crypto::calculate_key_id(&keypair.public_key_pem).unwrap();

        let signed = sign_pad(&pad, &keypair.private_key_pem, &key_id, "test@test.com").unwrap();
        assert_eq!(signed.signatures.len(), 1);
        assert_eq!(signed.signatures[0].algorithm, "ECDSA-P256");
        assert_eq!(signed.signatures[0].signer, "test@test.com");

        let verified = verify_pad_signature(
            &signed,
            &signed.signatures[0],
            Some(&keypair.public_key_pem),
        )
        .unwrap();
        assert!(verified);
    }

    #[test]
    fn verify_fails_with_wrong_key() {
        let pad = sample_pad();
        let kp1 = schemapin::crypto::generate_key_pair().unwrap();
        let kp2 = schemapin::crypto::generate_key_pair().unwrap();
        let kid = schemapin::crypto::calculate_key_id(&kp1.public_key_pem).unwrap();

        let signed = sign_pad(&pad, &kp1.private_key_pem, &kid, "test@test.com").unwrap();
        let verified =
            verify_pad_signature(&signed, &signed.signatures[0], Some(&kp2.public_key_pem))
                .unwrap();
        assert!(!verified);
    }

    #[test]
    fn multiple_signatures_independent() {
        let pad = sample_pad();
        let kp1 = schemapin::crypto::generate_key_pair().unwrap();
        let kp2 = schemapin::crypto::generate_key_pair().unwrap();
        let kid1 = schemapin::crypto::calculate_key_id(&kp1.public_key_pem).unwrap();
        let kid2 = schemapin::crypto::calculate_key_id(&kp2.public_key_pem).unwrap();

        let signed = sign_pad(&pad, &kp1.private_key_pem, &kid1, "signer1@test.com").unwrap();
        let double_signed =
            sign_pad(&signed, &kp2.private_key_pem, &kid2, "signer2@test.com").unwrap();
        assert_eq!(double_signed.signatures.len(), 2);

        let v1 = verify_pad_signature(
            &double_signed,
            &double_signed.signatures[0],
            Some(&kp1.public_key_pem),
        )
        .unwrap();
        let v2 = verify_pad_signature(
            &double_signed,
            &double_signed.signatures[1],
            Some(&kp2.public_key_pem),
        )
        .unwrap();
        assert!(v1);
        assert!(v2);
    }

    #[test]
    fn unknown_algorithm_returns_error() {
        let pad = sample_pad();
        let sig = crate::pad::Signature {
            signer: "test".to_string(),
            algorithm: "unknown".to_string(),
            key_id: "x".to_string(),
            signature: "x".to_string(),
        };
        assert!(verify_pad_signature(&pad, &sig, None).is_err());
    }
}
