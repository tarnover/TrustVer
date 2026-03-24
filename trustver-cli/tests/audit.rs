use assert_cmd::Command;
use predicates::prelude::*;
use std::process;
use tempfile::TempDir;

fn init_repo_with_commits() -> TempDir {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    process::Command::new("git")
        .args(["init"])
        .current_dir(p)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["config", "user.email", "t@t.com"])
        .current_dir(p)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["config", "user.name", "T"])
        .current_dir(p)
        .output()
        .unwrap();

    std::fs::write(p.join("a.txt"), "hello").unwrap();
    process::Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["commit", "-m", "feat: initial [h]\n\nAuthorship: h"])
        .current_dir(p)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["tag", "v0.1.0"])
        .current_dir(p)
        .output()
        .unwrap();

    std::fs::write(p.join("b.txt"), "world").unwrap();
    process::Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    process::Command::new("git")
        .args([
            "commit",
            "-m",
            "feat: second [hrai]\n\nAuthorship: hrai\nReviewer: t@t.com",
        ])
        .current_dir(p)
        .output()
        .unwrap();

    dir
}

#[test]
fn audit_with_explicit_range() {
    let dir = init_repo_with_commits();
    Command::cargo_bin("trustver")
        .unwrap()
        .args(["audit", "v0.1.0..HEAD"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Derived tag:"));
}

#[test]
fn audit_json_mode() {
    let dir = init_repo_with_commits();
    Command::cargo_bin("trustver")
        .unwrap()
        .args(["audit", "--json", "v0.1.0..HEAD"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("derived_tag"));
}
