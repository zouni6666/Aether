use std::collections::BTreeMap;

use aether_contracts::ProxySnapshot;
use aether_scheduler_core::{
    SchedulerMinimalCandidateSelectionCandidate, SchedulerTunnelAffinityBucket,
};
use serde_json::Value;
use tracing::warn;

use crate::ai_serving::{GatewayProviderTransportSnapshot, PlannerAppState};
use crate::scheduler::config::SchedulerOrderingConfig;

use super::candidate_resolution::read_candidate_transport_snapshot;

const TUNNEL_OWNER_INSTANCE_ID_EXTRA_KEY: &str = "tunnel_owner_instance_id";

pub(super) type CandidateTransportIdentity = (String, String, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidateTransportRankingFacts {
    pub(super) tunnel_bucket: SchedulerTunnelAffinityBucket,
    pub(super) keep_priority_on_conversion: bool,
}

#[derive(Debug, Default)]
pub(super) struct CandidateTransportRankingFactsCache {
    candidate_facts: BTreeMap<CandidateTransportIdentity, CandidateTransportRankingFacts>,
    configured_proxy_snapshots: BTreeMap<String, Option<ProxySnapshot>>,
    system_proxy_snapshot: Option<Option<ProxySnapshot>>,
    tunnel_buckets_by_node_id: BTreeMap<String, SchedulerTunnelAffinityBucket>,
}

pub(super) async fn resolve_cached_candidate_transport_ranking_facts(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
    ordering_config: SchedulerOrderingConfig,
) -> CandidateTransportRankingFacts {
    let identity = candidate_transport_identity(candidate);
    if let Some(facts) = cache.candidate_facts.get(&identity).copied() {
        return facts;
    }

    let facts =
        resolve_candidate_transport_ranking_facts(state, cache, candidate, ordering_config).await;
    cache.candidate_facts.insert(identity, facts);
    facts
}

pub(super) async fn resolve_cached_transport_ranking_facts(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
    transport: &GatewayProviderTransportSnapshot,
    ordering_config: SchedulerOrderingConfig,
) -> CandidateTransportRankingFacts {
    let identity = candidate_transport_identity(candidate);
    if let Some(facts) = cache.candidate_facts.get(&identity).copied() {
        return facts;
    }

    let facts = resolve_candidate_transport_ranking_facts_from_transport(
        state,
        cache,
        transport,
        ordering_config,
    )
    .await;
    cache.candidate_facts.insert(identity, facts);
    facts
}

pub(super) async fn candidate_keeps_priority_on_conversion(
    state: PlannerAppState<'_>,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
    ordering_config: SchedulerOrderingConfig,
) -> bool {
    let mut cache = CandidateTransportRankingFactsCache::default();
    resolve_candidate_transport_ranking_facts(state, &mut cache, candidate, ordering_config)
        .await
        .keep_priority_on_conversion
}

async fn resolve_candidate_transport_ranking_facts(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
    ordering_config: SchedulerOrderingConfig,
) -> CandidateTransportRankingFacts {
    let Some(transport) = read_candidate_transport_snapshot(state, candidate).await else {
        return CandidateTransportRankingFacts {
            tunnel_bucket: SchedulerTunnelAffinityBucket::Neutral,
            keep_priority_on_conversion: ordering_config.keep_priority_on_conversion,
        };
    };

    resolve_candidate_transport_ranking_facts_from_transport(
        state,
        cache,
        &transport,
        ordering_config,
    )
    .await
}

async fn resolve_candidate_transport_ranking_facts_from_transport(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    transport: &GatewayProviderTransportSnapshot,
    ordering_config: SchedulerOrderingConfig,
) -> CandidateTransportRankingFacts {
    CandidateTransportRankingFacts {
        tunnel_bucket: resolve_tunnel_owner_affinity_from_transport(state, cache, transport).await,
        keep_priority_on_conversion: ordering_config.keep_priority_on_conversion
            || transport.provider.keep_priority_on_conversion,
    }
}

async fn resolve_tunnel_owner_affinity_from_transport(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    transport: &GatewayProviderTransportSnapshot,
) -> SchedulerTunnelAffinityBucket {
    let Some(proxy) =
        resolve_transport_proxy_snapshot_with_tunnel_affinity_cached(state, cache, transport).await
    else {
        return SchedulerTunnelAffinityBucket::Neutral;
    };
    if proxy.enabled == Some(false) {
        return SchedulerTunnelAffinityBucket::Neutral;
    }
    let Some(node_id) = proxy
        .node_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return SchedulerTunnelAffinityBucket::Neutral;
    };

    if let Some(bucket) = cache.tunnel_buckets_by_node_id.get(node_id).copied() {
        return bucket;
    }

    let bucket = resolve_tunnel_owner_affinity_from_proxy(state, &proxy, node_id).await;
    cache
        .tunnel_buckets_by_node_id
        .insert(node_id.to_string(), bucket);
    bucket
}

