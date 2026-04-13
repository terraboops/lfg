use std::time::Duration;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::event;
use crate::render;
use crate::state::DisplayState;

// BLE UUIDs
const WRITE_UUIDS: &[&str] = &[
    "0000fa02-0000-1000-8000-00805f9b34fb",
    "0000fff2-0000-1000-8000-00805f9b34fb",
];
#[allow(dead_code)]
const NOTIFY_UUIDS: &[&str] = &[
    "0000fa03-0000-1000-8000-00805f9b34fb",
    "0000fff1-0000-1000-8000-00805f9b34fb",
];

const CMD_SCREEN_ON: &[u8] = &[0x05, 0x00, 0x07, 0x01, 0x01];
const CMD_BRIGHTNESS: &[u8] = &[0x05, 0x00, 0x04, 0x80, 100];

const BLE_DEBOUNCE_SECS: f64 = 2.0;
const BLE_POLL_SECS: f64 = 0.25;
const BLE_HEARTBEAT_SECS: f64 = 30.0;
const BLE_WRITE_TIMEOUT_SECS: u64 = 5;
const BLE_MAX_CONSECUTIVE_TIMEOUTS: u32 = 3;

const MAX_RECONNECT_DELAY_SECS: u64 = 60;

/// Build GIF packets for BLE transmission.
/// Each packet has a 16-byte header + up to 4096 bytes of GIF data.
pub fn build_gif_packets(gif_data: &[u8]) -> Vec<Vec<u8>> {
    const CHUNK_SIZE: usize = 4096;
    let crc = crc32fast::hash(gif_data);
    let mut packets = Vec::new();

    for (i, chunk) in gif_data.chunks(CHUNK_SIZE).enumerate() {
        let mut hdr = vec![0u8; 16];
        let total_len = (chunk.len() + 16) as u16;
        hdr[0] = (total_len & 0xFF) as u8;
        hdr[1] = ((total_len >> 8) & 0xFF) as u8;
        hdr[2] = 0x01;
        hdr[3] = 0x00;
        hdr[4] = if i > 0 { 0x02 } else { 0x00 };

        let gif_len = gif_data.len() as u32;
        hdr[5] = (gif_len & 0xFF) as u8;
        hdr[6] = ((gif_len >> 8) & 0xFF) as u8;
        hdr[7] = ((gif_len >> 16) & 0xFF) as u8;
        hdr[8] = ((gif_len >> 24) & 0xFF) as u8;

        hdr[9] = (crc & 0xFF) as u8;
        hdr[10] = ((crc >> 8) & 0xFF) as u8;
        hdr[11] = ((crc >> 16) & 0xFF) as u8;
        hdr[12] = ((crc >> 24) & 0xFF) as u8;

        hdr[13] = 0x05;
        hdr[14] = 0x00;
        hdr[15] = 0x0D;

        let mut packet = hdr;
        packet.extend_from_slice(chunk);
        packets.push(packet);
    }

    packets
}

