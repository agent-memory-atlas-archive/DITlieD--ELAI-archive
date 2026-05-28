//! E-8 task 4: admit path for optimized templates produced by the
//! Python optimizer service.
//!
//! Flow:
//!   1. Python optimizer writes OptimizedTemplate values to disk
//!      (or in-process for tests).
//!   2. Rust side reads + validates blake3 content hash via the
//!      canonical `reasoning_template::Blake3Hash` (no new hash impl,
//!      DECISION-3 + MEDIUM-5 from context-package).
//!   3. Runs the LPO no-frozen-edit predicate over the spec's frozen
//!      block-kinds (`baby_step_spec_lpo`).
//!   4. Optional cross-domain canary check (admission refused if the
//!      candidate regresses by ≥ -5pp absolute on any canary family --
//!      reusing the E-7 threshold per plan task 6).
//!   5. Admits as `ArchiveStatus::Candidate` (canonical lifecycle per
//!      reasoning_template.rs:258-265 -- "SHADOW" is plan-drift per
//!      DECISION-2).
//!   6. The held-out promotion gate + Two-Gate (E-7 promotion_gate)
//!      runs in a separate pass; this module exposes the admit
//!      verdict.
//!
//! Determinism: every public function is pure (no I/O) except where
//! noted. Live LLM calls live in the Python side; this module reads the
//! Python output as data.
//!
//! cfg-gating: the `MapElitesArchive` / `two_gate_promote` consumers
//! are gated behind `darwinian-evolve` per DECISION-3. The
//! `OptimizedTemplate` value type + the LPO + content-hash gates work
//! unconditionally so the default-features build still compiles +
//! tests the admission preconditions.
//!
//! No `.unwrap()` / `.expect()` outside test code.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::baby_step_spec_lpo::{
    assert_no_frozen_edit, BabyStepSpec, BlockKind, LpoViolation,
};
use crate::reasoning_template::{ArchiveStatus, Blake3Hash};

/// One cross-domain canary result. `delta_pct` is the candidate's lift
/// against the parent on the canary family; negative = regression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossDomainCanaryResult {
    /// Canary family slug (e.g. "math_word_problems").
    pub family: String,
    /// Lift in percentage points (candidate - parent on that family).
    pub delta_pct: f64,
}

/// Optimized-template value produced by the Python optimizer. Carried
/// across the FS boundary as JSON; the Rust admit path treats it as
/// data, NOT as a side-effecting message.
///
/// The `target_blocks` field names the BlockKinds the optimizer
/// actually rewrote. Used by the LPO guard to refuse any rewrite of a
/// frozen block (DETERMINISTIC_CHECK / ACCEPTANCE / INTEGRATION /
/// EXIT_CRITERIA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizedTemplate {
    /// New template body text.
    pub content: String,
    /// blake3 of `content` (canonical Blake3Hash, NOT raw [u8;32]).
    pub content_blake3: Blake3Hash,
    /// blake3 of the parent template body (the optimizer was
    /// initialised from this).
    pub parent_template_blake3: Blake3Hash,
    /// Semver-ish optimizer version string ("gepa-0.1.1+elai-e8").
    pub optimizer_version: String,
    /// uuid-v4 hex of the optimization run that produced this.
    pub optimization_run_id: String,
    /// Validation-set lift in percentage points vs the parent.
    pub val_lift_pct: f64,
    /// Held-out test set lift in percentage points (after transfer
    /// check).
    pub held_out_lift_pct: f64,
    /// Cross-domain canary measurements (per E-7 task 7 surface).
    pub cross_domain_canary_results: Vec<CrossDomainCanaryResult>,
    /// BlockKinds the optimizer rewrote. Must all be LPO-editable.
    pub target_blocks: Vec<BlockKind>,
}

