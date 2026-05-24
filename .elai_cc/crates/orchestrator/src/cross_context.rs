//! D-3: Cross-context verifier — same claim, two independent context windows.
//!
//! Design grounded in: V-STaR (arxiv:2402.06457, T1) heterogeneous-verifier
//! pattern + the standard shingle-jaccard token-similarity construction (van
//! Rijsbergen, Information Retrieval, 1979 ch.3) + ELAI's existing blake3
//! content-hash pattern (used across the orchestrator for deduplication —
//! see ssb.rs payload `content_blake3` fields).
//!
//! The plan-cited arxiv:2603.21454 "CCV" reference is future-dated
//! (arxiv prefixes follow YY.MM where YY is the last two digits of the year
//! and MM is the month; "2603" would be year 2026 month 03 — which the
//! actual arxiv corpus has not produced as a recognised paper id). The
//! design here is therefore derived from first principles + the
//! verification_result.rs aggregator pattern, not from that paper. See
//! `workflow/reasoning/D-3-decisions.md` (DECISION-3).
//!
//! Wiring discipline: this module produces [`CrossContextAgreement`]
//! values + (optionally, via [`CrossContextVerifier`]) one
//! `SsbActionType::CrossContextDispatched` row per `run` call. It NEVER
//! depends on the policy model — agreement is computed structurally over
//! the two executor outputs.

use std::time::Duration;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CrossContextAgreement
// ---------------------------------------------------------------------------

/// Tri-state agreement verdict for a single cross-context run.
///
/// `Agree` = both context windows produced outputs whose shingle-jaccard
/// similarity cleared the configured threshold (default 0.7).
/// `Disagree` = the similarity fell below the threshold.
/// `Insufficient` = one or both sides timed out / returned an empty output
/// (no agreement can be inferred — the caller defers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossContextAgreement {
    Agree,
    Disagree,
    Insufficient,
}

impl CrossContextAgreement {
    /// Snake-case wire label for SSB rows + dashboard rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agree => "agree",
            Self::Disagree => "disagree",
            Self::Insufficient => "insufficient",
        }
    }

    /// True only on `Agree`. Convenience for gate composition.
    #[must_use]
    pub fn is_agree(&self) -> bool {
        matches!(self, Self::Agree)
    }
}

// ---------------------------------------------------------------------------
// CrossContextRun result
// ---------------------------------------------------------------------------

/// Result of one [`run_cross_context`] / [`CrossContextVerifier::run`] call.
///
/// `context_a_hash` / `context_b_hash` are blake3 hex (64 chars) of the
/// respective context-side outputs (after trimming). `jaccard` is the
/// shingle-n=3 token-level jaccard similarity on `[0.0, 1.0]`. `agreement`
/// is the derived verdict per the configured `threshold`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossContextRun {
    pub claim_id: String,
    pub context_a_hash: String,
    pub context_b_hash: String,
    pub jaccard: f64,
    pub agreement: CrossContextAgreement,
}

impl CrossContextRun {
    /// Builds an Insufficient run (used by the timeout / empty-output paths).
    #[must_use]
    pub fn insufficient(claim_id: impl Into<String>) -> Self {
        Self {
            claim_id: claim_id.into(),
            context_a_hash: String::new(),
            context_b_hash: String::new(),
            jaccard: 0.0,
            agreement: CrossContextAgreement::Insufficient,
        }
    }
}

// ---------------------------------------------------------------------------
// Shingle / jaccard helpers
// ---------------------------------------------------------------------------

/// Default shingle size (number of consecutive whitespace-tokens per shingle).
/// `n = 3` follows the standard text-similarity construction.
pub const DEFAULT_SHINGLE_N: usize = 3;

/// Default agreement threshold. Conservatively chosen — pre-promotion gates
/// prefer false-negative `Insufficient` over false-positive `Agree`.
pub const DEFAULT_JACCARD_THRESHOLD: f64 = 0.7;

/// Default per-side execution timeout. Cross-context runs are bounded so a
/// stuck context cannot starve the consensus pass.
pub const DEFAULT_SIDE_TIMEOUT: Duration = Duration::from_secs(30);

