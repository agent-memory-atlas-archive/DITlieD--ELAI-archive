//! D-3: VerificationResult aggregator + multi-verifier consensus rule.
//!
//! Composes heterogeneous verifier signals (PPM score, intrinsic logprob /
//! entropy / KL signals, deterministic checks, future verifier kinds) into a
//! single [`AggregateVerdict`] consumed by the pre-promotion chain gate.
//!
//! Design grounded in V-STaR (arxiv:2402.06457, T1) — improving the
//! generator AND the verifier on different signals; single-verifier sign-off
//! is the documented failure mode. PRMBench (arxiv:2501.03124, T1) §4.2
//! reports that 25 evaluated PRMs / LLM-judges score worst on Deception
//! Resistance and Circular Logic; consensus across SignalKind sources is the
//! structural defense.
//!
//! This module is the *aggregator only* — the SSB producer + consensus
//! call-site live in `agent_pool.rs::run_consensus_pass` (see D-3 plan).
//! Wiring discipline: this file consumes [`crate::ppm_client::PpmScore`]
//! (D-2) but never invokes PpmClient itself — that is the consensus pass's
//! responsibility. Decoupling lets unit tests construct synthetic
//! VerificationResult values without spinning up a sidecar.

use serde::{Deserialize, Serialize};

use crate::ppm_client::PpmScore;

// ---------------------------------------------------------------------------
// VerifierKind: closed enum of signal-source families
// ---------------------------------------------------------------------------

/// One verifier family — the consensus rule treats each kind as a single
/// signal source (multiple verifiers of the same kind do NOT add weight to a
/// `VerifierKindHeterogeneous` rule).
///
/// L3-locked: agent self-modification must not introduce new kinds; doing so
/// would silently re-shape the consensus rule. New kinds land via plan-level
/// schema bumps.
///
/// E-53 plan-level extension (SCHEMA 139 -> 140): adds [`Self::Z3SmtCheck`]
/// as the 5th variant per the lock's documented escape clause "New kinds
/// land via plan-level schema bumps." See
/// workflow/reasoning/E-53-decisions.md for the L3-unlock rationale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    /// PPM (Process Preference Model) score from the second sidecar (D-2).
    Ppm,
    /// Intrinsic signals harvested from the policy model's own forward pass
    /// (logprob mean / variance, entropy, KL-to-base).
    Intrinsic,
    /// Deterministic-checker outcome (compile / test / lint / schema /
    /// registry check) treated as a binary pass/fail signal.
    Deterministic,
    /// Cross-context agreement — same claim run in two independent context
    /// windows; agreement threshold producer-side.
    CrossContext,
    /// E-53 plan-level extension (SCHEMA 139 -> 140): Z3 SMT-checker
    /// outcome. Distinct from `Deterministic` because SMT outcomes carry
    /// three abstain states (Unknown / Timeout / Inapplicable) that MUST
    /// NOT contribute to a `VerifierKindHeterogeneous` consensus vote.
    /// The aggregator treats Z3SmtCheck as a single source family per the
    /// L3-lock comment; the Z3 binary is RUNPOD-PENDING on the local dev
    /// host (anti-slopsquatting: no z3 crate dep added in this ship; the
    /// runner shells out at the boundary and surfaces Inapplicable when
    /// the binary is absent). See workflow/reasoning/E-53-decisions.md.
    Z3SmtCheck,
}

impl VerifierKind {
    /// Snake-case wire label used in SSB rows + dashboard rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ppm => "ppm",
            Self::Intrinsic => "intrinsic",
            Self::Deterministic => "deterministic",
            Self::CrossContext => "cross_context",
            Self::Z3SmtCheck => "z3_smt_check",
        }
    }
}

// ---------------------------------------------------------------------------
// IntrinsicSignals: scalar bundle from one forward pass
// ---------------------------------------------------------------------------

/// Scalar intrinsic signals captured from one inference pass. All fields are
/// `f32` to match the sidecar wire format; widening to f64 for SSB happens at
/// the producer site, not here.
///
/// Field semantics (per Phase 4G groundedness check + V-STaR §3.1):
/// - `logprob_mean`: mean per-token log-probability of the generated chain.
///   Higher = the model was more confident; typical range `[-5.0, 0.0]`.
/// - `logprob_var`: variance across tokens. Low variance + high mean = stable
///   answer; high variance = chain has uncertain spots.
/// - `entropy`: mean predictive entropy across positions. Typical range
///   `[0.0, ~10.0]` (nats; depends on vocab size).
/// - `step_count`: number of reasoning steps the chain emitted.
/// - `kl_to_base`: KL divergence vs. the policy-mode baseline (steering /
///   contrast measurement). `0.0` when steering is disabled.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IntrinsicSignals {
    pub logprob_mean: f32,
    pub logprob_var: f32,
    pub entropy: f32,
    pub step_count: u32,
    pub kl_to_base: f32,
}

