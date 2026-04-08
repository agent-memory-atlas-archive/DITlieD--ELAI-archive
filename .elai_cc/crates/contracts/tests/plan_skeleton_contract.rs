//! Contract tests for PlanSkeleton and coherence_check.
//! Verifies contract extraction, conflict detection, constraint merging,
//! and coherence enforcement.

use elai_orchestrator::coherence_check::{
    check_contracts_satisfied, check_no_undeclared_interfaces,
    check_version_incremented, run_coherence_checks, CoherenceError,
};
use elai_orchestrator::plan_skeleton::{
    ContractRef, ExtractedContracts, PhaseDetail, PhaseEntry,
    PlanSkeleton, PlanSkeletonError,
};

fn contract(name: &str, sig: &str) -> ContractRef {
    ContractRef {
        name: name.into(),
        type_signature: sig.into(),
        invariants: Vec::new(),
    }
}

fn make_chain_skeleton() -> PlanSkeleton {
    let mut skel = PlanSkeleton::new("test-pipe");
    skel.global_constraints
        .push("deterministic execution".into());

    let mut a = PhaseEntry::new("A", "phase A").unwrap();
    a.outputs.push(contract("x", "fn() -> X"));
    a.depended_by.push("B".into());
    skel.add_phase(a).unwrap();

    let mut b = PhaseEntry::new("B", "phase B").unwrap();
    b.depends_on.push("A".into());
    b.depended_by.push("C".into());
    b.constraints.push("b-local constraint".into());
    skel.add_phase(b).unwrap();

    let mut c = PhaseEntry::new("C", "phase C").unwrap();
    c.inputs.push(contract("y", "fn() -> Y"));
    c.depends_on.push("B".into());
    skel.add_phase(c).unwrap();

    skel
}

#[test]
fn test_extract_contracts_basic_chain() {
    let skel = make_chain_skeleton();
    let result = skel.extract_contracts("B").unwrap();

    // input_contracts = A's outputs (what B receives from deps)
    assert_eq!(result.input_contracts.len(), 1);
    assert_eq!(result.input_contracts[0].name, "x");
    assert_eq!(result.input_contracts[0].type_signature, "fn() -> X");

    // output_contracts = C's inputs (what B must produce for dependents)
    assert_eq!(result.output_contracts.len(), 1);
    assert_eq!(result.output_contracts[0].name, "y");
    assert_eq!(result.output_contracts[0].type_signature, "fn() -> Y");

    // shared_constraints = global first, then B's local
    assert_eq!(result.shared_constraints.len(), 2);
    assert_eq!(result.shared_constraints[0], "deterministic execution");
    assert_eq!(result.shared_constraints[1], "b-local constraint");
}

#[test]
fn test_extract_contracts_no_deps() {
    let mut skel = PlanSkeleton::new("test-pipe");
    skel.global_constraints.push("global-1".into());
    let entry = PhaseEntry::new("solo", "standalone phase").unwrap();
    skel.add_phase(entry).unwrap();

    let result = skel.extract_contracts("solo").unwrap();
    assert!(result.input_contracts.is_empty());
    assert!(result.output_contracts.is_empty());
    assert_eq!(result.shared_constraints, vec!["global-1"]);
}

#[test]
fn test_extract_contracts_phase_not_found() {
    let skel = PlanSkeleton::new("test-pipe");
    let err = skel.extract_contracts("nonexistent").unwrap_err();
    match err {
        PlanSkeletonError::PhaseNotFound(id) => {
            assert_eq!(id, "nonexistent");
        }
        other => panic!("expected PhaseNotFound, got: {other}"),
    }
}

#[test]
fn test_extract_contracts_missing_dep_phase() {
    let mut skel = PlanSkeleton::new("test-pipe");
    let mut entry = PhaseEntry::new("B", "phase B").unwrap();
    entry.depends_on.push("A".into()); // A doesn't exist
    skel.add_phase(entry).unwrap();

    let err = skel.extract_contracts("B").unwrap_err();
    match err {
        PlanSkeletonError::PhaseNotFound(id) => {
            assert_eq!(id, "A");
        }
        other => panic!("expected PhaseNotFound for missing dep, got: {other}"),
    }
}

