#!/bin/bash
# Invariant: Force reconnect recovers and resumes GIF sending
#
# Proves: After a force reconnect, the system scans, connects, and sends
# a GIF within a bounded time window (~15s: 6s scan + connect + debounce).
#
# Method: Record log position, trigger force reconnect, inject activity,
# then wait for a new "Sent animated GIF" log line. Must appear within 30s.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

echo "=== Chaos Test: Force reconnect recovery ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file (set LFG_LOG or run with stderr > /tmp/lfg.log)"
    exit 0
fi

# Inject activity to ensure there's a state change
inject_activity
sleep 1

MARK=$(log_mark "$LOG_FILE")

echo "Triggering force reconnect..."
curl -sf -X POST "$BASE/debug/ble-disconnect-simulate" > /dev/null

# Inject activity during reconnect to trigger state changes
for i in $(seq 1 5); do
    sleep 2
    inject_activity "chaos-recovery" "Read"
done

echo "Waiting for GIF send after reconnect (max 30s)..."
if wait_for_log "$MARK" "$LOG_FILE" "Sent animated GIF" 30; then
    echo ""
    echo "Recovery timeline:"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "Force BLE reconnect|Scanning|Connected to iDotMatrix|Sent animated GIF" | head -10
    echo ""
    echo "PASS: GIF send resumed after force reconnect"
else
    echo ""
    echo "FAIL: No GIF sent within 30s of force reconnect"
    echo "Recent logs:"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "WARN|INFO" | tail -10
    exit 1
fi
