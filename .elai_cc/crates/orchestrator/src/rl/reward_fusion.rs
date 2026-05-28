//! E-55 -- Task 3: hybrid PRM + ORM reward fusion.
//!
//! Fuses D-2's PPM step-score vector (PRM signal) with a verifiable ORM
//! signal (BuildGreen / TestsPass / ProbeSemantic / VerifierConsensus from
//! D-3 aggregator) into a single scalar reward. Default weights
//! `{prm: 0.4, orm: 0.6}` -- outcome dominates per Skywork lesson
//! (arXiv:2507.01352): "8B model on 26M curated pairs beats all existing
//! 70B RMs; quality > volume; outcome dominates over learned PRM".
//!
//! Output feeds D-5 BAI Beta-posterior via the additive
//! `ArmPosterior::update_with_fused(fused, threshold)` surface (see
//! `crate::bai` extension). Threshold-to-binary is the default Bernoulli
//! observation; Jeffreys-prior on fractional success is opt-in (left
//! deliberately to a follow-up patch).
//!
//! Pure functions only. No I/O, no clock, no RNG.

use crate::verification_result::VerifierKind;
use serde::{Deserialize, Serialize};

/// Default PRM weight in `FusionWeights`. Per Skywork lesson (arXiv:2507.01352).
pub const FUSION_DEFAULT_PRM_WEIGHT: f32 = 0.4;
/// Default ORM weight in `FusionWeights`. Outcome-dominant.
pub const FUSION_DEFAULT_ORM_WEIGHT: f32 = 0.6;
/// Default Bernoulli threshold for converting fused reward into a
/// Beta-posterior update.
pub const FUSION_BERNOULLI_THRESHOLD: f32 = 0.5;

/// Per-step PRM scores get folded into a single PRM-mean by `compute`.
/// Empty step-score vec collapses to PRM_mean=0.5 (uniform-uninformative)
/// rather than panicking.
const PRM_EMPTY_FALLBACK: f32 = 0.5;

/// Fusion mode. `Linear` blends prm_mean and orm_score by weights.
/// `Hierarchical` filters bottom-50% PRM then uses ORM as tie-breaker
/// (per HRM-style hierarchical reward in the May 2026 RL survey).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FusionMode {
    Linear,
    Hierarchical,
}

impl FusionMode {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Hierarchical => "hierarchical",
        }
    }
}

/// Per-task FusionWeights. Configurable, but both weights MUST be strictly
/// positive (validated via `validate`). Default `{prm: 0.4, orm: 0.6}`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FusionWeights {
    pub prm: f32,
    pub orm: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            prm: FUSION_DEFAULT_PRM_WEIGHT,
            orm: FUSION_DEFAULT_ORM_WEIGHT,
        }
    }
}

impl FusionWeights {
    /// Both weights MUST be strictly positive -- per plan's stub_ban
    /// "configurable but never both = 0".
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.prm.is_finite() || !self.orm.is_finite() {
            return Err("FusionWeights must be finite");
        }
        if self.prm <= 0.0 || self.orm <= 0.0 {
            return Err("FusionWeights must be strictly positive");
        }
        Ok(())
    }

    /// Normalized weights (sum to 1.0). Used by `Linear` mode so the
    /// fused reward stays on `[0, 1]` when both inputs are on `[0, 1]`.
    #[must_use]
    pub fn normalized(&self) -> (f32, f32) {
        let sum = self.prm + self.orm;
        if sum <= 0.0 {
            return (FUSION_DEFAULT_PRM_WEIGHT, FUSION_DEFAULT_ORM_WEIGHT);
        }
        (self.prm / sum, self.orm / sum)
    }
}

