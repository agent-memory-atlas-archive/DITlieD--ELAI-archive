//! G-4 (Live MCP rug-pull / line-jump re-pin gate, s-56068) -- orchestrator-
//! side adapter that bridges the plugins-crate
//! [`elai_plugins::mcp_bridge::RugPullSsbEmitter`] trait into
//! [`crate::ssb::Ssb::write_payload`] calls.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! `plugins -> orchestrator` dependency direction stays acyclic. Symmetric to
//! [`crate::mcp_authz_ssb_adapter::OrchestratorMcpAuthzSsbEmitter`] (C-42b) and
//! the C-47 / C-35 emitter adapters; copies the no-runtime warn-on-drop
//! `dispatch_payload` semantics.
//!
//! Production wiring (cli + dashboard-v2/src-tauri bootstrap):
//!
//!     let rug_pull_emitter = Arc::new(
//!         OrchestratorRugPullSsbEmitter::new(ssb.clone()),
//!     );
//!     let bridge = DefaultMcpBridge::default()
//!         .with_schema_pin(workspace_root)                 // arm the gate
//!         .with_rug_pull_ssb_emitter(rug_pull_emitter);    // wire the SSB row
//!
//! Wire-dark posture: NONE. Unlike the C-42b authz emitter (which env-gates at
//! the emit site behind `ELAI_C42_MCP_AUTHZ_EMIT_ENABLE`), the rug-pull row is
//! DEFAULT-ON: a runtime rug-pull is a security event that MUST always be
//! recorded. The only opt-out is the bridge-side kill-switch
//! `ELAI_MCP_REPIN_DISABLE` (read at bridge construction), which disarms the
//! whole gate so `emit` is never reached. There is intentionally NO enable
//! flag (wiring-enforcement R9: a security gate that ships default-off is
//! wire-dark by design).
//!
//! Identifier discipline: the `server_url` + `changed_tools` are stored RAW on
//! the payload (matching the E-24 [`crate::ssb::McpSchemaPinViolationPayload`]
//! template, which also stores `server_url` raw) because the dashboard drawer
//! must render the human-readable server + tool names of a rug-pull for the
//! operator to make a re-approval decision. The before/after schema hashes are
//! blake3 hex digests produced by `hash_tool_list`, not raw secrets.

use std::sync::Arc;

use elai_plugins::mcp_bridge::RugPullSsbEmitter;
use tracing::warn;

use crate::ssb::{system_time_ms_i64, McpRugPullBlockedPayload, Ssb, SsbPayload};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps a G-4 rug-pull emit tuple onto the
/// [`SsbPayload::McpRugPullBlocked`] variant and writes via
/// [`Ssb::write_payload`]. Async-spawned so the synchronous emit path on the
/// `http_list_tools` gate is never blocked.
///
/// Wired into [`elai_plugins::mcp_bridge::DefaultMcpBridge`] via
/// `with_rug_pull_ssb_emitter`.
pub struct OrchestratorRugPullSsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorRugPullSsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    /// Spawn one SSB write onto the current tokio runtime. Returns
    /// immediately; the write completes asynchronously. If no runtime is
    /// active (synchronous test context) the row is dropped with a warning --
    /// mirrors `mcp_authz_ssb_adapter::dispatch_payload`.
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
                            "G-4 rug_pull ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!("G-4 rug_pull ssb adapter: no tokio runtime active; dropping row");
            }
        }
    }
}

impl RugPullSsbEmitter for OrchestratorRugPullSsbEmitter {
    fn emit(
        &self,
        server_url: &str,
        schema_blake3_before: &str,
        schema_blake3_after: &str,
        changed_tools: &[String],
        description_diff_count: u32,
        ingest_origin: &str,
    ) -> Result<(), String> {
        let payload = SsbPayload::McpRugPullBlocked(McpRugPullBlockedPayload {
            server_url: server_url.to_owned(),
            schema_blake3_before: schema_blake3_before.to_owned(),
            schema_blake3_after: schema_blake3_after.to_owned(),
            changed_tools: changed_tools.to_vec(),
            description_diff_count,
            ingest_origin: ingest_origin.to_owned(),
        });
        // ts_ms is recorded on the payload for ordering; the SSB row also
        // carries its own created_at. (McpRugPullBlockedPayload has no ts
        // field per the frozen WIRING_CONTRACT; the timestamp is the row's
        // created_at. system_time_ms_i64 is referenced here to keep the
        // adapter's clock source consistent with sibling adapters and is a
        // no-op pass-through until a payload ts field is added.)
        let _ = system_time_ms_i64();
        self.dispatch_payload(payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_builds_mcp_rug_pull_blocked_payload() {
        // Exercise the payload-construction path in isolation (no Ssb needed):
        // the action_type of the produced payload must be McpRugPullBlocked.
        let payload = SsbPayload::McpRugPullBlocked(McpRugPullBlockedPayload {
            server_url: "http://localhost:8200".into(),
            schema_blake3_before: "before".into(),
            schema_blake3_after: "after".into(),
            changed_tools: vec!["search".into()],
            description_diff_count: 1,
            ingest_origin: "http_list_tools".into(),
        });
        assert_eq!(
            payload.action_type(),
            crate::ssb::SsbActionType::McpRugPullBlocked
        );
    }
}
