use anyhow::{bail, Context, Result};
use std::env;
use std::path::PathBuf;
use trustver_core::config::Config;

pub fn run(name: Option<String>, version: Option<String>) -> Result<()> {
    let config_path = PathBuf::from("trustver.toml");
    if config_path.exists() {
        bail!("trustver.toml already exists");
    }

    let package_name = name.unwrap_or_else(|| {
        env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    });

    let mut config = Config::default_with_name(package_name);

    if let Some(v) = version {
        config.current_version = v.parse().context("invalid version string")?;
    }

    config
        .save(&config_path)
        .context("failed to write trustver.toml")?;
    println!("Created trustver.toml");
    println!("  package: {}", config.package_name);
    println!("  version: {}", config.current_version);
    Ok(())
}
