use std::sync::Arc;
use std::time::{Duration, Instant};

use aether_contracts::ExecutionPlan;
use aether_runtime::{ConcurrencyGate, ConcurrencyPermit, MetricKind, MetricLabel, MetricSample};
use dashmap::DashMap;
use tokio::time::timeout;
use url::Url;

use crate::stage_metrics::observe_gateway_stage_ms;
use crate::GatewayError;

const GATE_NAME: &str = "gateway_upstream_target";
const DEFAULT_METRIC_TARGET_LIMIT: usize = 32;
const METRIC_TARGET_LIMIT_ENV: &str = "AETHER_GATEWAY_UPSTREAM_TARGET_GATE_METRIC_LIMIT";

#[derive(Debug)]
pub(crate) struct UpstreamTargetAdmission {
    limit: Option<usize>,
    queue_budget: Duration,
    gates: DashMap<String, Arc<ConcurrencyGate>>,
}

#[derive(Debug)]
pub(crate) struct UpstreamTargetAdmissionPermit {
    _permit: ConcurrencyPermit,
}

impl UpstreamTargetAdmission {
    pub(crate) fn new(limit: Option<usize>, queue_budget: Duration) -> Self {
        Self {
            limit,
            queue_budget,
            gates: DashMap::new(),
        }
    }

    pub(crate) async fn acquire(
        &self,
        plan: &ExecutionPlan,
        trace_id: &str,
    ) -> Result<Option<UpstreamTargetAdmissionPermit>, GatewayError> {
        let Some(limit) = self.limit else {
            return Ok(None);
        };
        let key = upstream_target_key(plan);
        let gate = self
            .gates
            .entry(key.clone())
            .or_insert_with(|| Arc::new(ConcurrencyGate::new(GATE_NAME, limit)))
            .clone();
        let started_at = Instant::now();
        let permit = match timeout(self.queue_budget, gate.acquire()).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(err)) => return Err(GatewayError::Internal(err.to_string())),
            Err(_) => {
                tracing::debug!(
                    event_name = "gateway_upstream_target_admission_timeout",
                    log_type = "ops",
                    trace_id,
                    target = key.as_str(),
                    limit,
                    queue_budget_ms = self.queue_budget.as_millis() as u64,
                    "gateway upstream target admission gate timed out"
                );
                return Err(GatewayError::AdmissionTimeout {
                    trace_id: trace_id.to_string(),
                    gate: GATE_NAME,
                    queue_budget_ms: self.queue_budget.as_millis() as u64,
                });
            }
        };
        observe_gateway_stage_ms(
            "stream_upstream_target_admission",
            started_at.elapsed().as_millis() as u64,
        );
        Ok(Some(UpstreamTargetAdmissionPermit { _permit: permit }))
    }

    pub(crate) fn metric_samples(&self) -> Vec<MetricSample> {
        let mut samples = vec![MetricSample::new(
            "upstream_target_gate_active_targets",
            "Number of upstream targets currently tracked by the gateway upstream target admission gates.",
            MetricKind::Gauge,
            self.gates.len() as u64,
        )];

        let Some(limit) = self.limit else {
            return samples;
        };

        samples.push(MetricSample::new(
            "upstream_target_gate_limit",
            "Configured per-upstream-target admission gate limit.",
            MetricKind::Gauge,
            limit as u64,
        ));

        let mut snapshots = self
            .gates
            .iter()
            .map(|entry| {
                let snapshot = entry.value().snapshot();
                (
                    entry.key().clone(),
                    snapshot.in_flight,
                    snapshot.available_permits,
                    snapshot.high_watermark,
                    snapshot.rejected,
                )
            })
            .collect::<Vec<_>>();
        snapshots.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| right.3.cmp(&left.3))
                .then_with(|| right.4.cmp(&left.4))
        });

        let metric_target_limit = upstream_target_metric_limit();
        for (target, in_flight, available, high_watermark, rejected) in
            snapshots.into_iter().take(metric_target_limit)
        {
            let labels = vec![MetricLabel::new("target", target)];
            samples.push(
                MetricSample::new(
                    "upstream_target_gate_in_flight",
                    "Current number of in-flight operations for an upstream target admission gate.",
                    MetricKind::Gauge,
                    in_flight as u64,
                )
                .with_labels(labels.clone()),
            );
            samples.push(
                MetricSample::new(
                    "upstream_target_gate_available_permits",
                    "Currently available permits for an upstream target admission gate.",
                    MetricKind::Gauge,
                    available as u64,
                )
                .with_labels(labels.clone()),
            );
            samples.push(
                MetricSample::new(
                    "upstream_target_gate_high_watermark",
                    "Highest observed in-flight count for an upstream target admission gate.",
                    MetricKind::Gauge,
                    high_watermark as u64,
                )
                .with_labels(labels.clone()),
            );
            samples.push(
                MetricSample::new(
                    "upstream_target_gate_rejected_total",
                    "Number of operations rejected by an upstream target admission gate.",
                    MetricKind::Counter,
                    rejected,
                )
                .with_labels(labels),
            );
        }

        samples
    }
}

