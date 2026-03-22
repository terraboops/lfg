#!/bin/bash
# Invariant: HTTP server stays responsive during BLE reconnection
#
# Proves: BLE failures never block the webhook handler.
# The BLE loop runs in a separate tokio task, so HTTP should always respond.
#
# Method: Trigger a force reconnect (which causes a 6s scan + connect cycle),
# then immediately hammer the HTTP server with requests. Every request must
# succeed within 1 second.

set -euo pipefail
source "$(dirname "$0")/lib.sh"
check_lfg

PASS=0
FAIL=0
TOTAL=20

echo "=== Chaos Test: HTTP responsive during BLE reconnect ==="
echo ""

# Trigger BLE disconnect
echo "Triggering BLE disconnect..."
curl -sf -X POST "$BASE/debug/ble-disconnect-simulate" > /dev/null

# Immediately start hammering HTTP
echo "Sending $TOTAL HTTP requests during reconnect window..."
for i in $(seq 1 $TOTAL); do
    START_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
    HTTP_CODE=$(curl -sf -o /dev/null -w '%{http_code}' --max-time 1 "$BASE/status" 2>/dev/null || echo "TIMEOUT")
    END_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
    ELAPSED=$((END_MS - START_MS))

    if [ "$HTTP_CODE" = "200" ] && [ "$ELAPSED" -lt 1000 ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        echo "  FAIL request $i: status=$HTTP_CODE elapsed=${ELAPSED}ms"
    fi
    sleep 0.2
done

echo ""
echo "Results: $PASS/$TOTAL passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
    echo "PASS: HTTP server stayed responsive throughout BLE reconnection"
else
    echo "FAIL: HTTP server was blocked during BLE reconnection"
    exit 1
fi
