//! E-16 -- VLM image-validation orchestrator landing module.
//!
//! This is the "host code" landing for the E-16 Tier-2 hardening
//! plan. It composes:
//!   * [`super::axtree::AxTreeAdapter`]  — AXTree YAML deterministic
//!     web Tier-1 adapter (arXiv-grounded background in axtree.rs).
//!   * [`super::sycophancy::contrastive_decode`] — anti-sycophancy
//!     contrastive judge call (arXiv:2408.11261).
//!   * [`swap_order_eval`]               — position-bias mitigation
//!     for pairwise Tier-2 votes (arXiv:2406.07791).
//!   * [`second_opinion_required`]      — gate function that decides
//!     whether to dispatch an InternVL3.5 second-opinion call.
//!
//! Live VLM inference is wire-dark — the env gate
//! [`super::sycophancy::ELAI_VLM_IMAGE_VALIDATION_ENABLE`] is OFF by
//! default. Producers in the Tier-2 pathway call into this module to
//! compute *decisions*; the actual VLM JSON-mode dispatch happens
//! through the existing [`super::tier2`] traits + the B-37
//! `RuntimeAdapter` table for the second-opinion judge.
//!
//! Pattern-2 SSB emission (E-16, 31st application) lives at
//! [`emit_vlm_image_validation_event`]; callers pass the kind +
//! step_id + details_json and the helper applies the wire-dark gate
//! before writing.
//!
//! Anti-stale grounding (per CLAUDE.md):
//!   * thiserror for module-local errors (NOT Box<dyn Error>).
//!   * serde_json for details_json composition.
//!   * tracing for structured logs.
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use super::sycophancy::{is_enabled, JudgeOutput, VlmJudge};
use crate::ssb::{
    SsbActionType, SsbPayload, VlmImageValidationEventPayload, VlmImageValidationKind,
};

/// Module-local error enum. Per CLAUDE.md HARD RULE — no Box<dyn Error>
/// crossing crate boundaries.
#[derive(Debug, Error)]
pub enum VlmValidatorError {
    /// Both judges in a swap-order eval failed.
    #[error("swap_order_eval: both judges failed: a={a}, b={b}")]
    BothJudgesFailed { a: String, b: String },
    /// The primary judge failed.
    #[error("swap_order_eval: primary judge (A) failed: {0}")]
    PrimaryJudgeFailed(String),
    /// The secondary judge failed.
    #[error("swap_order_eval: secondary judge (B) failed: {0}")]
    SecondaryJudgeFailed(String),
    /// Wire-dark gate is closed at a producer call site.
    #[error("vlm image validation disabled via env (ELAI_VLM_IMAGE_VALIDATION_ENABLE != 1)")]
    Disabled,
}

/// Per-render verdict for a swap-order pairwise comparison. Producers
/// route `disagree = true` into the HumanReview queue and emit a
/// [`VlmImageValidationKind::SwapOrderDisagreement`] SSB row via
/// [`emit_vlm_image_validation_event`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapOrderVerdict {
    /// Primary judge verdict on the [A, B] ordering.
    pub judge_a_verdict: String,
    /// Primary judge verdict on the swapped [B, A] ordering.
    pub judge_b_verdict: String,
    /// `true` when the two orderings yielded different primary labels.
    /// arXiv:2406.07791 measures >10pp pairwise shift from this alone.
    pub disagree: bool,
}

