//! Honest labeling for degraded retrieval context.

/// Marker attached when retrieval is operating below the full hybrid mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextIncompleteMarker {
    /// Canonical mode label (for example, `BM25_ONLY`).
    pub mode: String,
    /// Always true: degraded context raises audit strictness, never blocks.
    pub raise_audit_strictness: bool,
}

/// Return an honesty marker for a recognized non-hybrid retrieval mode.
///
/// Empty and unknown labels are intentionally ignored rather than guessed.
pub fn stamp_context_incomplete(mode: &str) -> Option<ContextIncompleteMarker> {
    let canonical = mode.trim().to_ascii_uppercase();
    match canonical.as_str() {
        "CODE_ONLY" | "TEXT_ONLY" | "SPARSE_ONLY" | "BM25_ONLY" => {
            Some(ContextIncompleteMarker {
                mode: canonical,
                raise_audit_strictness: true,
            })
        }
        "HYBRID" | "" => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::stamp_context_incomplete;

    #[test]
    fn hybrid_is_not_incomplete() {
        assert_eq!(stamp_context_incomplete("HYBRID"), None);
    }

    #[test]
    fn bm25_variants_raise_strictness() {
        for mode in ["BM25_ONLY", "bm25_only"] {
            let marker = stamp_context_incomplete(mode).expect("recognized degraded mode");
            assert_eq!(marker.mode, "BM25_ONLY");
            assert!(marker.raise_audit_strictness);
        }
    }

    #[test]
    fn sparse_only_is_incomplete() {
        let marker = stamp_context_incomplete("SPARSE_ONLY").expect("recognized degraded mode");
        assert_eq!(marker.mode, "SPARSE_ONLY");
        assert!(marker.raise_audit_strictness);
    }

    #[test]
    fn empty_is_not_incomplete() {
        assert_eq!(stamp_context_incomplete(""), None);
    }

    #[test]
    fn notifier_reason_append_contract() {
        let reason = "vector model unavailable";
        let marker = stamp_context_incomplete("BM25_ONLY");
        let rendered = if marker.is_some() {
            format!("{reason}; context-incomplete raise_audit_strictness=1")
        } else {
            reason.to_owned()
        };
        assert_eq!(rendered, "vector model unavailable; context-incomplete raise_audit_strictness=1");
    }
}
