import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Project, Task } from "../types.js";
import { jsonContent } from "../util/format.js";

export function registerProjectReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "list_projects",
    {
      title: "List projects",
      description:
        "List all projects in Things 3. Projects are containers for related tasks; they live inside an area (or stand alone).",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => jsonContent(await client.get<Project[]>(`/projects`))
  );

  server.registerTool(
    "get_project",
    {
      title: "Get project",
      description: "Fetch a single project by ID.",
      inputSchema: { id: z.string().min(1).describe("Things 3 project ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) =>
      jsonContent(await client.get<Project>(`/projects/${urlencode(id)}`))
  );

  server.registerTool(
    "list_project_tasks",
    {
      title: "List tasks in project",
      description: "Return tasks that belong to a specific project.",
      inputSchema: { id: z.string().min(1).describe("Things 3 project ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) =>
      jsonContent(await client.get<Task[]>(`/projects/${urlencode(id)}/tasks`))
  );
}
