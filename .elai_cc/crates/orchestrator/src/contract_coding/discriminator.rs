//! C-50 — Heuristic Discriminator pass.
//!
//! Paper grounding: arXiv:2604.13100 §4.4 — the Discriminator Agent
//! audits generated code against the contract `K = <N, Sigma, Delta>`.
//! The paper's Discriminator is RL-trained (Algorithm 1 — GRPO + KL
//! anchor); ELAI's C-50 ships a HEURISTIC pre-stage that surfaces
//! the obvious kernel violations (missing function signatures,
//! missing namespace sections, unrecorded state constraints).
//!
//! RUNPOD-PENDING: the full Generator -> Discriminator RL training
//! pipeline is deferred to a billable session (paper Algorithm 1,
//! Sections 4.4 + 4.5).
//!
//! Failure-observable discipline (per CLAUDE.md):
//!   - Every violation produces a structured [`ContractViolation`]
//!     with a `kind` field; consumers can never silently coerce
//!     "no body content matched the namespace" into "PASS".
//!   - `audit_against_contract` does NOT panic — it returns
//!     `AuditResult` with `is_consistent = false` and a populated
//!     `violations` Vec.

use serde::{Deserialize, Serialize};

use crate::contract_coding::spec::StructuralContract;

/// Categorical kind tag for a kernel violation. Stable wire format
/// — SSB rows reference these via `violation_kind_hash` (blake3
/// prefix of `as_str()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    /// A namespace section in `N` was not mentioned in the
    /// generated code at all.
    MissingNamespace,
    /// A signature in `Sigma` was not mentioned in the generated
    /// code (function name absent).
    MissingSignature,
    /// A state constraint name was not mentioned in the generated
    /// code (heuristic — the constraint surface should reference
    /// the rule by name for the auditor to grep).
    UnrecordedStateConstraint,
}

impl ViolationKind {
    /// Stable snake_case wire tag.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingNamespace => "missing_namespace",
            Self::MissingSignature => "missing_signature",
            Self::UnrecordedStateConstraint => "unrecorded_state_constraint",
        }
    }
}

/// Single kernel violation surfaced by the heuristic Discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractViolation {
    /// Which kernel slot was violated.
    pub kind: ViolationKind,
    /// The element name that failed (section / function /
    /// constraint identifier).
    pub element: String,
    /// Operator-facing prose. Stored on the row so the dashboard
    /// can render the violation without re-deriving from the kernel.
    pub message: String,
}

/// Aggregate verdict of `audit_against_contract`. `is_consistent`
/// is `true` iff `violations.is_empty()`. The two are kept as
/// separate fields so consumers can pattern-match on `is_consistent`
/// without inspecting the Vec, mirroring the paper's binary
/// verdict shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditResult {
    /// `true` iff the generated code satisfies every kernel slot.
    pub is_consistent: bool,
    /// Per-slot violation list. Empty iff `is_consistent` is true.
    pub violations: Vec<ContractViolation>,
}

impl AuditResult {
    /// Build a passing verdict (no violations).
    #[must_use]
    pub fn pass() -> Self {
        Self {
            is_consistent: true,
            violations: Vec::new(),
        }
    }

    /// Build a failing verdict from a violation list.
    #[must_use]
    pub fn fail(violations: Vec<ContractViolation>) -> Self {
        let is_consistent = violations.is_empty();
        Self {
            is_consistent,
            violations,
        }
    }
}

/// Heuristic Discriminator pass. Walks `generated_code` for every
/// namespace / signature / constraint name and records misses as
/// violations. Substring match is intentional — anything more
/// sophisticated needs the RUNPOD-PENDING RL Discriminator.
///
/// Determinism: scans are in kernel-declaration order. The
/// returned `violations` Vec follows the same order (N first,
/// then Sigma, then Delta).
#[must_use]
pub fn audit_against_contract(
    generated_code: &str,
    spec: &StructuralContract,
) -> AuditResult {
    let mut violations = Vec::new();

    // N — namespace section presence.
    for section in &spec.namespace {
        if !generated_code.contains(section.as_str()) {
            violations.push(ContractViolation {
                kind: ViolationKind::MissingNamespace,
                element: section.clone(),
                message: format!(
                    "namespace section `{}` not mentioned in generated code",
                    section
                ),
            });
        }
    }

    // Sigma — signature function-name presence.
    for sig in &spec.signatures {
        if !generated_code.contains(sig.function_name.as_str()) {
            violations.push(ContractViolation {
                kind: ViolationKind::MissingSignature,
                element: sig.function_name.clone(),
                message: format!(
                    "signature `{}` ({}) not mentioned in generated code",
                    sig.function_name, sig.signature
                ),
            });
        }
    }

    // Delta — state-constraint name presence.
    for c in &spec.state_space {
        if !generated_code.contains(c.name.as_str()) {
            violations.push(ContractViolation {
                kind: ViolationKind::UnrecordedStateConstraint,
                element: c.name.clone(),
                message: format!(
                    "state constraint `{}` not referenced in generated code",
                    c.name
                ),
            });
        }
    }

    AuditResult::fail(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_coding::spec::{StateConstraint, TypeSignature};

    fn sample_contract() -> StructuralContract {
        StructuralContract::new(
            vec!["alpha".into(), "beta".into()],
            vec![TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")],
            vec![StateConstraint::new("totality", 0, "add is total on i64")],
        )
    }

    #[test]
    fn passing_code_yields_consistent() {
        let code = "mod alpha { fn add() {} } mod beta {} // totality";
        let r = audit_against_contract(code, &sample_contract());
        assert!(r.is_consistent);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_namespace_surfaces_violation() {
        let code = "mod beta {} fn add() {} // totality";
        let r = audit_against_contract(code, &sample_contract());
        assert!(!r.is_consistent);
        assert!(r
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::MissingNamespace && v.element == "alpha"));
    }

    #[test]
    fn missing_signature_surfaces_violation() {
        let code = "mod alpha {} mod beta {} // totality";
        let r = audit_against_contract(code, &sample_contract());
        assert!(r
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::MissingSignature && v.element == "add"));
    }

    #[test]
    fn missing_constraint_surfaces_violation() {
        let code = "mod alpha {} mod beta {} fn add() {}";
        let r = audit_against_contract(code, &sample_contract());
        assert!(r.violations.iter().any(|v| {
            v.kind == ViolationKind::UnrecordedStateConstraint && v.element == "totality"
        }));
    }

    #[test]
    fn empty_contract_yields_pass() {
        let r = audit_against_contract("anything", &StructuralContract::empty());
        assert!(r.is_consistent);
    }

    #[test]
    fn violation_kind_wire_tags_stable() {
        assert_eq!(ViolationKind::MissingNamespace.as_str(), "missing_namespace");
        assert_eq!(ViolationKind::MissingSignature.as_str(), "missing_signature");
        assert_eq!(
            ViolationKind::UnrecordedStateConstraint.as_str(),
            "unrecorded_state_constraint"
        );
    }

    #[test]
    fn audit_result_serde_round_trip() {
        let r = AuditResult::fail(vec![ContractViolation {
            kind: ViolationKind::MissingSignature,
            element: "foo".into(),
            message: "x".into(),
        }]);
        let bytes = rmp_serde::to_vec(&r).expect("encode");
        let decoded: AuditResult = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, r);
    }
}
