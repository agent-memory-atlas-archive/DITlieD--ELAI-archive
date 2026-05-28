//! E-20 -- local-cloud routing substrate.
//!
//! Consolidates the 4 routing fragments the E-20 plan calls out into a
//! single sibling file under [`crate::cost`] (alongside E-39
//! `automix_gate` + `cost_router` + `cloud_abort` + `velocity_cap` +
//! `free_tier_guard`):
//!
//!   * [`confidence::compute_score`] -- entropy-of-first-token-derived
//!     confidence score in `[0.0, 1.0]` (arXiv:2510.01237 +
//!     arXiv:2511.07396; probe + perplexity dominate verbalized-self-
//!     confidence, NEVER ask the model "are you confident?").
//!   * [`HybridEscalationGate`] -- replaces the C-5
//!     `retry_cap == 1 && reason == "L3_failed"` escalation with
//!     "escalate when `confidence_score < L_HYBRID_THRESHOLD`". Composes
//!     with [`crate::cost::automix_gate::verify_before_escalate`] -- the
//!     gate emits its verdict, AutoMix self-verifies before the cloud
//!     dispatch happens (E-39 SHIPPED kept as the secondary safety net).
//!   * [`majority_vote`] -- sample-of-3 routing decision (arXiv:2406.12334
//!     same-prompt different-result + arXiv:2504.01282 Prompt-Reverse
//!     Inconsistency). 3 classifier samples vote on the routing decision
//!     (NOT the answer) for routing stability at temp>0.
//!   * [`semantic_reasoning::choose_reasoning_mode`] -- pre-cloud
//!     extended-thinking gate (arXiv:2510.08731 When to Reason IBM
//!     vLLM semantic router; -47.1% latency / -48.5% tokens / +10.2pp
//!     MMLU-Pro by NOT invoking reasoning when the prompt does not
//!     benefit).
//!
//! This module deliberately does NOT live at `orchestrator::routing::*`
//! because that namespace is owned by C-46 Uno-Orchestra (`routing/
//! difficulty.rs::TaskDifficultyClassifier` is an AgentRole router, not
//! a RuntimeBackend router) and was rejected for E-20 in
//! [E39_CLOUD_SPEND_PATH_DRIFT] / E-20 decision DECISION-1
//! (see workflow/reasoning/E-20-decisions.md).
//!
//! Wire-dark default; emission of [`crate::ssb::SsbActionType::
//! LocalCloudRoutingEvent`] rows requires
//! `ELAI_LOCAL_CLOUD_ROUTING_ENABLE=1` at the producer's call site.
//!
//! Source grounding (verified May 2026 via WebSearch):
//!   * arXiv:2511.07396 C3PO Nov 2025 -- conformal-cost-bounded
//!     cascade; self-supervised threshold tuning beats heuristic
//!     thresholds.
//!   * arXiv:2406.18665 RouteLLM LMSYS 2024 -- BERT-scale preference-
//!     trained router; 2x cost reduction at near-GPT-4 quality on
//!     Chatbot Arena preference data (80k battles).
//!   * arXiv:2310.12963 AutoMix ICLR 2025 -- POMDP self-verify cascade;
//!     >50% cost reduction at iso-quality across 5 datasets x 5 LMs.
//!   * arXiv:2510.01237 Confidence-Aware Routing Oct 2025 -- F1
//!     0.61 -> 0.82, 40% compute cut; documents that probe + perplexity
//!     signals dominate verbalized-self-confidence (do NOT ask the model
//!     to self-report confidence as routing signal).
//!   * arXiv:2510.08731 When to Reason IBM NeurIPS 2025 -- vLLM
//!     semantic router; -47.1% latency / -48.5% tokens / +10.2pp
//!     MMLU-Pro by withholding reasoning when prompt does not need it.
//!   * arXiv:2502.00674 Self-MoA Feb 2025 -- aggregating the SINGLE
//!     best LLM (+6.6% AlpacaEval / +3.8% avg across MMLU/CRUX/MATH);
//!     documents anti-pattern of mixing different LLMs (weakest-member
//!     drag).
//!   * arXiv:2511.03808 Prompt-Difficulty Prediction Nov 2025 --
//!     intermediate-representation probes from s1.1-32B for difficulty.
//!   * arXiv:2305.05176 FrugalGPT TMLR 2024 -- cascade with up to 98%
//!     cost reduction at GPT-4 parity.
//!
//! See workflow/reasoning/E-20-decisions.md for the path-drift
//! resolution and Pattern-2 consolidation rationale.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Env-gate constant. When set to `"1"` at process start, every emit
/// site under [`crate::cost::cloud_route`] writes its
/// [`crate::ssb::SsbActionType::LocalCloudRoutingEvent`] row. When
/// unset / `"0"`, the helpers short-circuit and return without
/// touching SSB.
///
/// Mirrors the precedent at
/// [`crate::cost::ENV_COST_TOKEN_ECONOMICS_ENABLE`] and the E-38
/// `ELAI_AI_ROADMAP_SIGNALS_ENABLE` gate. Wire-dark default keeps
/// production sessions on the existing C-5 cloud-mode behaviour until
/// the user opts in.
pub const ENV_LOCAL_CLOUD_ROUTING_ENABLE: &str = "ELAI_LOCAL_CLOUD_ROUTING_ENABLE";

