use std::time::Duration;

use serde_json::Value;

use crate::execution_runtime::MAX_STREAM_PREFETCH_BYTES;

const ANTHROPIC_PRECOMMIT_MAX_WAIT: Duration = Duration::from_millis(750);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StreamCommitPolicy {
    ResponseHeaders,
    FirstClassifiedBody,
    FirstAnthropicSemanticEvent {
        max_bytes: usize,
        max_wait: Duration,
    },
}

impl StreamCommitPolicy {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn for_response(
        has_direct_finalize: bool,
        content_type: Option<&str>,
        provider_api_format: &str,
        client_api_format: &str,
        has_private_stream_normalizer: bool,
        has_local_stream_rewriter: bool,
        force_prefetch: bool,
    ) -> Self {
        if !has_direct_finalize {
            return Self::FirstClassifiedBody;
        }

        if force_prefetch {
            return Self::FirstClassifiedBody;
        }

        let content_type = content_type
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if content_type.contains("text/event-stream") {
            if provider_api_format.eq_ignore_ascii_case("claude:messages")
                && provider_api_format.eq_ignore_ascii_case(client_api_format)
                && !has_private_stream_normalizer
                && !has_local_stream_rewriter
            {
                return Self::FirstAnthropicSemanticEvent {
                    max_bytes: MAX_STREAM_PREFETCH_BYTES,
                    max_wait: ANTHROPIC_PRECOMMIT_MAX_WAIT,
                };
            }
            return Self::ResponseHeaders;
        }

        if has_private_stream_normalizer || has_local_stream_rewriter {
            return Self::FirstClassifiedBody;
        }

        if !provider_api_format.eq_ignore_ascii_case(client_api_format) {
            return Self::FirstClassifiedBody;
        }

        if content_type.is_empty() {
            return Self::ResponseHeaders;
        }

        if content_type.contains("json") || content_type.ends_with("+json") {
            Self::FirstClassifiedBody
        } else {
            Self::ResponseHeaders
        }
    }

    pub(super) const fn commits_on_response_headers(self) -> bool {
        matches!(self, Self::ResponseHeaders)
    }

    pub(super) const fn requires_bounded_frame_wait(self) -> bool {
        matches!(self, Self::FirstAnthropicSemanticEvent { .. })
    }

    pub(super) const fn max_precommit_wait(self) -> Option<Duration> {
        match self {
            Self::FirstAnthropicSemanticEvent { max_wait, .. } => Some(max_wait),
            Self::ResponseHeaders | Self::FirstClassifiedBody => None,
        }
    }

