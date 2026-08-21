use serde_json::Value;

pub mod codex;
pub(crate) mod history;
pub mod request;
pub mod response;
pub mod spec;
pub mod stream;

const TOOL_ERROR_PREFIX: &str = "[tool error]";
const AETHER_REASONING_ITEM_ID_PREFIX: &str = "rs_aether_";

/// Controls which provider-owned reasoning items may be replayed on a Responses request.
///
/// OpenAI reasoning references are identified by their `rs...` item IDs. DeepSeek's Responses
/// contract instead returns opaque, id-less `reasoning_text` items whose `encrypted_content`
/// must be sent back unchanged on later tool turns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OpenAiResponsesReasoningReplayPolicy {
    #[default]
    OpenAiItemIds,
    DeepSeekOpaque,
}

/// Builds a stable, wire-compatible ID for a reasoning item synthesized by Aether.
///
/// The marker lets the outbound request sanitizer distinguish synthetic summaries from
/// provider-backed reasoning items. Synthetic items without encrypted reasoning state are useful
/// in client responses, but cannot be replayed as provider-owned reasoning state.
pub fn openai_responses_synthetic_reasoning_item_id(
    response_id: &str,
    output_index: usize,
) -> String {
    let seed = format!("{response_id}:{output_index}");
    format!(
        "{AETHER_REASONING_ITEM_ID_PREFIX}{}",
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, seed.as_bytes()).simple()
    )
}

/// Removes reasoning history items that cannot be replayed against an OpenAI Responses backend.
///
/// Reasoning IDs are opaque provider references and must never be repaired by changing their
/// prefix. Foreign IDs (for example `item_...`) are therefore removed. Aether-synthesized
/// reasoning summaries are also removed unless they carry encrypted reasoning state that can be
/// replayed statelessly.
pub fn strip_incompatible_openai_responses_reasoning_items(
    body: &mut Value,
    provider_api_format: &str,
) -> usize {
    strip_incompatible_openai_responses_reasoning_items_with_policy(
        body,
        provider_api_format,
        OpenAiResponsesReasoningReplayPolicy::OpenAiItemIds,
    )
}

pub fn strip_incompatible_openai_responses_reasoning_items_with_policy(
    body: &mut Value,
    provider_api_format: &str,
    policy: OpenAiResponsesReasoningReplayPolicy,
) -> usize {
    if !aether_ai_formats::is_openai_responses_family_format(provider_api_format) {
        return 0;
    }
    // DeepSeek's id-less opaque state is valid only on the normal Responses
    // continuation contract. Both the legacy Compact endpoint and the current
    // `compaction_trigger` operation must retain the strict OpenAI item-id
    // replay rules even when the same provider key serves ordinary Responses.
    let normal_responses =
        aether_ai_formats::normalize_api_format_alias(provider_api_format) == "openai:responses";
    let compact_operation = openai_responses_request_operation(provider_api_format, body).is_some();
    let policy = if policy == OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque
        && (!normal_responses || compact_operation)
    {
        OpenAiResponsesReasoningReplayPolicy::OpenAiItemIds
    } else {
        policy
    };
    let Some(items) = body.get_mut("input").and_then(Value::as_array_mut) else {
        return 0;
    };
    let original_len = items.len();
    items.retain(|item| openai_responses_reasoning_item_is_replayable(item, policy));
    original_len.saturating_sub(items.len())
}

fn openai_responses_reasoning_item_is_replayable(
    item: &Value,
    policy: OpenAiResponsesReasoningReplayPolicy,
) -> bool {
    let Some(object) = item.as_object() else {
        return true;
    };
    if object.get("type").and_then(Value::as_str) != Some("reasoning") {
        return true;
    }
    if policy == OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque
        && deepseek_opaque_reasoning_item_is_replayable(object)
    {
        return true;
    }
    let Some(id) = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| id.starts_with("rs"))
    else {
        return false;
    };
    if !id.starts_with(AETHER_REASONING_ITEM_ID_PREFIX) {
        return true;
    }
    object
        .get("encrypted_content")
        .and_then(Value::as_str)
        .is_some_and(|encrypted_content| !encrypted_content.trim().is_empty())
}