/// Returns `true` when [`ENV_LOCAL_CLOUD_ROUTING_ENABLE`] is set to
/// `"1"`. Single source of truth for the emission gate so every
/// producer agrees on the live/wire-dark check.
#[must_use]
pub fn local_cloud_routing_enabled() -> bool {
    std::env::var(ENV_LOCAL_CLOUD_ROUTING_ENABLE)
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Confidence score (E-20 task 1)
// ---------------------------------------------------------------------------

/// Confidence score in `[0.0, 1.0]`. 1.0 = high confidence (low
/// entropy, sharp distribution); 0.0 = low confidence (high entropy,
/// flat distribution).
pub type ConfidenceScore = f32;

/// Initial HYBRID escalation threshold per E-20 plan task 2. Calibrated
/// via the 200-task holdout (task 10, RUNPOD-PENDING). Per-L-layer
/// thresholds override this default; see the holdout TOML for the
/// final values.
///
/// Value: 0.55 = below = escalate to cloud; above = keep local. Chosen
/// halfway between AutoMix's reported 0.5 inflection point (arXiv:
/// 2310.12963) and the C3PO conformal upper bound (arXiv:2511.07396
/// fig. 3 cross-dataset average).
pub const L_HYBRID_THRESHOLD_DEFAULT: ConfidenceScore = 0.55;

/// Maximum entropy for a discrete distribution of size `vocab_size`.
/// `log(vocab_size)` in nats. Used by [`confidence::compute_score`] to
/// normalise raw entropy into the `[0.0, 1.0]` confidence range.
#[must_use]
fn max_entropy_nats(vocab_size: usize) -> f32 {
    if vocab_size <= 1 {
        return 0.0;
    }
    (vocab_size as f32).ln()
}

pub mod confidence {
    //! Confidence score derivation from first-token logprob/entropy.
    //!
    //! Per arXiv:2510.01237 + arXiv:2511.07396, the entropy of the
    //! first-token distribution is a strong post-hoc confidence signal
    //! that dominates verbalized self-confidence. The sidecar already
    //! returns `mean_logprob` (B-39 reasoning surface); when the full
    //! first-token distribution is available we compute Shannon entropy
    //! directly. Otherwise we fall back to `1.0 - sigmoid(-mean_logprob)`
    //! shaped as a defensive default.
    //!
    //! Returned scores are in `[0.0, 1.0]` -- HIGHER = MORE confident.

    use super::{max_entropy_nats, ConfidenceScore};

    /// Compute the confidence score from a first-token probability
    /// distribution (already softmax-normalised). High confidence =
    /// low entropy = score near 1.0.
    ///
    /// Returns `0.0` for an empty distribution (defensive: no signal
    /// is treated as "definitely low confidence" so the gate defaults
    /// to escalation).
    #[must_use]
    pub fn from_first_token_distribution(probs: &[f32]) -> ConfidenceScore {
        if probs.is_empty() {
            return 0.0;
        }
        // Shannon entropy in nats. Clamp probabilities to a tiny floor
        // so log(0) does not produce -inf and contaminate the sum.
        let eps: f32 = 1e-12;
        let entropy: f32 = probs
            .iter()
            .map(|p| {
                let p = p.max(eps);
                -p * p.ln()
            })
            .sum();
        let h_max = max_entropy_nats(probs.len());
        if h_max <= 0.0 {
            // Degenerate (single token) -- treat as fully confident.
            return 1.0;
        }
        let normalised = (entropy / h_max).clamp(0.0, 1.0);
        // 1 - normalised entropy ==> high entropy -> low confidence.
        1.0 - normalised
    }

    /// Defensive fallback when only a `mean_logprob` is available.
    /// Maps `mean_logprob` (typically `[-5.0, 0.0]` for sane local
    /// sidecar samples) through a logistic that anchors `-2.3` (e^-2.3
    /// ~ 0.1 token prob) at the 0.5 boundary.
    #[must_use]
    pub fn from_mean_logprob(mean_logprob: f32) -> ConfidenceScore {
        // Sigmoid centred at -2.3 with slope 1.0. Output in (0.0, 1.0).
        let z = mean_logprob + 2.3;
        let raw = 1.0 / (1.0 + (-z).exp());
        raw.clamp(0.0, 1.0)
    }

    /// Convenience: returns the confidence score directly from an
    /// optional distribution + always-available `mean_logprob`. Prefers
    /// the distribution when present.
    #[must_use]
    pub fn compute_score(
        first_token_probs: Option<&[f32]>,
        mean_logprob: f32,
    ) -> ConfidenceScore {
        match first_token_probs {
            Some(probs) if !probs.is_empty() => from_first_token_distribution(probs),
            _ => from_mean_logprob(mean_logprob),
        }
    }
}

// ---------------------------------------------------------------------------
// HybridEscalationGate (E-20 task 2)
// ---------------------------------------------------------------------------

/// 3-arm escalation verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridEscalationVerdict {
    /// Confidence is above threshold; keep the response local. NO
    /// cloud dispatch.
    SkipCloud,
    /// Confidence is below threshold; escalate to a stronger cloud
    /// backend (typically Sonnet).
    EscalateCloud,
    /// Confidence is far below threshold AND criticality is High;
    /// escalate directly to Opus (gated by typed-confirmation per the
    /// E-39 `cost_router::route_for_difficulty_and_budget` Opus path).
    EscalateOpus,
}

