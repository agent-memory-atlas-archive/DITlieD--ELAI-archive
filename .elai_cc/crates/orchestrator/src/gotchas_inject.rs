//! E-8 task 5: ETGPO-style top-3 gotchas injection.
//!
//! Per arXiv:2602.00997 (ETGPO): injecting an error-taxonomy / pitfalls
//! block into the optimizer's textual input reduces optimization-phase
//! token cost to ~1/3 of the baseline while preserving the converged
//! accuracy. ELAI's `gotchas.md` (workflow/memory-bank/gotchas.md) plus
//! the `failure_taxonomy.py` classifier (in `.claude/tools/`) play the
//! ETGPO error-taxonomy role.
//!
//! This module owns:
//!   * [`GotchaTag`] -- closed taxonomy of the 8 stable categories the
//!     failure_taxonomy classifier emits, plus a free-form `Other`
//!     escape hatch.
//!   * [`GotchaSnapshot`] -- one row per category with a recency-
//!     decayed activity count.
//!   * [`top_gotchas`] -- query the in-memory snapshot store for the
//!     top-N tags by activity over a sliding window (default 30 days).
//!   * [`render_pitfalls_block`] -- format the top-N tags as a
//!     `Pitfalls:` block ready to splice into the optimizer's role-
//!     prompt input.
//!
//! No I/O, no async, no global state. The snapshot store is passed in
//! by the caller (typically `optimize_cmd`) which read it from the
//! orchestrator-side SSB stream + gotchas.md parse.
//!
//! No `.unwrap()` / `.expect()` outside test code.

use serde::{Deserialize, Serialize};

/// 8 stable failure categories used by the `failure_taxonomy.py`
/// classifier (.claude/tools/failure_taxonomy.py), plus `Other` for
/// uncategorised hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GotchaTag {
    /// Build-system / compile-error category.
    BuildError,
    /// Type-error / signature-mismatch category.
    TypeError,
    /// Borrow / lifetime / ownership-conflict category.
    BorrowChecker,
    /// Async / await / Send-bound category.
    AsyncSendBound,
    /// FFI / native-call category.
    FfiBoundary,
    /// Schema-version / serde-rename / golden-file drift category.
    SchemaDrift,
    /// Test-fixture / golden-file mismatch category.
    FixtureDrift,
    /// Anti-slopsquatting / dep-version stale category.
    DepVersionStale,
    /// Free-form escape.
    Other,
}

impl GotchaTag {
    /// Snake-case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BuildError => "build_error",
            Self::TypeError => "type_error",
            Self::BorrowChecker => "borrow_checker",
            Self::AsyncSendBound => "async_send_bound",
            Self::FfiBoundary => "ffi_boundary",
            Self::SchemaDrift => "schema_drift",
            Self::FixtureDrift => "fixture_drift",
            Self::DepVersionStale => "dep_version_stale",
            Self::Other => "other",
        }
    }

    /// All known categories (excluding `Other`).
    #[must_use]
    pub const fn known() -> [Self; 8] {
        [
            Self::BuildError,
            Self::TypeError,
            Self::BorrowChecker,
            Self::AsyncSendBound,
            Self::FfiBoundary,
            Self::SchemaDrift,
            Self::FixtureDrift,
            Self::DepVersionStale,
        ]
    }
}

/// One row in the gotchas snapshot store.
///
/// `activity_score` is unitless -- the caller supplies an already-
/// recency-decayed value (e.g. exp(-age_days / 30) × hit_count) so this
/// module stays pure. Higher score = more relevant pitfall to surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GotchaSnapshot {
    pub tag: GotchaTag,
    /// Recency-decayed activity score over the configured window.
    pub activity_score: f64,
    /// Short human-readable label (e.g. "blake3 vs blake2 — anti-stale").
    pub label: String,
}

/// Sliding-window length the snapshot was computed over. Carried as
/// data (not a constant) so the dashboard panel can render the actual
/// window without re-reading config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowDays(pub u32);

/// Default sliding-window length when callers do not override it.
pub const DEFAULT_WINDOW_DAYS: WindowDays = WindowDays(30);

