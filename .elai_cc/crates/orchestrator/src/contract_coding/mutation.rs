//! C-50 — Mutation primitives per paper {Add, Update} closure
//! (arXiv:2604.13100 Section 4.3).
//!
//! The paper deliberately excludes `Delete` from the mutation
//! alphabet to avoid kernel inconsistency: a Delete on `Sigma` would
//! leave dangling `StateConstraint` references in `Delta` and a
//! Delete on `N` would orphan signatures. Add + Update keep the
//! kernel monotonically populated and re-projectable after each
//! mutation.
//!
//! This module ships the closed enum `ContractMutation` with EXACTLY
//! two variants — `Add(StateConstraint)` and `Update { target,
//! new }`. A unit test enforces the cardinality so a future
//! contributor cannot silently add a `Delete` variant.

use serde::{Deserialize, Serialize};

use crate::contract_coding::spec::{StateConstraint, StructuralContract};

/// Paper-grounded mutation primitive on a [`StructuralContract`]'s
/// `Delta` (state space). Only Add + Update are allowed; Delete is
/// excluded per paper §4.3 kernel-consistency rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractMutation {
    /// Append a new constraint to `Delta`.
    Add(StateConstraint),
    /// Replace an existing constraint at `target` (index into
    /// `Delta`) with `new`. The replacement constraint must still
    /// satisfy the contract's reference invariants once applied.
    Update {
        /// Index into `StructuralContract::state_space`.
        target: usize,
        /// The replacement constraint.
        new: StateConstraint,
    },
}

/// Application error produced by `apply_mutation`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MutationError {
    /// `Update` index points past the current `state_space` length.
    #[error("update target {0} is out of range; state_space has {1} entries")]
    UpdateOutOfRange(usize, usize),
    /// Applying the mutation would violate a contract invariant
    /// (`namespace_ref` out of range, etc).
    #[error("post-mutation contract validation failed: {0}")]
    ValidationFailed(String),
}

impl ContractMutation {
    /// Apply this mutation to a contract in place. Re-validates the
    /// contract after mutation; if validation fails the mutation is
    /// reverted (best-effort — `Update` snapshots the prior value).
    pub fn apply_to(&self, contract: &mut StructuralContract) -> Result<(), MutationError> {
        match self {
            Self::Add(constraint) => {
                contract.state_space.push(constraint.clone());
                if let Err(e) = contract.validate() {
                    // revert
                    contract.state_space.pop();
                    return Err(MutationError::ValidationFailed(e.to_string()));
                }
                Ok(())
            }
            Self::Update { target, new } => {
                let len = contract.state_space.len();
                if *target >= len {
                    return Err(MutationError::UpdateOutOfRange(*target, len));
                }
                let prior = contract.state_space[*target].clone();
                contract.state_space[*target] = new.clone();
                if let Err(e) = contract.validate() {
                    contract.state_space[*target] = prior;
                    return Err(MutationError::ValidationFailed(e.to_string()));
                }
                Ok(())
            }
        }
    }
}

/// Compile-time check that ContractMutation has exactly 2 variants.
/// This is the load-bearing closure rule from paper §4.3; if a
/// future contributor adds a `Delete` variant the exhaustive match
/// here will not compile, surfacing the closure violation at the
/// call site.
#[allow(dead_code)]
fn closure_check(m: &ContractMutation) -> &'static str {
    match m {
        ContractMutation::Add(_) => "add",
        ContractMutation::Update { .. } => "update",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_coding::spec::TypeSignature;

    fn base_contract() -> StructuralContract {
        StructuralContract::new(
            vec!["alpha".into()],
            vec![TypeSignature::new("foo", "fn foo()", "rust")],
            vec![StateConstraint::new("rule_a", 0, "alpha owns x")],
        )
    }

    #[test]
    fn add_appends_constraint() {
        let mut c = base_contract();
        let m = ContractMutation::Add(StateConstraint::new("rule_b", 0, "alpha owns y"));
        m.apply_to(&mut c).expect("add applies");
        assert_eq!(c.state_space.len(), 2);
        assert_eq!(c.state_space[1].name, "rule_b");
    }

    #[test]
    fn add_with_bad_ref_reverts() {
        let mut c = base_contract();
        let m = ContractMutation::Add(StateConstraint::new("rule_b", 99, "bad ref"));
        let err = m.apply_to(&mut c).unwrap_err();
        assert!(matches!(err, MutationError::ValidationFailed(_)));
        // ensure reverted
        assert_eq!(c.state_space.len(), 1, "must revert on bad ref");
    }

    #[test]
    fn update_replaces_constraint() {
        let mut c = base_contract();
        let m = ContractMutation::Update {
            target: 0,
            new: StateConstraint::new("rule_a_v2", 0, "alpha owns x updated"),
        };
        m.apply_to(&mut c).expect("update applies");
        assert_eq!(c.state_space[0].name, "rule_a_v2");
    }

    #[test]
    fn update_out_of_range_rejected() {
        let mut c = base_contract();
        let m = ContractMutation::Update {
            target: 99,
            new: StateConstraint::new("rule_x", 0, "x"),
        };
        let err = m.apply_to(&mut c).unwrap_err();
        assert!(matches!(err, MutationError::UpdateOutOfRange(99, 1)));
    }

    #[test]
    fn update_with_bad_ref_reverts() {
        let mut c = base_contract();
        let m = ContractMutation::Update {
            target: 0,
            new: StateConstraint::new("rule_a", 99, "bad ref"),
        };
        let err = m.apply_to(&mut c).unwrap_err();
        assert!(matches!(err, MutationError::ValidationFailed(_)));
        // reverted
        assert_eq!(c.state_space[0].name, "rule_a");
        assert_eq!(c.state_space[0].namespace_ref, 0);
    }

    #[test]
    fn serde_round_trip_preserves_variant() {
        let m = ContractMutation::Add(StateConstraint::new("rule_x", 0, "x"));
        let bytes = rmp_serde::to_vec(&m).expect("encode");
        let decoded: ContractMutation = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, m);
    }

    #[test]
    fn closure_check_covers_two_variants() {
        let add = ContractMutation::Add(StateConstraint::new("a", 0, "x"));
        let upd = ContractMutation::Update {
            target: 0,
            new: StateConstraint::new("b", 0, "y"),
        };
        assert_eq!(closure_check(&add), "add");
        assert_eq!(closure_check(&upd), "update");
    }
}
