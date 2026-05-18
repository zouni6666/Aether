use super::*;
use serde_json::json;

#[test]
fn provider_query_test_request_body_preserves_custom_model() {
    let payload = json!({
        "request_body": {
            "model": "custom-upstream-model",
            "messages": []
        }
    });

    let body = provider_query_build_test_request_body(&payload, "fallback-model");

    assert_eq!(body["model"], json!("custom-upstream-model"));
}

#[test]
fn provider_query_test_request_body_defaults_missing_model() {
    let payload = json!({
        "request_body": {
            "messages": []
        }
    });

    let body = provider_query_build_test_request_body(&payload, "fallback-model");

    assert_eq!(body["model"], json!("fallback-model"));
}

#[test]
fn provider_query_default_test_request_body_does_not_set_max_tokens() {
    let body = provider_query_build_test_request_body(&json!({}), "fallback-model");

    assert_eq!(body["model"], json!("fallback-model"));
    assert!(
        body.get("max_tokens").is_none(),
        "admin model test must not silently force a low max_tokens value"
    );
}

#[test]
fn provider_query_failover_request_body_overrides_custom_model() {
    let payload = json!({
        "request_body": {
            "model": "custom-upstream-model",
            "messages": []
        }
    });

    let body = provider_query_build_test_request_body_for_route(
        &payload,
        "failover-model",
        "/api/admin/provider-query/test-model-failover",
    );

    assert_eq!(body["model"], json!("failover-model"));
}

#[test]
fn provider_query_failover_request_body_uses_explicit_mapped_model() {
    let payload = json!({
        "mapped_model_name": "upstream-mapped-model",
        "request_body": {
            "model": "original-selected-model",
            "messages": []
        }
    });

    let body = provider_query_build_test_request_body_for_route(
        &payload,
        "upstream-mapped-model",
        "/api/admin/provider-query/test-model-failover",
    );

    assert_eq!(body["model"], json!("upstream-mapped-model"));
}

#[test]
fn provider_query_request_body_model_uses_non_empty_string_only() {
    let custom = json!({ "model": " custom-model " });
    let blank = json!({ "model": " " });
    let non_string = json!({ "model": 123 });

    assert_eq!(
        provider_query_request_body_model(&custom, "fallback-model"),
        "custom-model"
    );
    assert_eq!(
        provider_query_request_body_model(&blank, "fallback-model"),
        "fallback-model"
    );
    assert_eq!(
        provider_query_request_body_model(&non_string, "fallback-model"),
        "fallback-model"
    );
}

#[test]
fn provider_query_standard_test_resolves_codex_responses_upstream_streaming() {
    assert!(provider_query_resolve_standard_test_upstream_is_stream(
        None,
        "codex",
        "openai:responses",
    ));
    assert!(provider_query_resolve_standard_test_upstream_is_stream(
        Some(&json!({"upstream_stream_policy": "force_non_stream"})),
        "codex",
        "openai:responses",
    ));
    assert!(!provider_query_resolve_standard_test_upstream_is_stream(
        None,
        "codex",
        "openai:responses:compact",
    ));
    assert!(!provider_query_resolve_standard_test_upstream_is_stream(
        None,
        "custom",
        "openai:responses",
    ));
    assert!(provider_query_resolve_standard_test_upstream_is_stream(
        Some(&json!({"upstream_stream_policy": "force_stream"})),
        "custom",
        "openai:responses",
    ));
}

#[test]
fn provider_query_standard_test_reenforces_upstream_stream_body_field() {
    let endpoint_config = json!({"upstream_stream_policy": "force_stream"});
    let mut body = json!({"model": "gpt-5", "input": "hello", "stream": false});
    let upstream_is_stream = provider_query_resolve_standard_test_upstream_is_stream(
        Some(&endpoint_config),
        "codex",
        "openai:responses",
    );
    let require_body_stream_field =
        provider_query_request_requires_body_stream_field(&body, Some(&endpoint_config));

    crate::ai_serving::enforce_request_body_stream_field(
        &mut body,
        "openai:responses",
        upstream_is_stream,
        require_body_stream_field,
    );

    assert_eq!(body["stream"], json!(true));
}

