#!/bin/bash
# Invariant: Unplugging during an active GIF send triggers detection and recovery
#
# Proves: When the display is physically disconnected while packets are
# being written, the write timeout or is_connected timeout fires, and
# the system reconnects.
#
# MANUAL TEST — requires physical interaction.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

echo "=== Chaos Test: Unplug during GIF send (MANUAL) ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

# Generate activity so GIFs are flowing
echo "Generating agent activity..."
for i in $(seq 1 5); do
    inject_activity "chaos-unplug" "Bash"
    sleep 0.5
done
sleep 3

echo ""
echo ">>> UNPLUG THE DISPLAY NOW <<<"
echo "    (press Enter after unplugging)"
read -r

MARK=$(log_mark "$LOG_FILE")

# Keep generating activity to trigger writes
echo "Generating activity to trigger write attempts..."
for i in $(seq 1 10); do
    inject_activity "chaos-unplug" "Read"
    sleep 2
done

echo ""
echo "Checking for disconnect detection..."
TIMEOUTS=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "timed out" || true)
RECONNECTS=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "reconnecting" || true)

echo "Timeouts detected: $TIMEOUTS"
echo "Reconnect triggers: $RECONNECTS"

if [ "$RECONNECTS" -gt 0 ]; then
    echo ""
    echo "Detection log:"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "timed out|reconnect|disconnect|Scanning" | head -10
else
    echo "Waiting longer..."
    sleep 15
    RECONNECTS=$(tail -n +$((MARK + 1)) "$LOG_FILE" | grep -c "reconnecting" || true)
    if [ "$RECONNECTS" -eq 0 ]; then
        echo "FAIL: No disconnect detected"
        exit 1
    fi
fi

echo ""
echo ">>> REPLUG THE DISPLAY NOW <<<"
echo "    (press Enter after replugging)"
read -r

REPLUG_MARK=$(log_mark "$LOG_FILE")

echo "Waiting for recovery (max 30s)..."
if wait_for_log "$REPLUG_MARK" "$LOG_FILE" "Sent animated GIF" 30; then
    echo ""
    echo "PASS: Full unplug/replug cycle recovered"
    echo ""
    echo "Full timeline:"
    tail -n +$((MARK + 1)) "$LOG_FILE" | grep -E "timed out|reconnect|disconnect|Scanning|Found IDM|Connected|Sent animated" | head -15
else
    echo "FAIL: Display did not recover after replug"
    exit 1
fi
