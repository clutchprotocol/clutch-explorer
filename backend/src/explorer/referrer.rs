use crate::explorer::ingestion::RawTransaction;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

/// Ensure Clutch account addresses use a `0x` prefix for display and lookups.
pub fn normalize_hex_address(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let body = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
    if body.is_empty() {
        return None;
    }
    Some(format!("0x{}", body.to_ascii_lowercase()))
}

/// Match clutch-node RidePay referrer fee rounding (ceiling division).
pub fn referrer_fee_ceiling(percent: u8, fare: u64) -> u64 {
    if percent == 0 || fare == 0 {
        return 0;
    }
    (percent as u64 * fare + 99) / 100
}

pub fn parse_referrer(arguments: &Value) -> Option<String> {
    arguments
        .get("referrer")
        .and_then(|v| v.as_str())
        .and_then(|s| normalize_hex_address(s))
}
fn arg_str(arguments: &Value, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn arg_u64(arguments: &Value, key: &str) -> u64 {
    arguments
        .get(key)
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

async fn load_payload(
    pool: &PgPool,
    hash: &str,
    block_cache: &HashMap<String, Value>,
) -> Option<Value> {
    if let Some(args) = block_cache.get(hash) {
        return Some(args.clone());
    }
    let row: Option<String> = sqlx::query_scalar(
        "SELECT payload_json FROM transactions WHERE hash = $1",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let payload = row?;
    serde_json::from_str(&payload).ok()
}

/// Fill referrer / referrer-fee fields on indexed transactions.
pub async fn enrich_transactions(
    pool: &PgPool,
    txs: &mut [RawTransaction],
    request_fee_percent: u8,
    offer_fee_percent: u8,
) {
    let mut block_cache: HashMap<String, Value> = HashMap::new();
    for tx in txs.iter() {
        if let Some(ref payload) = tx.payload_json {
            if let Ok(v) = serde_json::from_str::<Value>(payload) {
                block_cache.insert(tx.hash.clone(), v);
            }
        }
    }

    for tx in txs.iter_mut() {
        let Some(payload_str) = tx.payload_json.clone() else {
            continue;
        };
        let Ok(arguments) = serde_json::from_str::<Value>(&payload_str) else {
            continue;
        };

        match tx.function_call_type.as_str() {
            "RideRequest" | "RideOffer" => {
                tx.referrer = parse_referrer(&arguments);
            }
            "RidePay" => {
                let fare = arg_u64(&arguments, "fare");
                let acceptance_hash = arg_str(
                    &arguments,
                    "ride_acceptance_transaction_hash",
                );
                let Some(acceptance_hash) = acceptance_hash else {
                    continue;
                };

                let offer_hash = match load_payload(pool, &acceptance_hash, &block_cache).await {
                    Some(acc_args) => arg_str(&acc_args, "ride_offer_transaction_hash"),
                    None => None,
                };
                let Some(offer_hash) = offer_hash else {
                    continue;
                };

                let offer_args = load_payload(pool, &offer_hash, &block_cache).await;
                let request_hash = offer_args
                    .as_ref()
                    .and_then(|o| arg_str(o, "ride_request_transaction_hash"));
                let offer_referrer = offer_args
                    .as_ref()
                    .and_then(|o| parse_referrer(o));

                let request_referrer = if let Some(ref rh) = request_hash {
                    load_payload(pool, rh, &block_cache)
                        .await
                        .as_ref()
                        .and_then(|r| parse_referrer(r))
                } else {
                    None
                };

                let mut request_fee = 0u64;
                let mut offer_fee = 0u64;
                if request_referrer.is_some() && request_fee_percent > 0 {
                    request_fee = referrer_fee_ceiling(request_fee_percent, fare);
                }
                if offer_referrer.is_some() && offer_fee_percent > 0 {
                    offer_fee = referrer_fee_ceiling(offer_fee_percent, fare);
                }

                tx.request_referrer = request_referrer;
                tx.offer_referrer = offer_referrer;
                tx.request_referrer_fee = request_fee;
                tx.offer_referrer_fee = offer_fee;
                tx.fee = request_fee + offer_fee;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalize_hex_address_adds_prefix() {
        assert_eq!(
            normalize_hex_address("0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f").as_deref(),
            Some("0x0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f")
        );
    }

    #[test]
    fn normalize_hex_address_lowercases() {
        assert_eq!(
            normalize_hex_address("0XABCDEF").as_deref(),
            Some("0xabcdef")
        );
    }

    #[test]
    fn parse_referrer_normalizes_payload() {
        let args = json!({ "referrer": "0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f" });
        assert_eq!(
            parse_referrer(&args).as_deref(),
            Some("0x0912514c7cc3eec2b2dab4e1d150c4b5eaee5a6f")
        );
    }

    #[test]
    fn referrer_fee_ceiling_small_fare() {
        assert_eq!(referrer_fee_ceiling(2, 3), 1);
        assert_eq!(referrer_fee_ceiling(2, 50), 1);
        assert_eq!(referrer_fee_ceiling(2, 100), 2);
    }
}
