use crate::api::response::build_local_http_error_response;
use crate::control::GatewayPublicRequestContext;
use crate::headers::RequestBodyNormalizationError;
use crate::{AppState, GatewayError};
use axum::body::{to_bytes, Body, Bytes};
use axum::http::{self, Response};
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{info, warn};

const REQUEST_BODY_READ_TIMEOUT_DETAIL: &str =
    "Request body read timed out before the gateway could route the request";
const REQUEST_BODY_READ_FAILED_DETAIL: &str = "Failed to read request body";

#[derive(Debug, Clone)]
pub(super) struct RequestBodyBufferPolicy {
    max_bytes: u64,
    read_timeout: Duration,
    queue_timeout: Duration,
    budget_bytes: usize,
    budget: Arc<Semaphore>,
}

impl RequestBodyBufferPolicy {
    pub(super) fn from_state(state: &AppState) -> Self {
        Self {
            max_bytes: crate::headers::max_request_body_bytes(),
            read_timeout: state.frontdoor_runtime_guards.request_body_read_timeout,
            queue_timeout: state.frontdoor_runtime_guards.internal_gate_queue_budget,
            budget_bytes: state
                .frontdoor_runtime_guards
                .request_body_buffer_budget_bytes,
            budget: Arc::clone(&state.request_body_buffer_budget),
        }
    }

    #[cfg(test)]
    pub(super) fn for_tests(max_bytes: u64, read_timeout: Duration) -> Self {
        let budget_bytes = usize::try_from(max_bytes)
            .unwrap_or(usize::MAX)
            .max(crate::state::REQUEST_BODY_BUFFER_PERMIT_BYTES);
        Self {
            max_bytes,
            read_timeout,
            queue_timeout: read_timeout,
            budget_bytes,
            budget: Arc::new(Semaphore::new(
                budget_bytes.saturating_add(crate::state::REQUEST_BODY_BUFFER_PERMIT_BYTES - 1)
                    / crate::state::REQUEST_BODY_BUFFER_PERMIT_BYTES,
            )),
        }
    }

    #[cfg(test)]
    pub(super) fn for_tests_with_budget(
        max_bytes: u64,
        read_timeout: Duration,
        queue_timeout: Duration,
        budget_bytes: usize,
        budget: Arc<Semaphore>,
    ) -> Self {
        Self {
            max_bytes,
            read_timeout,
            queue_timeout,
            budget_bytes,
            budget,
        }
    }
}

#[derive(Debug)]
pub(super) enum RequestBodyBufferError {
    Normalization(RequestBodyNormalizationError),
    TooLarge {
        limit_bytes: u64,
    },
    Overloaded {
        requested_bytes: usize,
        budget_bytes: usize,
        timeout_ms: u64,
    },
    Timeout {
        timeout_ms: u64,
    },
    ReadFailed {
        message: String,
    },
}

