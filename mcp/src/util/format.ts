/** Wrap a value as an MCP `text` content block holding pretty-printed JSON. */
export function jsonContent(value: unknown) {
  return {
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(value, null, 2),
      },
    ],
  };
}

/** Standard "204 / empty body" response shape. */
export function okContent(summary?: string) {
  const body: Record<string, unknown> = { ok: true };
  if (summary) body.summary = summary;
  return jsonContent(body);
}

/** Build a `?key=value&...` query string from a partial record. Skips undefined / null. */
export function queryString(params: Record<string, string | number | undefined | null>): string {
  const parts: string[] = [];
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === null) continue;
    parts.push(`${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`);
  }
  return parts.length === 0 ? "" : `?${parts.join("&")}`;
}
