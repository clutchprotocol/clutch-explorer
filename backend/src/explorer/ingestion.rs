use crate::explorer::error::ExplorerError;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type IngestFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ExplorerError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct RawHead {
    pub height: u64,
    pub hash: String,
    pub seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RawBlock {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub producer: String,
    pub tx_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RawTransaction {
    pub hash: String,
    pub block_height: u64,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub status: String,
    pub nonce: u64,
    pub tx_index: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RawAccountSnapshot {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

pub trait NodeIngestionSource: Send + Sync {
    fn fetch_head(&self) -> IngestFuture<'_, RawHead>;
    fn fetch_block_by_height(&self, height: u64) -> IngestFuture<'_, RawBlock>;
    fn fetch_transactions_by_block(&self, height: u64) -> IngestFuture<'_, Vec<RawTransaction>>;
    fn fetch_account_snapshot(&self, address: String) -> IngestFuture<'_, RawAccountSnapshot>;
}

pub struct NodeHttpIngestionSource {
    http: Client,
    metrics_url: String,
    ws_url: String,
}

impl NodeHttpIngestionSource {
    pub fn new(metrics_url: String, ws_url: String) -> Self {
        Self {
            http: Client::new(),
            metrics_url,
            ws_url,
        }
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, ExplorerError> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Node currently exposes a WS JSON-RPC interface; this HTTP endpoint is a scaffold
        // contract surface for indexer portability.
        let resp = self
            .http
            .post(&self.ws_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;
        let value = resp
            .json::<Value>()
            .await
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;
        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl NodeIngestionSource for NodeHttpIngestionSource {
    fn fetch_head(&self) -> IngestFuture<'_, RawHead> {
        Box::pin(async move {
            let metrics = self
                .http
                .get(&self.metrics_url)
                .send()
                .await
                .map_err(|e| ExplorerError::Upstream(e.to_string()))?
                .text()
                .await
                .map_err(|e| ExplorerError::Upstream(e.to_string()))?;

            let mut latest_index = None;
            let mut latest_hash = None;
            for line in metrics.lines() {
                if line.starts_with("latest_block_index ") {
                    latest_index = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok());
                } else if line.starts_with("latest_block{") {
                    latest_hash = line
                        .split("block_hash=\"")
                        .nth(1)
                        .and_then(|v| v.split('"').next())
                        .map(ToString::to_string);
                }
            }

            match (latest_index, latest_hash) {
                (Some(height), Some(hash)) => Ok(RawHead {
                    height,
                    hash,
                    seen_at: Utc::now(),
                }),
                _ => Err(ExplorerError::Upstream(
                    "latest_block metrics missing".to_string(),
                )),
            }
        })
    }

    fn fetch_block_by_height(&self, height: u64) -> IngestFuture<'_, RawBlock> {
        Box::pin(async move {
            let head = self.fetch_head().await?;
            let hash = if head.height == height {
                head.hash
            } else {
                format!("0xunknown{:064x}", height)
            };
            Ok(RawBlock {
                height,
                hash,
                parent_hash: format!("0xunknown{:064x}", height.saturating_sub(1)),
                producer: "unknown".to_string(),
                tx_count: 0,
                timestamp: Utc::now(),
            })
        })
    }

    fn fetch_transactions_by_block(&self, _height: u64) -> IngestFuture<'_, Vec<RawTransaction>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn fetch_account_snapshot(&self, address: String) -> IngestFuture<'_, RawAccountSnapshot> {
        Box::pin(async move {
            let result = self
                .rpc_call("get_account_balance", serde_json::json!({ "address": address }))
                .await?;
            let balance = result.get("balance").and_then(|v| v.as_u64()).unwrap_or(0);
            let nonce = self
                .rpc_call("get_next_nonce", serde_json::json!({ "address": address }))
                .await?
                .get("nonce")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Ok(RawAccountSnapshot {
                address,
                balance,
                nonce,
            })
        })
    }
}
