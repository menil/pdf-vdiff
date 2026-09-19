# AI Decision Record: Public Release Readiness Preparations

## Context & Goal
Prepare `pdf-vdiff` for public open-source release on GitHub and crates.io packaging. Ensure that the test suite does not dirty the working directory, lockfiles guarantee reproducible builds, repository metadata is complete, and CI workflows gracefully accommodate pull requests from external forks.

## Architecture & Key Decisions
1. **Isolated Tempdir Fixture Verification**:
   - `test_generate_fixtures_and_verify_sizes` was updated to generate synthetic PDFs into an isolated `tempfile::tempdir()` while checking committed fixtures against size bounds (< 15 KB).
   - Prevents `cargo test` from mutating committed `tests/fixtures/*.pdf` with dynamic PDFium creation timestamps.
   - Added `test_regenerate_committed_fixtures` marked `#[ignore]` for explicit developer regeneration when needed.
2. **Crate & Binary Alignment**:
   - Standardized `[package] name = "pdf-vdiff"` so `cargo install pdf-vdiff` matches both the binary name and repository slug, while keeping `[lib] name = "pdf_vdiff"`.
   - Enriched `Cargo.toml` with repository URLs, documentation links, categories (`command-line-utilities`, `text-processing`), and keywords.
3. **Deterministic Dependencies**:
   - Unignored and committed `Cargo.lock` (v4) to guarantee bit-for-bit reproducible dependency compilation in CI and downstream developer clones.
4. **CI Fork Safety**:
   - Added secret existence guards and a 10-minute timeout to `.github/workflows/pr-review.yml` to prevent automated review actions from failing on external fork pull requests that lack repository secrets.
5. **Git Hygiene**:
   - Cleaned up and consolidated `.gitignore` rules for Beads issue tracking, Dolt databases, and temporary diff files.

## Alternatives Considered & Rejected
- **Regenerating fixtures on pre-commit**: Rejected because committing new binary PDF timestamps on every test execution creates spurious git churn.
- **Excluding `Cargo.lock`**: Rejected because for CLI binary tools, a version-controlled lockfile is standard Rust best practice for reproducibility.
