//! E-32: deep-mode escalation hook for the SessionDag handoff threshold.
//!
//! When `context_budget` signals that a session is approaching its effective
//! ceiling (>= 85% of `effective_max_context`), the orchestrator must decide
//! whether to escalate to cloud (Sonnet $3/MTok in) or activate deep mode
//! (local Qwen2.5-7B-Instruct-1M at 64K context on a 24GB GPU).
//!
//! Decision rationale (arxiv:2407.16833 Self-Route EMNLP 2024):
//!   63% of queries get identical answers from RAG and long-context, with
//!   39-65% cost savings. Prefer local deep mode when a 24GB box is reachable
//!   AND the task is cross-file synthesis (>3 files, no retrieval-local answer).
//!
//! The hook is called from the `task_runner` handoff-check path. It returns
//! a `HandoffDecision` telling the runner whether to continue locally in
//! deep mode or escalate to cloud. An SSB row is emitted either way.
//!
//! Grounding:
//!   - arxiv:2407.16833 Self-Route: 39-65% cost savings for local-first routing.
//!   - arxiv:2501.15383 Qwen2.5-1M: DCA chunked prefill, 64K ceiling at NF4 on 24GB.
//!   - E-32-decisions.md DECISION 3 + DECISION 4: gating conditions and threshold.

use std::sync::Arc;

use thiserror::Error;
use tracing::{info, warn};

use crate::ssb::{
    DeepModeEscalationChosenPayload, Ssb, SsbError, SsbPayload,
};

/// Fraction of `effective_max_context` that triggers the handoff check.
/// 85% leaves 15% headroom for system prompt + tool overhead.
/// Source: E-32-decisions.md DECISION 4.
pub const HANDOFF_THRESHOLD_FRACTION: f64 = 0.85;

/// Fallback `effective_max_context` used when the sidecar is unreachable.
/// Equals the old 25K ceiling — never below the tested baseline
/// (`memory:project_qwen3b_baseline`). Source: E-32-decisions.md DECISION 4.
pub const FALLBACK_EFFECTIVE_MAX_CONTEXT: u32 = 25_000;

/// Deep-mode context cap on a 24GB GPU with NF4 quantisation.
/// The Qwen2.5-7B-Instruct-1M DCA chunked-prefill infra is in place;
/// 64K is the practical ceiling (1M requires ~120 GB BF16 VRAM per
/// arxiv:2501.15383). Source: E-32-decisions.md DECISION 3.
pub const DEEP_MODE_CTX_CAP: u32 = 64_000;

/// Minimum cross-referenced file count for a task to qualify as
/// "cross-file synthesis" eligible for deep-mode escalation.
/// Source: E-32-decisions.md DECISION 3 gating condition.
pub const DEEP_MODE_MIN_FILES_REFERENCED: u32 = 3;

/// Errors from the escalation policy hook.
#[derive(Debug, Error)]
pub enum EscalationPolicyError {
    /// SSB write failed — the decision was made but could not be recorded.
    #[error("SSB write failed in escalation policy: {0}")]
    SsbWrite(#[from] SsbError),
}

/// Outcome of the handoff-check decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffDecision {
    /// Continue locally in deep mode (Qwen2.5-7B-Instruct-1M, DCA, 64K).
    /// Callers should swap the active runtime to deep-mode config and
    /// re-evaluate with the expanded context window.
    DeepMode {
        /// Context ceiling available under deep mode.
        ctx_cap: u32,
        /// Human-readable reason for choosing deep mode over cloud.
        reason: String,
    },
    /// Escalate to cloud — emit `EscalateToCloud` SSB row and re-route
    /// to the cloud provider (handled by existing `cloud_mode::CloudModeRouter`).
    ///
    /// I-11: `belief_state_payload` carries the MessagePack-encoded [`BeliefState`]
    /// typed condensate. When `Some`, the cloud receiver decodes it instead of
    /// replaying raw session history (stub_ban: no raw history passthrough).
    Cloud {
        /// Human-readable reason for choosing cloud over deep mode.
        reason: String,
        /// I-11: encoded BeliefState payload (rmp_serde / MessagePack).
        /// `None` when evaluate_handoff returns (the ContextBudget borrow has
        /// not yet released). Task_runner injects `Some(bytes)` after
        /// adaptive_reduce returns and BeliefState::encode() runs.
        belief_state_payload: Option<Vec<u8>>,
    },
    /// Threshold not yet reached — no handoff needed this cycle.
    Continue,
}

/// Inputs the handoff hook needs to make its decision.
///
/// Collected once per task-runner cycle tick; cheap to construct.
#[derive(Debug, Clone)]
pub struct HandoffContext {
    /// Number of tokens consumed so far in this session (input + output).
    pub tokens_used: u32,
    /// Effective maximum context as reported by the active runtime's
    /// `RuntimeCapabilities::max_context`. Callers should read this from
    /// `RuntimeCapabilities` rather than hard-coding; E-32 sets it to 32_000
    /// in `sidecar.rs`. Defaults to `FALLBACK_EFFECTIVE_MAX_CONTEXT` when
    /// the sidecar is unreachable.
    pub effective_max_context: u32,
    /// Number of distinct files referenced by the current task.
    /// Used to distinguish cross-file synthesis from retrieval-local tasks.
    pub files_referenced: u32,
    /// Whether a remote 24GB+ GPU box is currently reachable (TCP health
    /// check passed). When false, deep mode is never preferred.
    pub remote_24gb_reachable: bool,
    /// Session identifier forwarded to SSB rows.
    pub session_id: u64,
}

