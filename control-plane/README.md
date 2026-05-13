# things-api control plane

A tiny HTTP service that brokers Cloudflare Tunnels for `things-api` users. On signup it:

1. Validates the requested username (3–32 chars, `a-z 0-9 -`, not a reserved name).
2. Creates a Cloudflare tunnel via the CF API → gets a tunnel UUID and a connector token.
3. Configures the tunnel ingress to forward `<username>.<ROOT_DOMAIN>` → `http://localhost:3333`.
4. Creates a CNAME `<username>.<ROOT_DOMAIN> → <tunnel-uuid>.cfargotunnel.com`.
5. Persists the row in SQLite and returns `{url, tunnel_token}` to the client.

Personal task data **never** touches this server. After signup the client runs `cloudflared` locally with the token; Cloudflare routes traffic directly to the user's Mac.

## Prerequisites

- A domain you own, added to Cloudflare. e.g. `anywhere-api.io`.
- A Cloudflare API token with the following permissions:
  - `Account → Cloudflare Tunnel → Edit`
  - `Zone → DNS → Edit` (scoped to that domain)
- Your CF Account ID and the Zone ID for the domain.

## Environment variables

| Var | Required | Notes |
|---|---|---|
| `ROOT_DOMAIN` | yes | e.g. `anywhere-api.io` |
| `CF_API_TOKEN` | yes | Cloudflare API token |
| `CF_ACCOUNT_ID` | yes | Cloudflare account ID |
| `CF_ZONE_ID` | yes | Zone ID for `ROOT_DOMAIN` |
| `PORT` | no | Default `8080` |
| `DATABASE_URL` | no | Default `sqlite://control-plane.db?mode=rwc`. On Fly we point this at the mounted volume. |

## Local run

```sh
cargo run -- \
  --root-domain anywhere-api.io \
  --cf-api-token <TOKEN> \
  --cf-account-id <ACCOUNT> \
  --cf-zone-id <ZONE>
```

Then:

```sh
curl -X POST http://localhost:8080/signup \
  -H 'content-type: application/json' \
  -d '{"username":"alice","email":"alice@example.com"}'
# → {"username":"alice","url":"https://alice.anywhere-api.io","tunnel_token":"eyJh..."}
```

## Deploy to Fly.io

```sh
fly apps create things-api-control-plane
fly volumes create control_plane_data --size 1
fly secrets set \
  ROOT_DOMAIN=anywhere-api.io \
  CF_API_TOKEN=... \
  CF_ACCOUNT_ID=... \
  CF_ZONE_ID=...
fly deploy
```

## Endpoints

- `GET /health` — liveness probe.
- `POST /signup` — body `{"username": "...", "email": "..."}`. Returns 201 with the tunnel token, or 400/409 on validation/uniqueness errors.

## Operational notes

- **Username squatting**: the reserved list in `src/routes.rs` blocks the obvious ones. Edit it before launch.
- **No recovery yet**: a user who loses their local tunnel token can't currently recover their subdomain. Future work: email-verified rotation.
- **Cleanup on failure**: every step that creates Cloudflare state has a rollback path on the failure case after it.
- **Database**: one SQLite table `accounts`. Trivial to migrate to Postgres later if needed.
