use super::*;

const RANGE: &str = "from=2026-09-01T23:45:00Z&to=2026-09-02T00:15:00Z&timezone=Asia%2FShanghai";

#[tokio::test]
async fn overview_live_rates_use_only_the_last_sixty_seconds() {
    let now = chrono::Utc::now().timestamp();
    let recent = sample_usage_row(
        "live-recent",
        "live-recent-request",
        None,
        None,
        None,
        "Provider",
        "model-1",
        100,
        20,
        1.0,
        0.8,
        now - 10,
    );
    let historical = sample_usage_row(
        "live-earlier",
        "live-earlier-request",
        None,
        None,
        None,
        "Provider",
        "model-1",
        1000,
        200,
        10.0,
        8.0,
        now - 120,
    );
    let repository = Arc::new(InMemoryUsageReadRepository::seed([recent, historical]));
    let gateway = build_router_with_state(
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(GatewayDataState::with_usage_reader_for_tests(repository)),
    );
    let (url, handle) = start_server(gateway).await;
    let response = admin_request(
        reqwest::Client::new().get(format!("{url}/api/admin/overview/operations/live")),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: serde_json::Value = response.json().await.unwrap();
    let activity = &value["data"]["recent_activity"]["data"];
    assert_eq!(activity["window_seconds"], 60);
    assert_eq!(activity["requests_per_minute"], 1);
    assert_eq!(activity["tokens_per_minute"], 120);
    assert_eq!(activity["requests_per_second"], 1.0 / 60.0);
    handle.abort();
}

#[tokio::test]
async fn overview_dashboard_summary_reads_only_the_new_collection_and_all_current_users() {
    let now = chrono::Utc::now();
    let since = now - chrono::Duration::minutes(1);
    let mut recent = sample_usage_row(
        "dashboard-new-row",
        "dashboard-new-request",
        Some("user-1"),
        Some("key-1"),
        Some("Personal"),
        "Provider",
        "model-1",
        100,
        20,
        1.0,
        0.8,
        now.timestamp(),
    );
    recent.request_metadata = Some(json!({"analytics_attribution":{"is_standalone":false}}));
    let mut historical = recent.clone();
    historical.id = "dashboard-old-row".into();
    historical.request_id = "dashboard-old-request".into();
    historical.created_at_unix_ms = (now - chrono::Duration::days(500)).timestamp() as u64;
    let repository = Arc::new(
        InMemoryUsageReadRepository::seed([recent, historical])
            .with_dashboard_stats_since(since)
            .with_analytics_users([
                sample_user_summary("user-1", "Alice", "user", true),
                sample_user_summary("user-2", "Bob", "user", false),
            ]),
    );
    let gateway = build_router_with_state(
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(GatewayDataState::with_usage_reader_for_tests(repository)),
    );
    let (url, handle) = start_server(gateway).await;
    let client = reqwest::Client::new();
    let endpoint = format!("{url}/api/admin/overview/dashboard/summary");
    assert!(matches!(
        client.get(&endpoint).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ));
    let response = admin_request(client.get(format!("{endpoint}?timezone=UTC")))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: serde_json::Value = response.json().await.unwrap();
    assert_eq!(value["stats_since"], since.to_rfc3339());
    assert_eq!(value["today"]["request_count"], 1);
    assert_eq!(value["total"]["request_count"], 1);
    assert_eq!(value["today"]["input_tokens"], 100);
    assert_eq!(value["today"]["output_tokens"], 20);
    assert_eq!(value["total"]["total_tokens"], 120);
    assert_eq!(value["total"]["billable_amount"]["value"], "0.80000000");
    assert_eq!(value["users"]["total"], 2);
    assert_eq!(value["active_days"], 1);
    assert_eq!(value["concurrency"]["scope"], "node");
    assert!(value["today"].get("latency_p95_ms").is_none());
    let invalid = admin_request(client.get(format!("{endpoint}?timezone=invalid")))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    handle.abort();
}

#[tokio::test]
async fn overview_dashboard_keeps_today_and_lifetime_totals_separate() {
    let now = chrono::Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let row = sample_usage_row(
        "today-row",
        "today-request",
        Some("user-1"),
        Some("key-1"),
        Some("Personal"),
        "Provider",
        "model-1",
        100,
        20,
        1.0,
        0.8,
        now.timestamp()
            .saturating_sub(1)
            .max(today_start.timestamp()),
    );
    let mut historical = row.clone();
    historical.id = "historical-row".into();
    historical.request_id = "historical-request".into();
    historical.created_at_unix_ms = (now - chrono::Duration::days(500)).timestamp() as u64;
    let mut future = row.clone();
    future.id = "future-row".into();
    future.request_id = "future-request".into();
    future.created_at_unix_ms = (now + chrono::Duration::days(1)).timestamp() as u64;
    let repository = Arc::new(
        InMemoryUsageReadRepository::seed([row, historical, future]).with_analytics_users([
            sample_user_summary("user-1", "Alice", "user", true),
            sample_user_summary("user-2", "Bob", "user", true),
        ]),
    );
    let gateway = build_router_with_state(
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(GatewayDataState::with_usage_reader_for_tests(repository)),
    );
    let (url, handle) = start_server(gateway).await;
    let client = reqwest::Client::new();
    let endpoint = format!("{url}/api/admin/overview/dashboard");
    let anonymous = client.get(&endpoint).send().await.unwrap();
    assert!(matches!(
        anonymous.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ));
    let response = admin_request(client.get(format!("{endpoint}?timezone=UTC")))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers()[http::header::CACHE_CONTROL]
        .to_str()
        .unwrap()
        .contains("no-store"));
    let data: serde_json::Value = response.json().await.unwrap();
    assert_eq!(data["today"]["data"]["request_count"], 1);
    assert_eq!(data["total"]["data"]["request_count"], 2);
    assert_eq!(data["today"]["data"]["total_tokens"], 120);
    assert_eq!(data["total"]["data"]["total_tokens"], 240);
    assert_eq!(
        data["today"]["data"]["billable_amount"]["value"],
        "0.80000000"
    );
    assert_eq!(
        data["total"]["data"]["billable_amount"]["value"],
        "1.60000000"
    );
    assert_eq!(data["today"]["data"]["enabled_users"], 2);
    assert_eq!(data["today"]["meta"]["range"]["timezone"], "UTC");
    assert_eq!(data["total"]["meta"]["range"]["period"], "all_time");
    assert_eq!(
        data["today"]["meta"]["read_revision"],
        data["total"]["meta"]["read_revision"]
    );
    assert_eq!(
        data["today"]["meta"]["range"]["to"],
        data["total"]["meta"]["range"]["to"]
    );
    let total_endpoint = format!("{endpoint}/total");
    let anonymous_total = client.get(&total_endpoint).send().await.unwrap();
    assert!(matches!(
        anonymous_total.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ));
    let pending = admin_request(client.get(format!("{total_endpoint}?timezone=UTC")))
        .send()
        .await
        .unwrap();
    assert_eq!(pending.status(), StatusCode::ACCEPTED);
    assert_eq!(
        pending.json::<serde_json::Value>().await.unwrap()["status"],
        "pending"
    );
    let ready = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let response =
                admin_request(client.get(format!("{total_endpoint}?timezone=Asia%2FShanghai")))
                    .send()
                    .await
                    .unwrap();
            if response.status() == StatusCode::OK {
                break response.json::<serde_json::Value>().await.unwrap();
            }
            assert_eq!(response.status(), StatusCode::ACCEPTED);
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(ready["status"], "ready");
    assert_eq!(ready["stale"], false);
    assert_eq!(ready["total"]["data"]["request_count"], 2);
    assert_eq!(ready["total"]["data"]["total_tokens"], 240);
    assert_eq!(
        ready["total"]["data"]["billable_amount"]["value"],
        "1.60000000"
    );
    assert_eq!(ready["total"]["meta"]["range"]["timezone"], "Asia/Shanghai");
    let cached_utc = admin_request(client.get(format!("{total_endpoint}?timezone=UTC")))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(
        cached_utc["total"]["meta"]["read_revision"],
        ready["total"]["meta"]["read_revision"]
    );
    assert_eq!(
        cached_utc["total"]["meta"]["generated_at"],
        ready["total"]["meta"]["generated_at"]
    );
    for query in [
        "from=2020-01-01T00:00:00Z",
        "user_id=user-1",
        "timezone=invalid",
        "timezone=UTC&timezone=UTC",
    ] {
        let response = admin_request(client.get(format!("{endpoint}?{query}")))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
    }
    handle.abort();
}

