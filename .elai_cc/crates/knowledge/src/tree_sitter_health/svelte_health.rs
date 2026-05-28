//! E-42 Tier-B — Svelte 5 runes detection + tree-sitter ERROR / MISSING
//! node counting.
//!
//! As of May 2026, no upstream `tree-sitter-svelte-ng` tag formally
//! supports Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`)
//! per the report's openreplay.com Svelte summer-2025 recap +
//! tree-sitter-grammars/tree-sitter-svelte issue tracker. This module
//! ships an honest "best-effort with health telemetry" path:
//!
//! 1. A regex pre-scan over the raw source for `$state(`, `$derived(`,
//!    `$effect(`, `$props(`. Cheap and parser-free; tolerates Svelte 4
//!    stores (`$store`) because those never carry the `(` open paren in
//!    declaration position.
//! 2. After tree-sitter parsing, count `ERROR` + `MISSING` nodes via a
//!    walker. The ratio over total nodes is the health signal.
//! 3. If runes are detected AND the ERROR rate exceeds
//!    [`ERROR_RATE_THRESHOLD`], the file is classified as
//!    [`HealthClassification::FallbackRawText`] so the ingest pipeline
//!    can downgrade to raw-text chunking.
//!
//! Pairs with `SsbActionType::TreeSitterGrammarEvent` carrying
//! `TreeSitterGrammarEventKind::SvelteRunesParseHealth` (see
//! `elai_orchestrator::ssb`). Producers gate emission on
//! [`super::is_enabled`].

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

/// ERROR + MISSING node ratio above which a runes-bearing Svelte file is
/// downgraded to raw-text chunking. 5% per the May 2026 report's
/// composite-language guidance + tree-sitter issue #3625
/// (multi-level injection unsolved).
pub const ERROR_RATE_THRESHOLD: f32 = 0.05;

/// Cached compiled regex for runes detection. Compiled once at the
/// first call; not hot enough to merit a `once_cell::sync::Lazy`
/// dependency on top of `OnceLock` which is std.
fn runes_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        // Matches the Svelte 5 rune call-sites at declaration position.
        // `\$` is a literal $; the `\(` is the open paren that
        // disambiguates from Svelte 4 `$store` automatic-subscribe.
        // Tolerates whitespace before the paren (e.g. `$state (`).
        Regex::new(r"\$(?:state|derived|effect|props)\s*\(")
            .expect("runes regex literal MUST compile")
    })
}

/// Returns `true` if any Svelte 5 rune call-site appears in `source`.
///
/// `$store` references without a `(` (Svelte 4 store autosubscribe)
/// are NOT matched and do not false-positive this detector.
#[must_use]
pub fn detect_runes(source: &str) -> bool {
    runes_regex().is_match(source)
}

/// Count of ERROR + MISSING nodes in a parsed tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorHealth {
    /// Number of `ERROR` nodes (parser recovery points).
    pub error: usize,
    /// Number of `MISSING` nodes (auto-inserted by parser to recover).
    pub missing: usize,
    /// Total node count under the root (inclusive of internal nodes).
    pub total: usize,
}

impl ErrorHealth {
    /// Combined error + missing ratio over total node count. Returns 0.0
    /// when total is 0 (defensive — empty tree).
    #[must_use]
    pub fn ratio(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let bad = self.error.saturating_add(self.missing);
        // Cast via u32 narrowing to avoid `as` on potentially-large
        // usize values; tree-sitter trees beyond u32::MAX nodes are
        // not feasible in any practical source file.
        let bad_f = u32::try_from(bad).unwrap_or(u32::MAX) as f32;
        let total_f = u32::try_from(self.total).unwrap_or(u32::MAX) as f32;
        bad_f / total_f
    }
}

/// Walk the tree and count ERROR + MISSING + total nodes.
///
/// Uses `walk()` cursor traversal so the cost is O(n) in node count.
/// Both `ERROR` and `MISSING` are first-class node kinds in
/// tree-sitter; `node.is_error()` covers the former and
/// `node.is_missing()` covers the latter (per the tree-sitter Rust
/// binding at `tree-sitter-0.26.8/binding_rust/lib.rs`).
#[must_use]
pub fn compute_health(tree: &Tree) -> ErrorHealth {
    let mut error = 0usize;
    let mut missing = 0usize;
    let mut total = 0usize;

    let mut cursor = tree.walk();
    walk_recursive(&mut cursor, &mut error, &mut missing, &mut total);

    ErrorHealth { error, missing, total }
}

