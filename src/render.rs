use std::io::Cursor;

use crate::sprites::{
    agent_color, font_glyph, tool_to_icon,
    IconDef, Rgb, SpriteVariant, ABILITIES, AGENT_X, COL_W, DISPLAY_SIZE, FIRE, HOST_LABEL_Y,
    ICON_DX, SLOT_LAYOUTS, THEMES, ZZZ,
};
use crate::state::{AgentState, DisplayState, Host, MAX_COLUMNS};

pub struct Canvas {
    pub pixels: [u8; DISPLAY_SIZE * DISPLAY_SIZE * 3],
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            pixels: [0; DISPLAY_SIZE * DISPLAY_SIZE * 3],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x < DISPLAY_SIZE && y < DISPLAY_SIZE {
            let off = (y * DISPLAY_SIZE + x) * 3;
            self.pixels[off] = r;
            self.pixels[off + 1] = g;
            self.pixels[off + 2] = b;
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, r: u8, g: u8, b: u8) {
        let mut cx = x;
        for ch in text.chars().map(|c| c.to_ascii_uppercase()) {
            if let Some(glyph) = font_glyph(ch) {
                for (row_i, row_str) in glyph.iter().enumerate() {
                    for (col_i, pixel) in row_str.chars().enumerate() {
                        if pixel == 'X' {
                            self.set_pixel(cx + col_i, y + row_i, r, g, b);
                        }
                    }
                }
            }
            cx += 4;
        }
    }

