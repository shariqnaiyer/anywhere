import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { ThingsClient, urlencode } from "../client.js";
import { SpecialList, type Task } from "../types.js";
import { jsonContent, okContent } from "../util/format.js";
import { normalizeDate } from "../util/dates.js";

/**
 * Translate the MCP-friendly `when` field back to the server's `activation_date`.
 * Two layers of work:
 *   1) "anytime" / "someday" aren't real dates — route via `list` instead.
 *   2) Date keywords ("today", "tomorrow", "next saturday", ISO dates) get
 *      normalized to a formal English date that AppleScript's `date "..."` accepts.
 *   Empty string is preserved (server treats it as "clear").
 */
function renameWhen<T extends { when?: string; due_date?: string; list?: string; [k: string]: unknown }>(
  obj: T
): Omit<T, "when"> & { activation_date?: string; list?: string } {
  const { when, ...rest } = obj;
  const out: Record<string, unknown> = { ...rest };

  if (when !== undefined) {
    const trimmed = when.trim().toLowerCase();
    if (trimmed === "anytime" || trimmed === "someday") {
      // Don't overwrite an explicit list= value the caller already set.
      if (out.list === undefined) out.list = trimmed;
    } else if (trimmed === "upcoming") {
      // 'upcoming' isn't a date either; route via list.
      if (out.list === undefined) out.list = "upcoming";
    } else if (when === "") {
      // Empty string = clear the activation date.
      out.activation_date = "";
    } else {
      out.activation_date = normalizeDate(when);
    }
  }

  if (typeof out.due_date === "string" && out.due_date !== "") {
    out.due_date = normalizeDate(out.due_date as string);
  }

  return out as Omit<T, "when"> & { activation_date?: string; list?: string };
}

const dateDesc =
  "Date string parseable by AppleScript (e.g. 'tomorrow', 'next monday', 'March 25, 2026', '2026-03-25').";
const clearableDateDesc =
  dateDesc + " Pass an empty string \"\" to clear this date.";
const clearableNameDesc =
  "Exact name to attach. Pass an empty string \"\" to detach.";

export function registerTaskWriteTools(server: McpServer, client: ThingsClient): void {
  server.registerTool(
    "create_task",
    {
      title: "Create task",
      description:
        "Create a new task in Things 3. Use `project` or `area` to file it; use `when` to schedule it (or `list` to place it on a special list).",
      inputSchema: {
        title: z.string().min(1).describe("Task title."),
        notes: z.string().optional().describe("Markdown-friendly notes body."),
        due_date: z.string().optional().describe("Deadline. " + dateDesc),
        when: z
          .string()
          .optional()
          .describe(
            'Scheduled date — "When" in the Things UI. ' +
              dateDesc +
              " Special values: 'today', 'tomorrow', 'anytime', 'someday'."
          ),
        list: SpecialList.optional().describe(
          "Place on a special list. Ignored if `project` or `area` is set."
        ),
        tags: z.array(z.string()).optional().describe("Tag names to attach."),
        project: z.string().optional().describe("Exact project name to file under."),
        area: z.string().optional().describe("Exact area name to file under."),
        contact: z
          .string()
          .optional()
          .describe("Exact contact name (must exist in Things contacts)."),
        checklist_items: z
          .array(z.string())
          .optional()
          .describe("Checklist sub-item titles."),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const body = renameWhen(input);
      const task = await client.post<Task>(`/tasks`, body);
      return jsonContent(task);
    }
  );

  server.registerTool(
    "update_task",
    {
      title: "Update task",
      description:
        "Patch an existing task. Only include fields you want to change. " +
        "For `due_date`, `when`, `project`, `area`, `contact`: pass an empty string \"\" to clear/detach.",
      inputSchema: {
        id: z.string().min(1).describe("Things 3 task ID."),
        title: z.string().optional(),
        notes: z.string().optional(),
        due_date: z.string().optional().describe("New deadline. " + clearableDateDesc),
        when: z
          .string()
          .optional()
          .describe("New scheduled date. " + clearableDateDesc),
        list: SpecialList.optional().describe("Move to a special list."),
        tags: z
          .array(z.string())
          .optional()
          .describe("Replace the task's tag list with this set."),
        project: z.string().optional().describe(clearableNameDesc),
        area: z.string().optional().describe(clearableNameDesc),
        contact: z.string().optional().describe(clearableNameDesc),
        completed: z.boolean().optional(),
        canceled: z.boolean().optional(),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async (input) => {
      const { id, ...rest } = input;
      const body = renameWhen(rest);
      const task = await client.patch<Task>(`/tasks/${urlencode(id)}`, body);
      return jsonContent(task);
    }
  );

  server.registerTool(
    "complete_task",
    {
      title: "Complete task",
      description: "Mark a task as completed.",
      inputSchema: { id: z.string().min(1).describe("Things 3 task ID.") },
      annotations: { idempotentHint: true, destructiveHint: false },
    },
    async ({ id }) => {
      await client.patch(`/tasks/${urlencode(id)}/complete`);
      return okContent(`Completed task ${id}.`);
    }
  );

  server.registerTool(
    "cancel_task",
    {
      title: "Cancel task",
      description:
        "Mark a task as canceled (struck through but kept in the logbook). Different from `delete_task`, which removes it.",
      inputSchema: { id: z.string().min(1).describe("Things 3 task ID.") },
      annotations: { idempotentHint: true, destructiveHint: false },
    },
    async ({ id }) => {
      await client.patch(`/tasks/${urlencode(id)}/cancel`);
      return okContent(`Canceled task ${id}.`);
    }
  );

  server.registerTool(
    "delete_task",
    {
      title: "Delete task",
      description:
        "Move a task to the Things 3 trash. Use `empty_trash` afterward to permanently remove it.",
      inputSchema: { id: z.string().min(1).describe("Things 3 task ID.") },
      annotations: { destructiveHint: true, idempotentHint: false },
    },
    async ({ id }) => {
      await client.delete(`/tasks/${urlencode(id)}`);
      return okContent(`Moved task ${id} to trash.`);
    }
  );

  server.registerTool(
    "parse_task",
    {
      title: "Parse Quicksilver string into a task",
      description:
        "Create a task from a Things 3 Quicksilver natural-language string (e.g. 'Buy milk @home #shopping !tomorrow'). " +
        "Use this ONLY when the user gave you a literal Quicksilver-style string; otherwise prefer `create_task` with structured fields.",
      inputSchema: {
        text: z.string().min(1).describe("Quicksilver-formatted input."),
      },
      annotations: { destructiveHint: false, idempotentHint: false },
    },
    async ({ text }) => {
      const task = await client.post<Task>(`/tasks/parse`, { text });
      return jsonContent(task);
    }
  );

  server.registerTool(
    "show_task",
    {
      title: "Show task in UI",
      description: "Focus the given task in the Things 3 UI (does not change data).",
      inputSchema: { id: z.string().min(1).describe("Things 3 task ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => {
      await client.post(`/tasks/${urlencode(id)}/show`);
      return okContent(`Shown task ${id}.`);
    }
  );

  server.registerTool(
    "edit_task",
    {
      title: "Open task editor",
      description: "Open the given task in the Things 3 quick-edit overlay (does not change data).",
      inputSchema: { id: z.string().min(1).describe("Things 3 task ID.") },
      annotations: { readOnlyHint: true, idempotentHint: true },
    },
    async ({ id }) => {
      await client.post(`/tasks/${urlencode(id)}/edit`);
      return okContent(`Opened editor for task ${id}.`);
    }
  );
}
