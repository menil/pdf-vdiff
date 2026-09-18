# Project Task Runner

# List available recipes
default:
    @just --list

# Format code and configuration files
format: sync-agent-ignore
    cargo fmt

# Run code and markdown linting checks
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Run all unit and integration tests (single-threaded due to PDFium C library global state)
test:
    cargo test -- --test-threads=1

# Run code coverage with fail-under threshold (single-threaded due to PDFium C library global state)
coverage:
    cargo llvm-cov --fail-under-lines 85 -- --test-threads=1

# Build release binary
build:
    cargo build --release

# Regenerate .claude/settings.json's Read-deny rules from .agentignore
sync-agent-ignore:
    @scripts/sync-agent-ignore.sh

# Check that .claude/settings.json is in sync with .agentignore
check-agent-ignore-sync:
    @scripts/sync-agent-ignore.sh --check

# Run all local checks (tests, format checks, lints, coverage)
validate:
    @echo "Running project validations..."
    just check-agent-ignore-sync
    just lint
    just test
    just coverage
