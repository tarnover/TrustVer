// Key management logic

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during key operations.
#[derive(Debug, Error)]
pub enum KeyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Key generation error: {0}")]
    Generation(String),
}

impl From<schemapin::crypto::Error> for KeyError {
    fn from(err: schemapin::crypto::Error) -> Self {
        KeyError::Generation(err.to_string())
    }
}

/// Result of a key generation operation.
#[derive(Debug)]
pub struct KeyGenerationResult {
    pub private_key_path: PathBuf,
    pub public_key_path: PathBuf,
    pub key_id: String,
}

/// Compute the key ID (SHA-256 fingerprint) for a public key PEM string.
pub fn compute_key_id(public_key_pem: &str) -> Result<String, KeyError> {
    let key_id = schemapin::crypto::calculate_key_id(public_key_pem)?;
    Ok(key_id)
}

/// Generate a new ECDSA P-256 key pair and write the PEM files to `output_dir`.
///
/// Files are named `{name}-private.pem` and `{name}-public.pem`.
pub fn generate_keypair(output_dir: &Path, name: &str) -> Result<KeyGenerationResult, KeyError> {
    fs::create_dir_all(output_dir)?;

    let keypair = schemapin::crypto::generate_key_pair()?;
    let key_id = schemapin::crypto::calculate_key_id(&keypair.public_key_pem)?;

    let private_key_path = output_dir.join(format!("{name}-private.pem"));
    let public_key_path = output_dir.join(format!("{name}-public.pem"));

    fs::write(&private_key_path, &keypair.private_key_pem)?;
    fs::write(&public_key_path, &keypair.public_key_pem)?;

    Ok(KeyGenerationResult {
        private_key_path,
        public_key_path,
        key_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn generate_keypair_produces_valid_pems() {
        let dir = TempDir::new().unwrap();
        let result = generate_keypair(dir.path(), "test").unwrap();
        assert!(result.private_key_path.exists());
        assert!(result.public_key_path.exists());
        assert!(!result.key_id.is_empty());

        let private = std::fs::read_to_string(&result.private_key_path).unwrap();
        let public = std::fs::read_to_string(&result.public_key_path).unwrap();
        assert!(private.contains("PRIVATE KEY"));
        assert!(public.contains("PUBLIC KEY"));
    }

    #[test]
    fn generated_keys_can_sign_and_verify() {
        let dir = TempDir::new().unwrap();
        let result = generate_keypair(dir.path(), "test").unwrap();
        let private = std::fs::read_to_string(&result.private_key_path).unwrap();
        let public = std::fs::read_to_string(&result.public_key_path).unwrap();

        let data = b"test data to sign";
        let signature = schemapin::crypto::sign_data(&private, data).unwrap();
        let valid = schemapin::crypto::verify_signature(&public, data, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn compute_key_id_works() {
        let keypair = schemapin::crypto::generate_key_pair().unwrap();
        let kid = compute_key_id(&keypair.public_key_pem).unwrap();
        assert!(!kid.is_empty());
    }
}
