//! Contract tests for CoherenceChecker (Phase 3G exit criteria).
//!
//! Covers all 3 coherence checks with valid + invalid cases (6 tests).

use elai_orchestrator::coherence_check::{
    check_contracts_satisfied, check_no_undeclared_interfaces,
    check_version_incremented,
};
use elai_orchestrator::plan_skeleton::{
    ContractRef, ExtractedContracts, PhaseDetail, PhaseEntry,
    PlanSkeleton,
};

fn contract(name: &str, sig: &str) -> ContractRef {
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

fn make_skeleton_with_phase(
    phase_id: &str,
    inputs: Vec<ContractRef>,
    outputs: Vec<ContractRef>,
) -> PlanSkeleton {
    let mut skel = PlanSkeleton::new("contract-pipe");
    let mut entry =
        PhaseEntry::new(phase_id, "test phase").unwrap();
    entry.inputs = inputs;
    entry.outputs = outputs;
    skel.add_phase(entry).unwrap();
    skel
}

// ---- Check 1: contracts satisfied -----------------------------------------

#[test]
fn check1_satisfied_all_implemented() {
    let extracted = ExtractedContracts {
        input_contracts: vec![contract("x", "fn() -> X")],
        output_contracts: vec![contract("y", "fn() -> Y")],
        shared_constraints: Vec::new(),
    };
    let detail = make_detail(
        "A",
        vec![
            contract("x", "fn() -> X"),
            contract("y", "fn() -> Y"),
        ],
        vec![],
    );
    let violations =
        check_contracts_satisfied(&detail, &extracted);
    assert!(
        violations.is_empty(),
        "all contracts implemented, expected 0 violations, got {}",
        violations.len()
    );
}

#[test]
fn check1_missing_contract_blocks() {
    let extracted = ExtractedContracts {
        input_contracts: vec![contract("x", "fn() -> X")],
        output_contracts: vec![contract("missing", "fn() -> M")],
        shared_constraints: Vec::new(),
    };
    let detail = make_detail(
        "A",
        vec![contract("x", "fn() -> X")],
        vec![],
    );
    let violations =
        check_contracts_satisfied(&detail, &extracted);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].check_number, 1);
    assert_eq!(violations[0].severity, "BLOCK");
    assert!(violations[0].message.contains("missing"));
}

// ---- Check 2: no undeclared interfaces ------------------------------------

#[test]
fn check2_no_undeclared_passes() {
    let skel = make_skeleton_with_phase(
        "A",
        vec![contract("x", "fn() -> X")],
        vec![contract("y", "fn() -> Y")],
    );
    let detail = make_detail(
        "A",
        vec![],
        vec![
            contract("x", "fn() -> X"),
            contract("y", "fn() -> Y"),
        ],
    );
    let violations =
        check_no_undeclared_interfaces(&detail, &skel)
            .unwrap();
    assert!(
        violations.is_empty(),
        "all declared interfaces exist in skeleton, expected 0 violations, got {}",
        violations.len()
    );
}

#[test]
fn check2_undeclared_interface_blocks() {
    let skel = make_skeleton_with_phase(
        "A",
        vec![contract("x", "fn() -> X")],
        vec![],
    );
    let detail = make_detail(
        "A",
        vec![],
        vec![
            contract("x", "fn() -> X"),
            contract("phantom", "fn() -> P"),
        ],
    );
    let violations =
        check_no_undeclared_interfaces(&detail, &skel)
            .unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].check_number, 2);
    assert_eq!(violations[0].severity, "BLOCK");
    assert!(violations[0].message.contains("phantom"));
}

// ---- Check 3: version incremented ----------------------------------------

#[test]
fn check3_version_incremented_passes() {
    let mut prev = PlanSkeleton::new("pipe");
    prev.version = 1;
    let mut pa = PhaseEntry::new("A", "phase A").unwrap();
    pa.outputs.push(contract("x", "fn() -> X"));
    pa.version = 1;
    prev.add_phase(pa).unwrap();

    let mut curr = PlanSkeleton::new("pipe");
    curr.version = 2;
    let mut ca = PhaseEntry::new("A", "phase A").unwrap();
    ca.outputs.push(contract("x", "fn() -> X2"));
    ca.version = 2;
    curr.add_phase(ca).unwrap();

    let detail = make_detail(
        "A",
        vec![],
        vec![contract("x", "fn() -> X2")],
    );
    let violations =
        check_version_incremented(&detail, &prev, &curr)
            .unwrap();
    assert!(
        violations.is_empty(),
        "versions incremented, expected 0 violations, got {}",
        violations.len()
    );
}

#[test]
fn check3_version_not_incremented_blocks() {
    let mut prev = PlanSkeleton::new("pipe");
    prev.version = 1;
    let mut pa = PhaseEntry::new("A", "phase A").unwrap();
    pa.outputs.push(contract("x", "fn() -> X"));
    pa.version = 1;
    prev.add_phase(pa).unwrap();

    let mut curr = PlanSkeleton::new("pipe");
    curr.version = 1; // NOT incremented
    let mut ca = PhaseEntry::new("A", "phase A").unwrap();
    ca.outputs.push(contract("x", "fn() -> X2")); // changed
    ca.version = 1; // NOT incremented
    curr.add_phase(ca).unwrap();

    let detail = make_detail(
        "A",
        vec![],
        vec![contract("x", "fn() -> X2")],
    );
    let violations =
        check_version_incremented(&detail, &prev, &curr)
            .unwrap();
    assert_eq!(violations.len(), 2); // phase + skeleton
    assert!(violations.iter().all(|v| v.check_number == 3));
    assert!(violations.iter().all(|v| v.severity == "BLOCK"));
}
