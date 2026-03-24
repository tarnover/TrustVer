use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use thiserror::Error;

// ---------------------------------------------------------------------------
// GitError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git command failed: {0}")]
    CommandFailed(String),

    #[error("git binary not found")]
    NotAvailable,

    #[error("git output parse error: {0}")]
    ParseError(String),
}

// ---------------------------------------------------------------------------
// GitCommit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub subject: String,
    pub body: String,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub is_merge: bool,
}

impl GitCommit {
    pub fn lines_changed(&self) -> u64 {
        self.lines_added + self.lines_deleted
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GitError::NotAvailable
            } else {
                GitError::CommandFailed(e.to_string())
            }
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Err(GitError::CommandFailed(stderr.trim().to_string()))
    }
}

/// Parse combined `--format=%H --numstat` output into a map of hash -> (added, deleted).
///
/// The output interleaves lines:
///   <40-char hex hash>
///   <added>\t<deleted>\t<filename>   (zero or more)
///   <next hash>
///   …
///
/// Binary files report `-` for added/deleted; those are counted as 0.
fn parse_numstats(output: &str) -> HashMap<&str, (u64, u64)> {
    let mut map: HashMap<&str, (u64, u64)> = HashMap::new();
    let mut current_hash: Option<&str> = None;

    for line in output.lines() {
        // A 40-char hex string identifies a commit hash line.
        if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            current_hash = Some(line);
            map.entry(line).or_insert((0, 0));
            continue;
        }

        // Empty lines between commits — skip.
        if line.trim().is_empty() {
            continue;
        }

        // Numstat line: "<added>\t<deleted>\t<filename>"
        if let Some(hash) = current_hash {
            let mut parts = line.splitn(3, '\t');
            if let (Some(added_str), Some(deleted_str)) = (parts.next(), parts.next()) {
                let added: u64 = added_str.trim().parse().unwrap_or(0);
                let deleted: u64 = deleted_str.trim().parse().unwrap_or(0);
                let entry = map.entry(hash).or_insert((0, 0));
                entry.0 += added;
                entry.1 += deleted;
            }
        }
    }

    map
}

/// Parse the `--format=%H%n%s%n%b%n---END-COMMIT---` log output into
/// a list of `(hash, subject, body)` tuples.
fn parse_log_entries(output: &str) -> Vec<(String, String, String)> {
    let mut result = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in output.lines() {
        if line == "---END-COMMIT---" {
            if !current_lines.is_empty() {
                let hash = current_lines.first().copied().unwrap_or("").to_string();
                let subject = current_lines.get(1).copied().unwrap_or("").to_string();
                let body = current_lines[2..].join("\n").trim().to_string();
                result.push((hash, subject, body));
                current_lines.clear();
            }
            continue;
        }
        current_lines.push(line);
    }

    result
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return the most recent tag matching `pattern`, or `None` if no tag exists.
pub fn git_latest_tag(repo_path: &Path, pattern: &str) -> Result<Option<String>, GitError> {
    let result = run_git(
        repo_path,
        &["describe", "--tags", "--abbrev=0", "--match", pattern],
    );

    match result {
        Ok(output) => Ok(Some(output.trim().to_string())),
        Err(GitError::CommandFailed(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Return commits in `from..to` (exclusive of `from`, inclusive of `to`).
pub fn git_log_range(repo_path: &Path, from: &str, to: &str) -> Result<Vec<GitCommit>, GitError> {
    let range = format!("{from}..{to}");
    git_log_internal(repo_path, Some(&range))
}

/// Return all commits in the repository (no range restriction).
pub fn git_log_all(repo_path: &Path) -> Result<Vec<GitCommit>, GitError> {
    git_log_internal(repo_path, None)
}

fn git_log_internal(repo_path: &Path, range: Option<&str>) -> Result<Vec<GitCommit>, GitError> {
    // Build the log command for subject/body.
    let format = "--format=%H%n%s%n%b%n---END-COMMIT---";
    let mut log_args: Vec<&str> = vec!["log"];
    if let Some(r) = range {
        log_args.push(r);
    }
    log_args.extend_from_slice(&[format, "--no-merges"]);

    let log_output = run_git(repo_path, &log_args)?;

    // Build the numstat command.
    let numstat_format = "--format=%H";
    let mut numstat_args: Vec<&str> = vec!["log"];
    if let Some(r) = range {
        numstat_args.push(r);
    }
    numstat_args.extend_from_slice(&[numstat_format, "--numstat", "--no-merges"]);

    let numstat_output = run_git(repo_path, &numstat_args)?;

    // Parse numstats into a lookup map.
    let numstat_map = parse_numstats(&numstat_output);

    // Parse log entries and combine.
    let entries = parse_log_entries(&log_output);
    let mut commits = Vec::with_capacity(entries.len());

    for (hash, subject, body) in entries {
        if hash.is_empty() {
            continue;
        }
        let (lines_added, lines_deleted) =
            numstat_map.get(hash.as_str()).copied().unwrap_or((0, 0));

        commits.push(GitCommit {
            hash,
            subject,
            body,
            lines_added,
            lines_deleted,
            is_merge: false,
        });
    }

    Ok(commits)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let p = dir.path();
        Command::new("git")
            .args(["init"])
            .current_dir(p)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(p)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(p)
            .output()
            .unwrap();
        dir
    }

    fn make_commit(dir: &std::path::Path, filename: &str, content: &str, message: &str) {
        std::fs::write(dir.join(filename), content).unwrap();
        Command::new("git")
            .args(["add", filename])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn latest_tag_finds_version_tag() {
        let dir = init_repo();
        let p = dir.path();
        make_commit(p, "a.txt", "hello", "initial [h]\n\nAuthorship: h");
        Command::new("git")
            .args(["tag", "v0.1.0"])
            .current_dir(p)
            .output()
            .unwrap();
        make_commit(p, "b.txt", "world", "second [h]\n\nAuthorship: h");

        let tag = git_latest_tag(p, "v*").unwrap();
        assert_eq!(tag, Some("v0.1.0".to_string()));
    }

    #[test]
    fn latest_tag_returns_none_when_no_tags() {
        let dir = init_repo();
        let p = dir.path();
        make_commit(p, "a.txt", "hello", "initial");

        let tag = git_latest_tag(p, "v*").unwrap();
        assert_eq!(tag, None);
    }

    #[test]
    fn log_range_returns_commits() {
        let dir = init_repo();
        let p = dir.path();
        make_commit(p, "a.txt", "hello", "feat: first [h]\n\nAuthorship: h");
        Command::new("git")
            .args(["tag", "v0.1.0"])
            .current_dir(p)
            .output()
            .unwrap();
        make_commit(p, "b.txt", "world", "feat: second [ai]\n\nAuthorship: ai");
        make_commit(
            p,
            "c.txt",
            "test",
            "fix: third [hrai]\n\nAuthorship: hrai\nReviewer: test@test.com",
        );

        let commits = git_log_range(p, "v0.1.0", "HEAD").unwrap();
        assert_eq!(commits.len(), 2);
        // Verify we got line counts
        assert!(commits.iter().all(|c| c.lines_changed() > 0));
    }

    #[test]
    fn log_all_returns_all_commits() {
        let dir = init_repo();
        let p = dir.path();
        make_commit(p, "a.txt", "hello", "first commit");
        make_commit(p, "b.txt", "world", "second commit");
        make_commit(p, "c.txt", "test", "third commit");

        let commits = git_log_all(p).unwrap();
        assert_eq!(commits.len(), 3);
    }
}