#[test]
fn provider_query_standard_test_aggregates_responses_stream_body() {
    let stream_body = concat!(
        "event: response.created\n",
        "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_123\",\"object\":\"response\",\"model\":\"gpt-5.4-mini\",\"status\":\"in_progress\",\"output\":[]}}\n\n",
        "event: response.output_text.delta\n",
        "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"Hello\"}\n\n",
        "event: response.completed\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_123\",\"object\":\"response\",\"model\":\"gpt-5.4-mini\",\"status\":\"completed\",\"output\":[]}}\n\n",
    );
    let result = aether_contracts::ExecutionResult {
        request_id: "provider-test".to_string(),
        candidate_id: Some("candidate-0".to_string()),
        status_code: 200,
        headers: BTreeMap::new(),
        body: Some(aether_contracts::ResponseBody {
            json_body: None,
            body_bytes_b64: Some(
                base64::engine::general_purpose::STANDARD.encode(stream_body.as_bytes()),
            ),
        }),
        telemetry: None,
        error: None,
    };

    let body = provider_query_standard_execution_response_body("openai:responses", &result)
        .expect("stream body should aggregate");

    assert_eq!(body["model"], json!("gpt-5.4-mini"));
    assert_eq!(body["output"][0]["content"][0]["text"], json!("Hello"));
}

#[test]
fn provider_query_standard_test_rejects_gemini_success_without_visible_output() {
    let result = aether_contracts::ExecutionResult {
        request_id: "provider-test".to_string(),
        candidate_id: Some("candidate-0".to_string()),
        status_code: 200,
        headers: BTreeMap::new(),
        body: Some(aether_contracts::ResponseBody {
            json_body: Some(json!({
                "candidates": [{
                    "content": {"role": "model"},
                    "finishReason": "MAX_TOKENS"
                }],
                "usageMetadata": {
                    "promptTokenCount": 8,
                    "candidatesTokenCount": 1,
                    "thoughtsTokenCount": 25,
                    "totalTokenCount": 34
                },
                "modelVersion": "gemini-3-flash-preview",
                "responseId": "resp-empty"
            })),
            body_bytes_b64: None,
        }),
        telemetry: None,
        error: None,
    };

    assert!(
        provider_query_standard_execution_response_body("gemini:generate_content", &result)
            .is_none()
    );
}

