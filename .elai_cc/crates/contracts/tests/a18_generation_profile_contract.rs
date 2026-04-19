//! Batch 4C (A-18) — GenerationProfile + dispatch selector contract.
//!
//! Pins the behaviour of [`elai_orchestrator::profile_selector`] so the
//! 7-profile set and the default selector's role + FSM mapping stay stable
//! across future refactors.
//!
//! Grounding:
//!   * crates/orchestrator/src/context_budget.rs L60-102 (enum + 7 variants)
//!   * crates/orchestrator/src/profile_selector.rs
//!   * reasoning-architecture-design.txt §GENERATION PARAMETERS

use elai_orchestrator::context_budget::GenerationProfile;
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::profile_selector::{
    profile_name, DefaultGenerationProfileSelector, DispatchRole, GenerationProfileSelector,
};

// --- (a) All 7 profiles enumerate a unique, non-empty name. ---

#[test]
fn contract_all_seven_profiles_have_unique_names() {
    let profiles = [
        GenerationProfile::FirstAttempt,
        GenerationProfile::MicroReflection,
        GenerationProfile::Interrogation,
        GenerationProfile::RepairRetry,
        GenerationProfile::TreeBranchA,
        GenerationProfile::TreeBranchB,
        GenerationProfile::TreeBranchC,
    ];
    let names: Vec<&'static str> = profiles.iter().map(|p| profile_name(*p)).collect();
    assert_eq!(names.len(), 7, "exactly 7 profiles enumerated");
    for name in &names {
        assert!(!name.is_empty(), "profile name must be non-empty");
    }
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 7, "all 7 profile names must be unique");
}

// --- (b) Default selector picks RepairRetry on Repair role. ---

#[test]
fn contract_default_selector_repair_role_picks_repair_retry() {
    let s = DefaultGenerationProfileSelector;
    let p = s.select(DispatchRole::Repair, FsmState::Executing);
    assert_eq!(p, GenerationProfile::RepairRetry);
    // Temperature and top_p must match the documented repair profile.
    assert!(
        (p.temperature() - 1.0).abs() < 0.001,
        "RepairRetry temperature must be 1.0"
    );
    assert!(
        (p.top_p() - 1.0).abs() < 0.001,
        "RepairRetry top_p must be 1.0"
    );
}

// --- (c) Default selector picks FirstAttempt on Executor role. ---

#[test]
fn contract_default_selector_executor_role_picks_first_attempt() {
    let s = DefaultGenerationProfileSelector;
    let p = s.select(DispatchRole::Executor, FsmState::Executing);
    assert_eq!(p, GenerationProfile::FirstAttempt);
    assert!(p.temperature() <= 0.5, "FirstAttempt must be low-temp");
}

// --- (d) Researching FSM overrides scalar roles to Interrogation. ---

#[test]
fn contract_researching_overrides_scalar_roles_to_interrogation() {
    let s = DefaultGenerationProfileSelector;
    for role in [
        DispatchRole::Executor,
        DispatchRole::Repair,
        DispatchRole::Auditor,
        DispatchRole::Research,
    ] {
        let p = s.select(role, FsmState::Researching);
        assert_eq!(
            p,
            GenerationProfile::Interrogation,
            "role {:?} in RESEARCHING must become Interrogation",
            role
        );
    }
}

// --- (e) Researching preserves AB-MCTS branch profiles. ---

#[test]
fn contract_researching_preserves_tree_branches() {
    let s = DefaultGenerationProfileSelector;
    assert_eq!(
        s.select(DispatchRole::TreeBranchA, FsmState::Researching),
        GenerationProfile::TreeBranchA
    );
    assert_eq!(
        s.select(DispatchRole::TreeBranchB, FsmState::Researching),
        GenerationProfile::TreeBranchB
    );
    assert_eq!(
        s.select(DispatchRole::TreeBranchC, FsmState::Researching),
        GenerationProfile::TreeBranchC
    );
}

// --- (f) DispatchRole.as_str returns a distinct lower-snake tag per role. ---

#[test]
fn contract_dispatch_role_as_str_unique_and_snake() {
    let roles = [
        DispatchRole::Executor,
        DispatchRole::Repair,
        DispatchRole::Auditor,
        DispatchRole::Research,
        DispatchRole::Interrogation,
        DispatchRole::TreeBranchA,
        DispatchRole::TreeBranchB,
        DispatchRole::TreeBranchC,
    ];
    let mut seen = std::collections::HashSet::new();
    for r in roles {
        let s = r.as_str();
        assert!(!s.is_empty());
        assert_eq!(
            s,
            s.to_lowercase(),
            "DispatchRole::as_str must be lower-case snake"
        );
        assert!(seen.insert(s), "duplicate role tag {s}");
    }
}

// --- (g) profile_name reflects the enum variant name (stable). ---

#[test]
fn contract_profile_name_matches_variant() {
    assert_eq!(profile_name(GenerationProfile::FirstAttempt), "FirstAttempt");
    assert_eq!(
        profile_name(GenerationProfile::MicroReflection),
        "MicroReflection"
    );
    assert_eq!(profile_name(GenerationProfile::RepairRetry), "RepairRetry");
    assert_eq!(profile_name(GenerationProfile::TreeBranchB), "TreeBranchB");
}

// --- (h) Temperature and top_p are bounded to valid generation ranges. ---

#[test]
fn contract_generation_profile_params_are_in_valid_ranges() {
    let profiles = [
        GenerationProfile::FirstAttempt,
        GenerationProfile::MicroReflection,
        GenerationProfile::Interrogation,
        GenerationProfile::RepairRetry,
        GenerationProfile::TreeBranchA,
        GenerationProfile::TreeBranchB,
        GenerationProfile::TreeBranchC,
    ];
    for p in profiles {
        assert!(
            p.temperature() >= 0.0 && p.temperature() <= 1.5,
            "{:?} temperature must be in [0.0, 1.5]",
            p
        );
        assert!(
            p.top_p() > 0.0 && p.top_p() <= 1.0,
            "{:?} top_p must be in (0.0, 1.0]",
            p
        );
    }
}