impl HybridEscalationVerdict {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SkipCloud => "skip_cloud",
            Self::EscalateCloud => "escalate_cloud",
            Self::EscalateOpus => "escalate_opus",
        }
    }
}

impl fmt::Display for HybridEscalationVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the escalation verdict. Emitted on the SSB row so the
/// dashboard can render "why did this go to cloud?".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationReason {
    /// Confidence score crossed the threshold.
    Confidence,
    /// Same-tier retry budget exhausted (C-5 retry_cap=1 fallback;
    /// kept as secondary safety net per E-20 plan task 2).
    RetryExhausted,
    /// L1 secret scanner blocked a cloud dispatch (composes with
    /// [`crate::ssb::SsbActionType::RouteBlocked`] reason="privacy_block").
    PrivacyBlock,
}

impl EscalationReason {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confidence => "confidence",
            Self::RetryExhausted => "retry_exhausted",
            Self::PrivacyBlock => "privacy_block",
        }
    }
}

/// L-layer (1..=5) tag for per-layer threshold override. L1 = quick
/// scaffolding, L5 = deep multi-file architecture. The plan recommends
/// per-L-layer thresholds; this struct holds the overrides and falls
/// back to [`L_HYBRID_THRESHOLD_DEFAULT`] when no override exists.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LLayerThresholds {
    /// `[l1, l2, l3, l4, l5]` override thresholds. `None` -> default.
    pub per_layer: [Option<ConfidenceScore>; 5],
}

impl LLayerThresholds {
    /// Threshold for `l_layer` (1..=5). Out-of-range layers fall back
    /// to the default.
    #[must_use]
    pub fn threshold_for(&self, l_layer: u8) -> ConfidenceScore {
        let idx = match l_layer {
            1..=5 => (l_layer as usize) - 1,
            _ => return L_HYBRID_THRESHOLD_DEFAULT,
        };
        self.per_layer[idx].unwrap_or(L_HYBRID_THRESHOLD_DEFAULT)
    }
}

