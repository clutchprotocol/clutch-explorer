//! The reserve position, republished for the public API.
//!
//! CLT claims to be fully reserved. That claim is checkable only if both sides of it are visible,
//! so this serves the reconciliation snapshot the treasury already computes on an interval: what
//! the chain says exists, what the ledger says is owed, and what is actually held against it.
//!
//! **One snapshot, not two numbers from two moments.** It would be easy to show live chain supply
//! next to the last reconciled reserve, and it would be wrong: a mint moves supply immediately and
//! the reserve figure only at the next run, so the pair would disagree routinely and a reader would
//! read ordinary lag as a shortfall. Everything here comes from a single run, at a single instant,
//! with that instant published alongside.
//!
//! `treasury-service` publishes no port, so this reaches it across the compose network. The
//! explorer is the thing with a public API, which is why the caching and the fetch timeout live
//! here rather than there.

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Long enough to bound load from a page that polls every ten seconds, short enough that nobody
/// reads a number half a minute out of date and calls it current. The underlying run happens on
/// the order of minutes, so a longer TTL would buy nothing.
const CACHE_TTL: Duration = Duration::from_secs(30);

/// A slow treasury must not become a slow explorer. The page renders without this section rather
/// than waiting on it.
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct ReserveClient {
    http: reqwest::Client,
    /// Empty when unset, which switches the feature off rather than failing. A deployment without a
    /// treasury is a real configuration — the explorer indexes a chain either way.
    url: String,
    cache: Arc<RwLock<Option<(Instant, Value)>>>,
}

impl ReserveClient {
    #[must_use]
    pub fn new(url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            url,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// The latest reconciliation, or an explicit statement that it could not be read.
    ///
    /// Never serves a stale cache on failure. A page that keeps showing "matched" while the
    /// treasury is unreachable is worse than one showing nothing: it reports a verification that
    /// is not happening. Failure is published as failure.
    pub async fn latest(&self) -> Value {
        if self.url.is_empty() {
            return json!({ "configured": false });
        }

        if let Some((fetched_at, value)) = self.cache.read().await.as_ref() {
            if fetched_at.elapsed() < CACHE_TTL {
                return with_age(value.clone(), fetched_at.elapsed());
            }
        }

        match self.fetch().await {
            Ok(value) => {
                *self.cache.write().await = Some((Instant::now(), value.clone()));
                with_age(value, Duration::ZERO)
            }
            Err(err) => {
                tracing::warn!(error = %err, "could not read the treasury reconciliation");
                json!({ "configured": true, "available": false, "error": err })
            }
        }
    }

    async fn fetch(&self) -> Result<Value, String> {
        let response = self
            .http
            .get(&self.url)
            .timeout(FETCH_TIMEOUT)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("treasury answered {}", response.status()));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("treasury answer was not JSON: {e}"))?;

        // `last_run: null` is a real state — a chain that has never reconciled — and is passed
        // through as-is rather than treated as an error.
        Ok(json!({
            "configured": true,
            "available": true,
            "last_run": body.get("last_run").cloned().unwrap_or(Value::Null),
        }))
    }
}

/// How old the served answer is. Without it a cached response is indistinguishable from a fresh
/// one, and the whole point of this endpoint is that the reader can judge freshness themselves.
fn with_age(mut value: Value, age: Duration) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert("cache_age_seconds".to_string(), json!(age.as_secs()));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_unset_url_switches_the_feature_off_rather_than_failing() {
        let client = ReserveClient::new(String::new());
        let value = client.latest().await;
        assert_eq!(value["configured"], json!(false));
        assert!(value.get("available").is_none(), "unconfigured is not the same as unavailable");
    }

    #[tokio::test]
    async fn an_unreachable_treasury_is_reported_as_unavailable() {
        // Port 1 on loopback: reliably refused, so this exercises the failure path without a
        // network or a fixture server.
        let client = ReserveClient::new("http://127.0.0.1:1/public/reconciliation".to_string());
        let value = client.latest().await;
        assert_eq!(value["configured"], json!(true));
        assert_eq!(value["available"], json!(false));
        assert!(value["error"].is_string());
        // The failure must not be cached as if it were an answer.
        assert!(client.cache.read().await.is_none());
    }

    #[tokio::test]
    async fn a_cached_answer_carries_its_age() {
        let client = ReserveClient::new("http://127.0.0.1:1/public/reconciliation".to_string());
        *client.cache.write().await = Some((Instant::now(), json!({"configured": true, "available": true})));
        let value = client.latest().await;
        assert_eq!(value["available"], json!(true));
        assert!(value["cache_age_seconds"].is_u64(), "a reader cannot judge freshness without it");
    }
}
