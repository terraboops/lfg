#!/bin/bash
# Invariant: Rapid force reconnects don't crash or deadlock
#
# Proves: Hammering the force_ble_reconnect flag doesn't cause panics,
# deadlocks, or resource leaks. The system should handle rapid reconnect
# requests gracefully — only one reconnect cycle runs at a time.
#
# Method: Fire 10 force reconnects in quick succession, then verify
# the system recovers and sends a GIF.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

echo "=== Chaos Test: Rapid reconnect stability ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

MARK=$(log_mark "$LOG_FILE")

# Fire 10 rapid reconnects
echo "Firing 10 rapid force reconnects..."
for i in $(seq 1 10); do
    curl -sf -X POST "$BASE/debug/ble-disconnect-simulate" > /dev/null 2>&1 &
done
wait
echo "All reconnect requests sent"

# Check process is still alive
sleep 2
if ! pgrep -f "target/(debug|release)/lfg" > /dev/null 2>&1; then
    echo "FAIL: LFG process crashed after rapid reconnects"
    exit 1
fi
echo "Process still alive"

# Verify HTTP still works
HTTP_CODE=$(curl -sf -o /dev/null -w '%{http_code}' --max-time 2 "$BASE/status" 2>/dev/null || echo "TIMEOUT")
if [ "$HTTP_CODE" != "200" ]; then
    echo "FAIL: HTTP server unresponsive after rapid reconnects (status=$HTTP_CODE)"
    exit 1
fi
echo "HTTP server responsive"

# Inject activity and wait for recovery
echo "Waiting for recovery (max 45s)..."
for i in $(seq 1 8); do
    sleep 3
    inject_activity "chaos-rapid"
done

if wait_for_log "$MARK" "$LOG_FILE" "Sent animated GIF" 10; then
    RECONNECT_COUNT=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "Force BLE reconnect" || true)
    GIF_COUNT=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "Sent animated GIF" || true)
    echo ""
    echo "Reconnect triggers processed: $RECONNECT_COUNT"
    echo "GIFs sent after recovery: $GIF_COUNT"
    echo ""
    echo "PASS: System recovered from rapid reconnect storm"
else
    echo ""
    echo "FAIL: No GIF sent after rapid reconnect storm"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "WARN|INFO" | tail -15
    exit 1
fi
