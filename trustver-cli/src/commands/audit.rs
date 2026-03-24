use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use trustver_core::commit::CommitMessage;
use trustver_core::derive::{derive_authorship, CommitInfo};
use trustver_core::git::{git_latest_tag, git_log_range};
use trustver_core::version::AuthorshipTag;

pub fn run(range: Option<String>, json: bool) -> Result<()> {
    let repo_path = PathBuf::from(".");
    let (from, to) = parse_range(range, &repo_path)?;

    let git_commits = git_log_range(&repo_path, &from, &to).context("failed to read git log")?;

    if git_commits.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"commits": 0, "message": "no commits in range"})
            );
        } else {
            println!("No commits in range {from}..{to}");
        }
        return Ok(());
    }

    let commit_infos: Vec<CommitInfo> = git_commits
        .iter()
        .map(|gc| {
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
            CommitInfo {
                tag,
                lines_changed: gc.lines_changed(),
                has_reviewer,
            }
        })
        .collect();

    let result = derive_authorship(&commit_infos, false).context("derivation failed")?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "range": format!("{from}..{to}"),
                "derived_tag": result.tag.to_string(),
                "total_commits": result.summary.total_commits,
                "untagged_commits": result.summary.untagged_commits,
                "total_lines": result.summary.total_lines,
                "tag_weights": result.summary.tag_weights.iter().map(|(k, v)| (k.to_string(), *v)).collect::<std::collections::HashMap<String, f64>>(),
                "warnings": result.warnings,
            })
        );
    } else {
        println!("Provenance Audit: {from}..{to}");
        println!("  Derived tag: {}", result.tag);
        println!(
            "  Commits: {} ({} untagged)",
            result.summary.total_commits, result.summary.untagged_commits
        );
        println!("  Lines changed: {}", result.summary.total_lines);
        println!();
        println!("  Authorship breakdown (by lines):");
        let mut weights: Vec<_> = result.summary.tag_weights.iter().collect();
        weights.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        for (tag, pct) in weights {
            if *pct > 0.0 {
                println!("    {tag}: {pct:.1}%");
            }
        }
        for w in &result.warnings {
            println!();
            println!("  Warning: {w}");
        }
    }

    Ok(())
}

fn parse_range(range: Option<String>, repo_path: &Path) -> Result<(String, String)> {
    match range {
        Some(r) if r.contains("..") => {
            let parts: Vec<&str> = r.splitn(2, "..").collect();
            Ok((parts[0].to_string(), parts[1].to_string()))
        }
        Some(single_ref) => Ok((single_ref, "HEAD".to_string())),
        None => {
            let tag = git_latest_tag(repo_path, "v*").context("failed to find latest tag")?;
            match tag {
                Some(t) => Ok((t, "HEAD".to_string())),
                None => bail!("no version tags found and no range specified"),
            }
        }
    }
}
