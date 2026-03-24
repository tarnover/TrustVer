use anyhow::{bail, Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const HOOK_SCRIPT: &str = r#"#!/bin/sh
# TrustVer commit-msg hook
# Validates commit messages against the TrustVer convention

trustver check-commit --file "$1"
"#;

pub fn install(force: bool) -> Result<()> {
    let hooks_dir = PathBuf::from(".git/hooks");
    if !hooks_dir.exists() {
        bail!("not a git repository (no .git/hooks directory)");
    }

    let hook_path = hooks_dir.join("commit-msg");
    if hook_path.exists() && !force {
        bail!(
            "commit-msg hook already exists at {}. Use --force to overwrite.",
            hook_path.display()
        );
    }

    fs::write(&hook_path, HOOK_SCRIPT).context("failed to write hook file")?;

    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms).context("failed to set hook permissions")?;

    println!("Installed commit-msg hook at {}", hook_path.display());
    Ok(())
}