/// Main BLE render loop. Runs in a spawned task.
pub async fn ble_loop(state: Arc<RwLock<DisplayState>>) {
    use btleplug::api::{Manager as _, Peripheral as _, WriteType};
    use btleplug::platform::Manager;

    // Create manager and adapter ONCE — reuse across reconnections
    let manager = match Manager::new().await {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to create BLE manager: {} — BLE disabled", e);
            return;
        }
    };
    let adapter = match manager.adapters().await {
        Ok(adapters) => match adapters.into_iter().next() {
            Some(a) => a,
            None => {
                warn!("No BLE adapters found — BLE disabled");
                return;
            }
        },
        Err(e) => {
            warn!("Failed to enumerate BLE adapters: {} — BLE disabled", e);
            return;
        }
    };

    let mut consecutive_failures: u32 = 0;

    loop {
        // Check for force-reconnect flag and reset watchdog baseline
        // (connect_ble has its own internal timeouts; the supervisor watchdog
        // should only scrutinize the steady-state inner loop).
        {
            let mut s = state.write().await;
            if s.force_ble_reconnect {
                s.force_ble_reconnect = false;
                info!("Force BLE reconnect requested");
            }
            s.last_gif_sent_at = std::time::Instant::now();
        }

        // Try to connect (reusing the adapter)
        let connection = match connect_ble(&adapter).await {
            Some(c) => {
                consecutive_failures = 0;
                c
            }
            None => {
                consecutive_failures += 1;
                let delay = (5 * consecutive_failures as u64).min(MAX_RECONNECT_DELAY_SECS);
                warn!(
                    "BLE connection attempt {} failed — retrying in {}s",
                    consecutive_failures, delay
                );
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }
        };

        let (peripheral, write_char) = connection;
        let mut last_hash: Option<String> = None;
        let mut change_detected_at: Option<tokio::time::Instant> = None;
        let mut last_successful_write = tokio::time::Instant::now();
        let mut consecutive_timeouts: u32 = 0;

        info!("BLE render loop started");

        loop {
            // Check for force-reconnect flag
            {
                let s = state.read().await;
                if s.force_ble_reconnect {
                    drop(s);
                    let mut s = state.write().await;
                    s.force_ble_reconnect = false;
                    warn!("Force BLE reconnect triggered — disconnecting");
                    break;
                }
            }

            // Note: no per-tick is_connected() check. On CoreBluetooth that call
            // isn't actually a cache lookup — it can stall behind in-flight
            // operations and trip our 5s timeout during heavy write activity,
            // forcing spurious reconnects on a device that's perfectly healthy.
            // Real disconnects are caught by the write path (explicit error) and
            // by the 30s heartbeat during idle stretches; the v0.2.0 watchdog
            // provides the final safety net if the loop ever stops progressing.

            // Check stale under write lock, then snapshot under read
            {
                let mut s = state.write().await;
                event::check_stale(&mut s);
            }
            let (current_hash, snap, any_requesting) = {
                let s = state.read().await;
                let hash = render::state_hash(&s);
                let snap = render::snapshot_state(&s);
                let any_req = s.any_requesting();
                (hash, snap, any_req)
            };

            let now = tokio::time::Instant::now();

            if Some(&current_hash) != last_hash.as_ref() {
                if change_detected_at.is_none() {
                    change_detected_at = Some(now);
                    info!("State change detected, debouncing {:.1}s", BLE_DEBOUNCE_SECS);
                }

                let elapsed = now.duration_since(change_detected_at.unwrap());
                if elapsed.as_secs_f64() >= BLE_DEBOUNCE_SECS {
                    let gif_data = render::build_animated_gif(&snap, any_requesting);
                    let packets = build_gif_packets(&gif_data);

                    debug!("BLE loop: sending {} GIF packets", packets.len());
                    let mut send_ok = true;
                    let mut timed_out = false;
                    for pkt in &packets {
                        // GIF data uses WriteWithoutResponse: the iDotMatrix firmware
                        // periodically stops ACK'ing packets (observed every ~1h), which
                        // made WriteWithResponse hang for 5s per packet and freeze the
                        // display for ~15s while consecutive_timeouts climbed. The 100ms
                        // inter-packet sleep below is effectively manual flow control,
                        // and the heartbeat (still WriteWithResponse) keeps us honest
                        // about whether the device is actually alive.
                        let write_result = tokio::time::timeout(
                            Duration::from_secs(BLE_WRITE_TIMEOUT_SECS),
                            peripheral.write(&write_char, pkt, WriteType::WithoutResponse),
                        ).await;
                        match write_result {
                            Ok(Ok(_)) => {
                                consecutive_timeouts = 0;
                                if packets.len() > 1 {
                                    tokio::time::sleep(Duration::from_millis(100)).await;
                                }
                            }
                            Ok(Err(e)) => {
                                warn!("BLE write error: {} — reconnecting", e);
                                send_ok = false;
                                break;
                            }
                            Err(_) => {
                                consecutive_timeouts += 1;
                                warn!(
                                    "BLE write timed out after {}s (consecutive_timeouts = {})",
                                    BLE_WRITE_TIMEOUT_SECS, consecutive_timeouts
                                );
                                if consecutive_timeouts >= BLE_MAX_CONSECUTIVE_TIMEOUTS {
                                    warn!(
                                        "{} consecutive BLE write timeouts — device stuck, reconnecting",
                                        consecutive_timeouts
                                    );
                                    send_ok = false;
                                }
                                timed_out = true;
                                break; // abandon this GIF send either way
                            }
                        }
                    }

                    if !send_ok {
                        break; // reconnect
                    }

                    if !timed_out {
                        last_hash = Some(current_hash);
                        change_detected_at = None;
                        last_successful_write = now;
                        state.write().await.last_gif_sent_at = std::time::Instant::now();
                        info!(
                            "Sent animated GIF ({} bytes, {})",
                            gif_data.len(),
                            if any_requesting { "fast" } else { "normal" }
                        );
                    }
                    // If timed_out but below threshold: don't update last_hash,
                    // so next poll cycle will re-attempt the send
                }
            } else {
                change_detected_at = None;

                // Hash matches what's on the device — loop is caught up. Tell
                // the watchdog so it doesn't trip on hook events that don't
                // affect display state (e.g. Notification).
                state.write().await.last_gif_sent_at = std::time::Instant::now();

                // Heartbeat: send brightness command periodically to detect stale connections
                if now.duration_since(last_successful_write).as_secs_f64() >= BLE_HEARTBEAT_SECS {
                    debug!("BLE loop: sending heartbeat");
                    let hb_result = tokio::time::timeout(
                        Duration::from_secs(BLE_WRITE_TIMEOUT_SECS),
                        peripheral.write(&write_char, CMD_BRIGHTNESS, WriteType::WithResponse),
                    ).await;
                    match hb_result {
                        Ok(Ok(_)) => {
                            consecutive_timeouts = 0;
                            last_successful_write = now;
                        }
                        Ok(Err(e)) => {
                            warn!("BLE heartbeat failed: {} — reconnecting", e);
                            break;
                        }
                        Err(_) => {
                            consecutive_timeouts += 1;
                            warn!(
                                "BLE heartbeat timed out after {}s (consecutive_timeouts = {})",
                                BLE_WRITE_TIMEOUT_SECS, consecutive_timeouts
                            );
                            if consecutive_timeouts >= BLE_MAX_CONSECUTIVE_TIMEOUTS {
                                warn!(
                                    "{} consecutive BLE write timeouts — device stuck, reconnecting",
                                    consecutive_timeouts
                                );
                                break;
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs_f64(BLE_POLL_SECS)).await;
        }

        // Cleanup on disconnect
        info!("Disconnecting BLE peripheral for reconnection...");
        match tokio::time::timeout(
            Duration::from_secs(BLE_WRITE_TIMEOUT_SECS),
            peripheral.disconnect(),
        ).await {
            Ok(_) => debug!("BLE disconnect completed"),
            Err(_) => warn!("BLE disconnect timed out — proceeding with reconnection anyway"),
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
        info!("Attempting BLE reconnection...");
    }
}

async fn connect_ble(
    adapter: &btleplug::platform::Adapter,
) -> Option<(btleplug::platform::Peripheral, btleplug::api::Characteristic)> {
    use btleplug::api::{Central, Peripheral as _, WriteType};

    info!("Scanning for IDM-* devices...");
    if let Err(e) = adapter.start_scan(btleplug::api::ScanFilter::default()).await {
        warn!("BLE scan start failed: {}", e);
        return None;
    }
    tokio::time::sleep(Duration::from_secs(6)).await;
    let _ = adapter.stop_scan().await;

    let peripherals = match adapter.peripherals().await {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to list BLE peripherals: {}", e);
            return None;
        }
    };

    // Find IDM-* device
    let mut idm_peripheral = None;
    let mut idm_name = String::from("unknown");
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            if let Some(name) = &props.local_name {
                if name.starts_with("IDM-") {
                    idm_name = name.clone();
                    idm_peripheral = Some(p);
                    break;
                }
            }
        }
    }
    let idm = match idm_peripheral {
        Some(p) => p,
        None => {
            warn!("No IDM-* device found during scan");
            return None;
        }
    };
    info!("Found {} at {:?}", idm_name, idm.id());

    match tokio::time::timeout(
        Duration::from_secs(10),
        idm.connect(),
    ).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            warn!("BLE connect to {} failed: {}", idm_name, e);
            return None;
        }
        Err(_) => {
            warn!("BLE connect to {} timed out after 10s — stale device?", idm_name);
            let _ = tokio::time::timeout(Duration::from_secs(2), idm.disconnect()).await;
            return None;
        }
    }
    match tokio::time::timeout(
        Duration::from_secs(10),
        idm.discover_services(),
    ).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            warn!("BLE service discovery failed: {}", e);
            let _ = tokio::time::timeout(Duration::from_secs(2), idm.disconnect()).await;
            return None;
        }
        Err(_) => {
            warn!("BLE service discovery timed out after 10s");
            let _ = tokio::time::timeout(Duration::from_secs(2), idm.disconnect()).await;
            return None;
        }
    }

    // Find write characteristic (resolve once, cache the handle)
    let mut write_char = None;
    for svc in idm.services() {
        for ch in &svc.characteristics {
            let uuid_str = ch.uuid.to_string();
            if WRITE_UUIDS.iter().any(|&u| u == uuid_str) {
                info!("Using write UUID: {}", uuid_str);
                write_char = Some(ch.clone());
                break;
            }
        }
        if write_char.is_some() {
            break;
        }
    }
    let write_char = match write_char {
        Some(c) => c,
        None => {
            warn!("No matching write characteristic found on {}", idm_name);
            let _ = idm.disconnect().await;
            return None;
        }
    };

    // Send screen on + brightness
    let _ = idm.write(&write_char, CMD_SCREEN_ON, WriteType::WithoutResponse).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _ = idm.write(&write_char, CMD_BRIGHTNESS, WriteType::WithoutResponse).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!("Connected to iDotMatrix — screen ON");
    Some((idm, write_char))
}