#[test]
fn provider_query_test_adapter_routes_fixed_provider_endpoint_types() {
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("custom", "openai:chat"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("codex", "openai:responses"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("codex", "openai:responses:compact"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("chatgpt_web", "openai:image"),
        Some(ProviderQueryTestAdapter::OpenAiImage)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("kiro", "claude:messages"),
        Some(ProviderQueryTestAdapter::Kiro)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format(
            "gemini_cli",
            "gemini:generate_content"
        ),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format(
            "antigravity",
            "gemini:generate_content"
        ),
        Some(ProviderQueryTestAdapter::Antigravity)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("custom", "openai:embedding"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("custom", "gemini:embedding"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("jina", "jina:rerank"),
        Some(ProviderQueryTestAdapter::Standard)
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("custom", "openai:video"),
        None
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("custom", "gemini:video"),
        None
    );
    assert_eq!(
        provider_query_test_adapter_for_provider_api_format("gemini", "gemini:files"),
        None
    );
}

#[test]
fn provider_query_endpoint_priority_prefers_text_before_cli_and_image() {
    assert_eq!(
        provider_query_model_test_endpoint_priority("custom", "openai:chat"),
        Some(0)
    );
    assert_eq!(
        provider_query_model_test_endpoint_priority("codex", "openai:responses:compact"),
        Some(1)
    );
    assert_eq!(
        provider_query_model_test_endpoint_priority("chatgpt_web", "openai:image"),
        Some(2)
    );
    assert_eq!(
        provider_query_model_test_endpoint_priority("antigravity", "gemini:generate_content"),
        Some(1)
    );
}

#[test]
fn provider_query_candidate_summary_marks_unused_after_first_success() {
    let attempts = vec![json!({
        "candidate_index": 0,
        "key_name": "winning-key",
        "key_id": "key-1",
        "auth_type": "api_key",
        "effective_model": "claude-haiku",
        "endpoint_api_format": "claude:messages",
        "endpoint_base_url": "https://api.example",
        "status": "success",
        "latency_ms": 123,
        "status_code": 200
    })];

    let summary = provider_query_candidate_summary_payload(3, 1, &attempts);

    assert_eq!(summary["total_candidates"], json!(3));
    assert_eq!(summary["attempted"], json!(1));
    assert_eq!(summary["success"], json!(1));
    assert_eq!(summary["unused"], json!(2));
    assert_eq!(summary["stop_reason"], json!("first_success"));
    assert_eq!(summary["winning_key_name"], json!("winning-key"));
    assert_eq!(
        summary["winning_endpoint_api_format"],
        json!("claude:messages")
    );
}

#[test]
fn provider_query_candidate_summary_reports_skipped_exhaustion() {
    let attempts = vec![json!({
        "candidate_index": 0,
        "status": "skipped",
        "skip_reason": "transport_api_format_mismatch"
    })];

    let summary = provider_query_candidate_summary_payload(1, 0, &attempts);

    assert_eq!(summary["total_candidates"], json!(1));
    assert_eq!(summary["attempted"], json!(0));
    assert_eq!(summary["skipped"], json!(1));
    assert_eq!(summary["unused"], json!(0));
    assert_eq!(summary["stop_reason"], json!("all_skipped"));
}

#[test]
fn provider_query_candidate_summary_counts_scheduler_skips_before_success() {
    let attempts = vec![
        json!({
            "candidate_index": 0,
            "status": "skipped",
            "skip_reason": "pool_account_exhausted"
        }),
        json!({
            "candidate_index": 1,
            "key_name": "winning-key",
            "key_id": "key-1",
            "auth_type": "oauth",
            "effective_model": "gpt-5.4-mini",
            "endpoint_api_format": "openai:responses",
            "endpoint_base_url": "https://chatgpt.com/backend-api/codex",
            "status": "success",
            "latency_ms": 123,
            "status_code": 200
        }),
    ];

    let summary = provider_query_candidate_summary_payload(4, 1, &attempts);

    assert_eq!(summary["total_candidates"], json!(4));
    assert_eq!(summary["attempted"], json!(1));
    assert_eq!(summary["success"], json!(1));
    assert_eq!(summary["skipped"], json!(1));
    assert_eq!(summary["unused"], json!(2));
    assert_eq!(summary["stop_reason"], json!("first_success"));
}

#[test]
fn provider_query_image_test_request_body_defaults_generation_prompt() {
    let payload = json!({"message": "draw a small icon"});

    let body = provider_query_build_openai_image_test_request_body_for_route(
        &payload,
        "gpt-image-1",
        "/api/admin/provider-query/test-model",
    );

    assert_eq!(body["model"], json!("gpt-image-1"));
    assert_eq!(body["prompt"], json!("draw a small icon"));
    assert_eq!(body["stream"], json!(true));
}

#[test]
fn provider_query_failover_image_test_request_body_overrides_model() {
    let payload = json!({
        "request_body": {
            "model": "old-image-model",
            "prompt": "draw"
        }
    });

    let body = provider_query_build_openai_image_test_request_body_for_route(
        &payload,
        "new-image-model",
        "/api/admin/provider-query/test-model-failover",
    );

    assert_eq!(body["model"], json!("new-image-model"));
}
