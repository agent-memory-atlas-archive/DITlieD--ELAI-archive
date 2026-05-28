//! E-20 task 7 -- PerRoleRuntime priority + task_shape_hint
//! round-trip + default mapping contract.

use elai_ipc_types::AgentRole;
use elai_orchestrator::per_role_runtime::{PerRoleRuntime, Priority, TaskShape};

#[test]
fn priority_labels_stable() {
    assert_eq!(Priority::Cost.as_str(), "cost");
    assert_eq!(Priority::Latency.as_str(), "latency");
    assert_eq!(Priority::Quality.as_str(), "quality");
}

#[test]
fn task_shape_labels_stable() {
    assert_eq!(TaskShape::Execution.as_str(), "execution");
    assert_eq!(TaskShape::Architecture.as_str(), "architecture");
    assert_eq!(TaskShape::Math.as_str(), "math");
}

#[test]
fn auditor_family_defaults_to_quality_priority() {
    let per_role = PerRoleRuntime::default();
    for role in [
        AgentRole::Auditor,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
    ] {
        assert_eq!(per_role.priority(role), Priority::Quality);
    }
}

#[test]
fn architect_family_defaults_to_architecture_shape() {
    let per_role = PerRoleRuntime::default();
    for role in [
        AgentRole::Designer,
        AgentRole::Auditor,
        AgentRole::Researcher,
    ] {
        assert_eq!(per_role.task_shape(role), TaskShape::Architecture);
    }
}

#[test]
fn executor_defaults_to_latency_execution() {
    let per_role = PerRoleRuntime::default();
    assert_eq!(per_role.priority(AgentRole::Executor), Priority::Latency);
    assert_eq!(per_role.task_shape(AgentRole::Executor), TaskShape::Execution);
}

#[test]
fn planner_roles_default_to_cost_execution() {
    let per_role = PerRoleRuntime::default();
    for role in [
        AgentRole::PromptEngineer,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
    ] {
        assert_eq!(per_role.priority(role), Priority::Cost);
        assert_eq!(per_role.task_shape(role), TaskShape::Execution);
    }
}

#[test]
fn set_priority_round_trip() {
    let mut per_role = PerRoleRuntime::default();
    per_role.set_priority(AgentRole::Executor, Priority::Quality);
    assert_eq!(per_role.priority(AgentRole::Executor), Priority::Quality);
}

#[test]
fn set_task_shape_round_trip() {
    let mut per_role = PerRoleRuntime::default();
    per_role.set_task_shape(AgentRole::Executor, TaskShape::Math);
    assert_eq!(per_role.task_shape(AgentRole::Executor), TaskShape::Math);
}

#[test]
fn default_priority_and_task_shape_cover_every_agent_role() {
    let per_role = PerRoleRuntime::default();
    for role in [
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
    ] {
        assert!(
            per_role.priority.contains_key(&role),
            "Priority for AgentRole::{role:?} missing from Default"
        );
        assert!(
            per_role.task_shape_hint.contains_key(&role),
            "TaskShape for AgentRole::{role:?} missing from Default"
        );
    }
}

#[test]
fn serde_round_trip_preserves_priority_and_task_shape() {
    let original = PerRoleRuntime::default();
    let json = serde_json::to_string(&original).expect("serialize");
    let decoded: PerRoleRuntime = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, decoded);
}

#[test]
fn forward_compat_default_returns_cost_execution_when_missing() {
    let mut per_role = PerRoleRuntime::default();
    per_role.priority.clear();
    per_role.task_shape_hint.clear();
    assert_eq!(per_role.priority(AgentRole::Auditor), Priority::Cost);
    assert_eq!(
        per_role.task_shape(AgentRole::Auditor),
        TaskShape::Execution
    );
}