/// ORM signal -- the verifiable outcome dimension. Four mutually-exclusive
/// variants map to a scalar `[0.0, 1.0]` reward via `score`. `VerifierConsensus`
/// composes with `crate::verification_result::aggregate` at the producer site;
/// this enum only carries the post-aggregation verdict + the participating
/// verifier kinds for SSB telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrmSignal {
    /// Compile / cargo-check pass.
    BuildGreen { build_ok: bool },
    /// Test runner outcome -- fraction of tests passing on `[0, 1]`.
    TestsPass { fraction_passing: f32 },
    /// E-54 sandbox test_executor result on a single probe (pass / fail).
    ProbeSemantic { passed: bool },
    /// D-3 ConsensusRule outcome over multiple `VerifierKind` sources.
    VerifierConsensus {
        verifiers: Vec<VerifierKind>,
        consensus_passed: bool,
    },
}

impl OrmSignal {
    /// Scalar reward in `[0.0, 1.0]`. Each variant maps deterministically;
    /// `TestsPass` clamps the fraction to `[0, 1]` defensively.
    #[must_use]
    pub fn score(&self) -> f32 {
        match self {
            Self::BuildGreen { build_ok } => {
                if *build_ok {
                    1.0
                } else {
                    0.0
                }
            }
            Self::TestsPass { fraction_passing } => fraction_passing.clamp(0.0, 1.0),
            Self::ProbeSemantic { passed } => {
                if *passed {
                    1.0
                } else {
                    0.0
                }
            }
            Self::VerifierConsensus {
                consensus_passed, ..
            } => {
                if *consensus_passed {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    /// Snake-case wire label (no payload). For SSB rows + dashboard rendering.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::BuildGreen { .. } => "build_green",
            Self::TestsPass { .. } => "tests_pass",
            Self::ProbeSemantic { .. } => "probe_semantic",
            Self::VerifierConsensus { .. } => "verifier_consensus",
        }
    }
}

/// Fused reward observation. Carried into `mona_composite_reward` for
/// penalty application + into BAI `update_with_fused` for posterior update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusedReward {
    pub prm_step_scores: Vec<f32>,
    pub orm_signal: OrmSignal,
    pub fused_reward: f32,
    pub weights: FusionWeights,
    pub mode: FusionMode,
}

impl FusedReward {
    /// Compute fused reward from PRM step-scores + ORM signal. Pure
    /// function -- no clock, no RNG, no I/O.
    #[must_use]
    pub fn compute(
        prm_step_scores: Vec<f32>,
        orm_signal: OrmSignal,
        weights: FusionWeights,
        mode: FusionMode,
    ) -> Self {
        let prm_mean = if prm_step_scores.is_empty() {
            PRM_EMPTY_FALLBACK
        } else {
            let sum: f32 = prm_step_scores.iter().copied().sum();
            (sum / prm_step_scores.len() as f32).clamp(0.0, 1.0)
        };
        let orm_score = orm_signal.score();

        let fused_reward = match mode {
            FusionMode::Linear => {
                let (w_prm, w_orm) = weights.normalized();
                (w_prm * prm_mean + w_orm * orm_score).clamp(0.0, 1.0)
            }
            FusionMode::Hierarchical => {
                // PRM gates bottom-50%; ORM tie-breaks above the gate.
                if prm_mean < 0.5 {
                    // Below PRM gate -- ORM penalty: keep half-credit for
                    // green ORM (encourages exploration) but cap at 0.5.
                    (0.5 * orm_score).clamp(0.0, 0.5)
                } else {
                    orm_score
                }
            }
        };

        Self {
            prm_step_scores,
            orm_signal,
            fused_reward,
            weights,
            mode,
        }
    }

    /// Convert the fused reward into a binary Bernoulli observation for
    /// the BAI Beta-posterior update. Default threshold 0.5.
    #[must_use]
    pub fn as_bernoulli_observation(&self, threshold: f32) -> bool {
        self.fused_reward >= threshold
    }

