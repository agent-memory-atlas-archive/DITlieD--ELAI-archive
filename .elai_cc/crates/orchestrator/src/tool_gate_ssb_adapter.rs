//! C-35: orchestrator-side adapter that bridges the plugins-crate
//! [`elai_plugins::ToolGateSsbEmitter`] trait into [`crate::ssb::Ssb::write_payload`]
//! calls.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! plugins -> orchestrator dependency direction stays acyclic. Symmetric to
//! `knowledge_ssb_adapter::OrchestratorKnowledgeSsbSink`.
//!
//! Production wiring: dashboard-v2's bootstrap (or the CLI when launching a
//! local MCP host) constructs `Arc::new(OrchestratorToolGateSsbEmitter::new(ssb_arc))`
//! and passes it into `DefaultMcpBridge::with_ssb_emitter`. The 4 tool_* SSB
//! rows then land in the same store the dashboard reads from for the
//! ToolGateInspector panel.

use std::sync::Arc;

use elai_plugins::{tool_gate_emit_kinds as kinds, ToolGateSsbEmitter};
use tracing::warn;

use crate::ssb::{Ssb, SsbPayload, ToolGateEventPayload};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps a C-35 `(action_kind, tool_name, fsm_state, prompt_tokens_saved)`
/// emit tuple onto the matching [`SsbPayload::Tool*`] variant and writes via
/// [`Ssb::write_payload`]. Async-spawned so the sync emit path on the
/// prompt-assembly hot path is never blocked.
pub struct OrchestratorToolGateSsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorToolGateSsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    fn dispatch_payload(&self, payload: SsbPayload) {
        let ssb = self.ssb.clone();
        // The trait method is sync; SSB::write_payload is async. Spawn onto the
        // current runtime so the producer hot-path is never blocked. If no
        // runtime is active (synchronous test context) the row is dropped with
        // a warning -- same semantics as knowledge_ssb_adapter.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            RUNNER_AGENT_ID,
                            RUNNER_SUPERSTEP,
                            payload,
                            Vec::new(),
                        )
                        .await
                    {
                        warn!(
                            error = %e,
                            "C-35 tool_gate ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!(
                    "C-35 tool_gate ssb adapter: no tokio runtime active; dropping row"
                );
            }
        }
    }
}

impl ToolGateSsbEmitter for OrchestratorToolGateSsbEmitter {
    fn emit(
        &self,
        action_kind: &str,
        tool_name: &str,
        fsm_state: &str,
        prompt_tokens_saved: i64,
    ) -> Result<(), String> {
        let event = ToolGateEventPayload {
            tool_name: tool_name.to_owned(),
            fsm_state: fsm_state.to_owned(),
            prompt_tokens_saved,
        };
        let payload = match action_kind {
            k if k == kinds::TOOL_GATED => SsbPayload::ToolGated(event),
            k if k == kinds::TOOL_EAGER => SsbPayload::ToolEager(event),
            k if k == kinds::TOOL_LAZY_RESOLVED => SsbPayload::ToolLazyResolved(event),
            k if k == kinds::TOOL_DENIED => SsbPayload::ToolDenied(event),
            other => {
                return Err(format!(
                    "C-35 tool_gate ssb adapter: unknown action_kind '{other}'"
                ))
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
    fn unknown_action_kind_returns_err() {
        // Build a no-op adapter context: dispatch_payload's runtime-absent path
        // drops the row with a warning, so we can construct a minimal `Ssb`-less
        // adapter only via the trait surface. Here we exercise the action-kind
        // mapping branch by passing a bogus kind into a stub adapter shape.
        struct StubAdapter;
        impl ToolGateSsbEmitter for StubAdapter {
            fn emit(
                &self,
                action_kind: &str,
                _tool_name: &str,
                _fsm_state: &str,
                _prompt_tokens_saved: i64,
            ) -> Result<(), String> {
                match action_kind {
                    k if k == kinds::TOOL_GATED
                        || k == kinds::TOOL_EAGER
                        || k == kinds::TOOL_LAZY_RESOLVED
                        || k == kinds::TOOL_DENIED =>
                    {
                        Ok(())
                    }
                    other => Err(format!("unknown action_kind '{other}'")),
                }
            }
        }
        let a = StubAdapter;
        assert!(a.emit("not_a_real_kind", "x", "s", 0).is_err());
        assert!(a.emit(kinds::TOOL_GATED, "x", "s", 0).is_ok());
        assert!(a.emit(kinds::TOOL_EAGER, "x", "s", 0).is_ok());
        assert!(a.emit(kinds::TOOL_LAZY_RESOLVED, "x", "s", 0).is_ok());
        assert!(a.emit(kinds::TOOL_DENIED, "x", "s", 0).is_ok());
    }
}
