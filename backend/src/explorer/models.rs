use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockDto {
    pub height: u64,
    pub hash: String,
    pub tx_count: u32,
    pub producer: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDto {
    pub hash: String,
    pub block_height: u64,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountDto {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub tx_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidatorDto {
    pub address: String,
    pub is_active: bool,
    pub blocks_produced: u64,
    pub peer_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsDto {
    pub latest_height: u64,
    pub tx_per_second: f64,
    pub total_transactions: u64,
    pub active_validators: usize,
    pub avg_block_time_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResultDto {
    pub kind: String,
    pub identifier: String,
    pub summary: String,
}
