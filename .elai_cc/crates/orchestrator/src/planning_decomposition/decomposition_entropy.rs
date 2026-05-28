//! Shannon-entropy decomposition stop rule.
//!
//! Plan task 5 of E-46. At each Concept / Phase decomposition fork compute
//! Shannon entropy `H = -sum(p_i * log2(p_i))` over the union of children
//! acceptance-criteria token sets (with `p_i` = frequency of token i
//! across the concatenated children tokens). Also compute parent-token
//! entropy `H_parent`. If `H_children / H_parent < ENTROPY_RATIO_THRESHOLD`,
//! the decomposition is flagged as low-entropy — children carry too little
//! information differentiation versus the parent goal, which the
//! multi-agent-failure-taxonomy paper (arXiv:2503.13657) categorises as
//! "improper task decomposition — too granular, too broad, not
//! serializable".
//!
//! Behaviour modes per plan §45:
//!   - `strict_mode = true`: refuse the decomposition (caller treats the
//!     parent as atomic; no children recursion).
//!   - `strict_mode = false` (default): allow but emit a `WARN`-shaped
//!     SSB row via the producers module so the per-level auditor surfaces
//!     it without blocking.
//!
//! Open-question floor: per plan §212, parents with `<=20` tokens after
//! tokenization yield unstable ratios. The
//! [`compute_entropy_ratio`] free function returns `EntropyVerdict::Skipped`
//! in that regime — the caller treats it as a no-op, not a low-entropy
//! signal.
//!
//! Producer-pure: this module emits no SSB rows. The caller in
//! [`crate::planning_decomposition::producers::emit_low_entropy_decomposition`]
//! handles the SSB write under the
//! [`crate::planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV`] gate.

use serde::{Deserialize, Serialize};

/// Default Shannon-entropy ratio threshold below which a decomposition is
/// flagged low-entropy. Plan task 5 §45 default = 0.6; surface in
/// `elai_cc.toml` `[planning.entropy] entropy_ratio_threshold` to
/// override.
pub const ENTROPY_RATIO_THRESHOLD: f32 = 0.6;

/// Minimum parent token count below which the entropy rule does not
/// fire (per plan open-question §212 — small parents yield unstable
/// ratios). Surface in `elai_cc.toml` `[planning.entropy] min_parent_tokens`.
pub const MIN_PARENT_TOKENS: usize = 20;

/// Closed sum over entropy verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntropyVerdict {
    /// Decomposition entropy ratio satisfies the threshold. `ratio` is the
    /// computed `H_children / H_parent`.
    Ok { ratio: f32, h_parent: f32, h_children: f32 },
    /// Decomposition entropy below threshold. Caller emits the
    /// `low_entropy_decomposition` SSB row + applies strict-mode policy.
    LowEntropy { ratio: f32, h_parent: f32, h_children: f32, threshold: f32 },
    /// Parent token count too small for stable ratio computation; rule
    /// is a no-op for this fork.
    Skipped { parent_token_count: usize, floor: usize },
}

impl EntropyVerdict {
    /// True iff this verdict indicates a low-entropy decomposition.
    #[must_use]
    pub fn is_low_entropy(&self) -> bool {
        matches!(self, EntropyVerdict::LowEntropy { .. })
    }

    /// Snake-case wire label for SSB / log fields.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok { .. } => "ok",
            Self::LowEntropy { .. } => "low_entropy",
            Self::Skipped { .. } => "skipped",
        }
    }
}

