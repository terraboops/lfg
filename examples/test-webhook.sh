#!/usr/bin/env bash
# test-webhook.sh — curl-based integration tests for the lfg webhook API
#
# Usage:
#   ./examples/test-webhook.sh                      # localhost:6969
#   ./examples/test-webhook.sh 192.168.1.42         # remote host, default port
#   ./examples/test-webhook.sh 192.168.1.42 5555    # remote host, custom port
#   LFG_TOKEN=secret ./examples/test-webhook.sh ... # with auth token

HOST="${1:-localhost}"
PORT="${2:-5555}"
BASE="http://${HOST}:${PORT}"

PASS=0
FAIL=0

# ── helpers ────────────────────────────────────────────────────────────────────

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }

# Build auth header array (empty if no token set)
auth_headers=()
if [[ -n "$LFG_TOKEN" ]]; then
  auth_headers=(-H "Authorization: Bearer $LFG_TOKEN")
fi

webhook() {
  # webhook <host> <text>
  curl -sf -X POST "${BASE}/webhook?host=${1}" \
    "${auth_headers[@]}" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"${2}\"}"
}

status_field() {
  # status_field <jq-like path using python>  e.g. "columns[0]['slots'][0]['state']"
  curl -sf "${BASE}/status" "${auth_headers[@]}" | \
    python3 -c "import sys,json; s=json.load(sys.stdin); print(${1})" 2>/dev/null
}

assert_eq() {
  local label="$1" got="$2" want="$3"
  if [[ "$got" == "$want" ]]; then
    green "  PASS  $label"
    (( PASS++ ))
  else
    red   "  FAIL  $label"
    red   "        got:  $got"
    red   "        want: $want"
    (( FAIL++ ))
  fi
}

assert_contains() {
  local label="$1" haystack="$2" needle="$3"
  if [[ "$haystack" == *"$needle"* ]]; then
    green "  PASS  $label"
    (( PASS++ ))
  else
    red   "  FAIL  $label"
    red   "        expected to contain: $needle"
    red   "        got: $haystack"
    (( FAIL++ ))
  fi
}

reset() {
  curl -sf -X POST "${BASE}/reset" "${auth_headers[@]}" > /dev/null
}

first_slot_field() {
  # first_slot_field <field>  — reads from columns[0].slots[0]
  status_field "s['columns'][0]['slots'][0]['${1}']"
}

# ── connectivity ───────────────────────────────────────────────────────────────

echo
echo "lfg webhook tests → ${BASE}"
echo "──────────────────────────────────────────"

echo
echo "[ connectivity ]"

resp=$(curl -sf "${BASE}/status" "${auth_headers[@]}" 2>&1)
if [[ $? -eq 0 ]]; then
  green "  PASS  GET /status reachable"
  (( PASS++ ))
else
  red   "  FAIL  GET /status unreachable — is lfg running on ${BASE}?"
  exit 1
fi

resp=$(curl -sf -X POST "${BASE}/reset" "${auth_headers[@]}" 2>&1)
assert_contains "POST /reset returns ok" "$resp" '"ok":true'

# ── basic lifecycle ────────────────────────────────────────────────────────────

echo
echo "[ basic lifecycle ]"
reset

webhook "testhost" "PreToolUse|sess-001|Read" > /dev/null
assert_eq "PreToolUse → state=working"   "$(first_slot_field state)"     "working"
assert_eq "PreToolUse → tool_name=Read"  "$(first_slot_field tool_name)" "Read"
assert_eq "PreToolUse increments stats"  \
  "$(status_field "s['stats']['tool_calls']")" "1"

webhook "testhost" "PostToolUse|sess-001|Read" > /dev/null
assert_eq "PostToolUse → state stays working" "$(first_slot_field state)" "working"

webhook "testhost" "Stop|sess-001|" > /dev/null
assert_eq "Stop → state=idle"     "$(first_slot_field state)"     "idle"
assert_eq "Stop clears tool_name" "$(first_slot_field tool_name)" ""

