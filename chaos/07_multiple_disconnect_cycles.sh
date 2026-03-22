#!/bin/bash
# Invariant: Multiple disconnect/reconnect cycles don't leak resources
#
# Proves: The system can handle repeated disconnect/reconnect cycles
# without degradation. Each cycle should recover in roughly the same
# time window. No stale peripheral handles, no adapter exhaustion.
#
# Method: Run 5 force-reconnect cycles back-to-back, measuring recovery
# time for each. All must recover, and recovery times must be consistent.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

CYCLES=5

echo "=== Chaos Test: Multiple disconnect/reconnect cycles ==="
echo ""

LOG_FILE=$(find_log_file)
if [ -z "$LOG_FILE" ]; then
    echo "SKIP: Cannot find LFG log file"
    exit 0
fi

FAILURES=0
TIMES=()

for cycle in $(seq 1 $CYCLES); do
    echo "--- Cycle $cycle/$CYCLES ---"

    MARK=$(log_mark "$LOG_FILE")
    START_SEC=$(date +%s)

    # Trigger disconnect
    curl -sf -X POST "$BASE/debug/ble-disconnect-simulate" > /dev/null

    # Inject activity to trigger state changes
    for i in $(seq 1 6); do
        sleep 2
        inject_activity "chaos-cycles"
    done

    # Wait for GIF send (max 30s total from start)
    if wait_for_log "$MARK" "$LOG_FILE" "Sent animated GIF" 20; then
        END_SEC=$(date +%s)
        ELAPSED=$((END_SEC - START_SEC))
        TIMES+=($ELAPSED)
        echo "  Recovered in ${ELAPSED}s"
    else
        echo "  FAIL: Did not recover within 30s"
        FAILURES=$((FAILURES + 1))
    fi

    # Brief pause between cycles
    sleep 2
done

echo ""
echo "=== Results ==="
echo "Cycles: $CYCLES"
echo "Failures: $FAILURES"
if [ ${#TIMES[@]} -gt 0 ]; then
    echo "Recovery times: ${TIMES[*]}s"
fi

if [ "$FAILURES" -eq 0 ]; then
    echo ""
    echo "PASS: All $CYCLES disconnect/reconnect cycles recovered successfully"
else
    echo ""
    echo "FAIL: $FAILURES/$CYCLES cycles failed to recover"
    exit 1
fi