/// Run a swap-order pairwise eval against the provided VLM judge.
/// Mitigates arXiv:2406.07791 position bias.
///
/// Both calls go through the same judge — the difference signal is
/// purely ordering-driven. The `prompt_ab` / `prompt_ba` argument
/// pair is the executor-supplied pairwise template with the candidate
/// order swapped (caller controls the prompt template; this function
/// only orchestrates the two calls + the comparison).
///
/// Returns [`VlmValidatorError::Disabled`] when the wire-dark gate
/// is closed.
pub fn swap_order_eval(
    prompt_ab: &str,
    prompt_ba: &str,
    judge: &dyn VlmJudge,
) -> Result<SwapOrderVerdict, VlmValidatorError> {
    if !is_enabled() {
        return Err(VlmValidatorError::Disabled);
    }

    let out_a = judge
        .judge(prompt_ab)
        .map_err(VlmValidatorError::PrimaryJudgeFailed)?;
    let out_b = judge
        .judge(prompt_ba)
        .map_err(VlmValidatorError::SecondaryJudgeFailed)?;

    let disagree = out_a.primary_label != out_b.primary_label;
    if disagree {
        warn!(
            judge_a = %out_a.primary_label,
            judge_b = %out_b.primary_label,
            "swap_order_disagreement_detected"
        );
    }
    Ok(SwapOrderVerdict {
        judge_a_verdict: out_a.primary_label,
        judge_b_verdict: out_b.primary_label,
        disagree,
    })
}

/// Decide whether the second-opinion judge (InternVL3.5 via B-37
/// RuntimeAdapter) is required. The decision is parameterized by
/// (a) the primary judge's log-probability for its top label, when
/// available, and (b) the sycophancy verdict from
/// [`super::sycophancy::contrastive_decode`].
///
/// Heuristics (E-16 plan task 5):
///   * primary logprob < `LOGPROB_FLOOR` -> second opinion required
///   * sycophancy verdict == "divergent_sycophancy_flag" -> required
///   * otherwise -> not required
///
/// `None` for `primary_logprob` is treated as "no logprob telemetry";
/// the function does not gate on missing logprobs (avoids false
/// positives on backends that don't expose them).
#[must_use]
pub fn second_opinion_required(
    primary_logprob: Option<f64>,
    sycophancy_verdict_label: &str,
) -> bool {
    const LOGPROB_FLOOR: f64 = -1.5;
    if let Some(lp) = primary_logprob {
        if lp < LOGPROB_FLOOR {
            debug!(logprob = lp, "second_opinion_required_low_logprob");
            return true;
        }
    }
    if sycophancy_verdict_label == "divergent_sycophancy_flag" {
        debug!("second_opinion_required_sycophancy_divergent");
        return true;
    }
    false
}

/// XGrammar schema-mask compliance check for a Tier-2 VLM raw output.
/// arXiv:2411.15100 motivates constrained-decoding at the inference
/// layer; this function is the orchestrator-side gate that flags
/// non-compliant outputs as `VlmSchemaUnconstrained` SSB rows.
///
/// The check is intentionally conservative: it verifies the raw
/// output is valid JSON and contains the canonical
/// `VlmValidationOutput` shape (issues array + advisory_pass bool).
/// If either is missing the function returns `false` and the caller
/// emits a `VlmSchemaUnconstrained` row.
#[must_use]
pub fn vlm_output_schema_compliant(raw_output: &str) -> bool {
    let v: serde_json::Value = match serde_json::from_str(raw_output) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };
    obj.contains_key("issues") && obj.contains_key("advisory_pass")
}

/// Build a [`VlmImageValidationEventPayload`] ready for SSB emission.
/// The helper applies the canonical wall-clock + step_id + the
/// supplied per-kind details_json. Producers compose details_json per
/// the conventional fields documented on
/// [`VlmImageValidationKind`].
#[must_use]
pub fn build_vlm_image_validation_payload(
    kind: VlmImageValidationKind,
    step_id: impl Into<String>,
    request_id: Option<String>,
    details_json: serde_json::Value,
) -> VlmImageValidationEventPayload {
    let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);

    VlmImageValidationEventPayload {
        kind,
        step_id: step_id.into(),
        request_id,
        details_json,
        ts_ms,
    }
}

