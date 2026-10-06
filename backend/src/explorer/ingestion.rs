use crate::explorer::activity::{
    fee_paid, parse_balance_effects_from_tx, parse_block_balance_effects, ParsedBalanceEffect,
};
use crate::explorer::error::ExplorerError;
use crate::explorer::referrer::parse_referrer;
use chrono::{DateTime, TimeZone, Utc};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone)]
pub struct RawHead {
    pub height: u64,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct RawBlock {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub producer: String,
    pub reward_recipient: String,
    pub block_reward: u64,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<RawTransaction>,
    /// Effects that belong to the block rather than one transaction (the author's fee credit).
    pub balance_effects: Vec<ParsedBalanceEffect>,
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
    pub function_call_type: String,
    pub is_ride_related: bool,
    pub nonce: u64,
    pub tx_index: u32,
    pub timestamp: DateTime<Utc>,
    pub referrer: Option<String>,
    pub request_referrer: Option<String>,
    pub offer_referrer: Option<String>,
    pub request_referrer_fee: u64,
    pub offer_referrer_fee: u64,
    pub payload_json: Option<String>,
    pub balance_effects: Vec<ParsedBalanceEffect>,
}

#[derive(Debug, Clone)]
pub struct RawAccountSnapshot {
    pub balance: u64,
    pub nonce: u64,
}

/// Reads the chain from one clutch-node: the head from its Prometheus metrics, everything else
/// over its WebSocket JSON-RPC.
pub struct NodeSource {
    http: Client,
    metrics_url: String,
    ws_url: String,
}

fn upstream(err: impl std::fmt::Display) -> ExplorerError {
    ExplorerError::Upstream(err.to_string())
}

fn parse_timestamp(value: Option<&Value>) -> DateTime<Utc> {
    value
        .and_then(|v| v.as_u64())
        .and_then(|secs| Utc.timestamp_opt(secs as i64, 0).single())
        .unwrap_or_else(Utc::now)
}

fn parse_amount(arguments: &Value) -> u64 {
    ["value", "fare", "amount"]
        .iter()
        .find_map(|key| arguments.get(*key).and_then(|v| v.as_u64()))
        .unwrap_or(0)
}

fn is_ride_function(function_call_type: &str) -> bool {
    matches!(
        function_call_type,
        "RideRequest"
            | "RideOffer"
            | "RideAcceptance"
            | "RidePay"
            | "RideCancel"
            | "RideRequestCancel"
    )
}

fn parse_transaction(
    tx: &Value,
    height: u64,
    idx: usize,
    block_ts: DateTime<Utc>,
) -> RawTransaction {
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
    let function_call_type = tx
        .get("data")
        .and_then(|d| d.get("function_call_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("Transfer")
        .to_string();
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
    let balance_effects = parse_balance_effects_from_tx(tx, height, idx as u32, block_ts);

    RawTransaction {
        hash,
        block_height: height,
        from,
        to,
        amount: parse_amount(&arguments),
        fee: fee_paid(&balance_effects),
        status: "confirmed".to_string(),
        is_ride_related: is_ride_function(&function_call_type),
        function_call_type,
        nonce,
        tx_index: idx as u32,
        timestamp: block_ts,
        referrer: parse_referrer(&arguments),
        request_referrer: None,
        offer_referrer: None,
        request_referrer_fee: 0,
        offer_referrer_fee: 0,
        payload_json: (!arguments.is_null()).then(|| arguments.to_string()),
        balance_effects,
    }
}

impl NodeSource {
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

        let (mut ws_stream, _) = connect_async(&self.ws_url).await.map_err(upstream)?;
        ws_stream
            .send(Message::Text(payload.to_string()))
            .await
            .map_err(upstream)?;

        let frame = ws_stream
            .next()
            .await
            .ok_or_else(|| upstream("empty rpc websocket response"))?
            .map_err(upstream)?;

        let text = match frame {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8(b.to_vec()).map_err(upstream)?,
            _ => return Err(upstream("unsupported rpc websocket frame")),
        };

        let value = serde_json::from_str::<Value>(&text).map_err(upstream)?;
        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }

    pub async fn fetch_head(&self) -> Result<RawHead, ExplorerError> {
        let metrics = self
            .http
            .get(&self.metrics_url)
            .send()
            .await
            .map_err(upstream)?
            .text()
            .await
            .map_err(upstream)?;

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
            (Some(height), Some(hash)) => Ok(RawHead { height, hash }),
            _ => Err(upstream("latest_block metrics missing")),
        }
    }

    /// The block at `height` with its transactions and balance effects, from one RPC call.
    pub async fn fetch_block(&self, height: u64) -> Result<RawBlock, ExplorerError> {
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
        let block_reward = block
            .get("block_reward")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let timestamp = parse_timestamp(block.get("timestamp"));

        let transactions = block
            .get("transactions")
            .and_then(|v| v.as_array())
            .map(|txs| {
                txs.iter()
                    .enumerate()
                    .map(|(idx, tx)| parse_transaction(tx, height, idx, timestamp))
                    .collect()
            })
            .unwrap_or_default();

        Ok(RawBlock {
            height,
            hash,
            parent_hash,
            reward_recipient: producer.clone(),
            producer,
            block_reward,
            timestamp,
            transactions,
            balance_effects: parse_block_balance_effects(&block, height, timestamp),
        })
    }

    pub async fn fetch_account_snapshot(
        &self,
        address: &str,
    ) -> Result<RawAccountSnapshot, ExplorerError> {
        let balance = self
            .rpc_call(
                "get_account_balance",
                serde_json::json!({ "address": address }),
            )
            .await?
            .get("balance")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let nonce = self
            .rpc_call("get_next_nonce", serde_json::json!({ "address": address }))
            .await?
            .get("nonce")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .saturating_sub(1);
        Ok(RawAccountSnapshot { balance, nonce })
    }
}
