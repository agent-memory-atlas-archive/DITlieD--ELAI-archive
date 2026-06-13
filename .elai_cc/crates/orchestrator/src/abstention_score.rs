// H-39 — AbstentionAwareScore: per-item scorer + aggregate.
//
// Closes the self-improvement loop misalignment surfaced by the
// Hallucination-Stations research thread: D-16's RefusalDecision::Abstain fires
// at inference but the reward the darwin_admit_loop climbs is abstention-blind
// (binary pass/fail). Under a binary grader, abstaining is never optimal
// (OpenAI arXiv:2509.04664 Observation 1); H-39 fixes the reward so the loop
// credits a correct abstention and penalises a confident-wrong answer strictly
// more than one.
//
// Grounding: arXiv:2509.04664 Table 2 — confidence-target grading protocol
//   penalty = t/(1-t) for target t.  t=0.75 -> 3x penalty (scalar 3.0; the
//   paper's '2x harder to game' is a different quantity), t=0.5 -> 1x, t=0.9 -> 9x.
//   ELAI default t=0.75 (LOCKED; the loop may NOT tune it towards 0, which
//   restores the guessing incentive).
//
// GD-1 compliance: the abstain bit is READ from the shipped RefusalDecision
// (conformal_calibration.rs:262). This module does NOT re-derive a new abstain
// signal — the caller passes a RefusalDecision produced by decide() /
// dispatch_conformal_refusal().
//
// Policy: no .unwrap()/.expect() in non-test code; no wall-clock; no I/O.
// Pure deterministic (no RNG). blake3 only where hashing is needed (none here).

use crate::calibration::RefusalDecision;

// ---------------------------------------------------------------------------
// LOCKED confidence target (anti-tuning freeze: TD-2 style).
// A self-mod loop that tunes this value toward 0 restores the guessing
// incentive. It is NOT configurable at runtime or via env.
// ---------------------------------------------------------------------------

/// Default confidence target t.
///
/// Locked at 0.75 (penalty scalar 3.0 -- abstaining is 3x cheaper than
/// confident-wrong), per arXiv:2509.04664 Table 2 middle recommendation.
/// HARD-LOCKED: the self-mod loop MUST NOT tune this value toward 0.
pub const DEFAULT_CONFIDENCE_TARGET: f64 = 0.75;

// ---------------------------------------------------------------------------
// Per-item outcome triple
// ---------------------------------------------------------------------------

/// The three-way outcome for one evaluation item.
///
/// `answered` = the model produced a substantive answer (not RefusalDecision::Abstain).
/// `correct`  = the eval harness judged the answer correct (pass verdict).
/// `refusal`  = the RefusalDecision produced by decide()/dispatch_conformal_refusal()
///              for this item — reused as-is, never re-derived (GD-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutcomeTriple {
    /// True when the model answered (i.e. RefusalDecision::Answer{..}).
    pub answered: bool,
    /// True when the harness verdict was PASS (only meaningful when answered=true).
    pub correct: bool,
    /// The RefusalDecision produced by D-16 conformal calibration.
    pub refusal: RefusalDecision,
}