/// Calibrated escalation gate. Replaces the C-5
/// `retry_cap == 1 && reason == "L3_failed"` logic with a calibrated
/// confidence-threshold cascade per arXiv:2511.07396 C3PO.
#[derive(Debug, Clone, PartialEq)]
pub struct HybridEscalationGate {
    /// Per-L-layer overrides. Populated from
    /// `[cloud.hybrid] confidence_threshold` + `[cloud.hybrid.per_layer]`
    /// in `elai_cc.toml`.
    pub thresholds: LLayerThresholds,
    /// "Far below" threshold for Opus escalation. Typically
    /// `L_HYBRID_THRESHOLD_DEFAULT / 2` (0.275). Configurable via
    /// `[cloud.hybrid] opus_threshold`.
    pub opus_threshold: ConfidenceScore,
}

impl Default for HybridEscalationGate {
    fn default() -> Self {
        Self {
            thresholds: LLayerThresholds::default(),
            opus_threshold: L_HYBRID_THRESHOLD_DEFAULT / 2.0,
        }
    }
}

impl HybridEscalationGate {
    /// Evaluate the gate for a single inference attempt.
    ///
    /// `score` is the confidence score (1.0 = high). `l_layer` is the
    /// task's L-layer (1..=5). `criticality_high` controls the
    /// EscalateCloud-vs-EscalateOpus split for very-low-confidence
    /// cases; same convention as
    /// [`crate::cost::cost_router::Criticality::High`].
    #[must_use]
    pub fn evaluate(
        &self,
        score: ConfidenceScore,
        l_layer: u8,
        criticality_high: bool,
    ) -> (HybridEscalationVerdict, EscalationReason) {
        let threshold = self.thresholds.threshold_for(l_layer);
        if score >= threshold {
            return (HybridEscalationVerdict::SkipCloud, EscalationReason::Confidence);
        }
        if score < self.opus_threshold && criticality_high {
            return (
                HybridEscalationVerdict::EscalateOpus,
                EscalationReason::Confidence,
            );
        }
        (HybridEscalationVerdict::EscalateCloud, EscalationReason::Confidence)
    }

    /// Convenience: caller already exhausted retries (C-5 retry_cap=1
    /// path). Always escalate cloud; reason = retry_exhausted.
    #[must_use]
    pub fn retry_exhausted_path(&self) -> (HybridEscalationVerdict, EscalationReason) {
        (HybridEscalationVerdict::EscalateCloud, EscalationReason::RetryExhausted)
    }

    /// Convenience: privacy scanner refused cloud dispatch. Returns
    /// SkipCloud + reason = privacy_block so the caller stays local and
    /// emits the existing `RouteBlocked` row with the same reason.
    #[must_use]
    pub fn privacy_block_path(&self) -> (HybridEscalationVerdict, EscalationReason) {
        (HybridEscalationVerdict::SkipCloud, EscalationReason::PrivacyBlock)
    }
}

// ---------------------------------------------------------------------------
// Majority vote (E-20 task 8)
// ---------------------------------------------------------------------------

pub mod majority_vote {
    //! Sample-of-N majority vote on the ROUTING DECISION (not the
    //! answer). Anchored on arXiv:2406.12334 same-prompt different-
    //! result inconsistency and arXiv:2504.01282 Prompt-Reverse
    //! Inconsistency. Cost: N classifier forward passes (cheap on the
    //! local sidecar); benefit: stable routing at temp > 0.

    use super::HybridEscalationVerdict;
    use std::collections::BTreeMap;

    /// Number of classifier samples per routing decision. Per E-20 plan
    /// task 8: "sample-of-3 majority vote ... cost: 3 SLM forward
    /// passes (cheap); benefit: stable routing." Named const (NOT
    /// magic number) per CLAUDE.md anti-stale.
    pub const N_ROUTING_SAMPLES: usize = 3;

