# AI Decision Record: Synthetic Test Fixtures, E2E Integration Suite, and 85% Code Coverage Verification

## Context & Problem Statement
`pdf-vdiff` requires a suite of lightweight (<15 KB), public synthetic test PDF fixtures committed to the repository, alongside an end-to-end integration test suite exercising the compiled binary across all supported CLI flags, themes, error branches, and edge cases to maintain a strict >= 85% test coverage standard.

## Architectural Decisions
1. **Lightweight Synthetic Fixtures**:
   - `identical_base.pdf` & `identical_target.pdf`: Single-page document pair asserting exit code `0`.
   - `edit_base.pdf` & `edit_target.pdf`: Single-page resume document pair with modifications, additions, and deletions asserting exit code `1`.
   - `multicolumn_base.pdf` & `multicolumn_target.pdf`: Two-column resume layout testing column bounding-box ordering.
   - `page_mismatch_base.pdf` (2 pages) & `page_mismatch_target.pdf` (1 page): Testing uneven document length rendering.
   - `no_text_image.pdf`: Single page with zero extractable text tokens asserting exit code `2` (`NoTextLayer`).

2. **Integration Test Suite**:
   - Automated via `assert_cmd` and `predicates` testing all 3-tier exit codes (`0`, `1`, `2`), error output formats, flag matrices (themes `intellij`, `github`, `classic`, `high-contrast`, granularities `word`, `line`, `character`, `--no-header`, `--verbose`, `--gutter-width`), and destination overwrite protection (`--force`).
   - Non-destructive hook for local private fixtures in `tests/fixtures_private/` when present.

3. **Coverage & Quality Verification**:
   - Reached **90.45% overall line coverage** across the workspace (`diff.rs`: 90.26%, `layout.rs`: 97.30%, `cluster.rs`: 98.40%, `pdf/extract.rs`: 93.48%, `pdf/composer.rs`: 86.89%, `main.rs`: 94.57%, `cli.rs`: 100%).

## Verification & Validation
- Executed `nix-shell --run "just validate"` successfully across formatting, linting, 47 unit/integration tests, and llvm-cov line threshold verification.
