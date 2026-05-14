import { z } from "zod";

/** The seven special lists Things 3 exposes. */
export const SpecialList = z.enum([
  "inbox",
  "today",
  "upcoming",
  "anytime",
  "someday",
  "logbook",
  "trash",
]);
export type SpecialList = z.infer<typeof SpecialList>;

/** Pagination fragment for endpoints that support it (`/tasks` and `/tasks/count`). */
export const PaginationShape = {
  limit: z
    .number()
    .int()
    .min(1)
    .max(500)
    .optional()
    .describe("Maximum number of results to return (default 100, max 500)."),
  offset: z
    .number()
    .int()
    .min(0)
    .optional()
    .describe("Number of results to skip from the start (default 0)."),
};

/** Wire types — kept loose since responses are passed through to the LLM as JSON. */
export interface Task {
  id: string;
  title: string;
  notes?: string | null;
  due_date?: string | null;
  activation_date?: string | null;
  list?: string | null;
  project?: string | null;
  area?: string | null;
  contact?: string | null;
  tags: string[];
  checklist_items: { title: string; completed: boolean }[];
  completed: boolean;
  canceled: boolean;
  creation_date?: string | null;
  modification_date?: string | null;
  completion_date?: string | null;
  cancellation_date?: string | null;
}

export interface Project {
  id: string;
  title: string;
  notes?: string | null;
  due_date?: string | null;
  activation_date?: string | null;
  area?: string | null;
  tags: string[];
  completed: boolean;
  canceled: boolean;
}

export interface Area {
  id: string;
  title: string;
  collapsed: boolean;
  tags: string[];
}

export interface Tag {
  id: string;
  name: string;
  keyboard_shortcut?: string | null;
  parent_tag?: string | null;
}

export interface ListInfo {
  id: string;
  name: string;
}

export interface CountResponse {
  count: number;
  scope: string;
}

export interface AppInfo {
  name: string;
  version: string;
  frontmost: boolean;
  current_list_name?: string | null;
  current_list_url?: string | null;
}
