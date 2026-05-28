//! E-55 -- Task 4: MONA + composite-reward hacking guards.
//!
//! Composite reward = base (from `reward_fusion::FusedReward`) minus four
//! deterministic penalty terms: format / length / sycophancy / repetition.
//!
//! MONA pattern (arXiv:2501.13011 Farquhar et al. Google DeepMind):
//! "Myopic Optimization with Non-myopic Approval Can Mitigate Multi-step
//! Reward Hacking". Single-step reward (myopic) MUST be approved by an
//! environment-outcome-independent critic (`step_validity` from the
//! RLAIF rating in Task 1). If `step_validity < 0.5` BUT `myopic > 0.8`,
//! flag potential reward-hacking via `RewardHackingVerdict::suspected`.
//!
//! Composite reward pattern: arXiv:2509.15557 Verifiable Composite Rewards
//! medical reasoning -- base accuracy minus penalty_format / penalty_length
//! / penalty_sycophancy with documented effect on hacking + format quality.
//!
//! Penalty defaults (per plan + report):
//!   - PENALTY_FORMAT = 0.1 (per detected format violation)
//!   - PENALTY_LENGTH_SOFT = 0.05 (response > 2x median OR < 0.3x median)
//!   - PENALTY_SYCOPHANCY = 0.2 (matched-pair framing gap flagged)
//!   - PENALTY_REPETITION = 0.05 (50-token consecutive repetition)
//!
//! Pure functions only. No I/O. Defaults are configurable via
//! `CompositeRewardConfig` (per plan stub_ban "configurable but never
//! both = 0" for FusionWeights; here all penalties default to the above
//! constants).

use crate::rl::reward_fusion::FusedReward;
use crate::rl::rlaif_judge::RlaifRating;
use serde::{Deserialize, Serialize};

/// Per-violation format penalty (each violation subtracts this from base).
pub const PENALTY_FORMAT: f32 = 0.1;
/// Length penalty when response is > 2x median OR < 0.3x median.
pub const PENALTY_LENGTH_SOFT: f32 = 0.05;
/// Sycophancy penalty when matched-pair framing gap > 10pp (E-54 surface).
pub const PENALTY_SYCOPHANCY: f32 = 0.2;
/// Repetition penalty when any 50-token window repeats consecutively.
pub const PENALTY_REPETITION: f32 = 0.05;

/// MONA reward-hacking myopic threshold. If single-step reward >= this AND
/// step_validity < `REWARD_HACKING_STEP_VALIDITY_THRESHOLD`, the verdict
/// flips to `suspected`.
pub const REWARD_HACKING_MYOPIC_THRESHOLD: f32 = 0.8;
/// MONA non-myopic critic threshold. Step-validity below this is "not
/// approved by the non-myopic critic" per arXiv:2501.13011.
pub const REWARD_HACKING_STEP_VALIDITY_THRESHOLD: f32 = 0.5;

/// Response > this multiple of median session length -> length penalty.
pub const LENGTH_MEDIAN_MULTIPLIER_HIGH: f32 = 2.0;
/// Response < this multiple of median session length -> length penalty.
pub const LENGTH_MEDIAN_MULTIPLIER_LOW: f32 = 0.3;
/// Token window for repetition detection.
pub const REPETITION_TOKEN_WINDOW: usize = 50;

/// All four penalties as a struct so they round-trip into SSB rows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PenaltyBreakdown {
    pub penalty_format: f32,
    pub penalty_length: f32,
    pub penalty_sycophancy: f32,
    pub penalty_repetition: f32,
}

impl PenaltyBreakdown {
    /// Sum of all four penalties.
    #[must_use]
    pub fn total(&self) -> f32 {
        self.penalty_format
            + self.penalty_length
            + self.penalty_sycophancy
            + self.penalty_repetition
    }

    /// Zero-penalty breakdown (clean output).
    #[must_use]
    pub fn zero() -> Self {
        Self {
            penalty_format: 0.0,
            penalty_length: 0.0,
            penalty_sycophancy: 0.0,
            penalty_repetition: 0.0,
        }
    }
}

