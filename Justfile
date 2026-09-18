# Project Task Runner

# List available recipes
default:
    @just --list

# Format code and configuration files
format: sync-agent-ignore
    @echo "No formatter configured yet. Customize this recipe in the Justfile!"

# Run code and markdown linting checks
lint:
    @echo "No linter configured yet. Customize this recipe in the Justfile!"

# Regenerate .claude/settings.json's Read-deny rules from .agentignore
sync-agent-ignore:
    @scripts/sync-agent-ignore.sh

# Check that .claude/settings.json is in sync with .agentignore
check-agent-ignore-sync:
    @scripts/sync-agent-ignore.sh --check

# Run all local checks (tests, format checks, lints)
validate:
    @echo "Running project validations..."
    just check-agent-ignore-sync
    just lint
