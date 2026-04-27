use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use crate::explorer::repository::{ExplorerRepository, RepoFuture};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

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
    timestamp: DateTime<Utc>,
    nonce: i64,
    tx_index: i32,
}

#[derive(FromRow)]
struct AccountRow {
    address: String,
    balance: i64,
    nonce: i64,
    tx_count: i64,
    is_contract: bool,
}

#[derive(FromRow)]
struct ValidatorRow {
    address: String,
    is_active: bool,
    blocks_produced: i64,
    peer_id: String,
}

impl ExplorerRepository for PostgresRepository {
    fn get_blocks(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<BlockListItemDto>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, BlockRow>(
                r#"
                SELECT height, hash, parent_hash, tx_count, producer, reward_recipient, block_reward, timestamp, total_fees
                FROM blocks
                ORDER BY height DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            Ok(rows
                .into_iter()
                .map(|r| BlockListItemDto {
                    height: r.height as u64,
                    hash: r.hash,
                    tx_count: r.tx_count as u32,
                    producer: r.producer,
                    reward_recipient: r.reward_recipient,
                    block_reward: r.block_reward as u64,
                    timestamp: r.timestamp,
                })
                .collect())
        })
    }

    fn get_block(&self, id: String) -> RepoFuture<'_, BlockDetailDto> {
        Box::pin(async move {
            let row = if let Ok(height) = id.parse::<i64>() {
                sqlx::query_as::<_, BlockRow>(
                    r#"
                    SELECT height, hash, parent_hash, tx_count, producer, reward_recipient, block_reward, timestamp, total_fees
                    FROM blocks
                    WHERE height = $1
                    "#,
                )
                .bind(height)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
            } else {
                sqlx::query_as::<_, BlockRow>(
                    r#"
                    SELECT height, hash, parent_hash, tx_count, producer, reward_recipient, block_reward, timestamp, total_fees
                    FROM blocks
                    WHERE hash = $1
                    "#,
                )
                .bind(&id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
            };

            let row = row.ok_or_else(|| ExplorerError::NotFound(format!("block {}", id)))?;
            Ok(BlockDetailDto {
                height: row.height as u64,
                hash: row.hash,
                parent_hash: row.parent_hash,
                tx_count: row.tx_count as u32,
                producer: row.producer,
                reward_recipient: row.reward_recipient,
                block_reward: row.block_reward as u64,
                timestamp: row.timestamp,
                total_fees: row.total_fees as u64,
            })
        })
    }

    fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<String>,
        status: Option<String>,
    ) -> RepoFuture<'_, Vec<TransactionListItemDto>> {
        Box::pin(async move {
            let mut sql = String::from(
                r#"
                SELECT hash, block_height, from_address, to_address, amount, fee, status, timestamp, nonce, tx_index
                FROM transactions
                "#,
            );

            let mut where_clauses = Vec::new();
            if address.is_some() {
                where_clauses.push("(LOWER(from_address) = LOWER($1) OR LOWER(to_address) = LOWER($1))");
            }
            if status.is_some() {
                where_clauses.push(if address.is_some() {
                    "status = $2"
                } else {
                    "status = $1"
                });
            }
            if !where_clauses.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&where_clauses.join(" AND "));
            }

            let lim_idx = if address.is_some() && status.is_some() {
                3
            } else if address.is_some() || status.is_some() {
                2
            } else {
                1
            };
            sql.push_str(&format!(
                " ORDER BY block_height DESC, tx_index ASC LIMIT ${} OFFSET ${}",
                lim_idx,
                lim_idx + 1
            ));

            let mut query = sqlx::query_as::<_, TxRow>(&sql);
            if let Some(addr) = address {
                query = query.bind(addr);
            }
            if let Some(st) = status {
                query = query.bind(st);
            }
            let rows = query
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            Ok(rows
                .into_iter()
                .map(|r| TransactionListItemDto {
                    hash: r.hash,
                    block_height: r.block_height as u64,
                    from: r.from_address,
                    to: r.to_address,
                    amount: r.amount as u64,
                    fee: r.fee as u64,
                    status: r.status,
                    timestamp: r.timestamp,
                })
                .collect())
        })
    }

    fn get_transaction(&self, hash: String) -> RepoFuture<'_, TransactionDetailDto> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, TxRow>(
                r#"
                SELECT hash, block_height, from_address, to_address, amount, fee, status, timestamp, nonce, tx_index
                FROM transactions
                WHERE hash = $1
                "#,
            )
            .bind(&hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?
            .ok_or_else(|| ExplorerError::NotFound(format!("transaction {}", hash)))?;

            Ok(TransactionDetailDto {
                hash: row.hash,
                block_height: row.block_height as u64,
                from: row.from_address,
                to: row.to_address,
                amount: row.amount as u64,
                fee: row.fee as u64,
                status: row.status,
                timestamp: row.timestamp,
                nonce: row.nonce as u64,
                tx_index: row.tx_index as u32,
            })
        })
    }

    fn get_account(&self, address: String) -> RepoFuture<'_, AccountDto> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, AccountRow>(
                r#"
                SELECT address, balance, nonce, tx_count, is_contract
                FROM accounts
                WHERE LOWER(address) = LOWER($1)
                "#,
            )
            .bind(&address)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            if let Some(row) = row {
                return Ok(AccountDto {
                    address: row.address,
                    balance: row.balance as u64,
                    nonce: row.nonce as u64,
                    tx_count: row.tx_count as u64,
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
            .bind(&address)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            if tx_count > 0 {
                return Ok(AccountDto {
                    address,
                    balance: 0,
                    nonce: 0,
                    tx_count: tx_count as u64,
                    is_contract: false,
                });
            }

            // Producer/validator addresses may appear in blocks even without account state or tx history.
            let validator_address = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT address
                FROM validators
                WHERE LOWER(address) = LOWER($1)
                LIMIT 1
                "#,
            )
            .bind(&address)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            let produced_blocks = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM blocks
                WHERE LOWER(producer) = LOWER($1) OR LOWER(reward_recipient) = LOWER($1)
                "#,
            )
            .bind(&address)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            if validator_address.is_some() || produced_blocks > 0 {
                return Ok(AccountDto {
                    address: validator_address.unwrap_or(address),
                    balance: 0,
                    nonce: 0,
                    tx_count: tx_count as u64,
                    is_contract: false,
                });
            }

            Err(ExplorerError::NotFound(format!("account {}", address)))
        })
    }

    fn get_validators(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<ValidatorDto>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, ValidatorRow>(
                r#"
                SELECT address, is_active, blocks_produced, peer_id
                FROM validators
                ORDER BY blocks_produced DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            Ok(rows
                .into_iter()
                .map(|r| ValidatorDto {
                    address: r.address,
                    is_active: r.is_active,
                    blocks_produced: r.blocks_produced as u64,
                    peer_id: r.peer_id,
                })
                .collect())
        })
    }

    fn get_stats(&self) -> RepoFuture<'_, StatsDto> {
        Box::pin(async move {
            let latest_height = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(height) FROM blocks")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
                .unwrap_or(0);

            let total_transactions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            let active_validators =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM validators WHERE is_active = TRUE")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            Ok(StatsDto {
                latest_height: latest_height as u64,
                tx_per_second: 0.0,
                total_transactions: total_transactions as u64,
                active_validators: active_validators as usize,
                avg_block_time_seconds: 0.0,
            })
        })
    }

    fn search(&self, query: String) -> RepoFuture<'_, Vec<SearchResultDto>> {
        Box::pin(async move {
            let q = query.trim().to_string();
            if q.is_empty() {
                return Ok(Vec::new());
            }
            let mut items = Vec::new();

            if q.starts_with("0x") {
                if sqlx::query_scalar::<_, Option<String>>(
                    "SELECT hash FROM transactions WHERE hash = $1 LIMIT 1",
                )
                .bind(&q)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
                .is_some()
                {
                    items.push(SearchResultDto {
                        kind: "transaction".to_string(),
                        identifier: q.clone(),
                        summary: "Transaction hash match".to_string(),
                    });
                }

                if sqlx::query_scalar::<_, Option<String>>(
                    "SELECT address FROM accounts WHERE address = $1 LIMIT 1",
                )
                .bind(&q)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
                .is_some()
                {
                    items.push(SearchResultDto {
                        kind: "account".to_string(),
                        identifier: q.clone(),
                        summary: "Account address match".to_string(),
                    });
                }

                if sqlx::query_scalar::<_, Option<String>>(
                    "SELECT hash FROM blocks WHERE hash = $1 LIMIT 1",
                )
                .bind(&q)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
                .is_some()
                {
                    items.push(SearchResultDto {
                        kind: "block".to_string(),
                        identifier: q.clone(),
                        summary: "Block hash match".to_string(),
                    });
                }
            } else if let Ok(height) = q.parse::<i64>() {
                if sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT height FROM blocks WHERE height = $1 LIMIT 1",
                )
                .bind(height)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ExplorerError::Storage(e.to_string()))?
                .is_some()
                {
                    items.push(SearchResultDto {
                        kind: "block".to_string(),
                        identifier: q.clone(),
                        summary: "Block height match".to_string(),
                    });
                }
            }

            Ok(items)
        })
    }
}
