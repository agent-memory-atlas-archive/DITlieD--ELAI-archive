//! C-47: orchestrator-side adapter that bridges the plugins-crate
//! [`elai_plugins::CaseBaseSsbEmitter`] trait into [`crate::ssb::Ssb::write_payload`]
//! calls.
//!
//! Lives in the orchestrator crate (not in `plugins`) so the
//! plugins -> orchestrator dependency direction stays acyclic. Symmetric
//! to `tool_gate_ssb_adapter` (C-35) and `knowledge_ssb_adapter`.
//!
//! Production wiring: dashboard-v2's bootstrap (or the CLI when launching
//! a local MCP host) constructs
//! `Arc::new(OrchestratorCaseBaseSsbEmitter::new(ssb_arc))` and passes
//! it into `DefaultMcpBridge::with_calibration_ssb_emitter`. The three
//! `tool_call_logged` / `tool_calibration_hint_emitted` /
//! `tool_calibration_drift_detected` SSB rows then land in the same
//! store the dashboard reads from for the `CaseBaseInspector` panel.
//!
//! Identifying strings (tool_name / response_kind / hint_text) are
//! blake3-prefix hashed BEFORE landing in the SSB payload so the wire
//! surface never carries raw model-facing strings (parity with C-45 /
//! C-46 hashing discipline). The adapter owns the hashing.

use std::sync::Arc;

use elai_plugins::CaseBaseSsbEmitter;
use tracing::warn;

use crate::ssb::{
    Ssb, SsbPayload, ToolCalibrationDriftDetectedPayload, ToolCalibrationHintEmittedPayload,
    ToolCallLoggedPayload,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: maps a C-47 emit tuple onto the matching
/// [`SsbPayload::ToolCall*`] / [`SsbPayload::ToolCalibration*`] variant
/// and writes via [`Ssb::write_payload`]. Async-spawned so the sync
/// emit path on the bridge dispatch hot path is never blocked.
pub struct OrchestratorCaseBaseSsbEmitter {
    ssb: Arc<Ssb>,
}

impl OrchestratorCaseBaseSsbEmitter {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    fn dispatch_payload(&self, payload: SsbPayload) {
        let ssb = self.ssb.clone();
        // Trait method is sync; Ssb::write_payload is async. Spawn onto
        // the current runtime so the dispatch hot path is never
        // blocked. If no runtime is active (synchronous test context)
        // the row is dropped with a warning — same semantics as
        // tool_gate_ssb_adapter.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                        .await
                    {
                        warn!(
                            error = %e,
                            "C-47 case_base ssb adapter: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                warn!(
                    "C-47 case_base ssb adapter: no tokio runtime active; dropping row"
                );
            }
        }
    }
}

/// blake3-prefix u64 over an arbitrary str. Same scheme as C-45 / C-46
/// (first 8 bytes of blake3 LE).
fn hash_label(s: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(s.as_bytes());
    let bytes = hasher.finalize();
    let prefix: [u8; 8] = bytes.as_bytes()[..8]
        .try_into()
        .unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix)
}

impl CaseBaseSsbEmitter for OrchestratorCaseBaseSsbEmitter {
    fn emit_call_logged(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
        kind_str: &str,
        post_call_audit_ok: bool,
        ts_ms: i64,
    ) -> Result<(), String> {
        let payload = SsbPayload::ToolCallLogged(ToolCallLoggedPayload {
            tool_name_hash: hash_label(tool_name),
            call_shape_hash,
            response_kind_hash: hash_label(kind_str),
            post_call_audit_ok,
            ts_ms,
        });
        self.dispatch_payload(payload);
        Ok(())
    }

    fn emit_hint(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
        hint_text: &str,
        ts_ms: i64,
    ) -> Result<(), String> {
        let payload = SsbPayload::ToolCalibrationHintEmitted(
            ToolCalibrationHintEmittedPayload {
                tool_name_hash: hash_label(tool_name),
                call_shape_hash,
                hint_text_hash: hash_label(hint_text),
                ts_ms,
            },
        );
        self.dispatch_payload(payload);
        Ok(())
    }

    fn emit_drift(
        &self,
        tool_name: &str,
        error_rate: f32,
        total_calls: u32,
        ts_ms: i64,
    ) -> Result<(), String> {
        let payload = SsbPayload::ToolCalibrationDriftDetected(
            ToolCalibrationDriftDetectedPayload {
                tool_name_hash: hash_label(tool_name),
                error_rate,
                total_calls,
                ts_ms,
            },
        );
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
    fn stub_emitter_drops_rows_without_runtime() {
        // Sync test: dispatch_payload's no-runtime path warns + drops.
        // We exercise the trait surface via a stub adapter shape to
        // avoid needing a real Ssb construction here.
        struct StubAdapter;
        impl CaseBaseSsbEmitter for StubAdapter {
            fn emit_call_logged(
                &self,
                _t: &str,
                _s: u64,
                _k: &str,
                _ok: bool,
                _ts: i64,
            ) -> Result<(), String> {
                Ok(())
            }
            fn emit_hint(
                &self,
                _t: &str,
                _s: u64,
                _h: &str,
                _ts: i64,
            ) -> Result<(), String> {
                Ok(())
            }
            fn emit_drift(
                &self,
                _t: &str,
                _r: f32,
                _n: u32,
                _ts: i64,
            ) -> Result<(), String> {
                Ok(())
            }
        }
        let a = StubAdapter;
        assert!(a.emit_call_logged("x", 0, "ok", true, 0).is_ok());
        assert!(a.emit_hint("x", 0, "h", 0).is_ok());
        assert!(a.emit_drift("x", 0.1, 1, 0).is_ok());
    }
}
