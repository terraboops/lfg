# OpenCode LFG Integration

Real-time LED panel monitoring for OpenCode using the native plugin system.

## Overview

This integration uses OpenCode's plugin system to send events directly to lfg when tools are executed. No MCP server required - just a simple drop-in plugin.

## Plugin Hooks Used

| OpenCode Event | lfg Event | Description |
|----------------|-----------|---------------|
| `tool.execute.before` | `PreToolUse` | Tool execution started |
| `tool.execute.after` | `PostToolUse` | Tool execution completed |
| `permission.ask` | `PermissionRequest` | Permission prompt shown |
| `session.idle` | `Stop` | Session completed |
| `session.error` | `Stop` | Session errored |

## Installation

### Option 1: Global Installation (Recommended)

```bash
# Create global plugins directory
mkdir -p ~/.config/opencode/plugins/

# Copy the plugin
cp examples/opencode/lfg-bridge.js ~/.config/opencode/plugins/
```

Then add the `plugin` key to `~/.config/opencode/opencode.json`:

```json
{
  "plugin": ["./plugins/lfg-bridge.js"]
}
```

### Option 2: Project-Specific Installation

```bash
# Create project plugins directory
mkdir -p .opencode/plugins/

# Copy the plugin
cp examples/opencode/lfg-bridge.js .opencode/plugins/
```

Then add the `plugin` key to your project's `opencode.json`:

```json
{
  "plugin": ["./plugins/lfg-bridge.js"]
}
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LFG_WEBHOOK_URL` | `http://localhost:6969/webhook` | lfg webhook endpoint |

### Example: Standalone OpenCode

```bash
# Start lfg first
lfg &

# Set webhook URL and run OpenCode
export LFG_WEBHOOK_URL="http://localhost:6969/webhook"
opencode
```

### Example: With Boopifier

```bash
# Using the boopifier wrapper
export LFG_WEBHOOK_URL="http://localhost:6969/webhook"
boopifier opencode
```

## Verification

1. Start lfg: `lfg &`
2. Install the plugin as shown above
3. Run OpenCode: `opencode`
4. Execute a tool (e.g., ask it to read a file)
5. Check the LED panel - you should see activity!

## Troubleshooting

### Plugin not loading

Check OpenCode startup logs for `[lfg-bridge] Plugin initialized`.

### Webhook not sending

Verify `LFG_WEBHOOK_URL` is set correctly:
```bash
echo $LFG_WEBHOOK_URL
```

### Events not appearing on panel

Check lfg is running and listening on the expected port:
```bash
curl http://localhost:6969/status
```

## How It Works

1. OpenCode loads plugins from `~/.config/opencode/plugins/` or `.opencode/plugins/`
2. The lfg-bridge plugin registers hooks for tool execution events
3. When OpenCode executes a tool, the plugin sends an HTTP POST to lfg
4. lfg receives the event and updates the LED panel display

## References

- [OpenCode Plugin Documentation](https://opencode.ai/docs/plugins/)
- [lfg Project](https://github.com/timlevett/lfg)