fn walk_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    error: &mut usize,
    missing: &mut usize,
    total: &mut usize,
) {
    loop {
        let node = cursor.node();
        *total = total.saturating_add(1);
        if node.is_error() {
            *error = error.saturating_add(1);
        }
        if node.is_missing() {
            *missing = missing.saturating_add(1);
        }
        if cursor.goto_first_child() {
            walk_recursive(cursor, error, missing, total);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

/// Classification verdict for a parsed Svelte file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthClassification {
    /// Parse health is acceptable; structural symbol extraction proceeds
    /// as normal.
    Healthy,
    /// Runes detected + ERROR rate above the threshold. The ingest
    /// pipeline should downgrade this file to raw-text chunking.
    FallbackRawText,
    /// No runes detected; the ERROR rate may still be non-zero but the
    /// file parses well enough for the standard pipeline.
    NoRunesHealthy,
    /// No runes detected but the parser produced many ERROR nodes —
    /// likely an unrelated parse failure (e.g. embedded
    /// language-server syntax). Surfaces as a separate signal so the
    /// dashboard can distinguish runes-only fallbacks from generic
    /// parse breakage.
    NoRunesDegraded,
}

/// Combine the runes flag + health ratio + threshold into a
/// classification verdict.
#[must_use]
pub fn classify(runes_detected: bool, health: &ErrorHealth) -> HealthClassification {
    let above_threshold = health.ratio() > ERROR_RATE_THRESHOLD;
    match (runes_detected, above_threshold) {
        (true, true) => HealthClassification::FallbackRawText,
        (true, false) => HealthClassification::Healthy,
        (false, false) => HealthClassification::NoRunesHealthy,
        (false, true) => HealthClassification::NoRunesDegraded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_state_rune() {
        assert!(detect_runes("let x = $state(0);"));
    }

    #[test]
    fn detects_derived_rune() {
        assert!(detect_runes("const total = $derived(items.length);"));
    }

    #[test]
    fn detects_effect_rune() {
        assert!(detect_runes("$effect(() => console.log(x));"));
    }

    #[test]
    fn detects_props_rune() {
        assert!(detect_runes("let { name } = $props();"));
    }

    #[test]
    fn detects_rune_with_whitespace_before_paren() {
        assert!(detect_runes("let x = $state ( 0 );"));
    }

    #[test]
    fn does_not_false_positive_on_svelte_4_store() {
        // `$store` without the `(` is Svelte 4 autosubscribe; must NOT
        // be detected as a rune.
        assert!(!detect_runes("console.log($store);"));
        assert!(!detect_runes("if ($count > 0) { return; }"));
    }

    #[test]
    fn does_not_false_positive_on_plain_dollar_or_jquery() {
        assert!(!detect_runes("let price = $100;"));
        assert!(!detect_runes("$(document).ready(...);"));
    }

    #[test]
    fn ratio_handles_empty_tree() {
        let h = ErrorHealth { error: 0, missing: 0, total: 0 };
        assert_eq!(h.ratio(), 0.0);
    }

    #[test]
    fn ratio_combines_error_and_missing() {
        let h = ErrorHealth { error: 5, missing: 5, total: 100 };
        // 10 / 100 = 0.1
        assert!((h.ratio() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn classify_runes_above_threshold_falls_back() {
        let h = ErrorHealth { error: 10, missing: 0, total: 100 }; // 10% > 5%
        assert_eq!(
            classify(true, &h),
            HealthClassification::FallbackRawText
        );
    }

    #[test]
    fn classify_runes_below_threshold_stays_healthy() {
        let h = ErrorHealth { error: 1, missing: 0, total: 100 }; // 1% < 5%
        assert_eq!(classify(true, &h), HealthClassification::Healthy);
    }

    #[test]
    fn classify_no_runes_healthy_when_clean() {
        let h = ErrorHealth { error: 0, missing: 0, total: 50 };
        assert_eq!(
            classify(false, &h),
            HealthClassification::NoRunesHealthy
        );
    }

    #[test]
    fn classify_no_runes_degraded_when_above_threshold() {
        let h = ErrorHealth { error: 10, missing: 5, total: 100 }; // 15%
        assert_eq!(
            classify(false, &h),
            HealthClassification::NoRunesDegraded
        );
    }

    #[test]
    fn compute_health_on_real_rust_tree() {
        // Indirect smoke test: parse a known-good snippet with the
        // tree-sitter-rust grammar and verify zero ERROR / MISSING.
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("rust grammar must load");
        let tree = parser
            .parse("fn main() {}", None)
            .expect("must parse");
        let h = compute_health(&tree);
        assert_eq!(h.error, 0);
        assert_eq!(h.missing, 0);
        assert!(h.total > 0);
    }
}
