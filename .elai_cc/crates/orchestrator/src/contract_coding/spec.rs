//! C-50 — Structural contract types per paper kernel
//! `K = <N, Sigma, Delta>` (arXiv:2604.13100 Section 4.2).
//!
//! `N` — module namespace (paper's "Constraint Projection" section
//! names): the list of named buckets the contract is partitioned
//! across. Stored as `Vec<String>` so consumers can iterate in
//! declaration order without an extra index.
//!
//! `Sigma` — type-signature set: the per-function or per-method
//! interface descriptors. The paper uses these to anchor the
//! Generator Agent's per-task emission (Section 4.3).
//!
//! `Delta` — state-space constraints: the structural invariants
//! across the kernel. These are NOT Hoare-style behavioral
//! assertions; they are structural rules ("module X must declare
//! function Y", "state field Z is owned by namespace W").
//!
//! Disjointness invariant (paper Section 4.2): the sections under
//! `N` must be pairwise disjoint. `StructuralContract::validate`
//! checks this and rejects duplicates.

use serde::{Deserialize, Serialize};

/// Per-function type-signature descriptor. The paper does not pin
/// a serialization format; we use a strict snake_case-named
/// struct so consumers can round-trip via `rmp_serde` or `serde_json`
/// without ambiguity.
///
/// `function_name` is the canonical identifier the Generator Agent
/// emits implementation for. `signature` is the textual signature
/// (e.g. `fn add(a: i64, b: i64) -> i64`). `language_hint` is the
/// target language; used by the [`crate::contract_coding::emitter`]
/// to pick the right scaffolder template.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeSignature {
    /// Canonical function or method name.
    pub function_name: String,
    /// Full textual signature (e.g. `fn add(a: i64, b: i64) -> i64`).
    pub signature: String,
    /// Target language for the body (e.g. `"rust"`, `"python"`,
    /// `"typescript"`).
    pub language_hint: String,
}

impl TypeSignature {
    /// Build a fresh `TypeSignature` with the three required fields.
    #[must_use]
    pub fn new(
        function_name: impl Into<String>,
        signature: impl Into<String>,
        language_hint: impl Into<String>,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            signature: signature.into(),
            language_hint: language_hint.into(),
        }
    }
}

/// Structural state-space constraint (paper `Delta`). NOT a Hoare
/// pre/postcondition — see module docs for the structural framing.
///
/// `name` identifies the constraint (e.g. `"snake_module_ownership"`).
/// `namespace_ref` is the index into [`StructuralContract::namespace`]
/// that owns the constraint (paper's section disjointness invariant).
/// `description` is the operator-facing prose form.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateConstraint {
    /// Stable identifier for the constraint.
    pub name: String,
    /// Index into `StructuralContract::namespace` (the owning
    /// section). Validated by `StructuralContract::validate` —
    /// out-of-range indices are rejected.
    pub namespace_ref: usize,
    /// Operator-facing prose description (no behavioral semantics —
    /// structural rule only).
    pub description: String,
}

impl StateConstraint {
    /// Build a fresh `StateConstraint`.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        namespace_ref: usize,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            namespace_ref,
            description: description.into(),
        }
    }
}

/// Paper kernel `K = <N, Sigma, Delta>` (Contract-Coding §4.2).
///
/// `namespace` — `N`: the ordered list of section names the contract
/// is partitioned across. Pairwise disjointness is enforced by
/// `validate`.
///
/// `signatures` — `Sigma`: the per-function signature set. Multiple
/// signatures may share the same `language_hint` but the
/// `function_name`s must be unique within the contract.
///
/// `state_space` — `Delta`: the structural state-space constraints.
/// Each constraint's `namespace_ref` must be a valid index into
/// `namespace`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralContract {
    /// `N` — module namespace section list.
    pub namespace: Vec<String>,
    /// `Sigma` — per-function signature set.
    pub signatures: Vec<TypeSignature>,
    /// `Delta` — state-space constraints.
    pub state_space: Vec<StateConstraint>,
}

/// Validation error produced by `StructuralContract::validate`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpecError {
    /// `N` contains a duplicate section name. Paper §4.2 requires
    /// pairwise disjoint sections.
    #[error("namespace section `{0}` is duplicated; paper §4.2 requires pairwise disjoint sections")]
    DuplicateNamespaceSection(String),
    /// `Sigma` contains two signatures with the same `function_name`.
    #[error("signature for function `{0}` is duplicated; each function_name must be unique")]
    DuplicateSignature(String),
    /// A `StateConstraint::namespace_ref` points outside the `N`
    /// section list.
    #[error("state constraint `{name}` references namespace index {idx} but namespace has only {len} sections")]
    NamespaceRefOutOfRange {
        /// Constraint name for diagnostic.
        name: String,
        /// The out-of-range index.
        idx: usize,
        /// Total namespace section count.
        len: usize,
    },
}

