use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountActivityDto, AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto,
    TransactionDetailDto, TransactionFilter, TransactionListItemDto, ValidatorDto,
};
use crate::explorer::referrer::normalize_hex_address;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

fn opt_normalized_address(value: Option<String>) -> Option<String> {
    value.and_then(|v| normalize_hex_address(&v))
}

const BLOCK_COLUMNS: &str =
    "height, hash, parent_hash, tx_count, producer, reward_recipient, block_reward, timestamp, total_fees";

const TX_COLUMNS: &str = "hash, block_height, from_address, to_address, amount, fee, status, \
     function_call_type, is_ride_related, timestamp, nonce, tx_index, referrer, request_referrer, \
     offer_referrer, request_referrer_fee, offer_referrer_fee";

#[derive(FromRow)]
struct BlockRow {
    height: i64,
    hash: String,
    parent_hash: String,
    tx_count: i32,
    producer: String,
    reward_recipient: String,
    block_reward: i64,
    timestamp: DateTime<Utc>,
    total_fees: i64,
}

#[derive(FromRow)]
struct TxRow {
    hash: String,
    block_height: i64,
    from_address: String,
    to_address: String,
    amount: i64,
    fee: i64,
    status: String,
    function_call_type: String,
    is_ride_related: bool,
    timestamp: DateTime<Utc>,
    nonce: i64,
    tx_index: i32,
    referrer: Option<String>,
    request_referrer: Option<String>,
    offer_referrer: Option<String>,
    request_referrer_fee: i64,
    offer_referrer_fee: i64,
}

#[derive(FromRow)]
struct AccountRow {
    address: String,
    balance: i64,
    nonce: i64,
    tx_count: i64,
    activity_count: i64,
    is_contract: bool,
}

#[derive(FromRow)]
struct ActivityRow {
    address: String,
    kind: String,
    label: String,
    delta: i64,
    direction: String,
    amount: i64,
    tx_hash: Option<String>,
    block_height: i64,
    tx_index: Option<i32>,
    function_call_type: Option<String>,
    counterparty: Option<String>,
    timestamp: DateTime<Utc>,
}

#[derive(FromRow)]
struct ValidatorRow {
    address: String,
    is_active: bool,
    blocks_produced: i64,
    peer_id: String,
}

impl From<BlockRow> for BlockListItemDto {
    fn from(r: BlockRow) -> Self {
        Self {
            height: r.height as u64,
            hash: r.hash,
            tx_count: r.tx_count as u32,
            producer: r.producer,
            reward_recipient: r.reward_recipient,
            block_reward: r.block_reward as u64,
            timestamp: r.timestamp,
        }
    }
}

impl From<BlockRow> for BlockDetailDto {
    fn from(r: BlockRow) -> Self {
        Self {
            height: r.height as u64,
            hash: r.hash,
            parent_hash: r.parent_hash,
            tx_count: r.tx_count as u32,
            producer: r.producer,
            reward_recipient: r.reward_recipient,
            block_reward: r.block_reward as u64,
            timestamp: r.timestamp,
            total_fees: r.total_fees as u64,
        }
    }
}

impl From<TxRow> for TransactionListItemDto {
    fn from(r: TxRow) -> Self {
        Self {
            hash: r.hash,
            block_height: r.block_height as u64,
            from: r.from_address,
            to: r.to_address,
            amount: r.amount as u64,
            fee: r.fee as u64,
            status: r.status,
            function_call_type: r.function_call_type,
            is_ride_related: r.is_ride_related,
            timestamp: r.timestamp,
            referrer: opt_normalized_address(r.referrer),
            request_referrer_fee: r.request_referrer_fee as u64,
            offer_referrer_fee: r.offer_referrer_fee as u64,
        }
    }
}

