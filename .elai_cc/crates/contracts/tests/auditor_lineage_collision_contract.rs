// E-30: Auditor-lineage collision detection contract.
//
// Pins the contract that `auditor_executor_lineage_collision()` correctly
// identifies when auditor and executor share the same lineage group,
// consistent with arXiv:2511.18397 (73% false-negative rate for same-lineage
// auditor/executor pairs).
//
// Policy (E-30):
//   - Autonomous mode: collision = BLOCK (caller must refuse dispatch).
//   - Interactive mode: collision = WARN (caller surfaces to user).
//
// This contract tests only the predicate; mode-specific policy enforcement
// is the caller's responsibility and is tested in orchestrator unit tests.

use elai_orchestrator::{auditor_executor_lineage_collision, LineageGroup};

#[test]
fn same_string_lineage_is_collision() {
    let auditor = LineageGroup::new("session-shared-parent");
    let executor = LineageGroup::new("session-shared-parent");
    assert!(
        auditor_executor_lineage_collision(&auditor, &executor),
        "same lineage group string must detect as collision"
    );
}

#[test]
fn different_string_lineage_no_collision() {
    let auditor = LineageGroup::new("audit-pool-alpha");
    let executor = LineageGroup::new("exec-pool-beta");
    assert!(
        !auditor_executor_lineage_collision(&auditor, &executor),
        "different lineage groups must not be a collision"
    );
}

#[test]
fn empty_string_lineage_collision_on_match() {
    // Two agents with empty lineage group (uninitialized) collide with each other.
    let a = LineageGroup::new("");
    let b = LineageGroup::new("");
    assert!(
        auditor_executor_lineage_collision(&a, &b),
        "two empty lineage groups must be treated as collision (fail-safe)"
    );
}

#[test]
fn empty_vs_non_empty_no_collision() {
    let a = LineageGroup::new("");
    let b = LineageGroup::new("session-xyz");
    assert!(
        !auditor_executor_lineage_collision(&a, &b),
        "empty vs non-empty lineage groups must not collide"
    );
}

#[test]
fn lineage_group_serde_round_trip() {
    let original = LineageGroup::new("session-round-trip-test");
    let json = serde_json::to_string(&original).expect("serialize");
    let decoded: LineageGroup = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, decoded);
}

#[test]
fn lineage_group_clone_equality() {
    let g = LineageGroup::new("clone-test");
    let cloned = g.clone();
    assert_eq!(g, cloned);
    assert!(!auditor_executor_lineage_collision(&g, &LineageGroup::new("other")));
}