impl IntrinsicSignals {
    /// Default minimum mean log-probability for an Intrinsic pass verdict.
    /// `-3.0` corresponds to roughly "confident" generations on Qwen2.5-Coder
    /// per workflow/papers/vstar-2402.06457-interface.md §3.1.
    pub const DEFAULT_LOGPROB_FLOOR: f32 = -3.0;
    /// Default maximum mean entropy. Above this the chain is "uncertain"
    /// and the Intrinsic signal does NOT contribute to consensus.
    pub const DEFAULT_ENTROPY_CEIL: f32 = 3.5;

    /// Returns `true` if the intrinsic signal passes the default floor /
    /// ceiling combo. Closed-form, no sidecar call.
    #[must_use]
    pub fn passes_default(&self) -> bool {
        self.logprob_mean >= Self::DEFAULT_LOGPROB_FLOOR && self.entropy <= Self::DEFAULT_ENTROPY_CEIL
    }
}

// ---------------------------------------------------------------------------
// VerificationResult: one verifier's typed output
// ---------------------------------------------------------------------------

/// One verifier's typed output, keyed by `(candidate_id, verifier_id)`. The
/// aggregator never mutates these — they are the immutable inputs to
/// [`aggregate`].
///
/// `score` is the verifier-native score on `[0.0, 1.0]` when applicable
/// (PPM, Intrinsic). For Deterministic / CrossContext verifiers it carries
/// `1.0` on pass and `0.0` on fail so weighted rules can use a uniform
/// scoring axis.
///
/// `signal_payload` carries intrinsic-signal scalars when `kind ==
/// Intrinsic`; for other kinds the payload is the zero-value (the field is
/// present unconditionally so SSB rows do not need shape-discrimination at
/// read time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    pub candidate_id: String,
    pub verifier_id: String,
    pub kind: VerifierKind,
    /// Per-verifier raw score on `[0.0, 1.0]`. `None` when the verifier did
    /// not produce a numeric score (e.g. unavailable / inconclusive).
    pub score: Option<f32>,
    /// Whether this verifier individually says the candidate is acceptable.
    /// Closed-form: derived from `score` + verifier-kind-specific floor at
    /// construction time.
    pub passed: bool,
    pub signal_payload: IntrinsicSignals,
    /// Unix-millis timestamp when this verifier produced the result.
    pub ts_ms: i64,
}

impl VerificationResult {
    /// Construct a verification result from a [`PpmScore`] (D-2 output).
    /// `gate_passed` from the PPM client becomes `passed` here.
    #[must_use]
    pub fn from_ppm(candidate_id: impl Into<String>, score: &PpmScore, ts_ms: i64) -> Self {
        Self {
            candidate_id: candidate_id.into(),
            verifier_id: score.verifier_id.clone(),
            kind: VerifierKind::Ppm,
            score: Some(score.score_mean),
            passed: score.gate_passed,
            signal_payload: IntrinsicSignals {
                logprob_mean: 0.0,
                logprob_var: score.score_var,
                entropy: 0.0,
                step_count: score.num_steps,
                kl_to_base: 0.0,
            },
            ts_ms,
        }
    }

    /// Construct a verification result from raw [`IntrinsicSignals`]. Passes
    /// when the signals clear [`IntrinsicSignals::passes_default`].
    #[must_use]
    pub fn from_intrinsic(
        candidate_id: impl Into<String>,
        verifier_id: impl Into<String>,
        signals: IntrinsicSignals,
        ts_ms: i64,
    ) -> Self {
        let passed = signals.passes_default();
        // Normalise the intrinsic-pass score to [0,1] linearly: closer to
        // DEFAULT_LOGPROB_FLOOR = closer to 0; logprob_mean >= 0 (saturating)
        // = 1. Keeps weighted rules monotone.
        let span = -IntrinsicSignals::DEFAULT_LOGPROB_FLOOR;
        let raw = (signals.logprob_mean - IntrinsicSignals::DEFAULT_LOGPROB_FLOOR) / span;
        let score = raw.clamp(0.0, 1.0);
        Self {
            candidate_id: candidate_id.into(),
            verifier_id: verifier_id.into(),
            kind: VerifierKind::Intrinsic,
            score: Some(score),
            passed,
            signal_payload: signals,
            ts_ms,
        }
    }

