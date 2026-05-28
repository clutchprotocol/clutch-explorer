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

## Referrer rewards (RidePay)

Referrer balances are credited on-chain when a **RidePay** transaction is applied. After upgrading clutch-node or explorer:

1. Rebuild and restart **all validator nodes** (referrer fee % and account canonicalization live in node config).
2. Restart the **explorer** backend and indexer so account snapshots use canonical `0x` addresses.
3. Run a **new** ride (RideRequest → Offer → Acceptance → RidePay). Past RidePay txs are not re-applied.
4. Confirm balance at `/address/0x…` (e.g. `http://localhost:5174/address/0x0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f`).

Node config must set `ride_request_referrer_fee_percent` and `ride_offer_referrer_fee_percent` (default **2** in deploy configs). Small fares use **ceiling** rounding (e.g. 2% of 3 CLT → 1 CLT per referrer side).

## Logging

Backend supports Seq with:

- `APP_SEQ_URL`
- `APP_SEQ_API_KEY`

Logs are sent to `{APP_SEQ_URL}/ingest/clef` in CLEF format.
