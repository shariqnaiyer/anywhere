import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient } from "../client.js";
import type { AppInfo } from "../types.js";
import { jsonContent } from "../util/format.js";

export function registerSystemReadTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "app_info",
    {
      title: "Things 3 app info",
      description:
        "Return Things 3 app state: name, version, whether it is frontmost, and the list currently focused in the UI.",
      inputSchema: {},
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async () => jsonContent(await client.get<AppInfo>(`/info`))
  );
}
