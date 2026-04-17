use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

pub struct SeqLogger {
    seq_url: String,
    api_key: String,
    client: Client,
}

impl SeqLogger {
    pub fn new(seq_url: &str, api_key: &str) -> Self {
        Self {
            seq_url: seq_url.to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn log_to_seq(
        &self,
        message: &str,
        level: &str,
        fields: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        let mut event = json!({
            "@t": Utc::now().to_rfc3339(),
            "@mt": message,
            "@l": level,
        });

        if let Some(fields_map) = fields.as_object() {
            for (key, value) in fields_map {
                event[key] = value.clone();
            }
        }

        let seq_address = format!("{}/ingest/clef", self.seq_url);
        let payload = format!("{}\n", event);
        let response = self
            .client
            .post(&seq_address)
            .header("Content-Type", "application/vnd.serilog.clef")
            .header("X-Seq-ApiKey", self.api_key.to_string())
            .body(payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_message = response.text().await?;
            Err(format!("Failed to send log: {}", error_message).into())
        }
    }
}

pub struct SeqLayer {
    logger: Arc<Mutex<SeqLogger>>,
}

impl SeqLayer {
    pub fn new(logger: Arc<Mutex<SeqLogger>>) -> Self {
        Self { logger }
    }
}

impl<S> Layer<S> for SeqLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let logger = self.logger.clone();
        let mut fields_map = HashMap::new();

        event.record(
            &mut |field: &tracing::field::Field, value: &dyn std::fmt::Debug| {
                fields_map.insert(field.name().to_string(), format!("{:?}", value));
            },
        );

        let fields_json = serde_json::to_value(fields_map).unwrap_or_else(|_| json!({}));
        let message = format!("Log event: {}", event.metadata().target());
        let level = event.metadata().level().as_str();

        tokio::spawn(async move {
            if let Err(err) = logger
                .lock()
                .await
                .log_to_seq(&message, level, &fields_json)
                .await
            {
                eprintln!("[SeqLayer] failed to send log to Seq: {}", err);
            }
        });
    }
}
