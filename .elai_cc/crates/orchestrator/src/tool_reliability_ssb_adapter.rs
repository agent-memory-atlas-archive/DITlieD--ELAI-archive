//! E-18: orchestrator-side adapter that bridges the plugins-crate
//! [`elai_plugins::ReliabilityGateSsbEmitter`] trait into
//! [`crate::ssb::Ssb::write_payload`] calls.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! `plugins -> orchestrator` dependency direction stays acyclic.
//! Symmetric to `tool_gate_ssb_adapter` (C-35) /
//! `tool_calibration_ssb_adapter` (C-47).
//!
//! Production wiring: dashboard-v2's bootstrap (or the CLI when
//! launching a local MCP host) constructs
//! `Arc::new(OrchestratorReliabilitySsbEmitter::new(ssb_arc))` and
//! passes it into
//! `DefaultMcpBridge::with_reliability_ssb_emitter`. The four
//! `tool_reliability_gate_passed` / `tool_reliability_gate_rejected` /
//! `tool_call_spilled` / `tool_call_retry_escalated` SSB rows then land
//! in the same store the dashboard reads from for the
//! `ToolReliabilityInspector` section of `SettingsWindow`.
//!
//! Identifying strings (tool_name, reject_reason, spill_path,
//! escalate_to_role) are blake3-prefix hashed BEFORE landing in the
//! SSB payload — parity with C-45 / C-46 / C-47 hashing discipline.

use std::sync::Arc;

use elai_plugins::{
    reliability_emit_kinds, ReliabilityGateSsbEmitter,
};
use tracing::warn;

use crate::ssb::{
    Ssb, SsbPayload, ToolCallRetryEscalatedPayload, ToolCallSpilledPayload,
    ToolReliabilityGatePassedPayload, ToolReliabilityGateRejectedPayload,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps an E-18 reliability emit tuple onto the matching
/// SSB payload variant and writes via [`Ssb::write_payload`]. The
/// emitter is sync (the bridge is on a hot path); the underlying
/// SSB write is async and is spawned onto the current runtime.
pub struct OrchestratorReliabilitySsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorReliabilitySsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    fn dispatch_payload(&self, payload: SsbPayload) {
        let ssb = self.ssb.clone();
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                        .await
                    {
                        warn!(
                            error = %e,
                            "E-18 reliability ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!(
                    "E-18 reliability ssb adapter: no tokio runtime active; dropping row"
                );
            }
        }
    }
}

/// blake3-prefix u64 over an arbitrary str. Same scheme as the C-47 +
/// C-45 / C-46 adapters (first 8 bytes of blake3 LE). Anti-stale:
/// NEVER blake2 / sha2.
fn hash_label(s: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(s.as_bytes());
    let bytes = hasher.finalize();
    let prefix: [u8; 8] = bytes.as_bytes()[..8]
        .try_into()
        .unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix)
}

/// Read the current wall-clock as UNIX ms (i64). On the impossible
/// pre-1970 branch we fall back to 0 rather than `.unwrap()` per
/// CLAUDE.md anti-unwrap rule.
fn now_ms_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_millis()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

impl ReliabilityGateSsbEmitter for OrchestratorReliabilitySsbEmitter {
    fn emit(
        &self,
        action_kind: &str,
        tool_name: &str,
        call_shape_hash: u64,
        extra: serde_json::Value,
    ) -> Result<(), String> {
        let ts_ms = now_ms_i64();
        let tool_name_hash = hash_label(tool_name);

        let payload = match action_kind {
            k if k == reliability_emit_kinds::TOOL_RELIABILITY_GATE_PASSED => {
                SsbPayload::ToolReliabilityGatePassed(ToolReliabilityGatePassedPayload {
                    tool_name_hash,
                    call_shape_hash,
                    ts_ms,
                })
            }
            k if k == reliability_emit_kinds::TOOL_RELIABILITY_GATE_REJECTED => {
                let reason_str = extra
                    .get("reject_reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unspecified");
                SsbPayload::ToolReliabilityGateRejected(ToolReliabilityGateRejectedPayload {
                    tool_name_hash,
                    reject_reason_hash: hash_label(reason_str),
                    ts_ms,
                })
            }
            k if k == reliability_emit_kinds::TOOL_CALL_SPILLED => {
                let spill_path = extra
                    .get("spill_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let payload_bytes = extra
                    .get("payload_bytes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                SsbPayload::ToolCallSpilled(ToolCallSpilledPayload {
                    tool_name_hash,
                    spill_path_hash: hash_label(spill_path),
                    payload_bytes,
                    ts_ms,
                })
            }
            k if k == reliability_emit_kinds::TOOL_CALL_RETRY_ESCALATED => {
                let attempt = extra
                    .get("attempt")
                    .and_then(|v| v.as_u64())
                    .and_then(|v| u32::try_from(v).ok())
                    .unwrap_or(0);
                let role = extra
                    .get("escalate_to_role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auditor");
                SsbPayload::ToolCallRetryEscalated(ToolCallRetryEscalatedPayload {
                    tool_name_hash,
                    attempt,
                    escalate_to_role_hash: hash_label(role),
                    ts_ms,
                })
            }
            other => {
                return Err(format!("unknown reliability action_kind: {other}"));
            }
        };
        self.dispatch_payload(payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_label_is_deterministic() {
        assert_eq!(hash_label("fs.read"), hash_label("fs.read"));
        assert_ne!(hash_label("fs.read"), hash_label("git.commit"));
    }

    #[test]
    fn now_ms_i64_is_positive() {
        let t = now_ms_i64();
        assert!(t > 0, "expected wall-clock > 0, got {t}");
    }

    #[test]
    fn reject_reason_string_hashes_to_distinct_u64s() {
        // The 4 wire labels from reliability_reject_reasons should all map
        // to distinct blake3 prefixes (collision risk is 2^-64 per pair).
        let a = hash_label("malformed_args");
        let b = hash_label("package_ref_rejected");
        let c = hash_label("retry_budget_exhausted");
        let d = hash_label("path_allowlist_violation");
        let set = [a, b, c, d];
        for i in 0..set.len() {
            for j in (i + 1)..set.len() {
                assert_ne!(set[i], set[j], "reject reason hashes collided at i={i} j={j}");
            }
        }
    }
}
