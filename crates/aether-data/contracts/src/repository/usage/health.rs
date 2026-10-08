use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthObservationObjectKind {
    ApiFormat,
    Model,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthObservationQuery {
    pub from_unix_ms: u64,
    pub to_unix_ms: u64,
    pub object_kind: HealthObservationObjectKind,
    /// None means all authorized administrative objects; Some(empty) means no objects.
    pub object_values: Option<Vec<String>>,
    pub segments: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthObservationMetrics {
    pub request_count: u64,
    pub succeeded_count: u64,
    pub failed_count: u64,
    pub in_progress_count: u64,
    pub cancelled_count: u64,
    pub service_succeeded_count: u64,
    pub service_failed_count: u64,
    pub excluded_count: u64,
    pub unknown_failure_count: u64,
    pub attempt_succeeded_count: u64,
    pub attempt_failed_count: u64,
    pub attempt_in_progress_count: u64,
    pub attempt_cancelled_count: u64,
    pub latency_sum_ms: f64,
    pub latency_sample_count: u64,
    pub last_request_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthObservationBucket {
    pub from_unix_ms: u64,
    pub to_unix_ms: u64,
    pub metrics: HealthObservationMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthObservationObject {
    pub object_value: String,
    pub metrics: HealthObservationMetrics,
    pub timeline: Vec<HealthObservationBucket>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HealthObservationSummary {
    pub overall: HealthObservationMetrics,
    pub objects: Vec<HealthObservationObject>,
    pub timeline: Vec<HealthObservationBucket>,
    pub data_through_unix_ms: Option<u64>,
}
