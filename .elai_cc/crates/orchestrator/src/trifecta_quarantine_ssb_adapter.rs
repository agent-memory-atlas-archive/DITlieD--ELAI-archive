//! G-7 (lethal-trifecta dual-LLM quarantine, s-56068) -- orchestrator-side
//! emitter that writes a [`crate::ssb::SsbPayload::TrifectaQuarantined`] row when
//! ALL THREE lethal-trifecta legs co-occur at a tool-call chokepoint
//! (secret-present via the G-9 redaction detector + untrusted span via the G-1
//! taint table + egress-capable tool via the capability gate) and the untrusted
//! span is held back from the tool-caller.
//!
//! This is the OFF-AGENT ARTIFACT the wiring gate measures (wiring-enforcement
//! R4 / R6): the row is written by the REAL production dispatch chokepoint
//! (`production_agent_rpc::translate_tool_request`), not by a `#[cfg(test)]`
//! caller and not by the agent that produced the feature.
//!
//! Wire-dark posture: a quarantine event is a security event that is recorded by
//! default. The only opt-out is the kill-switch `ELAI_TRIFECTA_QUARANTINE_DISABLE`
//! (read at the gate call site), which disarms the gate so this emitter is never
//! reached. There is intentionally NO enable flag (wiring-enforcement R9: a
//! security gate that ships default-off is wire-dark by design).
//!
//! Identifier discipline: `untrusted_origin_blake3` is a blake3 hex-prefix of the
//! untrusted origin (URL / server+tool / path / sandbox id), NOT a raw secret and
//! NOT the raw untrusted span. The component flags + `sink_tool` + verdict are
//! non-secret labels.

use std::sync::Arc;

use tracing::warn;

use crate::quarantine::TrifectaComponents;
use crate::ssb::{Ssb, SsbPayload, TrifectaQuarantinedPayload};
use crate::taint_flow_ssb_adapter::origin_blake3_prefix;
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Write one `trifecta_quarantined` SSB row for a detected + quarantined
/// trifecta. blake3 is the ONLY hash used (CLAUDE.md HARD RULE). The raw secret
/// and the raw untrusted span NEVER land in the payload — only the blake3 of the
/// origin + the boolean component flags.
///
/// * `ssb`               — the live SSB handle threaded through the dispatch fan.
/// * `components`         — the three detected legs (secret / untrusted / egress).
/// * `untrusted_origin`   — the raw untrusted ingest origin; hashed here.
/// * `sink_tool`          — the egress / destructive sink the agent was about to
///                          dispatch.
/// * `quarantined`        — whether the span was routed through the no-tool
///                          sub-LLM (true) or passed through clean (false).
/// * `sub_llm_verdict`    — the no-tool sub-LLM verdict label.
/// * `task_id`            — the task / turn id for dashboard correlation.
#[allow(clippy::too_many_arguments)]
pub fn emit_trifecta_quarantined(
    ssb: &Arc<Ssb>,
    components: TrifectaComponents,
    untrusted_origin: &str,
    sink_tool: &str,
    quarantined: bool,
    sub_llm_verdict: &str,
    task_id: &str,
) {
    let payload = SsbPayload::TrifectaQuarantined(TrifectaQuarantinedPayload {
        task_id: task_id.to_owned(),
        untrusted_origin_blake3: origin_blake3_prefix(untrusted_origin),
        sink_tool: sink_tool.to_owned(),
        secret_present: components.secret_present,
        untrusted_span: components.untrusted_span,
        egress_available: components.egress_available,
        quarantined,
        sub_llm_verdict: sub_llm_verdict.to_owned(),
    });
    dispatch_payload(ssb.clone(), payload);
}

/// Spawn one SSB write onto the current tokio runtime. Returns immediately; the
/// write completes asynchronously. Mirrors the G-1 taint_flow adapter exactly.
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
                        "G-7 trifecta_quarantined ssb adapter: write_payload failed (non-fatal)"
                    );
                }
            });
        }
        Err(_) => {
            warn!("G-7 trifecta_quarantined ssb adapter: no tokio runtime active; dropping row");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_builds_trifecta_payload() {
        let components = TrifectaComponents {
            secret_present: true,
            untrusted_span: true,
            egress_available: true,
        };
        let payload = SsbPayload::TrifectaQuarantined(TrifectaQuarantinedPayload {
            task_id: "t-1".into(),
            untrusted_origin_blake3: origin_blake3_prefix("https://evil.example/x"),
            sink_tool: "curl".into(),
            secret_present: components.secret_present,
            untrusted_span: components.untrusted_span,
            egress_available: components.egress_available,
            quarantined: true,
            sub_llm_verdict: "quarantined".into(),
        });
        assert_eq!(
            payload.action_type(),
            crate::ssb::SsbActionType::TrifectaQuarantined
        );
    }
}
