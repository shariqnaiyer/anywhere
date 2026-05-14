# things-mcp

An [MCP](https://modelcontextprotocol.io) server that lets Claude (Desktop, Code, or any MCP-aware client) browse and manage your Things 3 tasks. Built on top of the `things-api` HTTP server in this repo.

## What you get

39 tools across reads and writes:

- **Tasks** — list, count, get, create, update, complete, cancel, delete, parse Quicksilver strings, show in UI
- **Projects** — list, get, list tasks, create, update, delete, show
- **Areas** — list, get, list tasks, create, update, delete
- **Tags** — list, get, list tasks, list children, create, update, delete
- **System** — built-in lists, app info, Quick Entry, log completed, focus a list
- **Trash** — empty (requires `confirm: true`)

## Requirements

- macOS with Things 3 installed and running
- A running `things-api` server (see the [root README](../README.md))
- Node 18 or newer
- A bearer token at `~/Library/Application Support/things-api/auth_token` (the server writes this on first launch)

## Install

### From this checkout (recommended while pre-1.0)

```bash
cd mcp
npm install
npm run build
```

### From npm (when published)

```bash
npx -y things-mcp@latest --version
```

## Configure your MCP client

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```jsonc
{
  "mcpServers": {
    "things": {
      "command": "node",
      "args": ["/absolute/path/to/things-api/mcp/dist/index.js"]
    }
  }
}
```

Restart Claude Desktop. The 39 tools should appear in the tool picker.

### Claude Code

```bash
claude mcp add things node /absolute/path/to/things-api/mcp/dist/index.js
```

Or edit `~/.claude.json` / your project's `.claude/mcp.json` directly with the same `command`/`args` shape.

### With overrides

```jsonc
{
  "mcpServers": {
    "things": {
      "command": "node",
      "args": [
        "/absolute/path/to/things-api/mcp/dist/index.js",
        "--url",
        "http://127.0.0.1:3333"
      ],
      "env": {
        "THINGS_AUTH_TOKEN": "thingsapi_..."
      }
    }
  }
}
```

## Configuration resolution

| Setting  | Source (first match wins)                                                                                     | Default                  |
| -------- | ------------------------------------------------------------------------------------------------------------- | ------------------------ |
| Base URL | `--url` flag → `THINGS_API_URL` env                                                                           | `http://127.0.0.1:3333`  |
| Token    | `--token` flag → `THINGS_AUTH_TOKEN` env → `~/Library/Application Support/things-api/auth_token`              | (no default — fails out) |

The remote (cloudflared) URL from `account.json` is **not** auto-discovered — pass it explicitly via `--url` if you want to hit it.

## Date keywords

Pretty much anywhere a date is accepted (`when`, `due_date`), the MCP server translates these keywords into AppleScript-friendly formal dates before sending the request:

- `today`, `tomorrow`, `yesterday`
- `this weekend` / `weekend` (→ next Saturday)
- `next week` (→ next Monday)
- `next month`
- `next sunday`, `next monday`, ..., `next saturday`
- ISO dates (`2026-03-25`)

Anything else is passed through unchanged, so explicit dates like `"March 25, 2026"` still work.

Special cases:

- `when: "anytime"` and `when: "someday"` route the task to the matching list (these aren't valid AppleScript dates).
- Empty string (`""`) clears a date or detaches a project/area/contact (see each tool's description).

## Verify it works

```bash
# From the mcp/ directory after npm run build
npm run inspector
```

Opens `@modelcontextprotocol/inspector` for a click-driven test of every tool.

Or feed a JSON-RPC handshake directly:

```bash
{
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}'
  printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ping","arguments":{}}}'
  sleep 1
} | node dist/index.js
```

## Architecture

```
mcp/
├── package.json
├── tsconfig.json
├── bin/things-mcp.js        # executable launcher (npx-friendly)
├── src/
│   ├── index.ts             # entrypoint: arg parsing + stdio transport
│   ├── config.ts            # token + base URL resolution (mirrors src/config.rs)
│   ├── client.ts            # fetch-based HTTP wrapper
│   ├── types.ts             # wire types + zod fragments
│   ├── server.ts            # registers all tool modules
│   ├── tools/
│   │   ├── tasks.ts         # read tools
│   │   ├── tasks_write.ts   # mutation tools
│   │   ├── areas.ts
│   │   ├── areas_write.ts
│   │   ├── projects.ts
│   │   ├── projects_write.ts
│   │   ├── tags.ts
│   │   ├── tags_write.ts
│   │   ├── lists.ts
│   │   ├── system.ts
│   │   ├── system_write.ts
│   │   └── trash.ts
│   └── util/
│       ├── format.ts        # JSON-content helpers
│       └── dates.ts         # date-keyword normalizer
└── dist/                    # tsc output (gitignored)
```

## Non-goals (for now)

- Contacts / windows / `quit_app` are intentionally not exposed.
- Things 3 has no change notifications; tools are point-in-time. Re-call `list_tasks` to see updates.

## License

MIT — see the [root LICENSE](../LICENSE).
