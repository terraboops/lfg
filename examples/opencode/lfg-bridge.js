/**
 * LFG Bridge Plugin for OpenCode
 * 
 * This plugin sends events to the lfg LED panel server when OpenCode tools
 * are executed, allowing real-time visualization of agent activity.
 * 
 * Installation:
 * 1. Copy this file to ~/.config/opencode/plugins/lfg-bridge.js (global)
 *    OR .opencode/plugins/lfg-bridge.js (project-specific)
 * 2. Set LFG_WEBHOOK_URL environment variable (default: http://localhost:6969/webhook)
 * 3. Restart OpenCode
 */

const LFG_WEBHOOK_URL = process.env.LFG_WEBHOOK_URL || "http://localhost:6969/webhook";
const LFG_HOST_IDENTIFIER = "opencode";

/**
 * Send event to lfg webhook
 */
async function sendToLfg(eventType, payload) {
  try {
    const response = await fetch(LFG_WEBHOOK_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        host: LFG_HOST_IDENTIFIER,
        event: eventType,
        timestamp: Date.now(),
        ...payload
      })
    });

    if (!response.ok) {
      console.error(`[lfg-bridge] Webhook failed: ${response.status} ${response.statusText}`);
    }
  } catch (error) {
    console.error(`[lfg-bridge] Webhook error: ${error.message}`);
  }
}

/**
 * LFG Bridge Plugin
 * Hooks into OpenCode events and forwards them to lfg
 */
export const LfgBridge = async ({ project, client, $, directory, worktree }) => {
  console.log("[lfg-bridge] Plugin initialized");
  console.log(`[lfg-bridge] Webhook URL: ${LFG_WEBHOOK_URL}`);

  return {
    /**
     * Fires before a tool is executed
     * Maps to lfg: PreToolUse
     */
    "tool.execute.before": async (input, output) => {
      await sendToLfg("PreToolUse", {
        tool: input.tool,
        args: output.args,
        cwd: directory,
        worktree: worktree
      });
    },

    /**
     * Fires after a tool is executed
     * Maps to lfg: PostToolUse
     */
    "tool.execute.after": async (input, output) => {
      await sendToLfg("PostToolUse", {
        tool: input.tool,
        args: output.args,
        result: output.result,
        cwd: directory,
        worktree: worktree
      });
    },

    /**
     * Fires when a permission is requested
     * Maps to lfg: PermissionRequest
     */
    "permission.asked": async (input, output) => {
      await sendToLfg("PermissionRequest", {
        permission: input.permission,
        message: input.message
      });
    },

    /**
     * Fires when a permission is replied to
     * Maps to lfg: PermissionRequest (with response)
     */
    "permission.replied": async (input, output) => {
      await sendToLfg("PermissionRequest", {
        permission: input.permission,
        granted: output.granted,
        message: input.message
      });
    },

    /**
     * Fires when session becomes idle (completed)
     * Maps to lfg: Stop / SessionEnd
     */
    "session.idle": async (input, output) => {
      await sendToLfg("Stop", {
        reason: "session_idle",
        cwd: directory
      });
    },

    /**
     * Fires when session errors
     */
    "session.error": async (input, output) => {
      await sendToLfg("Stop", {
        reason: "error",
        error: input.error?.message || "Unknown error",
        cwd: directory
      });
    },

    /**
     * Fires on any event (catch-all for debugging)
     */
    event: async ({ event }) => {
      // Log all events for debugging (optional)
      // console.log(`[lfg-bridge] Event: ${event.type}`);
    }
  };
};

export default LfgBridge;
