# Clutch Explorer

Block explorer for the Clutch Protocol chain — Rust backend indexer + React frontend.

**Documentation:** https://docs.clutchprotocol.io/clutch-explorer/overview

## Quick start (clutch-deploy)

```bash
docker compose up -d
```

| Service | URL |
|---------|-----|
| Explorer UI | http://localhost:5174 |
| Explorer API | http://localhost:8088 |

## Local development

**Backend:**

```bash
cd backend
cargo run -- --env default
```

**Frontend:**

```bash
cd frontend
npm install
VITE_EXPLORER_API_URL=http://localhost:8088 npm run dev
```

Default UI port: `5174` (when run via compose) or `5173` (standalone Vite default).

## REST API

| Route | Description |
|-------|-------------|
| `GET /health`, `/ready` | Health checks |
| `GET /api/v1/blocks` | Block list |
| `GET /api/v1/blocks/:id` | Block detail |
| `GET /api/v1/transactions` | Transaction list |
| `GET /api/v1/transactions/:hash` | Transaction detail |
| `GET /api/v1/accounts/:address` | Account info |
| `GET /api/v1/accounts/:address/activity` | Balance activity |
| `GET /api/v1/validators` | Validator set |
| `GET /api/v1/search?q=` | Search |
| `GET /api/v1/stats` | Network stats |
| `GET /api/v1/reserve` | The reserve position behind CLT |

Full reference: https://docs.clutchprotocol.io/clutch-explorer/api-reference

### `/api/v1/reserve`

CLT claims to be fully reserved, and that claim is checkable only if both sides of it are visible.
This republishes the treasury's last reconciliation: what the chain says exists, what the ledger
says is owed, and what is actually held against it.

Every figure comes from a **single run**, with that run's timestamp. Live chain supply is
deliberately not mixed in: a mint moves supply at once and the reserve figure only at the next run,
so the pair would disagree routinely and ordinary lag would read as a shortfall.

It never serves a stale cache on failure — an unreachable treasury is reported as unreachable, not
as the last known figures, because a page still showing "matched" reports a verification that is not
happening. `configured: false` means this deployment has no treasury behind it, which is a real
configuration rather than an error, and the frontend renders nothing at all in that case.

Set `APP_TREASURY_PUBLIC_RECONCILIATION_URL` to switch it on.

## Indexer

Polls clutch-node blocks (default ~4s interval) and stores data in PostgreSQL.

## Docker images

- `ghcr.io/clutchprotocol/clutch-explorer-backend:latest`
- `ghcr.io/clutchprotocol/clutch-explorer-frontend:latest`
