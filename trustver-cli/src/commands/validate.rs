use anyhow::Result;
use trustver_core::version::TrustVersion;

pub fn run(version_string: &str, quiet: bool, json: bool) -> Result<()> {
    match version_string.parse::<TrustVersion>() {
        Ok(v) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": true,
                        "version": v.to_string(),
                        "macro": v.macro_ver,
                        "meso": v.meso,
                        "micro": v.micro,
                        "pre_release": v.pre_release,
                        "authorship": v.authorship.to_string(),
                    })
                );
            } else if !quiet {
                println!("Valid TrustVer: {v}");
                println!("  effort: {}.{}.{}", v.macro_ver, v.meso, v.micro);
                println!("  authorship: {}", v.authorship);
                if let Some(ref pre) = v.pre_release {
                    println!("  pre-release: {pre}");
                }
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": false,
                        "error": e.to_string(),
                    })
                );
            } else if !quiet {
                eprintln!("Invalid: {e}");
            }
            std::process::exit(1);
        }
    }
}