# ── requesting is sticky ───────────────────────────────────────────────────────

echo
echo "[ requesting is sticky ]"
reset

webhook "testhost" "PreToolUse|sess-002|Bash" > /dev/null
webhook "testhost" "PermissionRequest|sess-002|" > /dev/null
assert_eq "PermissionRequest → state=requesting" "$(first_slot_field state)" "requesting"

webhook "testhost" "PreToolUse|sess-002|Read" > /dev/null
assert_eq "PreToolUse does NOT override requesting" "$(first_slot_field state)" "requesting"

# ── PostToolUse clears requesting ──────────────────────────────────────────────

echo
echo "[ PostToolUse clears requesting ]"

webhook "testhost" "PostToolUse|sess-002|Bash" > /dev/null
assert_eq "PostToolUse clears requesting → working" "$(first_slot_field state)" "working"

webhook "testhost" "Stop|sess-002|" > /dev/null
assert_eq "Stop after requesting → idle" "$(first_slot_field state)" "idle"

# ── empty session_id is ignored ────────────────────────────────────────────────

echo
echo "[ empty session_id ignored ]"
reset

webhook "testhost" "PreToolUse||Read" > /dev/null
col0=$(curl -sf "${BASE}/status" "${auth_headers[@]}" | \
  python3 -c "import sys,json; s=json.load(sys.stdin); print(s['columns'][0])" 2>/dev/null)
assert_eq "event with empty session_id has no effect" "$col0" "None"

# ── X-Agent-Host header as alternative to query param ─────────────────────────

echo
echo "[ X-Agent-Host header ]"
reset

curl -sf -X POST "${BASE}/webhook" \
  "${auth_headers[@]}" \
  -H "Content-Type: application/json" \
  -H "X-Agent-Host: headerhost" \
  -d '{"text":"PreToolUse|sess-hdr|Glob"}' > /dev/null

# Hosts use generated IDs (G1, G2…) not the client_key string; verify the
# agent slot was populated, which confirms the header was accepted.
assert_eq "X-Agent-Host registers agent" "$(first_slot_field state)" "working"

# ── multiple agents same host ──────────────────────────────────────────────────

echo
echo "[ multiple agents, same host ]"
reset

webhook "multihost" "PreToolUse|sess-A|Read" > /dev/null
webhook "multihost" "PreToolUse|sess-B|Write" > /dev/null

agent_count=$(status_field "s['hosts'][list(s['hosts'].keys())[0]]['agent_count']")
assert_eq "two sessions register as 2 agents" "$agent_count" "2"

tool_calls=$(status_field "s['stats']['tool_calls']")
assert_eq "tool_calls counted across both agents" "$tool_calls" "2"

webhook "multihost" "Stop|sess-A|" > /dev/null
webhook "multihost" "Stop|sess-B|" > /dev/null

# ── SessionEnd clears slot after delay ────────────────────────────────────────

echo
echo "[ SessionEnd removes agent after delay ]"
reset

webhook "testhost" "PreToolUse|sess-end|Read" > /dev/null
webhook "testhost" "SessionEnd|sess-end|" > /dev/null

# SessionEnd triggers a 5-second deferred cleanup; slot still present immediately
state_immediate=$(first_slot_field state)
assert_eq "agent still present immediately after SessionEnd" \
  "$state_immediate" "idle"

printf "  waiting 6s for deferred cleanup..."
sleep 6
col0_after=$(curl -sf "${BASE}/status" "${auth_headers[@]}" | \
  python3 -c "import sys,json; s=json.load(sys.stdin); print(s['columns'][0])" 2>/dev/null)
assert_eq "agent slot cleared 6s after SessionEnd" "$col0_after" "None"

# ── summary ────────────────────────────────────────────────────────────────────

echo
echo "──────────────────────────────────────────"
total=$(( PASS + FAIL ))
if [[ $FAIL -eq 0 ]]; then
  green "All ${total} tests passed"
else
  red   "${FAIL}/${total} tests failed"
  exit 1
fi
