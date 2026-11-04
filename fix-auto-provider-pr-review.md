# AI Decision Record: Automated Provider Fallback for PR Code Reviews

## Context & Goal
Previously, `.github/workflows/pr-review.yml` configured the PR code review action using a static ternary expression:
```yaml
provider: ${{ (secrets.ANTHROPIC_API_KEY || secrets.CLAUDE_CODE_OAUTH_TOKEN) && 'anthropic' || 'openrouter' }}
```
When `CLAUDE_CODE_OAUTH_TOKEN` was present in repository secrets, this evaluated to `provider: anthropic`. In `menil/pr-code-review-action`, specifying `provider: anthropic` strictly enforces provider isolation, preventing fallback to OpenRouter when the Claude Code token encounters authentication or permission issues (such as HTTP 403 `oauth_not_allowed_for_organization`), credit depletion, or 5xx server errors.

The goal is to update the PR review workflow to specify `provider: auto`, allowing the action to try the Claude Code OAuth token first and automatically fall back to OpenRouter upon encountering token or server errors.

## Architecture & Key Decisions

1. **Explicit `provider: auto` in CI Workflow**:
   - Updated `.github/workflows/pr-review.yml` to specify `provider: 'auto'`.
   - Updated comments explaining that the action automatically prioritizes Claude Code OAuth tokens and falls back to OpenRouter on token errors, credit depletion, or 5xx server errors.

2. **Validation**:
   - Validated that `nix-shell --run "just validate"` passes all checks (agent ignore sync, formatting, clippy lints, unit/integration tests, and coverage threshold enforcement).
