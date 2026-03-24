use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn hook_install_creates_file() {
    let dir = TempDir::new().unwrap();
    std::process::Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();

    Command::cargo_bin("trustver").unwrap()
        .args(["hook", "install"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed commit-msg hook"));

    let hook = dir.path().join(".git/hooks/commit-msg");
    assert!(hook.exists());
    let content = std::fs::read_to_string(&hook).unwrap();
    assert!(content.contains("trustver check-commit"));
}

#[test]
fn hook_install_refuses_overwrite() {
    let dir = TempDir::new().unwrap();
    std::process::Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
    std::fs::write(dir.path().join(".git/hooks/commit-msg"), "existing").unwrap();

    Command::cargo_bin("trustver").unwrap()
        .args(["hook", "install"])
        .current_dir(dir.path())
        .assert()
        .code(2);
}

#[test]
fn hook_install_force_overwrites() {
    let dir = TempDir::new().unwrap();
    std::process::Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
    std::fs::write(dir.path().join(".git/hooks/commit-msg"), "existing").unwrap();

    Command::cargo_bin("trustver").unwrap()
        .args(["hook", "install", "--force"])
        .current_dir(dir.path())
        .assert()
        .success();
}
