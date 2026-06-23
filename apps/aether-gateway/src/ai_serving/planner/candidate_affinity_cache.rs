use aether_scheduler_core::{
    build_scheduler_affinity_cache_key_for_api_key_id_with_client_session, ClientSessionAffinity,
    SchedulerAffinityTarget, SchedulerMinimalCandidateSelectionCandidate,
};

use crate::ai_serving::{GatewayAuthApiKeySnapshot, PlannerAppState};
use crate::scheduler::affinity::SCHEDULER_AFFINITY_TTL;

const PLANNER_SCHEDULER_AFFINITY_MAX_ENTRIES: usize = 10_000;

pub(crate) fn has_explicit_session_affinity(
    client_session_affinity: Option<&ClientSessionAffinity>,
) -> bool {
    client_session_affinity.is_some_and(ClientSessionAffinity::has_session_key)
}

pub(crate) fn read_cached_scheduler_affinity_target(
    state: PlannerAppState<'_>,
    auth_snapshot: Option<&GatewayAuthApiKeySnapshot>,
    client_session_affinity: Option<&ClientSessionAffinity>,
    client_api_format: &str,
    requested_model: Option<&str>,
) -> Option<SchedulerAffinityTarget> {
    if !has_explicit_session_affinity(client_session_affinity) {
        return None;
    }
    let requested_model = requested_model
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let api_key_id = auth_snapshot
        .map(|snapshot| snapshot.api_key_id.trim())
        .filter(|value| !value.is_empty())?;
    let cache_key = build_scheduler_affinity_cache_key_for_api_key_id_with_client_session(
        api_key_id,
        client_api_format,
        requested_model,
        client_session_affinity,
    )?;

    state
        .app()
        .read_scheduler_affinity_target(&cache_key, SCHEDULER_AFFINITY_TTL)
}

pub(crate) fn remember_scheduler_affinity_for_candidate(
    state: PlannerAppState<'_>,
    auth_snapshot: Option<&GatewayAuthApiKeySnapshot>,
    client_session_affinity: Option<&ClientSessionAffinity>,
    client_api_format: &str,
    requested_model: &str,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
) {
    if !has_explicit_session_affinity(client_session_affinity) {
        return;
    }
    remember_scheduler_affinity_for_candidate_at_epoch(
        state,
        auth_snapshot,
        client_session_affinity,
        client_api_format,
        requested_model,
        candidate,
        None,
    );
}

pub(crate) fn remember_scheduler_affinity_for_candidate_at_epoch(
    state: PlannerAppState<'_>,
    auth_snapshot: Option<&GatewayAuthApiKeySnapshot>,
    client_session_affinity: Option<&ClientSessionAffinity>,
    client_api_format: &str,
    requested_model: &str,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
    expected_epoch: Option<u64>,
) {
    if !has_explicit_session_affinity(client_session_affinity) {
        return;
    }
    let Some(api_key_id) = auth_snapshot
        .map(|snapshot| snapshot.api_key_id.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(cache_key) = build_scheduler_affinity_cache_key_for_api_key_id_with_client_session(
        api_key_id,
        client_api_format,
        requested_model,
        client_session_affinity,
    ) else {
        return;
    };

    let _ = state.app().remember_scheduler_affinity_target_for_epoch(
        &cache_key,
        SchedulerAffinityTarget {
            provider_id: candidate.provider_id.clone(),
            endpoint_id: candidate.endpoint_id.clone(),
            key_id: candidate.key_id.clone(),
        },
        SCHEDULER_AFFINITY_TTL,
        PLANNER_SCHEDULER_AFFINITY_MAX_ENTRIES,
        expected_epoch,
    );
}
