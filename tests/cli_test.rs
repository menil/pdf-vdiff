use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_output() {
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("visual diff highlighting"));
}

#[test]
fn test_cli_version_output() {
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pdf-vdiff"));
}

#[test]
fn test_cli_missing_args_error() {
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Usage:"));
}
