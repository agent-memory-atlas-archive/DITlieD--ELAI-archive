//! C-52 — shared types for the Dafny verification spike.
//!
//! Per arXiv:2604.22601 Section 3:
//!   - Temperature sweep: {0.0, 0.2, 0.4, 0.6, 0.8} (5 points).
//!   - verify@k reported for k in {1, 3, 5}.
//!   - The paper's seven evaluated models include Gemma 4-31B as the
//!     top performer (RQ3a 90.91% verify@5; RQ3b-tied 81.82%). Per
//!     memory rule `feedback_no_gemma4.md` ELAI is BLOCKED from
//!     integrating Gemma 4. Substitutes from the paper's evaluation
//!     set: Codestral-22B-v0.1, GPT-OSS-120B, Qwen 3 Coder 30B,
//!     Qwen 3.5-9B, GPT-OSS-20B, Qwen 3.6-35B-A3B. The current ELAI
//!     sidecar default (Qwen 2.5 Coder 7B) is NOT in the paper's
//!     evaluation set; expected verify@5 below 45% on the signature
//!     tier — see workflow/reasoning/C-52-decisions.md.

use serde::{Deserialize, Serialize};

/// Temperature sweep per arXiv:2604.22601 Section 3.
///
/// Five sample points. The paper's per-model + per-tier optima are
/// distributed across this range; the optimum varies (Gemma 4-31B
/// peaked at T=0.2 / 0.6 for RQ3a, T=0.6 / 0.8 for RQ3b). Stored as
/// `[f32; 5]` so call sites can iterate by index without a heap
/// allocation. Const so the contract test G4 can assert membership
/// without parsing.
pub const TEMPERATURE_SWEEP: [f32; 5] = [0.0, 0.2, 0.4, 0.6, 0.8];

/// verify@k reporting tuple per paper Section 3. Three sample
/// points; k=5 is the strongest signal.
pub const VERIFY_K_VALUES: [u32; 3] = [1, 3, 5];

/// Paper's `Gemma 4-31B` is BLOCKED from ELAI sidecar integration
/// per memory rule `feedback_no_gemma4.md`. This constant is the
/// canonical block-list entry the contract test G6 asserts against.
pub const GEMMA4_BLOCKED: &str = "Gemma 4-31B";

/// Non-Gemma substitutes from the paper's seven-model evaluation
/// set. Reported verify@5 numbers in parentheses are RQ2 (signature
/// tier, the strongest non-self-healing tier):
///   - Qwen 3.5-9B (72.73%, T=0.8) — paper's strongest RQ2 non-Gemma
///   - GPT-OSS-120B (63.64%)
///   - Codestral-22B-v0.1 (63.64%)
///   - Qwen 3.6-35B-A3B (63.64%) MoE
///   - Qwen 3 Coder 30B (45.45%)
///   - GPT-OSS-20B (paper-reported)
///
/// ELAI's current sidecar default (Qwen 2.5 Coder 7B) is NOT in the
/// paper's evaluation set; expected verify@5 is below 45% per
/// `C-52-decisions.md`. The substitute list is the recommended
/// upgrade path when remote GPU is available.
pub const ELAI_SUBSTITUTE_MODELS: [&str; 6] = [
    "Qwen 3.5-9B",
    "GPT-OSS-120B",
    "Codestral-22B-v0.1",
    "Qwen 3.6-35B-A3B",
    "Qwen 3 Coder 30B",
    "GPT-OSS-20B",
];

/// Per-temperature sweep wrapper. Carries the five canonical sample
/// points as a value type so callers can serialize the configuration
/// alongside per-run results without recomputing the constant. Stored
/// as `[f32; 5]` (not `Vec<f32>`) so the type is `Copy` and the wire
/// shape is stable across runs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemperatureSweep {
    pub points: [f32; 5],
}

impl TemperatureSweep {
    /// Build the canonical sweep matching paper Section 3.
    pub const fn canonical() -> Self {
        Self {
            points: TEMPERATURE_SWEEP,
        }
    }

    /// Whether `t` is in the canonical sweep. Uses an epsilon
    /// comparison so callers that pass floats from JSON or CLI args
    /// round-trip cleanly.
    pub fn contains(&self, t: f32) -> bool {
        const EPS: f32 = 1e-6;
        self.points.iter().any(|p| (p - t).abs() < EPS)
    }
}

impl Default for TemperatureSweep {
    fn default() -> Self {
        Self::canonical()
    }
}

/// verify@k counter triple per paper Section 3.
///
/// Each field is a u32 counter (number of problems passing verify
/// at the corresponding k). Counters are AGGREGATE — the wire shape
/// matches the paper's headline-table cell shape (one cell per
/// (model, tier, k) tuple).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct VerifyAt {
    /// verify@1 — single-sample pass count.
    pub k_1: u32,
    /// verify@3 — at-least-one-of-three pass count.
    pub k_3: u32,
    /// verify@5 — at-least-one-of-five pass count.
    pub k_5: u32,
}

impl VerifyAt {
    /// Build a counter triple from raw counts.
    pub const fn new(k_1: u32, k_3: u32, k_5: u32) -> Self {
        Self { k_1, k_3, k_5 }
    }

    /// Whether k=5 (strongest signal per paper) is non-zero.
    pub fn has_any_k5(&self) -> bool {
        self.k_5 > 0
    }

    /// Monotonicity invariant: verify@1 <= verify@3 <= verify@5
    /// (more samples can only increase the at-least-one-pass count).
    pub fn is_monotonic(&self) -> bool {
        self.k_1 <= self.k_3 && self.k_3 <= self.k_5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_sweep_matches_paper_section_3() {
        let sweep = TemperatureSweep::canonical();
        // Five sample points per paper.
        assert_eq!(sweep.points.len(), 5);
        // Exact values per paper Section 3.
        assert!(sweep.contains(0.0));
        assert!(sweep.contains(0.2));
        assert!(sweep.contains(0.4));
        assert!(sweep.contains(0.6));
        assert!(sweep.contains(0.8));
        // Outside the sweep.
        assert!(!sweep.contains(1.0));
        assert!(!sweep.contains(0.5));
    }

    #[test]
    fn verify_k_values_match_paper() {
        assert_eq!(VERIFY_K_VALUES, [1u32, 3, 5]);
    }

    #[test]
    fn verify_at_monotonic_invariant() {
        let good = VerifyAt::new(3, 7, 10);
        assert!(good.is_monotonic());
        let bad = VerifyAt::new(10, 5, 3);
        assert!(!bad.is_monotonic());
    }

    #[test]
    fn gemma4_block_list_is_non_empty() {
        assert!(!GEMMA4_BLOCKED.is_empty());
        // Substitute list must NOT include Gemma 4.
        for m in ELAI_SUBSTITUTE_MODELS.iter() {
            assert!(
                !m.contains("Gemma 4"),
                "substitute list must not include Gemma 4 (memory:feedback_no_gemma4)"
            );
        }
    }
}
