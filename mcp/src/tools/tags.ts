import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Tag, Task } from "../types.js";
import { jsonContent } from "../util/format.js";

export function registerTagReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "list_tags",
    {
      title: "List tags",
      description: "List all tags in Things 3, including their parent-tag relationships and keyboard shortcuts.",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => jsonContent(await client.get<Tag[]>(`/tags`))
  );

  server.registerTool(
    "get_tag",
    {
      title: "Get tag",
      description: "Fetch a single tag by ID.",
      inputSchema: { id: z.string().min(1).describe("Things 3 tag ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => jsonContent(await client.get<Tag>(`/tags/${urlencode(id)}`))
  );

  server.registerTool(
    "list_tag_tasks",
    {
      title: "List tasks with tag",
      description: "Return tasks tagged with a specific tag.",
      inputSchema: { id: z.string().min(1).describe("Things 3 tag ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) =>
      jsonContent(await client.get<Task[]>(`/tags/${urlencode(id)}/tasks`))
  );

  server.registerTool(
    "list_tag_children",
    {
      title: "List child tags",
      description: "Return the immediate child tags of a parent tag (for hierarchical tag trees).",
      inputSchema: { id: z.string().min(1).describe("Things 3 tag ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) =>
      jsonContent(await client.get<Tag[]>(`/tags/${urlencode(id)}/children`))
  );
}