/// Supervises `ble_loop`. Restarts on exit/panic and escalates stuck-state
/// detection: first sets `force_ble_reconnect` to let the loop break out
/// cooperatively; if the loop stays wedged past the grace period, aborts
/// the task and respawns it.
pub async fn ble_supervisor(state: Arc<RwLock<DisplayState>>) {
    const CHECK_INTERVAL_SECS: u64 = 5;
    const STUCK_THRESHOLD_SECS: f64 = 20.0;

    loop {
        let task_state = state.clone();
        let mut handle = tokio::spawn(async move { ble_loop(task_state).await });
        let mut flag_set_at: Option<std::time::Instant> = None;

        loop {
            tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;

            if handle.is_finished() {
                match (&mut handle).await {
                    Ok(()) => warn!("BLE loop exited — respawning in 3s"),
                    Err(e) if e.is_cancelled() => {
                        warn!("BLE loop aborted by watchdog — respawning in 3s")
                    }
                    Err(e) => warn!("BLE loop panicked: {} — respawning in 3s", e),
                }
                break;
            }

            let (gif_stale_secs, events_since_gif, flag_still_set) = {
                let s = state.read().await;
                let now = std::time::Instant::now();
                (
                    now.duration_since(s.last_gif_sent_at).as_secs_f64(),
                    s.last_hook_event_at > s.last_gif_sent_at,
                    s.force_ble_reconnect,
                )
            };

            let is_stuck = events_since_gif && gif_stale_secs > STUCK_THRESHOLD_SECS;
            if !is_stuck {
                flag_set_at = None;
                continue;
            }

            match flag_set_at {
                None => {
                    warn!(
                        "BLE watchdog: no GIF sent in {:.1}s despite recent hook events — forcing reconnect",
                        gif_stale_secs
                    );
                    state.write().await.force_ble_reconnect = true;
                    flag_set_at = Some(std::time::Instant::now());
                }
                Some(set_at) => {
                    let grace = set_at.elapsed().as_secs_f64();
                    if flag_still_set && grace >= CHECK_INTERVAL_SECS as f64 {
                        warn!(
                            "BLE watchdog: loop wedged for {:.1}s after force_reconnect flag — aborting task",
                            gif_stale_secs
                        );
                        handle.abort();
                        let _ = (&mut handle).await;
                        break;
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
