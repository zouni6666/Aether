use super::*;
use aether_data::repository::routing_profiles::InMemoryRoutingGroupRepository;
use aether_data_contracts::repository::routing_profiles::{
    CreateRoutingGroupRecord, RoutingGroupWriteRepository, UpdateRoutingGroupRecord,
};

#[tokio::test]
async fn users_me_api_key_routing_selection_round_trips_and_cannot_bypass_visibility() {
    let groups = Arc::new(InMemoryRoutingGroupRepository::default());
    for (id, enabled, visible) in [
        ("public", true, true),
        ("hidden", true, false),
        ("disabled", false, true),
    ] {
        groups
            .create_routing_group(CreateRoutingGroupRecord {
                id: id.into(),
                name: format!("{id} strategy"),
                description: None,
                enabled,
                is_system_default: false,
                sort_order: 0,
                config_json: json!({"user_visible": visible}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();
    }
    let now = Utc::now();
    let mut user = sample_auth_user(now);
    user.role = "user".into();
    let token = build_test_auth_token(
        "access",
        serde_json::Map::from_iter([
            ("user_id".into(), json!(user.id)),
            ("role".into(), json!(user.role)),
            (
                "created_at".into(),
                json!(user.created_at.map(|date| date.to_rfc3339())),
            ),
            ("session_id".into(), json!("session-api-key-routing")),
        ]),
        now + chrono::Duration::hours(1),
    );
    let users = Arc::new(InMemoryUserReadRepository::seed_auth_users(vec![user]));
    let keys = Arc::new(InMemoryAuthApiKeySnapshotRepository::default());
    let (url, upstream_hits, gateway, upstream) = start_auth_gateway_with_builder(|| {
        let data = GatewayDataState::with_auth_api_key_repository_for_tests(keys)
            .with_user_reader(users)
            .with_routing_group_repository_for_tests(groups.clone())
            .with_encryption_key_for_tests(DEVELOPMENT_ENCRYPTION_KEY);
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(data)
            .with_auth_sessions_for_tests([sample_auth_session(
                "user-auth-1",
                "session-api-key-routing",
                "device-api-key-routing",
                "refresh-api-key-routing",
                now,
            )])
    })
    .await;
    let client = reqwest::Client::new();
    let endpoint = format!("{url}/api/users/me/api-keys");
    let request = |method: reqwest::Method, path: &str| {
        client
            .request(method, path)
            .bearer_auth(&token)
            .header("x-client-device-id", "device-api-key-routing")
            .header("user-agent", "AetherTest/1.0")
    };
    let response = request(reqwest::Method::POST, &endpoint)
        .json(&json!({
            "name": "selected key", "routing_group_id": "public",
            "feature_settings": {"chat_pii_redaction": {"enabled": true}},
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let created: serde_json::Value = response.json().await.unwrap();
    assert_eq!(created["routing_group_id"], "public");
    assert_eq!(created["routing_group_name"], "public strategy");
    assert_eq!(created["feature_settings"]["routing_group_id"], "public");
    assert!(created["feature_settings"]
        .get("routing_group_name")
        .is_none());
    let detail_url = format!("{endpoint}/{}", created["id"].as_str().unwrap());

    let list: serde_json::Value = request(reqwest::Method::GET, &endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list[0]["routing_group_id"], "public");
    assert_eq!(list[0]["routing_group_name"], "public strategy");
    let detail: serde_json::Value = request(reqwest::Method::GET, &detail_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(detail["routing_group_id"], "public");
    assert_eq!(detail["routing_group_name"], "public strategy");

    let response = request(reqwest::Method::PUT, &detail_url)
        .json(&json!({
            "name": "renamed key", "feature_settings": {
                "chat_pii_redaction": {"enabled": false}, "routing_group_id": "hidden",
            },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let edited: serde_json::Value = response.json().await.unwrap();
    assert_eq!(edited["name"], "renamed key");
    assert_eq!(edited["routing_group_id"], "public");
    assert_eq!(
        edited["feature_settings"]["chat_pii_redaction"]["enabled"],
        false
    );
    for id in ["hidden", "disabled", "missing"] {
        let rejected = request(reqwest::Method::PUT, &detail_url)
            .json(&json!({
                "name": "must not change", "routing_group_id": id,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::BAD_REQUEST, "id={id}");
    }
    let unchanged: serde_json::Value = request(reqwest::Method::GET, &detail_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(unchanged["name"], "renamed key");
    assert_eq!(unchanged["routing_group_id"], "public");

    groups
        .update_routing_group(
            "public",
            UpdateRoutingGroupRecord {
                name: Some("renamed strategy".into()),
                config_json: Some(json!({"user_visible": false})),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let response = request(reqwest::Method::PUT, &detail_url)
        .json(&json!({
            "name": "existing hidden choice", "routing_group_id": "public",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let renamed: serde_json::Value = response.json().await.unwrap();
    assert_eq!(renamed["routing_group_id"], "public");
    assert_eq!(renamed["routing_group_name"], "renamed strategy");
    let response = request(reqwest::Method::PUT, &detail_url)
        .json(&json!({"routing_group_id": null}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cleared: serde_json::Value = response.json().await.unwrap();
    assert_eq!(cleared["routing_group_id"], serde_json::Value::Null);
    assert_eq!(cleared["routing_group_name"], serde_json::Value::Null);
    assert!(cleared["feature_settings"]
        .get("routing_group_id")
        .is_none());
    assert_eq!(
        cleared["feature_settings"]["chat_pii_redaction"]["enabled"],
        false
    );

    assert_eq!(*upstream_hits.lock().unwrap(), 0);
    gateway.abort();
    upstream.abort();
}
