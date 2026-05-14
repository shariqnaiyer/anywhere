import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Area } from "../types.js";
import { jsonContent, okContent } from "../util/format.js";

export function registerAreaWriteTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "create_area",
    {
      title: "Create area",
      description: "Create a new area in Things 3 (top-level grouping for projects/tasks).",
      inputSchema: {
        title: z.string().min(1),
        tags: z.array(z.string()).optional(),
        collapsed: z.boolean().optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const a = await client.post<Area>(`/areas`, input);
      return jsonContent(a);
    }
  );

  server.registerTool(
    "update_area",
    {
      title: "Update area",
      description: "Patch an area's title, tags, or collapsed state.",
      inputSchema: {
        id: z.string().min(1),
        title: z.string().optional(),
        tags: z.array(z.string()).optional(),
        collapsed: z.boolean().optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const { id, ...rest } = input;
      const a = await client.patch<Area>(`/areas/${urlencode(id)}`, rest);
      return jsonContent(a);
    }
  );

  server.registerTool(
    "delete_area",
    {
      title: "Delete area",
      description:
        "Delete an area in Things 3. This will also remove its projects/tasks from the sidebar — use carefully.",
      inputSchema: { id: z.string().min(1) },
      annotations: { destructiveHint: true, idempotentHint: false },
    },
    async ({ id }) => {
      await client.delete(`/areas/${urlencode(id)}`);
      return okContent(`Deleted area ${id}.`);
    }
  );
}
