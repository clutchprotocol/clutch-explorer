use crate::explorer::activity::insert_account_activity;
use crate::explorer::error::ExplorerError;
use crate::explorer::ingestion::{NodeIngestionSource, RawHead};
use crate::explorer::referrer::{enrich_transactions, normalize_hex_address};
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

/// Ceiling on how far back to search for the last common ancestor during a reorg. A node
/// restart/wipe resolves in one or two steps; this only guards against a pathological case
/// (e.g. the explorer pointed at a node with a wholly unrelated chain) searching forever.
/// ponytail: fixed depth; make configurable if a real multi-block reorg ever needs more.
const MAX_REORG_SEARCH_DEPTH: u64 = 1000;

pub struct IndexerService {
    source: Arc<dyn NodeIngestionSource>,
    pool: PgPool,
    poll_interval_ms: u64,
    start_height: u64,
    ride_request_referrer_fee_percent: u8,
    ride_offer_referrer_fee_percent: u8,
}

/// Pure decision: has the chain diverged from what's already indexed, given the node's
/// reported head and our cursor? Extracted so the actual bug this guards against — silently
/// ignoring `head.height < cursor` — has a direct unit test.
fn head_has_diverged(head_height: u64, cursor: u64, head_hash_matches_cursor: bool) -> bool {
    if head_height < cursor {
        true
    } else if head_height == cursor {
        !head_hash_matches_cursor
    } else {
        false
    }
}

impl IndexerService {
    fn is_real_validator_address(address: &str) -> bool {
        let trimmed = address.trim();
        !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("unknown")
    }

    pub fn new(
        source: Arc<dyn NodeIngestionSource>,
        pool: PgPool,
        poll_interval_ms: u64,
        start_height: u64,
        ride_request_referrer_fee_percent: u8,
        ride_offer_referrer_fee_percent: u8,
    ) -> Self {
        Self {
            source,
            pool,
            poll_interval_ms,
            start_height,
            ride_request_referrer_fee_percent,
            ride_offer_referrer_fee_percent,
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
        if !Self::is_real_validator_address(producer) {
            return Ok(());
        }

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
            WHERE TRIM(producer) <> '' AND LOWER(producer) <> 'unknown'
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

        let canonical = normalize_hex_address(address).unwrap_or_else(|| address.to_string());

        let snapshot = self
            .source
            .fetch_account_snapshot(canonical.clone())
            .await?;

        let tx_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM transactions
            WHERE LOWER(from_address) = LOWER($1) OR LOWER(to_address) = LOWER($1)
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        let activity_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM account_activity
            WHERE LOWER(address) = LOWER($1)
            "#,
        )
        .bind(&canonical)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO accounts (address, balance, nonce, tx_count, activity_count, is_contract, updated_at)
            VALUES ($1, $2, $3, $4, $5, FALSE, NOW())
            ON CONFLICT (address) DO UPDATE SET
                balance = EXCLUDED.balance,
                nonce = EXCLUDED.nonce,
                tx_count = EXCLUDED.tx_count,
                activity_count = EXCLUDED.activity_count,
                updated_at = NOW()
            "#,
        )
        .bind(&canonical)
        .bind(snapshot.balance as i64)
        .bind(snapshot.nonce as i64)
        .bind(tx_count)
        .bind(activity_count)
        .execute(&self.pool)
        .await
        .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Compares the node's current block hash at `height` against what's stored in Postgres.
    /// A height we haven't indexed yet reads as "not diverged" — nothing to roll back, the
    /// caller should just index forward normally.
    async fn hash_matches_at(&self, height: u64, node_hash: &str) -> Result<bool, ExplorerError> {
        let stored: Option<String> = sqlx::query_scalar("SELECT hash FROM blocks WHERE height = $1")
            .bind(height as i64)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;
        Ok(match stored {
            Some(h) => h == node_hash,
            None => true,
        })
    }