fn deepseek_opaque_reasoning_item_is_replayable(object: &serde_json::Map<String, Value>) -> bool {
    if let Some(id) = object.get("id") {
        let id_is_empty = id.is_null() || id.as_str().is_some_and(|value| value.trim().is_empty());
        if !id_is_empty {
            return false;
        }
    }
    let has_encrypted_content = object
        .get("encrypted_content")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    let has_reasoning_text =
        object
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|content| {
                content.iter().any(|part| {
                    part.get("type").and_then(Value::as_str) == Some("reasoning_text")
                        && part.get("text").is_some_and(Value::is_string)
                })
            });
    has_encrypted_content && has_reasoning_text
}

/// Semantic operation carried by an OpenAI Responses request that asks the
/// service to compact a thread. The request still uses the Responses wire
/// contract and transport endpoint.
pub const OPENAI_RESPONSES_OPERATION_COMPACT: &str = "compact";

/// Resolves the operation expressed by an OpenAI Responses wire request.
///
/// `responses_compaction_v2` is represented by a `compaction_trigger` input
/// item on the normal Responses request. The legacy Compact API format is
/// retained as the same operation for observability and scoped model mapping.
pub fn openai_responses_request_operation(api_format: &str, body: &Value) -> Option<&'static str> {
    if aether_ai_formats::is_openai_responses_compact_format(api_format) {
        return Some(OPENAI_RESPONSES_OPERATION_COMPACT);
    }
    if !aether_ai_formats::is_openai_responses_format(api_format) {
        return None;
    }

    body.get("input")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("type").and_then(Value::as_str) == Some("compaction_trigger"))
        })
        .then_some(OPENAI_RESPONSES_OPERATION_COMPACT)
}

