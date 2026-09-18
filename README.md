# Project Template

A generic, modern project template pre-configured with developer tooling, Nix integration, local git hook validation, and automated AI code reviews.

## Features

- 🤖 **Automated PR Reviews**: Integrated via `menil/pr-code-review-action` using OpenRouter (free tier by default).
- ❄️ **Nix Shell**: Pre-configured `shell.nix` for consistent, reproducible developer environments.
- 🛠️ **Local Task Runner (`Justfile`)**: Standardized commands for formatting, linting, and validating code.
- 🛡️ **Git Hooks**: Pre-configured conventional commit title checks and automatic pre-commit quality checks.
- ⚡ **Direnv Ready**: Automatically configures local git hooks and Beads issue tracking upon entering the directory.
- ✅ **CI Validation**: A `validate` GitHub Actions workflow runs `just validate` on every push/PR, so checks aren't only enforced by the (bypassable) local pre-commit hook.

---

## Getting Started

### 1. Create a Repository from this Template

Click the **"Use this template"** button on GitHub, or create it via the GitHub CLI:
```bash
gh repo create my-new-project --template menil/project-template --private --clone
```

### 2. Configure GitHub Secrets

For the automated PR code reviews to run successfully, navigate to your new repository's **Settings > Secrets and variables > Actions** and add:

* **`OPENROUTER_API_KEY`**: Your OpenRouter API Key.

*(Note: The template uses GitHub's Action Sharing to fetch `menil/pr-code-review-action` keylessly. Ensure you have configured the action repository under **Settings > Actions > General > Access** to be accessible from other repositories owned by your user account).*

---

## Development Environment

### Nix Shell
Activate the Nix developer shell to load project tools:
```bash
nix-shell
```

### Task Runner (`Justfile`)
The following tasks are available via `just`:
- `just`: List all available tasks.
- `just format`: Format code and configuration files (also regenerates `.claude/settings.json` from `.agentignore`).
- `just lint`: Run code and markdown linters.
- `just sync-agent-ignore`: Regenerate `.claude/settings.json`'s `Read` deny rules from `.agentignore`.
- `just check-agent-ignore-sync`: Verify `.claude/settings.json` is in sync with `.agentignore` (no write).
- `just validate`: Execute all formatting, linting, and verification checks.

### Git Hook Checks
The project automatically configures local Git hooks:
- **`commit-msg`**: Validates that all commit titles adhere to the [Conventional Commits](https://www.conventionalcommits.org/) standard (e.g. `feat: add database support`).
- **`pre-commit`**: Automatically runs `just validate` before allowing a commit. If any check fails, the commit is aborted.

These hooks only run locally and can be skipped (`git commit --no-verify`) or simply never installed (e.g. a contributor who hasn't run `direnv allow`, or a commit made through GitHub's web UI). The `validate` GitHub Actions workflow (`.github/workflows/validate.yml`) runs the same `just validate` in CI on every push and pull request as a backstop that can't be bypassed the same way.

### Beads Issue Tracking
`.beads/` (Beads' local issue database) is gitignored, so each clone provisions its own: `.envrc` runs `bd init --skip-agents --init-if-missing` on every `cd` into the repo, and `bd` itself no-ops once it's already initialized. `--skip-agents` deliberately omits `bd init`'s own `AGENTS.md`/`CLAUDE.md`/`.codex/`/`.claude/` generation: those agent instructions are already deployed globally to Claude Code, OpenCode, Codex, Gemini, and Pi via this machine's dotfiles (home-manager config), so a per-repo copy would just be a stale duplicate that's also at odds with the standing "don't commit agent-config directories" rule.

### AI Agent Ignore Files
`.agentignore` at the repo root is the canonical, gitignore-syntax list of paths AI coding agents shouldn't read (dependencies, build output, secrets, caches, etc.). Where an agent supports it, its ignore file is a symlink to `.agentignore` so the pattern list never drifts:

- **Google Antigravity**: `.antigravityignore` → `.agentignore`. Note there are [open reports](https://github.com/google-antigravity/antigravity-cli/issues/309) that the Antigravity CLI doesn't always fully respect this file in practice.
- **OpenCode**: has no native ignore-file support yet. The closest option is the community [`opencode-ignore`](https://github.com/lgladysz/opencode-ignore) plugin, which you install via `opencode.json` and which reads a `.ignore` file. If you adopt it, symlink `.ignore` to `.agentignore` the same way.
- **Claude Code**: has **no** `.claudeignore` (or any other external ignore-file) mechanism — a symlink here would be inert. It automatically respects `.gitignore`. For checked-in paths it can't reach that way (e.g. lock files, vendored code), Claude Code supports `Read` deny rules in `.claude/settings.json` — see the [large-codebases guide](https://code.claude.com/docs/en/large-codebases.md#block-reads-of-generated-and-vendored-code). This repo ships a generated `.claude/settings.json` (tracked in git, like Claude Code's own convention for shared project settings — only `.claude/settings.local.json` is gitignored) so it's enforced out of the box.

To update the pattern list, edit `.agentignore` — the symlinked files pick up the change automatically, and `just format` (or `just sync-agent-ignore` directly) regenerates `.claude/settings.json`'s deny rules from it via `scripts/sync-agent-ignore.sh`, so the two never drift. `just validate` fails if `.claude/settings.json` is stale.
