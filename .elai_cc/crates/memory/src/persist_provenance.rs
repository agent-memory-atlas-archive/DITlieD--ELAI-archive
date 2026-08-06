//! J-93 LEG-C — G-1 trust labels that survive memory write/retrieve.
//!
//! Default-OFF kill switch: `ELAI_MEMORY_TAINT_PERSIST_ENABLE`. When OFF,
//! [`resolve_retrieved_taint`] returns `None` (no taint surfaced). When ON,
//! an unlabeled retrieved fact is treated as [`G1TrustLabel::Untrusted`],
//! never trusted-by-default (J-93 LEG-C residue rule).
use std::env;
use crate::{FactStore, MemoryError};

/// Environment variable name for the J-93 LEG-C default-OFF kill switch.
pub const ELAI_MEMORY_TAINT_PERSIST_ENABLE: &str = "ELAI_MEMORY_TAINT_PERSIST_ENABLE";

/// G-1 trust label persisted alongside a memory fact so capability_gate sees
/// the taint of a memory-retrieved fact in a new session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum G1TrustLabel {
    /// First-party trusted fact (user-confirmed or code-grounded).
    Trusted,
    /// Recallable: indirectly trusted, RAG retrieval can surface.
    Recallable,
    /// Untrusted: never injected without explicit confirmation.
    Untrusted,
}

impl G1TrustLabel {
    /// Serialize the label to its lowercase wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Recallable => "recallable",
            Self::Untrusted => "untrusted",
        }
    }

    /// Parse a label from its lowercase wire string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "trusted" => Some(Self::Trusted),
            "recallable" => Some(Self::Recallable),
            "untrusted" => Some(Self::Untrusted),
            _ => None,
        }
    }
}

/// Check whether the J-93 LEG-C kill switch is ON.
pub fn flag_enabled() -> bool {
    env::var(ELAI_MEMORY_TAINT_PERSIST_ENABLE)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Persist a G-1 trust label for `fact_id` in the side table.
pub fn persist_provenance(
    store: &FactStore,
    fact_id: &str,
    label: G1TrustLabel,
) -> Result<(), MemoryError> {
    store.persist_g1_taint(fact_id, label.as_str())
}

/// Resolve the G-1 taint label of a retrieved fact.
///
/// When the kill switch is OFF, returns `None` (no taint surfaced). When ON,
/// an unlabeled fact (ingested before this shipped) resolves to
/// [`G1TrustLabel::Untrusted`], never trusted-by-default.
pub fn resolve_retrieved_taint(
    store: &FactStore,
    fact_id: &str,
) -> Result<Option<G1TrustLabel>, MemoryError> {
    if !flag_enabled() {
        return Ok(None);
    }
    Ok(store
        .g1_taint(fact_id)?
        .and_then(|v| G1TrustLabel::from_str(&v))
        .or(Some(G1TrustLabel::Untrusted)))
}

/// Capability gate passthrough: the retrieved taint label IS the gate input.
pub fn capability_gate_on_retrieved_taint(label: G1TrustLabel) -> G1TrustLabel {
    label
}

#[cfg(test)]
mod j93_persist_provenance_tests {
    use super::*;

    #[test]
    fn labels_roundtrip() {
        let s = FactStore::open_in_memory().expect("store");
        persist_provenance(&s, "f", G1TrustLabel::Untrusted).expect("write");
        std::env::set_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE, "1");
        assert_eq!(
            resolve_retrieved_taint(&s, "f").expect("read"),
            Some(G1TrustLabel::Untrusted)
        );
        std::env::remove_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE);
    }

    #[test]
    fn capability_gate_preserves_untrusted() {
        assert_eq!(
            capability_gate_on_retrieved_taint(G1TrustLabel::Untrusted),
            G1TrustLabel::Untrusted
        );
    }

    #[test]
    fn flag_default_off() {
        std::env::remove_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE);
        assert!(!flag_enabled());
    }

    #[test]
    fn unlabeled_is_untrusted_when_flag_on() {
        let s = FactStore::open_in_memory().expect("store");
        std::env::set_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE, "1");
        // No persist_provenance call — fact has no stored label.
        assert_eq!(
            resolve_retrieved_taint(&s, "unlabeled").expect("read"),
            Some(G1TrustLabel::Untrusted)
        );
        std::env::remove_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE);
    }

    #[test]
    fn unlabeled_is_none_when_flag_off() {
        let s = FactStore::open_in_memory().expect("store");
        std::env::remove_var(ELAI_MEMORY_TAINT_PERSIST_ENABLE);
        // Flag OFF -> no taint surfaced at all.
        assert_eq!(
            resolve_retrieved_taint(&s, "unlabeled").expect("read"),
            None
        );
    }
}
