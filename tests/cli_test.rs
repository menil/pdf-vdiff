use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_or_version_output() {
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pdf-vdiff v0.1.0"));
}