fn encode_tool_result_error(output: Value, is_error: bool) -> Value {
    if !is_error {
        return output;
    }
    let detail = match output {
        Value::String(text) => text,
        Value::Null => String::new(),
        value => serde_json::to_string(&value).unwrap_or_else(|_| value.to_string()),
    };
    if detail.is_empty() {
        Value::String(TOOL_ERROR_PREFIX.to_string())
    } else {
        Value::String(format!("{TOOL_ERROR_PREFIX}\n{detail}"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        openai_responses_request_operation, openai_responses_synthetic_reasoning_item_id,
        strip_incompatible_openai_responses_reasoning_items,
        strip_incompatible_openai_responses_reasoning_items_with_policy,
        OpenAiResponsesReasoningReplayPolicy, OPENAI_RESPONSES_OPERATION_COMPACT,
    };

    #[test]
    fn resolves_compaction_trigger_as_compact_operation_on_responses_transport() {
        assert_eq!(
            openai_responses_request_operation(
                "openai:responses",
                &json!({
                    "input": [
                        {"role": "user", "content": "keep working"},
                        {"type": "compaction_trigger"}
                    ]
                }),
            ),
            Some(OPENAI_RESPONSES_OPERATION_COMPACT)
        );
        assert_eq!(
            openai_responses_request_operation(
                "openai:responses",
                &json!({"input": [{"role": "user", "content": "keep working"}]}),
            ),
            None
        );
    }

    #[test]
    fn resolves_legacy_compact_contract_without_a_body_marker() {
        assert_eq!(
            openai_responses_request_operation("openai:responses:compact", &json!({})),
            Some(OPENAI_RESPONSES_OPERATION_COMPACT)
        );
    }

    #[test]
    fn synthetic_reasoning_item_ids_are_stable_and_wire_compatible() {
        let first = openai_responses_synthetic_reasoning_item_id("resp_123", 0);
        let second = openai_responses_synthetic_reasoning_item_id("resp_123", 0);
        let other = openai_responses_synthetic_reasoning_item_id("resp_123", 1);

        assert!(first.starts_with("rs_aether_"));
        assert_eq!(first, second);
        assert_ne!(first, other);
    }

    #[test]
    fn strips_foreign_and_non_replayable_synthetic_reasoning_items() {
        let portable_synthetic = openai_responses_synthetic_reasoning_item_id("resp_123", 1);
        let local_synthetic = openai_responses_synthetic_reasoning_item_id("resp_123", 2);
        let mut body = json!({
            "input": [
                {"type": "reasoning", "id": "rs_provider_123", "summary": []},
                {"type": "reasoning", "id": "item_72d3bd8d367d01977ace23f1", "summary": []},
                {"type": "reasoning", "id": "resp_123_rs_0", "summary": []},
                {"type": "reasoning", "summary": []},
                {
                    "type": "reasoning",
                    "id": portable_synthetic,
                    "summary": [],
                    "encrypted_content": "opaque"
                },
                {"type": "reasoning", "id": local_synthetic, "summary": []},
                {"type": "message", "id": "item_message_123", "role": "user", "content": "hi"}
            ]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items(&mut body, "openai:responses"),
            4
        );
        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["id"], "rs_provider_123");
        assert_eq!(input[1]["encrypted_content"], "opaque");
        assert_eq!(input[2]["id"], "item_message_123");
    }

    #[test]
    fn reasoning_item_sanitizer_is_scoped_to_responses_targets() {
        let mut body = json!({
            "input": [{"type": "reasoning", "id": "item_foreign", "summary": []}]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items(&mut body, "openai:chat"),
            0
        );
        assert_eq!(body["input"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn deepseek_policy_preserves_idless_opaque_reasoning_text_only_for_deepseek() {
        let item = json!({
            "type": "reasoning",
            "encrypted_content": "550e8400-e29b-41d4-a716-446655440000",
            "content": [{
                "type": "reasoning_text",
                "text": "opaque provider reasoning that must be replayed"
            }]
        });
        let mut strict = json!({"input": [item.clone()]});
        let mut deepseek = json!({"input": [item]});

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut strict,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::OpenAiItemIds,
            ),
            1
        );
        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut deepseek,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            0
        );
        assert_eq!(deepseek["input"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn deepseek_policy_does_not_preserve_unbound_reasoning_summaries() {
        let mut body = json!({
            "input": [
                {
                    "type": "reasoning",
                    "content": [{"type": "reasoning_text", "text": "missing state"}]
                },
                {
                    "type": "reasoning",
                    "encrypted_content": "opaque-without-reasoning-text",
                    "summary": [{"type": "summary_text", "text": "summary"}]
                }
            ]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut body,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            2
        );
        assert_eq!(body["input"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn deepseek_policy_preserves_empty_reasoning_text_with_opaque_state() {
        let mut body = json!({
            "input": [{
                "type": "reasoning",
                "encrypted_content": "opaque-state",
                "content": [{"type": "reasoning_text", "text": ""}],
                "future_capability": {"preserve": true}
            }]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut body,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            0
        );
        assert_eq!(body["input"][0]["content"][0]["text"], "");
        assert_eq!(body["input"][0]["future_capability"]["preserve"], true);
    }

    #[test]
    fn deepseek_policy_rejects_non_string_reasoning_text() {
        let mut body = json!({
            "input": [{
                "type": "reasoning",
                "encrypted_content": "opaque-state",
                "content": [{"type": "reasoning_text", "text": {"not": "text"}}]
            }]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut body,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            1
        );
        assert_eq!(body["input"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn deepseek_policy_keeps_strict_replay_for_compaction_trigger_operation() {
        let mut body = json!({
            "input": [
                {
                    "type": "reasoning",
                    "encrypted_content": "opaque-state",
                    "content": [{"type": "reasoning_text", "text": "thinking"}]
                },
                {"type": "compaction_trigger"}
            ]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut body,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            1
        );
        assert_eq!(body["input"], json!([{"type": "compaction_trigger"}]));
    }

    #[test]
    fn deepseek_policy_does_not_preserve_opaque_item_with_foreign_id() {
        let mut body = json!({
            "input": [{
                "type": "reasoning",
                "id": "item_provider_owned",
                "encrypted_content": "opaque-state",
                "content": [{"type": "reasoning_text", "text": "thinking"}]
            }]
        });

        assert_eq!(
            strip_incompatible_openai_responses_reasoning_items_with_policy(
                &mut body,
                "openai:responses",
                OpenAiResponsesReasoningReplayPolicy::DeepSeekOpaque,
            ),
            1
        );
        assert_eq!(body["input"].as_array().map(Vec::len), Some(0));
    }
}
