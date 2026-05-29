CREATE TABLE IF NOT EXISTS account_activity (
    id BIGSERIAL PRIMARY KEY,
    address TEXT NOT NULL,
    kind TEXT NOT NULL,
    delta BIGINT NOT NULL,
    direction TEXT NOT NULL,
    amount BIGINT NOT NULL,
    tx_hash TEXT,
    block_height BIGINT NOT NULL,
    tx_index INT,
    function_call_type TEXT,
    counterparty TEXT,
    label TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activity_address ON account_activity (LOWER(address), block_height DESC);
CREATE INDEX IF NOT EXISTS idx_activity_tx ON account_activity (tx_hash);

CREATE UNIQUE INDEX IF NOT EXISTS idx_activity_dedup ON account_activity (
    LOWER(address),
    COALESCE(tx_hash, ''),
    kind,
    block_height,
    delta,
    COALESCE(tx_index, -1)
);
