use std::collections::BTreeMap;

use aether_data_contracts::repository::routing_profiles::RoutingGroupLookupKey;
use axum::{body::Body, http::StatusCode, response::Response};
use serde::Deserialize;
use serde_json::{Map, Value};

use super::{build_auth_error_response, normalize_feature_settings, AppState};
use crate::routing::selection::routing_group_is_user_visible;

const ROUTING_GROUP_ID: &str = "routing_group_id";

pub(super) fn deserialize_routing_group_patch<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

pub(super) fn api_key_routing_group_id(settings: Option<&Value>) -> Option<&str> {
    settings
        .and_then(|value| value.get(ROUTING_GROUP_ID))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) async fn validate_routing_group_patch(
    state: &AppState,
    current: Option<&Value>,
    requested: Option<Option<String>>,
) -> Result<Option<Option<String>>, Response<Body>> {
    let Some(Some(requested)) = requested else {
        return Ok(requested);
    };
    let id = requested.trim();
    if id.is_empty() || id.len() > 128 {
        return Err(build_auth_error_response(
            StatusCode::BAD_REQUEST,
            "routing_group_id 必须是有效的策略分组 ID；跟随默认请传 null",
            false,
        ));
    }
    // A group can become hidden or disabled after selection. An unrelated edit
    // (including a form resubmitting its unchanged selection) must remain valid.
    if api_key_routing_group_id(current) == Some(id) {
        return Ok(Some(Some(id.to_string())));
    }
    if !state.has_routing_group_data_reader() {
        return Err(build_auth_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "策略分组目录暂不可用",
            false,
        ));
    }
    let group = state
        .find_routing_group(RoutingGroupLookupKey::Id(id))
        .await
        .map_err(|error| {
            build_auth_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("user API key routing group lookup failed: {error:?}"),
                false,
            )
        })?;
    if !group
        .as_ref()
        .is_some_and(|group| group.enabled && routing_group_is_user_visible(group))
    {
        return Err(build_auth_error_response(
            StatusCode::BAD_REQUEST,
            "所选策略分组不存在或当前不可选",
            false,
        ));
    }
    Ok(Some(Some(id.to_string())))
}

/// Compose the initial create record. Existing-key updates must use the atomic
/// repository routing selection patch instead of merging a pre-read snapshot.
pub(super) fn merge_api_key_feature_settings(
    current: Option<&Value>,
    incoming: Option<Option<Value>>,
    routing_group_patch: Option<Option<String>>,
) -> Result<Option<Option<Value>>, String> {
    if incoming.is_none() && routing_group_patch.is_none() {
        return Ok(None);
    }
    let mut settings = normalize_feature_settings(incoming.unwrap_or_else(|| current.cloned()))?
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    settings.remove(ROUTING_GROUP_ID);
    settings.remove("routing_group_name");
    let group_id = routing_group_patch
        .unwrap_or_else(|| api_key_routing_group_id(current).map(ToOwned::to_owned));
    if let Some(group_id) = group_id {
        settings.insert(ROUTING_GROUP_ID.to_string(), Value::String(group_id));
    }
    Ok(Some(
        (!settings.is_empty()).then_some(Value::Object(settings)),
    ))
}

pub(super) fn normalize_api_key_feature_settings_patch(
    value: Option<Option<Value>>,
) -> Result<Option<Option<Value>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(Value::Object(mut settings)) = normalize_feature_settings(value)? else {
        return Ok(Some(None));
    };
    settings.remove(ROUTING_GROUP_ID);
    settings.remove("routing_group_name");
    Ok(Some(
        (!settings.is_empty()).then_some(Value::Object(settings)),
    ))
}

pub(super) async fn routing_group_names(
    state: &AppState,
    needed: bool,
) -> BTreeMap<String, String> {
    if !needed || !state.has_routing_group_data_reader() {
        return BTreeMap::new();
    }
    match state.list_routing_groups().await {
        Ok(groups) => groups
            .into_iter()
            .map(|group| (group.id, group.name))
            .collect(),
        Err(error) => {
            tracing::warn!(?error, "API key routing group names unavailable");
            BTreeMap::new()
        }
    }
}

