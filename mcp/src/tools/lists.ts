import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient } from "../client.js";
import type { ListInfo } from "../types.js";
import { jsonContent } from "../util/format.js";

export function registerListReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "list_lists",
    {
      title: "List built-in lists",
      description:
        "Return the seven built-in Things 3 lists (Inbox, Today, Upcoming, Anytime, Someday, Logbook, Trash) with their IDs.",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => jsonContent(await client.get<ListInfo[]>(`/lists`))
  );
}
