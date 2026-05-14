import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import type { Project } from "../types.js";
import { jsonContent, okContent } from "../util/format.js";
import { normalizeDate } from "../util/dates.js";

const dateDesc =
  "Date string parseable by AppleScript (e.g. 'tomorrow', 'next monday', 'March 25, 2026', '2026-03-25').";

function renameWhen<T extends { when?: string; due_date?: string; [k: string]: unknown }>(
  obj: T
): Omit<T, "when"> & { activation_date?: string } {
  const { when, ...rest } = obj;
  const out: Record<string, unknown> = { ...rest };
  if (when !== undefined) {
    if (when === "") {
      out.activation_date = "";
    } else {
      out.activation_date = normalizeDate(when);
    }
  }
  if (typeof out.due_date === "string" && out.due_date !== "") {
    out.due_date = normalizeDate(out.due_date as string);
  }
  return out as Omit<T, "when"> & { activation_date?: string };
}

export function registerProjectWriteTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "create_project",
    {
      title: "Create project",
      description:
        "Create a new project in Things 3. Use `area` to file it under an area; otherwise it becomes a top-level project.",
      inputSchema: {
        title: z.string().min(1),
        notes: z.string().optional(),
        due_date: z.string().optional().describe("Deadline. " + dateDesc),
        when: z.string().optional().describe("Scheduled date. " + dateDesc),
        area: z.string().optional().describe("Exact area name to file under."),
        tags: z.array(z.string()).optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const body = renameWhen(input);
      const p = await client.post<Project>(`/projects`, body);
      return jsonContent(p);
    }
  );

  server.registerTool(
    "update_project",
    {
      title: "Update project",
      description:
        "Patch a project. Only include fields to change. Pass empty string to clear date / detach area.",
      inputSchema: {
        id: z.string().min(1).describe("Things 3 project ID."),
        title: z.string().optional(),
        notes: z.string().optional(),
        due_date: z.string().optional(),
        when: z.string().optional(),
        area: z.string().optional(),
        tags: z.array(z.string()).optional(),
        completed: z.boolean().optional(),
        canceled: z.boolean().optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const { id, ...rest } = input;
      const body = renameWhen(rest);
      const p = await client.patch<Project>(`/projects/${urlencode(id)}`, body);
      return jsonContent(p);
    }
  );

  server.registerTool(
    "delete_project",
    {
      title: "Delete project",
      description: "Move a project to the Things 3 trash.",
      inputSchema: { id: z.string().min(1) },
      annotations: { destructiveHint: true, idempotentHint: false },
    },
    async ({ id }) => {
      await client.delete(`/projects/${urlencode(id)}`);
      return okContent(`Moved project ${id} to trash.`);
    }
  );

  server.registerTool(
    "show_project",
    {
      title: "Show project in UI",
      description: "Focus the given project in the Things 3 UI.",
      inputSchema: { id: z.string().min(1) },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => {
      await client.post(`/projects/${urlencode(id)}/show`);
      return okContent(`Shown project ${id}.`);
    }
  );
}