impl From<TxRow> for TransactionDetailDto {
    fn from(r: TxRow) -> Self {
        Self {
            hash: r.hash,
            block_height: r.block_height as u64,
            from: r.from_address,
            to: r.to_address,
            amount: r.amount as u64,
            fee: r.fee as u64,
            status: r.status,
            function_call_type: r.function_call_type,
            is_ride_related: r.is_ride_related,
            timestamp: r.timestamp,
            nonce: r.nonce as u64,
            tx_index: r.tx_index as u32,
            referrer: opt_normalized_address(r.referrer),
            request_referrer: opt_normalized_address(r.request_referrer),
            offer_referrer: opt_normalized_address(r.offer_referrer),
            request_referrer_fee: r.request_referrer_fee as u64,
            offer_referrer_fee: r.offer_referrer_fee as u64,
        }
    }
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_blocks(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BlockListItemDto>, ExplorerError> {
        let rows = sqlx::query_as::<_, BlockRow>(&format!(
            "SELECT {BLOCK_COLUMNS} FROM blocks ORDER BY height DESC LIMIT $1 OFFSET $2"
        ))
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// `id` is a height when it parses as one, otherwise a block hash.
    pub async fn get_block(&self, id: &str) -> Result<BlockDetailDto, ExplorerError> {
        let row = match id.parse::<i64>() {
            Ok(height) => {
                sqlx::query_as::<_, BlockRow>(&format!(
                    "SELECT {BLOCK_COLUMNS} FROM blocks WHERE height = $1"
                ))
                .bind(height)
                .fetch_optional(&self.pool)
                .await?
            }
            Err(_) => {
                sqlx::query_as::<_, BlockRow>(&format!(
                    "SELECT {BLOCK_COLUMNS} FROM blocks WHERE hash = $1"
                ))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?
            }
        };
        row.map(Into::into)
            .ok_or_else(|| ExplorerError::NotFound(format!("block {}", id)))
    }

    pub async fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionListItemDto>, ExplorerError> {
        let mut query = QueryBuilder::<Postgres>::new(format!(
            "SELECT {TX_COLUMNS} FROM transactions WHERE TRUE"
        ));
        // Transaction rows keep the address the node reported, which may or may not carry the
        // `0x` prefix, so match both spellings of the one the reader gave.
        if let Some(address) = filter.address {
            let bare = address
                .trim()
                .trim_start_matches("0x")
                .trim_start_matches("0X")
                .to_lowercase();
            let prefixed = format!("0x{}", bare);
            query
                .push(" AND (LOWER(from_address) IN (")
                .push_bind(prefixed.clone())
                .push(", ")
                .push_bind(bare.clone())
                .push(") OR LOWER(to_address) IN (")
                .push_bind(prefixed)
                .push(", ")
                .push_bind(bare)
                .push("))");
        }
        if let Some(status) = filter.status {
            query.push(" AND status = ").push_bind(status);
        }
        if let Some(height) = filter.block {
            query.push(" AND block_height = ").push_bind(height as i64);
        }
        if let Some(tx_type) = filter.tx_type {
            query.push(" AND function_call_type = ").push_bind(tx_type);
        }
        query
            .push(" ORDER BY block_height DESC, tx_index ASC LIMIT ")
            .push_bind(limit as i64)
            .push(" OFFSET ")
            .push_bind(offset as i64);

        let rows = query
            .build_query_as::<TxRow>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionDetailDto, ExplorerError> {
        sqlx::query_as::<_, TxRow>(&format!(
            "SELECT {TX_COLUMNS} FROM transactions WHERE hash = $1"
        ))
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?
        .map(Into::into)
        .ok_or_else(|| ExplorerError::NotFound(format!("transaction {}", hash)))
    }

    pub async fn get_account(&self, address: &str) -> Result<AccountDto, ExplorerError> {
        if address.trim().is_empty() || address.eq_ignore_ascii_case("unknown") {
            return Err(ExplorerError::NotFound(format!("account {}", address)));
        }

        let canonical = normalize_hex_address(address).unwrap_or_else(|| address.to_string());
        let legacy = canonical
            .strip_prefix("0x")
            .or_else(|| canonical.strip_prefix("0X"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| canonical.clone());

        let row = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT address, balance, nonce, tx_count, activity_count, is_contract
            FROM accounts
            WHERE LOWER(address) = LOWER($1)
               OR (LOWER($2) <> LOWER($1) AND LOWER(address) = LOWER($2))
            ORDER BY CASE WHEN LOWER(address) = LOWER($1) THEN 0 ELSE 1 END
            LIMIT 1
            "#,
        )
        .bind(&canonical)
        .bind(&legacy)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            return Ok(AccountDto {
                address: normalize_hex_address(&row.address).unwrap_or(row.address),
                balance: row.balance as u64,
                nonce: row.nonce as u64,
                tx_count: row.tx_count as u64,
                activity_count: row.activity_count as u64,
                is_contract: row.is_contract,
            });
        }

        // Fallback for addresses that exist only in tx history but not yet materialized in accounts table.
        let tx_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM transactions
            WHERE LOWER(from_address) = LOWER($1) OR LOWER(to_address) = LOWER($1)
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await?;

        let activity_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM account_activity
            WHERE LOWER(address) = LOWER($1)
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await?;

        if tx_count > 0 || activity_count > 0 {
            return Ok(AccountDto {
                address: canonical.clone(),
                balance: 0,
                nonce: 0,
                tx_count: tx_count as u64,
                activity_count: activity_count as u64,
                is_contract: false,
            });
        }

        // Producer/validator addresses may appear in blocks even without account state or tx history.
        let validator_address = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT address
            FROM validators
            WHERE LOWER(address) = LOWER($1) AND TRIM(address) <> '' AND LOWER(address) <> 'unknown'
            LIMIT 1
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await?;

        let produced_blocks = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM blocks
            WHERE (LOWER(producer) = LOWER($1) OR LOWER(reward_recipient) = LOWER($1))
              AND LOWER($1) <> 'unknown'
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await?;

        if validator_address.is_some() || produced_blocks > 0 {
            return Ok(AccountDto {
                address: validator_address
                    .and_then(|a| normalize_hex_address(&a))
                    .unwrap_or(canonical),
                balance: 0,
                nonce: 0,
                tx_count: tx_count as u64,
                activity_count: activity_count as u64,
                is_contract: false,
            });
        }

        Err(ExplorerError::NotFound(format!("account {}", address)))
    }

    pub async fn get_account_activity(
        &self,
        address: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AccountActivityDto>, ExplorerError> {
        if address.trim().is_empty() || address.eq_ignore_ascii_case("unknown") {
            return Err(ExplorerError::NotFound(format!("account {}", address)));
        }

        let canonical = normalize_hex_address(address).unwrap_or_else(|| address.to_string());

        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT address, kind, label, delta, direction, amount, tx_hash, block_height,
                   tx_index, function_call_type, counterparty, timestamp
            FROM account_activity
            WHERE LOWER(address) = LOWER($1)
            ORDER BY block_height DESC, COALESCE(tx_index, -1) DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&canonical)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AccountActivityDto {
                address: normalize_hex_address(&r.address).unwrap_or(r.address),
                kind: r.kind,
                label: r.label,
                delta: r.delta,
                direction: r.direction,
                amount: r.amount as u64,
                tx_hash: r.tx_hash,
                block_height: r.block_height as u64,
                tx_index: r.tx_index.map(|v| v as u32),
                function_call_type: r.function_call_type,
                counterparty: opt_normalized_address(r.counterparty),
                timestamp: r.timestamp,
            })
            .collect())
    }

