# AI Decision Record: Automatic Crates.io Publishing on Merge to Main

## Context & Goal
Provide automated, zero-friction crates.io release management for `pdf-vdiff`. Ensure that merging new versions to `main` publishes the crate to crates.io automatically using `CARGO_REGISTRY_TOKEN`, while gracefully skipping runs where the version is already published.

## Architecture & Key Decisions
1. **GitHub Action via `katyo/publish-crates@v2`**:
   - Integrated `katyo/publish-crates@v2` in `.github/workflows/publish.yml`.
   - Triggers on `push: branches: [main]`.
   - Checks crates.io registry before publishing; if the version in `Cargo.toml` is already present on crates.io, the step cleanly exits with success without failing or creating noisy alerts.
   - When a version bump commit is merged to `main`, it automatically authenticates with `CARGO_REGISTRY_TOKEN` and publishes the crate.
2. **Environment & Security**:
   - Scoped to `if: github.repository == 'menil/pdf-vdiff'` to avoid unintended execution on personal forks.
   - Concurrency group prevents race conditions during multiple rapid pushes to `main`.
   - Compilation verification during `cargo publish` uses stable Rust via `dtolnay/rust-toolchain@stable`.

## Alternatives Considered & Rejected
- **Manual publishing via local CLI**: Rejected because automated CI publishing ensures that releases are traceable, reproducible, and tied directly to canonical git commits on `main`.
- **Triggering on git tag push only**: Rejected in favor of version-detected publish on `main` merge, aligning with the user's preference for automated merge-to-main publishing.
