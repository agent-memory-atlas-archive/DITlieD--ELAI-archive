//! E-36 — document + vision RAG substrate.
//!
//! Producer crate surface for the
//! [`crate::ssb::SsbActionType::DocumentVisionRagEvent`] single-variant
//! Pattern-2 (34th application). Wire-dark by default — every emit site
//! checks [`document_vision_rag_emit_enabled`] (gated by
//! `ELAI_DOCUMENT_VISION_RAG_ENABLE=1`).
//!
//! Submodules:
//!   * [`visual_verifier`] — VLM-backed `VisualArtifactCheck` dispatch
//!     (10th `VerificationMethod` variant; HARD downweight rule for
//!     `ArtifactReliability::VlmCaption` verdicts).
//!   * [`policy`] — per-project enable gate + cloud-fallback mode check
//!     + storage-budget surface. Pure-compute; the producer call path
//!     wires `vision_rag_policy_decision` /
//!     `doc_parser_cloud_fallback_blocked_by_mode` rows.
//!
//! Plan: docs/thefinalglance/E-plan/E-36_document_vision_rag.md.
//! Decisions: workflow/reasoning/E-36-decisions.md (Tier-A + Tier-B).

pub mod policy;
pub mod visual_verifier;

/// Environment-variable name that gates production emission of
/// [`crate::ssb::SsbActionType::DocumentVisionRagEvent`] rows. Wire-dark
/// default: emission silently skipped until callers set this to a
/// truthy value (`"1"` / `"true"` / `"yes"`).
///
/// Mirrors `ELAI_PEER_SYSTEMS_ENABLE` / `ELAI_LOCAL_CLOUD_ROUTING_ENABLE`
/// / `ELAI_SKILL_SYSTEM_ENABLE` precedent.
pub const ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV: &str = "ELAI_DOCUMENT_VISION_RAG_ENABLE";

/// True when the wire-dark gate is ON. Returns false when unset / empty
/// / non-truthy value.
#[must_use]
pub fn document_vision_rag_emit_enabled() -> bool {
    match std::env::var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes")
        }
        Err(_) => false,
    }
}

/// Companion negation for callers that prefer the "skip if disabled"
/// idiom.
#[must_use]
pub fn document_vision_rag_emit_disabled() -> bool {
    !document_vision_rag_emit_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Cargo runs unit tests within one crate in parallel; the three env-
    // var tests below mutate the SAME process-global env var and race
    // with each other. Serialize env-mutating tests via this module-
    // level mutex. .unwrap() acceptable in #[cfg(test)] per CLAUDE.md.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn gate_default_is_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests set/clear under the ENV_LOCK serialization above.
        unsafe {
            std::env::remove_var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV);
        }
        assert!(document_vision_rag_emit_disabled());
        assert!(!document_vision_rag_emit_enabled());
    }

    #[test]
    fn gate_one_enables() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV, "1");
        }
        assert!(document_vision_rag_emit_enabled());
        unsafe {
            std::env::remove_var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV);
        }
    }

    #[test]
    fn gate_false_disables() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV, "false");
        }
        assert!(document_vision_rag_emit_disabled());
        unsafe {
            std::env::remove_var(ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV);
        }
    }
}
