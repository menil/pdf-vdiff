use assert_cmd::Command;
use clap_complete::Shell;
use pdf_vdiff::cli::{render_completions, render_man_page};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_generate_man_page() {
    let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
    cmd.arg("--generate-man")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH pdf-vdiff 1"))
        .stdout(predicate::str::contains("granularity"))
        .stdout(predicate::str::contains("theme"))
        .stdout(
            predicate::str::contains("gutter\\-width").or(predicate::str::contains("gutter-width")),
        )
        .stdout(predicate::str::contains("no\\-header").or(predicate::str::contains("no-header")));
}

#[test]
fn test_cli_generate_shell_completions() {
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Elvish,
        Shell::PowerShell,
    ] {
        let mut cmd = Command::cargo_bin("pdf-vdiff").unwrap();
        cmd.args(["--generate-completions", shell.to_string().as_str()])
            .assert()
            .success()
            .stdout(predicate::str::contains("pdf-vdiff"));
    }
}

#[test]
fn test_generate_assets_binary_and_committed_files_in_sync() {
    let temp_dir = tempfile::tempdir().expect("create tempdir");

    let mut cmd = Command::cargo_bin("generate-assets").unwrap();
    cmd.arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Generated man page"))
        .stdout(predicate::str::contains("✓ Generated tldr page"))
        .stdout(predicate::str::contains("✓ Generated shell completion"));

    // Check man page
    let gen_man =
        fs::read_to_string(temp_dir.path().join("man/pdf-vdiff.1")).expect("read gen man");
    let repo_man = fs::read_to_string(Path::new("man/pdf-vdiff.1")).expect("read repo man");
    assert_eq!(
        gen_man, repo_man,
        "Committed man/pdf-vdiff.1 is out of sync with generator"
    );

    // Check tldr
    let gen_tldr =
        fs::read_to_string(temp_dir.path().join("man/pdf-vdiff.tldr.md")).expect("read gen tldr");
    let repo_tldr = fs::read_to_string(Path::new("man/pdf-vdiff.tldr.md")).expect("read repo tldr");
    assert_eq!(
        gen_tldr, repo_tldr,
        "Committed man/pdf-vdiff.tldr.md is out of sync with generator"
    );

    // Check completions
    let shells = [
        (Shell::Bash, "completions/pdf-vdiff.bash"),
        (Shell::Zsh, "completions/_pdf-vdiff"),
        (Shell::Fish, "completions/pdf-vdiff.fish"),
        (Shell::Elvish, "completions/pdf-vdiff.elv"),
        (Shell::PowerShell, "completions/_pdf-vdiff.ps1"),
    ];

    for (_shell, rel_path) in shells {
        let gen_comp = fs::read_to_string(temp_dir.path().join(rel_path)).expect("read gen comp");
        let repo_comp = fs::read_to_string(Path::new(rel_path)).expect("read repo comp");
        assert_eq!(
            gen_comp, repo_comp,
            "Committed {} is out of sync with generator",
            rel_path
        );
    }
}

#[test]
fn test_render_helpers_direct() {
    let mut man_buf = Vec::new();
    render_man_page(&mut man_buf).expect("render man page");
    assert!(!man_buf.is_empty());

    let mut bash_buf = Vec::new();
    render_completions(Shell::Bash, &mut bash_buf).expect("render completions");
    assert!(!bash_buf.is_empty());
}
