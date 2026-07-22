//! J-39 T1: observe-only admission ceiling for the phase sizing splitter.
//!
//! Replaces the fixed [`crate::phase_sizing_splitter::DEFAULT_TOKEN_BUDGET`]
//! threshold with one derived from
//! [`crate::context_policy::effective_context_window`] under
//! [`crate::dispatcher::BackendMode::Local`]. When the effective window is 0
//! (misconfigured env), falls back to `DEFAULT_TOKEN_BUDGET` so prior
//! behaviour is preserved rather than silently skipping every split.

use crate::context_policy::effective_context_window;
use crate::dispatcher::BackendMode;
use crate::phase_sizing_splitter::DEFAULT_TOKEN_BUDGET;
use crate::ssb::{
    system_time_ms_i64, NodeAdmissionEventKind, NodeAdmissionEventPayload, Ssb, SsbPayload,
};
use std::sync::Arc;

/// Admission ceiling for the splitter threshold, derived from the live
/// effective context window under Local mode.
///
/// `(effective_context_window(Local) * budget_ratio) as u32`, falling back
/// to `DEFAULT_TOKEN_BUDGET` when the window is 0.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn admission_ceiling_threshold(budget_ratio: f32) -> u32 {
    let ceiling = effective_context_window(BackendMode::Local);
    if ceiling == 0 {
        return DEFAULT_TOKEN_BUDGET;
    }
    ((ceiling as f32) * budget_ratio) as u32
}

/// Emit a `NodeAdmissionEvent` SSB row best-effort from a sync context.
///
/// Mirrors `emit_context_calibration_event`: a missed SSB row is never a
/// correctness failure (the splitter verdict is already applied in-process).
/// When no tokio runtime is available, a transient current-thread runtime
/// is built so the row still lands (do NOT spawn-and-drop).
pub fn emit_node_admission_event(
    ssb: &Arc<Ssb>,
    kind: NodeAdmissionEventKind,
    step_id: &str,
    token_estimate: u32,
    ceiling_threshold: u32,
    details_json: serde_json::Value,
) {
    let ts_ms = system_time_ms_i64();
    let payload = SsbPayload::NodeAdmissionEvent(NodeAdmissionEventPayload {
        kind,
        step_id: step_id.to_owned(),
        token_estimate,
        ceiling_threshold,
        details_json,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "node_admission";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                node_admission.ssb_err = %e,
                "NodeAdmissionEvent SSB emit failed (admission unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}