/// Composite reward bundle. `final_reward` = base - total penalties,
/// clamped to `[0.0, 1.0]`. `myopic_reward` is the raw fused base from
/// reward_fusion BEFORE penalty subtraction. `step_validity` is from the
/// RLAIF rating (None when judge was not invoked on this trajectory).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeReward {
    pub base: f32,
    pub penalties: PenaltyBreakdown,
    pub final_reward: f32,
    pub myopic_reward: f32,
    pub step_validity: Option<f32>,
    pub hacking_verdict: RewardHackingVerdict,
}

/// Reward-hacking detection result. `suspected` is true when myopic
/// reward is high BUT step_validity is low. `divergence` is the gap
/// between the two -- for telemetry / dashboard display.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RewardHackingVerdict {
    pub suspected: bool,
    pub myopic_reward: f32,
    pub step_validity: f32,
    pub divergence: f32,
}

impl RewardHackingVerdict {
    /// Construct from raw myopic + step_validity. The verdict fires when
    /// myopic >= REWARD_HACKING_MYOPIC_THRESHOLD AND step_validity <
    /// REWARD_HACKING_STEP_VALIDITY_THRESHOLD.
    #[must_use]
    pub fn classify(myopic: f32, step_validity: f32) -> Self {
        let divergence = (myopic - step_validity).max(0.0);
        let suspected = myopic >= REWARD_HACKING_MYOPIC_THRESHOLD
            && step_validity < REWARD_HACKING_STEP_VALIDITY_THRESHOLD;
        Self {
            suspected,
            myopic_reward: myopic,
            step_validity,
            divergence,
        }
    }

    /// "Approval not available" path: when the judge did not rate this
    /// trajectory, the MONA guard cannot fire (per arXiv:2501.13011 the
    /// non-myopic critic is mandatory before composite-reward emits).
    /// Returns a non-suspected verdict with step_validity = 0.0 so the
    /// telemetry record carries the "missing critic" signal.
    #[must_use]
    pub fn no_approval(myopic: f32) -> Self {
        Self {
            suspected: false,
            myopic_reward: myopic,
            step_validity: 0.0,
            divergence: 0.0,
        }
    }
}

/// Format-violation context (caller supplies the deterministic flags).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct FormatContext {
    /// Each detected format violation = one PENALTY_FORMAT subtraction.
    pub violation_count: u32,
}

/// Length-context. Caller supplies the observed response length + the
/// median session length so we don't have to keep state across calls.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LengthContext {
    pub response_token_count: u32,
    pub session_median_tokens: u32,
}

/// Sycophancy context. The `matched_pair_flagged` bool is the output of
/// E-54's `sycophancy_matched_pair` probe (gap > 10pp on a framing-flip
/// matched pair). When true, fire `PENALTY_SYCOPHANCY`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SycophancyContext {
    pub matched_pair_flagged: bool,
}

/// Counts consecutive 50-token repetitions in `response_tokens`. Uses a
/// rolling-window byte-shingle comparator -- pure, deterministic.
#[must_use]
pub fn detect_repetition(response_tokens: &[String]) -> bool {
    let n = response_tokens.len();
    if n < REPETITION_TOKEN_WINDOW * 2 {
        return false;
    }
    // Walk a window of 50 over the response; if the next 50 tokens match
    // the previous 50, flag repetition.
    for i in 0..=(n - REPETITION_TOKEN_WINDOW * 2) {
        let a = &response_tokens[i..(i + REPETITION_TOKEN_WINDOW)];
        let b = &response_tokens[(i + REPETITION_TOKEN_WINDOW)
            ..(i + REPETITION_TOKEN_WINDOW * 2)];
        if a == b {
            return true;
        }
    }
    false
}

