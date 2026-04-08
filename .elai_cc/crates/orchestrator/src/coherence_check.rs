//! Coherence enforcement for phase elaborations.
//!
//! Three structural checks run after elaboration loading (3G.4):
//! 1. All extracted contracts are implemented.
//! 2. No undeclared interfaces are introduced.
//! 3. Contract modifications trigger version increments.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::plan_skeleton::{
    ContractRef, ExtractedContracts, PhaseDetail, PlanSkeleton,
};

// ---- Types ----------------------------------------------------------------

/// A single coherence violation (always BLOCK severity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoherenceViolation {
    pub check_number: u8,
    pub severity: &'static str,
    pub message: String,
}

/// Aggregated result of all coherence checks.
///
/// `structural_errors` captures errors from checks that could not run
/// (e.g. empty declared_interfaces, missing phase) without discarding
/// violations from checks that did run.
#[derive(Debug)]
pub struct CoherenceReport {
    pub violations: Vec<CoherenceViolation>,
    pub block_count: usize,
    pub structural_errors: Vec<CoherenceError>,
}

/// Errors that prevent coherence checks from running.
#[derive(Debug, Error)]
pub enum CoherenceError {
    #[error("declared_interfaces is empty — cannot validate")]
    EmptyDeclaredInterfaces,

    #[error("phase not found in skeleton: {0}")]
    PhaseNotFound(String),
}

// ---- Helpers --------------------------------------------------------------

/// Extract (name, type_signature) key for set comparison.
/// Invariants are intentionally excluded (deferred from 3G.3).
fn contract_key(c: &ContractRef) -> (String, String) {
    (c.name.clone(), c.type_signature.clone())
}

/// Build a set of contract keys from a slice of ContractRefs.
fn contract_key_set(
    contracts: &[ContractRef],
) -> BTreeSet<(String, String)> {
    contracts.iter().map(contract_key).collect()
}

// ---- Check 1: contracts satisfied -----------------------------------------

/// Verify that every extracted contract (input + output) appears in
/// `detail.implemented_contracts`. Match by (name, type_signature) only.
///
/// Direction: extracted must be a SUBSET of implemented.
/// Missing from implemented = violation. Extra in implemented = OK.
pub fn check_contracts_satisfied(
    detail: &PhaseDetail,
    extracted: &ExtractedContracts,
) -> Vec<CoherenceViolation> {
    let implemented = contract_key_set(&detail.implemented_contracts);

    let mut violations = Vec::new();

    for c in extracted
        .input_contracts
        .iter()
        .chain(extracted.output_contracts.iter())
    {
        let key = contract_key(c);
        if !implemented.contains(&key) {
            violations.push(CoherenceViolation {
                check_number: 1,
                severity: "BLOCK",
                message: format!(
                    "extracted contract '{}' (sig: '{}') not found in implemented_contracts",
                    c.name, c.type_signature,
                ),
            });
        }
    }

    violations
}

// ---- Check 2: no undeclared interfaces ------------------------------------

/// Verify that every declared interface is present in the skeleton's union
/// of ALL phase inputs + outputs (across ALL phases, not just current).
///
/// Returns `Err(CoherenceError::EmptyDeclaredInterfaces)` if
/// `detail.declared_interfaces` is empty.
pub fn check_no_undeclared_interfaces(
    detail: &PhaseDetail,
    skeleton: &PlanSkeleton,
) -> Result<Vec<CoherenceViolation>, CoherenceError> {
    if detail.declared_interfaces.is_empty() {
        return Err(CoherenceError::EmptyDeclaredInterfaces);
    }

    // Build union of ALL phases' inputs + outputs.
    let mut skeleton_interfaces = BTreeSet::new();
    for entry in skeleton.phases.values() {
        for c in entry.inputs.iter().chain(entry.outputs.iter()) {
            skeleton_interfaces.insert(contract_key(c));
        }
    }

    let mut violations = Vec::new();
    for c in &detail.declared_interfaces {
        let key = contract_key(c);
        if !skeleton_interfaces.contains(&key) {
            violations.push(CoherenceViolation {
                check_number: 2,
                severity: "BLOCK",
                message: format!(
                    "declared interface '{}' (sig: '{}') not found in skeleton phases",
                    c.name, c.type_signature,
                ),
            });
        }
    }

    Ok(violations)
}

// ---- Check 3: version incremented ----------------------------------------

