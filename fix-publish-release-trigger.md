# AI Decision Record: Fix Release Trigger Condition in Crates.io Publish Workflow

## Context & Goal
The automated crates.io publication workflow (`.github/workflows/publish.yml`) triggers on push to `main`. When a new version is published, it is expected to create a corresponding Git tag (`v<version>`) and GitHub Release with auto-generated release notes. In the initial publish run, `katyo/publish-crates@v2` successfully published `pdf-vdiff v0.1.0` to crates.io, but the subsequent tagging step was skipped because the condition `steps.publish.outputs.published == 'true'` evaluated to `false`.

## Architecture & Key Decisions
- **Output Data Format Handling**: `katyo/publish-crates@v2` returns `published` as a serialized JSON array string (e.g., `'[{"name":"pdf-vdiff","version":"0.1.0"}]'` on publish, or `'[]'` when skipped).
- **Evaluation Guard**: Updated the step conditional to:
  ```yaml
  if: steps.publish.outputs.published != '' && steps.publish.outputs.published != '[]'
  ```
  This string comparison correctly detects non-empty publish output without risking expression parsing errors when no packages are published.

## Alternatives Considered & Rejected
- `fromJSON(steps.publish.outputs.published)[0]`: While valid, string comparison against non-empty string and non-empty list (`''` and `'[]'`) is simple and robust across runner environments.
