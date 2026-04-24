use crate::explorer::error::ExplorerError;
use chrono::{DateTime, TimeZone, Utc};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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

        let (mut ws_stream, _) = connect_async(&self.ws_url)
            .await
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;

        ws_stream
            .send(Message::Text(payload.to_string().into()))
            .await
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;

        let frame = ws_stream
            .next()
            .await
            .ok_or_else(|| ExplorerError::Upstream("empty rpc websocket response".to_string()))?
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;

        let text = match frame {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8(b.to_vec())
                .map_err(|e| ExplorerError::Upstream(e.to_string()))?,
            _ => {
                return Err(ExplorerError::Upstream(
                    "unsupported rpc websocket frame".to_string(),
                ))
            }
        };

        let value = serde_json::from_str::<Value>(&text)
            .map_err(|e| ExplorerError::Upstream(e.to_string()))?;

        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }

    fn parse_timestamp(ts_secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(ts_secs, 0)
            .single()
            .unwrap_or_else(Utc::now)
    }

    fn parse_amount(arguments: &Value) -> u64 {
        arguments
            .get("value")
            .and_then(|v| v.as_u64())
            .or_else(|| arguments.get("fare").and_then(|v| v.as_u64()))
            .or_else(|| arguments.get("amount").and_then(|v| v.as_u64()))
            .unwrap_or(0)
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
            let block = self
                .rpc_call("get_block_by_index", serde_json::json!({ "index": height }))
                .await?;

            let hash = block
                .get("hash")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("0xunknown{:064x}", height));

            let parent_hash = block
                .get("previous_hash")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("0xunknown{:064x}", height.saturating_sub(1)));

            let producer = block
                .get("author")
                .and_then(|v| v.as_str())
                .filter(|v| !v.is_empty())
                .unwrap_or("unknown")
                .to_string();

            let tx_count = block
                .get("transactions")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len() as u32)
                .unwrap_or(0);

            let timestamp = block
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .map(|v| Self::parse_timestamp(v as i64))
                .unwrap_or_else(Utc::now);

            Ok(RawBlock {
                height,
                hash,
                parent_hash,
                producer,
                tx_count,
                timestamp,
            })
        })
    }

    fn fetch_transactions_by_block(&self, height: u64) -> IngestFuture<'_, Vec<RawTransaction>> {
        Box::pin(async move {
            let block = self
                .rpc_call("get_block_by_index", serde_json::json!({ "index": height }))
                .await?;

            let block_ts = block
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .map(|v| Self::parse_timestamp(v as i64))
                .unwrap_or_else(Utc::now);

            let txs = block
                .get("transactions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let mut out = Vec::with_capacity(txs.len());
            for (idx, tx) in txs.into_iter().enumerate() {
                let from = tx
                    .get("from")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let hash = tx
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("0x{}{:08x}", height, idx));

                let nonce = tx.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);

                let arguments = tx
                    .get("data")
                    .and_then(|d| d.get("arguments"))
                    .cloned()
                    .unwrap_or(Value::Null);

                let to = arguments
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let amount = Self::parse_amount(&arguments);

                out.push(RawTransaction {
                    hash,
                    block_height: height,
                    from,
                    to,
                    amount,
                    fee: 0,
                    status: "confirmed".to_string(),
                    nonce,
                    tx_index: idx as u32,
                    timestamp: block_ts,
                });
            }

            Ok(out)
        })
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
