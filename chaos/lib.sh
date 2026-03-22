#!/bin/bash
# Shared helpers for chaos tests
# Note: do NOT use set -e here — callers set their own error handling

BASE="http://localhost:5555"

# Check LFG is running and HTTP is responsive
check_lfg() {
    if ! pgrep -f "target/(debug|release)/lfg" > /dev/null 2>&1; then
        echo "ERROR: LFG is not running"
        exit 1
    fi
    if ! curl -sf --max-time 2 "$BASE/status" > /dev/null 2>&1; then
        echo "ERROR: LFG HTTP server not responding on :5555"
        exit 1
    fi
}

# Find the LFG log file. The process must be running with stderr going
# somewhere readable. We check, in order:
#   1. LFG_LOG env var (user can set this)
#   2. /tmp/lfg.log (conventional location)
#   3. Claude Code task output files (when run via `cargo run` in background)
find_log_file() {
    if [ -n "${LFG_LOG:-}" ] && [ -f "$LFG_LOG" ]; then
        echo "$LFG_LOG"
        return
    fi
    if [ -f /tmp/lfg.log ]; then
        echo "/tmp/lfg.log"
        return
    fi
    # Try Claude Code task output (most recently modified file with LFG tracing output)
    # Match LFG's actual tracing format to avoid picking up test runner output
    local candidates
    candidates=$(ls -t /private/tmp/claude-501/-Users-terra-Developer-lfg/*/tasks/*.output 2>/dev/null || true)
    for f in $candidates; do
        [ -f "$f" ] || continue
        local match
        match=$(grep -c "Listening on 0\.0\.0\.0:5555\|BLE render loop started\|Scanning for IDM-" "$f" 2>/dev/null || true)
        if [ "$match" -gt 0 ] 2>/dev/null; then
            echo "$f"
            return
        fi
    done
    echo ""
}

# Wait for a pattern to appear in recent log lines
# Usage: wait_for_log <lines_before> <log_file> <pattern> <max_seconds>
# Note: lines_before is a mark from log_mark. We check the last 500 lines
# each poll, which is sufficient for the debug log rate (~16 lines/sec).
wait_for_log() {
    local lines_before="$1"
    local log_file="$2"
    local pattern="$3"
    local max_secs="$4"
    local waited=0

    while [ $waited -lt "$max_secs" ]; do
        # Note: grep -q causes SIGPIPE on tail under pipefail, so redirect instead
        if tail -5000 "$log_file" | grep "$pattern" > /dev/null 2>&1; then
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done
    return 1
}

# Get new log lines since a mark
# Usage: new_logs <lines_before> <log_file>
new_logs() {
    tail -n +"$(($1 + 1))" "$2" 2>/dev/null || true
}

# Get current line count of log file
log_mark() {
    awk 'END {print NR}' "$1"
}

# Inject agent activity via webhook
inject_activity() {
    local host="${1:-chaos-test}"
    local tool="${2:-Bash}"
    curl -sf -X POST "$BASE/webhook" \
        -H "Content-Type: application/json" \
        -H "X-LFG-Host: $host" \
        -d "{\"text\": \"PreToolUse|chaostest|$tool\"}" > /dev/null 2>&1 || true
}