/// Admission verdict on a single optimized template.
///
/// `Candidate` is the canonical entry state per reasoning_template
/// lifecycle (DECISION-2). The plan's "SHADOW" label is rejected -- no
/// such variant exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveAdmission {
    /// Always `ArchiveStatus::Candidate` on first admit (canonical
    /// lifecycle Candidate -> Promoted | Retired).
    pub archive_status: ArchiveStatus,
    /// The hash that uniquely names the admitted template.
    pub content_blake3: Blake3Hash,
    /// The run-id correlator.
    pub optimization_run_id: String,
    /// `true` IFF cross-domain canary passed AND held-out lift met
    /// minimum.
    pub admitted: bool,
}

/// Cross-domain regression threshold from E-7 task 7. Candidate is
/// rejected IFF any canary family records `delta_pct <
/// REGRESSION_ABS_THRESHOLD_PCT` (i.e. -5pp absolute regression).
pub const REGRESSION_ABS_THRESHOLD_PCT: f64 = -5.0;

/// Default minimum validation lift for admission (plan §ACCEPTANCE-4:
/// admission requires val_lift_pct >= 5).
pub const DEFAULT_MIN_VAL_LIFT_PCT: f64 = 5.0;

/// Default allowed gap between val_lift_pct and held_out_lift_pct
/// (plan §ACCEPTANCE-4: "held-out_lift_pct >= val_lift_pct minus
/// ε=2pp").
pub const DEFAULT_HELD_OUT_EPSILON_PCT: f64 = 2.0;

/// Admit-path failure modes.
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdmitError {
    /// Recomputed blake3 of `content` did not match `content_blake3`.
    #[error("content hash mismatch (expected={expected_hex})")]
    HashMismatch { expected_hex: String },
    /// Optimizer rewrote a frozen BabyStepSpec block (LPO violation).
    #[error("LPO violation: {0}")]
    LpoViolation(LpoViolation),
    /// Validation lift below the configured minimum.
    #[error(
        "val_lift_pct {val} below minimum {min} (admission refused per ACCEPTANCE-4)"
    )]
    ValLiftBelowMinimum { val: f64, min: f64 },
    /// Held-out lift more than epsilon below validation lift (overfit
    /// signal per DSPy 20/80 split discipline).
    #[error(
        "held_out_lift_pct {held_out} more than {epsilon}pp below val_lift_pct {val} (overfit-risk; admission refused)"
    )]
    HeldOutOverfit { val: f64, held_out: f64, epsilon: f64 },
    /// Cross-domain canary recorded a regression at or below the
    /// REGRESSION_ABS_THRESHOLD_PCT band.
    #[error(
        "cross-domain regression on family `{family}` (delta_pct {delta_pct} <= threshold {threshold})"
    )]
    CrossDomainRegression {
        family: String,
        delta_pct: f64,
        threshold: f64,
    },
}

/// Recompute the blake3 of `content` (no parent hash mixed in -- the
/// `OptimizedTemplate.content_blake3` field is the bare content hash;
/// the parent-mixed variant lives in archive_chain).
#[must_use]
pub fn compute_optimized_content_hash(content: &str) -> Blake3Hash {
    let mut h = blake3::Hasher::new();
    h.update(content.as_bytes());
    Blake3Hash::from(h.finalize())
}

/// Verify the optimized template's `content_blake3` matches a fresh
/// blake3 over `content`. Pure / no I/O.
pub fn verify_content_hash(t: &OptimizedTemplate) -> Result<(), AdmitError> {
    let recomputed = compute_optimized_content_hash(&t.content);
    if recomputed == t.content_blake3 {
        Ok(())
    } else {
        Err(AdmitError::HashMismatch {
            expected_hex: t.content_blake3.as_hex(),
        })
    }
}

/// LPO guard: refuse if any of `target_blocks` is frozen, or if any
/// referenced block is missing from `spec`.
pub fn assert_lpo_safe(
    spec: &BabyStepSpec,
    target_blocks: &[BlockKind],
) -> Result<(), AdmitError> {
    for k in target_blocks {
        assert_no_frozen_edit(spec, *k).map_err(AdmitError::LpoViolation)?;
    }
    Ok(())
}

