import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import { SpecialList, PaginationShape, type Task, type CountResponse } from "../types.js";
import { jsonContent, queryString } from "../util/format.js";

export function registerTaskReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "list_tasks",
    {
      title: "List tasks",
      description:
        "List tasks from one of the seven special lists in Things 3. Defaults to the inbox if `list` is omitted. " +
        "Supports `limit` (default 100, max 500) and `offset` for pagination.",
      inputSchema: {
        list: SpecialList.optional().describe(
          "Which special list to read from. One of inbox, today, upcoming, anytime, someday, logbook, trash."
        ),
        ...PaginationShape,
      },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ list, limit, offset }) => {
      const qs = queryString({ list, limit, offset });
      const tasks = await client.get<Task[]>(`/tasks${qs}`);
      return jsonContent(tasks);
    }
  );

  server.registerTool(
    "list_selected_tasks",
    {
      title: "List selected tasks",
      description:
        "Return tasks currently selected in the Things 3 UI. Useful when the user says 'this task' or 'these'.",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => {
      const tasks = await client.get<Task[]>(`/tasks/selected`);
      return jsonContent(tasks);
    }
  );

  server.registerTool(
    "count_tasks",
    {
      title: "Count tasks",
      description:
        "Return how many tasks are in a special list, without fetching them. Defaults to inbox if `list` is omitted.",
      inputSchema: {
        list: SpecialList.optional(),
      },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ list }) => {
      const qs = queryString({ list });
      const r = await client.get<CountResponse>(`/tasks/count${qs}`);
      return jsonContent(r);
    }
  );

  server.registerTool(
    "get_task",
    {
      title: "Get task",
      description: "Fetch a single task by Things 3 ID.",
      inputSchema: {
        id: z.string().min(1).describe("Things 3 task ID."),
      },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => {
      const t = await client.get<Task>(`/tasks/${urlencode(id)}`);
      return jsonContent(t);
    }
  );
}
