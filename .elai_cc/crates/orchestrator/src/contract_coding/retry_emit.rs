//! Tranche B-6 quad C-50 -- ContractCodingEvent emit helpers for the
//! fix-and-rerun retry loop.
//!
//! Three thin wrappers ([`emit_retry_attempted`], [`emit_retry_resolved`],
//! [`emit_retry_abandoned`]) keep the call sites in
//! `task_runner::fix_and_rerun_until_green` one-line and ensure the
//! Pattern-2 payload shape stays consistent across producers. Wire-dark
//! unless `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1`; the gate is checked
//! inside the helpers so callers don't have to thread env reads through
//! the retry loop.
//!
//! See `workflow/reasoning/B6-quad-tranche-decisions.md`.

use std::sync::Arc;

use crate::ssb::{
    blake3_label_prefix, env_gate_truthy, system_time_ms_i64, ContractCodingEventKind,
    ContractCodingEventPayload, Ssb, SsbPayload, ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV,
    ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Tranche K / B.2 Option-B -- per-mode gate dispatcher.
///
/// Picks the correct env gate for the calling project mode so the web
/// and non-web rollouts stay TRULY INDEPENDENT (per the W3 finding in
/// the Tranche K context package -- option (ii) was chosen over the
/// shared OR-gate option (i) precisely so flipping one mode does NOT
/// silently widen the other).
///
/// * `non_web == false` (the existing web call sites): reads
///   [`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV`].
/// * `non_web == true` (the new Tranche K non-web caller): reads
///   [`ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV`].
///
/// Default (both env vars unset) returns `false` so the producer stays
/// wire-dark.
#[must_use]
pub fn c50_emit_gate_truthy(non_web: bool) -> bool {
    if non_web {
        env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV)
    } else {
        env_gate_truthy(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV)
    }
}

/// Truncate the carrier summary string to a reasonable wire-side cap so
/// `details_json` rows stay bounded. 240 chars matches the existing
/// `truncate_summary` cap used by `emit_autonomous_decision` in the same
/// retry loop (task_runner.rs:1093).
const DETAILS_SUMMARY_MAX_CHARS: usize = 240;

fn truncate_for_wire(s: &str) -> String {
    if s.chars().count() <= DETAILS_SUMMARY_MAX_CHARS {
        return s.to_owned();
    }
    let prefix: String = s.chars().take(DETAILS_SUMMARY_MAX_CHARS).collect();
    format!("{prefix}... [truncated]")
}

/// Saturate a u32 iteration counter into the u8 wire slot. Per
/// `fix_and_rerun_contract.rs:278` the production cap is 5; saturation is
/// defensive against future cap bumps.
fn saturate_iter_u8(n: u32) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// Tranche B-6 quad C-50 -- emit one
/// [`ContractCodingEventKind::RetryAttempted`] row.
///
/// Fires on EACH `FixerValidatorReport::Failed` branch inside the
/// fix_and_rerun retry loop (task_runner.rs:1057). Wire-dark unless
/// the per-mode gate is on:
/// * web callers (`non_web=false`): `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1`
/// * non-web callers (`non_web=true`): `ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE=1`
///
/// Tranche K / B.2 Option-B: the `non_web` discriminator routes through
/// [`c50_emit_gate_truthy`] so the two modes stay independent.
pub async fn emit_retry_attempted(
    ssb: &Arc<Ssb>,
    step_id: &str,
    iterations: u32,
    failure_summary: &str,
    non_web: bool,
) {
    if !c50_emit_gate_truthy(non_web) {
        return;
    }
    let payload = SsbPayload::ContractCodingEvent(ContractCodingEventPayload {
        kind: ContractCodingEventKind::RetryAttempted,
        step_id_hash: blake3_label_prefix(step_id),
        iterations: saturate_iter_u8(iterations),
        severity: 0,
        details_json: serde_json::json!({
            "caller_site": "fix_and_rerun_until_green::failed_branch",
            "failure_summary": truncate_for_wire(failure_summary),
            "step_id_label": step_id,
            "non_web": non_web,
        }),
        ts_ms: system_time_ms_i64(),
    });
    if let Err(e) = ssb
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(
            target: "orchestrator::contract_coding::retry_emit",
            error = %e,
            "C-50 retry_attempted emit failed (non-fatal)"
        );
    }
}

