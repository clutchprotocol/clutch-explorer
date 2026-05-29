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

Full reference: https://docs.clutchprotocol.io/clutch-explorer/api-reference

## Indexer

Polls clutch-node blocks (default ~4s interval) and stores data in PostgreSQL.

## Docker images

- `ghcr.io/clutchprotocol/clutch-explorer-backend:latest`
- `ghcr.io/clutchprotocol/clutch-explorer-frontend:latest`