    /// Outcome of the majority vote with the per-verdict tally for
    /// dashboard surfaces. `flapping` is `true` when no single verdict
    /// has a strict majority (i.e. all 3 samples differ, or a 1-1-1
    /// tie); the caller treats `flapping` as "escalate to a more
    /// careful path" (typically EscalateCloud).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MajorityVoteOutcome {
        pub winner: HybridEscalationVerdict,
        pub tally: BTreeMap<HybridEscalationVerdict, u8>,
        pub flapping: bool,
    }

    /// Aggregate `N_ROUTING_SAMPLES` verdicts into a single decision.
    /// Returns the most-common verdict. Ties resolve toward
    /// `EscalateCloud` (conservative bias: when in doubt, escalate
    /// rather than skip) and set `flapping = true`.
    #[must_use]
    pub fn route(samples: &[HybridEscalationVerdict]) -> MajorityVoteOutcome {
        let mut tally: BTreeMap<HybridEscalationVerdict, u8> = BTreeMap::new();
        for v in samples {
            *tally.entry(*v).or_insert(0) += 1;
        }
        let half = (samples.len() / 2) as u8;
        let mut winner = HybridEscalationVerdict::EscalateCloud;
        let mut top_count: u8 = 0;
        for (&v, &c) in &tally {
            if c > top_count {
                top_count = c;
                winner = v;
            }
        }
        let strict_majority_count = top_count > half;
        // flapping = no strict majority (tie or 3-way split)
        let flapping = !strict_majority_count;
        if flapping {
            winner = HybridEscalationVerdict::EscalateCloud;
        }
        MajorityVoteOutcome {
            winner,
            tally,
            flapping,
        }
    }
}

// ---------------------------------------------------------------------------
// Semantic reasoning mode toggle (E-20 task 9)
// ---------------------------------------------------------------------------

pub mod semantic_reasoning {
    //! Pre-cloud extended-thinking gate. Anchored on arXiv:2510.08731
    //! "When to Reason: Semantic Router for vLLM" (IBM, NeurIPS 2025):
    //! -47.1% latency / -48.5% tokens / +10.2pp MMLU-Pro by NOT
    //! invoking reasoning mode on prompts that do not benefit. Reuses
    //! the difficulty classifier signal from
    //! [`crate::cost::cost_router::TaskDifficultyClassifier`] +
    //! the task-shape hint from
    //! [`crate::per_role_runtime::TaskShape`].

    use serde::{Deserialize, Serialize};

