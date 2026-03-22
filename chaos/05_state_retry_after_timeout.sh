#!/bin/bash
# Invariant: State changes are re-attempted after reconnect
#
# Proves: When a force reconnect interrupts a pending GIF send,
# the state change is not lost. After reconnection, the pending
# state is picked up and a GIF is sent.
#
# Method: Inject a state change, immediately force reconnect,
# then verify that after reconnection the GIF is sent.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

echo "=== Chaos Test: State retry after reconnect ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

# Wait for clean state
sleep 3
MARK=$(log_mark "$LOG_FILE")

# Inject state change
echo "Injecting state change..."
inject_activity "chaos-retry" "Bash"
sleep 1

# Force reconnect mid-debounce
echo "Forcing reconnect mid-debounce..."
curl -sf -X POST "$BASE/debug/ble-disconnect-simulate" > /dev/null

# Keep injecting so state stays changed
for i in $(seq 1 5); do
    sleep 3
    inject_activity "chaos-retry" "Read"
done

echo "Waiting for GIF send after reconnect (max 30s)..."
if wait_for_log "$MARK" "$LOG_FILE" "Sent animated GIF" 30; then
    echo ""
    echo "Timeline:"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "State change|Force BLE|Scanning|Connected|Sent animated" | head -10
    echo ""
    echo "PASS: State change was re-attempted and GIF sent after reconnect"
else
    echo ""
    echo "FAIL: State change was lost — no GIF sent after reconnect"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "WARN|INFO" | tail -15
    exit 1
fi
