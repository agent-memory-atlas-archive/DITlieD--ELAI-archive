//! Inference-time N-candidate critic rerank pipeline.
//!
//! Reference: OpenHands Nov 2025 blog "SOTA on SWE-Bench-Verified with
//! inference-time scaling and critic-model rerank"
//! (openhands.dev/blog/sota-on-swe-bench-verified-with-inference-time-
//! scaling-and-critic-model). The OpenHands harness generates N=3-5
//! parallel candidate patches per task, scores each with a critic model,
//! and commits the argmax. Reported lift on SWE-Bench-Verified vs single-
//! sample baseline: ~13pp absolute at the time of publication.
//!
//! DISTINCT FROM [`crate::mini_critic::MiniCritic`]: mini_critic is the
//! ROLE-TIME critic surface (signature linting + lint heuristics applied
//! to one already-chosen response). This module is the INFERENCE-TIME
//! N-candidate reranker (rank parallel candidates BEFORE one is chosen).
//! Both can be active in the same pipeline; reviewer-confusion guard.
//!
//! Wire-dark: this module ships the typed surface + argmax core only.
//! Production wiring (calling
//! [`crate::runtime_orchestrator::RuntimeOrchestrator::try_complete`]
//! per candidate) is gated by
//! [`super::peer_systems_emit_enabled`] / `ELAI_PEER_SYSTEMS_ENABLE=1`
//! and is intentionally left to follow-on plans (Tier-C per
//! workflow/reasoning/E-21-decisions.md). The pure-compute argmax
//! kernel is unit-tested below.
//!
//! Plan: `docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md`
//! task 3. Decisions D-B-2 + D-B-4 in
//! `workflow/reasoning/E-21-decisions.md`.

use std::cmp::Ordering;

/// One candidate emitted by an N-candidate critic-rerank generation
/// round.
///
/// Carries the raw patch text plus a `runtime_label` identifying which
/// backend produced the candidate (e.g. `"anthropic_local_sidecar"` /
/// `"anthropic_cloud"`) and the critic-emitted `score` in `[0.0, 1.0]`.
/// The label is informational for SSB telemetry and for stable tie-
/// breaking (sort by label ascending when scores tie).
#[derive(Debug, Clone, PartialEq)]
pub struct CriticCandidate {
    /// Patch text (the unified-diff or replacement payload the runtime
    /// emitted). Opaque to the reranker; treated as a blob.
    pub patch: String,
    /// Runtime backend label that produced this candidate. Empty string
    /// is allowed (single-runtime fallback path) but disrecommended.
    pub runtime_label: String,
    /// Critic-emitted score in `[0.0, 1.0]`. Caller is responsible for
    /// clamping; `pick_argmax` does NOT range-check.
    pub score: f64,
}

/// Outcome of a single rerank round.
///
/// `pick == None` indicates the input was empty (no candidates were
/// generated this round). Callers should treat this as a
/// `critic_rerank_skipped` event upstream (emit
/// [`super::PeerSystemsEventKind::CriticRerankSkipped`] via the
/// `details_json` payload).
#[derive(Debug, Clone, PartialEq)]
pub struct CriticRerankOutcome {
    /// The chosen candidate (argmax over scores; stable tie-break).
    pub pick: Option<CriticCandidate>,
    /// Number of candidates considered this round.
    pub considered: usize,
    /// Maximum score among the candidates, or 0.0 if input empty.
    pub max_score: f64,
    /// Minimum score among the candidates, or 0.0 if input empty.
    pub min_score: f64,
}

/// Pure-compute rerank surface. State-less; lives as a unit struct so
/// callers can swap in a mocked variant in tests without changing the
/// call-site shape.
#[derive(Debug, Clone, Copy, Default)]
pub struct CriticRerank;

