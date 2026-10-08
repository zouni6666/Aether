use super::policy::{overall_status, HealthPolicy, HealthRatio, HealthStatus};
use super::publication::{HealthPublication, PUBLICATION_KEY};
use crate::handlers::shared::unix_ms_to_rfc3339;
use crate::{AppState, GatewayError};
use aether_data_contracts::repository::global_models::AdminGlobalModelListQuery;
use aether_data_contracts::repository::usage::{
    HealthObservationMetrics, HealthObservationObjectKind, HealthObservationQuery,
};
use axum::{body::Body, http::StatusCode, response::IntoResponse, response::Response, Json};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HealthAudience {
    Admin,
    Authenticated,
    Public,
}

#[derive(Clone, Debug)]
pub(super) struct HealthRequest {
    pub kind: HealthObservationObjectKind,
    pub from_unix_ms: u64,
    pub to_unix_ms: u64,
    pub limit: usize,
    pub offset: usize,
}

impl HealthRequest {
    pub fn parse(query: Option<&str>, public: bool, now_ms: u64) -> Result<Self, &'static str> {
        let mut params = BTreeMap::new();
        for (key, value) in url::form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            if !matches!(key.as_ref(), "kind" | "window" | "limit" | "offset") {
                return Err("Unsupported health query parameter");
            }
            if params
                .insert(key.into_owned(), value.into_owned())
                .is_some()
            {
                return Err("Duplicate health query parameter");
            }
        }
        let kind = match params
            .get("kind")
            .map(String::as_str)
            .unwrap_or("api_format")
        {
            "api_format" => HealthObservationObjectKind::ApiFormat,
            "model" => HealthObservationObjectKind::Model,
            "provider" if !public => HealthObservationObjectKind::Provider,
            _ => return Err("Unsupported health object kind"),
        };
        let hours = match params.get("window").map(String::as_str).unwrap_or("6h") {
            "1h" => 1,
            "6h" => 6,
            "24h" => 24,
            "72h" => 72,
            _ => return Err("Health window must be 1h, 6h, 24h or 72h"),
        };
        let parse_size = |name: &str, default: usize| -> Result<usize, &'static str> {
            params
                .get(name)
                .map(|value| value.parse().map_err(|_| "Invalid pagination value"))
                .unwrap_or(Ok(default))
        };
        let limit = parse_size("limit", 25)?;
        let offset = parse_size("offset", 0)?;
        if !(1..=100).contains(&limit) || offset > 10_000 {
            return Err("Health pagination exceeds the supported range");
        }
        Ok(Self {
            kind,
            from_unix_ms: now_ms.saturating_sub(hours * 3_600_000),
            to_unix_ms: now_ms,
            limit,
            offset,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct HealthCoverage {
    pub status: &'static str,
    pub sample_status: &'static str,
    pub classified_count: u64,
    pub unknown_failure_count: u64,
    pub excluded_count: u64,
    pub exclusion_policy: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct PublicHealthObject {
    pub id: String,
    pub kind: HealthObservationObjectKind,
    pub name: String,
    pub status: HealthStatus,
    pub request_count: u64,
    pub request_success: HealthRatio,
    pub service_availability: HealthRatio,
    pub coverage: HealthCoverage,
    pub average_latency_ms: Option<f64>,
    pub latency_sample_count: u64,
    pub last_request_at: Option<String>,
    pub timeline: Vec<PublicHealthBucket>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct PublicHealthBucket {
    pub from: String,
    pub to: String,
    pub status: HealthStatus,
    pub service_availability: HealthRatio,
    pub unknown_failure_count: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct AdminHealthObject {
    #[serde(flatten)]
    pub service: PublicHealthObject,
    pub source_value: String,
    pub attempts: AdminAttemptMetrics,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct AdminAttemptMetrics {
    pub succeeded_count: u64,
    pub failed_count: u64,
    pub in_progress_count: u64,
    pub cancelled_count: u64,
    pub success: HealthRatio,
}

pub(super) fn public_projection(
    id: String,
    kind: HealthObservationObjectKind,
    name: String,
    metrics: &HealthObservationMetrics,
    policy: &HealthPolicy,
) -> PublicHealthObject {
    let classified = metrics
        .service_succeeded_count
        .saturating_add(metrics.service_failed_count);
    PublicHealthObject {
        id,
        kind,
        name,
        status: policy.status(
            metrics.service_succeeded_count,
            metrics.service_failed_count,
            metrics.unknown_failure_count,
        ),
        request_count: metrics.request_count,
        request_success: HealthRatio::new(
            metrics.succeeded_count,
            metrics
                .succeeded_count
                .saturating_add(metrics.failed_count)
                .saturating_add(metrics.cancelled_count),
        ),
        service_availability: HealthRatio::new(metrics.service_succeeded_count, classified),
        coverage: HealthCoverage {
            status: if metrics.unknown_failure_count > 0 {
                "partial"
            } else {
                "complete"
            },
            sample_status: if classified == 0 {
                "empty"
            } else if classified < policy.minimum_samples {
                "insufficient"
            } else {
                "sufficient"
            },
            classified_count: classified,
            unknown_failure_count: metrics.unknown_failure_count,
            excluded_count: metrics.excluded_count,
            exclusion_policy: "client_cancelled_invalid_input_identity_or_quota_policy",
        },
        average_latency_ms: (metrics.latency_sample_count > 0)
            .then(|| metrics.latency_sum_ms / metrics.latency_sample_count as f64),
        latency_sample_count: metrics.latency_sample_count,
        last_request_at: metrics.last_request_at_unix_ms.and_then(unix_ms_to_rfc3339),
        timeline: Vec::new(),
    }
}

fn response_error(status: StatusCode, detail: &str) -> Response<Body> {
    (status, Json(json!({ "detail": detail }))).into_response()
}

async fn read_publication(state: &AppState) -> Result<HealthPublication, GatewayError> {
    let config = match state
        .read_system_config_json_value_strong(PUBLICATION_KEY)
        .await?
    {
        Some(value) => serde_json::from_value(value).map_err(|error| {
            GatewayError::Internal(format!("Invalid health publication configuration: {error}"))
        })?,
        None => HealthPublication::default(),
    };
    config
        .validate()
        .map_err(|error| GatewayError::Internal(error.to_string()))?;
    Ok(config)
}

pub(crate) async fn build_publication_response(
    state: &AppState,
    body: Option<&[u8]>,
) -> Response<Body> {
    if let Some(body) = body {
        let config: HealthPublication = match serde_json::from_slice(body) {
            Ok(config) => config,
            Err(_) => {
                return response_error(
                    StatusCode::BAD_REQUEST,
                    "Invalid health publication configuration",
                )
            }
        };
        if let Err(error) = config.validate() {
            return response_error(StatusCode::BAD_REQUEST, error);
        }
        let value = match serde_json::to_value(&config) {
            Ok(value) => value,
            Err(_) => {
                return response_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not encode health publication",
                )
            }
        };
        return match state
            .upsert_system_config_json_value(
                PUBLICATION_KEY,
                &value,
                Some("Public status object allowlist"),
            )
            .await
        {
            Ok(_) => Json(config).into_response(),
            Err(_) => response_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Health publication is unavailable",
            ),
        };
    }
    match read_publication(state).await {
        Ok(config) => Json(config).into_response(),
        Err(_) => response_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Health publication is unavailable",
        ),
    }
}

pub(crate) async fn build_health_v2_response(
    state: &AppState,
    path: &str,
    query: Option<&str>,
    audience: HealthAudience,
) -> Response<Body> {
    let now_ms = Utc::now().timestamp_millis().max(0) as u64;
    let request = match HealthRequest::parse(query, audience != HealthAudience::Admin, now_ms) {
        Ok(request) => request,
        Err(detail) => return response_error(StatusCode::BAD_REQUEST, detail),
    };
    let prefix = match audience {
        HealthAudience::Public => "/api/public/health/v2/",
        HealthAudience::Authenticated => "/api/users/me/health/v2/",
        HealthAudience::Admin => "/api/admin/endpoints/health/v2/",
    };
    let tail = path.strip_prefix(prefix).unwrap_or_default();
    if tail != "summary"
        && tail != "objects"
        && !tail
            .strip_prefix("objects/")
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    {
        return response_error(StatusCode::NOT_FOUND, "Health resource not found");
    }
    match build_health_payload(state, &request, tail, audience, now_ms).await {
        Ok(Some(payload)) => Json(payload).into_response(),
        Ok(None) => response_error(
            StatusCode::NOT_FOUND,
            "Health object is not published or does not exist",
        ),
        Err(error) => {
            tracing::warn!(error = %crate::error::redact_error_detail(&format!("{error:?}")), "health observation query failed");
            response_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Health observations are temporarily unavailable",
            )
        }
    }
}

async fn admin_objects(
    state: &AppState,
    kind: HealthObservationObjectKind,
) -> Result<BTreeMap<String, String>, GatewayError> {
    let mut objects = BTreeMap::new();
    match kind {
        HealthObservationObjectKind::ApiFormat => {
            let providers = state.list_provider_catalog_providers(true).await?;
            let ids: Vec<_> = providers
                .iter()
                .map(|provider| provider.id.clone())
                .collect();
            for endpoint in state
                .list_provider_catalog_endpoints_by_provider_ids(&ids)
                .await?
            {
                if endpoint.is_active {
                    objects.insert(endpoint.api_format.clone(), endpoint.api_format);
                }
            }
        }
        HealthObservationObjectKind::Provider => {
            for provider in state.list_provider_catalog_providers(false).await? {
                objects.insert(provider.id, provider.name);
            }
        }
        HealthObservationObjectKind::Model => {
            let mut offset = 0;
            loop {
                let page = state
                    .list_admin_global_models(&AdminGlobalModelListQuery {
                        offset,
                        limit: 500,
                        ..Default::default()
                    })
                    .await?;
                for model in &page.items {
                    objects.insert(model.name.clone(), model.display_name.clone());
                }
                offset += page.items.len();
                if offset >= page.total || page.items.is_empty() {
                    break;
                }
                if offset >= 10_000 {
                    return Err(GatewayError::Internal(
                        "Health model catalog exceeds query budget".into(),
                    ));
                }
            }
        }
    }
    Ok(objects)
}

async fn build_health_payload(
    state: &AppState,
    request: &HealthRequest,
    tail: &str,
    audience: HealthAudience,
    now_ms: u64,
) -> Result<Option<serde_json::Value>, GatewayError> {
    let policy = HealthPolicy::default();
    let public = audience == HealthAudience::Public;
    let redact_internal = audience != HealthAudience::Admin;
    let publication = if public {
        Some(read_publication(state).await?)
    } else {
        None
    };
    if publication.as_ref().is_some_and(|config| !config.enabled) {
        return Ok(None);
    }
    let published: BTreeMap<_, _> = publication
        .as_ref()
        .map(|config| {
            config
                .objects
                .iter()
                .filter(|object| object.kind == request.kind)
                .map(|object| (object.value.clone(), object))
                .collect()
        })
        .unwrap_or_default();
    let mut names = if public {
        published
            .iter()
            .map(|(value, object)| (value.clone(), object.display_name.clone()))
            .collect()
    } else {
        admin_objects(state, request.kind).await?
    };
    let observation = if public && published.is_empty() {
        Default::default()
    } else {
        state
            .data
            .summarize_health_observations(&HealthObservationQuery {
                from_unix_ms: request.from_unix_ms,
                to_unix_ms: request.to_unix_ms,
                object_kind: request.kind,
                object_values: public.then(|| published.keys().cloned().collect()),
                segments: 24,
            })
            .await
            .map_err(|error| GatewayError::Internal(error.to_string()))?
    };
    let mut metrics_by_value = BTreeMap::new();
    let mut timeline_by_value = BTreeMap::new();
    for object in observation.objects {
        if !public || published.contains_key(&object.object_value) {
            names
                .entry(object.object_value.clone())
                .or_insert_with(|| object.object_value.clone());
            timeline_by_value.insert(object.object_value.clone(), object.timeline);
            metrics_by_value.insert(object.object_value, object.metrics);
        }
    }
    let mut objects = Vec::new();
    for (value, name) in names {
        let metrics = metrics_by_value.remove(&value).unwrap_or_default();
        let id = if public {
            published[&value].public_id.clone()
        } else {
            URL_SAFE_NO_PAD.encode(value.as_bytes())
        };
        let mut service = public_projection(id, request.kind, name, &metrics, &policy);
        service.timeline = timeline_by_value
            .remove(&value)
            .unwrap_or_default()
            .into_iter()
            .map(|bucket| PublicHealthBucket {
                from: unix_ms_to_rfc3339(bucket.from_unix_ms).unwrap_or_default(),
                to: unix_ms_to_rfc3339(bucket.to_unix_ms).unwrap_or_default(),
                status: policy.status(
                    bucket.metrics.service_succeeded_count,
                    bucket.metrics.service_failed_count,
                    bucket.metrics.unknown_failure_count,
                ),
                service_availability: HealthRatio::new(
                    bucket.metrics.service_succeeded_count,
                    bucket
                        .metrics
                        .service_succeeded_count
                        .saturating_add(bucket.metrics.service_failed_count),
                ),
                unknown_failure_count: bucket.metrics.unknown_failure_count,
            })
            .collect();
        objects.push(AdminHealthObject {
            service,
            source_value: value,
            attempts: AdminAttemptMetrics {
                succeeded_count: metrics.attempt_succeeded_count,
                failed_count: metrics.attempt_failed_count,
                in_progress_count: metrics.attempt_in_progress_count,
                cancelled_count: metrics.attempt_cancelled_count,
                success: HealthRatio::new(
                    metrics.attempt_succeeded_count,
                    metrics
                        .attempt_succeeded_count
                        .saturating_add(metrics.attempt_failed_count),
                ),
            },
        });
    }
    let status = overall_status(objects.iter().map(|object| object.service.status));
    let meta = json!({
        "schema_version": 2, "metric_version": policy.version,
        "scope": { "kind": match audience { HealthAudience::Public => "published", HealthAudience::Authenticated => "authenticated", HealthAudience::Admin => "installation" }, "object_kind": request.kind },
        "range": { "from": unix_ms_to_rfc3339(request.from_unix_ms), "to": unix_ms_to_rfc3339(request.to_unix_ms), "timezone": "UTC", "time_basis": "request_started_at" },
        "generated_at": unix_ms_to_rfc3339(now_ms),
        "data_through": observation.data_through_unix_ms.and_then(unix_ms_to_rfc3339),
        "freshness": if observation.data_through_unix_ms.is_some_and(|time| now_ms.saturating_sub(time) > 120_000) { "stale" } else if observation.data_through_unix_ms.is_some() { "current" } else { "unknown" },
        "policy": policy,
    });
    let data = if tail == "summary" {
        json!({ "status": status, "object_count": objects.len(),
            "healthy_count": objects.iter().filter(|object| object.service.status == HealthStatus::Healthy).count(),
            "degraded_count": objects.iter().filter(|object| object.service.status == HealthStatus::Degraded).count(),
            "unavailable_count": objects.iter().filter(|object| object.service.status == HealthStatus::Unavailable).count(),
            "unknown_count": objects.iter().filter(|object| object.service.status == HealthStatus::Unknown).count(),
            "requests": public_projection(String::new(), request.kind, String::new(), &observation.overall, &policy),
        })
    } else if let Some(id) = tail.strip_prefix("objects/") {
        let Some(object) = objects.iter().find(|object| object.service.id == id) else {
            return Ok(None);
        };
        if redact_internal {
            serde_json::to_value(&object.service)
        } else {
            serde_json::to_value(object)
        }
        .map_err(|error| GatewayError::Internal(error.to_string()))?
    } else {
        let items: Vec<serde_json::Value> = objects
            .iter()
            .skip(request.offset)
            .take(request.limit)
            .map(|object| {
                if redact_internal {
                    serde_json::to_value(&object.service)
                } else {
                    serde_json::to_value(object)
                }
            })
            .collect::<Result<_, _>>()
            .map_err(|error| GatewayError::Internal(error.to_string()))?;
        json!({ "items": items, "total": objects.len(), "limit": request.limit, "offset": request.offset })
    };
    Ok(Some(json!({ "meta": meta, "data": data })))
}
