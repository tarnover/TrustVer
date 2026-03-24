// PAD generation logic

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::pad::{
    Authorship, AuthorshipDetail, BuildIdentity, Identity, PadDocument, PadError, Scope,
    SourceIdentity,
};

// ---------------------------------------------------------------------------
// GenerateOptions
// ---------------------------------------------------------------------------

pub struct GenerateOptions {
    pub artifact_paths: Vec<PathBuf>,
    pub scope: Scope,
    pub build_system: Option<String>,
    pub build_id: Option<String>,
    pub reproducible: bool,
    pub model: Option<String>,
    pub reviewers: Vec<String>,
    pub contribution_pct: Option<u8>,
    pub output_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Run a git command in `repo_path` and return trimmed stdout, or a `PadError::Git`
/// if the process fails or the output is not valid UTF-8.
fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, PadError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| PadError::Git(format!("failed to spawn git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PadError::Git(format!(
            "git {} failed: {stderr}",
            args.join(" ")
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| PadError::Git(format!("git output is not UTF-8: {e}")))?;
    Ok(stdout.trim().to_string())
}

/// SHA-256 hash of `data` as a 64-character lowercase hex string.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// generate_pad
// ---------------------------------------------------------------------------

/// Generate a PAD document from the current git repository state and the
/// supplied `options`.
pub fn generate_pad(
    config: &Config,
    repo_path: &Path,
    options: &GenerateOptions,
) -> Result<PadDocument, PadError> {
    // 1. Validate contribution_pct
    if let Some(pct) = options.contribution_pct {
        if pct > 100 {
            return Err(PadError::Invalid(format!(
                "contribution_pct {pct} exceeds 100"
            )));
        }
    }

    // 2. Source identity from git
    let commit = run_git(repo_path, &["rev-parse", "HEAD"])?;
    let branch_raw = run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = if branch_raw == "HEAD" {
        None // detached HEAD
    } else {
        Some(branch_raw)
    };
    let repository = run_git(repo_path, &["remote", "get-url", "origin"]).ok();

    let source = SourceIdentity {
        repository,
        commit,
        branch,
    };

    // 3. Hash artifacts
    let mut artifact_hashes: HashMap<String, String> = HashMap::new();
    let paths = &options.artifact_paths;
    match paths.len() {
        0 => {}
        1 => {
            let data = std::fs::read(&paths[0])?;
            artifact_hashes.insert("sha256".to_string(), sha256_hex(&data));
        }
        _ => {
            for path in paths {
                let data = std::fs::read(path)?;
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_string_lossy().into_owned());
                artifact_hashes.insert(format!("sha256:{filename}"), sha256_hex(&data));
            }
        }
    }

    // 4. Detect build identity: CLI args first, then env vars
    let build: Option<BuildIdentity> = {
        // CLI args take precedence
        if let Some(system) = &options.build_system {
            Some(BuildIdentity {
                system: system.clone(),
                build_id: options.build_id.clone(),
                reproducible: options.reproducible,
            })
        } else {
            // Detect from environment variables
            detect_build_from_env(options.reproducible)
        }
    };

    // 5. Authorship from config version tag + options
    let tag = config.current_version.authorship;
    let authorship = Authorship {
        tag,
        detail: AuthorshipDetail {
            ai_model: options.model.clone(),
            ai_contribution_pct: options.contribution_pct,
            human_reviewers: options.reviewers.clone(),
            review_timestamp: None,
        },
    };

    // 6. Timestamp
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    // 7. Assemble document
    let pad = PadDocument {
        trustver_spec: "0.3.0".to_string(),
        version: config.current_version.to_string(),
        package: config.package_name.clone(),
        timestamp,
        identity: Identity {
            artifact_hashes,
            source,
            build,
        },
        authorship,
        scope: options.scope,
        attestations: vec![],
        signatures: vec![],
    };

    Ok(pad)
}