    /// 2-arm mode toggle. `On` = invoke extended thinking; `Off` = skip.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ReasoningMode {
        On,
        Off,
    }

    impl ReasoningMode {
        /// Snake-case wire label.
        #[must_use]
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::On => "on",
                Self::Off => "off",
            }
        }
    }

    /// 3-arm difficulty bin matching the classifier output. Repeated
    /// here (not imported from `cost_router`) so this module stays
    /// usable from contexts that do not depend on the full classifier.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DifficultyBin {
        Easy,
        Medium,
        Hard,
    }

    impl DifficultyBin {
        /// Snake-case wire label.
        #[must_use]
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Easy => "easy",
                Self::Medium => "medium",
                Self::Hard => "hard",
            }
        }
    }

    /// 3-arm task-shape hint matching
    /// [`crate::per_role_runtime::TaskShape`]. Repeated here for the
    /// same isolation reason as [`DifficultyBin`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum TaskShapeHint {
        Execution,
        Architecture,
        Math,
    }

    impl TaskShapeHint {
        /// Snake-case wire label.
        #[must_use]
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Execution => "execution",
                Self::Architecture => "architecture",
                Self::Math => "math",
            }
        }
    }

    /// Choose whether to invoke extended thinking on a cloud dispatch.
    ///
    /// Heuristic (per arXiv:2510.08731 section 4):
    ///   * Easy execution -> Off (cheap lookups gain nothing from
    ///     extended thinking)
    ///   * Easy architecture -> Off
    ///   * Easy math -> Off (arithmetic; reasoning would be wasted)
    ///   * Medium math OR Medium architecture -> On (medium-complexity
    ///     reasoning benefits)
    ///   * Medium execution -> Off (typical code edit; speed > deep
    ///     thinking)
    ///   * Hard anything -> On (deep multi-step reasoning required)
    #[must_use]
    pub fn choose_reasoning_mode(
        difficulty: DifficultyBin,
        task_shape: TaskShapeHint,
    ) -> ReasoningMode {
        match (difficulty, task_shape) {
            (DifficultyBin::Easy, _) => ReasoningMode::Off,
            (DifficultyBin::Medium, TaskShapeHint::Execution) => ReasoningMode::Off,
            (DifficultyBin::Medium, TaskShapeHint::Architecture) => ReasoningMode::On,
            (DifficultyBin::Medium, TaskShapeHint::Math) => ReasoningMode::On,
            (DifficultyBin::Hard, _) => ReasoningMode::On,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_gate_default_off() {
        // Treat as a smoke check: helper compiles + returns a bool.
        let _ = local_cloud_routing_enabled();
    }

    #[test]
    fn confidence_score_from_empty_distribution_is_zero() {
        let score = confidence::from_first_token_distribution(&[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn confidence_score_from_sharp_distribution_is_high() {
        // Near-one-hot: most mass on a single token -> high confidence.
        let probs = [0.97, 0.01, 0.01, 0.01];
        let score = confidence::from_first_token_distribution(&probs);
        assert!(
            score > 0.85,
            "sharp distribution must be high confidence (got {score})"
        );
    }

    #[test]
    fn confidence_score_from_uniform_distribution_is_zero() {
        // Maximum entropy -> minimum confidence.
        let probs = [0.25, 0.25, 0.25, 0.25];
        let score = confidence::from_first_token_distribution(&probs);
        assert!(
            score < 0.05,
            "uniform distribution must be ~0 confidence (got {score})"
        );
    }

    #[test]
    fn confidence_from_mean_logprob_at_pivot() {
        // mean_logprob = -2.3 is the sigmoid centre -> ~0.5 confidence.
        let score = confidence::from_mean_logprob(-2.3);
        assert!(
            (score - 0.5).abs() < 0.05,
            "pivot must produce ~0.5 confidence (got {score})"
        );
    }

    #[test]
    fn compute_score_prefers_distribution_when_present() {
        let probs = [0.97, 0.01, 0.01, 0.01];
        let from_dist = confidence::compute_score(Some(&probs), -100.0);
        let from_logprob = confidence::compute_score(None, -100.0);
        // Distribution gives high confidence; logprob fallback at -100
        // gives near-zero confidence; assert they diverge as expected.
        assert!(from_dist > 0.5);
        assert!(from_logprob < 0.5);
    }

    #[test]
    fn l_layer_threshold_default_for_unset_layer() {
        let thresholds = LLayerThresholds::default();
        assert_eq!(thresholds.threshold_for(3), L_HYBRID_THRESHOLD_DEFAULT);
    }

    #[test]
    fn l_layer_threshold_override_applies() {
        let mut thresholds = LLayerThresholds::default();
        thresholds.per_layer[2] = Some(0.7);
        assert_eq!(thresholds.threshold_for(3), 0.7);
        assert_eq!(thresholds.threshold_for(1), L_HYBRID_THRESHOLD_DEFAULT);
    }

    #[test]
    fn l_layer_threshold_out_of_range_falls_back() {
        let thresholds = LLayerThresholds::default();
        assert_eq!(thresholds.threshold_for(0), L_HYBRID_THRESHOLD_DEFAULT);
        assert_eq!(thresholds.threshold_for(99), L_HYBRID_THRESHOLD_DEFAULT);
    }

    #[test]
    fn hybrid_gate_skips_cloud_above_threshold() {
        let gate = HybridEscalationGate::default();
        let (verdict, reason) = gate.evaluate(0.9, 3, false);
        assert_eq!(verdict, HybridEscalationVerdict::SkipCloud);
        assert_eq!(reason, EscalationReason::Confidence);
    }

    #[test]
    fn hybrid_gate_escalates_cloud_below_threshold() {
        let gate = HybridEscalationGate::default();
        let (verdict, _) = gate.evaluate(0.4, 3, false);
        assert_eq!(verdict, HybridEscalationVerdict::EscalateCloud);
    }

    #[test]
    fn hybrid_gate_escalates_opus_when_far_below_and_critical() {
        let gate = HybridEscalationGate::default();
        // opus_threshold = 0.275; score 0.1 is far below.
        let (verdict, reason) = gate.evaluate(0.1, 5, true);
        assert_eq!(verdict, HybridEscalationVerdict::EscalateOpus);
        assert_eq!(reason, EscalationReason::Confidence);
    }

    #[test]
    fn hybrid_gate_avoids_opus_when_critical_but_threshold_above() {
        let gate = HybridEscalationGate::default();
        let (verdict, _) = gate.evaluate(0.4, 5, true);
        // 0.4 below default 0.55 but above opus 0.275 -> EscalateCloud
        assert_eq!(verdict, HybridEscalationVerdict::EscalateCloud);
    }

    #[test]
    fn retry_exhausted_path_is_escalate_cloud() {
        let gate = HybridEscalationGate::default();
        let (verdict, reason) = gate.retry_exhausted_path();
        assert_eq!(verdict, HybridEscalationVerdict::EscalateCloud);
        assert_eq!(reason, EscalationReason::RetryExhausted);
    }

    #[test]
    fn privacy_block_path_stays_local() {
        let gate = HybridEscalationGate::default();
        let (verdict, reason) = gate.privacy_block_path();
        assert_eq!(verdict, HybridEscalationVerdict::SkipCloud);
        assert_eq!(reason, EscalationReason::PrivacyBlock);
    }

    #[test]
    fn majority_vote_constant_is_three() {
        assert_eq!(majority_vote::N_ROUTING_SAMPLES, 3);
    }

    #[test]
    fn majority_vote_strict_majority_wins() {
        use HybridEscalationVerdict::*;
        let samples = [SkipCloud, SkipCloud, EscalateCloud];
        let out = majority_vote::route(&samples);
        assert_eq!(out.winner, SkipCloud);
        assert!(!out.flapping);
    }

    #[test]
    fn majority_vote_three_way_tie_is_flapping_and_conservative() {
        use HybridEscalationVerdict::*;
        let samples = [SkipCloud, EscalateCloud, EscalateOpus];
        let out = majority_vote::route(&samples);
        assert!(out.flapping);
        assert_eq!(out.winner, EscalateCloud);
    }

    #[test]
    fn majority_vote_unanimous() {
        use HybridEscalationVerdict::*;
        let samples = [EscalateOpus, EscalateOpus, EscalateOpus];
        let out = majority_vote::route(&samples);
        assert_eq!(out.winner, EscalateOpus);
        assert!(!out.flapping);
    }

    #[test]
    fn semantic_reasoning_easy_off() {
        use semantic_reasoning::*;
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Easy, TaskShapeHint::Execution),
            ReasoningMode::Off
        );
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Easy, TaskShapeHint::Math),
            ReasoningMode::Off
        );
    }

    #[test]
    fn semantic_reasoning_medium_execution_off() {
        use semantic_reasoning::*;
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Medium, TaskShapeHint::Execution),
            ReasoningMode::Off
        );
    }

    #[test]
    fn semantic_reasoning_medium_architecture_on() {
        use semantic_reasoning::*;
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Medium, TaskShapeHint::Architecture),
            ReasoningMode::On
        );
    }

    #[test]
    fn semantic_reasoning_hard_always_on() {
        use semantic_reasoning::*;
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Hard, TaskShapeHint::Execution),
            ReasoningMode::On
        );
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Hard, TaskShapeHint::Architecture),
            ReasoningMode::On
        );
    }

    #[test]
    fn verdict_labels_stable() {
        assert_eq!(HybridEscalationVerdict::SkipCloud.as_str(), "skip_cloud");
        assert_eq!(
            HybridEscalationVerdict::EscalateCloud.as_str(),
            "escalate_cloud"
        );
        assert_eq!(HybridEscalationVerdict::EscalateOpus.as_str(), "escalate_opus");
    }

    #[test]
    fn reason_labels_stable() {
        assert_eq!(EscalationReason::Confidence.as_str(), "confidence");
        assert_eq!(EscalationReason::RetryExhausted.as_str(), "retry_exhausted");
        assert_eq!(EscalationReason::PrivacyBlock.as_str(), "privacy_block");
    }
}
