//! Standalone asset generator for `pdf-vdiff`.
//! Generates man pages (Section 1 roff and tldr) and shell completions for Bash, Zsh, Fish, Elvish, and PowerShell.

use clap_complete::Shell;
use pdf_vdiff::cli::{render_completions, render_man_page};
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const TLDR_CONTENT: &str = r#"# pdf-vdiff

> Fast CLI tool for side-by-side visual PDF diffing with vector fidelity.
> More information: <https://github.com/menil/pdf-vdiff>.

- Compare two PDF files and save the diff to a default filename:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}}

- Compare two PDFs and specify a custom output path:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} -o {{path/to/diff.pdf}}

- Automatically open the resulting diff in the default PDF viewer:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --open

- Diff using a specific color theme (e.g. github, intellij, classic, high-contrast):
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --theme {{github}}

- Adjust diff granularity to line or character level:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --granularity {{line|character|word}}

- Force overwrite an existing output file:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} -f -o {{path/to/diff.pdf}}
"#;

/// Generates all documentation and shell completion assets into `root_dir/man` and `root_dir/completions`.
pub fn generate_all_assets(root_dir: &Path) -> io::Result<()> {
    let man_dir = root_dir.join("man");
    let comp_dir = root_dir.join("completions");

    create_dir_all(&man_dir)?;
    create_dir_all(&comp_dir)?;

    // 1. Man page (roff)
    let man_path = man_dir.join("pdf-vdiff.1");
    let mut man_file = File::create(&man_path)?;
    render_man_page(&mut man_file)?;
    println!("✓ Generated man page: {}", man_path.display());

    // 2. tldr cheatsheet
    let tldr_path = man_dir.join("pdf-vdiff.tldr.md");
    let mut tldr_file = File::create(&tldr_path)?;
    tldr_file.write_all(TLDR_CONTENT.as_bytes())?;
    println!("✓ Generated tldr page: {}", tldr_path.display());

    // 3. Shell completions
    let shells = [
        (Shell::Bash, "pdf-vdiff.bash"),
        (Shell::Zsh, "_pdf-vdiff"),
        (Shell::Fish, "pdf-vdiff.fish"),
        (Shell::Elvish, "pdf-vdiff.elv"),
        (Shell::PowerShell, "_pdf-vdiff.ps1"),
    ];

    for (shell, filename) in shells {
        let path = comp_dir.join(filename);
        let mut file = File::create(&path)?;
        render_completions(shell, &mut file)?;
        file.flush()?;
        println!(
            "✓ Generated shell completion ({shell:?}): {}",
            path.display()
        );
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let target_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    generate_all_assets(&target_dir)
}
