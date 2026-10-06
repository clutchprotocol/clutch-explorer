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
| `app.rs` | `AppState` (repository + reserve client), router + CORS — **add new routes here** |
| `handlers.rs` | Axum handlers (paging defaults: `limit=20`, capped 100) |
| `postgres_repository.rs` | `PostgresRepository`, the API's whole read path |
| `reserve.rs` | `ReserveClient`: the treasury's reconciliation, cached, behind `/api/v1/reserve` |
| `indexer.rs` | `IndexerService` — the poll → fetch → upsert loop |
| `ingestion.rs` | `NodeSource` (talks to the node: metrics for the head, one `get_block_by_index` per block) |
| `activity.rs` | Parses `balance_effects` from node payloads into `account_activity` rows |
| `referrer.rs` | Referrer-fee enrichment (floor division, matches clutch-node), `normalize_hex_address` |
| `models.rs` | API DTOs; `error.rs` is `ExplorerError`, which renders itself as 404/400/502/503 JSON |
| `configuration.rs` | Config struct; `db.rs` migrations + cleanup; `seq.rs`/`tracing.rs` Seq logging |

### Indexing pipeline

1. Head discovery: scrape node **Prometheus metrics** (`node_metrics_url`) for `latest_block_index` / `latest_block{block_hash=...}` — not RPC.
2. For each height behind the cursor: `get_block_by_index` over **WebSocket JSON-RPC** (`node_ws_url`), plus `get_account_balance` / `get_next_nonce` per touched address.
3. Upsert `blocks` → `validators` (producer counts) → `transactions` (with referrer enrichment) → `account_activity` (balance effects) → `accounts` snapshots. Everything is `ON CONFLICT ... DO UPDATE`, so re-indexing a height is safe.
4. Cursor persisted in `indexer_cursor` (single row, id=1). On error the height is retried next poll; loop sleeps `indexer_poll_interval_ms` (default 4000).
5. **Reorg detection** (`IndexerService::reconcile_head` / `index_height`'s parent-hash check): every poll compares the node's reported head against what's indexed; a node behind the cursor (e.g. restarted with `developer_mode=true` and wiped) or a tip whose hash no longer matches triggers `find_fork_point` (walks backward comparing stored vs. live hashes) then `unwind_to` (deletes `blocks`/`account_activity` above the fork point — `transactions` cascades via FK — and rewinds the cursor). Forward indexing then naturally re-upserts correct data.

### REST endpoints (all GET; see `app.rs`)

`/health`, `/ready`, `/api/v1/blocks`, `/api/v1/blocks/:id` (height or hash),
`/api/v1/transactions` (`?address=&status=&block=&type=`; the filters travel as one `TransactionFilter` in `models.rs`), `/api/v1/transactions/:hash`,
`/api/v1/accounts/:address`, `/api/v1/accounts/:address/activity`,
`/api/v1/validators`, `/api/v1/search?q=`, `/api/v1/stats`, `/api/v1/reserve`.

New endpoint = handler in `handlers.rs` + route in `app.rs` + method on `PostgresRepository` +
DTO in `models.rs`. Frontend client: `frontend/src/api/client.ts` + `types.ts`.

### Config

- `config/{env}.toml` selected by `--env` (both binaries take it; default `default` → `config/default.toml`). Env vars with `APP_` prefix override (e.g. `APP_DATABASE_URL`); `.env` is loaded via dotenv.
- There is one read path, Postgres. The old `data_source = "node"` mode (and its `strict_mode` and
  `clutch_node_api_url` keys) was removed: it called a REST API the node never had. Those keys are
  ignored if a config still carries them.
- `developer_mode` / `cleanup_on_start`: truncate all tables on shutdown / startup.
- `ride_*_referrer_fee_bps` must match clutch-node config or RidePay fee display drifts.

### DB schema / migrations

- `migrations/*.sql` — tables: `blocks`, `transactions`, `accounts`, `validators`, `account_activity`, `indexer_cursor`.
- **Not sqlx-migrate.** `db.rs::run_migrations` runs each file via `include_str!` on every startup, splitting on `;`. Consequences: new migration files must be added to the `MIGRATIONS` list in `db.rs`; every statement must be idempotent (`IF NOT EXISTS` / `ADD COLUMN IF NOT EXISTS`); no `;` inside statement bodies (no PL/pgSQL functions).
- All queries use runtime `sqlx::query` (no `query!` macros) — **no sqlx offline mode / `.sqlx` dir / DATABASE_URL needed to compile**.

## Frontend (`frontend/`)

- `src/main.tsx` → `App.tsx` routes (all under `components/Layout.tsx`):
  `/` HomePage, `/blocks` + `/blocks/:id`, `/txs` + `/txs/:hash`, `/address/:address` (AddressPage, `?tab=transactions`), `/validators`, `/search?q=`, and a not-found page for anything else. Pages in `src/pages/`.
- API client: `src/api/client.ts` (`explorerApi`; throws `ApiError` with the HTTP status, so pages tell 404 from an outage), types in `src/api/types.ts`, formatting helpers in `src/utils/format.ts`, single stylesheet `src/styles.css`.
- Shared pieces: `components/ui.tsx` (panels, links, copy buttons, badges, pagination), `components/tables.tsx` (block and transaction tables), `components/SearchBox.tsx`. Data loading goes through `hooks/useApi.ts` (optional polling; a failed poll keeps what is on screen); list pages keep their page in `?page=` via `hooks/usePage.ts`.
- **Amounts are micro-dollars**: render them with `<Amount>` / `formatCltPrecise` (1 USD = 1,000,000 CLT), never the raw integer. The flat fee is 1,000 units, so cents alone would show $0.00.
- **Search asks the API** (`/api/v1/search`) instead of guessing from the input's shape: a 64-hex value can be a transaction or a block hash, and the node writes transaction hashes with `0x` but block hashes without, so the API matches both spellings.
- Look: the same "road" palette as clutchprotocol.io and the docs (concrete / asphalt, sign green `#0a5c45`, lane yellow `#e8b923`, Barlow + IBM Plex Mono). Tokens live at the top of `styles.css`; dark mode follows the system and the header toggle stores an override in `localStorage`.
- Env vars: `VITE_EXPLORER_API_URL` (base URL; `/api` is auto-appended; fallback `http://localhost:8088` on localhost, else relative `/api`) and optional `VITE_NETWORK_LABEL` for the header chip (otherwise read from the hostname: `*-stage.*` → Testnet, `*.clutchprotocol.io` → Mainnet pilot, else Local).

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