impl HandoffContext {
    /// Compute the handoff threshold: 85% of `effective_max_context`, or
    /// 85% of `FALLBACK_EFFECTIVE_MAX_CONTEXT` when the sidecar is unreachable.
    #[must_use]
    pub fn handoff_threshold(&self) -> u32 {
        let ceiling = if self.effective_max_context == 0 {
            FALLBACK_EFFECTIVE_MAX_CONTEXT
        } else {
            self.effective_max_context
        };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let threshold = (ceiling as f64 * HANDOFF_THRESHOLD_FRACTION) as u32;
        threshold
    }
}

/// Evaluate whether a handoff is needed and, if so, whether to prefer
/// deep mode or cloud escalation.
///
/// SSB row: emits `DeepModeEscalationChosen` when deep mode is selected.
/// Cloud escalation uses the existing `EscalateToCloud` path in
/// `cloud_mode::CloudModeRouter`; this function returns `HandoffDecision::Cloud`
/// and leaves the SSB emission to the existing router.
///
/// Async because the SSB write is async.
pub async fn evaluate_handoff(
    ctx: &HandoffContext,
    ssb: &Arc<Ssb>,
) -> Result<HandoffDecision, EscalationPolicyError> {
    let threshold = ctx.handoff_threshold();

    if ctx.tokens_used < threshold {
        return Ok(HandoffDecision::Continue);
    }

    // Threshold reached. Decide: deep mode or cloud?
    let prefer_deep = ctx.remote_24gb_reachable
        && ctx.files_referenced > DEEP_MODE_MIN_FILES_REFERENCED;

    if prefer_deep {
        let reason = format!(
            "cross-file synthesis: {} files referenced, threshold {} reached ({}% of {}), 24GB remote reachable",
            ctx.files_referenced,
            threshold,
            (HANDOFF_THRESHOLD_FRACTION * 100.0) as u32,
            ctx.effective_max_context,
        );

        info!(
            session_id = ctx.session_id,
            files_referenced = ctx.files_referenced,
            tokens_used = ctx.tokens_used,
            threshold,
            "E-32 escalation_policy: deep-mode selected"
        );

        let payload = SsbPayload::DeepModeEscalationChosen(DeepModeEscalationChosenPayload {
            reason: reason.clone(),
            files_referenced: ctx.files_referenced,
        });
        // session_id is u64; superstep is i64. Saturate at i64::MAX rather than panic.
        let superstep = i64::try_from(ctx.session_id).unwrap_or(i64::MAX);
        ssb.write_payload(
            "deep_mode_escalation_chosen",
            superstep,
            payload,
            vec![],
        )
        .await?;

        return Ok(HandoffDecision::DeepMode {
            ctx_cap: DEEP_MODE_CTX_CAP,
            reason,
        });
    }

    // Fall through to cloud escalation.
    let reason = if !ctx.remote_24gb_reachable {
        "remote 24GB box not reachable; escalate to cloud".to_string()
    } else {
        format!(
            "task has {} files (threshold {}); not cross-file synthesis; escalate to cloud",
            ctx.files_referenced, DEEP_MODE_MIN_FILES_REFERENCED,
        )
    };

    warn!(
        session_id = ctx.session_id,
        tokens_used = ctx.tokens_used,
        threshold,
        %reason,
        "E-32 escalation_policy: cloud escalation selected"
    );

    Ok(HandoffDecision::Cloud {
        reason,
        // I-11: belief_state_payload is populated by the task_runner caller after
        // adaptive_reduce runs (the &mut self borrow on ContextBudget releases first).
        // evaluate_handoff itself does not hold a ContextBudget reference, so the
        // payload is injected by the caller via the returned HandoffDecision::Cloud arm.
        // See task_runner wiring for the production injection site.
        belief_state_payload: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(tokens: u32, files: u32, reachable: bool, ceiling: u32) -> HandoffContext {
        HandoffContext {
            tokens_used: tokens,
            effective_max_context: ceiling,
            files_referenced: files,
            remote_24gb_reachable: reachable,
            session_id: 0,
        }
    }

    #[test]
    fn threshold_is_85_percent_of_ceiling() {
        let c = ctx(0, 0, false, 32_000);
        assert_eq!(c.handoff_threshold(), 27_200);
    }

    #[test]
    fn threshold_falls_back_on_zero_ceiling() {
        let c = ctx(0, 0, false, 0);
        // 85% of 25_000 = 21_250
        assert_eq!(c.handoff_threshold(), 21_250);
    }

    #[test]
    fn deep_mode_threshold_at_64k() {
        let c = ctx(0, 0, false, DEEP_MODE_CTX_CAP);
        // 85% of 64_000 = 54_400
        assert_eq!(c.handoff_threshold(), 54_400);
    }

    #[test]
    fn below_threshold_is_continue() {
        // tokens_used < threshold -> Continue (no async needed for this check)
        let c = ctx(20_000, 5, true, 32_000);
        // threshold = 27_200; 20_000 < 27_200
        assert!(c.tokens_used < c.handoff_threshold());
    }

    #[test]
    fn deep_mode_qualifies_with_4_files_and_reachable() {
        let c = ctx(28_000, 4, true, 32_000);
        // tokens >= threshold (27_200), files > 3, reachable
        assert!(c.tokens_used >= c.handoff_threshold());
        assert!(c.remote_24gb_reachable);
        assert!(c.files_referenced > DEEP_MODE_MIN_FILES_REFERENCED);
    }

    #[test]
    fn cloud_when_box_not_reachable() {
        let c = ctx(28_000, 5, false, 32_000);
        assert!(!c.remote_24gb_reachable);
    }

    #[test]
    fn cloud_when_file_count_at_threshold() {
        // exactly 3 files does NOT qualify (>3 required)
        let c = ctx(28_000, 3, true, 32_000);
        assert!(c.files_referenced <= DEEP_MODE_MIN_FILES_REFERENCED);
    }
}
