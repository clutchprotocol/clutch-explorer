use crate::explorer::error::ExplorerError;
use crate::explorer::ingestion::NodeIngestionSource;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

pub struct IndexerService {
    source: Arc<dyn NodeIngestionSource>,
    pool: PgPool,
    poll_interval_ms: u64,
    start_height: u64,
}

impl IndexerService {
    pub fn new(
        source: Arc<dyn NodeIngestionSource>,
        pool: PgPool,
        poll_interval_ms: u64,
        start_height: u64,
    ) -> Self {
        Self {
            source,
            pool,
            poll_interval_ms,
            start_height,
        }
    }

    async fn ensure_cursor(&self) -> Result<u64, ExplorerError> {
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT last_indexed_height FROM indexer_cursor WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        if let Some(v) = existing {
            return Ok(v as u64);
        }

        sqlx::query(
            "INSERT INTO indexer_cursor (id, last_indexed_height) VALUES (1, $1) ON CONFLICT (id) DO NOTHING",
        )
        .bind(self.start_height as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;
        Ok(self.start_height)
    }

    async fn set_cursor(&self, height: u64) -> Result<(), ExplorerError> {
        sqlx::query("UPDATE indexer_cursor SET last_indexed_height = $1, updated_at = NOW() WHERE id = 1")
            .bind(height as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn sync_validator_for_producer(&self, producer: &str) -> Result<(), ExplorerError> {
        sqlx::query(
            r#"
            WITH producer_count AS (
                SELECT COUNT(*)::BIGINT AS cnt
                FROM blocks
                WHERE producer = $1
            )
            INSERT INTO validators (address, is_active, blocks_produced, peer_id, updated_at)
            VALUES ($1, TRUE, (SELECT cnt FROM producer_count), '', NOW())
            ON CONFLICT (address) DO UPDATE SET
                is_active = TRUE,
                blocks_produced = EXCLUDED.blocks_produced,
                updated_at = NOW()
            "#,
        )
        .bind(producer)
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn sync_validators_from_blocks(&self) -> Result<(), ExplorerError> {
        sqlx::query(
            r#"
            INSERT INTO validators (address, is_active, blocks_produced, peer_id, updated_at)
            SELECT producer, TRUE, COUNT(*)::BIGINT, '', NOW()
            FROM blocks
            GROUP BY producer
            ON CONFLICT (address) DO UPDATE SET
                is_active = TRUE,
                blocks_produced = EXCLUDED.blocks_produced,
                updated_at = NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn ensure_genesis_indexed(&self) -> Result<(), ExplorerError> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM blocks WHERE height = 0",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        if exists == 0 {
            self.index_height(0).await?;
            info!("indexed genesis block (height 0)");
        }

        Ok(())
    }

    async fn sync_account_snapshot(&self, address: &str) -> Result<(), ExplorerError> {
        if address.trim().is_empty() || address == "unknown" {
            return Ok(());
        }

        let snapshot = self
            .source
            .fetch_account_snapshot(address.to_string())
            .await?;

        let tx_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM transactions
            WHERE LOWER(from_address) = LOWER($1) OR LOWER(to_address) = LOWER($1)
            "#,
        )
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO accounts (address, balance, nonce, tx_count, is_contract, updated_at)
            VALUES ($1, $2, $3, $4, FALSE, NOW())
            ON CONFLICT (address) DO UPDATE SET
                balance = EXCLUDED.balance,
                nonce = EXCLUDED.nonce,
                tx_count = EXCLUDED.tx_count,
                updated_at = NOW()
            "#,
        )
        .bind(snapshot.address)
        .bind(snapshot.balance as i64)
        .bind(snapshot.nonce as i64)
        .bind(tx_count)
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn index_height(&self, height: u64) -> Result<(), ExplorerError> {
        let block = self.source.fetch_block_by_height(height).await?;
        let producer = block.producer.clone();
        let reward_recipient = block.reward_recipient.clone();
        let block_reward = block.block_reward as i64;

        sqlx::query(
            r#"
            INSERT INTO blocks (height, hash, parent_hash, tx_count, producer, reward_recipient, block_reward, timestamp, total_fees)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (height) DO UPDATE SET
              hash = EXCLUDED.hash,
              parent_hash = EXCLUDED.parent_hash,
              tx_count = EXCLUDED.tx_count,
              producer = EXCLUDED.producer,
              reward_recipient = EXCLUDED.reward_recipient,
              block_reward = EXCLUDED.block_reward,
              timestamp = EXCLUDED.timestamp,
              total_fees = EXCLUDED.total_fees
            "#,
        )
        .bind(block.height as i64)
        .bind(block.hash.clone())
        .bind(block.parent_hash)
        .bind(block.tx_count as i32)
        .bind(producer.clone())
        .bind(reward_recipient)
        .bind(block_reward)
        .bind(block.timestamp)
        .bind(0i64)
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        self.sync_validator_for_producer(&producer).await?;

        let txs = self.source.fetch_transactions_by_block(height).await?;
        let mut addresses_to_sync: HashSet<String> = HashSet::new();
        addresses_to_sync.insert(producer);
        addresses_to_sync.insert(block.reward_recipient);

        for tx in txs {
            let from_address = tx.from.clone();
            let to_address = tx.to.clone();

            sqlx::query(
                r#"
                INSERT INTO transactions (
                    hash, block_height, from_address, to_address, amount, fee, status, function_call_type, is_ride_related, timestamp, nonce, tx_index
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
                ON CONFLICT (hash) DO UPDATE SET
                    block_height = EXCLUDED.block_height,
                    from_address = EXCLUDED.from_address,
                    to_address = EXCLUDED.to_address,
                    amount = EXCLUDED.amount,
                    fee = EXCLUDED.fee,
                    status = EXCLUDED.status,
                    function_call_type = EXCLUDED.function_call_type,
                    is_ride_related = EXCLUDED.is_ride_related,
                    timestamp = EXCLUDED.timestamp,
                    nonce = EXCLUDED.nonce,
                    tx_index = EXCLUDED.tx_index
                "#,
            )
            .bind(tx.hash)
            .bind(tx.block_height as i64)
            .bind(tx.from)
            .bind(tx.to)
            .bind(tx.amount as i64)
            .bind(tx.fee as i64)
            .bind(tx.status)
            .bind(tx.function_call_type)
            .bind(tx.is_ride_related)
            .bind(tx.timestamp)
            .bind(tx.nonce as i64)
            .bind(tx.tx_index as i32)
            .execute(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            addresses_to_sync.insert(from_address);
            addresses_to_sync.insert(to_address);
        }

        for address in addresses_to_sync {
            if let Err(err) = self.sync_account_snapshot(&address).await {
                error!("failed to sync account snapshot for {}: {}", address, err);
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<(), ExplorerError> {
        let mut cursor = self.ensure_cursor().await?;
        self.ensure_genesis_indexed().await?;
        self.sync_validators_from_blocks().await?;
        info!("indexer starting from cursor {}", cursor);

        loop {
            if cursor == 0 {
                if let Err(err) = self.ensure_genesis_indexed().await {
                    error!("failed to ensure genesis block is indexed: {}", err);
                }
            }
            match self.source.fetch_head().await {
                Ok(head) => {
                    if head.height > cursor {
                        for h in (cursor + 1)..=head.height {
                            match self.index_height(h).await {
                                Ok(_) => {
                                    self.set_cursor(h).await?;
                                    cursor = h;
                                    info!("indexed height {}", h);
                                }
                                Err(err) => {
                                    // Keep the service alive and retry this height on the next poll.
                                    error!("indexer failed to index height {}: {}", h, err);
                                    break;
                                }
                            }
                        }
                    } else if head.height == cursor {
                        info!("indexer up to date at {}", cursor);
                    }
                }
                Err(err) => {
                    error!("indexer fetch_head error: {}", err);
                }
            }
            sleep(Duration::from_millis(self.poll_interval_ms)).await;
        }
    }
}