pub(crate) fn upstream_target_key(plan: &ExecutionPlan) -> String {
    let parsed = Url::parse(plan.url.as_str()).ok();
    let Some(url) = parsed else {
        return fallback_target_key(plan);
    };
    let scheme = url.scheme().to_ascii_lowercase();
    let Some(host) = url.host_str().map(|host| host.to_ascii_lowercase()) else {
        return fallback_target_key(plan);
    };
    let port = url
        .port_or_known_default()
        .map(|port| port.to_string())
        .unwrap_or_else(|| "-".to_string());
    let proxy = plan
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.url.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    format!("{scheme}://{host}:{port}|proxy={proxy}")
}

fn fallback_target_key(plan: &ExecutionPlan) -> String {
    format!(
        "unparsed|provider={}|endpoint={}|url={}",
        plan.provider_id, plan.endpoint_id, plan.url
    )
}

fn upstream_target_metric_limit() -> usize {
    std::env::var(METRIC_TARGET_LIMIT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_METRIC_TARGET_LIMIT)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use aether_contracts::{ExecutionPlan, RequestBody};
    use serde_json::json;

    use super::*;

    fn test_plan(url: &str) -> ExecutionPlan {
        ExecutionPlan {
            request_id: "req-upstream-target".to_string(),
            candidate_id: Some("cand-upstream-target".to_string()),
            provider_name: Some("provider".to_string()),
            provider_id: "provider_id".to_string(),
            endpoint_id: "endpoint_id".to_string(),
            key_id: "key_id".to_string(),
            method: "POST".to_string(),
            url: url.to_string(),
            headers: Default::default(),
            content_type: Some("application/json".to_string()),
            content_encoding: None,
            body: RequestBody::from_json(json!({"stream": true})),
            stream: true,
            client_api_format: "openai".to_string(),
            provider_api_format: "openai".to_string(),
            model_name: Some("model".to_string()),
            proxy: None,
            transport_profile: None,
            timeouts: None,
        }
    }

    #[test]
    fn upstream_target_key_ignores_path_and_query() {
        let left = test_plan("http://127.0.0.1:18181/v1/chat/completions?x=1");
        let right = test_plan("http://127.0.0.1:18181/v1/responses");

        assert_eq!(upstream_target_key(&left), upstream_target_key(&right));
    }

    #[tokio::test]
    async fn acquire_times_out_when_target_gate_is_saturated() {
        let admission = UpstreamTargetAdmission::new(Some(1), Duration::from_millis(1));
        let plan = test_plan("http://127.0.0.1:18181/v1/chat/completions");
        let _first = admission
            .acquire(&plan, "trace-upstream-target")
            .await
            .expect("first acquire should succeed")
            .expect("gate enabled");

        let err = admission
            .acquire(&plan, "trace-upstream-target")
            .await
            .expect_err("second acquire should time out");

        assert!(matches!(
            err,
            GatewayError::AdmissionTimeout {
                gate: "gateway_upstream_target",
                ..
            }
        ));
    }
}
