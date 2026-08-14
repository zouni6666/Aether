use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ExecutionError, ExecutionResponseObservation, ExecutionStreamTerminalSummary,
    ExecutionTelemetry,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamFrameType {
    Headers,
    Data,
    Error,
    Telemetry,
    Eof,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamFramePayload {
    Headers {
        status_code: u16,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_observation: Option<ExecutionResponseObservation>,
    },
    Data {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        chunk_b64: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    Error {
        error: ExecutionError,
    },
    Telemetry {
        telemetry: ExecutionTelemetry,
    },
    Eof {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<ExecutionStreamTerminalSummary>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamFrame {
    #[serde(rename = "type")]
    pub frame_type: StreamFrameType,
    pub payload: StreamFramePayload,
}

impl StreamFrame {
    pub fn eof() -> Self {
        Self::eof_with_summary(None)
    }

    pub fn eof_with_summary(summary: Option<ExecutionStreamTerminalSummary>) -> Self {
        Self {
            frame_type: StreamFrameType::Eof,
            payload: StreamFramePayload::Eof { summary },
        }
    }
}