impl OutcomeTriple {
    /// Construct from a RefusalDecision + harness pass/fail verdict.
    ///
    /// `pass` is ignored when the decision is Abstain (answered=false).
    #[must_use]
    pub fn new(refusal: RefusalDecision, pass: bool) -> Self {
        let answered = refusal != RefusalDecision::Abstain;
        OutcomeTriple {
            answered,
            correct: answered && pass,
            refusal,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-item score
// ---------------------------------------------------------------------------

/// Score one evaluation item using the confidence-target grading protocol.
///
/// Scoring rules (per arXiv:2509.04664):
///   - correct & answered  ->  +1.0
///   - abstained            ->   0.0
///   - answered & wrong     ->  -t/(1-t)   [penalty, always strictly < 0]
///
/// `t` must be the LOCKED `DEFAULT_CONFIDENCE_TARGET` at production call sites.
/// Test code may pass a custom value to verify the formula, but all production
/// calls MUST use the locked constant so the loop cannot tune the penalty.
///
/// Panics: never (no division by zero — t is clamped to (0, 1) exclusive).
#[must_use]
pub fn abstention_aware_score(outcome: OutcomeTriple, t: f64) -> f64 {
    // Clamp t into (0, 1) exclusive so the formula is always finite.
    // In production t == DEFAULT_CONFIDENCE_TARGET = 0.75, which is well-formed.
    let t_clamped = t.clamp(f64::EPSILON, 1.0 - f64::EPSILON);
    let penalty = t_clamped / (1.0 - t_clamped);

    if !outcome.answered {
        // Abstain: 0 credit, 0 penalty. This is strictly better than a wrong
        // answer under any t in (0, 1).
        0.0
    } else if outcome.correct {
        // Correct answer: full credit.
        1.0
    } else {
        // Confident-wrong: negative penalty, strictly worse than abstaining.
        // At t=0.75: penalty = 0.75/0.25 = 3.0 -> score = -3.0.
        // This is the key incentive: confident-wrong < abstain.
        -penalty
    }
}

// ---------------------------------------------------------------------------
// RewardMode — alternate scoring modes (R2 falsifiability harness).
// ---------------------------------------------------------------------------

/// Scoring mode selector for `abstention_aware_score_with_mode`.
///
/// `Normal` is the production mode (correct=+1, abstain=0, wrong=-t/(1-t)).
/// `ConfidentGuessing` is the deliberately broken mode used exclusively in
/// contract tests to verify that R2 falsifiability holds: under this mode the
/// ranking flips (confident-wrong beats abstain), proving the metric changes
/// when the incentive is inverted.
///
/// Do NOT use `ConfidentGuessing` in any production call site. It is
/// `pub` only so the contract test (outside this crate) can drive it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardMode {
    /// Normal confidence-target grading (production default).
    Normal,
    /// Broken reward: confident-wrong = +1.0, abstain = 0.0.
    /// Deliberately inverts the incentive to exercise the R2 falsifiability arm.
    ConfidentGuessing,
}

/// Score one evaluation item with an explicit reward mode.
///
/// Under `RewardMode::Normal` this is identical to `abstention_aware_score`.
/// Under `RewardMode::ConfidentGuessing` the reward is inverted:
///   - answered & wrong  ->  +1.0  (confident-wrong rewarded)
///   - abstained          ->   0.0  (abstain no longer wins)
///   - correct & answered ->  +1.0  (unchanged; correct still rewarded)
///
/// This function exists solely to make the R2 falsifiability arm testable
/// without shipping a production footgun. All production callers MUST use
/// `abstention_aware_score` with `RewardMode::Normal` semantics (i.e. the
/// plain `abstention_aware_score` function).
#[must_use]
pub fn abstention_aware_score_with_mode(
    outcome: OutcomeTriple,
    t: f64,
    mode: RewardMode,
) -> f64 {
    match mode {
        RewardMode::Normal => abstention_aware_score(outcome, t),
        RewardMode::ConfidentGuessing => {
            if !outcome.answered {
                // Abstain: no credit under the broken incentive.
                0.0
            } else {
                // Both correct and confident-wrong receive +1.0.
                // This inverts the penalty, making confident-wrong as attractive
                // as a correct answer — the degenerate reward the lock prevents.
                1.0
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Aggregate accumulator
// ---------------------------------------------------------------------------

/// Per-run aggregate of abstention-aware scores over a corpus of items.
///
/// Accumulate via `record()`, then read the statistics via the accessor fields
/// or `mean_score()` / `abstention_rate()` / `confident_wrong_rate()`.
/// All values are in [0.0, 1.0] for rates; mean_score is in [-penalty, +1.0].
#[derive(Debug, Clone, Default)]
pub struct AbstentionAggregate {
    score_sum: f64,
    abstained_count: u64,
    confident_wrong_count: u64,
    n: u64,
}

impl AbstentionAggregate {
    /// Construct an empty aggregate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one item.
    ///
    /// `t` MUST be `DEFAULT_CONFIDENCE_TARGET` at production call sites.
    pub fn record(&mut self, outcome: OutcomeTriple, t: f64) {
        let s = abstention_aware_score(outcome, t);
        self.score_sum += s;
        if !outcome.answered {
            self.abstained_count += 1;
        } else if !outcome.correct {
            self.confident_wrong_count += 1;
        }
        self.n += 1;
    }

    /// Number of items recorded.
    #[must_use]
    pub fn n(&self) -> u64 {
        self.n
    }

    /// Mean abstention-aware score over all recorded items. Returns 0.0 if
    /// no items have been recorded (empty aggregate is neutral, not a failure).
    #[must_use]
    pub fn mean_score(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.score_sum / (self.n as f64)
        }
    }

    /// Fraction of items that were abstentions. 0.0 when n == 0.
    #[must_use]
    pub fn abstention_rate(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            (self.abstained_count as f64) / (self.n as f64)
        }
    }

    /// Fraction of items that were confident-wrong (answered but incorrect).
    /// 0.0 when n == 0.
    #[must_use]
    pub fn confident_wrong_rate(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            (self.confident_wrong_count as f64) / (self.n as f64)
        }
    }
}

// ---------------------------------------------------------------------------
// Eval-log parser (Risk-1 mitigation)
// ---------------------------------------------------------------------------

/// Parse the eval_log format "suite=X score=Y.Z clean|abstain|wrong" into an
/// OutcomeTriple. This is a BEST-EFFORT text parser; if the log carries no
/// refusal marker the RefusalDecision defaults to Answer{confidence: score}
/// and the outcome is pure pass/fail (abstention_rate = 0, which degrades
/// gracefully per the plan's explicit allowance).
///
/// Supported markers in the log string (case-insensitive):
///   - "abstain" or "refused"  -> RefusalDecision::Abstain, answered=false
///   - "wrong" or "fail"       -> Answer{confidence}, answered=true, correct=false
///   - "clean" or "pass"       -> Answer{confidence}, answered=true, correct=true
///
/// When the score token "score=Y.Z" is present it is used as the Answer
/// confidence. Falls back to 0.5 (neutral) when absent.
///
/// Returns `None` only when the log string is empty (degenerate candidate).
#[must_use]
pub fn parse_eval_log_outcome(eval_log: &str) -> Option<OutcomeTriple> {
    if eval_log.is_empty() {
        return None;
    }

    let lower = eval_log.to_lowercase();

    // Check for explicit abstain markers.
    if lower.contains("abstain") || lower.contains("refused") {
        return Some(OutcomeTriple {
            answered: false,
            correct: false,
            refusal: RefusalDecision::Abstain,
        });
    }

    // Extract score= token for Answer confidence.
    let confidence = extract_score_token(&lower).unwrap_or(0.5_f64);

    // Determine correctness from explicit markers; default to pass when
    // ambiguous (conservative: unstructured logs are not penalised).
    let pass = !(lower.contains("wrong") || lower.contains("fail"));

    Some(OutcomeTriple::new(
        RefusalDecision::Answer { confidence },
        pass,
    ))
}

// ---------------------------------------------------------------------------
// I-8 calibration corpus suite filter (F-2 remedy, WARN-1)
// ---------------------------------------------------------------------------

/// Suite names that MUST be excluded from the conformal calibration corpus.
///
/// `voluntary_abort` rows are emitted by the I-8 pre-dispatch dead-end guard
/// and the `acknowledge_infeasible` abort handler.  They carry synthetic
/// `mean_score=0.0 / abstention_rate=1.0` values that do not reflect real
/// task-difficulty distribution; including them would down-bias the corpus
/// and tighten tau artificially.
///
/// I-23 (unified calibration facade) MUST import this constant and apply
/// `is_suite_excluded_from_calibration` before ingesting any
/// `AbstentionScored` row into the corpus builder.
///
/// Fail-closed policy: any suite NOT in [`CALIBRATION_ALLOWED_SUITES`] is
/// treated as excluded until an explicit allow decision is made and the
/// constant is updated.  Unknown suites MUST be excluded, not allowed.
pub const CALIBRATION_EXCLUDED_SUITES: &[&str] = &["voluntary_abort"];

/// The inverse allow-list: explicitly vetted suites that MAY enter the
/// calibration corpus.  I-23 MUST gate ingestion on membership here rather
/// than on `CALIBRATION_EXCLUDED_SUITES` — the allow-list is fail-closed
/// (unknown = excluded), whereas the block-list is fail-open (unknown = allowed).
///
/// Currently covers the SecCodePLT bench suite only.  Extend here when a
/// new suite's corpus semantics are reviewed and approved.
pub const CALIBRATION_ALLOWED_SUITES: &[&str] = &["seccodeplt"];

/// Returns `true` when the suite should be EXCLUDED from the conformal
/// calibration corpus (fail-closed: unknown suites are excluded).
///
/// Usage: `if is_suite_excluded_from_calibration(row.suite.as_str()) { continue; }`
#[must_use]
pub fn is_suite_excluded_from_calibration(suite: &str) -> bool {
    // Fail-closed: a suite is allowed only when it is on the explicit allow-list.
    // This means unknown suites — suites added without a corpus-review decision —
    // are safely excluded until promoted to CALIBRATION_ALLOWED_SUITES.
    !CALIBRATION_ALLOWED_SUITES.contains(&suite)
}

/// Extract the float value from a "score=Y.Z" token in the (already
/// lowercased) log string. Returns `None` if no such token is present or
/// the value is non-finite.
fn extract_score_token(lower: &str) -> Option<f64> {
    let prefix = "score=";
    let start = lower.find(prefix)?;
    let rest = &lower[start + prefix.len()..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(rest.len());
    let token = &rest[..end];
    token.parse::<f64>().ok().filter(|v| v.is_finite())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // (a) Correct answer = +1.
    #[test]
    fn correct_answer_scores_plus_one() {
        let outcome = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, true);
        let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    // (a) Abstain = 0.
    #[test]
    fn abstain_scores_zero() {
        let outcome = OutcomeTriple::new(RefusalDecision::Abstain, false);
        let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
        assert!((s - 0.0).abs() < f64::EPSILON);
    }

    // (a) Confident-wrong = -t/(1-t) at default t=0.75 -> -3.0.
    #[test]
    fn confident_wrong_scores_negative_penalty() {
        let outcome = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false);
        let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
        // t=0.75 -> penalty=0.75/0.25=3.0 -> score=-3.0
        assert!(
            (s - (-3.0)).abs() < 1e-10,
            "expected -3.0, got {s}"
        );
    }

    // (b) score(correct-abstain) > score(confident-wrong).
    #[test]
    fn abstain_strictly_better_than_confident_wrong() {
        let abstain = OutcomeTriple::new(RefusalDecision::Abstain, false);
        let wrong = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false);
        let s_abstain = abstention_aware_score(abstain, DEFAULT_CONFIDENCE_TARGET);
        let s_wrong = abstention_aware_score(wrong, DEFAULT_CONFIDENCE_TARGET);
        assert!(
            s_abstain > s_wrong,
            "abstain ({s_abstain}) must beat confident-wrong ({s_wrong})"
        );
    }

    // (e) t is LOCKED — the module does not expose a setter; DEFAULT_CONFIDENCE_TARGET is const.
    // (Verified structurally: the const is not mutable and is not read from env.)

    // (f) Determinism: same input -> same aggregate.
    #[test]
    fn aggregate_deterministic_same_inputs() {
        let outcomes = vec![
            OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.8 }, true),
            OutcomeTriple::new(RefusalDecision::Abstain, false),
            OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false),
        ];
        let mut agg1 = AbstentionAggregate::new();
        let mut agg2 = AbstentionAggregate::new();
        for &o in &outcomes {
            agg1.record(o, DEFAULT_CONFIDENCE_TARGET);
            agg2.record(o, DEFAULT_CONFIDENCE_TARGET);
        }
        assert_eq!(agg1.n(), agg2.n());
        assert!((agg1.mean_score() - agg2.mean_score()).abs() < f64::EPSILON);
        assert!((agg1.abstention_rate() - agg2.abstention_rate()).abs() < f64::EPSILON);
        assert!(
            (agg1.confident_wrong_rate() - agg2.confident_wrong_rate()).abs() < f64::EPSILON
        );
    }

    // Eval-log parser: abstain marker.
    #[test]
    fn parse_eval_log_abstain_marker() {
        let triple = parse_eval_log_outcome("suite=math score=0.2 abstain").unwrap();
        assert!(!triple.answered);
        assert_eq!(triple.refusal, RefusalDecision::Abstain);
    }

    // Eval-log parser: pass verdict.
    #[test]
    fn parse_eval_log_pass() {
        let triple = parse_eval_log_outcome("suite=math score=0.8 clean").unwrap();
        assert!(triple.answered);
        assert!(triple.correct);
    }

    // Eval-log parser: wrong verdict.
    #[test]
    fn parse_eval_log_wrong() {
        let triple = parse_eval_log_outcome("suite=math score=0.6 wrong").unwrap();
        assert!(triple.answered);
        assert!(!triple.correct);
    }

    // Empty log returns None.
    #[test]
    fn parse_eval_log_empty_returns_none() {
        assert!(parse_eval_log_outcome("").is_none());
    }

    // Broken arm test: reward_confident_guessing mode uses t=0.0 (via clamp,
    // penalty -> 0, making confident-wrong score = 0 = abstain, or
    // with very small t the penalty approaches 0).
    // This test documents that flipping t->0 degrades to binary grading.
    #[test]
    fn broken_arm_low_t_makes_wrong_equal_abstain() {
        let abstain = OutcomeTriple::new(RefusalDecision::Abstain, false);
        let wrong = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false);
        // t=epsilon -> penalty -> epsilon/(1-epsilon) ≈ epsilon ≈ 0.
        let s_abstain = abstention_aware_score(abstain, f64::EPSILON);
        let s_wrong = abstention_aware_score(wrong, f64::EPSILON);
        // penalty ≈ 0 -> wrong ≈ 0 = abstain -> not strictly better.
        // Contrast with DEFAULT_CONFIDENCE_TARGET where wrong = -3.
        assert!(
            (s_wrong - s_abstain).abs() < 0.01,
            "with t≈0, penalty≈0, wrong≈0 (broken arm behaviour)"
        );
    }
}