/// Tranche B-6 quad C-50 -- emit one
/// [`ContractCodingEventKind::RetryResolved`] row.
///
/// Fires when the validator pass after a fixer turn returns `Green`
/// (task_runner.rs:1055 Healed branch). Wire-dark unless the per-mode
/// gate is on:
/// * web callers (`non_web=false`): `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1`
/// * non-web callers (`non_web=true`): `ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE=1`
pub async fn emit_retry_resolved(
    ssb: &Arc<Ssb>,
    step_id: &str,
    iterations: u32,
    non_web: bool,
) {
    if !c50_emit_gate_truthy(non_web) {
        return;
    }
    let payload = SsbPayload::ContractCodingEvent(ContractCodingEventPayload {
        kind: ContractCodingEventKind::RetryResolved,
        step_id_hash: blake3_label_prefix(step_id),
        iterations: saturate_iter_u8(iterations),
        severity: 0,
        details_json: serde_json::json!({
            "caller_site": "fix_and_rerun_until_green::healed_branch",
            "final_state": "healed",
            "step_id_label": step_id,
            "non_web": non_web,
        }),
        ts_ms: system_time_ms_i64(),
    });
    if let Err(e) = ssb
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(
            target: "orchestrator::contract_coding::retry_emit",
            error = %e,
            "C-50 retry_resolved emit failed (non-fatal)"
        );
    }
}

/// Tranche B-6 quad C-50 -- emit one
/// [`ContractCodingEventKind::RetryAbandoned`] row.
///
/// Fires when the retry loop exhausts `max_iterations` and escalates to
/// a human reviewer (task_runner.rs:1109 EscalatedToHuman branch).
/// Severity = 1 (critical). Wire-dark unless the per-mode gate is on:
/// * web callers (`non_web=false`): `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1`
/// * non-web callers (`non_web=true`): `ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE=1`
pub async fn emit_retry_abandoned(
    ssb: &Arc<Ssb>,
    step_id: &str,
    iterations: u32,
    escalation_reason: &str,
    non_web: bool,
) {
    if !c50_emit_gate_truthy(non_web) {
        return;
    }
    let payload = SsbPayload::ContractCodingEvent(ContractCodingEventPayload {
        kind: ContractCodingEventKind::RetryAbandoned,
        step_id_hash: blake3_label_prefix(step_id),
        iterations: saturate_iter_u8(iterations),
        severity: 1,
        details_json: serde_json::json!({
            "caller_site": "fix_and_rerun_until_green::escalated_branch",
            "escalation_reason": truncate_for_wire(escalation_reason),
            "step_id_label": step_id,
            "non_web": non_web,
        }),
        ts_ms: system_time_ms_i64(),
    });
    if let Err(e) = ssb
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(
            target: "orchestrator::contract_coding::retry_emit",
            error = %e,
            "C-50 retry_abandoned emit failed (non-fatal)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_for_wire_passes_short_strings() {
        let s = "short summary";
        assert_eq!(truncate_for_wire(s), s);
    }

    #[test]
    fn truncate_for_wire_caps_long_strings() {
        let long: String = "x".repeat(300);
        let trimmed = truncate_for_wire(&long);
        assert!(trimmed.ends_with("... [truncated]"));
        assert!(trimmed.chars().count() <= DETAILS_SUMMARY_MAX_CHARS + 32);
    }

    #[test]
    fn saturate_iter_u8_caps_at_max() {
        assert_eq!(saturate_iter_u8(5), 5);
        assert_eq!(saturate_iter_u8(255), 255);
        assert_eq!(saturate_iter_u8(256), u8::MAX);
        assert_eq!(saturate_iter_u8(u32::MAX), u8::MAX);
    }
}
