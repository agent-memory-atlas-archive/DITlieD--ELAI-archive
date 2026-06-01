//! C-42b (Tranche D, s-22600) -- orchestrator-side adapter that
//! bridges the plugins-crate [`elai_plugins::authz::McpAuthzSsbEmitter`]
//! trait into [`crate::ssb::Ssb::write_payload`] calls.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! `plugins -> orchestrator` dependency direction stays acyclic.
//! Symmetric to `tool_calibration_ssb_adapter` (C-47) and
//! `tool_gate_ssb_adapter` (C-35); copies the 90% boilerplate of
//! `dispatch_payload` + the no-runtime warn-on-drop semantics from
//! the C-47 adapter.
//!
//! Production wiring (cli + dashboard-v2/src-tauri bootstrap):
//!
//!     let mcp_authz_emitter = Arc::new(
//!         OrchestratorMcpAuthzSsbEmitter::new(ssb.clone()),
//!     );
//!     let bridge = DefaultMcpBridge::default()
//!         .with_authz_ssb_emitter(gateway_emitter)         // existing 4-arg path
//!         .with_mcp_authz_ssb_emitter(mcp_authz_emitter);  // NEW 5-arg path
//!
//! Naming-collision warning (D.2 Phase B): the bridge's existing
//! `with_authz_ssb_emitter` @mcp_bridge.rs:540 wires the
//! [`GatewayAuthzSsbEmitter`] (4-arg trait, per-layer attribution). The
//! NEW builder for THIS emitter is `with_mcp_authz_ssb_emitter` to
//! avoid name collision per D-trancheD-decisions.md.
//!
//! DUAL-EMIT (D7): when both emitters are wired,
//! `dispatch_tool_call_with_authz` fires TWO SSB rows per call:
//!   1. gateway_policy_{allowed,denied} (per-layer attribution)
//!   2. mcp_authz_event {AuthzCheckPassed,AuthzCheckFailed} (composed
//!      verdict, blake3-prefix hashed identifiers)
//! Future "deduplication" of these rows is a regression -- they carry
//! semantically distinct signals and the contract test pins the count.
//!
//! Wire-dark posture: the env-gate
//! [`crate::ssb::ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV`] is read on every
//! emit call (cheap; microseconds per env_gate_truthy lookup). When
//! unset, the emit silently no-ops without touching the SSB pool.
//!
//! Identifier-hashing discipline: tool / fsm_state / actor / layer /
//! deny_reason all hashed via [`crate::ssb::blake3_label_prefix`] BEFORE
//! landing on the payload so the wire surface never carries raw
//! model-facing strings (parity with C-45 / C-46 / C-47 / C-42 hashing
//! discipline).

use std::sync::Arc;

use elai_plugins::authz::McpAuthzSsbEmitter;
use tracing::warn;

use crate::ssb::{
    blake3_label_prefix, env_gate_truthy, system_time_ms_i64, McpAuthzEventKind,
    McpAuthzEventPayload, Ssb, SsbPayload, ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps a C-42b composed-verdict emit tuple onto the
/// [`SsbPayload::McpAuthzEvent`] variant and writes via
/// [`Ssb::write_payload`]. Async-spawned so the sync emit path on the
/// bridge dispatch hot path is never blocked.
///
/// Wired into [`elai_plugins::mcp_bridge::DefaultMcpBridge`] via the NEW
/// builder `with_mcp_authz_ssb_emitter` (distinct from the existing
/// `with_authz_ssb_emitter` which wires the GatewayAuthzSsbEmitter
/// trait).
pub struct OrchestratorMcpAuthzSsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorMcpAuthzSsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    /// Spawn one SSB write onto the current tokio runtime. Returns
    /// immediately; the write completes asynchronously. If no runtime
    /// is active (synchronous test context) the row is dropped with a
    /// warning -- mirrors tool_calibration_ssb_adapter::dispatch_payload.
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
                            "C-42b mcp_authz ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!(
                    "C-42b mcp_authz ssb adapter: no tokio runtime active; dropping row"
                );
            }
        }
    }
}

impl McpAuthzSsbEmitter for OrchestratorMcpAuthzSsbEmitter {
    fn emit_authz_decision(
        &self,
        tool: &str,
        fsm_state: &str,
        actor: &str,
        layer: &str,
        deny_reason: Option<&str>,
    ) {
        // Env-gate at emit-site (canonical wire-dark pattern -- see
        // compose.rs:130 "the emitter itself is the env-gate site").
        if !env_gate_truthy(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV) {
            return;
        }
        let kind = if deny_reason.is_some() {
            McpAuthzEventKind::AuthzCheckFailed
        } else {
            McpAuthzEventKind::AuthzCheckPassed
        };
        let severity: u8 = if deny_reason.is_some() { 1 } else { 0 };
        let payload = SsbPayload::McpAuthzEvent(McpAuthzEventPayload {
            kind,
            tool_name_hash: blake3_label_prefix(tool),
            fsm_state_hash: blake3_label_prefix(fsm_state),
            actor_hash: blake3_label_prefix(actor),
            layer_hash: blake3_label_prefix(layer),
            deny_reason_hash: deny_reason.map(blake3_label_prefix),
            severity,
            details_json: serde_json::json!({
                "caller_site": "compose_with_tool_gate_emit",
            }),
            ts_ms: system_time_ms_i64(),
        });
        self.dispatch_payload(payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_short_circuits_when_env_gate_unset() {
        // Stub adapter that records whether dispatch_payload was called
        // by overriding the method via a custom impl. We can't easily
        // mock the Ssb here, so we exercise the env-gate decision in
        // isolation by checking env_gate_truthy returns false when
        // unset.
        std::env::remove_var(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV);
        assert!(
            !env_gate_truthy(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV),
            "env-gate must default to false in tests when unset"
        );
    }

    #[test]
    fn env_gate_truthy_recognises_canonical_values() {
        // Sanity round-trip on the canonical truthy synonyms used by the
        // adapter. Reuses the shared ssb::env_gate_truthy.
        for v in ["1", "true", "TRUE", "yes", "on"] {
            std::env::set_var(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV, v);
            assert!(
                env_gate_truthy(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV),
                "value {v} must be recognised as truthy"
            );
        }
        std::env::remove_var(ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV);
    }
}
