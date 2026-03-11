use tracing::info;

use crate::sprites::THEMES;
use crate::state::{Agent, AgentState, Column, DisplayState, Host, SessionInfo, MAX_COLUMNS};

fn alloc_agent_num(state: &DisplayState) -> Option<u32> {
    let mut used = std::collections::HashSet::new();
    for info in state.session_map.values() {
        if let Some(n) = info.agent_id.strip_prefix('P').and_then(|s| s.parse::<u32>().ok()) {
            used.insert(n);
        }
    }
    (0..10).find(|n| !used.contains(n))
}

fn find_or_create_host(state: &mut DisplayState, client_key: &str) -> Option<String> {
    if let Some(hid) = state.host_by_client.get(client_key) {
        return Some(hid.clone());
    }

    // Find first empty column
    let col_idx = (0..MAX_COLUMNS).find(|&i| state.columns[i].is_none())?;

    let host_id = format!("G{}", state.next_host_num);
    state.next_host_num += 1;

    let label = if client_key.len() <= 6 {
        client_key.to_string()
    } else {
        host_id.clone()
    };

    let themes = &*THEMES;
    let theme_idx = state.hosts.len() % themes.len();

    state.hosts.insert(
        host_id.clone(),
        Host {
            theme_index: theme_idx,
            columns: vec![col_idx],
            agent_count: 0,
            label,
        },
    );

    state.columns[col_idx] = Some(Column {
        host_id: host_id.clone(),
        slots: [None, None],
    });

    state
        .host_by_client
        .insert(client_key.to_string(), host_id.clone());

    info!(
        "New host {} for client {} (theme: {}, col: {})",
        host_id, client_key, themes[theme_idx].name, col_idx
    );

    Some(host_id)
}

fn shift_columns_right(state: &mut DisplayState, from_pos: usize) -> bool {
    let last_used = (0..MAX_COLUMNS)
        .rev()
        .find(|&i| state.columns[i].is_some());
    let last_used = match last_used {
        Some(l) if l < MAX_COLUMNS - 1 => l,
        _ => return false,
    };

    for i in (from_pos..=last_used).rev() {
        state.columns[i + 1] = state.columns[i].take();
    }
    state.columns[from_pos] = None;

    for host in state.hosts.values_mut() {
        for c in host.columns.iter_mut() {
            if *c >= from_pos {
                *c += 1;
            }
        }
        host.columns.sort();
    }
    for info in state.session_map.values_mut() {
        if info.col >= from_pos {
            info.col += 1;
        }
    }
    true
}

fn add_column_for_host(state: &mut DisplayState, host_id: &str) -> (Option<usize>, Option<usize>) {
    let rightmost = match state.hosts.get(host_id) {
        Some(h) => *h.columns.iter().max().unwrap(),
        None => return (None, None),
    };

    let insert_pos = rightmost + 1;

    if insert_pos >= MAX_COLUMNS {
        return (None, None);
    }

    if state.columns[insert_pos].is_some() {
        if !shift_columns_right(state, insert_pos) {
            return (None, None);
        }
    }

    state.columns[insert_pos] = Some(Column {
        host_id: host_id.to_string(),
        slots: [None, None],
    });

    if let Some(host) = state.hosts.get_mut(host_id) {
        host.columns.push(insert_pos);
        host.columns.sort();
    }

    (Some(insert_pos), Some(0))
}

fn find_slot_in_host(state: &mut DisplayState, host_id: &str) -> (Option<usize>, Option<usize>) {
    let cols = match state.hosts.get(host_id) {
        Some(h) => h.columns.clone(),
        None => return (None, None),
    };

    for col_idx in cols.iter().copied() {
        if let Some(col) = &state.columns[col_idx] {
            for row in 0..2 {
                if col.slots[row].is_none() {
                    return (Some(col_idx), Some(row));
                }
            }
        }
    }

    add_column_for_host(state, host_id)
}

pub fn join_agent(
    state: &mut DisplayState,
    session_id: &str,
    client_key: &str,
    profile: Option<&str>,
    pixels: Option<Vec<u8>>,
) -> Option<(String, String)> {
    if let Some(info) = state.session_map.get(session_id) {
        return Some((info.agent_id.clone(), info.host_id.clone()));
    }

    let host_id = find_or_create_host(state, client_key)?;

    let (col_idx, row) = find_slot_in_host(state, &host_id);
    let col_idx = col_idx?;
    let row = row?;

    let themes = &*THEMES;
    let host = state.hosts.get_mut(&host_id)?;
    let sprite_idx = host.agent_count % themes[host.theme_index].sprites.len();
    host.agent_count += 1;
    let theme_index = host.theme_index;

    let agent_num = alloc_agent_num(state)?;
    let agent_id = format!("P{}", agent_num);

    let mut agent = Agent::new(
        agent_id.clone(),
        session_id.to_string(),
        AgentState::Idle,
        String::new(),
        theme_index,
        sprite_idx,
    );

    if let Some(px) = pixels {
        agent.pixel_override = Some(px);
    } else if let Some(prof) = profile {
        let parts: Vec<&str> = prof.splitn(2, '/').collect();
        let theme_name = parts[0];
        let idx: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        for (ti, t) in themes.iter().enumerate() {
            if t.name.eq_ignore_ascii_case(theme_name) {
                agent.sprite_override = Some(crate::state::SpriteOverride { theme: ti, index: idx });
                break;
            }
        }
    }

    if let Some(col) = state.columns[col_idx].as_mut() {
        col.slots[row] = Some(agent);
    }

    state.session_map.insert(
        session_id.to_string(),
        SessionInfo {
            agent_id: agent_id.clone(),
            host_id: host_id.clone(),
            col: col_idx,
            row,
        },
    );

    info!(
        "Agent {} joined host {} at col={} row={}",
        agent_id, host_id, col_idx, row
    );

    Some((agent_id, host_id))
}