    pub async fn get_validators(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ValidatorDto>, ExplorerError> {
        let rows = sqlx::query_as::<_, ValidatorRow>(
            r#"
            SELECT address, is_active, blocks_produced, peer_id
            FROM validators
            WHERE TRIM(address) <> '' AND LOWER(address) <> 'unknown'
            ORDER BY blocks_produced DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ValidatorDto {
                address: r.address,
                is_active: r.is_active,
                blocks_produced: r.blocks_produced as u64,
                peer_id: r.peer_id,
            })
            .collect())
    }

    pub async fn get_stats(&self) -> Result<StatsDto, ExplorerError> {
        let latest_height = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(height) FROM blocks")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        let total_transactions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.pool)
            .await?;

        let active_validators =
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM validators WHERE is_active = TRUE AND TRIM(address) <> '' AND LOWER(address) <> 'unknown'",
            )
                .fetch_one(&self.pool)
                .await?;

        // Both rates come from the newest 100 blocks: the span between the first and last of
        // them, the number of gaps, and the transactions they carry.
        let (span_seconds, gaps, recent_txs) =
            sqlx::query_as::<_, (Option<f64>, i64, Option<i64>)>(
                r#"
            SELECT EXTRACT(EPOCH FROM MAX(timestamp) - MIN(timestamp))::float8,
                   GREATEST(COUNT(*) - 1, 0),
                   SUM(tx_count)::bigint
            FROM (SELECT timestamp, tx_count FROM blocks ORDER BY height DESC LIMIT 100) recent
            "#,
            )
            .fetch_one(&self.pool)
            .await?;
        let span_seconds = span_seconds.unwrap_or(0.0);
        let (avg_block_time_seconds, tx_per_second) = if gaps > 0 && span_seconds > 0.0 {
            (
                span_seconds / gaps as f64,
                recent_txs.unwrap_or(0) as f64 / span_seconds,
            )
        } else {
            (0.0, 0.0)
        };

