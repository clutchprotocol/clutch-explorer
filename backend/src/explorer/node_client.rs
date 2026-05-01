use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeClientError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Clone)]
pub struct NodeClient {
    http_client: Client,
    base_url: String,
    strict_mode: bool,
}

impl NodeClient {
    pub fn new(base_url: String, strict_mode: bool) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            strict_mode,
        }
    }

    async fn get_json(&self, path: &str) -> Result<Value, NodeClientError> {
        let base = self.base_url.trim_end_matches('/');
        let target = format!("{}/{}", base, path.trim_start_matches('/'));
        let response = self.http_client.get(target).send().await?;
        if response.status().is_success() {
            Ok(response.json::<Value>().await?)
        } else if response.status().as_u16() == 404 {
            Err(NodeClientError::NotFound(path.to_string()))
        } else {
            Err(NodeClientError::InvalidRequest(format!(
                "upstream status {} for {}",
                response.status(),
                path
            )))
        }
    }

    pub async fn latest_blocks(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BlockListItemDto>, NodeClientError> {
        if let Ok(payload) = self
            .get_json(&format!("api/v1/blocks?limit={}&offset={}", limit, offset))
            .await
        {
            if let Some(items) = payload.get("items").and_then(|v| v.as_array()) {
                let mapped = items
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| BlockListItemDto {
                        height: item
                            .get("height")
                            .and_then(|v| v.as_u64())
                            .unwrap_or((offset + idx) as u64),
                        hash: item
                            .get("hash")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        tx_count: item
                            .get("tx_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32,
                        producer: item
                            .get("producer")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        reward_recipient: item
                            .get("reward_recipient")
                            .and_then(|v| v.as_str())
                            .or_else(|| item.get("producer").and_then(|v| v.as_str()))
                            .unwrap_or("unknown")
                            .to_string(),
                        block_reward: item
                            .get("block_reward")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        timestamp: item
                            .get("timestamp")
                            .and_then(|v| v.as_str())
                            .and_then(|v| v.parse().ok())
                            .unwrap_or_else(Utc::now),
                    })
                    .collect::<Vec<_>>();
                if !mapped.is_empty() {
                    return Ok(mapped);
                }
            }
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(
                "no block data returned from upstream node".to_string(),
            ));
        }
        let now = Utc::now();
        Ok((0..limit)
            .map(|i| {
                let height = 100000u64.saturating_sub((offset + i) as u64);
                BlockListItemDto {
                    height,
                    hash: format!("0xblock{:064x}", height),
                    tx_count: (((offset + i) % 6) + 1) as u32,
                    producer: format!("0xvalidator{:040x}", (offset + i) % 5),
                    reward_recipient: format!("0xvalidator{:040x}", (offset + i) % 5),
                    block_reward: 0,
                    timestamp: now - chrono::TimeDelta::seconds((offset + i) as i64 * 6),
                }
            })
            .collect())
    }

    pub async fn block_by_id(&self, id: &str) -> Result<BlockDetailDto, NodeClientError> {
        if let Ok(payload) = self.get_json(&format!("api/v1/blocks/{}", id)).await {
            return Ok(BlockDetailDto {
                height: payload.get("height").and_then(|v| v.as_u64()).unwrap_or(0),
                hash: payload
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or(id)
                    .to_string(),
                parent_hash: payload
                    .get("parent_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0xparent")
                    .to_string(),
                tx_count: payload
                    .get("tx_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                producer: payload
                    .get("producer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                reward_recipient: payload
                    .get("reward_recipient")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("producer").and_then(|v| v.as_str()))
                    .unwrap_or("unknown")
                    .to_string(),
                block_reward: payload
                    .get("block_reward")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                timestamp: payload
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(Utc::now),
                total_fees: payload.get("total_fees").and_then(|v| v.as_u64()).unwrap_or(0),
            });
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(format!(
                "block {} not found in upstream node",
                id
            )));
        }
        let height = id.parse::<u64>().unwrap_or(100000);
        Ok(BlockDetailDto {
            height,
            hash: format!("0xblock{:064x}", height),
            parent_hash: format!("0xblock{:064x}", height.saturating_sub(1)),
            tx_count: 8,
            producer: "0xvalidator000000000000000000000000000000000001".to_string(),
            reward_recipient: "0xvalidator000000000000000000000000000000000001".to_string(),
            block_reward: 0,
            timestamp: Utc::now(),
            total_fees: 24,
        })
    }

    pub async fn latest_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<TransactionListItemDto>, NodeClientError> {
        let mut path = format!("api/v1/transactions?limit={}&offset={}", limit, offset);
        if let Some(addr) = address {
            path.push_str(&format!("&address={}", addr));
        }
        if let Some(st) = status {
            path.push_str(&format!("&status={}", st));
        }

        if let Ok(payload) = self.get_json(&path).await {
            if let Some(items) = payload.get("items").and_then(|v| v.as_array()) {
                let mapped = items
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| TransactionListItemDto {
                        hash: item
                            .get("hash")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        block_height: item
                            .get("block_height")
                            .and_then(|v| v.as_u64())
                            .unwrap_or((offset + idx) as u64),
                        from: item
                            .get("from")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        to: item
                            .get("to")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        amount: item.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                        fee: item.get("fee").and_then(|v| v.as_u64()).unwrap_or(0),
                        status: item
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("confirmed")
                            .to_string(),
                        function_call_type: item
                            .get("function_call_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Transfer")
                            .to_string(),
                        is_ride_related: item
                            .get("is_ride_related")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        timestamp: item
                            .get("timestamp")
                            .and_then(|v| v.as_str())
                            .and_then(|v| v.parse().ok())
                            .unwrap_or_else(Utc::now),
                    })
                    .collect::<Vec<_>>();
                if !mapped.is_empty() {
                    return Ok(mapped);
                }
            }
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(
                "no transaction data returned from upstream node".to_string(),
            ));
        }

        let now = Utc::now();
        Ok((0..limit)
            .map(|i| {
                let index = offset + i;
                TransactionListItemDto {
                hash: format!("0xtx{:064x}", 900000 + index),
                block_height: 100000u64.saturating_sub(index as u64),
                from: format!("0xfrom{:040x}", index % 1000),
                to: format!("0xto{:040x}", (index + 1) % 1000),
                amount: 100 + index as u64,
                fee: 1,
                status: status.unwrap_or("confirmed").to_string(),
                function_call_type: "Transfer".to_string(),
                is_ride_related: false,
                timestamp: now - chrono::TimeDelta::seconds(index as i64 * 4),
                }
            })
            .collect())
    }

    pub async fn transaction_by_hash(
        &self,
        hash: &str,
    ) -> Result<TransactionDetailDto, NodeClientError> {
        if let Ok(payload) = self.get_json(&format!("api/v1/transactions/{}", hash)).await {
            return Ok(TransactionDetailDto {
                hash: payload
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or(hash)
                    .to_string(),
                block_height: payload
                    .get("block_height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                from: payload
                    .get("from")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                to: payload
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                amount: payload.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                fee: payload.get("fee").and_then(|v| v.as_u64()).unwrap_or(0),
                status: payload
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("confirmed")
                    .to_string(),
                function_call_type: payload
                    .get("function_call_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Transfer")
                    .to_string(),
                is_ride_related: payload
                    .get("is_ride_related")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                timestamp: payload
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(Utc::now),
                nonce: payload.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0),
                tx_index: payload.get("tx_index").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            });
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(format!(
                "transaction {} not found in upstream node",
                hash
            )));
        }
        Ok(TransactionDetailDto {
            hash: hash.to_string(),
            block_height: 99999,
            from: "0xfrom000000000000000000000000000000000000".to_string(),
            to: "0xto0000000000000000000000000000000000000".to_string(),
            amount: 120,
            fee: 1,
            status: "confirmed".to_string(),
            function_call_type: "Transfer".to_string(),
            is_ride_related: false,
            timestamp: Utc::now(),
            nonce: 78,
            tx_index: 3,
        })
    }

    pub async fn account_by_address(&self, address: &str) -> Result<AccountDto, NodeClientError> {
        if let Ok(payload) = self.get_json(&format!("api/v1/accounts/{}", address)).await {
            return Ok(AccountDto {
                address: payload
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or(address)
                    .to_string(),
                balance: payload.get("balance").and_then(|v| v.as_u64()).unwrap_or(0),
                nonce: payload.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0),
                tx_count: payload.get("tx_count").and_then(|v| v.as_u64()).unwrap_or(0),
                is_contract: payload
                    .get("is_contract")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            });
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(format!(
                "account {} not found in upstream node",
                address
            )));
        }
        Ok(AccountDto {
            address: address.to_string(),
            balance: 145_000,
            nonce: 42,
            tx_count: 128,
            is_contract: address.ends_with("c"),
        })
    }

    pub async fn validators(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ValidatorDto>, NodeClientError> {
        if let Ok(payload) = self
            .get_json(&format!("api/v1/validators?limit={}&offset={}", limit, offset))
            .await
        {
            if let Some(items) = payload.get("items").and_then(|v| v.as_array()) {
                let mapped = items
                    .iter()
                    .map(|item| ValidatorDto {
                        address: item
                            .get("address")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        is_active: item
                            .get("is_active")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        blocks_produced: item
                            .get("blocks_produced")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        peer_id: item
                            .get("peer_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    })
                    .collect::<Vec<_>>();
                if !mapped.is_empty() {
                    return Ok(mapped);
                }
            }
        }
        if self.strict_mode {
            return Err(NodeClientError::NotFound(
                "no validator data returned from upstream node".to_string(),
            ));
        }
        Ok(vec![
            ValidatorDto {
                address: "0xvalidator000000000000000000000000000000000001".to_string(),
                is_active: true,
                blocks_produced: 1200 + offset as u64,
                peer_id: "12D3KooWNode1".to_string(),
            },
            ValidatorDto {
                address: "0xvalidator000000000000000000000000000000000002".to_string(),
                is_active: true,
                blocks_produced: 1185 + offset as u64,
                peer_id: "12D3KooWNode2".to_string(),
            },
        ]
        .into_iter()
        .take(limit.max(1))
        .collect())
    }

    pub async fn stats(&self) -> Result<StatsDto, NodeClientError> {
        Ok(StatsDto {
            latest_height: 1000,
            tx_per_second: 2.8,
            total_transactions: 54_300,
            active_validators: 2,
            avg_block_time_seconds: 3.2,
        })
    }
}
