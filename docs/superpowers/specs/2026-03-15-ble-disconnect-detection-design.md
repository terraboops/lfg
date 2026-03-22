# BLE Disconnect Detection via Consecutive Write Timeouts

## Context

When the iDotMatrix display is physically unplugged and re-plugged while LFG is running, the BLE loop does not detect the disconnection. macOS CoreBluetooth reports `is_connected() == true` even after the device vanishes, and `peripheral.write()` with `WriteType::WithResponse` hangs indefinitely rather than returning an error. The result is a permanently stuck BLE loop — no GIFs are sent, no errors are logged, and the display never recovers until the process is restarted.

The reconnection process takes ~10 seconds (6s scan + connect + service discovery) and causes a visible flash on the display, so false positive reconnections should be minimized.

## Design

### Write timeout wrapping

Every BLE call that communicates with the device gets wrapped in `tokio::time::timeout()`:

- `peripheral.write()` — GIF packet sends and heartbeat brightness commands
- `peripheral.is_connected()` — periodic connectivity check

Timeout duration: **5 seconds** (`BLE_WRITE_TIMEOUT_SECS = 5`).

This prevents the loop from hanging indefinitely. A timeout returns control to the loop so the failure counter can be evaluated.

### Consecutive failure counter

A `consecutive_timeouts: u32` counter, initialized to `0` when a new BLE connection is established. This is a **new variable** in the inner loop scope, separate from the existing `consecutive_failures` variable in the outer reconnect loop (which tracks connection-attempt failures).

The counter is **shared across both GIF-write and heartbeat-write paths** — a heartbeat timeout followed by two GIF timeouts should trigger reconnection just as three GIF timeouts would.

**On any successful write (individual GIF packet or heartbeat):** reset counter to `0`.

**On a write timeout (Elapsed):** increment counter by `1`. Check threshold immediately (inside the packet loop, not after it — this prevents large multi-packet GIFs from requiring N*packet_count timeouts). If threshold reached, break to reconnect. If below threshold, abandon the current GIF send: break from the packet loop, skip the `last_hash = Some(current_hash)` assignment so the next poll cycle re-renders and re-attempts the send.

**On an explicit write error (Err from btleplug):** reconnect immediately. An actual error from the BLE stack (e.g. `NotConnected`) is definitive evidence of disconnection; no need to wait for consecutive failures.

**Reconnect threshold:** when `consecutive_timeouts >= 3` (`BLE_MAX_CONSECUTIVE_TIMEOUTS = 3`), log a warning with the count and break to the reconnect loop.

With a 5s timeout and threshold of 3, worst-case detection time is **15 seconds** of timeouts + ~10 seconds to reconnect = **~25 seconds** total recovery.

### `is_connected()` policy

The timeout on `is_connected()` triggers **immediate reconnection** (no consecutive counter). This is a local API call to the BLE stack, not a device write. If it hangs for 5 seconds, something is fundamentally broken at the OS/btleplug level and waiting for more evidence would not help.

### Intentional exclusions

The `CMD_SCREEN_ON` and `CMD_BRIGHTNESS` writes in `connect_ble()` use `WriteType::WithoutResponse` and are not wrapped in timeouts. These run on a freshly established connection, where a hang is extremely unlikely.

### Constants

| Constant | Value | Purpose |
|---|---|---|
| `BLE_WRITE_TIMEOUT_SECS` | `5` | Timeout for each write/is_connected call |
| `BLE_MAX_CONSECUTIVE_TIMEOUTS` | `3` | Number of consecutive timeouts before reconnecting |

### Affected code

**File:** `src/ble.rs`

**Inner loop changes:**

1. Add `let mut consecutive_timeouts: u32 = 0;` alongside existing `last_hash`, `change_detected_at`, `last_successful_write` variables at the top of the inner connection scope.

2. **GIF packet write block** (the `for pkt in &packets` loop): wrap each `peripheral.write()` in `tokio::time::timeout()`.
   - `Ok(Ok(_))`: reset `consecutive_timeouts = 0`, continue sending packets.
   - `Ok(Err(e))`: log error, break to reconnect immediately.
   - `Err(_)` (timeout): increment `consecutive_timeouts`, log warning. If `>= BLE_MAX_CONSECUTIVE_TIMEOUTS`, break to reconnect. Otherwise, break from the packet loop (abandoning the partial send) and continue the outer poll loop without updating `last_hash`.

3. **Heartbeat write block** (the `if now.duration_since(last_successful_write)` branch): same timeout wrapping.
   - `Ok(Ok(_))`: reset `consecutive_timeouts = 0`, update `last_successful_write`.
   - `Ok(Err(e))`: log error, break to reconnect immediately.
   - `Err(_)` (timeout): increment `consecutive_timeouts`, log warning. If `>= BLE_MAX_CONSECUTIVE_TIMEOUTS`, break to reconnect. Otherwise, continue the poll loop.

4. **`is_connected()` check** (the `peripheral.is_connected()` call): wrap in timeout, reconnect immediately on timeout (no counter involvement).

## Verification

1. **Build:** `cargo check` — must compile without warnings.
2. **Normal operation:** Run LFG with the display connected. Confirm GIFs send normally and "Sent animated GIF" logs appear. Confirm no spurious timeout warnings.
3. **Unplug test:** While LFG is running and sending GIFs, physically unplug the display. Confirm:
   - Timeout warnings appear in logs within ~5s of the first failed write
   - After 3 consecutive timeouts (~15s), a reconnection is triggered
   - Log shows the consecutive timeout count in the reconnection warning
4. **Replug test:** Plug the display back in before/during reconnection. Confirm it reconnects and resumes sending GIFs.
5. **Transient resilience:** If possible, test with a brief BLE interruption (e.g. momentary signal obstruction). A single timeout should log a warning but NOT trigger reconnection. The counter should reset on the next successful write.