    pub(super) const fn is_native_anthropic(self) -> bool {
        matches!(self, Self::FirstAnthropicSemanticEvent { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StreamCommitState {
    Uncommitted,
    Committed,
    Terminal,
}

#[derive(Debug, PartialEq)]
pub(super) enum StreamPrecommitObservation {
    Pending,
    Commit,
    UpstreamError { status_code: u16, body_json: Value },
}

#[derive(Debug)]
pub(super) struct StreamCommitGate {
    policy: StreamCommitPolicy,
    state: StreamCommitState,
    observed_bytes: usize,
    anthropic: AnthropicSsePrecommitInspector,
}

impl StreamCommitGate {
    pub(super) fn new(policy: StreamCommitPolicy) -> Self {
        let state = if policy.commits_on_response_headers() {
            StreamCommitState::Committed
        } else {
            StreamCommitState::Uncommitted
        };
        Self {
            policy,
            state,
            observed_bytes: 0,
            anthropic: AnthropicSsePrecommitInspector::default(),
        }
    }

    pub(super) const fn state(&self) -> StreamCommitState {
        self.state
    }

    pub(super) const fn is_uncommitted(&self) -> bool {
        matches!(self.state, StreamCommitState::Uncommitted)
    }

    pub(super) fn observe_provider_bytes(&mut self, chunk: &[u8]) -> StreamPrecommitObservation {
        if self.state != StreamCommitState::Uncommitted {
            return StreamPrecommitObservation::Commit;
        }

        let StreamCommitPolicy::FirstAnthropicSemanticEvent { max_bytes, .. } = self.policy else {
            return StreamPrecommitObservation::Pending;
        };

        self.observed_bytes = self.observed_bytes.saturating_add(chunk.len());
        match self.anthropic.observe(chunk, max_bytes) {
            AnthropicSseObservation::Pending => {}
            AnthropicSseObservation::SemanticEvent => {
                self.state = StreamCommitState::Committed;
                return StreamPrecommitObservation::Commit;
            }
            AnthropicSseObservation::Error(body_json) => {
                self.state = StreamCommitState::Terminal;
                return StreamPrecommitObservation::UpstreamError {
                    status_code: anthropic_error_status_code(&body_json),
                    body_json,
                };
            }
        }

        if self.observed_bytes >= max_bytes {
            self.commit();
            StreamPrecommitObservation::Commit
        } else {
            StreamPrecommitObservation::Pending
        }
    }

    pub(super) fn commit(&mut self) {
        if self.state == StreamCommitState::Uncommitted {
            self.state = StreamCommitState::Committed;
        }
    }
}

#[derive(Debug)]
enum AnthropicSseObservation {
    Pending,
    SemanticEvent,
    Error(Value),
}

#[derive(Debug, Default)]
struct AnthropicSsePrecommitInspector {
    buffered: Vec<u8>,
}

impl AnthropicSsePrecommitInspector {
    fn observe(&mut self, chunk: &[u8], max_bytes: usize) -> AnthropicSseObservation {
        let remaining = max_bytes.saturating_sub(self.buffered.len());
        let truncated = chunk.len() > remaining;
        self.buffered
            .extend_from_slice(&chunk[..chunk.len().min(remaining)]);

        while let Some((record_end, separator_len)) = find_sse_record_boundary(&self.buffered) {
            let record = self.buffered[..record_end].to_vec();
            self.buffered.drain(..record_end + separator_len);
            match classify_anthropic_sse_record(&record) {
                AnthropicSseObservation::Pending => {}
                decision => return decision,
            }
        }

        if truncated {
            AnthropicSseObservation::SemanticEvent
        } else {
            AnthropicSseObservation::Pending
        }
    }
}

pub(super) fn find_sse_record_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    let mut cursor = 0;
    while cursor < buffer.len() {
        let (line_end, line_ending_len) = next_sse_line_ending(buffer, cursor)?;
        let next_line_start = line_end + line_ending_len;
        let Some((next_line_end, next_line_ending_len)) =
            next_sse_line_ending(buffer, next_line_start)
        else {
            return None;
        };
        if next_line_end == next_line_start {
            return Some((
                line_end,
                line_ending_len.saturating_add(next_line_ending_len),
            ));
        }
        cursor = next_line_start;
    }
    None
}

fn next_sse_line_ending(buffer: &[u8], start: usize) -> Option<(usize, usize)> {
    let relative = buffer
        .get(start..)?
        .iter()
        .position(|byte| matches!(byte, b'\r' | b'\n'))?;
    let index = start + relative;
    let ending_len = if buffer[index] == b'\r' && buffer.get(index + 1) == Some(&b'\n') {
        2
    } else {
        1
    };
    Some((index, ending_len))
}

fn classify_anthropic_sse_record(record: &[u8]) -> AnthropicSseObservation {
    let Ok(record) = std::str::from_utf8(record) else {
        return AnthropicSseObservation::Pending;
    };
    let normalized_record = record.replace("\r\n", "\n").replace('\r', "\n");
    let mut event_type = None;
    let mut data = String::new();
    for line in normalized_record.lines() {
        if line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            let value = value.trim();
            if !value.is_empty() {
                event_type = Some(value);
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }
    if data.trim().is_empty() {
        return AnthropicSseObservation::Pending;
    }

    let Ok(body_json) = serde_json::from_str::<Value>(data.trim()) else {
        return AnthropicSseObservation::Pending;
    };
    let payload_type = body_json.get("type").and_then(Value::as_str).map(str::trim);
    if event_type == Some("error") || payload_type == Some("error") {
        return AnthropicSseObservation::Error(body_json);
    }

    let semantic_type = match (event_type, payload_type) {
        (Some(event_type), Some(payload_type)) if event_type == payload_type => Some(event_type),
        (None, Some(payload_type)) => Some(payload_type),
        _ => None,
    };
    if semantic_type.is_some_and(is_anthropic_semantic_event_type) {
        AnthropicSseObservation::SemanticEvent
    } else {
        AnthropicSseObservation::Pending
    }
}

fn is_anthropic_semantic_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        "message_start"
            | "content_block_start"
            | "content_block_delta"
            | "content_block_stop"
            | "message_delta"
            | "message_stop"
    )
}

pub(super) fn anthropic_error_status_code(body_json: &Value) -> u16 {
    let error_type = body_json
        .get("error")
        .and_then(|error| error.get("type"))
        .or_else(|| body_json.get("type"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    match error_type {
        "invalid_request_error" => 400,
        "authentication_error" => 401,
        "permission_error" => 403,
        "not_found_error" => 404,
        "request_too_large" => 413,
        "rate_limit_error" => 429,
        "overloaded_error" => 529,
        _ => 500,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        anthropic_error_status_code, StreamCommitGate, StreamCommitPolicy, StreamCommitState,
        StreamPrecommitObservation,
    };

    fn native_anthropic_policy() -> StreamCommitPolicy {
        StreamCommitPolicy::FirstAnthropicSemanticEvent {
            max_bytes: 16_384,
            max_wait: Duration::from_millis(750),
        }
    }

    #[test]
    fn policy_selects_bounded_anthropic_gate_only_for_native_same_format_sse() {
        let native = StreamCommitPolicy::for_response(
            true,
            Some("text/event-stream; charset=utf-8"),
            "claude:messages",
            "claude:messages",
            false,
            false,
            false,
        );
        assert!(native.is_native_anthropic());
        assert_eq!(
            native.max_precommit_wait(),
            Some(Duration::from_millis(750))
        );
        assert!(StreamCommitPolicy::for_response(
            true,
            Some("text/event-stream"),
            "openai:chat",
            "claude:messages",
            false,
            false,
            false,
        )
        .commits_on_response_headers());
        assert!(StreamCommitPolicy::for_response(
            true,
            Some("text/event-stream"),
            "claude:messages",
            "claude:messages",
            false,
            true,
            false,
        )
        .commits_on_response_headers());
    }

    #[test]
    fn gate_detects_anthropic_error_across_every_chunk_boundary() {
        let event = b"event: error\r\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"busy\"}}\r\n\r\n";
        for split in 1..event.len() {
            let mut gate = StreamCommitGate::new(native_anthropic_policy());
            let first_observation = gate.observe_provider_bytes(&event[..split]);
            if matches!(
                first_observation,
                StreamPrecommitObservation::UpstreamError {
                    status_code: 529,
                    ..
                }
            ) {
                assert_eq!(event[split - 1], b'\r');
            } else {
                assert_eq!(first_observation, StreamPrecommitObservation::Pending);
                assert!(matches!(
                    gate.observe_provider_bytes(&event[split..]),
                    StreamPrecommitObservation::UpstreamError {
                        status_code: 529,
                        ..
                    }
                ));
            }
            assert_eq!(gate.state(), StreamCommitState::Terminal);
        }
    }

    #[test]
    fn gate_detects_cr_only_and_mixed_line_ending_errors() {
        for event in [
            "event: error\rdata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\"}}\r\r",
            "event: error\r\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\"}}\n\r",
        ] {
            for split in 1..event.len() {
                let mut gate = StreamCommitGate::new(native_anthropic_policy());
                assert_eq!(
                    gate.observe_provider_bytes(&event.as_bytes()[..split]),
                    StreamPrecommitObservation::Pending,
                    "gate committed before complete mixed-line event at split {split}",
                );
                assert!(matches!(
                    gate.observe_provider_bytes(&event.as_bytes()[split..]),
                    StreamPrecommitObservation::UpstreamError {
                        status_code: 529,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn unknown_and_ping_events_do_not_commit_before_anthropic_error() {
        let mut gate = StreamCommitGate::new(native_anthropic_policy());
        assert_eq!(
            gate.observe_provider_bytes(
                b"event: future_event\ndata: {\"type\":\"future_event\",\"value\":1}\n\n"
            ),
            StreamPrecommitObservation::Pending
        );
        assert_eq!(
            gate.observe_provider_bytes(b"event: ping\ndata: {\"type\":\"ping\"}\n\n"),
            StreamPrecommitObservation::Pending
        );
        assert!(matches!(
            gate.observe_provider_bytes(
                b"event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"rate_limit_error\"}}\n\n"
            ),
            StreamPrecommitObservation::UpstreamError {
                status_code: 429,
                ..
            }
        ));
    }

    #[test]
    fn first_semantic_event_commits_before_later_error_in_same_chunk() {
        let mut gate = StreamCommitGate::new(native_anthropic_policy());
        let observation = gate.observe_provider_bytes(
            concat!(
                "event: message_start\n",
                "data: {\"type\":\"message_start\",\"message\":{}}\n\n",
                "event: error\n",
                "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\"}}\n\n",
            )
            .as_bytes(),
        );

        assert_eq!(observation, StreamPrecommitObservation::Commit);
        assert_eq!(gate.state(), StreamCommitState::Committed);
    }

    #[test]
    fn transport_fragment_count_does_not_commit_an_incomplete_anthropic_error() {
        let policy = StreamCommitPolicy::FirstAnthropicSemanticEvent {
            max_bytes: 1024,
            max_wait: Duration::from_millis(750),
        };
        let mut gate = StreamCommitGate::new(policy);
        let event = b"event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\"}}\n\n";
        for byte in &event[..event.len() - 1] {
            assert_eq!(
                gate.observe_provider_bytes(std::slice::from_ref(byte)),
                StreamPrecommitObservation::Pending,
            );
        }
        assert!(matches!(
            gate.observe_provider_bytes(&event[event.len() - 1..]),
            StreamPrecommitObservation::UpstreamError {
                status_code: 529,
                ..
            }
        ));
    }

    #[test]
    fn anthropic_error_status_mapping_matches_messages_api_taxonomy() {
        for (error_type, status_code) in [
            ("invalid_request_error", 400),
            ("authentication_error", 401),
            ("permission_error", 403),
            ("not_found_error", 404),
            ("request_too_large", 413),
            ("rate_limit_error", 429),
            ("overloaded_error", 529),
            ("api_error", 500),
        ] {
            let body = serde_json::json!({
                "type": "error",
                "error": { "type": error_type, "message": "upstream failure" }
            });
            assert_eq!(
                anthropic_error_status_code(&body),
                status_code,
                "unexpected status for {error_type}"
            );
        }
    }
}
