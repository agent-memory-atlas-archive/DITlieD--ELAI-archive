//! E-42 Tier-B — large-file ceiling guard for tree-sitter ingest.
//!
//! tree-sitter is O(n) memory + ~1.2s for 1.6 MB JSON
//! (tree-sitter/tree-sitter#1277). Above 2 MiB the parse time becomes
//! disruptive to the index loop. The DKG ingest path consults
//! [`should_skip_for_size`] before invoking the parser; oversize files
//! get chunked as raw text instead and the
//! `SsbActionType::TreeSitterGrammarEvent` row carrying
//! `TreeSitterGrammarEventKind::TreeSitterSkippedLargeFile` is
//! emitted for observability.
//!
//! Pairs with `SsbActionType::TreeSitterGrammarEvent` (see
//! `elai_orchestrator::ssb`). Producers gate emission on
//! [`super::is_enabled`].

use serde::{Deserialize, Serialize};

/// File size ceiling for tree-sitter parse (2 MiB). Above this the
/// ingest pipeline downgrades to raw-text chunking and emits a
/// `tree_sitter_skipped_large_file` SSB row.
///
/// Named constant per CLAUDE.md "no magic numbers" rule.
pub const TREE_SITTER_LARGE_FILE_BYTES: u64 = 2 * 1024 * 1024;

/// One-shot record for an oversize file that was skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LargeFileSkip {
    /// Project-relative path of the skipped file.
    pub relative_path: String,
    /// File size in bytes (always > [`TREE_SITTER_LARGE_FILE_BYTES`]).
    pub byte_count: u64,
    /// The byte ceiling at the time of the decision (allows the
    /// dashboard to display the threshold as configured at decision
    /// time even if a future patch changes the const).
    pub threshold_bytes: u64,
}

impl LargeFileSkip {
    /// Build a record for a skipped file. Caller must already have
    /// verified `byte_count > TREE_SITTER_LARGE_FILE_BYTES`; this is
    /// asserted in debug builds.
    #[must_use]
    pub fn new(relative_path: impl Into<String>, byte_count: u64) -> Self {
        debug_assert!(
            byte_count > TREE_SITTER_LARGE_FILE_BYTES,
            "LargeFileSkip::new called with non-oversize byte count {byte_count}"
        );
        Self {
            relative_path: relative_path.into(),
            byte_count,
            threshold_bytes: TREE_SITTER_LARGE_FILE_BYTES,
        }
    }
}

/// Returns `true` when `byte_count` exceeds [`TREE_SITTER_LARGE_FILE_BYTES`].
/// The ingest pipeline calls this BEFORE invoking the parser, so an
/// oversize file never enters tree-sitter at all.
#[must_use]
pub const fn should_skip_for_size(byte_count: u64) -> bool {
    byte_count > TREE_SITTER_LARGE_FILE_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_is_exactly_2_mib() {
        assert_eq!(TREE_SITTER_LARGE_FILE_BYTES, 2_097_152);
    }

    #[test]
    fn should_skip_returns_false_at_threshold() {
        // Exactly 2 MiB is NOT skipped — only strictly greater.
        assert!(!should_skip_for_size(TREE_SITTER_LARGE_FILE_BYTES));
    }

    #[test]
    fn should_skip_returns_true_above_threshold() {
        assert!(should_skip_for_size(TREE_SITTER_LARGE_FILE_BYTES + 1));
        assert!(should_skip_for_size(10 * 1024 * 1024));
    }

    #[test]
    fn record_carries_threshold_at_decision_time() {
        let rec = LargeFileSkip::new("vendor/bundle.json", 3 * 1024 * 1024);
        assert_eq!(rec.threshold_bytes, TREE_SITTER_LARGE_FILE_BYTES);
        assert_eq!(rec.byte_count, 3 * 1024 * 1024);
        assert_eq!(rec.relative_path, "vendor/bundle.json");
    }
}
