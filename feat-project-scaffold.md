# AI Decision Record: Rust Project Scaffold and Toolchain Configuration

## Context & Goal
Initialize the `pdf-vdiff` repository with a reproducible Nix developer shell, Task runner recipes (`Justfile`), dual-target library/binary Cargo layout, and private test fixture ignore rules.

## Architecture & Key Decisions
1. **Nix-Shell Toolchain with LLVM Tools**:
   - Packaged `rustc`, `cargo`, `clippy`, `rustfmt`, `cargo-llvm-cov`, and `llvmPackages.llvm` directly in `shell.nix`.
   - Exported `LLVM_COV` and `LLVM_PROFDATA` in `shellHook` so coverage checks run seamlessly in any developer or CI environment.
2. **Dual-Target Crate Layout**:
   - `src/lib.rs` (`pdf_vdiff`) and `src/main.rs` (`pdf-vdiff`) configured with full dependencies (`clap`, `pdfium-render`, `similar`, `unicode-normalization`, `thiserror`, `anyhow`, `open`, `tempfile`).
3. **Private Testing Privacy Guard**:
   - Added `tests/fixtures_private/` and `*.local.pdf` to `.gitignore` and `.agentignore` to ensure local real-world PDFs are never tracked.

## Alternatives Considered & Rejected
- *Single-binary crate*: Rejected because unit testing core engines and reaching the 85% coverage bar without binary subprocesses requires a library root (`src/lib.rs`).
