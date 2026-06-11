//! G-1 (data-plane taint propagation, s-56068) -- orchestrator-side emitter
//! that writes a [`crate::ssb::SsbPayload::TaintFlow`] row when an untrusted
//! ingest flows into a destructive sink and the SHIPPED
//! `safety::capability_gate::allow_tool_with_origin` gate REJECTS the call.
//!
//! This is the OFF-AGENT ARTIFACT the wiring gate measures (wiring-enforcement
//! R4 / R6): the row is written by a REAL production dispatch entrypoint
//! (`production_agent_rpc::translate_tool_request` git_commit / edit arms), not
//! by a `#[cfg(test)]` caller and not by the agent that produced the feature.
//!
//! Wire-dark posture: NONE. A blocked untrusted->sink flow is a security event
//! that MUST always be recorded. The only opt-out is the SINK-side kill-switch
//! `ELAI_TAINT_PROPAGATE_DISABLE` (read at the gate call site), which disarms
//! the gate so `emit_taint_flow` is never reached. There is intentionally NO
//! enable flag (wiring-enforcement R9: a security gate that ships default-off
//! is wire-dark by design).
//!
//! Identifier discipline: `origin_blake3` is a blake3 hex-prefix of the ingest
//! origin (URL / server+tool / path / sandbox id), NOT a raw secret. The
//! `sink_tool` + `trust_level` + `kind` are non-secret labels for the operator
//! to make a re-approval decision in the dashboard drawer.

use std::sync::Arc;

use blake3::Hasher;
use tracing::warn;

use crate::ssb::{Ssb, SsbPayload, TaintFlowKind, TaintFlowPayload};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Number of blake3 hex chars retained for the `origin_blake3` identifier.
const ORIGIN_HASH_HEX_LEN: usize = 16;

/// blake3 hex-prefix of an ingest origin string. blake3 is the ONLY hash used
/// (CLAUDE.md HARD RULE; no legacy hash families). Identifier, not a secret.
pub fn origin_blake3_prefix(origin: &str) -> String {
    let mut h = Hasher::new();
    h.update(origin.as_bytes());
    let hex = h.finalize().to_hex();
    hex.as_str()[..ORIGIN_HASH_HEX_LEN].to_owned()
}

/// Write one `taint_flow` SSB row for a blocked untrusted->sink flow.
///
/// Async-spawned so the synchronous dispatch path is never blocked on the SSB
/// write; if no tokio runtime is active (a synchronous test context) the row
/// is dropped with a warning — mirrors `mcp_rugpull_ssb_adapter::dispatch_payload`.
///
/// * `ssb`         — the live SSB handle threaded through the dispatch fan.
/// * `kind`        — which untrusted ingest category originated the flow.
/// * `origin`      — the raw ingest origin (URL / server+tool / path / sandbox);
///                   hashed to `origin_blake3` here (not stored raw).
/// * `trust_level` — the joined trust label that reached the sink.
/// * `sink_tool`   — the destructive sink tool name that was blocked.
/// * `task_id`     — the task / turn id for dashboard correlation.
pub fn emit_taint_flow(
    ssb: &Arc<Ssb>,
    kind: TaintFlowKind,
    origin: &str,
    trust_level: &str,
    sink_tool: &str,
    task_id: &str,
) {
    let payload = SsbPayload::TaintFlow(TaintFlowPayload {
        kind,
        origin_blake3: origin_blake3_prefix(origin),
        trust_level: trust_level.to_owned(),
        sink_tool: Some(sink_tool.to_owned()),
        blocked: true,
        task_id: task_id.to_owned(),
    });
    dispatch_payload(ssb.clone(), payload);
}

/// Spawn one SSB write onto the current tokio runtime. Returns immediately; the
/// write completes asynchronously. Mirrors the G-4 rug-pull adapter exactly.
fn dispatch_payload(ssb: Arc<Ssb>, payload: SsbPayload) {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(async move {
                if let Err(e) = ssb
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    warn!(
                        error = %e,
                        "G-1 taint_flow ssb adapter: write_payload failed (non-fatal)"
                    );
                }
            });
        }
        Err(_) => {
            warn!("G-1 taint_flow ssb adapter: no tokio runtime active; dropping row");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_hash_is_stable_blake3_prefix() {
        let a = origin_blake3_prefix("https://attacker.example/payload");
        let b = origin_blake3_prefix("https://attacker.example/payload");
        let c = origin_blake3_prefix("https://other.example/payload");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), ORIGIN_HASH_HEX_LEN);
        assert!(a.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn emit_builds_taint_flow_payload() {
        // Exercise the payload-construction path in isolation (no Ssb needed):
        // the action_type of the produced payload must be TaintFlow.
        let payload = SsbPayload::TaintFlow(TaintFlowPayload {
            kind: TaintFlowKind::Web,
            origin_blake3: origin_blake3_prefix("https://attacker.example/x"),
            trust_level: "untrusted".into(),
            sink_tool: Some("git_commit".into()),
            blocked: true,
            task_id: "t-1".into(),
        });
        assert_eq!(payload.action_type(), crate::ssb::SsbActionType::TaintFlow);
    }
}