async fn resolve_transport_proxy_snapshot_with_tunnel_affinity_cached(
    state: PlannerAppState<'_>,
    cache: &mut CandidateTransportRankingFactsCache,
    transport: &GatewayProviderTransportSnapshot,
) -> Option<ProxySnapshot> {
    for raw in [
        transport.key.proxy.as_ref(),
        transport.endpoint.proxy.as_ref(),
        transport.provider.proxy.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        let cache_key = proxy_config_cache_key(raw);
        if let Some(snapshot) = cache.configured_proxy_snapshots.get(&cache_key) {
            if snapshot.is_some() {
                return snapshot.clone();
            }
            continue;
        }
        let snapshot = state
            .app()
            .resolve_configured_proxy_snapshot_with_tunnel_affinity(Some(raw))
            .await;
        cache
            .configured_proxy_snapshots
            .insert(cache_key, snapshot.clone());
        if snapshot.is_some() {
            return snapshot;
        }
    }

    if let Some(snapshot) = cache.system_proxy_snapshot.as_ref() {
        return snapshot.clone();
    }
    let snapshot = state.app().resolve_system_proxy_snapshot().await;
    cache.system_proxy_snapshot = Some(snapshot.clone());
    snapshot
}

async fn resolve_tunnel_owner_affinity_from_proxy(
    state: PlannerAppState<'_>,
    proxy: &ProxySnapshot,
    node_id: &str,
) -> SchedulerTunnelAffinityBucket {
    if state.app().tunnel.has_local_proxy(node_id) {
        return SchedulerTunnelAffinityBucket::LocalTunnel;
    }

    if let Some(owner_instance_id) = proxy_tunnel_owner_instance_id(proxy) {
        return if owner_instance_id == state.app().tunnel.local_instance_id() {
            SchedulerTunnelAffinityBucket::LocalTunnel
        } else {
            SchedulerTunnelAffinityBucket::RemoteTunnel
        };
    }

    match state
        .app()
        .tunnel
        .lookup_attachment_owner(state.app().data.as_ref(), node_id)
        .await
    {
        Ok(Some(owner)) if owner.gateway_instance_id == state.app().tunnel.local_instance_id() => {
            SchedulerTunnelAffinityBucket::LocalTunnel
        }
        Ok(Some(_)) => SchedulerTunnelAffinityBucket::RemoteTunnel,
        Ok(None) => SchedulerTunnelAffinityBucket::Neutral,
        Err(error) => {
            warn!(
                event_name = "candidate_transport_ranking_facts_tunnel_owner_lookup_failed",
                log_type = "event",
                node_id = node_id,
                error = %error,
                "failed to load tunnel attachment owner while evaluating candidate transport ranking facts"
            );
            SchedulerTunnelAffinityBucket::Neutral
        }
    }
}

fn candidate_transport_identity(
    candidate: &SchedulerMinimalCandidateSelectionCandidate,
) -> CandidateTransportIdentity {
    (
        candidate.provider_id.clone(),
        candidate.endpoint_id.clone(),
        candidate.key_id.clone(),
    )
}

fn proxy_config_cache_key(raw: &Value) -> String {
    serde_json::to_string(raw).unwrap_or_else(|_| raw.to_string())
}

fn proxy_tunnel_owner_instance_id(proxy: &ProxySnapshot) -> Option<&str> {
    proxy
        .extra
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|extra| extra.get(TUNNEL_OWNER_INSTANCE_ID_EXTRA_KEY))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
