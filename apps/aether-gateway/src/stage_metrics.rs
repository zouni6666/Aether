use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

use aether_runtime::{MetricKind, MetricLabel, MetricSample};
use serde_json::{Map, Value};

const BUCKETS_MS: [u64; 12] = [1, 5, 10, 25, 50, 100, 250, 500, 1_000, 2_500, 5_000, 10_000];

const STAGES: [&str; 56] = [
    "frontdoor_handler_queue",
    "frontdoor_admission",
    "frontdoor_context",
    "frontdoor_body_buffer",
    "frontdoor_owner_forward",
    "frontdoor_auth_model",
    "frontdoor_rpm",
    "frontdoor_local_ai_public",
    "frontdoor_execute_stream",
    "frontdoor_execute_sync",
    "stream_candidate_slot",
    "stream_path_step",
    "stream_candidate_next",
    "stream_candidate_source_next",
    "stream_candidate_plan_build",
    "stream_candidate_payload_parts",
    "stream_candidate_proxy",
    "stream_candidate_report_context",
    "stream_candidate_decision_build",
    "openai_chat_payload_parts_prepare",
    "openai_chat_payload_model_directives",
    "openai_chat_payload_redaction",
    "chat_pii_redaction_request_cache_hit",
    "chat_pii_redaction_runtime_config",
    "chat_pii_redaction_feature_settings",
    "chat_pii_redaction_mask_body",
    "openai_chat_payload_auth_prepare",
    "openai_chat_payload_body_build",
    "candidate_page_load",
    "candidate_page_resolve",
    "pool_cursor_next_key",
    "pool_score_load",
    "pool_score_key_rows",
    "pool_runtime_state",
    "candidate_transport_snapshot",
    "candidate_resolution_core",
    "candidate_resolution_transport_read",
    "candidate_resolution_rank",
    "direct_reqwest_client_prewarm",
    "stream_candidate_execute",
    "stream_candidate_unused",
    "stream_usage_pending",
    "stream_provider_in_flight",
    "stream_upstream_target_admission",
    "stream_upstream_headers",
    "stream_first_frame",
    "stream_first_data",
    "stream_response_policy",
    "stream_response_ready",
    "stream_total",
    "direct_passthrough_upstream_body_first",
    "direct_passthrough_first_client_send",
    "direct_passthrough_body_send_wait",
    "direct_passthrough_body_recv_first",
    "direct_build_body",
    "direct_send_headers",
];

const TRACE_STAGE_CAPACITY: usize = 16;
const STAGE_TRACE_MODE_ENV: &str = "AETHER_GATEWAY_STAGE_TRACE_MODE";
const STAGE_TRACE_SLOW_MS_ENV: &str = "AETHER_GATEWAY_STAGE_TRACE_SLOW_MS";
const STAGE_TRACE_SAMPLE_RATE_ENV: &str = "AETHER_GATEWAY_STAGE_TRACE_SAMPLE_RATE";
const DEFAULT_STAGE_TRACE_SLOW_MS: u64 = 1_000;

static METRICS: LazyLock<Vec<StageMetric>> =
    LazyLock::new(|| STAGES.iter().map(|stage| StageMetric::new(stage)).collect());
static STAGE_TRACE_CONFIG: LazyLock<RequestStageTraceConfig> =
    LazyLock::new(read_stage_trace_config);
static STAGE_TRACE_SAMPLE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct StageMetric {
    stage: &'static str,
    count: AtomicU64,
    sum_ms: AtomicU64,
    max_ms: AtomicU64,
    buckets: Vec<AtomicU64>,
}

impl StageMetric {
    fn new(stage: &'static str) -> Self {
        Self {
            stage,
            count: AtomicU64::new(0),
            sum_ms: AtomicU64::new(0),
            max_ms: AtomicU64::new(0),
            buckets: BUCKETS_MS.iter().map(|_| AtomicU64::new(0)).collect(),
        }
    }

