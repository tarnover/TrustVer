use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn check_valid_commit() {
    Command::cargo_bin("trustver").unwrap()
        .args(["check-commit", "feat(auth): add PKCE [hrai]\n\nBody.\n\nAuthorship: hrai\nReviewer: test@test.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conformant"));
}

#[test]
fn check_invalid_commit_no_tag() {
    Command::cargo_bin("trustver").unwrap()
        .args(["check-commit", "feat: some change"])
        .assert()
        .code(1);
}

#[test]
fn check_commit_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let msg_path = dir.path().join("COMMIT_MSG");
    std::fs::write(&msg_path, "feat: change [h]\n\nAuthorship: h").unwrap();

    Command::cargo_bin("trustver").unwrap()
        .args(["check-commit", "--file", msg_path.to_str().unwrap()])
        .assert()
        .success();
}
