#!/bin/bash
# Run all automated chaos tests (skips manual tests)
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PASS=0
FAIL=0
SKIP=0

echo "╔══════════════════════════════════════════╗"
echo "║       BLE Chaos Test Suite               ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Check LFG is running
if ! pgrep -f "target/(debug|release)/lfg" > /dev/null 2>&1; then
    echo "ERROR: LFG is not running. Start with: RUST_LOG=lfg=debug cargo run"
    exit 1
fi

# Check display is connected
if ! curl -sf http://localhost:5555/status > /dev/null 2>&1; then
    echo "ERROR: LFG HTTP server not responding on :5555"
    exit 1
fi

echo "LFG is running and HTTP is responsive."

# Find log file ONCE and export for all tests
source "$SCRIPT_DIR/lib.sh"
export LFG_LOG
LFG_LOG=$(find_log_file)
if [ -z "$LFG_LOG" ]; then
    echo "ERROR: Cannot find LFG log file. Run with: RUST_LOG=lfg=debug cargo run 2>&1 | tee /tmp/lfg.log"
    exit 1
fi
echo "Log file: $LFG_LOG"
echo ""

AUTOMATED_TESTS=(
    "01_http_responsive_during_reconnect.sh"
    "02_force_reconnect_recovery.sh"
    "03_rapid_reconnect_stability.sh"
    "04_backoff_cap.sh"
    "05_state_retry_after_timeout.sh"
    "07_multiple_disconnect_cycles.sh"
)

for test in "${AUTOMATED_TESTS[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if bash "$SCRIPT_DIR/$test"; then
        PASS=$((PASS + 1))
    else
        EXIT_CODE=$?
        if [ "$EXIT_CODE" -eq 0 ]; then
            SKIP=$((SKIP + 1))
        else
            FAIL=$((FAIL + 1))
        fi
    fi
    echo ""
    # Let BLE settle between tests
    sleep 5
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo ""
echo "Manual tests not run (require physical interaction):"
echo "  - 06_unplug_during_gif_send.sh"
echo "  - 08_idle_heartbeat_detection.sh"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