    pub fn draw_sprite(
        &mut self,
        x: usize,
        y: usize,
        sprite: &SpriteVariant,
        dim: bool,
        tick: usize,
    ) {
        let p = if dim {
            (
                (sprite.primary.0 / 6).max(4),
                (sprite.primary.1 / 6).max(4),
                (sprite.primary.2 / 6).max(4),
            )
        } else {
            sprite.primary
        };
        let s = if dim { (28, 28, 28) } else { sprite.skin };

        let frame = &sprite.frames[tick % sprite.frames.len()];
        for (row_i, row_str) in frame.iter().enumerate() {
            for (col_i, ch) in row_str.chars().enumerate() {
                match ch {
                    'P' => {
                        let (sr, sg, sb) = shade(p.0, p.1, p.2, row_i, col_i);
                        self.set_pixel(x + col_i, y + row_i, sr, sg, sb);
                    }
                    'S' => {
                        let (sr, sg, sb) = shade(s.0, s.1, s.2, row_i, col_i);
                        self.set_pixel(x + col_i, y + row_i, sr, sg, sb);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn draw_raw_pixels(&mut self, x: usize, y: usize, data: &[u8], dim: bool) {
        for row_i in 0..8 {
            for col_i in 0..8 {
                let off = (row_i * 8 + col_i) * 3;
                if off + 2 < data.len() {
                    let (mut pr, mut pg, mut pb) = (data[off], data[off + 1], data[off + 2]);
                    if pr == 0 && pg == 0 && pb == 0 {
                        continue;
                    }
                    if dim {
                        pr = (pr / 6).max(4);
                        pg = (pg / 6).max(4);
                        pb = (pb / 6).max(4);
                    }
                    self.set_pixel(x + col_i, y + row_i, pr, pg, pb);
                }
            }
        }
    }

    pub fn draw_icon(
        &mut self,
        x: usize,
        y: usize,
        icon: &IconDef,
        tick: usize,
    ) {
        let (r, g, b) = icon.color;
        let (r2, g2, b2) = icon.color2.unwrap_or(icon.color);
        let bitmap = &icon.frames[tick % icon.frames.len()];
        for (row_i, row_str) in bitmap.iter().enumerate() {
            for (col_i, ch) in row_str.chars().enumerate() {
                match ch {
                    'X' => {
                        let (sr, sg, sb) = shade(r, g, b, row_i, col_i);
                        self.set_pixel(x + col_i, y + row_i, sr, sg, sb);
                    }
                    'N' => {
                        let (sr, sg, sb) = shade(r2, g2, b2, row_i, col_i);
                        self.set_pixel(x + col_i, y + row_i, sr, sg, sb);
                    }
                    'H' => {
                        self.set_pixel(x + col_i, y + row_i, r, g, b);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn draw_marching_border(&mut self, x: usize, y: usize, tick: usize) {
        let bx0 = if x > 0 { x - 1 } else { 0 };
        let by0 = if y > 0 { y - 1 } else { 0 };
        let bx1 = (x + 8).min(63);
        let by1 = (y + 8).min(63);
        let amber: Rgb = (255, 150, 0);
        let phase = tick % 3;

        for px_x in bx0..=bx1 {
            if (px_x - bx0 + phase) % 3 != 0 {
                self.set_pixel(px_x, by0, amber.0, amber.1, amber.2);
                self.set_pixel(px_x, by1, amber.0, amber.1, amber.2);
            }
        }
        for px_y in (by0 + 1)..by1 {
            if (px_y - by0 + phase) % 3 != 0 {
                self.set_pixel(bx0, px_y, amber.0, amber.1, amber.2);
                self.set_pixel(bx1, px_y, amber.0, amber.1, amber.2);
            }
        }
    }

    pub fn draw_status_icon(
        &mut self,
        x: usize,
        y: usize,
        agent_state: AgentState,
        tool_name: &str,
        tick: usize,
    ) {
        match agent_state {
            AgentState::Working => {
                let icon = &ABILITIES[tool_to_icon(tool_name)];
                self.draw_icon(x, y, icon, tick);
            }
            AgentState::Idle => {
                let (r, g, b) = ZZZ.color;
                let (fr, fg, fb) = if tick % 2 == 0 {
                    ((r / 4).max(5), (g / 4).max(5), (b / 4).max(5))
                } else {
                    (r / 2, g / 2, b / 2)
                };
                let bitmap = &ZZZ.frames[tick % ZZZ.frames.len()];
                for (row_i, row_str) in bitmap.iter().enumerate() {
                    for (col_i, ch) in row_str.chars().enumerate() {
                        if ch == 'X' {
                            self.set_pixel(x + col_i, y + row_i, fr, fg, fb);
                        }
                    }
                }
                self.draw_marching_border(x, y, tick);
            }
            AgentState::Requesting => {
                self.draw_icon(x, y, &FIRE, tick);
                self.draw_marching_border(x, y, tick);
            }
        }
    }

    pub fn draw_summary_row(&mut self, stats_display: &crate::state::StatsDisplay) {
        // Separator at y=58
        for x in 0..DISPLAY_SIZE {
            self.set_pixel(x, 58, 40, 40, 80);
        }

        let dim: Rgb = (80, 160, 160);
        let bright: Rgb = (140, 220, 220);

        // Panel 1: agent-minutes
        self.draw_text(1, 59, "T", dim.0, dim.1, dim.2);
        self.draw_text(5, 59, &stats_display.time_str, bright.0, bright.1, bright.2);

        // Panel 2: tool calls
        self.draw_text(23, 59, "C", dim.0, dim.1, dim.2);
        self.draw_text(27, 59, &stats_display.tool_str, bright.0, bright.1, bright.2);

        // Panel 3: unique agents
        self.draw_text(47, 59, "A", dim.0, dim.1, dim.2);
        self.draw_text(51, 59, &stats_display.agnt_str, bright.0, bright.1, bright.2);
    }

}

fn shade(r: u8, g: u8, b: u8, row_i: usize, col_i: usize) -> (u8, u8, u8) {
    let vy = row_i as f64 / 7.0;
    let vx = col_i as f64 / 7.0;
    let t = vy * 0.6 + vx * 0.4;

    let (rf, gf, bf) = (r as f64, g as f64, b as f64);

    if t < 0.35 {
        let f = 1.0 - t / 0.35;
        (
            (rf + 90.0 * f).min(255.0) as u8,
            (gf + 60.0 * f).min(255.0) as u8,
            (bf - 40.0 * f).max(0.0) as u8,
        )
    } else if t > 0.55 {
        let f = (t - 0.55) / 0.45;
        (
            (rf * (1.0 - 0.55 * f)).max(0.0) as u8,
            (gf * (1.0 - 0.45 * f)).max(0.0) as u8,
            (bf + 40.0 * f).min(255.0) as u8,
        )
    } else {
        (r, g, b)
    }
}

/// Snapshot of state needed for rendering (taken under lock, rendered without lock).
#[derive(Clone)]
pub struct FrameSnapshot {
    pub columns: [Option<ColumnSnapshot>; MAX_COLUMNS],
    pub hosts: std::collections::HashMap<String, Host>,
    pub stats_display: crate::state::StatsDisplay,
}

#[derive(Clone)]
pub struct ColumnSnapshot {
    pub host_id: String,
    pub slots: [Option<AgentSnapshot>; 2],
}

#[derive(Clone)]
pub struct AgentSnapshot {
    pub agent_id: String,
    pub state: AgentState,
    pub tool_name: String,
    pub theme_index: usize,
    pub sprite_index: usize,
    pub sprite_override: Option<crate::state::SpriteOverride>,
    pub pixel_override: Option<Vec<u8>>,
}

pub fn snapshot_state(state: &DisplayState) -> FrameSnapshot {
    let columns = std::array::from_fn(|i| {
        state.columns[i].as_ref().map(|col| ColumnSnapshot {
            host_id: col.host_id.clone(),
            slots: std::array::from_fn(|row| {
                col.slots[row].as_ref().map(|a| AgentSnapshot {
                    agent_id: a.agent_id.clone(),
                    state: a.state,
                    tool_name: a.tool_name.clone(),
                    theme_index: a.theme_index,
                    sprite_index: a.sprite_index,
                    sprite_override: a.sprite_override.clone(),
                    pixel_override: a.pixel_override.clone(),
                })
            }),
        })
    });

    FrameSnapshot {
        columns,
        hosts: state.hosts.clone(),
        stats_display: state.stats_display.clone(),
    }
}

pub fn build_frame(canvas: &mut Canvas, snap: &FrameSnapshot, tick: usize) {
    canvas.clear();
    let themes = &*THEMES;
    let mut drawn_hosts = std::collections::HashSet::new();

    for col_idx in 0..MAX_COLUMNS {
        let col = match &snap.columns[col_idx] {
            Some(c) => c,
            None => continue,
        };

        let col_x = AGENT_X[col_idx];
        let host_id = &col.host_id;
        let host = match snap.hosts.get(host_id) {
            Some(h) => h,
            None => continue,
        };
        // Draw host label
        if drawn_hosts.insert(host_id.clone()) {
            let mut label = host.label.clone();
            if host.agent_count <= 2 && label.len() > 3 {
                label = format!("{}.", &label[..2]);
            }
            let host_cols = &host.columns;
            let group_left = AGENT_X[host_cols[0]];
            let group_right = AGENT_X[*host_cols.last().unwrap()] + COL_W;
            let group_w = group_right - group_left;
            let label_w = label.len() * 4 - 1;
            let label_x = group_left + group_w.saturating_sub(label_w) / 2;
            canvas.draw_text(label_x, HOST_LABEL_Y, &label, 180, 180, 255);
        }

        // Draw each slot
        for row in 0..2 {
            let layout = &SLOT_LAYOUTS[row];
            let sprite_x = col_x + ICON_DX;

            if let Some(agent) = &col.slots[row] {
                // Determine sprite
                if let Some(px_data) = &agent.pixel_override {
                    canvas.draw_raw_pixels(sprite_x, layout.sprite_y, px_data, false);
                } else {
                    let (ti, si) = if let Some(so) = &agent.sprite_override {
                        (so.theme, so.index)
                    } else {
                        (agent.theme_index, agent.sprite_index)
                    };
                    let spr = &themes[ti].sprites[si % themes[ti].sprites.len()];
                    canvas.draw_sprite(sprite_x, layout.sprite_y, spr, false, tick);
                }

                let (ar, ag, ab) = agent_color(&agent.agent_id);
                canvas.draw_text(col_x + 2, layout.id_y, &agent.agent_id, ar, ag, ab);
                canvas.draw_status_icon(
                    sprite_x,
                    layout.status_y,
                    agent.state,
                    &agent.tool_name,
                    tick,
                );
            }
        }
    }

    canvas.draw_summary_row(&snap.stats_display);
}

pub fn build_animated_gif(snap: &FrameSnapshot, any_requesting: bool) -> Vec<u8> {
    let frame_delay_cs = if any_requesting { 50u16 } else { 125u16 }; // centiseconds
    let mut canvas = Canvas::new();
    let pixel_count = DISPLAY_SIZE * DISPLAY_SIZE;

    // Render all 6 frames and collect raw RGB data
    let mut frame_rgb: Vec<Vec<u8>> = Vec::with_capacity(6);
    for tick in 0..6 {
        build_frame(&mut canvas, snap, tick);
        frame_rgb.push(canvas.pixels.to_vec());
    }

    // Collect all pixels from all frames for global palette quantization
    let mut all_pixels: Vec<u8> = Vec::with_capacity(6 * pixel_count * 4);
    for rgb in &frame_rgb {
        for i in 0..pixel_count {
            let off = i * 3;
            all_pixels.push(rgb[off]);     // R
            all_pixels.push(rgb[off + 1]); // G
            all_pixels.push(rgb[off + 2]); // B
            all_pixels.push(255);          // A (required by NeuQuant)
        }
    }

    // Build a global 256-color palette using NeuQuant
    let nq = color_quant::NeuQuant::new(1, 256, &all_pixels);
    let mut palette = nq.color_map_rgb();

    // Pin pure black at palette index 0. The GIF Logical Screen Descriptor's
    // background_color_index defaults to 0, and the iDotMatrix firmware uses
    // it as the "off" colour. NeuQuant's ordering is input-dependent, so
    // without this the background drifts to whatever colour happens to win
    // index 0 — frequently a dim purple/orange/etc.
    let black_idx = nq.index_of(&[0, 0, 0, 255]);
    if black_idx != 0 {
        palette.swap(0, black_idx * 3);
        palette.swap(1, black_idx * 3 + 1);
        palette.swap(2, black_idx * 3 + 2);
    }
    palette[0] = 0;
    palette[1] = 0;
    palette[2] = 0;

    // Encode GIF with global color table
    let mut buf = Cursor::new(Vec::new());
    {
        let mut encoder = gif::Encoder::new(
            &mut buf,
            DISPLAY_SIZE as u16,
            DISPLAY_SIZE as u16,
            &palette,
        ).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();

        for rgb in &frame_rgb {
            // Map each pixel to nearest palette index, mirroring the 0 ↔ black_idx
            // swap applied to the palette above.
            let mut indices = vec![0u8; pixel_count];
            for i in 0..pixel_count {
                let off = i * 3;
                let raw = nq.index_of(&[rgb[off], rgb[off + 1], rgb[off + 2], 255]);
                indices[i] = if raw == black_idx {
                    0
                } else if raw == 0 {
                    black_idx as u8
                } else {
                    raw as u8
                };
            }

            let mut frame = gif::Frame::default();
            frame.width = DISPLAY_SIZE as u16;
            frame.height = DISPLAY_SIZE as u16;
            frame.delay = frame_delay_cs;
            frame.buffer = std::borrow::Cow::Borrowed(&indices);
            encoder.write_frame(&frame).unwrap();
        }
    }

    buf.into_inner()
}

pub fn state_hash(state: &DisplayState) -> String {
    let mut parts = Vec::new();
    for i in 0..MAX_COLUMNS {
        match &state.columns[i] {
            None => parts.push("_".to_string()),
            Some(col) => {
                for row in 0..2 {
                    match &col.slots[row] {
                        Some(a) => {
                            parts.push(format!(
                                "{}:{}:{}",
                                a.state.as_str(),
                                a.tool_name,
                                a.theme_index
                            ));
                        }
                        None => parts.push("-".to_string()),
                    }
                }
            }
        }
    }

    parts.push(if state.any_requesting() { "F" } else { "N" }.to_string());
    parts.push(format!("t={}", state.stats_display.time_str));
    parts.push(format!("c={}", state.stats_display.tool_str));
    parts.push(format!("a={}", state.stats_display.agnt_str));
    parts.join("|")
}
