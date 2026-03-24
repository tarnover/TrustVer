use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use trustver_core::commit::{Severity, ValidationIssue};
use trustver_core::config::Config;
use trustver_core::pad::attest::append_attestation;
use trustver_core::pad::generate::{generate_pad, GenerateOptions};
use trustver_core::pad::sign::{sign_pad, sign_pad_cosign, verify_pad_signature};
use trustver_core::pad::validate::validate_pad;
use trustver_core::pad::{PadDocument, Scope};

#[allow(clippy::too_many_arguments)]
pub fn generate(
    artifacts: Vec<String>,
    scope: &str,
    build_system: Option<String>,
    build_id: Option<String>,
    reproducible: bool,
    model: Option<String>,
    reviewers: Vec<String>,
    contribution_pct: Option<u8>,
    output: Option<String>,
) -> Result<()> {
    let config_path = PathBuf::from("trustver.toml");
    let config = Config::load(&config_path)
        .context("failed to load trustver.toml (run 'trustver init' first)")?;

    let parsed_scope: Scope = scope
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid scope: {e}"))?;

    let artifact_paths: Vec<PathBuf> = artifacts.into_iter().map(PathBuf::from).collect();

    let options = GenerateOptions {
        artifact_paths,
        scope: parsed_scope,
        build_system,
        build_id,
        reproducible,
        model,
        reviewers,
        contribution_pct,
        output_path: output.as_deref().map(PathBuf::from),
    };

    let repo_path = PathBuf::from(".");
    let pad = generate_pad(&config, &repo_path, &options)
        .context("failed to generate PAD")?;

    let out_path = match output {
        Some(ref p) => PathBuf::from(p),
        None => PathBuf::from(format!("{}-{}.pad.json", config.package_name, config.current_version)),
    };

    pad.save(&out_path).context("failed to save PAD file")?;
    println!("Generated PAD: {}", out_path.display());
    Ok(())
}

pub fn sign(
    pad_file: &str,
    key: Option<String>,
    public_key: Option<String>,
    key_id: Option<String>,
    signer: &str,
    sigstore: bool,
) -> Result<()> {
    let pad_path = Path::new(pad_file);
    let pad = PadDocument::load(pad_path).context("failed to load PAD file")?;

    let signed = if sigstore {
        sign_pad_cosign(&pad, signer, pad_path).context("cosign signing failed")?
    } else {
        let key_path = key.as_deref().ok_or_else(|| {
            anyhow::anyhow!("--key is required when not using --sigstore")
        })?;
        let private_pem = std::fs::read_to_string(key_path)
            .context("failed to read private key file")?;

        let resolved_key_id = if let Some(ref kid) = key_id {
            kid.clone()
        } else if let Some(ref pub_key_path) = public_key {
            let pub_pem = std::fs::read_to_string(pub_key_path)
                .context("failed to read public key file")?;
            trustver_core::key::compute_key_id(&pub_pem)
                .context("failed to compute key ID from public key")?
        } else {
            bail!("either --key-id or --public-key is required to determine the key ID");
        };

        sign_pad(&pad, &private_pem, &resolved_key_id, signer)
            .context("signing failed")?
    };

    signed.save(pad_path).context("failed to save signed PAD")?;
    println!(
        "Signed PAD: {} ({} signature(s) total)",
        pad_path.display(),
        signed.signatures.len()
    );
    Ok(())
}

pub fn attest(
    pad_file: &str,
    attestation_type: &str,
    attester: &str,
    detail: Option<String>,
    detail_file: Option<String>,
    sign_key: Option<String>,
    unsigned: bool,
) -> Result<()> {
    if sign_key.is_none() && !unsigned {
        bail!("either --sign-key or --unsigned must be specified");
    }

    let pad_path = Path::new(pad_file);
    let pad = PadDocument::load(pad_path).context("failed to load PAD file")?;

    let detail_value: serde_json::Value = if let Some(ref json_str) = detail {
        serde_json::from_str(json_str).context("failed to parse --detail as JSON")?
    } else if let Some(ref file_path) = detail_file {
        let contents = std::fs::read_to_string(file_path)
            .context("failed to read detail file")?;
        serde_json::from_str(&contents).context("failed to parse detail file as JSON")?
    } else {
        serde_json::json!({})
    };

    let sign_key_pem: Option<String> = if let Some(ref key_path) = sign_key {
        Some(
            std::fs::read_to_string(key_path)
                .context("failed to read sign key file")?,
        )
    } else {
        None
    };

    let attested = append_attestation(
        &pad,
        attestation_type,
        attester,
        detail_value,
        sign_key_pem.as_deref(),
    )
    .context("failed to append attestation")?;

    attested.save(pad_path).context("failed to save PAD file")?;
    println!(
        "Attested PAD: {} ({} attestation(s) total)",
        pad_path.display(),
        attested.attestations.len()
    );
    Ok(())
}

pub fn validate(
    pad_file: &str,
    verify: bool,
    public_key: Option<String>,
    json: bool,
) -> Result<()> {
    let pad_path = Path::new(pad_file);
    let pad = PadDocument::load(pad_path).context("failed to load PAD file")?;

    let mut issues: Vec<ValidationIssue> = validate_pad(&pad);

    if verify {
        let pub_pem: Option<String> = if let Some(ref pk_path) = public_key {
            Some(
                std::fs::read_to_string(pk_path)
                    .context("failed to read public key file")?,
            )
        } else {
            None
        };

        for (idx, sig) in pad.signatures.iter().enumerate() {
            match verify_pad_signature(&pad, sig, pub_pem.as_deref()) {
                Ok(true) => {}
                Ok(false) => {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: format!("signature[{idx}] verification failed (signer: {})", sig.signer),
                    });
                }
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: format!("signature[{idx}] could not be verified: {e}"),
                    });
                }
            }
        }
    }

    let errors: Vec<&ValidationIssue> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    let warnings: Vec<&ValidationIssue> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();

    if json {
        let output = serde_json::json!({
            "pad_file": pad_file,
            "errors": errors.iter().map(|i| &i.message).collect::<Vec<_>>(),
            "warnings": warnings.iter().map(|i| &i.message).collect::<Vec<_>>(),
            "valid": errors.is_empty(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        for w in &warnings {
            eprintln!("Warning: {}", w.message);
        }
        for e in &errors {
            eprintln!("Error:   {}", e.message);
        }
        if errors.is_empty() {
            println!("PAD is valid: {}", pad_file);
        } else {
            println!("PAD validation failed: {} error(s)", errors.len());
        }
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
