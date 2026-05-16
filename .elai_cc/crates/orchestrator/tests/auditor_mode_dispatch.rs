//! B-41 Slice 3i — auditor mode -> agent-role + cascade routing.
//!
//! Each of the 6 AuditorMode values must route to a DISTINCT AgentRole (Plan
//! maps back to the pre-existing Auditor variant for back-compat). Cascade
//! parent-tier mapping is also pinned here so the navigator can't silently
//! drift from the dispatcher's mapping.

use elai_ipc_types::AgentRole;
use elai_orchestrator::auditor_mode::AuditorMode;
use elai_orchestrator::per_level_audit_dispatch::parent_tier_label;
use elai_orchestrator::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};
use elai_orchestrator::auditor_mode::TierNodeId;

#[test]
fn six_modes_each_route_to_distinct_agent_role() {
    let pairs = [
        (AuditorMode::Concepts, AgentRole::AuditorConcepts),
        (AuditorMode::Phases, AgentRole::AuditorPhases),
        (AuditorMode::Steps, AgentRole::AuditorSteps),
        (AuditorMode::BabySteps, AgentRole::AuditorBabySteps),
        (
            AuditorMode::BabyStepDetail,
            AgentRole::AuditorBabyStepDetail,
        ),
        (AuditorMode::Plan, AgentRole::Auditor),
    ];
    // Distinctness: every mode -> distinct role except (no exception —
    // even Plan -> Auditor is distinct from the other 5).
    let mut roles = std::collections::HashSet::new();
    for (mode, expected) in pairs {
        assert_eq!(
            mode.map_to_agent_role(),
            expected,
            "mode {:?} must route to {:?}",
            mode,
            expected
        );
        roles.insert(format!("{:?}", expected));
    }
    assert_eq!(roles.len(), 6, "6 distinct AgentRole targets required");
}

#[test]
fn snake_case_labels_are_stable_and_unique() {
    let modes = [
        AuditorMode::Concepts,
        AuditorMode::Phases,
        AuditorMode::Steps,
        AuditorMode::BabySteps,
        AuditorMode::BabyStepDetail,
        AuditorMode::Plan,
    ];
    let labels: Vec<&str> = modes.iter().map(|m| m.as_str()).collect();
    let mut sorted = labels.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 6, "6 unique labels required");
    assert_eq!(
        labels,
        vec![
            "concepts",
            "phases",
            "steps",
            "babysteps",
            "babystep_detail",
            "plan"
        ]
    );
}

#[test]
fn cascade_parent_mapping_covers_all_tier_combinations() {
    // (mode, tier) -> expected parent
    let cases: Vec<(AuditorMode, TierNodeId, &str)> = vec![
        // Concepts at top tier — parent is itself (no cascade beyond root).
        (AuditorMode::Concepts, TierNodeId::Concept(ConceptId(1)), "concept"),
        (AuditorMode::Phases, TierNodeId::Phase(PhaseId(1)), "concept"),
        (AuditorMode::Steps, TierNodeId::Step(StepId(1)), "phase"),
        (AuditorMode::BabySteps, TierNodeId::BabyStep(BabyStepId(1)), "step"),
        // BabyStepDetail cascade override (DECISION-CASCADE).
        (
            AuditorMode::BabyStepDetail,
            TierNodeId::BabyStep(BabyStepId(1)),
            "baby_step",
        ),
    ];
    for (mode, tier, expected) in cases {
        let got = parent_tier_label(mode, tier);
        assert_eq!(
            got, expected,
            "mode={:?} tier={:?} expected parent={expected} got={got}",
            mode, tier
        );
    }
}

#[test]
fn babystep_detail_cascade_is_robust_to_inner_tier_id() {
    // Per DECISION-CASCADE, BabyStepDetail collapses to baby_step EVEN IF the
    // inner TierNodeId variant somehow says otherwise (defensive depth).
    for tier in [
        TierNodeId::Concept(ConceptId(1)),
        TierNodeId::Phase(PhaseId(1)),
        TierNodeId::Step(StepId(1)),
        TierNodeId::BabyStep(BabyStepId(1)),
    ] {
        assert_eq!(
            parent_tier_label(AuditorMode::BabyStepDetail, tier),
            "baby_step"
        );
    }
}
