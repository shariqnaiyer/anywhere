# things-tui

Terminal UI for [things-api](../). Browse Inbox, Today, projects, areas, and tags,
and create / edit / complete / schedule / move tasks — all from your terminal.

```
┌─ Lists ──────────┬─ Today ─────────────────────────┬─ Detail ──────────┐
│ 📥 Inbox      3  │ ☐ Buy groceries                 │ Email Sarah       │
│ ⭐ Today      5  │ ☐ Finish proposal      · #work  │                   │
│ 📅 Upcoming      │▶☐ Email Sarah         · @sarah  │ Notes:            │
│ 🗂 Anytime       │ ☐ Read RFC 9110                 │  re: Q1 review    │
│ 💭 Someday       │                                 │                   │
│ 📚 Logbook       │                                 │ When:    Today    │
│ 🗑 Trash         │                                 │ Tags:    #personal│
│                  │                                 │ Project: Q1 plan  │
│ AREAS            │                                 │                   │
│  Work         8  │                                 │                   │
│ ...              │                                 │                   │
└──────────────────┴─────────────────────────────────┴───────────────────┘
 j/k nav · h/l focus · Enter edit · Space done · n new · t tags · s schedule · / search · ? help · q quit
```

## Install

```sh
make install            # /usr/local/bin/things-tui
# or
cargo install --path .
```

## First run

```sh
things-tui              # walks you through endpoint selection (local / remote / auto)
```

The TUI re-uses the things-api server's config directory:

```
~/Library/Application Support/things-api/
  auth_token        ← read by the TUI to authenticate API calls
  account.json      ← read by the TUI for the remote URL
  tui.json          ← written by the TUI to remember your endpoint choice
```

To reset the saved config:

```sh
things-tui --reconfigure
```

## Modes

| Mode    | What happens                                                              |
|---------|---------------------------------------------------------------------------|
| local   | Always talk to `http://127.0.0.1:3333` (the local server).               |
| remote  | Always talk to the URL in `account.json` (e.g. `<you>.<root-domain>`).   |
| auto    | Try local first; if unreachable, fall back to remote. **Recommended.**   |

Per-invocation overrides:

```sh
things-tui --endpoint local
things-tui --endpoint remote
things-tui --url http://192.168.1.10:3333
things-tui --token thingsapi_<paste>
```

## Key bindings

### Browser

| Key             | Action                                       |
|-----------------|----------------------------------------------|
| `j` / `k`       | Move down / up                               |
| `h` / `l`       | Focus left / right pane                      |
| `Tab` / `S-Tab` | Cycle pane focus                             |
| `PgUp` / `PgDn` | Jump 10 rows                                 |
| `g` / `G`       | Top / bottom                                 |
| `r`             | Refresh from server                          |
| `Enter`         | Activate sidebar item / edit task            |
| `Space`         | Toggle complete                              |
| `x`             | Cancel task                                  |
| `n`             | New task (uses the current sidebar context)  |
| `N`             | New project                                  |
| `d`             | Move task to Trash                           |
| `D`             | Empty Trash (only from the Trash list)       |
| `s`             | Schedule (when)                              |
| `t`             | Edit tags                                    |
| `m`             | Move to list / project / area                |
| `/`             | Filter visible list (client-side)            |
| `Ctrl-C`        | Open the Things 3 Quick Entry popup          |
| `?`             | Help overlay                                 |
| `q`             | Quit                                         |

### Modals

| Key       | Action            |
|-----------|-------------------|
| `Tab`     | Next form field   |
| `Ctrl-S`  | Save form         |
| `Esc`     | Close modal       |

### Schedule modal

Date choices auto-format to AppleScript-friendly strings ("April 14, 2026"):

```
  Today
  Tomorrow
  This Weekend (Saturday)
  Next Week (Monday)
  Anytime
  Someday
  Specific date…
  Clear schedule
```

## Build

```sh
make             # debug
make release     # release
make universal   # fat Intel + Apple Silicon binary into dist/
```

Requires Rust 1.75+ and (for `universal`) both Darwin Rust targets:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

## Architecture

- **`api/`** — thin reqwest wrapper, one method per HTTP endpoint.
- **`config.rs`** — `tui.json` schema + probing logic for endpoint selection.
- **`app.rs`** — `App` state, `Message` enum, the tokio event loop. All network
  calls are spawned tasks that send messages back through a channel.
- **`ui/`** — pure rendering. Three panes (`sidebar`, `task_list`, `detail`),
  plus modal overlays under `ui/modals/`.
- **`keys.rs`** — central keymap so binding changes are one-liners.

Mutations are optimistic where it's safe (toggling complete), and reconciled
after the server responds. AppleScript is slow enough (50–200 ms per op) that
this matters for keyboard feel.

## Non-goals (v1)

- Mouse drag-and-drop
- Persistent offline cache
- Multi-account switching
- Markdown rendering in the notes pane
- Windows / Linux (no Things 3 there; remote mode could work but isn't tested)