/// Cross-domain canary guard: refuse if any family regressed beyond
/// the absolute threshold.
pub fn assert_no_cross_domain_regression(
    results: &[CrossDomainCanaryResult],
) -> Result<(), AdmitError> {
    for r in results {
        if r.delta_pct <= REGRESSION_ABS_THRESHOLD_PCT {
            return Err(AdmitError::CrossDomainRegression {
                family: r.family.clone(),
                delta_pct: r.delta_pct,
                threshold: REGRESSION_ABS_THRESHOLD_PCT,
            });
        }
    }
    Ok(())
}

/// Held-out / overfit guard: refuse if held_out_lift_pct is more than
/// `epsilon_pct` below val_lift_pct.
pub fn assert_held_out_not_overfit(
    val_lift_pct: f64,
    held_out_lift_pct: f64,
    epsilon_pct: f64,
) -> Result<(), AdmitError> {
    if held_out_lift_pct + epsilon_pct < val_lift_pct {
        return Err(AdmitError::HeldOutOverfit {
            val: val_lift_pct,
            held_out: held_out_lift_pct,
            epsilon: epsilon_pct,
        });
    }
    Ok(())
}

/// Minimum validation lift guard.
pub fn assert_val_lift_min(val_lift_pct: f64, min_pct: f64) -> Result<(), AdmitError> {
    if val_lift_pct < min_pct {
        return Err(AdmitError::ValLiftBelowMinimum {
            val: val_lift_pct,
            min: min_pct,
        });
    }
    Ok(())
}

/// Top-level admit: chain all four guards, returning an
/// `ArchiveAdmission` on success. First admit is ALWAYS
/// `ArchiveStatus::Candidate` per DECISION-2.
pub fn admit_optimized_template(
    t: &OptimizedTemplate,
    spec: &BabyStepSpec,
) -> Result<ArchiveAdmission, AdmitError> {
    verify_content_hash(t)?;
    assert_lpo_safe(spec, &t.target_blocks)?;
    assert_val_lift_min(t.val_lift_pct, DEFAULT_MIN_VAL_LIFT_PCT)?;
    assert_held_out_not_overfit(
        t.val_lift_pct,
        t.held_out_lift_pct,
        DEFAULT_HELD_OUT_EPSILON_PCT,
    )?;
    assert_no_cross_domain_regression(&t.cross_domain_canary_results)?;
    Ok(ArchiveAdmission {
        archive_status: ArchiveStatus::Candidate,
        content_blake3: t.content_blake3,
        optimization_run_id: t.optimization_run_id.clone(),
        admitted: true,
    })
}

