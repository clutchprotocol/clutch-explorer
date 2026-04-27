CREATE TABLE IF NOT EXISTS blocks (
    height BIGINT PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    parent_hash TEXT NOT NULL,
    tx_count INTEGER NOT NULL DEFAULT 0,
    producer TEXT NOT NULL,
    reward_recipient TEXT NOT NULL DEFAULT '',
    block_reward BIGINT NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL,
    total_fees BIGINT NOT NULL DEFAULT 0
);

ALTER TABLE blocks ADD COLUMN IF NOT EXISTS reward_recipient TEXT NOT NULL DEFAULT '';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS block_reward BIGINT NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp_desc ON blocks(timestamp DESC);

CREATE TABLE IF NOT EXISTS transactions (
    hash TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL REFERENCES blocks(height) ON DELETE CASCADE,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0,
    fee BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    nonce BIGINT NOT NULL DEFAULT 0,
    tx_index INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height DESC, tx_index ASC);
CREATE INDEX IF NOT EXISTS idx_transactions_from_address ON transactions(from_address);
CREATE INDEX IF NOT EXISTS idx_transactions_to_address ON transactions(to_address);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp_desc ON transactions(timestamp DESC);

CREATE TABLE IF NOT EXISTS accounts (
    address TEXT PRIMARY KEY,
    balance BIGINT NOT NULL DEFAULT 0,
    nonce BIGINT NOT NULL DEFAULT 0,
    tx_count BIGINT NOT NULL DEFAULT 0,
    is_contract BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_accounts_updated_at_desc ON accounts(updated_at DESC);

CREATE TABLE IF NOT EXISTS validators (
    address TEXT PRIMARY KEY,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    blocks_produced BIGINT NOT NULL DEFAULT 0,
    peer_id TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_validators_blocks_produced_desc ON validators(blocks_produced DESC);

CREATE TABLE IF NOT EXISTS indexer_cursor (
    id INTEGER PRIMARY KEY,
    last_indexed_height BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
