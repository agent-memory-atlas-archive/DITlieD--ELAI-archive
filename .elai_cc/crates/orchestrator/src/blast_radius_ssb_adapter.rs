//! G-3 (deterministic runtime blast-radius gates, s-56068) -- orchestrator-side
//! emitter that writes a [`crate::ssb::SsbPayload::BlastRadiusBlocked`] row when
//! a fail-closed gate refuses an operation at a REAL production entrypoint:
//!
//!   * the FS arm (`FsEscape`) -- a shell-EXPANDED filesystem-mutating target
//!     resolved OUTSIDE the per-run worktree root, emitted from
//!     `production_agent_rpc::translate_tool_request` (the `fs_write` / `bash`
//!     dispatch arms that already hold the live `Arc<Ssb>`).
//!   * the egress arm (`EgressDenied`) -- an outbound host was NUL/control-
//!     rejected or resolved outside the pinned allow-set, emitted from
//!     `web_research_orchestrator::fetch_and_extract` (which holds the
//!     `ssb_writer: Option<Arc<Ssb>>`).
//!
//! This is the OFF-AGENT ARTIFACT the wiring gate measures (wiring-enforcement
//! R4 / R6): the row is written by a REAL production dispatch entrypoint
//! through the SAME live `Arc<Ssb>` that wrote the rest of the stream, NOT by a
//! `#[cfg(test)]` caller and NOT by the agent that produced the feature.
//!
//! Wire-dark posture: NONE. A blocked blast-radius operation is a security
//! event that MUST always be recorded. The only opt-out is the kill-switch
//! `ELAI_BLAST_RADIUS_DISABLE` (read at the gate call site), which disarms the
//! gate so the emit is never reached. There is intentionally NO enable flag
//! (wiring-enforcement R9: a security gate that ships default-off is wire-dark
//! by design).
//!
//! Identifier discipline: `worktree_root_blake3` is a blake3 hex-prefix of the
//! canonical worktree root (NOT the absolute path verbatim). The full pre- AND
//! post-expansion command strings ARE recorded (that is the `#10077` forensics
//! fix); they are the command the agent issued and the command the shell would
//! actually have run, neither of which is a secret.

use std::sync::Arc;

use tracing::warn;

use crate::ssb::{BlastRadiusBlockedPayload, BlastRadiusKind, Ssb, SsbPayload};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Kill-switch env var. When set (to any value), the gate is disarmed at its
/// call site and this emitter is never reached. Default-ON otherwise (R9).
pub const BLAST_RADIUS_DISABLE_ENV: &str = "ELAI_BLAST_RADIUS_DISABLE";

/// True when the blast-radius gate kill-switch is set. Read at the gate call
/// site BEFORE the gate runs; mirrors `production_agent_rpc::taint_gate_disabled`.
pub fn blast_radius_gate_disabled() -> bool {
    std::env::var_os(BLAST_RADIUS_DISABLE_ENV).is_some()
}

/// Write one `blast_radius_blocked` SSB row for a fail-closed gate block.
///
/// Async-spawned so the synchronous dispatch path is never blocked on the SSB
/// write; if no tokio runtime is active (a synchronous test context) the row is
/// dropped with a warning -- mirrors `taint_flow_ssb_adapter::dispatch_payload`.
///
/// * `ssb`            -- the live SSB handle threaded through the dispatch fan.
/// * `kind`           -- which boundary was crossed (FS escape vs egress).
/// * `pre_expansion`  -- the original, un-expanded command/argv.
/// * `post_expansion` -- the resolved/canonicalised target (FS) or rejected
///                       host+ip (egress).
/// * `target_or_host` -- the specific resolved FS target, or rejected host.
/// * `worktree_root_blake3` -- blake3 hex-prefix of the run's containment root
///                       (`""` for the egress arm).
/// * `trust_label`    -- the G-1 provenance join when present (`""` otherwise).
/// * `now_ms`         -- injected wall-clock ms (the caller passes its
///                       `ClockProvider` value; never `SystemTime::now()` here).
#[allow(clippy::too_many_arguments)]
pub fn emit_blast_radius_blocked(
    ssb: &Arc<Ssb>,
    kind: BlastRadiusKind,
    pre_expansion: &str,
    post_expansion: &str,
    target_or_host: &str,
    worktree_root_blake3: &str,
    trust_label: &str,
    now_ms: i64,
) {
    let payload = SsbPayload::BlastRadiusBlocked(BlastRadiusBlockedPayload {
        kind,
        pre_expansion_command: pre_expansion.to_owned(),
        post_expansion_command: post_expansion.to_owned(),
        target_or_host: target_or_host.to_owned(),
        worktree_root_blake3: worktree_root_blake3.to_owned(),
        trust_label: trust_label.to_owned(),
        ts_ms: now_ms,
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
                        "G-3 blast_radius ssb adapter: write_payload failed (non-fatal)"
                    );
                }
            });
        }
        Err(_) => {
            warn!("G-3 blast_radius ssb adapter: no tokio runtime active; dropping row");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_switch_default_off_means_gate_armed() {
        // With the env var unset the gate is ARMED (default-ON, R9).
        std::env::remove_var(BLAST_RADIUS_DISABLE_ENV);
        assert!(!blast_radius_gate_disabled(), "gate must be armed by default");
    }

    #[test]
    fn emit_builds_blast_radius_payload() {
        // Exercise the payload-construction path in isolation: the action_type
        // of the produced payload must be BlastRadiusBlocked.
        let payload = SsbPayload::BlastRadiusBlocked(BlastRadiusBlockedPayload {
            kind: BlastRadiusKind::FsEscape,
            pre_expansion_command: "rm -rf ~/x".into(),
            post_expansion_command: "/home/user/x".into(),
            target_or_host: "/home/user/x".into(),
            worktree_root_blake3: "abc123".into(),
            trust_label: String::new(),
            ts_ms: 1,
        });
        assert_eq!(
            payload.action_type(),
            crate::ssb::SsbActionType::BlastRadiusBlocked
        );
    }
}