    /// Walks backward from `from_height` for the last height whose stored hash still matches
    /// the node's current hash there. `Some(h)` = last common ancestor, everything indexed
    /// above `h` is stale; `None` = even genesis diverged, wipe and reindex from scratch.
    async fn find_fork_point(&self, from_height: u64) -> Result<Option<u64>, ExplorerError> {
        let mut height = from_height;
        let mut steps = 0u64;
        loop {
            let node_block = self.source.fetch_block_by_height(height).await?;
            if self.hash_matches_at(height, &node_block.hash).await? {
                return Ok(Some(height));
            }
            if height == 0 || steps >= MAX_REORG_SEARCH_DEPTH {
                return Ok(None);
            }
            height -= 1;
            steps += 1;
        }
    }

    /// Deletes all indexed data above `fork_point` (or everything, including genesis, if
    /// `None`) and rewinds the cursor so the main loop resumes from the correct point.
    /// `transactions` cascade-deletes via its FK on `blocks`; `account_activity` has no FK
    /// to `blocks` and is deleted explicitly. Validator producer counts are a running
    /// `COUNT(*) FROM blocks`, so they're recomputed once the stale rows are gone.
    async fn unwind_to(&self, fork_point: Option<u64>) -> Result<u64, ExplorerError> {
        let (boundary, new_cursor): (i64, u64) = match fork_point {
            Some(h) => (h as i64, h),
            None => (-1, 0),
        };

        sqlx::query("DELETE FROM account_activity WHERE block_height > $1")
            .bind(boundary)
            .execute(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        sqlx::query("DELETE FROM blocks WHERE height > $1")
            .bind(boundary)
            .execute(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

        self.set_cursor(new_cursor).await?;
        self.sync_validators_from_blocks().await?;

        Ok(new_cursor)
    }

    /// Runs once per poll before the forward walk. Free in the steady state — reuses the
    /// `head` already fetched this poll — and only reaches out to the node again once a
    /// mismatch is actually suspected (node behind cursor, or the tip's hash disagrees).
    async fn reconcile_head(&self, head: &RawHead, cursor: &mut u64) -> Result<(), ExplorerError> {
        if *cursor == 0 {
            return Ok(());
        }

        let head_hash_matches_cursor = if head.height == *cursor {
            self.hash_matches_at(*cursor, &head.hash).await?
        } else {
            true // irrelevant when heights differ; head_has_diverged ignores it in that case
        };

        if !head_has_diverged(head.height, *cursor, head_hash_matches_cursor) {
            return Ok(());
        }

        let search_from = head.height.min(*cursor);
        error!(
            "chain divergence detected: node head {} vs indexed cursor {}; searching for fork point from {}",
            head.height, *cursor, search_from
        );
        let fork_point = self.find_fork_point(search_from).await?;
        let old_cursor = *cursor;
        let new_cursor = self.unwind_to(fork_point).await?;
        error!(
            "reorg handled: rewound cursor from {} to {}",
            old_cursor, new_cursor
        );
        *cursor = new_cursor;
        Ok(())
    }

    async fn index_height(&self, height: u64) -> Result<(), ExplorerError> {
        let block = self.source.fetch_block_by_height(height).await?;

        if height > 0 && !self.hash_matches_at(height - 1, &block.parent_hash).await? {
            // The block we're about to index doesn't chain from what we have stored for the
            // previous height — a reorg happened below our current polling frontier, where
            // `reconcile_head`'s cursor-only check wouldn't see it. Unwind and let the next
            // poll resume from the corrected cursor.
            error!(
                "parent hash mismatch indexing height {}: reorg below the current frontier",
                height
            );
            let fork_point = self.find_fork_point(height - 1).await?;
            self.unwind_to(fork_point).await?;
            return Err(ExplorerError::Storage(format!(
                "reorg detected while indexing height {}; cursor rewound, retrying next poll",
                height
            )));
        }

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

        let block_data = self.source.fetch_transactions_by_block(height).await?;
        let mut txs = block_data.transactions;
        enrich_transactions(
            &self.pool,
            &mut txs,
            self.ride_request_referrer_fee_percent,
            self.ride_offer_referrer_fee_percent,
        )
        .await;

        for effect in &block_data.block_balance_effects {
            insert_account_activity(&self.pool, effect).await?;
        }

        let mut addresses_to_sync: HashSet<String> = HashSet::new();
        if Self::is_real_validator_address(&producer) {
            addresses_to_sync.insert(producer);
        }
        if Self::is_real_validator_address(&block.reward_recipient) {
            addresses_to_sync.insert(block.reward_recipient);
        }

        for tx in txs {
            let from_address = tx.from.clone();
            let to_address = tx.to.clone();

            sqlx::query(
                r#"
                INSERT INTO transactions (
                    hash, block_height, from_address, to_address, amount, fee, status, function_call_type, is_ride_related, timestamp, nonce, tx_index,
                    referrer, request_referrer, offer_referrer, request_referrer_fee, offer_referrer_fee, payload_json
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
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
                    tx_index = EXCLUDED.tx_index,
                    referrer = EXCLUDED.referrer,
                    request_referrer = EXCLUDED.request_referrer,
                    offer_referrer = EXCLUDED.offer_referrer,
                    request_referrer_fee = EXCLUDED.request_referrer_fee,
                    offer_referrer_fee = EXCLUDED.offer_referrer_fee,
                    payload_json = EXCLUDED.payload_json
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
            .bind(tx.referrer.as_deref())
            .bind(tx.request_referrer.as_deref())
            .bind(tx.offer_referrer.as_deref())
            .bind(tx.request_referrer_fee as i64)
            .bind(tx.offer_referrer_fee as i64)
            .bind(tx.payload_json.as_deref())
            .execute(&self.pool)
            .await
            .map_err(|e| ExplorerError::Storage(e.to_string()))?;

            for effect in &tx.balance_effects {
                insert_account_activity(&self.pool, effect).await?;
                addresses_to_sync.insert(effect.address.clone());
                if let Some(ref cp) = effect.counterparty {
                    addresses_to_sync.insert(cp.clone());
                }
            }

            addresses_to_sync.insert(from_address);
            addresses_to_sync.insert(to_address);
            if let Some(ref r) = tx.referrer {
                addresses_to_sync.insert(r.clone());
            }
            if let Some(ref r) = tx.request_referrer {
                addresses_to_sync.insert(r.clone());
            }
            if let Some(ref r) = tx.offer_referrer {
                addresses_to_sync.insert(r.clone());
            }
        }

        for effect in &block_data.block_balance_effects {
            addresses_to_sync.insert(effect.address.clone());
            if let Some(ref cp) = effect.counterparty {
                addresses_to_sync.insert(cp.clone());
            }
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
                    if let Err(err) = self.reconcile_head(&head, &mut cursor).await {
                        error!("indexer reorg reconciliation failed: {}", err);
                    }
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

#[cfg(test)]
mod tests {
    use super::head_has_diverged;

    #[test]
    fn node_behind_cursor_is_always_a_divergence() {
        // The bug this guards against: the old code had no branch at all for this case and
        // silently idled forever (e.g. after clutch-node restarts with developer_mode=true
        // and wipes back to a lower height).
        assert!(head_has_diverged(50, 100, true));
        assert!(head_has_diverged(50, 100, false));
    }

    #[test]
    fn same_height_diverged_only_if_hash_disagrees() {
        assert!(!head_has_diverged(100, 100, true));
        assert!(head_has_diverged(100, 100, false));
    }

    #[test]
    fn normal_forward_progress_is_not_a_divergence() {
        assert!(!head_has_diverged(150, 100, true));
        assert!(!head_has_diverged(150, 100, false));
    }
}
