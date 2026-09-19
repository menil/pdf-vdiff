# AI Decision Record: Allow Idempotent Publish Runs on Unversioned Main Commits

## Context & Goal
When non-release commits (such as documentation updates or workflow adjustments) are pushed or merged into `main`, the `publish.yml` workflow runs. By default, `katyo/publish-crates@v2` performs a strict consistency check that fails the entire build if repository files were modified since the last published release without a corresponding version bump in `Cargo.toml`.

## Architecture & Key Decisions
- **Enable `ignore-unpublished-changes: true`**: Configures `katyo/publish-crates@v2` to skip publishing gracefully (exit code 0) when files have changed without a version bump.
- **Maintain Release Safety**: When a developer intentionally bumps `version` in `Cargo.toml`, `katyo/publish-crates` detects the new version, compiles, uploads to crates.io, and triggers the automated Git tagging and GitHub Release creation.

## Alternatives Considered & Rejected
- Triggering publish workflow only on release/tag: Rejected because the project goal is automatic publishing to crates.io directly on merge to `main` when `Cargo.toml` is bumped, with automatic Git tag creation upon publication.