    fn observe(&self, elapsed_ms: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_ms.fetch_add(elapsed_ms, Ordering::Relaxed);
        update_max(&self.max_ms, elapsed_ms);
        for (index, bucket) in BUCKETS_MS.iter().enumerate() {
            if elapsed_ms <= *bucket {
                self.buckets[index].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn samples(&self) -> Vec<MetricSample> {
        let stage_label = vec![MetricLabel::new("stage", self.stage)];
        let mut samples = vec![
            MetricSample::new(
                "gateway_stage_latency_count",
                "Number of gateway stage latency observations.",
                MetricKind::Counter,
                self.count.load(Ordering::Relaxed),
            )
            .with_labels(stage_label.clone()),
            MetricSample::new(
                "gateway_stage_latency_sum_ms",
                "Total gateway stage latency in milliseconds.",
                MetricKind::Counter,
                self.sum_ms.load(Ordering::Relaxed),
            )
            .with_labels(stage_label.clone()),
            MetricSample::new(
                "gateway_stage_latency_max_ms",
                "Maximum observed gateway stage latency in milliseconds since process start.",
                MetricKind::Gauge,
                self.max_ms.load(Ordering::Relaxed),
            )
            .with_labels(stage_label.clone()),
        ];
        for (index, upper_bound_ms) in BUCKETS_MS.iter().enumerate() {
            samples.push(
                MetricSample::new(
                    "gateway_stage_latency_bucket",
                    "Cumulative gateway stage latency observations less than or equal to the bucket upper bound.",
                    MetricKind::Counter,
                    self.buckets[index].load(Ordering::Relaxed),
                )
                .with_labels(vec![
                    MetricLabel::new("stage", self.stage),
                    MetricLabel::new("le_ms", upper_bound_ms.to_string()),
                ]),
            );
        }
        samples
    }
}

pub(crate) fn observe_gateway_stage_ms(stage: &'static str, elapsed_ms: u64) {
    if let Some(metric) = METRICS.iter().find(|metric| metric.stage == stage) {
        metric.observe(elapsed_ms);
    }
}

pub(crate) fn gateway_stage_metric_samples() -> Vec<MetricSample> {
    METRICS.iter().flat_map(StageMetric::samples).collect()
}

fn update_max(max: &AtomicU64, value: u64) {
    let mut current = max.load(Ordering::Relaxed);
    while value > current {
        match max.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequestStageTrace {
    mode: RequestStageTraceMode,
    slow_ms: u64,
    sampled: bool,
    stages: Vec<(&'static str, u64)>,
}

#[derive(Debug, Clone, Copy)]
struct RequestStageTraceConfig {
    mode: RequestStageTraceMode,
    slow_ms: u64,
    sample_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestStageTraceMode {
    Off,
    Slow,
    Sample,
    All,
}

impl RequestStageTrace {
    pub(crate) fn from_env() -> Self {
        let config = *STAGE_TRACE_CONFIG;
        let sampled = config.sample_rate > 0.0 && random_unit_sample() < config.sample_rate;
        Self {
            mode: config.mode,
            slow_ms: config.slow_ms,
            sampled,
            stages: Vec::with_capacity(TRACE_STAGE_CAPACITY),
        }
    }

    pub(crate) fn observe(&mut self, stage: &'static str, elapsed_ms: u64) {
        if self.mode == RequestStageTraceMode::Off {
            return;
        }
        if let Some((_, existing)) = self
            .stages
            .iter_mut()
            .find(|(existing_stage, _)| *existing_stage == stage)
        {
            *existing = elapsed_ms;
            return;
        }
        if self.stages.len() < TRACE_STAGE_CAPACITY {
            self.stages.push((stage, elapsed_ms));
        }
    }

    pub(crate) fn into_metadata_value(self, fallback_elapsed_ms: Option<u64>) -> Option<Value> {
        if self.mode == RequestStageTraceMode::Off || self.stages.is_empty() {
            return None;
        }

        let max_observed_ms = self
            .stages
            .iter()
            .map(|(_, elapsed_ms)| *elapsed_ms)
            .max()
            .unwrap_or(0);
        let fallback_elapsed_ms = fallback_elapsed_ms.unwrap_or(0);
        let slow = max_observed_ms.max(fallback_elapsed_ms) >= self.slow_ms;
        let should_emit = match self.mode {
            RequestStageTraceMode::Off => false,
            RequestStageTraceMode::Slow => slow || self.sampled,
            RequestStageTraceMode::Sample => self.sampled,
            RequestStageTraceMode::All => true,
        };
        if !should_emit {
            return None;
        }

        let mut object = Map::new();
        for (stage, elapsed_ms) in self.stages {
            object.insert(stage.to_string(), Value::from(elapsed_ms));
        }
        Some(Value::Object(object))
    }
}

pub(crate) fn observe_gateway_stage_trace_ms(
    trace: &mut RequestStageTrace,
    stage: &'static str,
    elapsed_ms: u64,
) {
    observe_gateway_stage_ms(stage, elapsed_ms);
    trace.observe(stage, elapsed_ms);
}

pub(crate) fn attach_stage_trace_to_report_context(
    report_context: Option<Value>,
    stage_timings_ms: Option<Value>,
) -> Option<Value> {
    let Some(stage_timings_ms) = stage_timings_ms else {
        return report_context;
    };

    let mut object = match report_context {
        Some(Value::Object(object)) => object,
        Some(other) => Map::from_iter([("seed".to_string(), other)]),
        None => Map::new(),
    };
    object.insert("stage_timings_ms".to_string(), stage_timings_ms);
    Some(Value::Object(object))
}

fn read_stage_trace_config() -> RequestStageTraceConfig {
    RequestStageTraceConfig {
        mode: read_stage_trace_mode(),
        slow_ms: read_stage_trace_slow_ms(),
        sample_rate: read_stage_trace_sample_rate(),
    }
}

fn read_stage_trace_mode() -> RequestStageTraceMode {
    match std::env::var(STAGE_TRACE_MODE_ENV)
        .ok()
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("all") => RequestStageTraceMode::All,
        Some("sample") => RequestStageTraceMode::Sample,
        Some("off") | Some("none") | Some("disabled") | Some("0") => RequestStageTraceMode::Off,
        _ => RequestStageTraceMode::Slow,
    }
}

fn read_stage_trace_slow_ms() -> u64 {
    std::env::var(STAGE_TRACE_SLOW_MS_ENV)
        .ok()
        .as_deref()
        .map(str::trim)
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_STAGE_TRACE_SLOW_MS)
}

fn read_stage_trace_sample_rate() -> f64 {
    std::env::var(STAGE_TRACE_SAMPLE_RATE_ENV)
        .ok()
        .as_deref()
        .map(str::trim)
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

fn random_unit_sample() -> f64 {
    let mut value = STAGE_TRACE_SAMPLE_COUNTER
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    let mixed = value.wrapping_mul(0x2545_f491_4f6c_dd1d);
    (mixed as f64) / (u64::MAX as f64)
}
