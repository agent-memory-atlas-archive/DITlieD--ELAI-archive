//! E-26 contract: PerRoleRuntime SGLang routing under
//! `ELAI_SGLANG_BASE_URL` env var.
//!
//! Verifies:
//! 1. With env unset, PerRoleRuntime::default() preserves B-37
//!    behaviour: PromptEngineer + IdeaFramer + IdeaCritic remain on
//!    Sidecar.
//! 2. With env set, the same three roles route to Sglang.
//! 3. Non-planner-shape roles (Executor, Researcher, MemoryWriter,
//!    Auditor) are unaffected.
//! 4. The 5 per-level Auditor variants (B-41) remain on AnthropicApi
//!    regardless of env state.
//! 5. The full-coverage invariant (all 16 AgentRole variants mapped)
//!    holds under both env states.
//!
//! ENV_LOCK serializes the env mutations to avoid races on parallel
//! `cargo test` runs — mirrors the D-30 calibration_bootstrap_contract
//! precedent.

use std::sync::Mutex;

use elai_ipc_types::AgentRole;
use elai_orchestrator::per_role_runtime::PerRoleRuntime;
use elai_orchestrator::runtime_config::ELAI_SGLANG_BASE_URL_ENV;
use elai_providers::runtime::RuntimeBackend;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_env() {
    std::env::remove_var(ELAI_SGLANG_BASE_URL_ENV);
}

fn set_env() {
    std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, "http://example.test:30000");
}

#[test]
fn env_unset_preserves_b37_planner_routing() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_SGLANG_BASE_URL_ENV).ok();
    clear_env();

    let per_role = PerRoleRuntime::default();
    assert_eq!(per_role.get(AgentRole::PromptEngineer), RuntimeBackend::Sidecar);
    assert_eq!(per_role.get(AgentRole::IdeaFramer), RuntimeBackend::Sidecar);
    assert_eq!(per_role.get(AgentRole::IdeaCritic), RuntimeBackend::Sidecar);

    if let Some(v) = prev {
        std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, v);
    }
}

#[test]
fn env_set_routes_planner_shape_to_sglang() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_SGLANG_BASE_URL_ENV).ok();
    set_env();

    let per_role = PerRoleRuntime::default();
    assert_eq!(per_role.get(AgentRole::PromptEngineer), RuntimeBackend::Sglang);
    assert_eq!(per_role.get(AgentRole::IdeaFramer), RuntimeBackend::Sglang);
    assert_eq!(per_role.get(AgentRole::IdeaCritic), RuntimeBackend::Sglang);

    clear_env();
    if let Some(v) = prev {
        std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, v);
    }
}

#[test]
fn env_set_does_not_affect_non_planner_roles() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_SGLANG_BASE_URL_ENV).ok();
    set_env();

    let per_role = PerRoleRuntime::default();
    // Reasoning-heavy AnthropicApi tier untouched.
    assert_eq!(per_role.get(AgentRole::Designer), RuntimeBackend::AnthropicApi);
    assert_eq!(per_role.get(AgentRole::Auditor), RuntimeBackend::AnthropicApi);
    assert_eq!(per_role.get(AgentRole::Researcher), RuntimeBackend::AnthropicApi);
    assert_eq!(per_role.get(AgentRole::MediaValidator), RuntimeBackend::AnthropicApi);
    // Local Sidecar tier untouched.
    assert_eq!(per_role.get(AgentRole::Executor), RuntimeBackend::Sidecar);
    assert_eq!(per_role.get(AgentRole::ContextCompiler), RuntimeBackend::Sidecar);
    assert_eq!(per_role.get(AgentRole::MemoryWriter), RuntimeBackend::Sidecar);
    assert_eq!(per_role.get(AgentRole::InjectionClassifier), RuntimeBackend::Sidecar);

    clear_env();
    if let Some(v) = prev {
        std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, v);
    }
}

#[test]
fn env_set_preserves_per_level_auditor_routing() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_SGLANG_BASE_URL_ENV).ok();
    set_env();

    let per_role = PerRoleRuntime::default();
    for role in [
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
    ] {
        assert_eq!(
            per_role.get(role),
            RuntimeBackend::AnthropicApi,
            "AuditorPerLevel role {role:?} must stay on AnthropicApi even with SGLang env set"
        );
    }

    clear_env();
    if let Some(v) = prev {
        std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, v);
    }
}

#[test]
fn full_coverage_invariant_holds_under_both_env_states() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_SGLANG_BASE_URL_ENV).ok();

    let all_roles: [AgentRole; 16] = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::InjectionClassifier,
    ];

    // Env unset.
    clear_env();
    let per_role = PerRoleRuntime::default();
    for role in all_roles {
        assert!(
            per_role.list().iter().any(|(r, _)| *r == role),
            "AgentRole::{role:?} missing with env unset"
        );
    }

    // Env set.
    set_env();
    let per_role = PerRoleRuntime::default();
    for role in all_roles {
        assert!(
            per_role.list().iter().any(|(r, _)| *r == role),
            "AgentRole::{role:?} missing with env set"
        );
    }

    clear_env();
    if let Some(v) = prev {
        std::env::set_var(ELAI_SGLANG_BASE_URL_ENV, v);
    }
}
