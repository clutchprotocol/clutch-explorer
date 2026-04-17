# Clutch Explorer

`clutch-explorer` is a blockchain explorer stack for the Clutch network:

- Rust backend (`backend`) exposing explorer APIs
- React frontend (`frontend`) for blocks, transactions, accounts, validators, search, and stats
- Dockerized setup aligned with other Clutch projects

## Local Backend

```bash
cd backend
cargo run -- --env default
```

Default API: `http://localhost:8088`

## Local Frontend

```bash
cd frontend
npm install
npm run dev
```

Default UI: `http://localhost:5173`

Configure backend URL with `VITE_EXPLORER_API_URL`.

## Docker Compose

```bash
docker compose up -d --build
```

Services:

- explorer backend: `http://localhost:8088`
- explorer frontend: `http://localhost:5174`

## API Surface (Phase 1)

- `GET /health`
- `GET /ready`
- `GET /api/v1/blocks?limit=20`
- `GET /api/v1/blocks/{heightOrHash}`
- `GET /api/v1/transactions?limit=20`
- `GET /api/v1/transactions/{hash}`
- `GET /api/v1/accounts/{address}`
- `GET /api/v1/validators`
- `GET /api/v1/search?q=...`
- `GET /api/v1/stats`

## Logging

Backend supports Seq with:

- `APP_SEQ_URL`
- `APP_SEQ_API_KEY`

Logs are sent to `{APP_SEQ_URL}/ingest/clef` in CLEF format.
