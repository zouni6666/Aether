use std::sync::{Arc, Mutex};

use aether_data::repository::users::InMemoryUserReadRepository;
use axum::body::Body;
use axum::routing::any;
use axum::{extract::Request, Router};
use http::StatusCode;
use serde_json::json;

use super::super::{build_router_with_state, start_server, AppState};
use crate::constants::{
    GATEWAY_HEADER, TRUSTED_ADMIN_SESSION_ID_HEADER, TRUSTED_ADMIN_USER_ID_HEADER,
    TRUSTED_ADMIN_USER_ROLE_HEADER,
};
use crate::data::GatewayDataState;

async fn assert_admin_billing_get_returns_local(
    path: &str,
) -> (serde_json::Value, Arc<Mutex<usize>>) {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        path,
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(AppState::new().expect("gateway should build"));
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}{path}"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    let status = response.status();
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    gateway_handle.abort();
    upstream_handle.abort();
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);
    assert_eq!(status, StatusCode::OK);
    (payload, upstream_hits)
}

async fn send_admin_billing_request(
    gateway_url: &str,
    method: http::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, format!("{gateway_url}{path}"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123");
    if let Some(body) = body {
        request = request.json(&body);
    }
    request.send().await.expect("request should succeed")
}

#[tokio::test]
async fn user_account_history_http_filters_wallet_and_plan_history() {
    use aether_data::repository::{
        billing::{InMemoryBillingReadRepository, UserPlanEntitlementRecord},
        users::StoredUserAuthRecord,
        wallet::{InMemoryWalletRepository, StoredWalletSnapshot},
    };

    let users = ["user-1", "user-2"].map(|id| {
        StoredUserAuthRecord::new(
            id.to_string(),
            Some(format!("{id}@example.com")),
            true,
            id.to_string(),
            Some("hash".to_string()),
            "user".to_string(),
            "local".to_string(),
            None,
            None,
            None,
            true,
            false,
            None,
            None,
        )
        .expect("user should build")
    });
    let wallets = ["user-1", "user-2"].map(|id| {
        StoredWalletSnapshot::new(
            format!("wallet-{id}"),
            Some(id.to_string()),
            None,
            12.5,
            2.5,
            "finite".to_string(),
            "USD".to_string(),
            "active".to_string(),
            30.0,
            10.0,
            3.0,
            1.5,
            1_710_000_000,
        )
        .expect("wallet should build")
    });
    let now = chrono::Utc::now().timestamp().max(0) as u64;
    let entitlements = [
        ("current", "user-1", "active"),
        ("revoked", "user-1", "revoked"),
        ("another-user", "user-2", "revoked"),
    ]
    .map(|(id, user_id, status)| UserPlanEntitlementRecord {
        id: id.to_string(),
        user_id: user_id.to_string(),
        plan_id: "plan-1".to_string(),
        payment_order_id: format!("order-{id}"),
        status: status.to_string(),
        starts_at_unix_secs: now - 60,
        expires_at_unix_secs: now + 3600,
        entitlements_snapshot: json!([]),
        created_at_unix_secs: now - 60,
        updated_at_unix_secs: now,
    });
    let state = AppState::new().unwrap().with_data_state_for_tests(
        GatewayDataState::with_user_billing_and_wallet_for_tests(
            Arc::new(InMemoryUserReadRepository::seed_auth_users(users)),
            Arc::new(InMemoryBillingReadRepository::seed_user_plan_entitlements(
                entitlements,
            )),
            Arc::new(InMemoryWalletRepository::seed(wallets)),
        ),
    );
    let (url, handle) = start_server(build_router_with_state(state)).await;
    let wallet_path = "/api/admin/wallets?user_id=user-1&owner_type=user&limit=1&offset=0";
    let response = send_admin_billing_request(&url, http::Method::GET, wallet_path, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let wallet: serde_json::Value = response.json().await.unwrap();
    assert_eq!(wallet["total"], 1);
    assert_eq!(wallet["items"].as_array().unwrap().len(), 1);
    assert_eq!(wallet["items"][0]["id"], "wallet-user-1");
    assert_eq!(wallet["items"][0]["user_id"], "user-1");

    let response = send_admin_billing_request(
        &url,
        http::Method::GET,
        "/api/admin/wallets?user_id=missing-user&limit=1",
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let missing: serde_json::Value = response.json().await.unwrap();
    assert_eq!(missing["total"], 0);
    assert_eq!(missing["items"], json!([]));

    let path = "/api/admin/users/user-1/billing/entitlements";
    for query in ["", "?include_inactive=false"] {
        let response =
            send_admin_billing_request(&url, http::Method::GET, &format!("{path}{query}"), None)
                .await;
        assert_eq!(response.status(), StatusCode::OK);
        let current: serde_json::Value = response.json().await.unwrap();
        assert_eq!(current["total"], 1);
        assert_eq!(current["items"][0]["id"], "current");
        assert_eq!(current["items"][0]["active"], true);
    }
    let response = send_admin_billing_request(
        &url,
        http::Method::GET,
        &format!("{path}?include_inactive=true"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let history: serde_json::Value = response.json().await.unwrap();
    assert_eq!(history["total"], 2);
    let items = history["items"].as_array().unwrap();
    assert!(items.iter().all(|item| item["user_id"] == "user-1"));
    assert!(items
        .iter()
        .any(|item| item["id"] == "revoked" && item["active"] == false));

    let response = send_admin_billing_request(
        &url,
        http::Method::GET,
        &format!("{path}?include_inactive=invalid"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    handle.abort();
}

#[tokio::test]
async fn gateway_handles_admin_billing_presets_locally_with_trusted_admin_principal() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/billing/presets",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(AppState::new().expect("gateway should build"));
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/billing/presets"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["items"][0]["name"], "aether-core");
    assert_eq!(payload["items"][0]["version"], "1.0");
    assert_eq!(payload["items"][0]["collector_count"], 16);
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_handles_admin_billing_apply_preset_locally_with_trusted_admin_principal() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/billing/presets/apply",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_admin_billing_collectors_for_tests(
                Vec::<crate::AdminBillingCollectorRecord>::new(),
            ),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let merge_response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/presets/apply",
        Some(json!({"preset":"aether-core","mode":"merge"})),
    )
    .await;
    assert_eq!(merge_response.status(), StatusCode::OK);
    let merge_payload: serde_json::Value =
        merge_response.json().await.expect("json body should parse");
    assert_eq!(merge_payload["ok"], json!(true));
    assert_eq!(merge_payload["preset"], json!("aether-core"));
    assert_eq!(merge_payload["mode"], json!("merge"));
    assert_eq!(merge_payload["created"], json!(16));
    assert_eq!(merge_payload["updated"], json!(0));
    assert_eq!(merge_payload["skipped"], json!(0));
    assert_eq!(merge_payload["errors"], json!([]));

    let merge_again_response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/presets/apply",
        Some(json!({"preset":"default","mode":"merge"})),
    )
    .await;
    assert_eq!(merge_again_response.status(), StatusCode::OK);
    let merge_again_payload: serde_json::Value = merge_again_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(merge_again_payload["ok"], json!(true));
    assert_eq!(merge_again_payload["preset"], json!("aether-core"));
    assert_eq!(merge_again_payload["created"], json!(0));
    assert_eq!(merge_again_payload["updated"], json!(0));
    assert_eq!(merge_again_payload["skipped"], json!(16));

    let overwrite_response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/presets/apply",
        Some(json!({"preset":"aether-core","mode":"overwrite"})),
    )
    .await;
    assert_eq!(overwrite_response.status(), StatusCode::OK);
    let overwrite_payload: serde_json::Value = overwrite_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(overwrite_payload["ok"], json!(true));
    assert_eq!(overwrite_payload["preset"], json!("aether-core"));
    assert_eq!(overwrite_payload["mode"], json!("overwrite"));
    assert_eq!(overwrite_payload["created"], json!(0));
    assert_eq!(overwrite_payload["updated"], json!(16));
    assert_eq!(overwrite_payload["skipped"], json!(0));
    assert_eq!(overwrite_payload["errors"], json!([]));
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_returns_conflict_for_admin_billing_apply_preset_when_backend_unavailable() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/billing/presets/apply",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let mut state = AppState::new()
        .expect("gateway should build")
        .with_data_state_for_tests(crate::data::GatewayDataState::with_user_reader_for_tests(
            Arc::new(InMemoryUserReadRepository::seed_auth_users(Vec::new())),
        ));
    state.admin_billing_collector_store = None;
    let gateway = build_router_with_state(state);
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/presets/apply",
        Some(json!({"preset":"aether-core","mode":"merge"})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["error_code"], "read_only_mode");
    assert_eq!(payload["detail"], "当前为只读模式，无法应用计费预设");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_handles_admin_billing_rule_routes_locally_with_trusted_admin_principal() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream_hits_clone_detail = Arc::clone(&upstream_hits);
    let upstream = Router::new()
        .route(
            "/api/admin/billing/rules",
            any(move |_request: Request| {
                let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
                async move {
                    *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                    (StatusCode::OK, Body::from("unexpected upstream hit"))
                }
            }),
        )
        .route(
            "/api/admin/billing/rules/rule-1",
            any(move |_request: Request| {
                let upstream_hits_inner = Arc::clone(&upstream_hits_clone_detail);
                async move {
                    *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                    (StatusCode::OK, Body::from("unexpected upstream hit"))
                }
            }),
        );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_admin_billing_rules_for_tests([
                crate::AdminBillingRuleRecord {
                    id: "rule-1".to_string(),
                    name: "chat-input".to_string(),
                    task_type: "chat".to_string(),
                    global_model_id: Some("global-gpt-5".to_string()),
                    model_id: Some("model-1".to_string()),
                    expression: "input_tokens * 0.000001".to_string(),
                    variables: json!({ "base": 1.5 }),
                    dimension_mappings: json!({
                        "input_tokens": { "source": "dimension", "dimension": "input_tokens" }
                    }),
                    is_enabled: true,
                    created_at_unix_ms: 1_710_000_000,
                    updated_at_unix_secs: 1_710_000_100,
                },
                crate::AdminBillingRuleRecord {
                    id: "rule-2".to_string(),
                    name: "image-output".to_string(),
                    task_type: "image".to_string(),
                    global_model_id: None,
                    model_id: Some("model-2".to_string()),
                    expression: "images * 0.01".to_string(),
                    variables: json!({}),
                    dimension_mappings: json!({
                        "images": { "source": "dimension", "dimension": "image_count" }
                    }),
                    is_enabled: false,
                    created_at_unix_ms: 1_710_000_000,
                    updated_at_unix_secs: 1_710_000_050,
                },
            ]),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let list_response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/billing/rules?task_type=chat&is_enabled=true&page=1&page_size=1"
        ))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value =
        list_response.json().await.expect("json body should parse");
    assert_eq!(list_payload["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(list_payload["items"][0]["id"], "rule-1");
    assert_eq!(list_payload["items"][0]["name"], "chat-input");
    assert_eq!(list_payload["items"][0]["task_type"], "chat");
    assert_eq!(list_payload["items"][0]["is_enabled"], true);
    assert_eq!(list_payload["total"], 1);
    assert_eq!(list_payload["page"], 1);
    assert_eq!(list_payload["page_size"], 1);
    assert_eq!(list_payload["pages"], 1);

    let detail_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/billing/rules/rule-1"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_payload: serde_json::Value = detail_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(detail_payload["id"], "rule-1");
    assert_eq!(detail_payload["name"], "chat-input");
    assert_eq!(detail_payload["task_type"], "chat");
    assert_eq!(detail_payload["global_model_id"], "global-gpt-5");
    assert_eq!(detail_payload["model_id"], "model-1");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);
    gateway_handle.abort();
    upstream_handle.abort();

    let upstream =
        Router::new().route(
            "/api/admin/billing/rules",
            any(|_request: Request| async move {
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }),
        );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(AppState::new().expect("gateway should build"));
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let create_response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/rules",
        Some(json!({
            "name": "rule",
            "task_type": "chat",
            "model_id": "model-1",
            "expression": "input_tokens * 0.000001",
            "variables": { "base": 1.5 },
            "dimension_mappings": {
                "input_tokens": { "source": "dimension", "dimension": "input_tokens" }
            },
            "is_enabled": true
        })),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_payload: serde_json::Value = create_response
        .json()
        .await
        .expect("json body should parse");
    let rule_id = create_payload["id"]
        .as_str()
        .expect("rule id should exist")
        .to_string();
    assert_eq!(create_payload["name"], json!("rule"));
    assert_eq!(create_payload["model_id"], json!("model-1"));

    let update_response = send_admin_billing_request(
        &gateway_url,
        http::Method::PUT,
        &format!("/api/admin/billing/rules/{rule_id}"),
        Some(json!({
            "name": "rule-updated",
            "task_type": "video",
            "global_model_id": "global-model-1",
            "expression": "max(output_tokens, 1)",
            "variables": { "factor": 2 },
            "dimension_mappings": {
                "output_tokens": { "source": "dimension", "dimension": "output_tokens" }
            },
            "is_enabled": false
        })),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_payload: serde_json::Value = update_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(update_payload["id"], json!(rule_id));
    assert_eq!(update_payload["name"], json!("rule-updated"));
    assert_eq!(update_payload["task_type"], json!("video"));
    assert_eq!(update_payload["global_model_id"], json!("global-model-1"));
    assert_eq!(update_payload["model_id"], serde_json::Value::Null);
    assert_eq!(update_payload["is_enabled"], json!(false));

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_returns_conflict_for_admin_billing_rule_create_when_backend_unavailable() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/billing/rules",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let mut state = AppState::new()
        .expect("gateway should build")
        .with_data_state_for_tests(crate::data::GatewayDataState::with_user_reader_for_tests(
            Arc::new(InMemoryUserReadRepository::seed_auth_users(Vec::new())),
        ));
    state.admin_billing_rule_store = None;
    let gateway = build_router_with_state(state);
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/rules",
        Some(json!({
            "name": "rule",
            "task_type": "chat",
            "model_id": "model-1",
            "expression": "input_tokens * 0.000001",
            "variables": {},
            "dimension_mappings": {},
            "is_enabled": true
        })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["error_code"], "read_only_mode");
    assert_eq!(payload["detail"], "当前为只读模式，无法创建计费规则");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_handles_admin_billing_collector_routes_locally_with_trusted_admin_principal() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream_hits_clone_list = Arc::clone(&upstream_hits);
    let upstream = Router::new()
        .route(
            "/api/admin/billing/collectors",
            any(move |_request: Request| {
                let upstream_hits_inner = Arc::clone(&upstream_hits_clone_list);
                async move {
                    *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                    (StatusCode::OK, Body::from("unexpected upstream hit"))
                }
            }),
        )
        .route(
            "/api/admin/billing/collectors/collector-1",
            any(move |_request: Request| {
                let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
                async move {
                    *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                    (StatusCode::OK, Body::from("unexpected upstream hit"))
                }
            }),
        );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_admin_billing_collectors_for_tests([
                crate::AdminBillingCollectorRecord {
                    id: "collector-1".to_string(),
                    api_format: "OPENAI".to_string(),
                    task_type: "chat".to_string(),
                    dimension_name: "tokens".to_string(),
                    source_type: "request".to_string(),
                    source_path: Some("usage.input_tokens".to_string()),
                    value_type: "float".to_string(),
                    transform_expression: None,
                    default_value: None,
                    priority: 10,
                    is_enabled: true,
                    created_at_unix_ms: 1_710_000_000,
                    updated_at_unix_secs: 1_710_000_100,
                },
                crate::AdminBillingCollectorRecord {
                    id: "collector-2".to_string(),
                    api_format: "OPENAI".to_string(),
                    task_type: "chat".to_string(),
                    dimension_name: "latency".to_string(),
                    source_type: "computed".to_string(),
                    source_path: None,
                    value_type: "int".to_string(),
                    transform_expression: Some("max(latency_ms, 1)".to_string()),
                    default_value: Some("1".to_string()),
                    priority: 5,
                    is_enabled: false,
                    created_at_unix_ms: 1_710_000_000,
                    updated_at_unix_secs: 1_710_000_050,
                },
            ]),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;
    let list_response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/billing/collectors?api_format=OPENAI&task_type=chat&is_enabled=true&page=1&page_size=10"
        ))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value =
        list_response.json().await.expect("json body should parse");
    assert_eq!(list_payload["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(list_payload["items"][0]["id"], "collector-1");
    assert_eq!(list_payload["items"][0]["dimension_name"], "tokens");
    assert_eq!(list_payload["total"], 1);
    assert_eq!(list_payload["page"], 1);
    assert_eq!(list_payload["page_size"], 10);
    assert_eq!(list_payload["pages"], 1);

    let detail_response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/billing/collectors/collector-1"
        ))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_payload: serde_json::Value = detail_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(detail_payload["id"], "collector-1");
    assert_eq!(detail_payload["api_format"], "OPENAI");
    assert_eq!(detail_payload["source_path"], "usage.input_tokens");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);
    gateway_handle.abort();
    upstream_handle.abort();

    let upstream =
        Router::new().route(
            "/api/admin/billing/collectors",
            any(|_request: Request| async move {
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }),
        );
    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(AppState::new().expect("gateway should build"));
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let create_response = send_admin_billing_request(
        &gateway_url,
        http::Method::POST,
        "/api/admin/billing/collectors",
        Some(json!({
            "api_format": "openai",
            "task_type": "chat",
            "dimension_name": "tokens",
            "source_type": "request",
            "source_path": "usage.input_tokens",
            "value_type": "float",
            "priority": 10,
            "is_enabled": true
        })),
    )
    .await;
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_payload: serde_json::Value = create_response
        .json()
        .await
        .expect("json body should parse");
    let collector_id = create_payload["id"]
        .as_str()
        .expect("collector id should exist")
        .to_string();
    assert_eq!(create_payload["api_format"], json!("OPENAI"));
    assert_eq!(create_payload["source_path"], json!("usage.input_tokens"));

    let update_response = send_admin_billing_request(
        &gateway_url,
        http::Method::PUT,
        &format!("/api/admin/billing/collectors/{collector_id}"),
        Some(json!({
            "api_format": "openai",
            "task_type": "chat",
            "dimension_name": "tokens",
            "source_type": "computed",
            "source_path": null,
            "value_type": "int",
            "transform_expression": "max(total_tokens, 1)",
            "default_value": "1",
            "priority": 12,
            "is_enabled": false
        })),
    )
    .await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_payload: serde_json::Value = update_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(update_payload["id"], json!(collector_id));
    assert_eq!(update_payload["source_type"], json!("computed"));
    assert_eq!(update_payload["source_path"], serde_json::Value::Null);
    assert_eq!(
        update_payload["transform_expression"],
        json!("max(total_tokens, 1)")
    );
    assert_eq!(update_payload["default_value"], json!("1"));
    assert_eq!(update_payload["is_enabled"], json!(false));

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn provider_expense_http_contract_requires_admin_and_records_retries_only_once() {
    use aether_data::repository::{
        billing::InMemoryBillingReadRepository,
        provider_catalog::InMemoryProviderCatalogReadRepository,
    };
    use aether_data_contracts::repository::provider_catalog::StoredProviderCatalogProvider;
    let provider = StoredProviderCatalogProvider::new(
        "provider-1".into(),
        "=Supplier".into(),
        None,
        "custom".into(),
    )
    .unwrap();
    let catalog = Arc::new(InMemoryProviderCatalogReadRepository::seed(
        vec![provider],
        vec![],
        vec![],
    ));
    let state = AppState::new().unwrap().with_data_state_for_tests(
        GatewayDataState::with_billing_reader_for_tests(Arc::new(
            InMemoryBillingReadRepository::default(),
        ))
        .with_provider_catalog_reader(catalog),
    );
    let (url, handle) = start_server(build_router_with_state(state)).await;
    let path = "/api/admin/billing/provider-expenses";
    let client = reqwest::Client::new();
    assert!(matches!(
        client
            .get(format!("{url}{path}"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ));
    let payload = json!({"client_request_id":uuid::Uuid::new_v4().to_string(),"provider_id":"provider-1","kind":"subscription","amount":"12.30","currency":"USD","paid_at":"2026-09-20T00:00:00Z","period_start":"2026-09-20T00:00:00Z","period_end":"2026-10-20T00:00:00Z","note":"=SUM(1,2)"});
    let first =
        send_admin_billing_request(&url, http::Method::POST, path, Some(payload.clone())).await;
    assert_eq!(first.status(), StatusCode::OK);
    let first: serde_json::Value = first.json().await.unwrap();
    assert_eq!(first["item"]["amount"], "12.30000000");
    let again =
        send_admin_billing_request(&url, http::Method::POST, path, Some(payload.clone())).await;
    assert_eq!(again.status(), StatusCode::OK);
    let again: serde_json::Value = again.json().await.unwrap();
    assert_eq!(first["item"]["id"], again["item"]["id"]);
    let mut conflict = payload.clone();
    conflict["amount"] = json!("13");
    assert_eq!(
        send_admin_billing_request(&url, http::Method::POST, path, Some(conflict))
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let range = "?from=2026-09-01T00:00:00Z&to=2026-10-01T00:00:00Z&limit=1&offset=5";
    let page =
        send_admin_billing_request(&url, http::Method::GET, &format!("{path}{range}"), None).await;
    assert_eq!(page.status(), StatusCode::OK);
    let page: serde_json::Value = page.json().await.unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"], json!([]));
    assert_eq!(page["totals"][0]["subscription_amount"], "12.30000000");
    let csv = send_admin_billing_request(
        &url,
        http::Method::GET,
        &format!("{path}{range}&format=csv"),
        None,
    )
    .await;
    assert_eq!(csv.status(), StatusCode::OK);
    let csv = csv.text().await.unwrap();
    assert!(csv.contains("'=Supplier"));
    assert!(csv.contains("'=SUM(1,2)"));
    assert!(csv.contains("12.30000000"));
    let accounts = send_admin_billing_request(
        &url,
        http::Method::GET,
        "/api/admin/billing/provider-accounts",
        None,
    )
    .await;
    assert_eq!(accounts.status(), StatusCode::OK);
    let accounts: serde_json::Value = accounts.json().await.unwrap();
    assert_eq!(accounts["items"][0]["provider_id"], "provider-1");
    assert!(accounts["items"][0]["balance"].is_null());
    let void_path = format!("{path}/{}/void", first["item"]["id"].as_str().unwrap());
    let voided = send_admin_billing_request(&url, http::Method::POST, &void_path, None).await;
    assert_eq!(voided.status(), StatusCode::OK);
    let voided: serde_json::Value = voided.json().await.unwrap();
    assert_eq!(voided["item"]["status"], "void");
    let again = send_admin_billing_request(&url, http::Method::POST, &void_path, None).await;
    assert_eq!(again.status(), StatusCode::OK);
    let again: serde_json::Value = again.json().await.unwrap();
    assert_eq!(voided, again);
    let page: serde_json::Value =
        send_admin_billing_request(&url, http::Method::GET, &format!("{path}{range}"), None)
            .await
            .json()
            .await
            .unwrap();
    assert_eq!(page["total"], 0);
    assert_eq!(page["totals"], json!([]));
    handle.abort();
}
