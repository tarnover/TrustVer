use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn init_creates_config() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("trustver")
        .unwrap()
        .args(["init", "--name", "testpkg"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created trustver.toml"));

    let content = std::fs::read_to_string(dir.path().join("trustver.toml")).unwrap();
    assert!(content.contains("testpkg"));
    assert!(content.contains("0.1.0+mix"));
}

#[test]
fn init_refuses_if_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("trustver.toml"), "existing").unwrap();

    Command::cargo_bin("trustver")
        .unwrap()
        .args(["init", "--name", "testpkg"])
        .current_dir(dir.path())
        .assert()
        .code(2);
}