/// Detect a CI build system from well-known environment variables.
fn detect_build_from_env(reproducible: bool) -> Option<BuildIdentity> {
    // GITHUB_ACTIONS
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        let build_id = std::env::var("GITHUB_RUN_ID").ok();
        return Some(BuildIdentity {
            system: "github-actions".to_string(),
            build_id,
            reproducible,
        });
    }
    // GITLAB_CI
    if std::env::var("GITLAB_CI").is_ok() {
        let build_id = std::env::var("CI_PIPELINE_ID").ok();
        return Some(BuildIdentity {
            system: "gitlab-ci".to_string(),
            build_id,
            reproducible,
        });
    }
    // CIRCLECI
    if std::env::var("CIRCLECI").is_ok() {
        let build_id = std::env::var("CIRCLE_BUILD_NUM").ok();
        return Some(BuildIdentity {
            system: "circleci".to_string(),
            build_id,
            reproducible,
        });
    }
    // JENKINS
    if std::env::var("JENKINS_URL").is_ok() {
        let build_id = std::env::var("BUILD_ID").ok();
        return Some(BuildIdentity {
            system: "jenkins".to_string(),
            build_id,
            reproducible,
        });
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::pad::Scope;
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
        std::fs::write(p.join("file.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(p)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(p)
            .output()
            .unwrap();
        dir
    }

    fn default_options() -> GenerateOptions {
        GenerateOptions {
            artifact_paths: vec![],
            scope: Scope::Stable,
            build_system: None,
            build_id: None,
            reproducible: false,
            model: None,
            reviewers: vec![],
            contribution_pct: None,
            output_path: None,
        }
    }

    #[test]
    fn generate_pad_basic() {
        let dir = init_repo();
        let config = Config::default_with_name("testpkg".to_string());
        let pad = generate_pad(&config, dir.path(), &default_options()).unwrap();
        assert_eq!(pad.package, "testpkg");
        assert_eq!(pad.trustver_spec, "0.3.0");
        assert_eq!(pad.scope, Scope::Stable);
        assert!(!pad.identity.source.commit.is_empty());
        assert!(pad.signatures.is_empty());
        assert!(pad.attestations.is_empty());
    }

    #[test]
    fn generate_pad_with_artifact_hash() {
        let dir = init_repo();
        let artifact = dir.path().join("artifact.bin");
        std::fs::write(&artifact, b"artifact content").unwrap();
        let config = Config::default_with_name("testpkg".to_string());
        let mut opts = default_options();
        opts.artifact_paths = vec![artifact];
        let pad = generate_pad(&config, dir.path(), &opts).unwrap();
        assert!(pad.identity.artifact_hashes.contains_key("sha256"));
        assert_eq!(pad.identity.artifact_hashes["sha256"].len(), 64);
    }

    #[test]
    fn generate_pad_with_authorship_detail() {
        let dir = init_repo();
        let config = Config::default_with_name("testpkg".to_string());
        let mut opts = default_options();
        opts.scope = Scope::Preview;
        opts.build_system = Some("local".to_string());
        opts.model = Some("claude-opus-4-6".to_string());
        opts.reviewers = vec!["jascha@tarnover.com".to_string()];
        opts.contribution_pct = Some(85);
        let pad = generate_pad(&config, dir.path(), &opts).unwrap();
        assert_eq!(
            pad.authorship.detail.ai_model.as_deref(),
            Some("claude-opus-4-6")
        );
        assert_eq!(pad.authorship.detail.ai_contribution_pct, Some(85));
        assert_eq!(
            pad.authorship.detail.human_reviewers,
            vec!["jascha@tarnover.com"]
        );
        assert_eq!(pad.scope, Scope::Preview);
        assert_eq!(pad.identity.build.as_ref().unwrap().system, "local");
    }

    #[test]
    fn contribution_pct_over_100_fails() {
        let dir = init_repo();
        let config = Config::default_with_name("testpkg".to_string());
        let mut opts = default_options();
        opts.contribution_pct = Some(101);
        assert!(generate_pad(&config, dir.path(), &opts).is_err());
    }
}