impl RequestBodyBufferError {
    pub(super) fn http_status(&self) -> http::StatusCode {
        match self {
            Self::Normalization(error) => error.http_status(),
            Self::TooLarge { .. } => http::StatusCode::PAYLOAD_TOO_LARGE,
            Self::Overloaded { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout { .. } => http::StatusCode::REQUEST_TIMEOUT,
            Self::ReadFailed { .. } => http::StatusCode::BAD_REQUEST,
        }
    }

    fn client_message(&self) -> String {
        match self {
            Self::Normalization(error) => error.client_message(),
            Self::TooLarge { limit_bytes } => format!("Request body exceeds {limit_bytes} bytes"),
            Self::Overloaded { .. } => {
                "Request body buffering capacity is temporarily exhausted".to_string()
            }
            Self::Timeout { .. } => REQUEST_BODY_READ_TIMEOUT_DETAIL.to_string(),
            Self::ReadFailed { .. } => REQUEST_BODY_READ_FAILED_DETAIL.to_string(),
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            Self::Normalization(error) => match error {
                RequestBodyNormalizationError::UnsupportedContentEncoding(_) => {
                    "unsupported_content_encoding"
                }
                RequestBodyNormalizationError::DecodeFailed { .. } => "decode_failed",
                RequestBodyNormalizationError::DecompressedBodyTooLarge { .. } => {
                    "decompressed_body_too_large"
                }
                RequestBodyNormalizationError::RequestBodyTooLarge { .. } => {
                    "request_body_too_large"
                }
            },
            Self::TooLarge { .. } => "request_body_too_large",
            Self::Overloaded { .. } => "request_body_buffer_overloaded",
            Self::Timeout { .. } => "request_body_read_timeout",
            Self::ReadFailed { .. } => "request_body_read_failed",
        }
    }
}

fn request_body_buffer_reservation_bytes(headers: &http::HeaderMap, max_bytes: u64) -> usize {
    let max_bytes = usize::try_from(max_bytes).unwrap_or(usize::MAX);
    let encoded = headers
        .get(http::header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .is_some_and(|value| !value.is_empty() && !value.eq_ignore_ascii_case("identity"));
    if encoded {
        return max_bytes;
    }
    headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .map(|value| value.min(max_bytes))
        .unwrap_or(max_bytes)
}

fn request_body_buffer_reservation_permits(reservation_bytes: usize) -> u32 {
    let permits = reservation_bytes
        .max(1)
        .saturating_add(crate::state::REQUEST_BODY_BUFFER_PERMIT_BYTES - 1)
        / crate::state::REQUEST_BODY_BUFFER_PERMIT_BYTES;
    u32::try_from(permits).unwrap_or(u32::MAX).max(1)
}

pub(super) async fn buffer_and_normalize_request_body(
    request_body: &mut Option<Body>,
    headers: &mut http::HeaderMap,
    body_owner_expectation: &'static str,
    trace_id: &str,
    method: &http::Method,
    path_and_query: &str,
    phase: &'static str,
    policy: RequestBodyBufferPolicy,
) -> Result<Bytes, RequestBodyBufferError> {
    if let Err(err) =
        crate::headers::check_request_content_length_with_limit(headers, policy.max_bytes)
    {
        return Err(RequestBodyBufferError::Normalization(err));
    }

    let reservation_bytes = request_body_buffer_reservation_bytes(headers, policy.max_bytes);
    let reservation_permits = request_body_buffer_reservation_permits(reservation_bytes);
    let queue_timeout_ms = policy.queue_timeout.as_millis() as u64;
    let _budget_permit = match tokio::time::timeout(
        policy.queue_timeout,
        Arc::clone(&policy.budget).acquire_many_owned(reservation_permits),
    )
    .await
    {
        Ok(Ok(permit)) => permit,
        Ok(Err(_)) | Err(_) => {
            return Err(RequestBodyBufferError::Overloaded {
                requested_bytes: reservation_bytes,
                budget_bytes: policy.budget_bytes,
                timeout_ms: queue_timeout_ms,
            });
        }
    };

    let read_started_at = Instant::now();
    let timeout_ms = policy.read_timeout.as_millis() as u64;
    info!(
        event_name = "frontdoor_request_body_buffer_started",
        log_type = "event",
        trace_id,
        method = %method,
        path = %path_and_query,
        phase,
        max_body_bytes = policy.max_bytes,
        reserved_body_bytes = reservation_bytes,
        body_buffer_budget_bytes = policy.budget_bytes,
        timeout_ms,
        "gateway started buffering request body"
    );

    let body_limit = usize::try_from(policy.max_bytes).unwrap_or(usize::MAX);
    let body = match tokio::time::timeout(
        policy.read_timeout,
        to_bytes(
            request_body.take().expect(body_owner_expectation),
            body_limit,
        ),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(err)) if request_body_collection_exceeded_limit(&err) => {
            return Err(RequestBodyBufferError::TooLarge {
                limit_bytes: policy.max_bytes,
            });
        }
        Ok(Err(err)) => {
            return Err(RequestBodyBufferError::ReadFailed {
                message: err.to_string(),
            });
        }
        Err(_) => {
            return Err(RequestBodyBufferError::Timeout { timeout_ms });
        }
    };

    let normalized = crate::headers::normalize_request_body_headers_and_bytes_with_limit(
        headers,
        body,
        policy.max_bytes,
    )
    .map_err(RequestBodyBufferError::Normalization)?;
    info!(
        event_name = "frontdoor_request_body_buffer_completed",
        log_type = "event",
        trace_id,
        method = %method,
        path = %path_and_query,
        phase,
        body_bytes = normalized.len(),
        elapsed_ms = read_started_at.elapsed().as_millis() as u64,
        "gateway completed buffering request body"
    );
    Ok(normalized)
}

fn request_body_collection_exceeded_limit(error: &(dyn StdError + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if error.to_string().contains("length limit exceeded") {
            return true;
        }
        current = error.source();
    }
    false
}

pub(super) fn build_request_body_buffer_error_response(
    trace_id: &str,
    request_context: &GatewayPublicRequestContext,
    error: &RequestBodyBufferError,
) -> Result<Response<Body>, GatewayError> {
    warn!(
        event_name = "frontdoor_request_body_buffer_failed",
        log_type = "ops",
        trace_id,
        method = %request_context.request_method,
        path = %request_context.request_path_and_query(),
        status_code = error.http_status().as_u16(),
        reason = error.reason(),
        detail = %error.client_message(),
        read_error = match error {
            RequestBodyBufferError::ReadFailed { message } => message.as_str(),
            _ => "",
        },
        buffer_requested_bytes = match error {
            RequestBodyBufferError::Overloaded { requested_bytes, .. } => *requested_bytes,
            _ => 0,
        },
        buffer_budget_bytes = match error {
            RequestBodyBufferError::Overloaded { budget_bytes, .. } => *budget_bytes,
            _ => 0,
        },
        buffer_queue_timeout_ms = match error {
            RequestBodyBufferError::Overloaded { timeout_ms, .. } => *timeout_ms,
            _ => 0,
        },
        "gateway rejected request body before local execution planning"
    );
    build_local_http_error_response(
        trace_id,
        request_context.control_decision.as_ref(),
        error.http_status(),
        error.client_message().as_str(),
    )
}
