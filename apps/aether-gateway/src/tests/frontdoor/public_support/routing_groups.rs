use super::*;
use aether_data::repository::routing_profiles::InMemoryRoutingGroupRepository;
use aether_data_contracts::repository::routing_profiles::{
    CreateRoutingGroupRecord, RoutingGroupWriteRepository, UpdateRoutingGroupRecord,
};

#[tokio::test]
async fn users_me_routing_groups_requires_auth_and_exposes_only_public_enabled_summaries() {
    let repository = Arc::new(InMemoryRoutingGroupRepository::default());
    for (id, config, enabled, is_default) in [
        (
            "discount",
            json!({ "user_visible": true, "billing_multiplier": 0.5,
            "disabled_providers": ["secret-provider"] }),
            true,
            false,
        ),
        ("regular", json!({ "user_visible": true }), true, false),
        (
            "free",
            json!({ "user_visible": true, "billing_multiplier": 0.0 }),
            true,
            false,
        ),
        (
            "private-default",
            json!({ "user_visible": false }),
            true,
            true,
        ),
        ("legacy-private", json!({}), true, false),
        ("disabled", json!({ "user_visible": true }), false, false),
    ] {
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: id.into(),
                name: format!("{id}-name"),
                description: Some("private-description".into()),
                enabled,
                is_system_default: is_default,
                sort_order: 0,
                config_json: config,
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
            ("session_id".into(), json!("session-routing-groups")),
        ]),
        now + chrono::Duration::hours(1),
    );
    let users = Arc::new(InMemoryUserReadRepository::seed_auth_users(vec![user]));
    let (url, upstream_hits, gateway, upstream) = start_auth_gateway_with_builder(|| {
        let data = GatewayDataState::with_user_reader_for_tests(users)
            .with_routing_group_repository_for_tests(repository.clone());
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(data)
            .with_auth_sessions_for_tests([sample_auth_session(
                "user-auth-1",
                "session-routing-groups",
                "device-routing-groups",
                "refresh-placeholder",
                now,
            )])
    })
    .await;
    let client = reqwest::Client::new();
    let endpoint = format!("{url}/api/users/me/routing-groups");
    let unauthorized = client.get(&endpoint).send().await.unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let request = || {
        client
            .get(&endpoint)
            .bearer_auth(&token)
            .header("x-client-device-id", "device-routing-groups")
            .header("user-agent", "AetherTest/1.0")
    };
    let response = request().send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = response.json().await.unwrap();
    assert_eq!(payload["total"], 3);
    let items = payload["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    for (id, multiplier) in [("discount", 0.5), ("regular", 1.0), ("free", 0.0)] {
        assert_eq!(
            items.iter().find(|item| item["id"] == id).unwrap(),
            &json!({
                "id": id, "name": format!("{id}-name"),
                "billing_multiplier": multiplier, "is_default": false,
            })
        );
    }
    assert!(!payload.to_string().contains("private"));
    assert!(!payload.to_string().contains("secret-provider"));
    repository
        .update_routing_group(
            "private-default",
            UpdateRoutingGroupRecord {
                config_json: Some(json!({ "user_visible": true })),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let payload: serde_json::Value = request().send().await.unwrap().json().await.unwrap();
    assert_eq!(payload["total"], 4);
    assert_eq!(
        payload["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "private-default")
            .unwrap()["is_default"],
        true
    );
    assert_eq!(*upstream_hits.lock().unwrap(), 0);
    gateway.abort();
    upstream.abort();
}
