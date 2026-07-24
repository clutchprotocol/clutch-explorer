# clutch-explorer

Block explorer for Clutch Protocol. Two independent halves in one repo — see the workspace-level
`../CLAUDE.md` for how this fits into the wider stack; this file covers repo internals only.

```
backend/    Rust (Axum + sqlx/Postgres). ONE crate, TWO binaries:
            - clutch-explorer-backend (src/main.rs)  → REST API on :8088
            - indexer (src/bin/indexer.rs)           → polls clutch-node, writes Postgres
frontend/   React 19 + Vite + react-router 7 (no state library, plain fetch)
docker-compose.yml   Postgres + indexer + backend + frontend (expects external `clutch-network`)
```

## Backend (`backend/`)

All logic lives in `src/explorer/`:

| Path | Role |
|------|------|
| `run.rs` | `run_api` / `run_indexer` entrypoints (tracing, pool, migrations, graceful shutdown) |
| `app.rs` | Router + CORS — **add new routes here** |
| `handlers.rs` | Axum handlers (paging defaults: `limit=20`, capped 100; error → HTTP mapping) |
| `state.rs` | `AppState` / `ExplorerService`, picks repository by `data_source` config |
| `repository.rs` | `ExplorerRepository` trait |
| `postgres_repository.rs` | Normal read path (Postgres) |
| `node_repository.rs` + `node_client.rs` | Alternate `data_source = "node"` read path (queries node directly, no DB) |
| `indexer.rs` | `IndexerService` — the poll → fetch → upsert loop |
| `ingestion.rs` | `NodeIngestionSource` trait + `NodeHttpIngestionSource` (talks to the node) |
| `activity.rs` | Parses `balance_effects` from node payloads into `account_activity` rows |
| `referrer.rs` | Referrer-fee enrichment (ceiling division matches clutch-node), `normalize_hex_address` |
| `models.rs` | API DTOs; `error.rs` maps to 404/400/502/503 |
| `configuration.rs` | Config struct; `db.rs` migrations + cleanup; `seq.rs`/`tracing.rs` Seq logging |

### Indexing pipeline

