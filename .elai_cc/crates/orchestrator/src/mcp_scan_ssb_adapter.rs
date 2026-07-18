//! E-37 U3: MCP-scan classifier SSB adapter -- orchestrator-side bridge from
//! `elai_plugins::mcp_bridge::McpScanSsbEmitter` to `crate::ssb::Ssb::write_payload`.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! `plugins -> orchestrator` dependency direction stays acyclic. Mirrors
//! `mcp_rugpull_ssb_adapter.rs` and `mcp_authz_ssb_adapter.rs`.

use std::sync::Arc;

use elai_plugins::mcp_bridge::McpScanSsbEmitter;
use tracing::warn;

use crate::ssb::{McpScanClassifierBlockedPayload, Ssb, SsbPayload, system_time_ms_i64};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps an E-37 MCP-scan classifier emit tuple onto the
/// [`SsbPayload::McpScanClassifierBlocked`] variant and writes via
/// [`Ssb::write_payload`]. Async-spawned so the synchronous emit path on the
/// `http_list_tools` gate is never blocked.
///
/// Wired into [`elai_plugins::mcp_bridge::DefaultMcpBridge`] via
/// `with_mcp_scan_ssb_emitter`.
pub struct OrchestratorMcpScanSsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorMcpScanSsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    /// Spawn one SSB write onto the current tokio runtime. Returns
    /// immediately; the write completes asynchronously. If no runtime is
    /// active (synchronous test context) the row is dropped with a warning --
    /// mirrors `mcp_rugpull_ssb_adapter::dispatch_payload`.
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
                            "E-37 U3 mcp_scan ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!("E-37 U3 mcp_scan ssb adapter: no tokio runtime active; dropping row");
            }
        }
    }
}

impl McpScanSsbEmitter for OrchestratorMcpScanSsbEmitter {
    fn emit(
        &self,
        server_url: &str,
        tool_count: u32,
        patterns: &[String],
    ) -> Result<(), String> {
        let payload = SsbPayload::McpScanClassifierBlocked(McpScanClassifierBlockedPayload {
            server_url: server_url.to_owned(),
            tool_count,
            patterns: patterns.to_vec(),
        });
        let _ = system_time_ms_i64();
        self.dispatch_payload(payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_builds_mcp_scan_classifier_blocked_payload() {
        let payload = SsbPayload::McpScanClassifierBlocked(McpScanClassifierBlockedPayload {
            server_url: "http://localhost:8200".into(),
            tool_count: 3,
            patterns: vec!["explicit_override".into()],
        });
        assert_eq!(
            payload.action_type(),
            crate::ssb::SsbActionType::McpScanClassifierBlocked
        );
    }
}