    /// Construct a verification result from a deterministic-check outcome.
    /// Carries `score = 1.0` on pass / `0.0` on fail so weighted rules can
    /// blend it with PPM / Intrinsic uniformly.
    #[must_use]
    pub fn from_deterministic(
        candidate_id: impl Into<String>,
        verifier_id: impl Into<String>,
        passed: bool,
        ts_ms: i64,
    ) -> Self {
        Self {
            candidate_id: candidate_id.into(),
            verifier_id: verifier_id.into(),
            kind: VerifierKind::Deterministic,
            score: Some(if passed { 1.0 } else { 0.0 }),
            passed,
            signal_payload: IntrinsicSignals {
                logprob_mean: 0.0,
                logprob_var: 0.0,
                entropy: 0.0,
                step_count: 0,
                kl_to_base: 0.0,
            },
            ts_ms,
        }
    }
}

// ---------------------------------------------------------------------------
// ConsensusRule: closed enum of aggregation strategies
// ---------------------------------------------------------------------------

/// Closed-enum aggregation strategy.
///
/// Variants:
/// - [`ConsensusRule::AllPass`]: every result in the input slice MUST pass.
/// - [`ConsensusRule::KofN`]: at least `k` of `n` results must pass; `n` is
///   echoed for clarity (caller-side sanity check vs. slice length).
/// - [`ConsensusRule::WeightedScoreThreshold`]: sum of `(verifier_kind ->
///   weight) * score` must clear `threshold`. Each `kind` contributes at most
///   once (deduped on `kind`).
/// - [`ConsensusRule::VerifierKindHeterogeneous`]: every `kind` in
///   `required_kinds` must be present AND its result must pass. Defends
///   against "5 PPM verifiers agree" trivial consensus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConsensusRule {
    AllPass,
    KofN {
        k: u32,
        n: u32,
    },
    WeightedScoreThreshold {
        weights: Vec<(VerifierKind, f32)>,
        threshold: f32,
    },
    VerifierKindHeterogeneous {
        required_kinds: Vec<VerifierKind>,
    },
}

impl ConsensusRule {
    /// Default rule per D-3 plan §3.3 (companion §3.3 lines 242-256):
    /// `KofN { k: 2, n: 3 }` — at least 2 of 3 verifiers must agree before
    /// promotion. Tunable per template_kind in D-9 L3 config.
    #[must_use]
    pub fn default_two_of_three() -> Self {
        ConsensusRule::KofN { k: 2, n: 3 }
    }

    /// Stable serialization id used in SSB rows + Tauri payloads.
    #[must_use]
    pub fn rule_id(&self) -> &'static str {
        match self {
            Self::AllPass => "all_pass",
            Self::KofN { .. } => "k_of_n",
            Self::WeightedScoreThreshold { .. } => "weighted_score_threshold",
            Self::VerifierKindHeterogeneous { .. } => "verifier_kind_heterogeneous",
        }
    }
}

// ---------------------------------------------------------------------------
// AggregateVerdict
// ---------------------------------------------------------------------------

/// Aggregate verdict — the output of [`aggregate`].
///
/// `Pass` = consensus rule satisfied. `Fail` = rule violated; reasons are
/// snake_case strings so dashboard + SSB consumers don't need to re-parse
/// the rule. `Inconclusive` = required signal was None / structurally
/// absent; caller defers promotion rather than rejecting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum AggregateVerdict {
    Pass,
    Fail { reasons: Vec<String> },
    Inconclusive { reason: String },
}

impl AggregateVerdict {
    /// True when the verdict is `Pass`. Used by the pre-promotion gate.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Snake-case verdict label for SSB rows.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail { .. } => "fail",
            Self::Inconclusive { .. } => "inconclusive",
        }
    }
}

// ---------------------------------------------------------------------------
// aggregate: the consensus computation
// ---------------------------------------------------------------------------

