use super::api::{public_projection, AdminAttemptMetrics, AdminHealthObject, HealthRequest};
use super::policy::{overall_status, HealthPolicy, HealthRatio, HealthStatus};
use super::publication::HealthPublication;
use aether_data_contracts::repository::usage::{
    HealthObservationMetrics, HealthObservationObjectKind,
};
use serde_json::json;

#[test]
fn health_policy_distinguishes_unknown_and_insufficient_samples() {
    let policy = HealthPolicy::default();
    assert_eq!(policy.status(0, 0, 0), HealthStatus::Unknown);
    assert_eq!(policy.status(19, 0, 0), HealthStatus::Unknown);
    assert_eq!(policy.status(100, 0, 1), HealthStatus::Unknown);
    assert_eq!(policy.status(0, 100, 1), HealthStatus::Unavailable);
    assert_eq!(policy.status(99, 1, 0), HealthStatus::Healthy);
    assert_eq!(policy.status(96, 4, 0), HealthStatus::Degraded);
    assert_eq!(policy.status(90, 10, 0), HealthStatus::Unavailable);
    assert_eq!(HealthRatio::new(0, 0).value, None);
    assert_eq!(
        overall_status([HealthStatus::Healthy, HealthStatus::Unknown].into_iter()),
        HealthStatus::Unknown
    );
}

#[test]
fn health_query_rejects_ambiguous_and_internal_public_filters() {
    assert!(HealthRequest::parse(Some("provider_id=secret"), true, 30_000_000).is_err());
    assert!(HealthRequest::parse(Some("kind=provider"), true, 30_000_000).is_err());
    assert!(HealthRequest::parse(Some("window=6h&window=24h"), false, 30_000_000).is_err());
    assert!(HealthRequest::parse(Some("limit=101"), false, 30_000_000).is_err());
    let query = HealthRequest::parse(
        Some("kind=model&window=1h&limit=50&offset=25"),
        true,
        30_000_000,
    )
    .unwrap();
    assert_eq!(query.from_unix_ms, 26_400_000);
    assert_eq!(query.to_unix_ms, 30_000_000);
    assert_eq!(query.offset, 25);
}

#[test]
fn public_publication_is_explicit_and_rejects_provider_and_duplicate_sources() {
    let config: HealthPublication = serde_json::from_value(json!({"enabled": true, "objects": [
        {"public_id": "chat", "kind": "api_format", "value": "openai:chat", "display_name": "Chat"}
    ]}))
    .unwrap();
    assert!(config.validate().is_ok());
    let mut duplicated = config.clone();
    duplicated.objects.push(duplicated.objects[0].clone());
    assert!(duplicated.validate().is_err());
    let mut provider = config;
    provider.objects[0].kind = HealthObservationObjectKind::Provider;
    assert!(provider.validate().is_err());
    assert!(serde_json::from_value::<HealthPublication>(
        json!({"enabled": true, "objects": [], "publish_all": true})
    )
    .is_err());
}

#[test]
fn public_dto_cannot_serialize_internal_source_or_attempts() {
    let metrics = HealthObservationMetrics {
        request_count: 105,
        succeeded_count: 100,
        failed_count: 4,
        cancelled_count: 1,
        service_succeeded_count: 100,
        service_failed_count: 2,
        excluded_count: 2,
        unknown_failure_count: 1,
        ..Default::default()
    };
    let public = public_projection(
        "chat".into(),
        HealthObservationObjectKind::ApiFormat,
        "Chat".into(),
        &metrics,
        &HealthPolicy::default(),
    );
    assert_eq!(public.request_success.denominator, 105);
    assert_eq!(public.service_availability.denominator, 102);
    let admin = AdminHealthObject {
        service: public.clone(),
        source_value: "internal-provider-id".into(),
        attempts: AdminAttemptMetrics {
            succeeded_count: 100,
            failed_count: 20,
            in_progress_count: 1,
            cancelled_count: 1,
            success: HealthRatio::new(100, 120),
        },
    };
    let public_json = serde_json::to_value(public).unwrap();
    assert!(public_json.get("source_value").is_none());
    assert!(public_json.get("attempts").is_none());
    assert!(!public_json.to_string().contains("internal-provider-id"));
    assert!(serde_json::to_value(admin)
        .unwrap()
        .get("attempts")
        .is_some());
}
