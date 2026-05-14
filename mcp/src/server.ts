import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { ThingsClient } from "./client.js";
import { registerTaskReadTools } from "./tools/tasks.js";
import { registerTaskWriteTools } from "./tools/tasks_write.js";
import { registerAreaReadTools } from "./tools/areas.js";
import { registerAreaWriteTools } from "./tools/areas_write.js";
import { registerProjectReadTools } from "./tools/projects.js";
import { registerProjectWriteTools } from "./tools/projects_write.js";
import { registerTagReadTools } from "./tools/tags.js";
import { registerTagWriteTools } from "./tools/tags_write.js";
import { registerListReadTools } from "./tools/lists.js";
import { registerSystemReadTools } from "./tools/system.js";
import { registerSystemWriteTools } from "./tools/system_write.js";
import { registerTrashTools } from "./tools/trash.js";

export function registerTools(server: McpServer, client: ThingsClient): void {
  // Reads
  registerTaskReadTools(server, client);
  registerAreaReadTools(server, client);
  registerProjectReadTools(server, client);
  registerTagReadTools(server, client);
  registerListReadTools(server, client);
  registerSystemReadTools(server, client);

  // Writes
  registerTaskWriteTools(server, client);
  registerProjectWriteTools(server, client);
  registerAreaWriteTools(server, client);
  registerTagWriteTools(server, client);
  registerSystemWriteTools(server, client);
  registerTrashTools(server, client);
}