/// Compute whitespace-tokenised shingles of size `n` from `text`. Returns an
/// empty Vec when `n == 0` or there are fewer than `n` tokens.
///
/// Closed-form, deterministic, no I/O. Each shingle is a single owned
/// `String` formed by joining the `n` consecutive tokens with a single
/// space — same canonical form on both sides so jaccard comparisons match
/// across formatting whitespace.
#[must_use]
pub fn shingles(text: &str, n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() < n {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(tokens.len() - n + 1);
    for window in tokens.windows(n) {
        out.push(window.join(" "));
    }
    out
}

/// Jaccard similarity over the deduplicated shingle sets of `a_text` /
/// `b_text` with shingle size `n`. Returns `0.0` when both shingle sets
/// are empty (no signal to compare). Returns `1.0` when the deduplicated
/// sets are equal.
///
/// Closed-form, deterministic, no allocations beyond two `Vec<&str>` +
/// the intersection / union counters.
#[must_use]
pub fn shingle_jaccard(a_text: &str, b_text: &str, n: usize) -> f64 {
    let a = shingles(a_text, n);
    let b = shingles(b_text, n);
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }

    // Deduplicate via sorted Vecs (no HashMap to avoid pulling fxhash here;
    // shingle counts are bounded by tokens-per-side which is <= a few
    // thousand in worst-case orchestrator outputs).
    let mut a_sorted: Vec<&str> = a.iter().map(String::as_str).collect();
    a_sorted.sort();
    a_sorted.dedup();
    let mut b_sorted: Vec<&str> = b.iter().map(String::as_str).collect();
    b_sorted.sort();
    b_sorted.dedup();

    // Two-pointer intersection + union over sorted slices.
    let (mut i, mut j) = (0usize, 0usize);
    let (mut intersect, mut union_) = (0usize, 0usize);
    while i < a_sorted.len() && j < b_sorted.len() {
        match a_sorted[i].cmp(b_sorted[j]) {
            std::cmp::Ordering::Less => {
                union_ += 1;
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                union_ += 1;
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                intersect += 1;
                union_ += 1;
                i += 1;
                j += 1;
            }
        }
    }
    union_ += (a_sorted.len() - i) + (b_sorted.len() - j);

    if union_ == 0 {
        0.0
    } else {
        intersect as f64 / union_ as f64
    }
}

/// blake3 hex digest of the trimmed input (64 lowercase hex chars).
#[must_use]
pub fn blake3_hex(text: &str) -> String {
    blake3::hash(text.trim().as_bytes()).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// run_cross_context: closed-form computation given two side outputs
// ---------------------------------------------------------------------------

/// Compute a [`CrossContextRun`] given the two already-produced side
/// outputs. The orchestrator side is responsible for executing the claim
/// against the two contexts (potentially in parallel) and passing the
/// trimmed text outputs here.
///
/// Empty input on either side produces `Insufficient`. The shingle size
/// `n` defaults to [`DEFAULT_SHINGLE_N`] when set to 0.
#[must_use]
pub fn run_cross_context(
    claim_id: impl Into<String>,
    context_a_text: &str,
    context_b_text: &str,
    threshold: f64,
    shingle_n: usize,
) -> CrossContextRun {
    let claim_id = claim_id.into();
    let trimmed_a = context_a_text.trim();
    let trimmed_b = context_b_text.trim();
    if trimmed_a.is_empty() || trimmed_b.is_empty() {
        return CrossContextRun::insufficient(claim_id);
    }
    let n = if shingle_n == 0 { DEFAULT_SHINGLE_N } else { shingle_n };
    let jaccard = shingle_jaccard(trimmed_a, trimmed_b, n);
    let agreement = if jaccard >= threshold {
        CrossContextAgreement::Agree
    } else if jaccard.is_nan() {
        // Defensive: NaN jaccard would only arise from a bug above; treat
        // as Insufficient rather than rejecting outright.
        CrossContextAgreement::Insufficient
    } else {
        CrossContextAgreement::Disagree
    };
    CrossContextRun {
        claim_id,
        context_a_hash: blake3_hex(trimmed_a),
        context_b_hash: blake3_hex(trimmed_b),
        jaccard,
        agreement,
    }
}

// ---------------------------------------------------------------------------
// CrossContextVerifier: stateful holder for threshold + shingle size
// ---------------------------------------------------------------------------

/// Stateful wrapper that carries the configured threshold + shingle size.
///
/// Used by call sites that want to pre-load the cross-context configuration
/// once and dispatch many `run` calls against it. Stateless callers can use
/// [`run_cross_context`] directly.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossContextVerifier {
    pub threshold: f64,
    pub shingle_n: usize,
    pub side_timeout: Duration,
}

