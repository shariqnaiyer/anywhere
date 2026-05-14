/**
 * Thin async wrapper around the things-api HTTP server. Mirrors
 * tui/src/api/client.rs (same endpoints, same bearer-token auth, same
 * "flatten errors to a single string" decoding strategy).
 */

export class ApiError extends Error {
  status: number;
  body: string;
  constructor(status: number, body: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

export class ThingsClient {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.token = token;
  }

  /** GET /health — no auth. Used for the startup probe. */
  async ping(timeoutMs: number = 2000): Promise<void> {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), timeoutMs);
    try {
      const resp = await fetch(`${this.baseUrl}/health`, { signal: ctrl.signal });
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }
    } finally {
      clearTimeout(t);
    }
  }

  private headers(): Record<string, string> {
    return {
      authorization: `Bearer ${this.token}`,
      "content-type": "application/json",
    };
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), 30000);
    let resp: Response;
    try {
      resp = await fetch(url, {
        method,
        headers: this.headers(),
        body: body === undefined ? undefined : JSON.stringify(body),
        signal: ctrl.signal,
      });
    } catch (e) {
      clearTimeout(t);
      const msg = (e as Error).message || String(e);
      throw new Error(`${method} ${path}: ${msg}`);
    }
    clearTimeout(t);

    const text = await resp.text();
    if (!resp.ok) {
      throw new ApiError(resp.status, text, errorMessage(resp.status, text));
    }
    if (!text) {
      // 204 / empty body — return null cast to T (callers know which methods are empty).
      return null as unknown as T;
    }
    try {
      return JSON.parse(text) as T;
    } catch (e) {
      throw new Error(
        `${method} ${path}: decode failed (${(e as Error).message}): ${text}`
      );
    }
  }

  get<T>(path: string): Promise<T> {
    return this.request<T>("GET", path);
  }
  post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("POST", path, body ?? {});
  }
  patch<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("PATCH", path, body ?? {});
  }
  delete<T>(path: string): Promise<T> {
    return this.request<T>("DELETE", path);
  }
}

function errorMessage(status: number, body: string): string {
  // Try to pull {"error": "..."} out of the body for nicer messages.
  if (body) {
    try {
      const parsed = JSON.parse(body) as { error?: string };
      if (parsed && typeof parsed.error === "string") {
        return `HTTP ${status}: ${parsed.error}`;
      }
    } catch {
      // fall through
    }
    return `HTTP ${status}: ${body}`;
  }
  return `HTTP ${status}`;
}

/** Percent-encode a path segment. Things 3 IDs are alphanumerics, but tag/area/list names can be anything. */
export function urlencode(s: string): string {
  return encodeURIComponent(s);
}
