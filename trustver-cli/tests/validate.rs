use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn validate_valid_version() {
    Command::cargo_bin("trustver").unwrap()
        .args(["validate", "2.4.0+hrai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid TrustVer"));
}

#[test]
fn validate_invalid_version() {
    Command::cargo_bin("trustver").unwrap()
        .args(["validate", "2.4.0"])
        .assert()
        .code(1);
}

#[test]
fn validate_quiet_mode() {
    Command::cargo_bin("trustver").unwrap()
        .args(["validate", "--quiet", "2.4.0+hrai"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn validate_json_mode() {
    Command::cargo_bin("trustver").unwrap()
        .args(["validate", "--json", "2.4.0+hrai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\":true"));
}