        Ok(StatsDto {
            latest_height: latest_height as u64,
            tx_per_second,
            total_transactions: total_transactions as u64,
            active_validators: active_validators as usize,
            avg_block_time_seconds,
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResultDto>, ExplorerError> {
        let q = query.trim().to_string();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();

        if let Ok(height) = q.parse::<i64>() {
            let found =
                sqlx::query_scalar::<_, i64>("SELECT height FROM blocks WHERE height = $1 LIMIT 1")
                    .bind(height)
                    .fetch_optional(&self.pool)
                    .await?;
            if let Some(height) = found {
                items.push(SearchResultDto {
                    kind: "block".to_string(),
                    identifier: height.to_string(),
                    summary: "Block height match".to_string(),
                });
            }
            return Ok(items);
        }

        // The indexer stores hashes exactly as the node reports them: transaction hashes with a
        // `0x` prefix, block hashes without. Addresses are stored canonical (`0x` + lowercase).
        // Accept either spelling from the reader and match both forms.
        let body = q
            .strip_prefix("0x")
            .or_else(|| q.strip_prefix("0X"))
            .unwrap_or(&q)
            .to_lowercase();
        if body.is_empty() || !body.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(items);
        }
        let prefixed = format!("0x{}", body);

        if let Some(hash) = sqlx::query_scalar::<_, String>(
            "SELECT hash FROM transactions WHERE LOWER(hash) IN ($1, $2) LIMIT 1",
        )
        .bind(&body)
        .bind(&prefixed)
        .fetch_optional(&self.pool)
        .await?
        {
            items.push(SearchResultDto {
                kind: "transaction".to_string(),
                identifier: hash,
                summary: "Transaction hash match".to_string(),
            });
        }

        if let Some(hash) = sqlx::query_scalar::<_, String>(
            "SELECT hash FROM blocks WHERE LOWER(hash) IN ($1, $2) LIMIT 1",
        )
        .bind(&body)
        .bind(&prefixed)
        .fetch_optional(&self.pool)
        .await?
        {
            items.push(SearchResultDto {
                kind: "block".to_string(),
                identifier: hash,
                summary: "Block hash match".to_string(),
            });
        }

        if body.len() == 40 {
            let address = sqlx::query_scalar::<_, String>(
                "SELECT address FROM accounts WHERE LOWER(address) IN ($1, $2) \
                 ORDER BY CASE WHEN LOWER(address) = $1 THEN 0 ELSE 1 END LIMIT 1",
            )
            .bind(&prefixed)
            .bind(&body)
            .fetch_optional(&self.pool)
            .await?;
            items.push(SearchResultDto {
                kind: "account".to_string(),
                summary: if address.is_some() {
                    "Account address match".to_string()
                } else {
                    "Address with no indexed activity".to_string()
                },
                identifier: address.unwrap_or(prefixed),
            });
        }

        Ok(items)
    }
}