    /// Convenience: default-threshold Bernoulli observation.
    #[must_use]
    pub fn as_default_bernoulli(&self) -> bool {
        self.as_bernoulli_observation(FUSION_BERNOULLI_THRESHOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_outcome_dominant() {
        let w = FusionWeights::default();
        assert!((w.prm - 0.4).abs() < f32::EPSILON);
        assert!((w.orm - 0.6).abs() < f32::EPSILON);
        assert!(w.validate().is_ok());
        let (np, no) = w.normalized();
        assert!((np + no - 1.0).abs() < f32::EPSILON);
        assert!(no > np, "ORM must dominate by default");
    }

    #[test]
    fn weights_reject_zero_or_negative() {
        assert!(FusionWeights { prm: 0.0, orm: 0.6 }.validate().is_err());
        assert!(FusionWeights { prm: 0.4, orm: 0.0 }.validate().is_err());
        assert!(FusionWeights { prm: -0.1, orm: 0.6 }.validate().is_err());
        assert!(FusionWeights {
            prm: f32::NAN,
            orm: 0.6
        }
        .validate()
        .is_err());
    }

    #[test]
    fn orm_score_maps_all_four_variants() {
        assert_eq!(OrmSignal::BuildGreen { build_ok: true }.score(), 1.0);
        assert_eq!(OrmSignal::BuildGreen { build_ok: false }.score(), 0.0);
        assert!(
            (OrmSignal::TestsPass {
                fraction_passing: 0.75
            }
            .score()
                - 0.75)
                .abs()
                < f32::EPSILON
        );
        assert_eq!(
            OrmSignal::ProbeSemantic { passed: true }.score(),
            1.0
        );
        let consensus = OrmSignal::VerifierConsensus {
            verifiers: vec![VerifierKind::Ppm, VerifierKind::Deterministic],
            consensus_passed: true,
        };
        assert_eq!(consensus.score(), 1.0);
    }

    #[test]
    fn linear_fusion_outcome_dominates() {
        let prm = vec![0.2_f32, 0.3, 0.4];
        let orm = OrmSignal::BuildGreen { build_ok: true };
        let fused = FusedReward::compute(
            prm,
            orm,
            FusionWeights::default(),
            FusionMode::Linear,
        );
        // prm_mean = 0.3, orm_score = 1.0, weights normalized (0.4, 0.6)
        // -> 0.4*0.3 + 0.6*1.0 = 0.72
        assert!((fused.fused_reward - 0.72).abs() < 0.001);
        assert!(fused.as_default_bernoulli());
    }

    #[test]
    fn hierarchical_below_prm_gate_caps_at_half() {
        let prm = vec![0.2_f32, 0.3]; // mean = 0.25 (below 0.5 gate)
        let orm = OrmSignal::BuildGreen { build_ok: true };
        let fused = FusedReward::compute(
            prm,
            orm,
            FusionWeights::default(),
            FusionMode::Hierarchical,
        );
        // Below gate -> 0.5 * 1.0 = 0.5
        assert!((fused.fused_reward - 0.5).abs() < 0.001);
    }

    #[test]
    fn fused_handles_empty_prm_vec() {
        let fused = FusedReward::compute(
            vec![],
            OrmSignal::TestsPass {
                fraction_passing: 1.0,
            },
            FusionWeights::default(),
            FusionMode::Linear,
        );
        // prm_mean fallback 0.5; 0.4*0.5 + 0.6*1.0 = 0.8
        assert!((fused.fused_reward - 0.8).abs() < 0.001);
    }

    #[test]
    fn fused_handles_all_orm_variants() {
        let prm = vec![0.5_f32];
        let variants = [
            OrmSignal::BuildGreen { build_ok: true },
            OrmSignal::TestsPass {
                fraction_passing: 0.8,
            },
            OrmSignal::ProbeSemantic { passed: true },
            OrmSignal::VerifierConsensus {
                verifiers: vec![VerifierKind::Ppm],
                consensus_passed: true,
            },
        ];
        let mut rewards = vec![];
        for orm in variants {
            let f = FusedReward::compute(
                prm.clone(),
                orm,
                FusionWeights::default(),
                FusionMode::Linear,
            );
            rewards.push(f.fused_reward);
        }
        // All four variants compute without panic; at least 2 distinct
        // values (BuildGreen=1.0 vs TestsPass=0.8 differ).
        assert_eq!(rewards.len(), 4);
        let distinct: std::collections::BTreeSet<_> =
            rewards.iter().map(|r| (r * 1000.0) as i32).collect();
        assert!(distinct.len() >= 2);
    }
}
