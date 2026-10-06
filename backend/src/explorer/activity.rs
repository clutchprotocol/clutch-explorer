use crate::explorer::referrer::normalize_hex_address;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParsedBalanceEffect {
    pub address: String,
    pub kind: String,
    pub delta: i64,
    pub counterparty: Option<String>,
    pub tx_hash: Option<String>,
    pub block_height: u64,
    pub tx_index: Option<u32>,
    pub function_call_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub fn effect_label(kind: &str) -> &'static str {
    match kind {
        "transfer_out" => "Transfer sent",
        "transfer_in" => "Transfer received",
        "ride_acceptance_debit" => "Ride acceptance hold",
        "ride_pay_driver_credit" => "Driver payout",
        "referrer_request_fee" => "Referrer reward (request app)",
        "referrer_offer_fee" => "Referrer reward (offer app)",
        "ride_cancel_refund" => "Ride cancel refund",
        "ride_auto_release" => "Ride auto-release",
        "block_reward" => "Block reward",
        "mint" => "Mint",
        "burn" => "Burn",
        "tx_fee_paid" => "Network fee",
        "tx_fee_earned" => "Block fees earned",
        _ => "Balance change",
    }
}

/// The fee a transaction paid, read from its `tx_fee_paid` effects. The node's transaction payload
/// carries no fee field, so this is the only place it can be learned. Zero when the node merged
/// the fee into another write (a ride acceptance folds it into the escrow debit) or when the sender
/// produced the block (no self-fee).
pub fn fee_paid(effects: &[ParsedBalanceEffect]) -> u64 {
    effects
        .iter()
        .filter(|e| e.kind == "tx_fee_paid" && e.delta < 0)
        .map(|e| e.delta.unsigned_abs())
        .sum()
}

/// What the block author earned in fees, from the block's `tx_fee_earned` effect.
pub fn fees_earned(block_effects: &[ParsedBalanceEffect]) -> u64 {
    block_effects
        .iter()
        .filter(|e| e.kind == "tx_fee_earned" && e.delta > 0)
        .map(|e| e.delta as u64)
        .sum()
}