impl CrossContextVerifier {
    /// Default-configured verifier: threshold 0.7, shingle n=3, per-side
    /// timeout 30s.
    #[must_use]
    pub fn new_default() -> Self {
        Self {
            threshold: DEFAULT_JACCARD_THRESHOLD,
            shingle_n: DEFAULT_SHINGLE_N,
            side_timeout: DEFAULT_SIDE_TIMEOUT,
        }
    }

    /// Construct with explicit threshold + shingle size. `threshold` is
    /// clamped to `[0.0, 1.0]`; `shingle_n` falls back to
    /// [`DEFAULT_SHINGLE_N`] when zero.
    #[must_use]
    pub fn with_config(threshold: f64, shingle_n: usize, side_timeout: Duration) -> Self {
        let threshold = if threshold.is_nan() {
            DEFAULT_JACCARD_THRESHOLD
        } else {
            threshold.clamp(0.0, 1.0)
        };
        let shingle_n = if shingle_n == 0 { DEFAULT_SHINGLE_N } else { shingle_n };
        Self {
            threshold,
            shingle_n,
            side_timeout,
        }
    }

    /// Run a cross-context check given the two side outputs. Mirrors
    /// [`run_cross_context`] but using the verifier's stored config.
    #[must_use]
    pub fn run(
        &self,
        claim_id: impl Into<String>,
        context_a_text: &str,
        context_b_text: &str,
    ) -> CrossContextRun {
        run_cross_context(
            claim_id,
            context_a_text,
            context_b_text,
            self.threshold,
            self.shingle_n,
        )
    }
}