#[test]
fn test_extract_contracts_contract_conflict() {
    let mut skel = PlanSkeleton::new("test-pipe");

    let mut dep1 = PhaseEntry::new("D1", "dep 1").unwrap();
    dep1.outputs.push(contract("shared_name", "fn() -> TypeA"));
    dep1.depended_by.push("B".into());
    skel.add_phase(dep1).unwrap();

    let mut dep2 = PhaseEntry::new("D2", "dep 2").unwrap();
    dep2.outputs.push(contract("shared_name", "fn() -> TypeB"));
    dep2.depended_by.push("B".into());
    skel.add_phase(dep2).unwrap();

    let mut b = PhaseEntry::new("B", "phase B").unwrap();
    b.depends_on.push("D1".into());
    b.depends_on.push("D2".into());
    skel.add_phase(b).unwrap();

    let err = skel.extract_contracts("B").unwrap_err();
    match err {
        PlanSkeletonError::ContractConflict {
            name,
            phase_a,
            phase_b,
        } => {
            assert_eq!(name, "shared_name");
            assert_eq!(phase_a, "D1");
            assert_eq!(phase_b, "D2");
        }
        other => panic!("expected ContractConflict, got: {other}"),
    }
}

#[test]
fn test_extract_contracts_shared_constraints() {
    let mut skel = PlanSkeleton::new("test-pipe");
    skel.global_constraints.push("g1".into());
    skel.global_constraints.push("g2".into());

    let mut entry = PhaseEntry::new("P", "phase P").unwrap();
    entry.constraints.push("local-1".into());
    entry.constraints.push("local-2".into());
    skel.add_phase(entry).unwrap();

    let result = skel.extract_contracts("P").unwrap();
    assert_eq!(
        result.shared_constraints,
        vec!["g1", "g2", "local-1", "local-2"]
    );
}

#[test]
fn test_extract_contracts_dedup_same_signature() {
    // Same contract name + same type_signature from two deps = deduplicate, NOT conflict
    let mut skel = PlanSkeleton::new("test-pipe");

    let mut dep1 = PhaseEntry::new("D1", "dep 1").unwrap();
    dep1.outputs.push(contract("shared", "fn() -> Same"));
    dep1.depended_by.push("B".into());
    skel.add_phase(dep1).unwrap();

    let mut dep2 = PhaseEntry::new("D2", "dep 2").unwrap();
    dep2.outputs.push(contract("shared", "fn() -> Same"));
    dep2.depended_by.push("B".into());
    skel.add_phase(dep2).unwrap();

    let mut b = PhaseEntry::new("B", "phase B").unwrap();
    b.depends_on.push("D1".into());
    b.depends_on.push("D2".into());
    skel.add_phase(b).unwrap();

    let result = skel.extract_contracts("B").unwrap();
    // Should have exactly 1 entry, not 2 (deduplication)
    assert_eq!(result.input_contracts.len(), 1);
    assert_eq!(result.input_contracts[0].name, "shared");
}

#[test]
fn test_extract_contracts_depended_by_conflict() {
    // Two depended_by phases request same contract name with different type_signature
    let mut skel = PlanSkeleton::new("test-pipe");

    let mut b = PhaseEntry::new("B", "phase B").unwrap();
    b.depended_by.push("C1".into());
    b.depended_by.push("C2".into());
    skel.add_phase(b).unwrap();

    let mut c1 = PhaseEntry::new("C1", "consumer 1").unwrap();
    c1.inputs.push(contract("data", "fn() -> TypeX"));
    c1.depends_on.push("B".into());
    skel.add_phase(c1).unwrap();

    let mut c2 = PhaseEntry::new("C2", "consumer 2").unwrap();
    c2.inputs.push(contract("data", "fn() -> TypeY"));
    c2.depends_on.push("B".into());
    skel.add_phase(c2).unwrap();

    let err = skel.extract_contracts("B").unwrap_err();
    match err {
        PlanSkeletonError::ContractConflict {
            name,
            phase_a,
            phase_b,
        } => {
            assert_eq!(name, "data");
            assert_eq!(phase_a, "C1");
            assert_eq!(phase_b, "C2");
        }
        other => panic!("expected ContractConflict on depended_by path, got: {other}"),
    }
}

#[test]
fn test_extract_contracts_depended_by_dedup() {
    // Two depended_by phases request same contract name+sig = deduplicate
    let mut skel = PlanSkeleton::new("test-pipe");

    let mut b = PhaseEntry::new("B", "phase B").unwrap();
    b.depended_by.push("C1".into());
    b.depended_by.push("C2".into());
    skel.add_phase(b).unwrap();

    let mut c1 = PhaseEntry::new("C1", "consumer 1").unwrap();
    c1.inputs.push(contract("data", "fn() -> Same"));
    c1.depends_on.push("B".into());
    skel.add_phase(c1).unwrap();

    let mut c2 = PhaseEntry::new("C2", "consumer 2").unwrap();
    c2.inputs.push(contract("data", "fn() -> Same"));
    c2.depends_on.push("B".into());
    skel.add_phase(c2).unwrap();

    let result = skel.extract_contracts("B").unwrap();
    assert_eq!(result.output_contracts.len(), 1);
    assert_eq!(result.output_contracts[0].name, "data");
}