/// Apply `rule` to `results` and return the aggregate verdict.
///
/// Empty `results` always returns `Inconclusive` — the caller decided to ask
/// for consensus but produced no signals; the rule cannot be applied.
///
/// Closed-form, no I/O, no allocations beyond the failure-reason strings.
/// Deterministic given identical inputs (no clock / RNG).
#[must_use]
pub fn aggregate(rule: &ConsensusRule, results: &[VerificationResult]) -> AggregateVerdict {
    if results.is_empty() {
        return AggregateVerdict::Inconclusive {
            reason: "no_verifiers".into(),
        };
    }

    match rule {
        ConsensusRule::AllPass => {
            let failing: Vec<String> = results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| r.verifier_id.clone())
                .collect();
            if failing.is_empty() {
                AggregateVerdict::Pass
            } else {
                AggregateVerdict::Fail { reasons: failing }
            }
        }
        ConsensusRule::KofN { k, n } => {
            let n_actual = results.len() as u32;
            if n_actual < *n {
                return AggregateVerdict::Inconclusive {
                    reason: format!("k_of_n_short_input_{n_actual}_of_{n}"),
                };
            }
            let pass_count = results.iter().filter(|r| r.passed).count() as u32;
            if pass_count >= *k {
                AggregateVerdict::Pass
            } else {
                let failing: Vec<String> = results
                    .iter()
                    .filter(|r| !r.passed)
                    .map(|r| r.verifier_id.clone())
                    .collect();
                AggregateVerdict::Fail { reasons: failing }
            }
        }
        ConsensusRule::WeightedScoreThreshold { weights, threshold } => {
            // Dedupe by kind: each kind contributes at most one weighted score.
            // Pick the highest-scoring result per kind (defends against gaming
            // via repeated low-score verifiers of the same kind).
            let mut total = 0.0f32;
            let mut considered = 0u32;
            for (kind, weight) in weights {
                let best: Option<f32> = results
                    .iter()
                    .filter(|r| r.kind == *kind)
                    .filter_map(|r| r.score)
                    .fold(None, |acc, s| match acc {
                        None => Some(s),
                        Some(prev) => Some(prev.max(s)),
                    });
                if let Some(score) = best {
                    total += score * weight;
                    considered += 1;
                }
            }
            if considered == 0 {
                return AggregateVerdict::Inconclusive {
                    reason: "weighted_no_kinds_present".into(),
                };
            }
            if total >= *threshold {
                AggregateVerdict::Pass
            } else {
                AggregateVerdict::Fail {
                    reasons: vec![format!("weighted_total_{total:.3}_below_{threshold:.3}")],
                }
            }
        }
        ConsensusRule::VerifierKindHeterogeneous { required_kinds } => {
            let mut missing: Vec<String> = Vec::new();
            let mut failing: Vec<String> = Vec::new();
            for kind in required_kinds {
                // For each required kind, at least one result of that kind
                // must be present AND pass.
                let any_for_kind: Vec<&VerificationResult> =
                    results.iter().filter(|r| r.kind == *kind).collect();
                if any_for_kind.is_empty() {
                    missing.push(kind.as_str().into());
                    continue;
                }
                if !any_for_kind.iter().any(|r| r.passed) {
                    let ids: Vec<String> =
                        any_for_kind.iter().map(|r| r.verifier_id.clone()).collect();
                    failing.push(format!("{}={}", kind.as_str(), ids.join(",")));
                }
            }
            if !missing.is_empty() {
                return AggregateVerdict::Inconclusive {
                    reason: format!("missing_kinds={}", missing.join(",")),
                };
            }
            if failing.is_empty() {
                AggregateVerdict::Pass
            } else {
                AggregateVerdict::Fail { reasons: failing }
            }
        }
    }
}

impl VerificationResult {
    /// Sugar: forward to [`aggregate`]. Lives on the type for ergonomic
    /// call-site reading (`VerificationResult::aggregate(&rule, &results)`).
    #[must_use]
    pub fn aggregate(rule: &ConsensusRule, results: &[VerificationResult]) -> AggregateVerdict {
        aggregate(rule, results)
    }
}

// ---------------------------------------------------------------------------
// Gate helper: pre-promotion gate consumer
// ---------------------------------------------------------------------------

/// Composite gate: returns `Ok(())` only when both the [`crate::archive_chain::pre_promotion_chain_gate`]
/// passes AND `verdict.is_pass()`. The D-1 gate is the chain-consistency
/// half; this layer adds the consensus half. Production callers use this
/// instead of the raw chain gate so a clean chain alone cannot promote a
/// candidate that failed consensus.
///
/// Implemented as a free function so the task_runner / agent_pool callers do
/// not need to thread the chain through the aggregator.
pub fn gate_with_consensus(
    chain: &crate::archive_chain::ArchiveChain,
    verdict: &AggregateVerdict,
) -> Result<(), GateRejection> {
    crate::archive_chain::pre_promotion_chain_gate(chain)
        .map_err(|e| GateRejection::ChainBroken(e.to_string()))?;
    if verdict.is_pass() {
        Ok(())
    } else {
        Err(GateRejection::ConsensusFailed(verdict.clone()))
    }
}

