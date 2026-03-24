use anyhow::{bail, Context, Result};
use std::io::Read;
use trustver_core::commit::{CommitMessage, Severity};

pub fn run(message: Option<String>, file: Option<String>, json: bool) -> Result<()> {
    let msg = if let Some(m) = message {
        m
    } else if let Some(path) = file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read commit message file: {path}"))?
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)
            .context("failed to read from stdin")?;
        buf
    };

    let msg = msg.trim();
    if msg.is_empty() {
        bail!("empty commit message");
    }

    let commit = match CommitMessage::parse(msg) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({
                    "valid": false,
                    "errors": [e.to_string()],
                    "warnings": [],
                }));
            } else {
                eprintln!("Parse error: {e}");
            }
            std::process::exit(1);
        }
    };

    let issues = commit.validate();
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();

    if json {
        println!("{}", serde_json::json!({
            "valid": errors.is_empty(),
            "errors": errors.iter().map(|i| &i.message).collect::<Vec<_>>(),
            "warnings": warnings.iter().map(|i| &i.message).collect::<Vec<_>>(),
        }));
    } else {
        for e in &errors {
            eprintln!("ERROR: {}", e.message);
        }
        for w in &warnings {
            eprintln!("WARNING: {}", w.message);
        }
        if errors.is_empty() {
            println!("Commit message is TrustVer-conformant.");
        }
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