/// Verify that if contracts changed between prev and curr skeleton for the
/// given phase, both the phase version and skeleton version were incremented.
///
/// Caller provides both snapshots — this function never fetches them.
pub fn check_version_incremented(
    detail: &PhaseDetail,
    prev_skeleton: &PlanSkeleton,
    curr_skeleton: &PlanSkeleton,
) -> Result<Vec<CoherenceViolation>, CoherenceError> {
    let phase_id = &detail.phase_id;

    let prev_entry = prev_skeleton
        .phases
        .get(phase_id)
        .ok_or_else(|| CoherenceError::PhaseNotFound(phase_id.clone()))?;
    let curr_entry = curr_skeleton
        .phases
        .get(phase_id)
        .ok_or_else(|| CoherenceError::PhaseNotFound(phase_id.clone()))?;

    // Compare contracts by extracting keys from inputs + outputs.
    let prev_keys: BTreeSet<_> = prev_entry
        .inputs
        .iter()
        .chain(prev_entry.outputs.iter())
        .map(contract_key)
        .collect();
    let curr_keys: BTreeSet<_> = curr_entry
        .inputs
        .iter()
        .chain(curr_entry.outputs.iter())
        .map(contract_key)
        .collect();

    if prev_keys == curr_keys {
        // No contract change — no version enforcement needed.
        return Ok(Vec::new());
    }

    let mut violations = Vec::new();

    if curr_entry.version <= prev_entry.version {
        violations.push(CoherenceViolation {
            check_number: 3,
            severity: "BLOCK",
            message: format!(
                "contracts changed in phase '{}' but phase version not incremented ({} <= {})",
                phase_id, curr_entry.version, prev_entry.version,
            ),
        });
    }

    if curr_skeleton.version <= prev_skeleton.version {
        violations.push(CoherenceViolation {
            check_number: 3,
            severity: "BLOCK",
            message: format!(
                "contracts changed in phase '{}' but skeleton version not incremented ({} <= {})",
                phase_id, curr_skeleton.version, prev_skeleton.version,
            ),
        });
    }

    Ok(violations)
}

// ---- Aggregated runner ----------------------------------------------------

/// Run all three coherence checks and aggregate results.
///
/// Structural errors (empty declared_interfaces, missing phase) are
/// captured in `structural_errors` without discarding violations from
/// checks that did run.
pub fn run_coherence_checks(
    detail: &PhaseDetail,
    extracted: &ExtractedContracts,
    skeleton: &PlanSkeleton,
    prev_skeleton: Option<&PlanSkeleton>,
) -> CoherenceReport {
    let mut violations = Vec::new();
    let mut structural_errors = Vec::new();

    // Check 1: contracts satisfied (infallible).
    violations.extend(check_contracts_satisfied(detail, extracted));

    // Check 2: no undeclared interfaces.
    match check_no_undeclared_interfaces(detail, skeleton) {
        Ok(v) => violations.extend(v),
        Err(e) => structural_errors.push(e),
    }

    // Check 3: version incremented (only if prev snapshot provided).
    if let Some(prev) = prev_skeleton {
        match check_version_incremented(detail, prev, skeleton) {
            Ok(v) => violations.extend(v),
            Err(e) => structural_errors.push(e),
        }
    }

    let block_count =
        violations.iter().filter(|v| v.severity == "BLOCK").count();
    CoherenceReport {
        violations,
        block_count,
        structural_errors,
    }
}

