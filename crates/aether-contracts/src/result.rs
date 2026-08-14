use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ExecutionError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionResponseObservation {
    pub request_started_at_unix_ms: u64,
    pub response_headers_observed_at_unix_ms: u64,
    pub request_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionTelemetry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttfb_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_body: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_bytes_b64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    pub request_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,
    pub status_code: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_observation: Option<ExecutionResponseObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<ResponseBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<ExecutionTelemetry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecutionError>,
}
