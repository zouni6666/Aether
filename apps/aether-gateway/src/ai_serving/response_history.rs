use crate::ai_serving::{
    hydrate_response_history, normalize_api_format_alias, record_converted_response_history,
    response_history_is_loaded, response_history_storage_key, ResponseHistoryRecord,
};
use aether_runtime_state::RuntimeState;
use serde_json::Value;
use tracing::warn;

use crate::GatewayError;

pub(crate) async fn hydrate_openai_response_history(
    runtime_state: &RuntimeState,
    request: &Value,
    client_api_format: &str,
    provider_api_format: &str,
    history_scope: &str,
) -> Result<(), GatewayError> {
    if normalize_api_format_alias(client_api_format) != "openai:responses"
        || normalize_api_format_alias(provider_api_format) != "openai:chat"
    {
        return Ok(());
    }
    let Some(previous_response_id) = request
        .get("previous_response_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    if response_history_is_loaded(previous_response_id, Some(history_scope)) {
        return Ok(());
    }

    let storage_key = response_history_storage_key(previous_response_id, Some(history_scope));
    let payload = runtime_state.kv_get(&storage_key).await.map_err(|error| {
        warn!(
            event_name = "openai_response_history_read_failed",
            log_type = "ops",
            backend = runtime_state.backend_kind().as_str(),
            error = ?error,
            "gateway failed to read shared OpenAI response history"
        );
        GatewayError::Internal("OpenAI response history lookup failed".to_string())
    })?;
    let Some(payload) = payload else {
        return Ok(());
    };
    if let Err(error) =
        hydrate_response_history(previous_response_id, Some(history_scope), &payload)
    {
        let _ = runtime_state.kv_delete(&storage_key).await;
        warn!(
            event_name = "openai_response_history_invalid",
            log_type = "ops",
            backend = runtime_state.backend_kind().as_str(),
            error = %error,
            "gateway rejected invalid shared OpenAI response history"
        );
        return Err(GatewayError::Internal(
            "OpenAI response history validation failed".to_string(),
        ));
    }
    Ok(())
}

pub(crate) async fn persist_response_history_record(
    runtime_state: &RuntimeState,
    record: ResponseHistoryRecord,
) {
    if let Err(error) = runtime_state
        .kv_set(&record.storage_key, record.payload, Some(record.ttl))
        .await
    {
        warn!(
            event_name = "openai_response_history_write_failed",
            log_type = "ops",
            backend = runtime_state.backend_kind().as_str(),
            error = ?error,
            "gateway failed to persist shared OpenAI response history"
        );
    }
}

pub(crate) async fn persist_converted_response_history(
    runtime_state: &RuntimeState,
    report_context: &Value,
    response: Option<&Value>,
) {
    let Some(response) = response else {
        return;
    };
    if let Some(record) = record_converted_response_history(report_context, response) {
        persist_response_history_record(runtime_state, record).await;
    }
}