impl CriticRerank {
    /// Pick the argmax candidate by score. Stable tie-break order:
    /// (1) highest score, (2) lexicographically smallest `runtime_label`,
    /// (3) lexicographically smallest `patch`. Determinism matters here
    /// so re-running the harness on the same N candidates always picks
    /// the same one, which lets the auditor compare runs by `patch_hash`
    /// alone.
    ///
    /// Returns a populated [`CriticRerankOutcome`] for any input;
    /// `pick == None` only when `candidates` is empty.
    ///
    /// NaN scores are treated as smaller than any real number (a NaN
    /// score loses every comparison and never wins the argmax). This
    /// matches the AutoCodeRover-style sort convention used by
    /// [`super::sbfl::compute_ochiai`] (descending by score, ties
    /// broken by path/line).
    #[must_use]
    pub fn pick_argmax(candidates: &[CriticCandidate]) -> CriticRerankOutcome {
        if candidates.is_empty() {
            return CriticRerankOutcome {
                pick: None,
                considered: 0,
                max_score: 0.0,
                min_score: 0.0,
            };
        }

        let mut best_idx: usize = 0;
        // Initialize range trackers from the FIRST finite score we see;
        // a leading NaN must not poison max_score/min_score for the
        // remainder of the loop. If no finite score exists in the entire
        // input, we surface 0.0 / 0.0 (same convention as the empty-
        // input branch above).
        let mut max_score: f64 = if candidates[0].score.is_finite() {
            candidates[0].score
        } else {
            f64::NEG_INFINITY
        };
        let mut min_score: f64 = if candidates[0].score.is_finite() {
            candidates[0].score
        } else {
            f64::INFINITY
        };

        for (i, c) in candidates.iter().enumerate().skip(1) {
            // Explicit NaN guard: partial_cmp returns None for any
            // comparison involving NaN, which previously fell into the
            // "_ => false" arm. That meant a NaN at index 0 stayed as
            // best_idx even when a later candidate had a finite score
            // (because partial_cmp(finite, NaN) is also None -> Less
            // branch -> stronger=false). Handle the three NaN cases
            // explicitly so a NaN never wins and a NaN-seeded best_idx
            // gets replaced by the first finite candidate.
            let cur_nan = c.score.is_nan();
            let best_nan = candidates[best_idx].score.is_nan();
            let stronger = match (cur_nan, best_nan) {
                (true, _) => false,        // NaN current never wins
                (false, true) => true,     // finite always beats NaN best
                (false, false) => match c.score.partial_cmp(&candidates[best_idx].score) {
                    Some(Ordering::Greater) => true,
                    Some(Ordering::Equal) => {
                        match c.runtime_label.cmp(&candidates[best_idx].runtime_label) {
                            Ordering::Less => true,
                            Ordering::Equal => {
                                matches!(c.patch.cmp(&candidates[best_idx].patch), Ordering::Less)
                            }
                            Ordering::Greater => false,
                        }
                    }
                    _ => false,
                },
            };
            if stronger {
                best_idx = i;
            }

            // Track score range across the whole input, including NaN
            // -> finite-only update (NaN cannot widen the range).
            if c.score.is_finite() {
                if !max_score.is_finite() || c.score > max_score {
                    max_score = c.score;
                }
                if !min_score.is_finite() || c.score < min_score {
                    min_score = c.score;
                }
            }
        }

        // If no finite score was ever observed, normalize the sentinel
        // INFINITY values back to 0.0 so callers see the same "empty
        // signal" they would for an empty input.
        if !max_score.is_finite() {
            max_score = 0.0;
        }
        if !min_score.is_finite() {
            min_score = 0.0;
        }

        CriticRerankOutcome {
            pick: Some(candidates[best_idx].clone()),
            considered: candidates.len(),
            max_score,
            min_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(patch: &str, label: &str, score: f64) -> CriticCandidate {
        CriticCandidate {
            patch: patch.to_string(),
            runtime_label: label.to_string(),
            score,
        }
    }

    #[test]
    fn empty_input_returns_no_pick() {
        let out = CriticRerank::pick_argmax(&[]);
        assert!(out.pick.is_none());
        assert_eq!(out.considered, 0);
        assert!(out.max_score.abs() < 1e-9);
        assert!(out.min_score.abs() < 1e-9);
    }

    #[test]
    fn argmax_picks_highest_score() {
        let candidates = vec![
            cand("patch-a", "local", 0.3),
            cand("patch-b", "cloud", 0.8),
            cand("patch-c", "local", 0.5),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.patch, "patch-b");
        assert!((pick.score - 0.8).abs() < 1e-9);
        assert_eq!(out.considered, 3);
        assert!((out.max_score - 0.8).abs() < 1e-9);
        assert!((out.min_score - 0.3).abs() < 1e-9);
    }

    #[test]
    fn ties_break_by_runtime_label_ascending() {
        // Both have score 0.7. Label "cloud" < "local" -> cloud wins.
        let candidates = vec![
            cand("p-local", "local", 0.7),
            cand("p-cloud", "cloud", 0.7),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.runtime_label, "cloud");
        assert_eq!(pick.patch, "p-cloud");
    }

    #[test]
    fn ties_within_label_break_by_patch_ascending() {
        // Same score, same label -> patch text tie-break.
        let candidates = vec![
            cand("zzz-patch", "x", 0.5),
            cand("aaa-patch", "x", 0.5),
            cand("mmm-patch", "x", 0.5),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.patch, "aaa-patch");
    }

    #[test]
    fn nan_score_never_wins() {
        // A NaN candidate must lose to any real-score candidate so the
        // pipeline survives upstream sentinel-as-NaN bugs without
        // returning a meaningless patch.
        let candidates = vec![
            cand("nan-patch", "broken", f64::NAN),
            cand("good-patch", "ok", 0.1),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.patch, "good-patch");
        assert!((pick.score - 0.1).abs() < 1e-9);
    }

    #[test]
    fn nan_at_index_zero_loses_to_later_finite() {
        // Regression: previously a NaN at index 0 seeded best_idx and
        // partial_cmp(finite, NaN) -> None -> Less branch left best_idx
        // unchanged. Explicit NaN guard must let the first finite
        // candidate displace a NaN-seeded best.
        let candidates = vec![
            cand("nan-first", "broken", f64::NAN),
            cand("real-1", "ok", 0.2),
            cand("real-2", "ok", 0.5),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.patch, "real-2");
        assert!((pick.score - 0.5).abs() < 1e-9);
        assert!((out.max_score - 0.5).abs() < 1e-9);
        assert!((out.min_score - 0.2).abs() < 1e-9);
    }

    #[test]
    fn all_nan_returns_zeroed_range() {
        // When every candidate has NaN score, max_score/min_score
        // collapse to 0.0 (same convention as the empty-input branch).
        // The pick is still Some (index 0) because pick.is_none() is
        // reserved for the empty-input case only.
        let candidates = vec![
            cand("nan-a", "x", f64::NAN),
            cand("nan-b", "y", f64::NAN),
        ];
        let out = CriticRerank::pick_argmax(&candidates);
        assert!(out.pick.is_some());
        assert!(out.max_score.abs() < 1e-9);
        assert!(out.min_score.abs() < 1e-9);
    }

    #[test]
    fn single_candidate_passes_through() {
        let candidates = vec![cand("solo", "only", 0.42)];
        let out = CriticRerank::pick_argmax(&candidates);
        let pick = out.pick.expect("must pick");
        assert_eq!(pick.patch, "solo");
        assert_eq!(out.considered, 1);
        assert!((out.max_score - 0.42).abs() < 1e-9);
        assert!((out.min_score - 0.42).abs() < 1e-9);
    }

    #[test]
    fn determinism_repeated_calls_pick_same() {
        // Stable tie-break: re-running the same input must produce the
        // same pick across repeated calls (the harness depends on this
        // for patch_hash auditor comparison).
        let candidates = vec![
            cand("a", "x", 0.5),
            cand("b", "y", 0.5),
            cand("c", "x", 0.5),
        ];
        let first = CriticRerank::pick_argmax(&candidates);
        let second = CriticRerank::pick_argmax(&candidates);
        let third = CriticRerank::pick_argmax(&candidates);
        assert_eq!(first.pick, second.pick);
        assert_eq!(second.pick, third.pick);
    }
}
