# BLE Chaos Tests

Tests to validate BLE disconnect detection and reconnection resilience.

**Prerequisites:** LFG must be running with `RUST_LOG=lfg=debug` and the iDotMatrix display connected.

## Running

```bash
# Run all automated tests
./chaos/run_all.sh

# Run individual tests
./chaos/01_http_responsive_during_reconnect.sh
./chaos/02_force_reconnect_recovery.sh
./chaos/03_rapid_reconnect_stability.sh
./chaos/04_backoff_cap.sh
./chaos/05_state_retry_after_timeout.sh

# Manual tests (require physical interaction)
./chaos/06_unplug_during_gif_send.sh
./chaos/07_multiple_disconnect_cycles.sh
./chaos/08_idle_heartbeat_detection.sh
```
