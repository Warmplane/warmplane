// Rust guideline compliant 2026-08-15

//! SIEM telemetry exporter dispatchers for Splunk HEC and HTTP Webhook endpoints.
//!
//! Provides batched, asynchronous transmission of audit events to external security collectors
//! with timeout protections and non-blocking background execution.

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::{
    audit::models::AuditEvent,
    config::{SiemConfig, SiemTargetConfig},
};

/// High-throughput dispatcher exporting audit events to configured SIEM destinations.
#[derive(Clone)]
pub struct SiemDispatcher {
    client: Client,
    targets: Vec<SiemTargetConfig>,
}

impl SiemDispatcher {
    /// Creates a new `SiemDispatcher` given the optional `SiemConfig`.
    pub fn new(config: Option<SiemConfig>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let targets = config.map(|c| c.targets).unwrap_or_default();
        Self { client, targets }
    }

    /// Dispatches a batch of signed audit records to all configured SIEM destinations.
    pub async fn dispatch_batch(&self, events: &[AuditEvent]) {
        if self.targets.is_empty() || events.is_empty() {
            return;
        }

        for target in &self.targets {
            match target {
                SiemTargetConfig::Webhook {
                    url,
                    auth_header,
                    headers,
                } => {
                    let mut req = self.client.post(url).json(&events);
                    if let Some(ref auth) = auth_header {
                        req = req.header("Authorization", auth);
                    }
                    for (k, v) in headers {
                        req = req.header(k, v);
                    }

                    match req.send().await {
                        Ok(res) if res.status().is_success() => {
                            info!(target = %url, count = events.len(), "successfully exported audit events to webhook SIEM");
                        }
                        Ok(res) => {
                            warn!(target = %url, status = %res.status(), "webhook SIEM rejected audit export batch");
                        }
                        Err(err) => {
                            error!(target = %url, error = %err, "failed to send audit batch to webhook SIEM");
                        }
                    }
                }
                SiemTargetConfig::SplunkHec {
                    url,
                    token,
                    index,
                    source,
                } => {
                    // Splunk HEC expects a stream of JSON objects with { time, event, index, source }
                    let mut body = String::new();
                    for event in events {
                        let epoch_secs = (event.timestamp_ns as f64) / 1_000_000_000.0;
                        let mut hec_entry = json!({
                            "time": epoch_secs,
                            "event": event,
                            "sourcetype": "warmplane:audit",
                        });
                        if let Some(ref idx) = index {
                            hec_entry["index"] = json!(idx);
                        }
                        if let Some(ref src) = source {
                            hec_entry["source"] = json!(src);
                        }
                        if let Ok(line) = serde_json::to_string(&hec_entry) {
                            body.push_str(&line);
                            body.push('\n');
                        }
                    }

                    let req = self
                        .client
                        .post(url)
                        .header("Authorization", format!("Splunk {}", token))
                        .header("Content-Type", "application/json")
                        .body(body);

                    match req.send().await {
                        Ok(res) if res.status().is_success() => {
                            info!(target = %url, count = events.len(), "successfully exported audit events to Splunk HEC");
                        }
                        Ok(res) => {
                            warn!(target = %url, status = %res.status(), "Splunk HEC rejected audit export batch");
                        }
                        Err(err) => {
                            error!(target = %url, error = %err, "failed to send audit batch to Splunk HEC");
                        }
                    }
                }
            }
        }
    }
}