#[tokio::test]
async fn overview_preserves_scope_pagination_amounts_and_csv_across_precise_range() {
    let from = chrono::DateTime::parse_from_rfc3339("2026-09-01T23:45:00Z")
        .unwrap()
        .timestamp();
    let mut row = sample_usage_row(
        "row-1",
        "request-1",
        Some("user-1"),
        Some("key-1"),
        Some("Personal"),
        "Provider",
        "model-1",
        100,
        20,
        1.0,
        0.8,
        from + 60,
    );
    row.request_metadata =
        Some(json!({"analytics_attribution": {"is_standalone": false, "record_kind": "request"}}));
    let mut outside = row.clone();
    outside.id = "row-outside".into();
    outside.request_id = "request-outside".into();
    outside.created_at_unix_ms = (from + 30 * 60) as u64;
    let repository = Arc::new(
        InMemoryUsageReadRepository::seed([row, outside]).with_analytics_users([
            sample_user_summary("user-1", "Alice", "user", true),
            sample_user_summary("user-2", "Bob", "user", true),
        ]),
    );
    let gateway = build_router_with_state(
        AppState::new()
            .unwrap()
            .with_data_state_for_tests(GatewayDataState::with_usage_reader_for_tests(repository)),
    );
    let (url, handle) = start_server(gateway).await;
    let client = reqwest::Client::new();
    let anonymous = client
        .get(format!("{url}/api/admin/overview/summary?{RANGE}"))
        .send()
        .await
        .unwrap();
    assert!(matches!(
        anonymous.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ));

    let summary = admin_request(client.get(format!("{url}/api/admin/overview/summary?{RANGE}")))
        .send()
        .await
        .unwrap();
    assert_eq!(summary.status(), StatusCode::OK);
    assert!(summary
        .headers()
        .get(http::header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("no-store"));
    let summary: serde_json::Value = summary.json().await.unwrap();
    assert_eq!(summary["data"]["request_count"], 1);
    assert_eq!(summary["data"]["billable_amount"]["value"], "0.80000000");
    assert_eq!(summary["data"]["quota_covered_amount"]["status"], "unknown");
    assert_eq!(summary["meta"]["range"]["time_basis"], "request_started_at");

    let charts = admin_request(client.get(format!(
        "{url}/api/admin/overview/dashboard/charts?{RANGE}&granularity=day"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(charts.status(), StatusCode::OK);
    let charts: serde_json::Value = charts.json().await.unwrap();
    assert_eq!(charts["data"]["summary"]["request_count"], 1);
    assert_eq!(charts["data"]["series"].as_array().unwrap().len(), 1);
    assert_eq!(charts["data"]["models"][0]["id"], "model-1");
    assert_eq!(
        charts["data"]["models"][0]["billable_amount"]["value"],
        "0.80000000"
    );
    assert_eq!(
        charts["data"]["providers"][0]["billable_amount"]["value"],
        "0.80000000"
    );

    let users = admin_request(client.get(format!(
        "{url}/api/admin/overview/users?{RANGE}&limit=1&offset=1&sort=request_count"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(users.status(), StatusCode::OK);
    let users: serde_json::Value = users.json().await.unwrap();
    assert_eq!(users["data"]["total"], 2);
    assert_eq!(users["data"]["items"][0]["user_id"], "user-2");
    assert_eq!(users["data"]["items"][0]["request_count"], 0);
    assert_eq!(users["data"]["summary"]["user_count"], 2);
    assert_eq!(users["data"]["summary"]["active_user_count"], 1);
    assert_eq!(users["data"]["summary"]["request_count"], 1);
    assert_eq!(
        users["data"]["summary"]["billable_amount"]["value"],
        "0.80000000"
    );
    assert!(users["data"]["finance_summary"].is_null());
    assert!(users["data"]["items"][0]["finance"].is_null());

    let csv = admin_request(client.get(format!(
        "{url}/api/admin/overview/users?{RANGE}&format=csv&limit=1"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(csv.status(), StatusCode::OK);
    let csv = csv.text().await.unwrap();
    assert!(csv.contains("Alice") && csv.contains("Bob"));
    assert!(csv.contains("finance.recharge_amount.value"));
    assert!(csv.contains("finance.plan_purchase_amount.value"));

    let detail =
        admin_request(client.get(format!("{url}/api/admin/overview/users/user-1?{RANGE}")))
            .send()
            .await
            .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail: serde_json::Value = detail.json().await.unwrap();
    assert_eq!(detail["meta"]["scope"]["kind"], "credential_owner");
    assert_eq!(detail["data"]["summary"]["request_count"], 1);
    assert!(detail["data"]["finance"].is_null());
    assert!(detail["data"]["payments"].is_null());

    let conflicting = admin_request(client.get(format!(
        "{url}/api/admin/overview/users/user-1?{RANGE}&user_id=user-2&payment_limit=2"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(conflicting.status(), StatusCode::BAD_REQUEST);

    let consumption = admin_request(client.get(format!(
        "{url}/api/admin/overview/consumption?{RANGE}&status=success&sort=started_at"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(consumption.status(), StatusCode::OK);
    let consumption: serde_json::Value = consumption.json().await.unwrap();
    assert_eq!(consumption["data"]["items"][0]["id"], "row-1");
    assert_eq!(consumption["data"]["items"][0]["request_id"], "request-1");

    for (filter, expected) in [
        (
            "api_key_id=key-1&provider_id=provider-1&request_id=request-1&status=success",
            1,
        ),
        ("api_key_id=other-key", 0),
        ("user_id=user-1&attribution_kind=employee", 1),
        ("user_id=user-2&attribution_kind=employee", 0),
        ("provider_id=other-provider", 0),
        ("slow_threshold_ms=5000", 0),
        (
            "slow_threshold_ms=350&is_stream=false&endpoint_kind=chat",
            1,
        ),
    ] {
        let records = admin_request(client.get(format!(
            "{url}/api/admin/usage/records?{RANGE}&{filter}&include_total=true"
        )))
        .send()
        .await
        .unwrap();
        assert_eq!(records.status(), StatusCode::OK, "filter: {filter}");
        let records: serde_json::Value = records.json().await.unwrap();
        assert_eq!(records["total"], expected, "filter: {filter}");
    }

    let invalid = admin_request(client.get(format!(
        "{url}/api/admin/overview/summary?{RANGE}&made_up=1"
    )))
    .send()
    .await
    .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    handle.abort();
}
