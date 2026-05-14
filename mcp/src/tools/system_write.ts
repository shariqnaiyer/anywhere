import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import { okContent } from "../util/format.js";

export function registerSystemWriteTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "show_list",
    {
      title: "Show list in UI",
      description:
        "Focus a built-in list (inbox/today/upcoming/anytime/someday/logbook/trash) in the Things 3 UI.",
      inputSchema: {
        name: z
          .enum(["inbox", "today", "upcoming", "anytime", "someday", "logbook", "trash"])
          .describe("Which list to focus."),
      },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ name }) => {
      await client.post(`/lists/${urlencode(name)}/show`);
      return okContent(`Shown list ${name}.`);
    }
  );

  server.registerTool(
    "show_quick_entry",
    {
      title: "Show Quick Entry panel",
      description:
        "Open the Things 3 Quick Entry panel, optionally pre-filled. With `autofill: true`, Things 3 captures from the foreground app.",
      inputSchema: {
        title: z.string().optional(),
        notes: z.string().optional(),
        due_date: z.string().optional(),
        tags: z.array(z.string()).optional(),
        autofill: z
          .boolean()
          .optional()
          .describe("If true, Things 3 prefills the panel from the frontmost app (e.g. Mail)."),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      await client.post(`/system/quick-entry`, input);
      return okContent("Opened Quick Entry.");
    }
  );

  server.registerTool(
    "log_completed_now",
    {
      title: "Log completed items now",
      description:
        "Run Things 3's 'Log Completed Items' action immediately, moving today's completed tasks into the Logbook.",
      inputSchema: {},
      annotations: { destructiveHint: false, idempotentHint: true },
    },
    async () => {
      await client.post(`/system/log-completed`);
      return okContent("Logged completed items.");
    }
  );
}