pub(super) fn routing_group_payload_fields(
    settings: Option<&Value>,
    names: &BTreeMap<String, String>,
) -> Map<String, Value> {
    let id = api_key_routing_group_id(settings);
    Map::from_iter([
        (ROUTING_GROUP_ID.to_string(), serde_json::json!(id)),
        (
            "routing_group_name".to_string(),
            serde_json::json!(id.and_then(|id| names.get(id))),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aether_data::repository::routing_profiles::InMemoryRoutingGroupRepository;
    use aether_data_contracts::repository::routing_profiles::StoredRoutingGroup;
    use serde_json::json;

    use super::*;

    fn group(id: &str, enabled: bool, visible: Value) -> StoredRoutingGroup {
        StoredRoutingGroup {
            id: id.to_string(),
            name: format!("{id} current name"),
            description: None,
            enabled,
            is_system_default: false,
            sort_order: 0,
            config_json: json!({"user_visible": visible}),
            version: 1,
            created_at: 1,
            updated_at: 1,
            published_at: None,
        }
    }

    fn state() -> AppState {
        let repository = Arc::new(InMemoryRoutingGroupRepository::seed(
            [
                group("public", true, json!(true)),
                group("hidden", true, json!(false)),
                group("disabled", false, json!(true)),
                group("legacy", true, Value::Null),
                group("malformed", true, json!("true")),
            ],
            [],
            [],
        ));
        AppState::new().unwrap().with_data_state_for_tests(
            crate::data::GatewayDataState::disabled()
                .with_routing_group_repository_for_tests(repository),
        )
    }

    #[tokio::test]
    async fn only_enabled_user_visible_groups_can_be_newly_selected() {
        let state = state();
        for id in [
            "",
            " ",
            "missing",
            "hidden",
            "disabled",
            "legacy",
            "malformed",
        ] {
            assert_eq!(
                validate_routing_group_patch(&state, None, Some(Some(id.to_string())))
                    .await
                    .unwrap_err()
                    .status(),
                StatusCode::BAD_REQUEST,
                "id={id}"
            );
        }
        assert_eq!(
            validate_routing_group_patch(&state, None, Some(Some(" public ".to_string())))
                .await
                .unwrap(),
            Some(Some("public".to_string()))
        );
        let unavailable = AppState::new()
            .unwrap()
            .with_data_state_for_tests(crate::data::GatewayDataState::disabled());
        assert_eq!(
            validate_routing_group_patch(&unavailable, None, Some(Some("public".into())))
                .await
                .unwrap_err()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn unchanged_or_cleared_choices_do_not_require_current_visibility_or_catalog_access() {
        let state = AppState::new()
            .unwrap()
            .with_data_state_for_tests(crate::data::GatewayDataState::disabled());
        let current = json!({"routing_group_id": "hidden"});
        for patch in [None, Some(None), Some(Some("hidden".to_string()))] {
            assert_eq!(
                validate_routing_group_patch(&state, Some(&current), patch.clone())
                    .await
                    .unwrap(),
                patch
            );
        }
    }

    #[test]
    fn feature_patch_does_not_carry_a_pre_read_routing_selection() {
        assert_eq!(
            normalize_api_key_feature_settings_patch(None).unwrap(),
            None
        );
        assert_eq!(
            normalize_api_key_feature_settings_patch(Some(None)).unwrap(),
            Some(None)
        );
        let patch = normalize_api_key_feature_settings_patch(Some(Some(json!({
            "routing_group_id": "stale-or-forged",
            "routing_group_name": "stale name",
            "chat_pii_redaction": {"enabled": false},
        }))))
        .unwrap()
        .flatten()
        .unwrap();
        assert!(patch.get("routing_group_id").is_none());
        assert!(patch.get("routing_group_name").is_none());
        assert_eq!(patch["chat_pii_redaction"]["enabled"], false);
    }

    #[test]
    fn feature_updates_cannot_inject_replace_or_clear_a_routing_choice() {
        let current =
            json!({"routing_group_id": "saved", "chat_pii_redaction": {"enabled": false}});
        let injection = json!({"routing_group_id": "hidden", "routing_group_name": "forged", "chat_pii_redaction": {"enabled": true}});
        let created = merge_api_key_feature_settings(None, Some(Some(injection.clone())), None)
            .unwrap()
            .flatten()
            .unwrap();
        assert!(created.get("routing_group_id").is_none());
        assert!(created.get("routing_group_name").is_none());
        for feature_patch in [
            Some(injection),
            Some(json!({"routing_group_id": null})),
            None,
        ] {
            let updated = merge_api_key_feature_settings(Some(&current), Some(feature_patch), None)
                .unwrap()
                .flatten()
                .unwrap();
            assert_eq!(updated["routing_group_id"], "saved");
            assert!(updated.get("routing_group_name").is_none());
        }
        let untouched = merge_api_key_feature_settings(Some(&current), None, None).unwrap();
        assert_eq!(
            untouched, None,
            "name/rate/IP-only updates must leave settings untouched"
        );
    }

    #[test]
    fn validated_top_level_selection_and_clear_preserve_other_feature_settings() {
        let current = json!({"routing_group_id": "saved", "chat_pii_redaction": {"enabled": false}, "another_setting": 7});
        let selected =
            merge_api_key_feature_settings(Some(&current), None, Some(Some("public".to_string())))
                .unwrap()
                .flatten()
                .unwrap();
        assert_eq!(selected["routing_group_id"], "public");
        assert_eq!(selected["another_setting"], 7);
        assert_eq!(selected["chat_pii_redaction"]["enabled"], false);
        let cleared = merge_api_key_feature_settings(Some(&selected), None, Some(None))
            .unwrap()
            .flatten()
            .unwrap();
        assert!(cleared.get("routing_group_id").is_none());
        assert_eq!(cleared["another_setting"], 7);
        let replacement = merge_api_key_feature_settings(
            None,
            Some(Some(json!({"routing_group_id": "hidden"}))),
            Some(Some("public".to_string())),
        )
        .unwrap()
        .flatten()
        .unwrap();
        assert_eq!(replacement, json!({"routing_group_id": "public"}));
    }

    #[tokio::test]
    async fn names_are_resolved_from_the_catalog_without_persisting_a_name_snapshot() {
        let state = state();
        let names = routing_group_names(&state, true).await;
        let current = json!({"routing_group_id": "hidden", "routing_group_name": "stale"});
        let fields = routing_group_payload_fields(Some(&current), &names);
        assert_eq!(fields["routing_group_id"], "hidden");
        assert_eq!(fields["routing_group_name"], "hidden current name");
        let missing =
            routing_group_payload_fields(Some(&json!({"routing_group_id": "deleted"})), &names);
        assert_eq!(missing["routing_group_id"], "deleted");
        assert_eq!(missing["routing_group_name"], Value::Null);
        let default = routing_group_payload_fields(None, &names);
        assert_eq!(default["routing_group_id"], Value::Null);
        assert_eq!(default["routing_group_name"], Value::Null);
    }
}