pub fn parse_balance_effects_from_tx(
    tx: &Value,
    block_height: u64,
    tx_index: u32,
    block_ts: DateTime<Utc>,
) -> Vec<ParsedBalanceEffect> {
    let tx_hash = tx.get("hash").and_then(|v| v.as_str()).map(String::from);
    let function_call_type = tx
        .get("data")
        .and_then(|d| d.get("function_call_type"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let from = tx.get("from").and_then(|v| v.as_str()).map(String::from);

    parse_effects_array(
        tx.get("balance_effects"),
        block_height,
        tx_index,
        block_ts,
        tx_hash,
        function_call_type,
        from,
    )
}

pub fn parse_block_balance_effects(
    block: &Value,
    block_height: u64,
    block_ts: DateTime<Utc>,
) -> Vec<ParsedBalanceEffect> {
    parse_effects_array(
        block.get("balance_effects"),
        block_height,
        0,
        block_ts,
        None,
        None,
        None,
    )
}

fn parse_effects_array(
    effects: Option<&Value>,
    block_height: u64,
    tx_index: u32,
    block_ts: DateTime<Utc>,
    tx_hash: Option<String>,
    function_call_type: Option<String>,
    default_counterparty: Option<String>,
) -> Vec<ParsedBalanceEffect> {
    let Some(arr) = effects.and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for item in arr {
        let Some(parsed) = parse_one_effect(
            item,
            block_height,
            tx_index,
            block_ts,
            tx_hash.clone(),
            function_call_type.clone(),
            default_counterparty.clone(),
        ) else {
            continue;
        };
        out.push(parsed);
    }
    out
}

fn parse_one_effect(
    item: &Value,
    block_height: u64,
    tx_index: u32,
    block_ts: DateTime<Utc>,
    tx_hash: Option<String>,
    function_call_type: Option<String>,
    default_counterparty: Option<String>,
) -> Option<ParsedBalanceEffect> {
    let effect = item.get("effect").unwrap_or(item);
    let address = effect
        .get("address")
        .and_then(|v| v.as_str())
        .and_then(|s| normalize_hex_address(s))?;
    let delta = effect.get("delta").and_then(|v| v.as_i64())?;
    let kind = effect
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let counterparty = effect
        .get("counterparty")
        .and_then(|v| v.as_str())
        .and_then(|s| normalize_hex_address(s))
        .or(default_counterparty);

    let ts = item
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .map(|secs| Utc.timestamp_opt(secs as i64, 0).single())
        .flatten()
        .unwrap_or(block_ts);

    let tx_hash = item
        .get("tx_hash")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or(tx_hash);
    let tx_index = item
        .get("tx_index")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .or(if tx_hash.is_some() {
            Some(tx_index)
        } else {
            None
        });
    let function_call_type = item
        .get("function_call_type")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or(function_call_type);

    Some(ParsedBalanceEffect {
        address,
        kind,
        delta,
        counterparty,
        tx_hash,
        block_height: item
            .get("block_height")
            .and_then(|v| v.as_u64())
            .unwrap_or(block_height),
        tx_index,
        function_call_type,
        timestamp: ts,
    })
}

pub fn activity_direction(delta: i64) -> &'static str {
    if delta >= 0 {
        "in"
    } else {
        "out"
    }
}

pub fn activity_amount(delta: i64) -> i64 {
    delta.unsigned_abs() as i64
}

pub async fn insert_account_activity(
    pool: &sqlx::PgPool,
    effect: &ParsedBalanceEffect,
) -> Result<(), crate::explorer::error::ExplorerError> {
    let direction = activity_direction(effect.delta);
    let amount = activity_amount(effect.delta);
    let label = effect_label(&effect.kind);

    sqlx::query(
        r#"
        INSERT INTO account_activity (
            address, kind, delta, direction, amount, tx_hash, block_height, tx_index,
            function_call_type, counterparty, label, timestamp
        )
        SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
        WHERE NOT EXISTS (
            SELECT 1 FROM account_activity
            WHERE LOWER(address) = LOWER($1)
              AND COALESCE(tx_hash, '') = COALESCE($6, '')
              AND kind = $2
              AND block_height = $7
              AND delta = $3
              AND COALESCE(tx_index, -1) = COALESCE($8, -1)
        )
        "#,
    )
    .bind(&effect.address)
    .bind(&effect.kind)
    .bind(effect.delta)
    .bind(direction)
    .bind(amount)
    .bind(effect.tx_hash.as_deref())
    .bind(effect.block_height as i64)
    .bind(effect.tx_index.map(|v| v as i32))
    .bind(effect.function_call_type.as_deref())
    .bind(effect.counterparty.as_deref())
    .bind(label)
    .bind(effect.timestamp)
    .execute(pool)
    .await
    .map_err(|e| crate::explorer::error::ExplorerError::Storage(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn parse_tx_balance_effects_referrer_request_fee() {
        let block_ts = Utc::now();
        let tx = json!({
            "hash": "0xabc123",
            "from": "0xpassenger",
            "balance_effects": [
                {
                    "address": "0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f",
                    "delta": 1,
                    "kind": "referrer_request_fee",
                    "counterparty": "0xpassenger"
                }
            ]
        });

        let effects = parse_balance_effects_from_tx(&tx, 4, 0, block_ts);
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0].address,
            "0x0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f"
        );
        assert_eq!(effects[0].kind, "referrer_request_fee");
        assert_eq!(effects[0].delta, 1);
        assert_eq!(effects[0].tx_hash.as_deref(), Some("0xabc123"));
        assert_eq!(effect_label("referrer_request_fee"), "Referrer reward (request app)");
    }

    #[test]
    fn parse_block_balance_effects_block_reward() {
        let block_ts = Utc::now();
        let block = json!({
            "balance_effects": [
                {
                    "address": "0x9b6e8afff8329743cac73dbef83ca3cbf9a74c20",
                    "delta": 50,
                    "kind": "block_reward"
                }
            ]
        });

        let effects = parse_block_balance_effects(&block, 1, block_ts);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].kind, "block_reward");
        assert_eq!(effects[0].delta, 50);
        assert!(effects[0].tx_hash.is_none());
    }

    #[test]
    fn activity_direction_and_amount() {
        assert_eq!(activity_direction(5), "in");
        assert_eq!(activity_direction(-3), "out");
        assert_eq!(activity_amount(-3), 3);
    }

    fn effect(kind: &str, delta: i64) -> ParsedBalanceEffect {
        ParsedBalanceEffect {
            address: "0x01".to_string(),
            kind: kind.to_string(),
            delta,
            counterparty: None,
            tx_hash: None,
            block_height: 1,
            tx_index: None,
            function_call_type: None,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn fee_paid_sums_only_fee_debits() {
        let effects = vec![
            effect("transfer_out", -5_000_000),
            effect("tx_fee_paid", -1_000),
            effect("transfer_in", 5_000_000),
        ];
        assert_eq!(fee_paid(&effects), 1_000);
        assert_eq!(fee_paid(&[effect("transfer_out", -10)]), 0);
    }

    #[test]
    fn fees_earned_reads_the_block_credit() {
        let effects = vec![effect("tx_fee_earned", 3_000), effect("block_reward", 7)];
        assert_eq!(fees_earned(&effects), 3_000);
        assert_eq!(fees_earned(&[]), 0);
    }

}
