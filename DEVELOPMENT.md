# Development Guide

This document outlines developer tooling, workflow conventions, and validation processes for contributing to `pdf-vdiff`.

---

## 1. Developer Environment

### Nix Shell (Recommended)

The project includes a `shell.nix` configuration providing Rust, `PDFium` dynamic libraries, `just`, and development tools:

```bash
nix-shell
```

Alternatively, if you use `direnv`, running `direnv allow` automatically loads the Nix environment and local hooks upon entering the directory.

---

## 2. Task Runner (`Justfile`)

Common development tasks are managed via [`just`](https://github.com/casey/just):

| Command | Description |
| :--- | :--- |
| `just` | List all available tasks |
| `just format` | Format Rust source code (`cargo fmt`) and sync agent ignore settings |
| `just lint` | Run format checks and Clippy linter with strict `-D warnings` |
| `just test` | Run unit and integration tests (single-threaded for PDFium C safety) |
| `just coverage` | Run `cargo-llvm-cov` with 85% line-coverage threshold |
| `just build` | Build release binary (`cargo build --release`) |
| `just sync-agent-ignore` | Regenerate `.claude/settings.json` deny rules from `.agentignore` |
| `just check-agent-ignore-sync` | Check whether `.claude/settings.json` is in sync with `.agentignore` |
| `just validate` | Run full validation suite (agent sync, lint, test, and coverage) |

---

## 3. Git Hooks & Conventions

### Conventional Commits
All commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification (e.g. `feat: ...`, `fix: ...`, `docs: ...`). Commit titles must remain under 72 characters. A `commit-msg` hook verifies this locally.

### Pre-commit Validation
A `pre-commit` hook automatically runs `just validate` before allowing a commit to be created.

### Continuous Integration (CI)
A GitHub Actions workflow executes `just validate` on all pushes and pull requests to enforce formatting, linting, tests, and code coverage checks.

---

## 4. Beads Issue Tracking

The repository supports issue tracking via the [Beads](https://github.com/menil/beads) (`bd`) CLI tool:
- Find ready tasks: `bd ready`
- View task details: `bd show <task_id>`
- Close completed task: `bd close <task_id>`

---

## 5. AI Agent Ignore Rules

`.agentignore` at the repository root defines paths that AI coding agents should not inspect (e.g., build artifacts, caches, dependencies):
- **Antigravity**: `.antigravityignore` symlinks to `.agentignore`.
- **Claude Code**: `.claude/settings.json` Read-deny rules are synchronized from `.agentignore` via `scripts/sync-agent-ignore.sh`.
- Run `just format` or `just sync-agent-ignore` after modifying `.agentignore`.
