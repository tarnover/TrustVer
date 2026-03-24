use assert_cmd::Command;
use predicates::prelude::*;
use std::process;
use tempfile::TempDir;

fn init_repo_with_config() -> TempDir {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    process::Command::new("git").args(["init"]).current_dir(p).output().unwrap();
    process::Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(p).output().unwrap();
    process::Command::new("git").args(["config", "user.name", "T"]).current_dir(p).output().unwrap();
    std::fs::write(p.join("file.txt"), "hello").unwrap();
    process::Command::new("git").args(["add", "."]).current_dir(p).output().unwrap();
    process::Command::new("git").args(["commit", "-m", "initial [h]\n\nAuthorship: h"]).current_dir(p).output().unwrap();

    Command::cargo_bin("trustver").unwrap()
        .args(["init", "--name", "testpkg"])
        .current_dir(p)
        .assert()
        .success();

    dir
}

#[test]
fn pad_generate_creates_file() {
    let dir = init_repo_with_config();
    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "generate"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated PAD"));

    // Check a .pad.json file was created
    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().to_string_lossy().contains(".pad.json"))
        .collect();
    assert!(!entries.is_empty(), "no .pad.json file found");
}

#[test]
fn pad_validate_on_generated_pad() {
    let dir = init_repo_with_config();

    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "generate", "--output", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success();

    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "validate", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PAD is valid"));
}

#[test]
fn pad_sign_and_verify() {
    let dir = init_repo_with_config();
    let key_dir = dir.path().join(".trustver/keys");

    // Generate key
    Command::cargo_bin("trustver").unwrap()
        .args(["key", "generate", "--output-dir", key_dir.to_str().unwrap()])
        .assert()
        .success();

    // Generate PAD
    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "generate", "--output", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Sign PAD
    Command::cargo_bin("trustver").unwrap()
        .args([
            "pad", "sign", "test.pad.json",
            "--key", key_dir.join("trustver-private.pem").to_str().unwrap(),
            "--public-key", key_dir.join("trustver-public.pem").to_str().unwrap(),
            "--signer", "test@test.com",
        ])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Signed PAD"));

    // Verify
    Command::cargo_bin("trustver").unwrap()
        .args([
            "pad", "validate", "test.pad.json",
            "--verify",
            "--public-key", key_dir.join("trustver-public.pem").to_str().unwrap(),
        ])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn pad_attest_appends() {
    let dir = init_repo_with_config();

    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "generate", "--output", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success();

    Command::cargo_bin("trustver").unwrap()
        .args([
            "pad", "attest", "test.pad.json",
            "--attestation-type", "test-verified",
            "--attester", "ci@test.com",
            "--detail", r#"{"passed": 52}"#,
            "--unsigned",
        ])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1 attestation"));
}

#[test]
fn pad_validate_json_mode() {
    let dir = init_repo_with_config();

    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "generate", "--output", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success();

    Command::cargo_bin("trustver").unwrap()
        .args(["pad", "validate", "--json", "test.pad.json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\": true"));
}
