//! E-6 — PhaseKind: kind-of-work tag used to gate adversary requirement.
//!
//! Resolves integration-explorer GAP-1: the plan signature
//! `adversary_verdict_required(phase: &PhaseKind) -> bool` had no
//! existing type to bind to. FsmState + FsmSubstate were considered
//! and rejected as too coarse to attribute MAST-mode predicate per
//! work-kind. A single phase can map to multiple FsmStates over a
//! run (e.g. a single "Code" phase covers PLANNING + TESTING_FIRST +
//! EXECUTING + AUDITING).
//!
//! This enum is INDEPENDENT of FsmState. The orchestrator caller
//! supplies the appropriate PhaseKind at the cross-family-gate query
//! site (typically AUDITING -> MEMORIZING transition).

use serde::{Deserialize, Serialize};

/// What kind of work is the orchestrator currently auditing?
///
/// Six variants drawn from the MAST corpus + ELAI's auditor finding taxonomy.
/// `Code` / `Architecture` / `Security` require cross-family adversary
/// confirmation per the plan; `Docs` / `Test` / `Other` do not.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    /// Source-code production (functions, types, business logic).
    /// HIGHEST adversary requirement — code is where MAST modes
    /// FM-1.1 / FM-2.6 / FM-3.3 land. Cross-family adversary REQUIRED.
    Code,
    /// Architectural / API / contract design (interface boundaries,
    /// type definitions, wire protocols). Adversary REQUIRED — these
    /// changes propagate across the orchestrator + dashboard
    /// boundary.
    Architecture,
    /// Security-sensitive changes (auth, permissions, sandbox, secret
    /// handling). Adversary REQUIRED — same-family blind-spot
    /// (64.5% per arxiv:2507.02778) is most dangerous here.
    Security,
    /// Documentation, comments, plan files. Adversary NOT required —
    /// no executable surface; reviewer can catch issues directly.
    Docs,
    /// Test code (unit tests, contract tests, integration tests).
    /// Adversary NOT required — tests are themselves a verifier,
    /// the test framework + auditor pass already constitutes a
    /// cross-check.
    Test,
    /// Catch-all for changes that don't fit the other 5 buckets
    /// (e.g. build-script tweaks, formatting). Adversary NOT required.
    Other,
}

impl PhaseKind {
    /// Wire-stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Architecture => "architecture",
            Self::Security => "security",
            Self::Docs => "docs",
            Self::Test => "test",
            Self::Other => "other",
        }
    }

    /// True iff this phase kind REQUIRES a cross-family-confirmed
    /// adversary verdict before AUDITING -> MEMORIZING is allowed.
    ///
    /// Sibling of [`crate::adversary::adversary_verdict_required`]
    /// (which is the public adversary-side entry; this is the
    /// PhaseKind-side mirror for callers that already hold a
    /// PhaseKind value).
    #[must_use]
    pub const fn requires_cross_family_adversary(self) -> bool {
        matches!(self, Self::Code | Self::Architecture | Self::Security)
    }

    /// All 6 phase kinds for iteration.
    pub const ALL: [PhaseKind; 6] = [
        Self::Code,
        Self::Architecture,
        Self::Security,
        Self::Docs,
        Self::Test,
        Self::Other,
    ];
}

impl std::fmt::Display for PhaseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_arch_security_require_adversary() {
        assert!(PhaseKind::Code.requires_cross_family_adversary());
        assert!(PhaseKind::Architecture.requires_cross_family_adversary());
        assert!(PhaseKind::Security.requires_cross_family_adversary());
    }

    #[test]
    fn docs_test_other_do_not_require_adversary() {
        assert!(!PhaseKind::Docs.requires_cross_family_adversary());
        assert!(!PhaseKind::Test.requires_cross_family_adversary());
        assert!(!PhaseKind::Other.requires_cross_family_adversary());
    }

    #[test]
    fn as_str_stable() {
        assert_eq!(PhaseKind::Code.as_str(), "code");
        assert_eq!(PhaseKind::Other.as_str(), "other");
    }

    #[test]
    fn all_6_distinct() {
        use std::collections::BTreeSet;
        let set: BTreeSet<&str> = PhaseKind::ALL.iter().map(|p| p.as_str()).collect();
        assert_eq!(set.len(), 6);
    }
}
