use http::Uri;

use super::{classify_control_route, headers};

#[test]
fn classifies_admin_stats_provider_quota_usage_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/providers/quota-usage"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("provider_quota_usage"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_comparison_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/comparison"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("comparison"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_error_distribution_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/errors/distribution"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("error_distribution"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_performance_percentiles_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/performance/percentiles"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("performance_percentiles")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_provider_performance_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/performance/providers"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("provider_performance"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_cost_forecast_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/cost/forecast"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("cost_forecast"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_leaderboard_api_keys_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/leaderboard/api-keys"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("leaderboard_api_keys"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_leaderboard_models_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/leaderboard/models"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("leaderboard_models"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_leaderboard_user_groups_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/leaderboard/user-groups"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("leaderboard_user_groups")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_leaderboard_users_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/leaderboard/users"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("leaderboard_users"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_cost_savings_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/cost/savings"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("cost_savings"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_stats_time_series_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/stats/time-series"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("stats_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("time_series"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:stats")
    );
    assert!(!decision.is_execution_runtime_candidate());
}
