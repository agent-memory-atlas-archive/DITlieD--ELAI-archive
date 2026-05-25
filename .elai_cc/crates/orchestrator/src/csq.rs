//! E-53 — Counterfactual Safety Question (CSQ) classifier.
//!
//! Runtime Rust impl of the CSQ HARD RULE documented in CLAUDE.md
//! ("CSQ: before finalizing trading/ML logic, call
//! mcp_think(step_type=counterfactual): 'If [X] wrong, code will
//! [Y].' Add guard if none."). The CLAUDE.md prose rule remains
//! authoritative for the LLM-side reasoning loop; THIS module is
//! the runtime check the orchestrator can call against generated
//! code or path inputs to decide whether to dispatch a formal
//! verifier (Dafny / Z3 / Kani / Verus) or skip.
//!
//! GROUNDING:
//!   - CLAUDE.md CSQ rule (project root) — prose rule, authoritative
//!     for LLM behavior.
//!   - `task_runner.rs::BlockSource::CsqCritical` (line 12665) —
//!     pre-existing real surface, the canonical block-source sentinel
//!     emitted when a critical-path classification needs a guard.
//!     This module produces the classification; BlockSource consumes.
//!   - DRIFT-3 of E-53 context package: plan called for a
//!     `CounterfactualClass::{Critical, NonCritical}` enum that did
//!     NOT exist in the tree. The plan's enum is rejected; the real
//!     surface keeps `BlockSource::CsqCritical` as the sentinel, and
//!     this module's [`CounterfactualClass`] is the local classifier
//!     output that gets MAPPED to `BlockSource::CsqCritical` at the
//!     caller. Wiring direction: classify -> map to BlockSource.
//!
//! HEURISTIC SCOPE (E-53, lean ship):
//!   - Path-keyword heuristic only. Patterns covering auth / money /
//!     crypto / parser / untrusted-input territory get classified
//!     Risky; everything else Unknown. The classifier intentionally
//!     does NOT return CounterfactualSafe — only the formal verifier
//!     can promote a path to Safe. This preserves the
//!     "verifier-or-skip" semantics; the orchestrator never silently
//!     blesses a path.
//!   - False-positive rate on real codebases is UNKNOWN until live
//!     run (see workflow/reasoning/E-53-decisions.md unknowns block).
//!
//! WIRE-DARK posture: this module is consumed by
//! `crate::formal_verify::orchestrator::run` (E-53 pipeline). The
//! classifier output gates whether the pipeline emits a formal-
//! verifier dispatch SSB row (FormalVerifierCompleted) or short-
//! circuits with Inapplicable.

use serde::{Deserialize, Serialize};

/// Outcome of the heuristic CSQ classifier.
///
/// `CounterfactualSafe` is INTENTIONALLY UNREACHABLE from this
/// module's heuristics — only a formal verifier (Dafny / Z3 / Kani /
/// Verus) can promote a path. The orchestrator inspects the variant
/// before dispatching the formal-verify pipeline and chooses:
///   - `CounterfactualRisky` -> dispatch formal verifier
///   - `CounterfactualUnknown` -> skip (heuristic could not decide)
///   - `CounterfactualSafe` -> reserved for verifier-attested paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterfactualClass {
    /// Path matches a heuristic risk pattern (auth / money / crypto /
    /// parser / untrusted-input). Maps to
    /// `task_runner::BlockSource::CsqCritical` when the caller wants
    /// to surface a block on the BabyStepOutcome.
    CounterfactualRisky,
    /// Heuristic could not classify. Caller chooses whether to skip
    /// formal verification (lean default) or escalate to the
    /// SpecValidator agent for a richer classification (future
    /// extension; not in lean E-53 ship).
    CounterfactualUnknown,
    /// Reserved for verifier-attested paths. Producible ONLY by a
    /// formal-verifier outcome of `Verified`, never by the heuristic.
    CounterfactualSafe,
}

impl CounterfactualClass {
    /// Snake-case wire label used in SSB rows + dashboard rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CounterfactualRisky => "counterfactual_risky",
            Self::CounterfactualUnknown => "counterfactual_unknown",
            Self::CounterfactualSafe => "counterfactual_safe",
        }
    }

    /// Whether the classification indicates the path SHOULD route to
    /// a formal verifier. Lean policy: only `Risky` routes; `Unknown`
    /// and `Safe` skip dispatch (Safe is already verifier-attested;
    /// Unknown lacks the heuristic signal to justify the cost).
    #[must_use]
    pub fn dispatches_formal_verifier(self) -> bool {
        matches!(self, Self::CounterfactualRisky)
    }
}