/// Build a contract-test fixture: a valid `OptimizedTemplate` whose
/// hash + LPO + lift values all pass. Test-only.
#[cfg(test)]
pub(crate) fn fixture_admit_ok() -> (OptimizedTemplate, BabyStepSpec) {
    use crate::baby_step_spec_lpo::SpecBlock;

    let content = "improved role prompt body".to_string();
    let content_blake3 = compute_optimized_content_hash(&content);
    let parent_template_blake3 =
        compute_optimized_content_hash("original role prompt body");
    let spec = BabyStepSpec::from_blocks(vec![
        SpecBlock {
            kind: BlockKind::Task,
            body: "do x".into(),
        },
        SpecBlock {
            kind: BlockKind::RolePromptBody,
            body: "original".into(),
        },
        SpecBlock {
            kind: BlockKind::DeterministicCheck,
            body: "shell command".into(),
        },
    ]);
    let t = OptimizedTemplate {
        content,
        content_blake3,
        parent_template_blake3,
        optimizer_version: "gepa-0.1.1+elai-e8".into(),
        optimization_run_id: "run-e8-fixture".into(),
        val_lift_pct: 8.0,
        held_out_lift_pct: 7.0,
        cross_domain_canary_results: vec![
            CrossDomainCanaryResult {
                family: "math".into(),
                delta_pct: 1.5,
            },
            CrossDomainCanaryResult {
                family: "code".into(),
                delta_pct: 0.0,
            },
        ],
        target_blocks: vec![BlockKind::RolePromptBody],
    };
    (t, spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baby_step_spec_lpo::SpecBlock;

    #[test]
    fn first_admit_status_is_candidate() {
        let (t, spec) = fixture_admit_ok();
        let admission = admit_optimized_template(&t, &spec).expect("admit ok");
        assert_eq!(admission.archive_status, ArchiveStatus::Candidate);
        assert!(admission.admitted);
    }

    #[test]
    fn lpo_frozen_block_rejected() {
        let (mut t, spec) = fixture_admit_ok();
        t.target_blocks = vec![BlockKind::DeterministicCheck];
        let result = admit_optimized_template(&t, &spec);
        match result {
            Err(AdmitError::LpoViolation(LpoViolation::FrozenBlockMutated { kind })) => {
                assert_eq!(kind, "deterministic_check");
            }
            other => panic!("expected LpoViolation::FrozenBlockMutated, got {other:?}"),
        }
    }

    #[test]
    fn val_lift_below_minimum_rejected() {
        let (mut t, spec) = fixture_admit_ok();
        t.val_lift_pct = 3.0; // < default 5.0
        t.held_out_lift_pct = 2.5;
        let result = admit_optimized_template(&t, &spec);
        assert!(matches!(
            result,
            Err(AdmitError::ValLiftBelowMinimum { val, min })
                if (val - 3.0).abs() < 1e-9 && (min - 5.0).abs() < 1e-9
        ));
    }

    #[test]
    fn held_out_overfit_rejected() {
        let (mut t, spec) = fixture_admit_ok();
        t.val_lift_pct = 10.0;
        t.held_out_lift_pct = 5.0; // gap 5 > epsilon 2.
        let result = admit_optimized_template(&t, &spec);
        assert!(matches!(result, Err(AdmitError::HeldOutOverfit { .. })));
    }

    #[test]
    fn cross_domain_regression_rejected() {
        let (mut t, spec) = fixture_admit_ok();
        t.cross_domain_canary_results = vec![CrossDomainCanaryResult {
            family: "math".into(),
            delta_pct: -10.0, // beyond -5pp threshold.
        }];
        let result = admit_optimized_template(&t, &spec);
        assert!(matches!(
            result,
            Err(AdmitError::CrossDomainRegression { family, .. }) if family == "math"
        ));
    }

    #[test]
    fn content_hash_mismatch_rejected() {
        let (mut t, spec) = fixture_admit_ok();
        // tamper without re-hashing
        t.content = "tampered".into();
        let result = admit_optimized_template(&t, &spec);
        assert!(matches!(result, Err(AdmitError::HashMismatch { .. })));
    }

    #[test]
    fn lpo_block_missing_rejected() {
        let spec = BabyStepSpec::from_blocks(vec![SpecBlock {
            kind: BlockKind::Task,
            body: "x".into(),
        }]);
        let (mut t, _) = fixture_admit_ok();
        t.target_blocks = vec![BlockKind::RolePromptBody];
        let result = admit_optimized_template(&t, &spec);
        match result {
            Err(AdmitError::LpoViolation(LpoViolation::BlockMissing { kind })) => {
                assert_eq!(kind, "role_prompt_body");
            }
            other => panic!("expected LpoViolation::BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn boundary_val_lift_exactly_at_minimum_admits() {
        let (mut t, spec) = fixture_admit_ok();
        t.val_lift_pct = DEFAULT_MIN_VAL_LIFT_PCT;
        t.held_out_lift_pct = DEFAULT_MIN_VAL_LIFT_PCT - DEFAULT_HELD_OUT_EPSILON_PCT;
        assert!(admit_optimized_template(&t, &spec).is_ok());
    }

    #[test]
    fn cross_domain_at_minus_5pp_exactly_blocks() {
        // Threshold is "delta_pct <= -5.0" so a -5 is BLOCKED.
        let (mut t, spec) = fixture_admit_ok();
        t.cross_domain_canary_results = vec![CrossDomainCanaryResult {
            family: "math".into(),
            delta_pct: -5.0,
        }];
        let result = admit_optimized_template(&t, &spec);
        assert!(matches!(result, Err(AdmitError::CrossDomainRegression { .. })));
    }
}
