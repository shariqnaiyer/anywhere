import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient } from "../client.js";
import { okContent } from "../util/format.js";

export function registerTrashTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "empty_trash",
    {
      title: "Empty Things 3 trash",
      description:
        "Permanently delete every task and project currently in the trash. Irreversible. " +
        "Must pass `confirm: true` to execute.",
      inputSchema: {
        confirm: z
          .literal(true)
          .describe("Set to `true` to actually empty the trash. Anything else aborts."),
      },
      annotations: { destructiveHint: true, idempotentHint: false },
    },
    async ({ confirm }) => {
      if (confirm !== true) {
        return okContent("Aborted — confirm was not true.");
      }
      await client.delete(`/trash`);
      return okContent("Emptied trash.");
    }
  );
}