/// Risk-pattern keyword list. The classifier flags any path whose
/// lowercase form contains any of these substrings.
///
/// Curation guidance (E-53 lean): keep the list tight. Each entry
/// should map to a real failure mode where a counterfactual mis-spec
/// causes silent damage (security boundary, financial state,
/// untrusted parser surface). New entries SHOULD land via a
/// patch-N follow-up with at least one fixture demonstrating the
/// failure mode, not via aspirational addition.
pub const RISK_PATTERNS: &[&str] = &[
    "auth",
    "login",
    "credential",
    "token",
    "password",
    "secret",
    "crypto",
    "cipher",
    "sign",
    "verify_sig",
    "money",
    "payment",
    "billing",
    "balance",
    "parser",
    "deserialize",
    "untrusted",
    "sandbox_escape",
];

/// Classify a path string. Returns [`CounterfactualClass::CounterfactualRisky`]
/// when any [`RISK_PATTERNS`] substring matches the lowercase path;
/// [`CounterfactualClass::CounterfactualUnknown`] otherwise.
///
/// Case-insensitive. Substring match (no path-segment splitting) so
/// the classifier flags both `crates/auth/src/lib.rs` and
/// `crates/foo/src/auth.rs` without depending on filesystem layout.
#[must_use]
pub fn classify_path(path: &str) -> CounterfactualClass {
    let lower = path.to_ascii_lowercase();
    if RISK_PATTERNS.iter().any(|pat| lower.contains(pat)) {
        CounterfactualClass::CounterfactualRisky
    } else {
        CounterfactualClass::CounterfactualUnknown
    }
}

/// Map a classifier outcome to the canonical block-source sentinel
/// the task_runner consumes. Returns `Some(reason_tag)` for Risky
/// (caller surfaces a CSQ_BLOCK on the BabyStepOutcome) and `None`
/// for Unknown / Safe.
///
/// The returned string is the snake-case reason tag matching
/// [`crate::task_runner::BlockSource::CsqCritical`]'s `reason_tag()`
/// ("CSQ_BLOCK"). Importing BlockSource here would create a circular
/// dep between csq and task_runner; the string sentinel keeps the
/// modules decoupled.
#[must_use]
pub fn block_reason_tag(class: CounterfactualClass) -> Option<&'static str> {
    match class {
        CounterfactualClass::CounterfactualRisky => Some("CSQ_BLOCK"),
        CounterfactualClass::CounterfactualUnknown | CounterfactualClass::CounterfactualSafe => {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_auth_path_returns_risky() {
        assert_eq!(
            classify_path("crates/auth/src/lib.rs"),
            CounterfactualClass::CounterfactualRisky
        );
    }

    #[test]
    fn classify_payment_path_returns_risky() {
        assert_eq!(
            classify_path("crates/billing/src/payment_handler.rs"),
            CounterfactualClass::CounterfactualRisky
        );
    }

    #[test]
    fn classify_crypto_path_returns_risky_case_insensitive() {
        assert_eq!(
            classify_path("Crates/Crypto/Cipher.rs"),
            CounterfactualClass::CounterfactualRisky
        );
    }

    #[test]
    fn classify_parser_path_returns_risky() {
        assert_eq!(
            classify_path("src/json_parser.rs"),
            CounterfactualClass::CounterfactualRisky
        );
    }

    #[test]
    fn classify_unrelated_path_returns_unknown() {
        assert_eq!(
            classify_path("crates/dashboard/ui/panel.svelte"),
            CounterfactualClass::CounterfactualUnknown
        );
    }

    #[test]
    fn dispatches_formal_verifier_only_for_risky() {
        assert!(CounterfactualClass::CounterfactualRisky.dispatches_formal_verifier());
        assert!(!CounterfactualClass::CounterfactualUnknown.dispatches_formal_verifier());
        assert!(!CounterfactualClass::CounterfactualSafe.dispatches_formal_verifier());
    }

    #[test]
    fn block_reason_tag_only_for_risky() {
        assert_eq!(
            block_reason_tag(CounterfactualClass::CounterfactualRisky),
            Some("CSQ_BLOCK")
        );
        assert_eq!(
            block_reason_tag(CounterfactualClass::CounterfactualUnknown),
            None
        );
        assert_eq!(
            block_reason_tag(CounterfactualClass::CounterfactualSafe),
            None
        );
    }

    #[test]
    fn as_str_wire_tags_are_snake_case() {
        assert_eq!(
            CounterfactualClass::CounterfactualRisky.as_str(),
            "counterfactual_risky"
        );
        assert_eq!(
            CounterfactualClass::CounterfactualUnknown.as_str(),
            "counterfactual_unknown"
        );
        assert_eq!(
            CounterfactualClass::CounterfactualSafe.as_str(),
            "counterfactual_safe"
        );
    }
}