#[test]
fn test_extract_contracts_missing_depended_by_phase() {
    // depended_by references a phase not in the skeleton
    let mut skel = PlanSkeleton::new("test-pipe");
    let mut entry = PhaseEntry::new("B", "phase B").unwrap();
    entry.depended_by.push("ghost".into());
    skel.add_phase(entry).unwrap();

    let err = skel.extract_contracts("B").unwrap_err();
    match err {
        PlanSkeletonError::PhaseNotFound(id) => {
            assert_eq!(id, "ghost");
        }
        other => panic!("expected PhaseNotFound for missing depended_by, got: {other}"),
    }
}

// ---- Coherence check contract tests ----

fn make_detail_for_contract(
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

#[test]
fn contract_coherence_check1_subset_enforcement() {
    let detail = make_detail_for_contract(
        "A",
        vec![contract("x", "fn() -> X")],
        vec![contract("x", "fn() -> X")],
    );
    let extracted = ExtractedContracts {
        input_contracts: vec![contract("x", "fn() -> X")],
        output_contracts: vec![contract("missing", "fn() -> M")],
        shared_constraints: Vec::new(),
    };
    let violations = check_contracts_satisfied(&detail, &extracted);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].check_number, 1);
    assert_eq!(violations[0].severity, "BLOCK");
}

#[test]
fn contract_coherence_check2_undeclared_detection() {
    let skel = make_chain_skeleton();
    let detail = make_detail_for_contract(
        "B",
        vec![],
        vec![
            contract("x", "fn() -> X"), // exists in skeleton (A's output)
            contract("phantom", "fn() -> P"), // not in skeleton
        ],
    );
    let violations =
        check_no_undeclared_interfaces(&detail, &skel).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].check_number, 2);
    assert!(violations[0].message.contains("'phantom'"));
}

#[test]
fn contract_coherence_check2_empty_declared_err() {
    let skel = make_chain_skeleton();
    let detail = make_detail_for_contract("B", vec![], vec![]);
    let err =
        check_no_undeclared_interfaces(&detail, &skel).unwrap_err();
    assert!(matches!(err, CoherenceError::EmptyDeclaredInterfaces));
}

#[test]
fn contract_coherence_check3_version_enforcement() {
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

    let detail = make_detail_for_contract(
        "A",
        vec![],
        vec![contract("x", "fn() -> X2")],
    );
    let violations =
        check_version_incremented(&detail, &prev, &curr).unwrap();
    assert_eq!(violations.len(), 2); // both phase + skeleton
    assert!(violations.iter().all(|v| v.check_number == 3));
    assert!(violations.iter().all(|v| v.severity == "BLOCK"));
}

#[test]
fn contract_coherence_run_all_aggregation() {
    let mut skel = PlanSkeleton::new("pipe");
    let mut a = PhaseEntry::new("A", "phase A").unwrap();
    a.outputs.push(contract("x", "fn() -> X"));
    skel.add_phase(a).unwrap();

    let detail = make_detail_for_contract(
        "A",
        vec![], // missing implementation -> check 1 violation
        vec![
            contract("x", "fn() -> X"),
            contract("ghost", "fn() -> G"), // undeclared -> check 2 violation
        ],
    );
    let extracted = ExtractedContracts {
        input_contracts: vec![contract("x", "fn() -> X")],
        output_contracts: Vec::new(),
        shared_constraints: Vec::new(),
    };
    let report =
        run_coherence_checks(&detail, &extracted, &skel, None);
    // check 1: 'x' not in implemented -> 1 violation
    // check 2: 'ghost' not in skeleton -> 1 violation
    assert_eq!(report.block_count, 2);
    assert!(report.structural_errors.is_empty());
    assert!(report.violations.iter().any(|v| v.check_number == 1));
    assert!(report.violations.iter().any(|v| v.check_number == 2));
}

#[test]
fn contract_coherence_mixed_violations_and_structural_err() {
    // Check 1 violations must be preserved even when Check 2 returns Err.
    let mut skel = PlanSkeleton::new("pipe");
    let mut a = PhaseEntry::new("A", "phase A").unwrap();
    a.outputs.push(contract("x", "fn() -> X"));
    skel.add_phase(a).unwrap();

    let detail = make_detail_for_contract(
        "A",
        vec![], // missing -> check 1 violation
        vec![], // empty -> check 2 structural error
    );
    let extracted = ExtractedContracts {
        input_contracts: vec![contract("x", "fn() -> X")],
        output_contracts: Vec::new(),
        shared_constraints: Vec::new(),
    };
    let report =
        run_coherence_checks(&detail, &extracted, &skel, None);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].check_number, 1);
    assert_eq!(report.structural_errors.len(), 1);
}
