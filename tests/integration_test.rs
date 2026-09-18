//! End-to-end integration tests exercising the pdf-vdiff CLI binary across all synthetic fixtures.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_integration_identical_fixtures() {
    let dir = fixtures_dir();
    let base = dir.join("identical_base.pdf");
    let target = dir.join("identical_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("identical_diff.pdf");

    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("are identical"));

    assert!(out.exists());
}

#[test]
fn test_integration_edit_fixtures() {
    let dir = fixtures_dir();
    let base = dir.join("edit_base.pdf");
    let target = dir.join("edit_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("edit_diff.pdf");

    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("Differences detected"));

    assert!(out.exists());
    assert!(std::fs::metadata(&out).unwrap().len() > 1000);
}

#[test]
fn test_integration_multicolumn_fixtures() {
    let dir = fixtures_dir();
    let base = dir.join("multicolumn_base.pdf");
    let target = dir.join("multicolumn_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("mc_diff.pdf");

    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg("github")
        .arg("--granularity")
        .arg("word")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("Differences detected"));

    assert!(out.exists());
}

#[test]
fn test_integration_page_mismatch_fixtures() {
    let dir = fixtures_dir();
    let base = dir.join("page_mismatch_base.pdf");
    let target = dir.join("page_mismatch_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("pm_diff.pdf");

    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg("classic")
        .arg("--no-header")
        .arg("-v")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("Differences detected"));

    assert!(out.exists());
}

#[test]
fn test_integration_no_text_layer_error() {
    let dir = fixtures_dir();
    let no_text = dir.join("no_text_image.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("err_diff.pdf");

    // Both with no text
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&no_text)
        .arg(&no_text)
        .arg("-o")
        .arg(&out)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "contains no extractable text layer",
        ));
}

#[test]
fn test_integration_file_not_found_error() {
    let dir = fixtures_dir();
    let target = dir.join("edit_target.pdf");
    let nonexistent = dir.join("does_not_exist_file.pdf");

    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&nonexistent)
        .arg(&target)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_integration_overwrite_guard_and_force() {
    let dir = fixtures_dir();
    let base = dir.join("edit_base.pdf");
    let target = dir.join("edit_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out = out_dir.path().join("existing_file.pdf");

    std::fs::write(&out, b"pre-existing data").expect("write existing");

    // 1. Without --force -> exit code 2
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("already exists"));

    // 2. With --force -> exit code 1 and file overwritten
    let mut cmd_force = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd_force
        .arg(&base)
        .arg(&target)
        .arg("-o")
        .arg(&out)
        .arg("-f")
        .assert()
        .failure()
        .code(1);

    let content = std::fs::read(&out).expect("read");
    assert_ne!(content, b"pre-existing data");
    assert!(content.starts_with(b"%PDF-"));
}

#[test]
fn test_integration_theme_and_granularity_variants() {
    let dir = fixtures_dir();
    let base = dir.join("edit_base.pdf");
    let target = dir.join("edit_target.pdf");
    let out_dir = tempfile::tempdir().expect("tempdir");

    for theme in &["intellij", "github", "classic", "high-contrast"] {
        for gran in &["word", "line", "character"] {
            let out = out_dir.path().join(format!("diff_{}_{}.pdf", theme, gran));
            let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
            cmd.arg(&base)
                .arg(&target)
                .arg("-o")
                .arg(&out)
                .arg("--theme")
                .arg(theme)
                .arg("--granularity")
                .arg(gran)
                .arg("--gutter-width")
                .arg("32")
                .assert()
                .failure()
                .code(1);
            assert!(out.exists());
        }
    }
}

#[test]
fn test_integration_private_fixtures_if_present() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base = manifest.join("tests/fixtures_private/base.pdf");
    let target = manifest.join("tests/fixtures_private/tailored.pdf");

    if base.exists() && target.exists() {
        let out_dir = tempfile::tempdir().expect("tempdir");
        let out = out_dir.path().join("private_diff.pdf");

        let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
        cmd.arg(&base)
            .arg(&target)
            .arg("-o")
            .arg(&out)
            .arg("-f")
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::contains("Differences detected"));

        assert!(out.exists());
        assert!(std::fs::metadata(&out).unwrap().len() > 10000);
    }
}
