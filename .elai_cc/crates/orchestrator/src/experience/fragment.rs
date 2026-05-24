//! C-49 — Prompt fragment representation.
//!
//! A `PromptFragment` is a small unit of prompt text (typically a single
//! sentence or one prompt-section) that was observed in a successful
//! task trace. The distiller ranks fragments by `freq_used * success_rate`
//! when projecting the per-skill trace bin into a small reusable
//! suggestion list (matching the heuristic adaptation of
//! arXiv:2605.14443's "amortized prompt refinement" — see the module
//! header for grounding caveats).

use serde::{Deserialize, Serialize};

/// A single distilled prompt fragment.
///
/// Identity is `text`; `freq_used` and `success_rate` are aggregate
/// statistics over the per-skill trace buffer. `ts_ms_last_used` is
/// recorded so the lookup surface can break ranking ties in favour of
/// recent fragments (matching the paper's "historical buffer slice
/// H_c" notion at the heuristic level only — we do not implement the
/// RL training loop here).
///
/// Field invariants enforced by the distiller (paper Section 4.2.2,
/// Eq. 5 buffer-insertion threshold):
///   - `success_rate` in `[0.0, 1.0]` (clamped by [`Self::new`]).
///   - `freq_used` non-negative by `u32` typing.
///
/// `Eq` intentionally NOT derived because `f32` is not `Eq` (NaN-payload
/// IEEE-754 quirk; matches the precedent of `SkillManifest` post-C-45).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptFragment {
    /// Raw fragment text. Length-bounded by the caller; the distiller
    /// does not impose a hard cap (callers should trim before insertion).
    pub text: String,
    /// Count of successful traces this fragment appeared in for the
    /// owning skill.
    pub freq_used: u32,
    /// Empirical success rate (`successful_appearances / total_uses`).
    /// Clamped to `[0.0, 1.0]` on construction.
    pub success_rate: f32,
    /// Most-recent `ts_ms` (wall-clock milliseconds since UNIX epoch) at
    /// which this fragment appeared in a successful trace. Used by the
    /// ranker for tie-breaking; producer-side clock per the project's
    /// entropy-quarantine rule (caller supplies it).
    pub ts_ms_last_used: u64,
}

impl PromptFragment {
    /// Construct a new fragment, clamping `success_rate` to `[0, 1]` and
    /// rejecting non-finite inputs (paper-anchored range — Eq. 5
    /// invariant; matches the SkillManifest::update_credit_score clamp).
    #[must_use]
    pub fn new(text: String, freq_used: u32, success_rate: f32, ts_ms_last_used: u64) -> Self {
        let clamped = if success_rate.is_finite() {
            success_rate.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            text,
            freq_used,
            success_rate: clamped,
            ts_ms_last_used,
        }
    }

    /// Combined ranking score: `freq_used * success_rate`. Higher = more
    /// useful per the heuristic distillation pattern. Returns `f32` so
    /// callers can sort with `partial_cmp` (NaN cannot occur because
    /// [`Self::new`] rejects non-finite inputs).
    #[must_use]
    pub fn rank_score(&self) -> f32 {
        // freq_used is u32; cast to f32 is widening on all 32-bit-or-
        // smaller integer counts the distiller will plausibly produce.
        self.freq_used as f32 * self.success_rate
    }

    /// blake3-prefix u64 hash of `text`. Used for SSB
    /// PromptFragmentSelected / Discarded wire surface (raw text never
    /// crosses the dashboard boundary; matches the C-41 /
    /// `skill_name_hash` precedent at `skill_registry::skill_name_hash`).
    #[must_use]
    pub fn text_hash(&self) -> u64 {
        let hash = blake3::hash(self.text.as_bytes());
        let bytes = hash.as_bytes();
        let mut prefix = [0u8; 8];
        prefix.copy_from_slice(&bytes[..8]);
        u64::from_le_bytes(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_rate_clamps_high() {
        let f = PromptFragment::new("x".into(), 3, 2.0, 0);
        assert!((f.success_rate - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn success_rate_clamps_low() {
        let f = PromptFragment::new("x".into(), 3, -0.5, 0);
        assert!(f.success_rate.abs() < f32::EPSILON);
    }

    #[test]
    fn success_rate_nan_becomes_zero() {
        let f = PromptFragment::new("x".into(), 3, f32::NAN, 0);
        assert!(f.success_rate.abs() < f32::EPSILON);
    }

    #[test]
    fn rank_score_is_product() {
        let f = PromptFragment::new("x".into(), 4, 0.5, 0);
        assert!((f.rank_score() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn text_hash_is_stable_and_distinct() {
        let a = PromptFragment::new("hello".into(), 0, 0.0, 0);
        let b = PromptFragment::new("hello".into(), 99, 1.0, 12345);
        let c = PromptFragment::new("world".into(), 0, 0.0, 0);
        assert_eq!(a.text_hash(), b.text_hash());
        assert_ne!(a.text_hash(), c.text_hash());
    }
}