1. Head discovery: scrape node **Prometheus metrics** (`node_metrics_url`) for `latest_block_index` / `latest_block{block_hash=...}` — not RPC.
2. For each height behind the cursor: `get_block_by_index` over **WebSocket JSON-RPC** (`node_ws_url`), plus `get_account_balance` / `get_next_nonce` per touched address.
3. Upsert `blocks` → `validators` (producer counts) → `transactions` (with referrer enrichment) → `account_activity` (balance effects) → `accounts` snapshots. Everything is `ON CONFLICT ... DO UPDATE`, so re-indexing a height is safe.
4. Cursor persisted in `indexer_cursor` (single row, id=1). On error the height is retried next poll; loop sleeps `indexer_poll_interval_ms` (default 4000).
5. **Reorg detection** (`IndexerService::reconcile_head` / `index_height`'s parent-hash check): every poll compares the node's reported head against what's indexed; a node behind the cursor (e.g. restarted with `developer_mode=true` and wiped) or a tip whose hash no longer matches triggers `find_fork_point` (walks backward comparing stored vs. live hashes) then `unwind_to` (deletes `blocks`/`account_activity` above the fork point — `transactions` cascades via FK — and rewinds the cursor). Forward indexing then naturally re-upserts correct data.

### REST endpoints (all GET; see `app.rs`)

`/health`, `/ready`, `/api/v1/blocks`, `/api/v1/blocks/:id` (height or hash),
`/api/v1/transactions` (`?address=&status=`), `/api/v1/transactions/:hash`,
`/api/v1/accounts/:address`, `/api/v1/accounts/:address/activity`,
`/api/v1/validators`, `/api/v1/search?q=`, `/api/v1/stats`.

New endpoint = handler in `handlers.rs` + route in `app.rs` + method on `ExplorerService`
(`state.rs`) + trait method in `repository.rs` + impls in `postgres_repository.rs` and
`node_repository.rs` + DTO in `models.rs`. Frontend client: `frontend/src/api/client.ts` + `types.ts`.

### Config

- `config/{env}.toml` selected by `--env` (both binaries take it; default `default` → `config/default.toml`). Env vars with `APP_` prefix override (e.g. `APP_DATABASE_URL`); `.env` is loaded via dotenv.
- `data_source`: `"postgres"` (normal) or `"node"` (DB-less passthrough).
- `developer_mode` / `cleanup_on_start`: truncate all tables on shutdown / startup.
- `ride_*_referrer_fee_percent` must match clutch-node config or RidePay fee display drifts.

### DB schema / migrations

- `migrations/*.sql` — tables: `blocks`, `transactions`, `accounts`, `validators`, `account_activity`, `indexer_cursor`.
- **Not sqlx-migrate.** `db.rs::run_migrations` runs each file via `include_str!` on every startup, splitting on `;`. Consequences: new migration files must be manually wired into `db.rs`; every statement must be idempotent (`IF NOT EXISTS` / `ADD COLUMN IF NOT EXISTS`); no `;` inside statement bodies (no PL/pgSQL functions).
- All queries use runtime `sqlx::query` (no `query!` macros) — **no sqlx offline mode / `.sqlx` dir / DATABASE_URL needed to compile**.

## Frontend (`frontend/`)

- `src/main.tsx` → `App.tsx` routes (all under `components/Layout.tsx`):
  `/` HomePage, `/blocks` + `/blocks/:id`, `/txs` + `/txs/:hash`, `/address/:address` (AddressPage), `/validators`. Pages in `src/pages/`.
- API client: `src/api/client.ts` (`explorerApi`), types in `src/api/types.ts`, formatting helpers in `src/utils/format.ts`, single stylesheet `src/styles.css`.
- Only env var: `VITE_EXPLORER_API_URL` (base URL; `/api` is auto-appended). Fallback: `http://localhost:8088` on localhost, else relative `/api`.

## Commands

```powershell
# Backend API (from backend/)
cargo run -- --env default            # needs Postgres + a running clutch-node
cargo run --bin indexer -- --env default   # the indexer is a separate process
cargo test

# Frontend (from frontend/)
npm install
$env:VITE_EXPLORER_API_URL = "http://localhost:8088"; npm run dev   # Vite dev server
npm run build

# Full explorer stack (from repo root; requires `docker network create clutch-network` and a node)
docker compose up -d --build
```

Local Postgres without the full stack:
`docker run -d --name explorer-pg -p 5432:5432 -e POSTGRES_DB=clutch_explorer -e POSTGRES_PASSWORD=postgres postgres:16-alpine`
then `$env:APP_DATABASE_URL = "postgres://postgres:postgres@localhost:5432/clutch_explorer"`.

## Gotchas

- `config/default.toml` mostly uses Docker hostnames (`node1`, `explorer-postgres`); only `node_ws_url` defaults to `ws://localhost:8081/ws` for host-local indexer runs. For a fully host-local run, override the rest via `APP_*` env vars (or a new `config/local.toml` + `--env local`) — point at `localhost:8081` / `localhost:5432`. Docker paths (repo compose and clutch-deploy) set `APP_NODE_WS_URL=ws://node1:8081/ws`.
- `node_ws_url` goes straight to tokio-tungstenite: it must be a `ws://` URL, never `http://`.
- Ports: 8088 = API, 5174 = frontend **only via compose** (nginx 80→5174). Standalone `npm run dev` uses Vite's default 5173, which collides with clutch-hub-demo-app — pass `--port` if running both.
- CORS: `allowed_origins` config, `*` or comma-separated list; GET only (`app.rs`). New non-GET routes need the CORS layer updated too.
- Migrations rerun on every boot of either binary — keep them idempotent.
- Both binaries log to Seq (`seq_url`); it tolerates Seq being absent.
- Frontend Dockerfile bakes `VITE_EXPLORER_API_URL` at **build** time (compose arg) — changing it requires an image rebuild.
