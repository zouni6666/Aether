use aether_admission_core::{
    AdmissionDecision, AdmissionPolicy, AdmissionRejectReason, AdmissionRequest,
    DefaultAdmissionPolicy, ResourceClass,
};

use crate::DEFAULT_TUNNEL_PROBE_BODY_LIMIT_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelAdmissionClass {
    Connection,
    Relay { streaming: bool },
    Probe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TunnelAdmissionRequest<'a> {
    pub trace_id: &'a str,
    pub class: TunnelAdmissionClass,
    pub body_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TunnelAdmissionPolicy {
    policy: DefaultAdmissionPolicy,
}

impl TunnelAdmissionPolicy {
    pub fn decide(&self, request: TunnelAdmissionRequest<'_>) -> AdmissionDecision {
        let body_limit = body_limit(request.class);
        if body_limit.is_some_and(|limit| request.body_bytes > limit) {
            return AdmissionDecision::Reject(AdmissionRejectReason::BodyTooLarge);
        }

        match self.policy.decide(AdmissionRequest {
            trace_id: request.trace_id,
            class: resource_class(request.class),
            body_bytes: request.body_bytes,
        }) {
            AdmissionDecision::Admit(mut budget) => {
                budget.body_bytes = body_limit.unwrap_or(0);
                AdmissionDecision::Admit(budget)
            }
            rejected => rejected,
        }
    }
}

const fn body_limit(class: TunnelAdmissionClass) -> Option<usize> {
    match class {
        TunnelAdmissionClass::Connection => Some(0),
        TunnelAdmissionClass::Relay { .. } => None,
        TunnelAdmissionClass::Probe => Some(DEFAULT_TUNNEL_PROBE_BODY_LIMIT_BYTES),
    }
}

const fn resource_class(class: TunnelAdmissionClass) -> ResourceClass {
    match class {
        TunnelAdmissionClass::Connection => ResourceClass::Streaming,
        TunnelAdmissionClass::Relay { streaming: true } => ResourceClass::Streaming,
        TunnelAdmissionClass::Relay { streaming: false } | TunnelAdmissionClass::Probe => {
            ResourceClass::Interactive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TunnelAdmissionClass, TunnelAdmissionPolicy, TunnelAdmissionRequest};
    use aether_admission_core::{AdmissionDecision, AdmissionRejectReason, DbClass};

    use crate::DEFAULT_TUNNEL_PROBE_BODY_LIMIT_BYTES;

    #[test]
    fn stream_connection_reserves_stream_and_upstream_permits() {
        let decision = TunnelAdmissionPolicy::default().decide(TunnelAdmissionRequest {
            trace_id: "trace-1",
            class: TunnelAdmissionClass::Connection,
            body_bytes: 0,
        });
        let AdmissionDecision::Admit(budget) = decision else {
            panic!("connection should be admitted");
        };
        assert_eq!(budget.stream_permits, 1);
        assert_eq!(budget.upstream_permits, 1);
        assert_eq!(budget.db_class, DbClass::ForegroundWrite);
    }

    #[test]
    fn relay_budget_has_no_body_limit() {
        let policy = TunnelAdmissionPolicy::default();
        let decision = policy.decide(TunnelAdmissionRequest {
            trace_id: "trace-2",
            class: TunnelAdmissionClass::Relay { streaming: false },
            body_bytes: usize::MAX,
        });
        let AdmissionDecision::Admit(budget) = decision else {
            panic!("relay body should be admitted without a size limit");
        };
        assert_eq!(budget.body_bytes, 0);
    }

    #[test]
    fn probe_budget_still_enforces_body_limit() {
        let policy = TunnelAdmissionPolicy::default();
        assert_eq!(
            policy.decide(TunnelAdmissionRequest {
                trace_id: "trace-3",
                class: TunnelAdmissionClass::Probe,
                body_bytes: DEFAULT_TUNNEL_PROBE_BODY_LIMIT_BYTES + 1,
            }),
            AdmissionDecision::Reject(AdmissionRejectReason::BodyTooLarge)
        );
    }
}