impl StructuralContract {
    /// Build an empty contract (no `N`, no `Sigma`, no `Delta`).
    /// Used by builder-style construction in the emitter / scaffolder.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            namespace: Vec::new(),
            signatures: Vec::new(),
            state_space: Vec::new(),
        }
    }

    /// Build a contract from the three kernel components in one shot.
    #[must_use]
    pub fn new(
        namespace: Vec<String>,
        signatures: Vec<TypeSignature>,
        state_space: Vec<StateConstraint>,
    ) -> Self {
        Self {
            namespace,
            signatures,
            state_space,
        }
    }

    /// Validate the disjointness + reference invariants. Returns
    /// `Err` on the first violation found (no recovery — the caller
    /// fixes and re-validates).
    pub fn validate(&self) -> Result<(), SpecError> {
        // N: pairwise disjoint sections. We use an O(n^2) scan
        // because section counts are tiny (paper's Roguelike was
        // 15-25 sections); a HashSet would still be O(n) for the
        // expected size and adds an allocation we do not need.
        for i in 0..self.namespace.len() {
            for j in (i + 1)..self.namespace.len() {
                if self.namespace[i] == self.namespace[j] {
                    return Err(SpecError::DuplicateNamespaceSection(
                        self.namespace[i].clone(),
                    ));
                }
            }
        }

        // Sigma: unique function_name per signature.
        for i in 0..self.signatures.len() {
            for j in (i + 1)..self.signatures.len() {
                if self.signatures[i].function_name == self.signatures[j].function_name {
                    return Err(SpecError::DuplicateSignature(
                        self.signatures[i].function_name.clone(),
                    ));
                }
            }
        }

        // Delta: namespace_ref in range.
        for c in &self.state_space {
            if c.namespace_ref >= self.namespace.len() {
                return Err(SpecError::NamespaceRefOutOfRange {
                    name: c.name.clone(),
                    idx: c.namespace_ref,
                    len: self.namespace.len(),
                });
            }
        }
        Ok(())
    }

    /// True iff all three kernel components are empty. Useful for
    /// scaffolder pre-checks ("there is nothing to emit").
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.namespace.is_empty() && self.signatures.is_empty() && self.state_space.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_contract_validates() {
        let c = StructuralContract::empty();
        assert!(c.is_empty());
        c.validate().expect("empty contract validates");
    }

    #[test]
    fn duplicate_namespace_rejected() {
        let c = StructuralContract::new(
            vec!["alpha".into(), "alpha".into()],
            vec![],
            vec![],
        );
        let err = c.validate().unwrap_err();
        assert!(matches!(err, SpecError::DuplicateNamespaceSection(ref s) if s == "alpha"));
    }

    #[test]
    fn duplicate_signature_rejected() {
        let c = StructuralContract::new(
            vec!["alpha".into()],
            vec![
                TypeSignature::new("foo", "fn foo() -> ()", "rust"),
                TypeSignature::new("foo", "fn foo(a: i64) -> i64", "rust"),
            ],
            vec![],
        );
        let err = c.validate().unwrap_err();
        assert!(matches!(err, SpecError::DuplicateSignature(ref s) if s == "foo"));
    }

    #[test]
    fn out_of_range_namespace_ref_rejected() {
        let c = StructuralContract::new(
            vec!["alpha".into()],
            vec![],
            vec![StateConstraint::new("rule_a", 5, "out of range")],
        );
        let err = c.validate().unwrap_err();
        match err {
            SpecError::NamespaceRefOutOfRange { name, idx, len } => {
                assert_eq!(name, "rule_a");
                assert_eq!(idx, 5);
                assert_eq!(len, 1);
            }
            other => panic!("expected NamespaceRefOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn well_formed_contract_validates() {
        let c = StructuralContract::new(
            vec!["alpha".into(), "beta".into()],
            vec![TypeSignature::new("foo", "fn foo() -> ()", "rust")],
            vec![StateConstraint::new("rule_a", 0, "alpha owns x")],
        );
        c.validate().expect("validates");
    }

    #[test]
    fn serde_round_trip_preserves_fields() {
        let c = StructuralContract::new(
            vec!["alpha".into()],
            vec![TypeSignature::new("foo", "fn foo() -> ()", "rust")],
            vec![StateConstraint::new("rule_a", 0, "desc")],
        );
        let bytes = rmp_serde::to_vec(&c).expect("encode");
        let decoded: StructuralContract = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, c);
    }
}
