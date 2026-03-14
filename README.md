# LFG

WoW-inspired raid frames on a 64x64 LED panel for monitoring your coding agents.

<!-- TODO: replace with actual GIF of the display -->
![LFG in action](docs/hero.gif)

## What is this?

LFG turns a [$25 iDotMatrix LED panel](https://idotmatrix.com) into a real-time raid frame display for your AI coding agents. Each agent gets a sprite, a player ID, and a status icon — just like watching your party in a 10-man raid.

**States:**
- **Working** — agent is actively using tools (sword/potion/compass icon)
- **Idle** — agent has stopped (zzz with marching border)
- **Requesting** — agent needs approval, i.e. *standing in fire* (fire icon with marching border)

The most important thing LFG does is make idle and approval states impossible to miss. When an agent needs you, you see the fire icon in your peripheral vision without switching windows.

## How it works

```
Claude Code / Cursor hooks
        ↓
   boopifier (event normalizer)
        ↓
   HTTP webhook POST
        ↓
   LFG (Rust server)
        ↓
   BLE → iDotMatrix 64x64 LED
```

LFG receives webhook events (`PreToolUse`, `PostToolUse`, `PermissionRequest`, `Stop`, etc.), manages an agent state machine, renders 8x8 pixel-art sprites into animated GIF frames, and sends them over BLE to the display.

## Features

- **10 agent slots** across 5 columns, 2 rows — grouped by host (IDE/terminal)
- **11 sprite themes** — Slimes, Ghosts, Space Invaders, Pac-Men, Mushrooms, Jumpman, Creepers, Frogger, Q*bert, Kirby, Zelda Hearts
- **5 ability icons** — Star (subagent), Sword (write/edit/bash), Potion (think), Chest (read), Compass (search/web)
- **BLE auto-discovery** and reconnection with heartbeat keepalive
- **SQLite persistence** for cumulative stats across restarts
- **Stats bar** — agent-minutes, tool calls, unique agents
- **Multi-IDE support** — Claude Code and Cursor via [boopifier](https://github.com/terraboops/boopifier)
- **HTTP API** for status, reset, theme switching, and debug GIF export

## Quick start

### Hardware

You need an iDotMatrix 64x64 LED panel (~$25 on AliExpress). Any `IDM-*` BLE device should work.

### Build & run

```bash
# Clone and build
git clone https://github.com/terraboops/lfg.git
cd lfg
cargo build --release

# Run (auto-discovers IDM-* BLE device)
./target/release/lfg

# Run without BLE (HTTP-only, for testing)
./target/release/lfg --no-ble
```

### Configure hooks

LFG receives events via HTTP webhooks. Use [boopifier](https://github.com/terraboops/boopifier) to wire up Claude Code and/or Cursor hooks:

```bash
# Claude Code hooks (~/.claude/hooks.json)
# Cursor hooks (~/.cursor/hooks.json)
# See boopifier README for setup
```

Or send events directly:

```bash
curl -X POST http://localhost:5555/webhook \
  -H 'Content-Type: application/json' \
  -d '{"text": "PreToolUse|session-id-here|Bash"}' \
  -G -d 'host=claude'
```

### API

```bash
curl localhost:5555/status          # Current state
curl localhost:5555/hosts           # Host/agent mapping
curl -X POST localhost:5555/reset   # Clear everything
curl localhost:5555/theme           # List themes
curl -X POST localhost:5555/theme/2 # Set theme (Space Invaders)
```

## The state machine

Getting agent state right is the hard part. Hooks fire out of order and overlap — `PermissionRequest` and `PreToolUse` arrive ~100μs apart for the same tool, and `PostToolUse` fires after every tool call, not just approvals.

The key design decisions:

- **Requesting is sticky** — only cleared by `PostToolUse` (approval granted + tool ran) or `Stop`/`SessionEnd`
- **PreToolUse won't override Requesting** — prevents the fire icon from flickering away while blocked on approval
- **PostToolUse doesn't transition to Idle** — tools fire rapidly in sequence; only `Stop` means truly idle
- **Idle and Requesting always win** from `Stop`/`SessionEnd` and `PermissionRequest` respectively

## Architecture

```
src/
├── main.rs      # CLI args, DB init, server startup
├── http.rs      # Axum routes (webhook, status, themes)
├── event.rs     # State machine, stale agent cleanup, stats
├── gateway.rs   # Host/column allocation, agent join logic
├── state.rs     # Shared state types (Agent, Column, Host, Stats)
├── render.rs    # Canvas, sprite/icon drawing, GIF encoding
├── sprites.rs   # 8x8 pixel art themes, icons, font, layout
├── ble.rs       # BLE discovery, connection, GIF packet protocol
└── db.rs        # SQLite persistence for cumulative stats
```

## License

MIT
