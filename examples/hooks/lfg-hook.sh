#!/usr/bin/env bash
# LFG webhook hook — sends Claude Code / Cursor / Codex events directly to LFG.
# No dependencies beyond curl and jq.
#
# Usage (Claude Code):
#   Add to ~/.claude/settings.json under "hooks"
#
# Usage (Cursor):
#   Add to ~/.cursor/hooks.json
#
# Usage (Codex CLI):
#   Add to ~/.codex/config.toml
#
# The script reads hook JSON from stdin, extracts the event info,
# and POSTs it to LFG's webhook endpoint.

set -euo pipefail

LFG_URL="${LFG_URL:-http://127.0.0.1:5555/webhook}"
LFG_HOST="${LFG_HOST:-claude}"
LFG_TOKEN="${LFG_TOKEN:-}"

# Read hook payload from stdin
input=$(cat)

# Extract fields — Claude Code uses PascalCase, Cursor uses camelCase, Codex uses snake_case
hook_event=$(echo "$input" | jq -r '.hook_event_name // .hookEventName // empty' 2>/dev/null)
session_id=$(echo "$input" | jq -r '.session_id // .sessionId // .conversation_id // .conversationId // empty' 2>/dev/null)
tool_name=$(echo "$input" | jq -r '.tool_name // .toolName // empty' 2>/dev/null)

# Normalize event names to PascalCase (handles Cursor camelCase and Codex snake_case)
case "$hook_event" in
  preToolUse)           hook_event="PreToolUse" ;;
  postToolUse)          hook_event="PostToolUse" ;;
  beforeShellExecution) hook_event="PreToolUse"; tool_name="Bash" ;;
  afterShellExecution)  hook_event="PostToolUse"; tool_name="Bash" ;;
  afterFileEdit)        hook_event="PostToolUse"; tool_name="Edit" ;;
  stop)                 hook_event="Stop" ;;
  sessionStart)         hook_event="SessionStart" ;;
  sessionEnd)           hook_event="SessionEnd" ;;
  pre_tool_use)         hook_event="PreToolUse" ;;
  post_tool_use)        hook_event="PostToolUse" ;;
  post_tool_use_failure) hook_event="PostToolUse" ;;
  permission_request)   hook_event="PermissionRequest" ;;
  session_start)        hook_event="SessionStart" ;;
  session_stop)         hook_event="Stop" ;;
esac

[ -z "$hook_event" ] && exit 0

curl -sf -X POST "$LFG_URL?host=$LFG_HOST" \
  ${LFG_TOKEN:+-H "Authorization: Bearer $LFG_TOKEN"} \
  -H 'Content-Type: application/json' \
  -d "{\"text\": \"${hook_event}|${session_id}|${tool_name}\"}" \
  >/dev/null 2>&1 &

exit 0