// ---- Tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_skeleton::{PhaseEntry, PlanSkeleton};

    fn cref(name: &str, sig: &str) -> ContractRef {
        ContractRef {
            name: name.into(),
            type_signature: sig.into(),
            invariants: Vec::new(),
        }
    }

    fn make_detail(
        phase_id: &str,
        implemented: Vec<ContractRef>,
        declared: Vec<ContractRef>,
    ) -> PhaseDetail {
        PhaseDetail {
            phase_id: phase_id.into(),
            skeleton_version: 1,
            contracts: ExtractedContracts {
                input_contracts: Vec::new(),
                output_contracts: Vec::new(),
                shared_constraints: Vec::new(),
            },
            implemented_contracts: implemented,
            declared_interfaces: declared,
            content: String::new(),
            token_count: 0,
        }
    }

    fn make_skeleton_with_interfaces() -> PlanSkeleton {
        let mut skel = PlanSkeleton::new("test-pipe");
        let mut a = PhaseEntry::new("A", "phase A").unwrap();
        a.outputs.push(cref("x", "fn() -> X"));
        a.inputs.push(cref("w", "fn() -> W"));
        skel.add_phase(a).unwrap();

        let mut b = PhaseEntry::new("B", "phase B").unwrap();
        b.inputs.push(cref("y", "fn() -> Y"));
        b.outputs.push(cref("z", "fn() -> Z"));
        skel.add_phase(b).unwrap();
        skel
    }

    // ---- Check 1 tests ----

    #[test]
    fn check1_all_satisfied() {
        let detail = make_detail(
            "A",
            vec![cref("x", "fn() -> X"), cref("y", "fn() -> Y")],
            vec![cref("x", "fn() -> X")],
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: vec![cref("y", "fn() -> Y")],
            shared_constraints: Vec::new(),
        };
        let v = check_contracts_satisfied(&detail, &extracted);
        assert!(v.is_empty());
    }

    #[test]
    fn check1_missing_one() {
        let detail = make_detail(
            "A",
            vec![cref("x", "fn() -> X")],
            vec![cref("x", "fn() -> X")],
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: vec![cref("y", "fn() -> Y")],
            shared_constraints: Vec::new(),
        };
        let v = check_contracts_satisfied(&detail, &extracted);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_number, 1);
        assert_eq!(v[0].severity, "BLOCK");
        assert!(v[0].message.contains("'y'"));
    }

    #[test]
    fn check1_invariants_ignored() {
        // Same name+sig but different invariants should still match.
        let mut impl_ref = cref("x", "fn() -> X");
        impl_ref.invariants = vec!["inv-A".into()];
        let mut ext_ref = cref("x", "fn() -> X");
        ext_ref.invariants = vec!["inv-B".into()];

        let detail = make_detail("A", vec![impl_ref], vec![]);
        let extracted = ExtractedContracts {
            input_contracts: vec![ext_ref],
            output_contracts: Vec::new(),
            shared_constraints: Vec::new(),
        };
        let v = check_contracts_satisfied(&detail, &extracted);
        assert!(v.is_empty(), "invariants should be ignored in matching");
    }

    #[test]
    fn check1_extra_implemented_ok() {
        // Extra in implemented is NOT a violation.
        let detail = make_detail(
            "A",
            vec![cref("x", "fn() -> X"), cref("extra", "fn() -> E")],
            vec![],
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: Vec::new(),
            shared_constraints: Vec::new(),
        };
        let v = check_contracts_satisfied(&detail, &extracted);
        assert!(v.is_empty());
    }

    // ---- Check 2 tests ----

    #[test]
    fn check2_subset_ok() {
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail(
            "A",
            vec![],
            vec![cref("x", "fn() -> X"), cref("y", "fn() -> Y")],
        );
        let v =
            check_no_undeclared_interfaces(&detail, &skel).unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn check2_undeclared_present() {
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail(
            "A",
            vec![],
            vec![
                cref("x", "fn() -> X"),
                cref("unknown", "fn() -> ?"),
            ],
        );
        let v =
            check_no_undeclared_interfaces(&detail, &skel).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_number, 2);
        assert!(v[0].message.contains("'unknown'"));
    }

    #[test]
    fn check2_empty_declared_err() {
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail("A", vec![], vec![]);
        let err =
            check_no_undeclared_interfaces(&detail, &skel).unwrap_err();
        assert!(matches!(err, CoherenceError::EmptyDeclaredInterfaces));
    }

    // ---- Check 3 tests ----

    #[test]
    fn check3_both_incremented_ok() {
        let mut prev = PlanSkeleton::new("pipe");
        prev.version = 1;
        let mut pa = PhaseEntry::new("A", "phase A").unwrap();
        pa.outputs.push(cref("x", "fn() -> X"));
        pa.version = 1;
        prev.add_phase(pa).unwrap();

        let mut curr = PlanSkeleton::new("pipe");
        curr.version = 2;
        let mut ca = PhaseEntry::new("A", "phase A").unwrap();
        ca.outputs.push(cref("x", "fn() -> X2")); // changed sig
        ca.version = 2;
        curr.add_phase(ca).unwrap();

        let detail = make_detail("A", vec![], vec![cref("x", "fn() -> X2")]);
        let v =
            check_version_incremented(&detail, &prev, &curr).unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn check3_phase_version_not_incremented() {
        let mut prev = PlanSkeleton::new("pipe");
        prev.version = 1;
        let mut pa = PhaseEntry::new("A", "phase A").unwrap();
        pa.outputs.push(cref("x", "fn() -> X"));
        pa.version = 1;
        prev.add_phase(pa).unwrap();

        let mut curr = PlanSkeleton::new("pipe");
        curr.version = 2;
        let mut ca = PhaseEntry::new("A", "phase A").unwrap();
        ca.outputs.push(cref("x", "fn() -> X2")); // changed
        ca.version = 1; // NOT incremented
        curr.add_phase(ca).unwrap();

        let detail = make_detail("A", vec![], vec![cref("x", "fn() -> X2")]);
        let v =
            check_version_incremented(&detail, &prev, &curr).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_number, 3);
        assert!(v[0].message.contains("phase version"));
    }

    #[test]
    fn check3_skeleton_version_not_incremented() {
        let mut prev = PlanSkeleton::new("pipe");
        prev.version = 1;
        let mut pa = PhaseEntry::new("A", "phase A").unwrap();
        pa.outputs.push(cref("x", "fn() -> X"));
        pa.version = 1;
        prev.add_phase(pa).unwrap();

        let mut curr = PlanSkeleton::new("pipe");
        curr.version = 1; // NOT incremented
        let mut ca = PhaseEntry::new("A", "phase A").unwrap();
        ca.outputs.push(cref("x", "fn() -> X2")); // changed
        ca.version = 2;
        curr.add_phase(ca).unwrap();

        let detail = make_detail("A", vec![], vec![cref("x", "fn() -> X2")]);
        let v =
            check_version_incremented(&detail, &prev, &curr).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_number, 3);
        assert!(v[0].message.contains("skeleton version"));
    }

    #[test]
    fn check3_no_change_no_violation() {
        let mut prev = PlanSkeleton::new("pipe");
        prev.version = 1;
        let mut pa = PhaseEntry::new("A", "phase A").unwrap();
        pa.outputs.push(cref("x", "fn() -> X"));
        pa.version = 1;
        prev.add_phase(pa).unwrap();

        let mut curr = PlanSkeleton::new("pipe");
        curr.version = 1; // same — but no contract change so OK
        let mut ca = PhaseEntry::new("A", "phase A").unwrap();
        ca.outputs.push(cref("x", "fn() -> X")); // same
        ca.version = 1;
        curr.add_phase(ca).unwrap();

        let detail = make_detail("A", vec![], vec![cref("x", "fn() -> X")]);
        let v =
            check_version_incremented(&detail, &prev, &curr).unwrap();
        assert!(v.is_empty());
    }

    // ---- Aggregated tests ----

    #[test]
    fn run_all_checks_aggregates() {
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail(
            "A",
            vec![cref("x", "fn() -> X")], // missing 'y' from extracted
            vec![
                cref("x", "fn() -> X"),
                cref("unknown", "fn() -> ?"), // undeclared
            ],
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: vec![cref("y", "fn() -> Y")],
            shared_constraints: Vec::new(),
        };

        let report =
            run_coherence_checks(&detail, &extracted, &skel, None);
        // Check 1: 'y' missing from implemented -> 1 violation
        // Check 2: 'unknown' not in skeleton -> 1 violation
        assert_eq!(report.violations.len(), 2);
        assert_eq!(report.block_count, 2);
        assert!(report.structural_errors.is_empty());
        assert!(report.violations.iter().any(|v| v.check_number == 1));
        assert!(report.violations.iter().any(|v| v.check_number == 2));
    }

    #[test]
    fn run_all_check1_violations_preserved_on_check2_err() {
        // Check 1 has violations + Check 2 returns Err (empty declared).
        // Violations from Check 1 must NOT be discarded.
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail(
            "A",
            vec![], // missing implementation -> check 1 violation
            vec![], // empty declared -> check 2 Err
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: Vec::new(),
            shared_constraints: Vec::new(),
        };
        let report =
            run_coherence_checks(&detail, &extracted, &skel, None);
        // Check 1 violation preserved despite Check 2 Err.
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].check_number, 1);
        assert_eq!(report.structural_errors.len(), 1);
        assert!(matches!(
            report.structural_errors[0],
            CoherenceError::EmptyDeclaredInterfaces,
        ));
    }

    #[test]
    fn run_all_check1_violations_preserved_on_check3_err() {
        // Check 1 has violations + Check 3 returns Err (missing phase).
        // Violations from Check 1 must NOT be discarded.
        let skel = make_skeleton_with_interfaces();
        let detail = make_detail(
            "A",
            vec![], // missing implementation -> check 1 violation
            vec![cref("x", "fn() -> X")], // valid declared
        );
        let extracted = ExtractedContracts {
            input_contracts: vec![cref("x", "fn() -> X")],
            output_contracts: Vec::new(),
            shared_constraints: Vec::new(),
        };
        // prev_skeleton has no phase "A" -> PhaseNotFound error.
        let prev = PlanSkeleton::new("pipe");
        let report = run_coherence_checks(
            &detail, &extracted, &skel, Some(&prev),
        );
        // Check 1 violation preserved despite Check 3 Err.
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].check_number, 1);
        assert_eq!(report.structural_errors.len(), 1);
        assert!(matches!(
            report.structural_errors[0],
            CoherenceError::PhaseNotFound(_),
        ));
    }
}