/// Return the top-N gotchas by `activity_score` from the supplied
/// snapshot store. N defaults to 3 per ETGPO's "top-K with K=3"
/// recommendation; callers may request more.
///
/// Total / pure / deterministic. Returns at most `n` entries (fewer if
/// the store is smaller). Stable secondary sort by tag wire-string so
/// ties produce reproducible output.
#[must_use]
pub fn top_gotchas(store: &[GotchaSnapshot], n: usize) -> Vec<GotchaSnapshot> {
    let mut sorted: Vec<GotchaSnapshot> = store.to_vec();
    sorted.sort_by(|a, b| {
        // Higher activity_score first; on tie, lexicographic tag.
        b.activity_score
            .partial_cmp(&a.activity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.tag.as_str().cmp(b.tag.as_str()))
    });
    sorted.into_iter().take(n).collect()
}

/// Format a `Pitfalls:` block ready to splice into the optimizer's
/// role-prompt input. Each pitfall is rendered as
/// `- <tag>: <label>` on its own line.
///
/// Returns the empty string when `pitfalls` is empty so the caller can
/// safely concatenate without worrying about a stray heading.
#[must_use]
pub fn render_pitfalls_block(pitfalls: &[GotchaSnapshot]) -> String {
    if pitfalls.is_empty() {
        return String::new();
    }
    let mut out = String::from("Pitfalls (top-");
    out.push_str(&pitfalls.len().to_string());
    out.push_str(", recency-decayed activity, ETGPO arXiv:2602.00997):\n");
    for p in pitfalls {
        out.push_str("- ");
        out.push_str(p.tag.as_str());
        out.push_str(": ");
        out.push_str(&p.label);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(tag: GotchaTag, score: f64, label: &str) -> GotchaSnapshot {
        GotchaSnapshot {
            tag,
            activity_score: score,
            label: label.into(),
        }
    }

    #[test]
    fn top_gotchas_returns_at_most_n() {
        let store = vec![
            snap(GotchaTag::BuildError, 5.0, "rustc E0277"),
            snap(GotchaTag::TypeError, 3.0, "wrong signature"),
            snap(GotchaTag::SchemaDrift, 4.0, "golden drift"),
        ];
        let top = top_gotchas(&store, 2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn top_gotchas_ordered_descending_by_score() {
        let store = vec![
            snap(GotchaTag::BuildError, 1.0, "x"),
            snap(GotchaTag::TypeError, 5.0, "y"),
            snap(GotchaTag::SchemaDrift, 3.0, "z"),
        ];
        let top = top_gotchas(&store, 3);
        assert_eq!(top[0].tag, GotchaTag::TypeError);
        assert_eq!(top[1].tag, GotchaTag::SchemaDrift);
        assert_eq!(top[2].tag, GotchaTag::BuildError);
    }

    #[test]
    fn top_gotchas_stable_tie_by_tag_lex() {
        let store = vec![
            snap(GotchaTag::TypeError, 2.0, "a"),
            snap(GotchaTag::BuildError, 2.0, "b"),
        ];
        let top = top_gotchas(&store, 2);
        // build_error < type_error lexicographically.
        assert_eq!(top[0].tag, GotchaTag::BuildError);
        assert_eq!(top[1].tag, GotchaTag::TypeError);
    }

    #[test]
    fn render_pitfalls_block_empty_returns_empty_string() {
        let out = render_pitfalls_block(&[]);
        assert!(out.is_empty());
    }

    #[test]
    fn render_pitfalls_block_lists_each_tag() {
        let store = vec![
            snap(GotchaTag::BuildError, 5.0, "rustc E0277"),
            snap(GotchaTag::SchemaDrift, 4.0, "golden drift"),
        ];
        let out = render_pitfalls_block(&store);
        assert!(out.contains("Pitfalls"));
        assert!(out.contains("build_error: rustc E0277"));
        assert!(out.contains("schema_drift: golden drift"));
        assert!(out.contains("ETGPO arXiv:2602.00997"));
    }

    #[test]
    fn known_tags_have_eight_entries() {
        assert_eq!(GotchaTag::known().len(), 8);
    }

    #[test]
    fn empty_store_returns_empty_top() {
        assert!(top_gotchas(&[], 3).is_empty());
    }
}
