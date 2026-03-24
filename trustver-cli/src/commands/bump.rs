use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use trustver_core::commit::CommitMessage;
use trustver_core::config::Config;
use trustver_core::derive::{derive_authorship, CommitInfo};
use trustver_core::git::{git_latest_tag, git_log_all, git_log_range};
use trustver_core::version::{AuthorshipTag, BumpLevel};

pub fn run(
    level: &str,
    authorship_override: Option<String>,
    strict: bool,
    from_ref: Option<String>,
    create_tag: bool,
    json: bool,
) -> Result<()> {
    let config_path = PathBuf::from("trustver.toml");
    let mut config = Config::load(&config_path)
        .context("failed to load trustver.toml (run 'trustver init' first)")?;

    let bump_level = match level {
        "macro" => BumpLevel::Macro,
        "meso" => BumpLevel::Meso,
        "micro" => BumpLevel::Micro,
        _ => bail!("invalid bump level: {level}. Must be macro, meso, or micro"),
    };

    let effective_strict = strict || config.strict;

    let authorship = if let Some(ref tag_str) = authorship_override {
        let tag: AuthorshipTag = tag_str
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid authorship tag: {tag_str}"))?;
        eprintln!("Note: authorship overridden to [{tag}]. Document the rationale in the PAD when available.");
        tag
    } else {
        let repo_path = PathBuf::from(".");
        let from = if let Some(ref r) = from_ref {
            r.clone()
        } else {
            git_latest_tag(&repo_path, "v*")
                .context("failed to find latest tag")?
                .unwrap_or_else(|| {
                    eprintln!("Warning: no previous version tag found, scanning all commits");
                    String::new()
                })
        };

        let git_commits = if from.is_empty() {
            git_log_all(&repo_path).context("failed to read git log")?
        } else {
            git_log_range(&repo_path, &from, "HEAD").context("failed to read git log")?
        };

        let commit_infos: Vec<CommitInfo> = git_commits.iter().map(|gc| {
            let full_msg = if gc.body.is_empty() {
                gc.subject.clone()
            } else {
                format!("{}\n\n{}", gc.subject, gc.body)
            };
            let parsed = CommitMessage::parse(&full_msg).ok();
            let tag = parsed.as_ref().and_then(|c| {
                c.trailers
                    .get("Authorship")
                    .and_then(|v| v.parse::<AuthorshipTag>().ok())
                    .or(c.authorship_tag)
            });
            let has_reviewer = parsed
                .as_ref()
                .map(|c| c.trailers.contains_key("Reviewer"))
                .unwrap_or(false);
            CommitInfo { tag, lines_changed: gc.lines_changed(), has_reviewer }
        }).collect();

        let result = derive_authorship(&commit_infos, effective_strict)
            .context("authorship derivation failed")?;
        for w in &result.warnings {
            eprintln!("Warning: {w}");
        }
        result.tag
    };

    let new_version = config.current_version.bump(bump_level, authorship);
    let old_version = config.current_version.to_string();
    config.current_version = new_version.clone();
    config.save(&config_path).context("failed to update trustver.toml")?;

    if create_tag {
        let tag_name = format!(
            "v{}.{}.{}",
            new_version.macro_ver, new_version.meso, new_version.micro
        );
        let output = std::process::Command::new("git")
            .args(["tag", &tag_name])
            .output()
            .context("failed to create git tag")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("failed to create tag {tag_name}: {stderr}");
        }
        if !json {
            eprintln!("Created tag: {tag_name}");
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "old_version": old_version,
            "new_version": new_version.to_string(),
            "bump_level": level,
            "authorship": authorship.to_string(),
            "overridden": authorship_override.is_some(),
        }));
    } else {
        println!("{old_version} -> {new_version}");
    }

    Ok(())
}
