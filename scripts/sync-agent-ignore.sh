#!/usr/bin/env bash
# Regenerates .claude/settings.json's Read-deny rules from .agentignore, so
# Claude Code (which has no external ignore-file mechanism) enforces the same
# patterns as the gitignore-syntax .agentignore without hand-duplicating them.
#
# Usage:
#   scripts/sync-agent-ignore.sh          # regenerate .claude/settings.json
#   scripts/sync-agent-ignore.sh --check  # verify it's already in sync (no write)
set -euo pipefail

cd "$(dirname "$0")/.."

deny=()
while IFS= read -r line || [[ -n "$line" ]]; do
  line="${line%$'\r'}"
  [[ -z "$line" || "$line" == \#* ]] && continue
  if [[ "$line" == */ ]]; then
    deny+=("Read(./**/${line%/}/**)")
  else
    deny+=("Read(./**/${line})")
  fi
done < .agentignore

generated="$(printf '%s\n' "${deny[@]}" | jq -R . | jq -s '{permissions: {deny: .}}')"

if [[ "${1:-}" == "--check" ]]; then
  current="$(cat .claude/settings.json 2>/dev/null || true)"
  if [[ "$current" != "$generated" ]]; then
    echo "error: .claude/settings.json is stale — run 'just sync-agent-ignore' to regenerate" >&2
    exit 1
  fi
  echo ".claude/settings.json is in sync with .agentignore"
else
  mkdir -p .claude
  printf '%s\n' "$generated" > .claude/settings.json
  echo "Regenerated .claude/settings.json from .agentignore"
fi
