import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Area, Task } from "../types.js";
import { jsonContent } from "../util/format.js";

export function registerAreaReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "list_areas",
    {
      title: "List areas",
      description:
        "List all Areas in Things 3. Areas are top-level groupings (e.g. 'Career', 'Home') that contain projects and tasks.",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => jsonContent(await client.get<Area[]>(`/areas`))
  );

  server.registerTool(
    "get_area",
    {
      title: "Get area",
      description: "Fetch a single area by ID.",
      inputSchema: { id: z.string().min(1).describe("Things 3 area ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => jsonContent(await client.get<Area>(`/areas/${urlencode(id)}`))
  );

  server.registerTool(
    "list_area_tasks",
    {
      title: "List tasks in area",
      description:
        "Return tasks that belong to a specific area (including tasks nested inside projects under that area).",
      inputSchema: { id: z.string().min(1).describe("Things 3 area ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) =>
      jsonContent(await client.get<Task[]>(`/areas/${urlencode(id)}/tasks`))
  );
}
