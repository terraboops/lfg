#!/bin/bash
# Invariant: Reconnection backoff caps at MAX_RECONNECT_DELAY_SECS (60s)
#
# Proves: The delay formula (5 * consecutive_failures) never exceeds 60s.
# Checks log output for "retrying in Ns" messages.

set -euo pipefail
source "$(dirname "$0")/lib.sh"

echo "=== Chaos Test: Backoff cap at 60s ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

# Extract all retry delay values from logs
DELAYS=$(grep -oP "retrying in \K[0-9]+" "$LOG_FILE" 2>/dev/null || grep -oE "retrying in [0-9]+" "$LOG_FILE" 2>/dev/null | grep -oE "[0-9]+$" || true)

if [ -z "$DELAYS" ]; then
    echo "No retry delays found in logs."
    echo "SKIP: Need a failed reconnection sequence to test backoff cap"
    exit 0
fi

MAX_DELAY=0
COUNT=0
VIOLATIONS=0

while read -r delay; do
    COUNT=$((COUNT + 1))
    if [ "$delay" -gt "$MAX_DELAY" ]; then
        MAX_DELAY=$delay
    fi
    if [ "$delay" -gt 60 ]; then
        VIOLATIONS=$((VIOLATIONS + 1))
        echo "  VIOLATION: delay=${delay}s exceeds 60s cap"
    fi
done <<< "$DELAYS"

echo "Retry delays found: $COUNT"
echo "Maximum delay observed: ${MAX_DELAY}s"
echo ""

if [ "$VIOLATIONS" -eq 0 ]; then
    echo "PASS: All retry delays within 60s cap (max observed: ${MAX_DELAY}s)"
else
    echo "FAIL: $VIOLATIONS delays exceeded the 60s cap"
    exit 1
fi
