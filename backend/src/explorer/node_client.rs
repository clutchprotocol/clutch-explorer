use crate::explorer::models::{AccountDto, BlockDto, StatsDto, TransactionDto, ValidatorDto};
use chrono::Utc;
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeClientError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct NodeClient {
    http_client: Client,
    base_url: String,
}

impl NodeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
        }
    }

    pub async fn latest_blocks(&self, limit: usize) -> Result<Vec<BlockDto>, NodeClientError> {
        let _ = &self.http_client;
        let _ = &self.base_url;
        let now = Utc::now();
        Ok((0..limit)
            .map(|i| BlockDto {
                height: 1000 - i as u64,
                hash: format!("0xblock{:064x}", i),
                tx_count: ((i % 5) + 1) as u32,
                producer: format!("0xvalidator{:040x}", i % 3),
                timestamp: now,
            })
            .collect())
    }

    pub async fn block_by_id(&self, id: &str) -> Result<BlockDto, NodeClientError> {
        let height = id.parse::<u64>().unwrap_or(1000);
        Ok(BlockDto {
            height,
            hash: format!("0xblock{:064x}", height),
            tx_count: 4,
            producer: "0xvalidator000000000000000000000000000000000001".to_string(),
            timestamp: Utc::now(),
        })
    }

    pub async fn latest_transactions(
        &self,
        limit: usize,
    ) -> Result<Vec<TransactionDto>, NodeClientError> {
        let now = Utc::now();
        Ok((0..limit)
            .map(|i| TransactionDto {
                hash: format!("0xtx{:064x}", i),
                block_height: 1000 - i as u64,
                from: format!("0xfrom{:040x}", i),
                to: format!("0xto{:040x}", i + 1),
                amount: 100 + i as u64,
                fee: 1,
                status: "confirmed".to_string(),
                timestamp: now,
            })
            .collect())
    }

    pub async fn transaction_by_hash(&self, hash: &str) -> Result<TransactionDto, NodeClientError> {
        Ok(TransactionDto {
            hash: hash.to_string(),
            block_height: 999,
            from: "0xfrom000000000000000000000000000000000000".to_string(),
            to: "0xto0000000000000000000000000000000000000".to_string(),
            amount: 120,
            fee: 1,
            status: "confirmed".to_string(),
            timestamp: Utc::now(),
        })
    }

    pub async fn account_by_address(&self, address: &str) -> Result<AccountDto, NodeClientError> {
        Ok(AccountDto {
            address: address.to_string(),
            balance: 145_000,
            nonce: 42,
            tx_count: 128,
        })
    }

    pub async fn validators(&self) -> Result<Vec<ValidatorDto>, NodeClientError> {
        Ok(vec![
            ValidatorDto {
                address: "0xvalidator000000000000000000000000000000000001".to_string(),
                is_active: true,
                blocks_produced: 1200,
                peer_id: "12D3KooWNode1".to_string(),
            },
            ValidatorDto {
                address: "0xvalidator000000000000000000000000000000000002".to_string(),
                is_active: true,
                blocks_produced: 1185,
                peer_id: "12D3KooWNode2".to_string(),
            },
        ])
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
