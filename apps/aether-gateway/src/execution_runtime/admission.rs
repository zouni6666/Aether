//! Shared admission helpers for local upstream execution.
//!
//! The stream candidate loop and long-lived WebSocket turns both need to
//! participate in the same gateway-wide upstream execution gate.  Keep the
//! provider abstraction here so tests can supply an isolated gate while
//! production callers use `AppState` directly.

use std::time::Duration;

use aether_runtime::{ConcurrencyGate, ConcurrencyPermit};
use tokio::time::timeout;

use crate::stage_metrics::observe_gateway_stage_ms;
use crate::{AppState, GatewayError};

pub(crate) const UPSTREAM_EXECUTION_GATE_NAME: &str = "gateway_upstream_execution";

pub(crate) trait UpstreamExecutionGateProvider {
    fn upstream_execution_gate(&self) -> Option<&ConcurrencyGate>;
    fn upstream_execution_gate_queue_budget(&self) -> Duration;
}

impl UpstreamExecutionGateProvider for AppState {
    fn upstream_execution_gate(&self) -> Option<&ConcurrencyGate> {
        self.upstream_execution_gate.as_deref()
    }

    fn upstream_execution_gate_queue_budget(&self) -> Duration {
        self.frontdoor_runtime_guards.internal_gate_queue_budget
    }
}

/// Acquires the shared gateway-wide upstream execution permit.
///
/// A missing gate is an intentional configuration (unlimited), so callers
/// receive `Ok(None)`.  Saturation keeps the existing candidate-level
/// `AdmissionTimeout` contract used by the HTTP stream path.
pub(crate) async fn acquire_upstream_execution_gate(
    state: &(impl UpstreamExecutionGateProvider + ?Sized),
    trace_id: &str,
) -> Result<Option<ConcurrencyPermit>, GatewayError> {
    let Some(gate) = state.upstream_execution_gate() else {
        return Ok(None);
    };
    let budget = state.upstream_execution_gate_queue_budget();
    let gate_wait_started_at = std::time::Instant::now();
    match timeout(budget, gate.acquire()).await {
        Ok(Ok(permit)) => {
            observe_gateway_stage_ms(
                "upstream_execution_gate_wait",
                gate_wait_started_at.elapsed().as_millis() as u64,
            );
            Ok(Some(permit))
        }
        Ok(Err(err)) => Err(GatewayError::Internal(err.to_string())),
        Err(_) => Err(GatewayError::AdmissionTimeout {
            trace_id: trace_id.to_string(),
            gate: UPSTREAM_EXECUTION_GATE_NAME,
            queue_budget_ms: budget.as_millis() as u64,
        }),
    }
}