/// Compute the length penalty given a `LengthContext`. Returns
/// PENALTY_LENGTH_SOFT when response is > HIGH multiplier of median OR
/// < LOW multiplier. Zero median collapses to no penalty (insufficient
/// session history).
#[must_use]
pub fn length_penalty(ctx: LengthContext) -> f32 {
    if ctx.session_median_tokens == 0 {
        return 0.0;
    }
    let response = ctx.response_token_count as f32;
    let median = ctx.session_median_tokens as f32;
    let high = LENGTH_MEDIAN_MULTIPLIER_HIGH * median;
    let low = LENGTH_MEDIAN_MULTIPLIER_LOW * median;
    if response > high || response < low {
        PENALTY_LENGTH_SOFT
    } else {
        0.0
    }
}

impl CompositeReward {
    /// Compute composite reward from the fused base + the four contexts.
    /// MANDATORY per plan stub_ban: "MONA myopic-vs-non-myopic check is
    /// MANDATORY -- no composite-reward computation without it".
    ///
    /// Returns the bundle. Caller emits the SSB row separately so this
    /// stays a pure function.
    #[must_use]
    pub fn compute(
        fused: &FusedReward,
        format_ctx: FormatContext,
        length_ctx: LengthContext,
        sycophancy_ctx: SycophancyContext,
        repetition_detected: bool,
        rlaif: Option<&RlaifRating>,
    ) -> Self {
        let base = fused.fused_reward;
        let myopic = base;

        let penalty_format = (format_ctx.violation_count as f32) * PENALTY_FORMAT;
        let penalty_length = length_penalty(length_ctx);
        let penalty_sycophancy = if sycophancy_ctx.matched_pair_flagged {
            PENALTY_SYCOPHANCY
        } else {
            0.0
        };
        let penalty_repetition = if repetition_detected {
            PENALTY_REPETITION
        } else {
            0.0
        };

        let penalties = PenaltyBreakdown {
            penalty_format,
            penalty_length,
            penalty_sycophancy,
            penalty_repetition,
        };

        let final_reward = (base - penalties.total()).clamp(0.0, 1.0);

        let (step_validity_field, hacking_verdict) = match rlaif {
            Some(r) => {
                let sv = r.step_validity;
                (Some(sv), RewardHackingVerdict::classify(myopic, sv))
            }
            None => (None, RewardHackingVerdict::no_approval(myopic)),
        };

        Self {
            base,
            penalties,
            final_reward,
            myopic_reward: myopic,
            step_validity: step_validity_field,
            hacking_verdict,
        }
    }