/// Tokenize a string into lowercase ascii-alphanumeric tokens of length
/// `>= 2`. Stop-word filtering is OPTIONAL and applied by the caller via
/// [`tokenize_no_stopwords`] when desired (the goal-drift module passes a
/// stop-word list; the entropy rule keeps stop words to avoid biasing
/// `p_i` distributions on already-short acceptance-criteria strings).
#[must_use]
pub fn tokenize_simple(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            current.push(c.to_ascii_lowercase());
        } else if !current.is_empty() {
            if current.len() >= 2 {
                out.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if current.len() >= 2 {
        out.push(current);
    }
    out
}

/// Shannon entropy `H = -sum(p_i * log2(p_i))` over a token bag.
/// Returns `0.0` when the token bag is empty (no entropy = no
/// information). Anti-stale-training: no `f64::log2` panic — we filter
/// zero counts before division.
#[must_use]
pub fn shannon_entropy(tokens: &[String]) -> f32 {
    if tokens.is_empty() {
        return 0.0;
    }
    let total = tokens.len() as f32;
    let mut counts: rustc_hash::FxHashMap<&str, u32> = rustc_hash::FxHashMap::default();
    for t in tokens {
        *counts.entry(t.as_str()).or_insert(0) += 1;
    }
    let mut h = 0.0_f32;
    for (_tok, c) in counts.iter() {
        let p = (*c as f32) / total;
        if p > 0.0 {
            h -= p * p.log2();
        }
    }
    h
}

/// Compute the entropy ratio `H_children / H_parent` for a decomposition
/// fork. The children token bag is the concatenation of every child's
/// tokens (token frequency carries the information distribution); the
/// parent token bag is the parent's own acceptance-criteria tokens.
///
/// Returns [`EntropyVerdict::Skipped`] when `H_parent` < small epsilon
/// (avoids division-by-zero) or when the parent token count is below
/// [`MIN_PARENT_TOKENS`].
///
/// Caller is responsible for emitting the SSB row when the verdict is
/// [`EntropyVerdict::LowEntropy`]. The producer
/// [`crate::planning_decomposition::producers::emit_low_entropy_decomposition`]
/// is the single canonical emit site.
#[must_use]
pub fn compute_entropy_ratio(
    parent_tokens: &[String],
    children_tokens: &[Vec<String>],
    threshold: f32,
) -> EntropyVerdict {
    if parent_tokens.len() < MIN_PARENT_TOKENS {
        return EntropyVerdict::Skipped {
            parent_token_count: parent_tokens.len(),
            floor: MIN_PARENT_TOKENS,
        };
    }
    let h_parent = shannon_entropy(parent_tokens);
    if h_parent < f32::EPSILON {
        // Parent has zero entropy (all tokens identical) — no meaningful
        // ratio possible; treat as Skipped rather than divide-by-zero.
        return EntropyVerdict::Skipped {
            parent_token_count: parent_tokens.len(),
            floor: MIN_PARENT_TOKENS,
        };
    }
    let concat: Vec<String> = children_tokens
        .iter()
        .flat_map(|child| child.iter().cloned())
        .collect();
    let h_children = shannon_entropy(&concat);
    let ratio = h_children / h_parent;
    let clamped_threshold = threshold.clamp(0.0, 10.0);
    if ratio < clamped_threshold {
        EntropyVerdict::LowEntropy {
            ratio,
            h_parent,
            h_children,
            threshold: clamped_threshold,
        }
    } else {
        EntropyVerdict::Ok {
            ratio,
            h_parent,
            h_children,
        }
    }
}

/// Convenience wrapper accepting raw strings; tokenizes via
/// [`tokenize_simple`] and forwards to [`compute_entropy_ratio`].
#[must_use]
pub fn compute_entropy_ratio_from_strings(
    parent_str: &str,
    children_strs: &[&str],
    threshold: f32,
) -> EntropyVerdict {
    let parent_tokens = tokenize_simple(parent_str);
    let children_tokens: Vec<Vec<String>> =
        children_strs.iter().map(|s| tokenize_simple(s)).collect();
    compute_entropy_ratio(&parent_tokens, &children_tokens, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_parent_tokens() -> Vec<String> {
        // 24 unique-ish tokens to satisfy MIN_PARENT_TOKENS=20.
        vec![
            "user", "submits", "form", "captures", "email", "name", "phone",
            "validates", "schema", "stores", "database", "indexed", "tested",
            "endpoint", "secured", "rate", "limited", "audit", "logged",
            "monitored", "deployable", "observable", "reliable", "verified",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    #[test]
    fn tokenize_simple_lowercases_and_filters() {
        let toks = tokenize_simple("User Submits Form. Validates SCHEMA!");
        assert_eq!(toks, vec!["user", "submits", "form", "validates", "schema"]);
    }

    #[test]
    fn tokenize_simple_drops_short() {
        let toks = tokenize_simple("a bc def, x! YZ");
        // "a" / "x" / "yz" — "a" (len=1) drops, "x" (len=1) drops, "yz" (len=2) keeps
        assert_eq!(toks, vec!["bc", "def", "yz"]);
    }

    #[test]
    fn shannon_entropy_empty_is_zero() {
        assert_eq!(shannon_entropy(&[]), 0.0);
    }

    #[test]
    fn shannon_entropy_uniform_max() {
        let toks: Vec<String> = (0..8).map(|i| format!("t{}", i)).collect();
        let h = shannon_entropy(&toks);
        // 8 uniform tokens => H = log2(8) = 3.0 bits
        assert!((h - 3.0).abs() < 1e-5, "got h={}", h);
    }

    #[test]
    fn shannon_entropy_all_same_is_zero() {
        let toks: Vec<String> = (0..8).map(|_| "same".to_string()).collect();
        let h = shannon_entropy(&toks);
        assert!(h < 1e-5, "got h={}", h);
    }

    #[test]
    fn compute_entropy_ratio_skipped_on_short_parent() {
        let parent: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let children: Vec<Vec<String>> = vec![];
        let verdict = compute_entropy_ratio(&parent, &children, ENTROPY_RATIO_THRESHOLD);
        match verdict {
            EntropyVerdict::Skipped { parent_token_count, floor } => {
                assert_eq!(parent_token_count, 3);
                assert_eq!(floor, MIN_PARENT_TOKENS);
            }
            _ => panic!("expected Skipped on short parent"),
        }
    }

    #[test]
    fn compute_entropy_ratio_ok_when_children_diverse() {
        let parent = long_parent_tokens();
        // children mirror the parent token distribution — high entropy ratio
        let child_a: Vec<String> =
            ["user", "submits", "form", "validates", "schema", "stores", "database"]
                .into_iter()
                .map(String::from)
                .collect();
        let child_b: Vec<String> =
            ["tested", "endpoint", "secured", "rate", "limited", "audit", "logged"]
                .into_iter()
                .map(String::from)
                .collect();
        let verdict = compute_entropy_ratio(&parent, &[child_a, child_b], ENTROPY_RATIO_THRESHOLD);
        assert!(matches!(verdict, EntropyVerdict::Ok { .. }), "got {:?}", verdict);
        assert!(!verdict.is_low_entropy());
    }

    #[test]
    fn compute_entropy_ratio_low_when_children_collapsed() {
        let parent = long_parent_tokens();
        // all children identical -> zero entropy in concat -> ratio = 0
        let child_a: Vec<String> = (0..10).map(|_| "same".to_string()).collect();
        let child_b: Vec<String> = (0..10).map(|_| "same".to_string()).collect();
        let verdict = compute_entropy_ratio(&parent, &[child_a, child_b], ENTROPY_RATIO_THRESHOLD);
        match verdict {
            EntropyVerdict::LowEntropy { ratio, threshold, .. } => {
                assert!(ratio < threshold, "ratio={} thr={}", ratio, threshold);
            }
            _ => panic!("expected LowEntropy, got {:?}", verdict),
        }
        assert!(verdict.is_low_entropy());
    }

    #[test]
    fn compute_entropy_ratio_threshold_clamped() {
        // negative threshold -> clamped to 0 -> Ok unless concat is empty
        let parent = long_parent_tokens();
        let child_a: Vec<String> = parent.clone();
        let verdict = compute_entropy_ratio(&parent, &[child_a], -1.0);
        assert!(matches!(verdict, EntropyVerdict::Ok { .. }));
    }

    #[test]
    fn compute_entropy_ratio_from_strings_low_when_repeated() {
        let parent_str =
            "user submits form captures email name phone validates schema stores database \
             indexed tested endpoint secured rate limited audit logged monitored \
             deployable observable reliable verified";
        let children = ["same same same same same same same same same", "same same same same"];
        let verdict =
            compute_entropy_ratio_from_strings(parent_str, &children, ENTROPY_RATIO_THRESHOLD);
        assert!(verdict.is_low_entropy(), "got {:?}", verdict);
    }
}