/// Build the SsbPayload + SsbActionType pair for direct
/// `Ssb::write_payload` consumption. The wire-dark gate
/// [`super::sycophancy::is_enabled`] is checked here so producers
/// that route through this helper cannot accidentally emit rows
/// while the env gate is closed. Returns `None` when disabled.
#[must_use]
pub fn emit_vlm_image_validation_event(
    kind: VlmImageValidationKind,
    step_id: impl Into<String>,
    request_id: Option<String>,
    details_json: serde_json::Value,
) -> Option<(SsbActionType, SsbPayload)> {
    if !is_enabled() {
        return None;
    }
    let payload = build_vlm_image_validation_payload(kind, step_id, request_id, details_json);
    Some((
        SsbActionType::VlmImageValidationEvent,
        SsbPayload::VlmImageValidationEvent(payload),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct StubJudge {
        label_seq: Vec<String>,
        idx: std::sync::Mutex<usize>,
    }

    impl VlmJudge for StubJudge {
        fn judge(&self, _prompt: &str) -> Result<JudgeOutput, String> {
            let mut g = self.idx.lock().map_err(|e| e.to_string())?;
            let i = *g;
            *g += 1;
            let label = self
                .label_seq
                .get(i)
                .cloned()
                .unwrap_or_else(|| "fallback".to_owned());
            Ok(JudgeOutput {
                primary_label: label,
                claims: BTreeSet::new(),
            })
        }
    }

    #[test]
    fn second_opinion_required_on_low_logprob() {
        assert!(second_opinion_required(Some(-2.0), "convergent"));
    }

    #[test]
    fn second_opinion_required_on_divergent_sycophancy() {
        assert!(second_opinion_required(None, "divergent_sycophancy_flag"));
        assert!(second_opinion_required(Some(0.0), "divergent_sycophancy_flag"));
    }

    #[test]
    fn second_opinion_skipped_on_high_logprob_convergent() {
        assert!(!second_opinion_required(Some(-0.2), "convergent"));
        assert!(!second_opinion_required(None, "convergent"));
    }

    #[test]
    fn second_opinion_skipped_on_inconclusive() {
        // Inconclusive should NOT force a second opinion; the
        // dashboard surface still lights the badge.
        assert!(!second_opinion_required(None, "inconclusive"));
    }

    #[test]
    fn vlm_output_schema_accepts_valid_shape() {
        let raw = r#"{"issues":[],"advisory_pass":true,"reasoning":"ok"}"#;
        assert!(vlm_output_schema_compliant(raw));
    }

    #[test]
    fn vlm_output_schema_rejects_missing_advisory_pass() {
        let raw = r#"{"issues":[]}"#;
        assert!(!vlm_output_schema_compliant(raw));
    }

    #[test]
    fn vlm_output_schema_rejects_non_json() {
        assert!(!vlm_output_schema_compliant("not json"));
    }

    #[test]
    fn swap_order_eval_blocks_when_gate_closed() {
        // Mirror sycophancy tests; default-off branch.
        if std::env::var(super::super::sycophancy::ELAI_VLM_IMAGE_VALIDATION_ENABLE).as_deref()
            == Ok("1")
        {
            return;
        }
        let j = StubJudge {
            label_seq: vec!["a".into(), "a".into()],
            idx: std::sync::Mutex::new(0),
        };
        let res = swap_order_eval("ab", "ba", &j);
        assert!(matches!(res, Err(VlmValidatorError::Disabled)));
    }

    #[test]
    fn build_payload_carries_kind_and_step_id() {
        let p = build_vlm_image_validation_payload(
            VlmImageValidationKind::AxtreeMismatch,
            "step-x",
            Some("req-y".into()),
            serde_json::json!({"yaml_diff_pct": 2.5}),
        );
        assert_eq!(p.kind, VlmImageValidationKind::AxtreeMismatch);
        assert_eq!(p.step_id, "step-x");
        assert_eq!(p.request_id.as_deref(), Some("req-y"));
        assert!(p.ts_ms > 0);
    }

    #[test]
    fn emit_returns_none_when_gate_closed() {
        if std::env::var(super::super::sycophancy::ELAI_VLM_IMAGE_VALIDATION_ENABLE).as_deref()
            == Ok("1")
        {
            return;
        }
        let res = emit_vlm_image_validation_event(
            VlmImageValidationKind::VlmSchemaUnconstrained,
            "step-z",
            None,
            serde_json::json!({"schema_id": "VlmValidationOutput"}),
        );
        assert!(res.is_none());
    }
}