/// Reason a composite gate rejected a promotion. Exposed as its own error
/// type so callers can branch on the rejection family without re-parsing
/// strings.
#[derive(Debug, Clone, PartialEq)]
pub enum GateRejection {
    ChainBroken(String),
    ConsensusFailed(AggregateVerdict),
}

impl std::fmt::Display for GateRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainBroken(s) => write!(f, "chain_broken: {s}"),
            Self::ConsensusFailed(v) => write!(f, "consensus_failed: {}", v.as_str()),
        }
    }
}

impl std::error::Error for GateRejection {}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_result(kind: VerifierKind, verifier_id: &str, passed: bool, score: f32) -> VerificationResult {
        VerificationResult {
            candidate_id: "cand-1".into(),
            verifier_id: verifier_id.into(),
            kind,
            score: Some(score),
            passed,
            signal_payload: IntrinsicSignals {
                logprob_mean: 0.0,
                logprob_var: 0.0,
                entropy: 0.0,
                step_count: 0,
                kl_to_base: 0.0,
            },
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn empty_results_is_inconclusive() {
        let v = aggregate(&ConsensusRule::AllPass, &[]);
        match v {
            AggregateVerdict::Inconclusive { reason } => assert_eq!(reason, "no_verifiers"),
            other => panic!("expected Inconclusive, got {other:?}"),
        }
    }

    #[test]
    fn all_pass_succeeds_when_all_pass() {
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Intrinsic, "i1", true, 0.8),
        ];
        assert_eq!(aggregate(&ConsensusRule::AllPass, &r), AggregateVerdict::Pass);
    }

    #[test]
    fn all_pass_fails_on_any_failure() {
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Intrinsic, "i1", false, 0.2),
        ];
        match aggregate(&ConsensusRule::AllPass, &r) {
            AggregateVerdict::Fail { reasons } => {
                assert_eq!(reasons, vec!["i1".to_string()]);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn k_of_n_passes_with_threshold_met() {
        let rule = ConsensusRule::KofN { k: 2, n: 3 };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Intrinsic, "i1", true, 0.8),
            mk_result(VerifierKind::Deterministic, "d1", false, 0.0),
        ];
        assert_eq!(aggregate(&rule, &r), AggregateVerdict::Pass);
    }

    #[test]
    fn k_of_n_fails_when_threshold_missed() {
        let rule = ConsensusRule::KofN { k: 2, n: 3 };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Intrinsic, "i1", false, 0.4),
            mk_result(VerifierKind::Deterministic, "d1", false, 0.0),
        ];
        match aggregate(&rule, &r) {
            AggregateVerdict::Fail { reasons } => {
                assert_eq!(reasons.len(), 2);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn k_of_n_inconclusive_on_short_input() {
        let rule = ConsensusRule::KofN { k: 2, n: 3 };
        let r = vec![mk_result(VerifierKind::Ppm, "p1", true, 0.9)];
        match aggregate(&rule, &r) {
            AggregateVerdict::Inconclusive { reason } => {
                assert!(reason.starts_with("k_of_n_short_input"));
            }
            other => panic!("expected Inconclusive, got {other:?}"),
        }
    }

    #[test]
    fn weighted_threshold_passes_above_threshold() {
        let rule = ConsensusRule::WeightedScoreThreshold {
            weights: vec![(VerifierKind::Ppm, 0.6), (VerifierKind::Intrinsic, 0.4)],
            threshold: 0.7,
        };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),       // 0.54
            mk_result(VerifierKind::Intrinsic, "i1", true, 0.8), // 0.32
        ];
        // total = 0.86 >= 0.7
        assert_eq!(aggregate(&rule, &r), AggregateVerdict::Pass);
    }

    #[test]
    fn weighted_threshold_fails_below_threshold() {
        let rule = ConsensusRule::WeightedScoreThreshold {
            weights: vec![(VerifierKind::Ppm, 0.5), (VerifierKind::Intrinsic, 0.5)],
            threshold: 0.7,
        };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.4),       // 0.20
            mk_result(VerifierKind::Intrinsic, "i1", true, 0.5), // 0.25
        ];
        // total = 0.45 < 0.7
        match aggregate(&rule, &r) {
            AggregateVerdict::Fail { reasons } => {
                assert!(reasons[0].contains("weighted_total"));
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn weighted_threshold_picks_best_per_kind() {
        let rule = ConsensusRule::WeightedScoreThreshold {
            weights: vec![(VerifierKind::Ppm, 1.0)],
            threshold: 0.8,
        };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", false, 0.2),
            mk_result(VerifierKind::Ppm, "p2", true, 0.9), // best Ppm score wins
        ];
        assert_eq!(aggregate(&rule, &r), AggregateVerdict::Pass);
    }

    #[test]
    fn heterogeneous_passes_when_all_kinds_present_and_pass() {
        let rule = ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![VerifierKind::Ppm, VerifierKind::Intrinsic, VerifierKind::Deterministic],
        };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Intrinsic, "i1", true, 0.8),
            mk_result(VerifierKind::Deterministic, "d1", true, 1.0),
        ];
        assert_eq!(aggregate(&rule, &r), AggregateVerdict::Pass);
    }

    #[test]
    fn heterogeneous_inconclusive_on_missing_kind() {
        let rule = ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![VerifierKind::Ppm, VerifierKind::Deterministic],
        };
        let r = vec![mk_result(VerifierKind::Ppm, "p1", true, 0.9)];
        match aggregate(&rule, &r) {
            AggregateVerdict::Inconclusive { reason } => {
                assert!(reason.contains("deterministic"));
            }
            other => panic!("expected Inconclusive, got {other:?}"),
        }
    }

    #[test]
    fn heterogeneous_fails_when_kind_present_but_fails() {
        let rule = ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![VerifierKind::Ppm, VerifierKind::Deterministic],
        };
        let r = vec![
            mk_result(VerifierKind::Ppm, "p1", true, 0.9),
            mk_result(VerifierKind::Deterministic, "d1", false, 0.0),
        ];
        match aggregate(&rule, &r) {
            AggregateVerdict::Fail { reasons } => {
                assert!(reasons.iter().any(|r| r.contains("deterministic")));
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn intrinsic_passes_default_floor() {
        let signals = IntrinsicSignals {
            logprob_mean: -1.0,
            logprob_var: 0.1,
            entropy: 2.0,
            step_count: 3,
            kl_to_base: 0.0,
        };
        assert!(signals.passes_default());
    }

    #[test]
    fn intrinsic_fails_low_logprob() {
        let signals = IntrinsicSignals {
            logprob_mean: -5.0,
            logprob_var: 0.1,
            entropy: 2.0,
            step_count: 3,
            kl_to_base: 0.0,
        };
        assert!(!signals.passes_default());
    }

    #[test]
    fn intrinsic_fails_high_entropy() {
        let signals = IntrinsicSignals {
            logprob_mean: -1.0,
            logprob_var: 0.1,
            entropy: 5.0,
            step_count: 3,
            kl_to_base: 0.0,
        };
        assert!(!signals.passes_default());
    }

    #[test]
    fn rule_id_round_trip() {
        assert_eq!(ConsensusRule::AllPass.rule_id(), "all_pass");
        assert_eq!(ConsensusRule::KofN { k: 2, n: 3 }.rule_id(), "k_of_n");
        assert_eq!(
            ConsensusRule::WeightedScoreThreshold {
                weights: vec![],
                threshold: 0.0
            }
            .rule_id(),
            "weighted_score_threshold"
        );
        assert_eq!(
            ConsensusRule::VerifierKindHeterogeneous {
                required_kinds: vec![]
            }
            .rule_id(),
            "verifier_kind_heterogeneous"
        );
    }

    #[test]
    fn verifier_kind_serde_round_trip() {
        for k in [
            VerifierKind::Ppm,
            VerifierKind::Intrinsic,
            VerifierKind::Deterministic,
            VerifierKind::CrossContext,
            VerifierKind::Z3SmtCheck,
        ] {
            let s = serde_json::to_string(&k).expect("serialize");
            let back: VerifierKind = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(k, back);
        }
    }

    #[test]
    fn aggregate_assoc_fn_matches_free_fn() {
        let r = vec![mk_result(VerifierKind::Ppm, "p1", true, 0.9)];
        let rule = ConsensusRule::AllPass;
        assert_eq!(aggregate(&rule, &r), VerificationResult::aggregate(&rule, &r));
    }
}