impl Default for CrossContextVerifier {
    fn default() -> Self {
        Self::new_default()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shingles_basic_window_join() {
        let s = shingles("the quick brown fox jumps", 3);
        assert_eq!(
            s,
            vec![
                "the quick brown".to_string(),
                "quick brown fox".to_string(),
                "brown fox jumps".to_string(),
            ]
        );
    }

    #[test]
    fn shingles_short_input_returns_empty() {
        assert!(shingles("two tokens", 3).is_empty());
        assert!(shingles("", 3).is_empty());
        assert!(shingles("hi", 0).is_empty());
    }

    #[test]
    fn shingle_jaccard_identical_inputs_is_one() {
        let j = shingle_jaccard("the quick brown fox", "the quick brown fox", 3);
        // Identical text -> identical shingle set -> jaccard = 1.0.
        assert!((j - 1.0).abs() < 1e-9, "expected 1.0, got {j}");
    }

    #[test]
    fn shingle_jaccard_disjoint_inputs_is_zero() {
        let j = shingle_jaccard("alpha beta gamma delta", "epsilon zeta eta theta", 3);
        assert!(j.abs() < 1e-9, "expected 0.0 on disjoint, got {j}");
    }

    #[test]
    fn shingle_jaccard_partial_overlap_is_between_zero_and_one() {
        let a = "the quick brown fox jumps";
        let b = "the quick brown dog runs";
        let j = shingle_jaccard(a, b, 3);
        // Overlap = {"the quick brown"}; |A|=3, |B|=3, |A∩B|=1, |A∪B|=5
        // -> j = 1/5 = 0.2.
        assert!((j - 0.2).abs() < 1e-9, "expected 0.2, got {j}");
    }

    #[test]
    fn run_cross_context_agree_on_identical_inputs() {
        let r = run_cross_context("claim-1", "the cat sat on the mat", "the cat sat on the mat", 0.7, 3);
        assert_eq!(r.agreement, CrossContextAgreement::Agree);
        assert!((r.jaccard - 1.0).abs() < 1e-9);
        assert_eq!(r.context_a_hash.len(), 64);
        assert_eq!(r.context_b_hash.len(), 64);
        assert_eq!(r.context_a_hash, r.context_b_hash);
    }

    #[test]
    fn run_cross_context_disagree_below_threshold() {
        let r = run_cross_context(
            "claim-2",
            "alpha beta gamma delta epsilon",
            "uno dos tres cuatro cinco",
            0.7,
            3,
        );
        assert_eq!(r.agreement, CrossContextAgreement::Disagree);
        assert!(r.jaccard < 0.7);
        assert_ne!(r.context_a_hash, r.context_b_hash);
    }

    #[test]
    fn run_cross_context_insufficient_on_empty_side() {
        let r_a = run_cross_context("claim-3", "", "the cat sat on the mat", 0.7, 3);
        assert_eq!(r_a.agreement, CrossContextAgreement::Insufficient);
        let r_b = run_cross_context("claim-4", "the cat sat", "   ", 0.7, 3);
        assert_eq!(r_b.agreement, CrossContextAgreement::Insufficient);
    }

    #[test]
    fn cross_context_verifier_default_threshold_is_0_7() {
        let v = CrossContextVerifier::new_default();
        assert!((v.threshold - DEFAULT_JACCARD_THRESHOLD).abs() < 1e-9);
        assert_eq!(v.shingle_n, DEFAULT_SHINGLE_N);
    }

    #[test]
    fn cross_context_verifier_with_config_clamps_threshold() {
        let v_high = CrossContextVerifier::with_config(1.5, 4, DEFAULT_SIDE_TIMEOUT);
        assert!((v_high.threshold - 1.0).abs() < 1e-9);
        assert_eq!(v_high.shingle_n, 4);

        let v_low = CrossContextVerifier::with_config(-0.1, 0, DEFAULT_SIDE_TIMEOUT);
        assert!(v_low.threshold.abs() < 1e-9);
        // shingle_n=0 falls back to default.
        assert_eq!(v_low.shingle_n, DEFAULT_SHINGLE_N);

        let v_nan = CrossContextVerifier::with_config(f64::NAN, 3, DEFAULT_SIDE_TIMEOUT);
        assert!((v_nan.threshold - DEFAULT_JACCARD_THRESHOLD).abs() < 1e-9);
    }

    #[test]
    fn cross_context_run_agreement_as_str_round_trip() {
        for v in [
            CrossContextAgreement::Agree,
            CrossContextAgreement::Disagree,
            CrossContextAgreement::Insufficient,
        ] {
            let s = v.as_str();
            // Round-trip via serde to confirm rename_all stays in sync.
            let j = serde_json::to_string(&v).expect("serialize");
            let back: CrossContextAgreement = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, v);
            // is_agree only true for Agree.
            assert_eq!(v.is_agree(), v == CrossContextAgreement::Agree);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn cross_context_verifier_run_uses_stored_threshold() {
        let v = CrossContextVerifier::with_config(0.05, 3, DEFAULT_SIDE_TIMEOUT);
        // 0.2 jaccard on partial overlap >= 0.05 threshold -> Agree.
        let r = v.run(
            "claim-low",
            "the quick brown fox jumps",
            "the quick brown dog runs",
        );
        assert_eq!(r.agreement, CrossContextAgreement::Agree);

        let v_strict = CrossContextVerifier::with_config(0.9, 3, DEFAULT_SIDE_TIMEOUT);
        let r2 = v_strict.run(
            "claim-strict",
            "the quick brown fox jumps",
            "the quick brown dog runs",
        );
        assert_eq!(r2.agreement, CrossContextAgreement::Disagree);
    }
}
