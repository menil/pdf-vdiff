# `pdf-vdiff`

[![Crates.io](https://img.shields.io/crates/v/pdf-vdiff.svg)](https://crates.io/crates/pdf-vdiff)
[![docs.rs](https://docs.rs/pdf-vdiff/badge.svg)](https://docs.rs/pdf-vdiff)
[![CI](https://github.com/menil/pdf-vdiff/actions/workflows/validate.yml/badge.svg)](https://github.com/menil/pdf-vdiff/actions/workflows/validate.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, standalone CLI tool for side-by-side visual PDF diffing, designed primarily to inspect and verify changes between a base resume/CV and an AI-tailored version while preserving 100% vector fidelity and text searchability. It is used by [JobGitOps](https://github.com/menil/JobGitOps) to inspect resume diffs before submission.

`pdf-vdiff` compares two PDF documents (such as your canonical base resume/CV and an AI-customized version for a job application) and generates a single side-by-side landscape PDF with IntelliJ/GitHub-style visual diff highlighting. It allows you to immediately spot reworded bullet points, added keywords, and omitted sections before submitting your application.

---
<img width="1439" height="959" alt="Screenshot 2026-09-18 at 16 51 54" src="https://github.com/user-attachments/assets/0c0c2b74-d0ea-451a-a942-d1fa0b930bc3" />

---

## Features

- 📄 **100% Vector Fidelity**: Embeds native PDF page objects directly—no lossy rasterization or blurry text.
- 🔍 **Full Text Searchability**: Diff output maintains searchable and selectable text layers.
- 🧠 **Intelligent Hierarchical Diffing**: Extracts text tokens with exact bounding boxes via PDFium, clusters lines and columns spatially, and computes Myers sequence diffs at word, line, or character granularity.
- 🎨 **Multiple Color Themes**: Choose between `intellij` (default), `github`, `classic`, and `high-contrast` palettes.
- ⚡ **Zero-Friction CLI**: Automatically opens output in your system default PDF viewer (`--open`), supports custom gutter widths, optional header banners, and structured exit codes.
- 🛡️ **Hardened Ingestion**: Pre-flight dimension and page count validation, with PDFium V8 script execution explicitly disabled for security.

---

## Example

An example comparison of a base resume against a tailored resume is provided in the [`example/`](example/) directory:

- [`example/resume_base.pdf`](example/resume_base.pdf) — Base document
- [`example/resume_tailored.pdf`](example/resume_tailored.pdf) — Modified / tailored document
- [`example/resume_diff.pdf`](example/resume_diff.pdf) — Generated side-by-side visual diff

To generate the diff for this example:

```bash
pdf-vdiff example/resume_base.pdf example/resume_tailored.pdf -o example/resume_diff.pdf
```

To automatically open the diff in your system viewer after generation:

```bash
pdf-vdiff example/resume_base.pdf example/resume_tailored.pdf -o example/resume_diff.pdf --open
```

---

## Installation & Setup

### CLI via Cargo (Crates.io)

```bash
cargo install pdf-vdiff
```

### As a Rust Library

Add `pdf-vdiff` to your `Cargo.toml`:

```bash
cargo add pdf-vdiff
```

Or manually:

```toml
[dependencies]
pdf-vdiff = "0.1.1" # x-release-please-version
```

Programmatic usage in Rust:

```rust
use pdf_vdiff::{diff_documents, DiffGranularity, PageText};

// Diff page tokens with configurable granularity (Word, Line, Character)
let diff_result = diff_documents(&base_pages, &tailored_pages, DiffGranularity::Word);

println!(
    "Detected {} difference(s) across {} page(s)",
    diff_result.total_differences,
    diff_result.pages.len()
);
```

### Using Nix Flakes

`pdf-vdiff` provides full Nix Flake support (requires Nix 2.4+ with `flakes` and `nix-command` enabled). You can run it instantly without manual installation or dependencies:

```bash
# Run directly from GitHub
nix run github:menil/pdf-vdiff -- base.pdf tailored.pdf --open

# Install to your global Nix profile ($PATH)
nix profile install github:menil/pdf-vdiff
```

#### In Another Repository's `flake.nix`

Add `pdf-vdiff` as an input to include it in your developer shells or packages:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pdf-vdiff.url = "github:menil/pdf-vdiff";
    pdf-vdiff.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, pdf-vdiff, ... }:
    let
      system = "aarch64-darwin"; # or "x86_64-linux", etc.
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pdf-vdiff.packages.${system}.default
        ];
      };
    };
}
```

### Traditional Nix Shell

If you are developing inside this repository, enter the pre-configured developer shell:

```bash
nix-shell # or nix develop
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/menil/pdf-vdiff.git
cd pdf-vdiff

# Build release binary
cargo build --release

# Binary will be available at ./target/release/pdf-vdiff
```

---

## Usage

```bash
pdf-vdiff [OPTIONS] <BASE_PDF> <TAILORED_PDF>
```

### Options & Flags

| Option / Flag | Description | Default |
| :--- | :--- | :--- |
| `<BASE_PDF>` | Path to the original / base PDF document | *(Required)* |
| `<TAILORED_PDF>` | Path to the modified / tailored PDF document | *(Required)* |
| `-o, --output <PATH>` | Target output PDF file path | `<base_stem>_vs_<tailored_stem>_diff.pdf` |
| `-f, --force` | Overwrite destination output file if it already exists | `false` |
| `--open` | Automatically open the generated diff in default PDF viewer | `false` |
| `--theme <THEME>` | Color palette (`intellij`, `github`, `classic`, `high-contrast`) | `intellij` |
| `--granularity <MODE>` | Diff granularity (`word`, `line`, `character`) | `word` |
| `--gutter-width <PT>` | Spacing in points between left and right pages | `24.0` |
| `--no-header` | Suppress the top header/metadata banner | `false` |
| `--max-pages <NUM>` | Maximum page count threshold to prevent unbounded processing | `250` |
| `-v, --verbose` | Enable verbose structural logging | `false` |
| `--generate-completions <SHELL>` | Generate shell completions (`bash`, `zsh`, `fish`, `elvish`, `powershell`) | |
| `--generate-man` | Generate Section 1 roff man page to stdout | `false` |
| `-h, --help` | Print help information | |
| `-V, --version` | Print version information | |

### Examples

```bash
# Compare two documents and open immediately in system default PDF viewer
pdf-vdiff original.pdf modified.pdf --open

# Use GitHub color theme with line-level diff granularity
pdf-vdiff original.pdf modified.pdf --theme github --granularity line

# Custom output destination and overwrite if target exists
pdf-vdiff base.pdf tailored.pdf -o ./output/diff.pdf --force

# Remove the top metadata banner and adjust gutter width
pdf-vdiff v1.pdf v2.pdf --no-header --gutter-width 32.0
```

### Shell Completions & Man Pages

Pre-built shell completions and Unix man pages are maintained in [`completions/`](completions/) and [`man/`](man/):

#### Shell Completions
Generate or source completions on the fly for your active shell:

```bash
# Bash (add to ~/.bashrc)
eval "$(pdf-vdiff --generate-completions bash)"

# Zsh (add to ~/.zshrc)
eval "$(pdf-vdiff --generate-completions zsh)"

# Fish (add to ~/.config/fish/config.fish)
pdf-vdiff --generate-completions fish | source
```

#### Man Pages & TLDR
- **Unix Man Page**: Installed automatically with package managers (`man pdf-vdiff`), or view directly:
  - **macOS**: `pdf-vdiff --generate-man | mandoc`
  - **Linux**: `pdf-vdiff --generate-man | man -l -`
  - **Direct file**: `man man/pdf-vdiff.1`
- **TLDR Cheatsheet**: Available in [`man/pdf-vdiff.tldr.md`](man/pdf-vdiff.tldr.md).

### Exit Codes

| Code | Status | Description |
| :---: | :--- | :--- |
| `0` | **Identical Documents** | No differences detected. Generates diff PDF without highlights and exits 0. |
| `1` | **Differences Found** | Differences detected. Generates diff PDF with highlights and exits 1. |
| `2` | **Execution Error** | Fatal error (missing file, invalid permissions without `--force`, parse error). |

---

## Development

A [`Justfile`](Justfile) task runner is included for common development workflows:

```bash
# List available recipes
just

# Run test suite
just test

# Check formatting and linting
just lint

# Run all validations (tests, linter, format check, coverage)
just validate
```

For comprehensive guidelines on Nix environment setup, Git hooks, Beads issue tracking, and AI ignore policies, see the [Development Guide](DEVELOPMENT.md).

---

## License

This project is licensed under the [MIT License](LICENSE).
