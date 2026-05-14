import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Tag } from "../types.js";
import { jsonContent, okContent } from "../util/format.js";

export function registerTagWriteTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "create_tag",
    {
      title: "Create tag",
      description: "Create a new tag, optionally nested under a parent tag.",
      inputSchema: {
        name: z.string().min(1),
        keyboard_shortcut: z.string().optional(),
        parent_tag: z.string().optional().describe("Exact parent tag name (for hierarchical tags)."),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const t = await client.post<Tag>(`/tags`, input);
      return jsonContent(t);
    }
  );

  server.registerTool(
    "update_tag",
    {
      title: "Update tag",
      description: "Patch a tag. Empty string for `parent_tag` detaches it from its parent.",
      inputSchema: {
        id: z.string().min(1),
        name: z.string().optional(),
        keyboard_shortcut: z.string().optional(),
        parent_tag: z.string().optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const { id, ...rest } = input;
      const t = await client.patch<Tag>(`/tags/${urlencode(id)}`, rest);
      return jsonContent(t);
    }
  );

  server.registerTool(
    "delete_tag",
    {
      title: "Delete tag",
      description: "Delete a tag. Tasks/projects that had this tag lose it but are not deleted.",
      inputSchema: { id: z.string().min(1) },
      annotations: { destructiveHint: true, idempotentHint: false },
    },
    async ({ id }) => {
      await client.delete(`/tags/${urlencode(id)}`);
      return okContent(`Deleted tag ${id}.`);
    }
  );
}
