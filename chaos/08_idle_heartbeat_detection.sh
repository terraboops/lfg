#!/bin/bash
# Invariant: Heartbeat detects disconnect when no state changes occur
#
# Proves: When the display is idle (no agent activity), the 30s heartbeat
# is the only write. If it times out 3 times, reconnection triggers.
#
# MANUAL TEST — requires physical interaction.
# Expected detection: up to 30s (heartbeat interval) + 15s (3x5s timeouts) = 45s

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

echo "=== Chaos Test: Idle heartbeat detection (MANUAL) ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

# Reset to idle
echo "Resetting to idle state..."
curl -sf -X POST "$BASE/reset" > /dev/null 2>&1 || true
sleep 3

echo "DO NOT trigger any agent activity during this test."
echo ""
echo ">>> UNPLUG THE DISPLAY NOW <<<"
echo "    (press Enter after unplugging)"
read -r

MARK=$(log_mark "$LOG_FILE")
UNPLUG_SEC=$(date +%s)

echo "Waiting for heartbeat timeout detection (max 60s)..."

WAITED=0
while [ $WAITED -lt 60 ]; do
    RECONNECTS=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "device stuck, reconnecting" || true)
    if [ "$RECONNECTS" -gt 0 ]; then
        DETECT_SEC=$(date +%s)
        echo ""
        echo "Disconnect detected after $((DETECT_SEC - UNPLUG_SEC))s"
        echo ""
        tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "heartbeat|timed out|reconnect|disconnect" | head -10
        break
    fi
    sleep 1
    WAITED=$((WAITED + 1))
done

if [ "$RECONNECTS" -eq 0 ]; then
    echo "FAIL: No disconnect detected within 60s via heartbeat path"
    exit 1
fi

echo ""
echo ">>> REPLUG THE DISPLAY NOW <<<"
echo "    (press Enter after replugging)"
read -r

REPLUG_MARK=$(log_mark "$LOG_FILE")

echo "Waiting for recovery (max 30s)..."
if wait_for_log "$REPLUG_MARK" "$LOG_FILE" "Connected to iDotMatrix" 30; then
    echo ""
    echo "PASS: Idle heartbeat detected disconnect and recovered"
else
    echo "FAIL: Display did not reconnect after replug"
    exit 1
fi
