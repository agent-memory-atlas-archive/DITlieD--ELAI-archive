//! J-31: SystemContractEvent SSB emitter (system-composition canvas).
//! Mirrors emit_node_admission_event. A missed SSB row is never a
//! correctness failure (the canvas op is already applied in-process).

use crate::ssb::{
    system_time_ms_i64, Ssb, SsbPayload, SystemContractEventKind,
    SystemContractEventPayload,
};
use std::sync::Arc;

/// Emit a `SystemContractEvent` SSB row best-effort from a sync context.
pub fn emit_system_contract_event(
    ssb: &Arc<Ssb>,
    kind: SystemContractEventKind,
    node_id: &str,
    binding_kind: &str,
    binding_ref: &str,
    verdict: &str,
    details_json: serde_json::Value,
) {
    let ts_ms = system_time_ms_i64();
    let payload = SsbPayload::SystemContractEvent(SystemContractEventPayload {
        kind,
        node_id: node_id.to_owned(),
        binding_kind: binding_kind.to_owned(),
        binding_ref: binding_ref.to_owned(),
        verdict: verdict.to_owned(),
        details_json,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "system_canvas";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                system_canvas.ssb_err = %e,
                "SystemContractEvent SSB emit failed (canvas op unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => { handle.spawn(emit); }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all().build()
            {
                rt.block_on(emit);
            }
        }
    }
}
