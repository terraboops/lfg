/**
 * LFG Bridge Plugin for OpenCode
 *
 * Sends events to the lfg LED panel server when OpenCode tools are executed,
 * allowing real-time visualization of agent activity.
 *
 * Installation:
 * 1. Copy this file to ~/.config/opencode/plugins/lfg-bridge.js
 * 2. Add to ~/.config/opencode/opencode.json:
 *      "plugin": ["./plugins/lfg-bridge.js"]
 * 3. Set LFG_WEBHOOK_URL if lfg is not on the default port (optional)
 * 4. Restart OpenCode
 */

const LFG_WEBHOOK_URL = process.env.LFG_WEBHOOK_URL || "http://localhost:6969/webhook";
const LFG_HOST_IDENTIFIER = "opencode";

/**
 * Send event to lfg webhook.
 * Wire format: POST /webhook?host=<host>  { "text": "EventType|sessionID|toolName" }
 */
async function sendToLfg(eventType, sessionID, toolName) {
  const url = new URL(LFG_WEBHOOK_URL);
  url.searchParams.set("host", LFG_HOST_IDENTIFIER);
  try {
    const response = await fetch(url.toString(), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ text: `${eventType}|${sessionID}|${toolName}` })
    });
    if (!response.ok) {
      console.error(`[lfg-bridge] Webhook failed: ${response.status} ${response.statusText}`);
    }
  } catch (error) {
    console.error(`[lfg-bridge] Webhook error: ${error.message}`);
  }
}

/**
 * LFG Bridge Plugin — implements the OpenCode PluginModule interface.
 * The `server` export is required by OpenCode's plugin loader.
 */
export const server = async ({ project, client, $, directory, worktree }) => {
  return {
    /** Fires before a tool is executed → PreToolUse */
    "tool.execute.before": async (input, output) => {
      await sendToLfg("PreToolUse", input.sessionID ?? "", input.tool ?? "");
    },

    /** Fires after a tool is executed → PostToolUse */
    "tool.execute.after": async (input, output) => {
      await sendToLfg("PostToolUse", input.sessionID ?? "", input.tool ?? "");
    },

    /** Fires when permission is requested → PermissionRequest */
    "permission.ask": async (input, output) => {
      await sendToLfg("PermissionRequest", input.sessionID ?? "", "");
    },

    /** Catch-all: detect session.idle and session.error → Stop */
    event: async ({ event }) => {
      if (event.type === "session.idle" || event.type === "session.error") {
        await sendToLfg("Stop", event.properties?.sessionID ?? "", "");
      }
    },
  };
};
