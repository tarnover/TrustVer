use anyhow::{Context, Result};
use std::path::PathBuf;
use trustver_core::key::generate_keypair;

pub fn generate(output_dir: &str, name: &str) -> Result<()> {
    let dir = PathBuf::from(output_dir);
    let result = generate_keypair(&dir, name)
        .context("failed to generate keypair")?;

    println!("Generated ECDSA P-256 keypair:");
    println!("  Private key: {}", result.private_key_path.display());
    println!("  Public key:  {}", result.public_key_path.display());
    println!("  Key ID:      {}", result.key_id);
    eprintln!();
    eprintln!("WARNING: Keep your private key secure. Do not commit it to version control.");
    Ok(())
}
