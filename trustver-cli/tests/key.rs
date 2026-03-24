use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn key_generate_creates_files() {
    let dir = TempDir::new().unwrap();
    let key_dir = dir.path().join("keys");

    Command::cargo_bin("trustver").unwrap()
        .args(["key", "generate", "--output-dir", key_dir.to_str().unwrap(), "--name", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Key ID:"));

    assert!(key_dir.join("test-private.pem").exists());
    assert!(key_dir.join("test-public.pem").exists());
}