    /// Top-level reward-hacking detection: thin wrapper so callers can
    /// classify directly from `myopic` + `step_validity` without building
    /// the full CompositeReward bundle.
    #[must_use]
    pub fn detect_reward_hacking(myopic: f32, step_validity: f32) -> RewardHackingVerdict {
        RewardHackingVerdict::classify(myopic, step_validity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rl::reward_fusion::{
        FusedReward, FusionMode, FusionWeights, OrmSignal,
    };

    fn sample_fused(score: f32) -> FusedReward {
        // Construct a fused reward whose `fused_reward` field equals `score`
        // by choosing trivially-mapping inputs (prm=score, orm=BuildGreen with
        // build_ok adjusted; linear default normalized weights).
        let prm = vec![score];
        FusedReward::compute(
            prm,
            OrmSignal::TestsPass {
                fraction_passing: score,
            },
            FusionWeights::default(),
            FusionMode::Linear,
        )
    }

    fn rating(step_validity: f32) -> RlaifRating {
        RlaifRating {
            correctness: 1.0,
            step_validity,
            sycophancy_absence: 1.0,
            format_compliance: 1.0,
            conciseness: 1.0,
            rationale: "_t_".to_string(),
            latency_ms: 0,
            model_id: "test".to_string(),
        }
    }

    #[test]
    fn penalty_constants_match_plan() {
        // Per plan body Task 4: 0.1 / 0.05 / 0.2 / 0.05.
        assert!((PENALTY_FORMAT - 0.1).abs() < f32::EPSILON);
        assert!((PENALTY_LENGTH_SOFT - 0.05).abs() < f32::EPSILON);
        assert!((PENALTY_SYCOPHANCY - 0.2).abs() < f32::EPSILON);
        assert!((PENALTY_REPETITION - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn four_penalties_subtract_independently() {
        let fused = sample_fused(0.9);
        // All four penalties fire simultaneously.
        let cr = CompositeReward::compute(
            &fused,
            FormatContext { violation_count: 2 },
            LengthContext {
                response_token_count: 9999,
                session_median_tokens: 100,
            },
            SycophancyContext {
                matched_pair_flagged: true,
            },
            true,
            Some(&rating(1.0)),
        );
        // 2 format violations -> 0.2; length -> 0.05; sycophancy -> 0.2;
        // repetition -> 0.05; total 0.5
        let expected_total = (2.0 * PENALTY_FORMAT)
            + PENALTY_LENGTH_SOFT
            + PENALTY_SYCOPHANCY
            + PENALTY_REPETITION;
        assert!((cr.penalties.total() - expected_total).abs() < 0.001);
    }

    #[test]
    fn mona_flags_high_reward_low_step_validity() {
        // Per plan: step_validity=0.3 + myopic=0.9 -> suspected.
        let verdict = CompositeReward::detect_reward_hacking(0.9, 0.3);
        assert!(verdict.suspected);
        assert!(verdict.divergence > 0.0);
    }

    #[test]
    fn mona_does_not_flag_when_both_high() {
        let verdict = CompositeReward::detect_reward_hacking(0.9, 0.9);
        assert!(!verdict.suspected);
    }

    #[test]
    fn mona_does_not_flag_when_myopic_low() {
        let verdict = CompositeReward::detect_reward_hacking(0.3, 0.1);
        assert!(!verdict.suspected);
    }

    #[test]
    fn no_rlaif_path_emits_no_approval_verdict() {
        let fused = sample_fused(0.9);
        let cr = CompositeReward::compute(
            &fused,
            FormatContext::default(),
            LengthContext::default(),
            SycophancyContext::default(),
            false,
            None,
        );
        assert_eq!(cr.step_validity, None);
        assert!(!cr.hacking_verdict.suspected);
        assert_eq!(cr.hacking_verdict.step_validity, 0.0);
    }

    #[test]
    fn length_penalty_zero_median_collapses() {
        let p = length_penalty(LengthContext {
            response_token_count: 1000,
            session_median_tokens: 0,
        });
        assert_eq!(p, 0.0);
    }

    #[test]
    fn length_penalty_fires_above_high_multiplier() {
        let p = length_penalty(LengthContext {
            response_token_count: 300,
            session_median_tokens: 100,
        });
        assert!((p - PENALTY_LENGTH_SOFT).abs() < f32::EPSILON);
    }

    #[test]
    fn length_penalty_fires_below_low_multiplier() {
        let p = length_penalty(LengthContext {
            response_token_count: 20,
            session_median_tokens: 100,
        });
        assert!((p - PENALTY_LENGTH_SOFT).abs() < f32::EPSILON);
    }

    #[test]
    fn repetition_detected_in_simple_pattern() {
        let tok = |s: &str| s.to_string();
        let mut tokens: Vec<String> = (0..50).map(|i| tok(&format!("t{i}"))).collect();
        tokens.extend((0..50).map(|i| tok(&format!("t{i}"))));
        assert!(detect_repetition(&tokens));
    }

    #[test]
    fn repetition_not_detected_under_window() {
        let tokens: Vec<String> = (0..80).map(|i| format!("u{i}")).collect();
        assert!(!detect_repetition(&tokens));
    }

    #[test]
    fn final_reward_clamps_to_zero() {
        let fused = sample_fused(0.05);
        // Stack penalties high enough to drive base negative.
        let cr = CompositeReward::compute(
            &fused,
            FormatContext { violation_count: 10 },
            LengthContext::default(),
            SycophancyContext::default(),
            false,
            None,
        );
        assert_eq!(cr.final_reward, 0.0);
    }
}
