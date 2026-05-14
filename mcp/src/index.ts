#!/usr/bin/env node
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { resolveConfig } from "./config.js";
import { ThingsClient } from "./client.js";
import { registerTools } from "./server.js";

const VERSION = "0.1.0";

function parseArgs(argv: string[]): { url?: string; token?: string } {
  const out: { url?: string; token?: string } = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--url" && i + 1 < argv.length) {
      out.url = argv[++i];
    } else if (a === "--token" && i + 1 < argv.length) {
      out.token = argv[++i];
    } else if (a === "--version" || a === "-V") {
      process.stdout.write(`things-mcp ${VERSION}\n`);
      process.exit(0);
    } else if (a === "--help" || a === "-h") {
      process.stdout.write(
        `things-mcp ${VERSION} — MCP server for Things 3 via things-api\n` +
          `\n` +
          `Usage: things-mcp [--url <base-url>] [--token <bearer>]\n` +
          `\n` +
          `Resolution order (first match wins):\n` +
          `  base URL : --url → THINGS_API_URL → http://127.0.0.1:3333\n` +
          `  token    : --token → THINGS_AUTH_TOKEN → ~/Library/Application Support/things-api/auth_token\n`
      );
      process.exit(0);
    }
  }
  return out;
}

async function main(): Promise<void> {
  const cli = parseArgs(process.argv.slice(2));
  const cfg = resolveConfig({ urlOverride: cli.url, tokenOverride: cli.token });

  if (!cfg.token) {
    process.stderr.write(
      `things-mcp: no auth token found.\n` +
        `  Looked at: --token flag, $THINGS_AUTH_TOKEN, ${cfg.tokenPathChecked}\n` +
        `  Start the things-api server once to generate one, or pass --token.\n`
    );
    process.exit(1);
  }

  const client = new ThingsClient(cfg.baseUrl, cfg.token);

  // Best-effort ping. Don't exit on failure — the server might come up later;
  // first real tool call will surface the connection error to Claude clearly.
  try {
    await client.ping(2000);
  } catch (e) {
    process.stderr.write(
      `things-mcp: warning — ${cfg.baseUrl} is unreachable (${(e as Error).message}).\n` +
        `  Tool calls will retry against this URL.\n`
    );
  }

  const server = new McpServer({
    name: "things-mcp",
    version: VERSION,
  });

  // Always-on diagnostic tool.
  server.registerTool(
    "ping",
    {
      title: "Ping",
      description:
        "Health check. Returns the things-mcp version and whether the things-api server is reachable.",
      inputSchema: {},
    },
    async () => {
      let reachable = true;
      let detail = "";
      try {
        await client.ping(2000);
      } catch (e) {
        reachable = false;
        detail = (e as Error).message;
      }
      const result = {
        ok: reachable,
        version: VERSION,
        baseUrl: cfg.baseUrl,
        ...(detail ? { error: detail } : {}),
      };
      return {
        content: [
          { type: "text", text: JSON.stringify(result, null, 2) },
        ],
      };
    }
  );

  registerTools(server, client);

  const transport = new StdioServerTransport();
  await server.connect(transport);

  // Keep the process alive; the transport handles stdin EOF.
}

// Suppress unhandledRejection noise on stdout — those would corrupt the JSON-RPC stream.
process.on("unhandledRejection", (reason) => {
  process.stderr.write(`things-mcp unhandledRejection: ${String(reason)}\n`);
});

main().catch((err) => {
  process.stderr.write(`things-mcp fatal: ${err?.stack ?? err}\n`);
  process.exit(1);
});

// Ignore unused var lint — `z` is here so tools/*.ts can import zod from the same dep tree.
void z;
