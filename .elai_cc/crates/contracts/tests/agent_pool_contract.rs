//! Contract: elai_cc orchestrator must expose AgentPool trait with
//! spawn, spawn_parallel, communicate methods. InProcessAgentPool must
//! implement AgentPool with VRAM gating and entropy-quarantined UUIDs.

use std::sync::Arc;

use elai_core::SeededRng;
use elai_orchestrator::{
    AgentHandle, AgentPool, AgentPoolConfig, AgentPoolError, AgentRole, AgentStatus,
    InProcessAgentPool, ModelPool,
};

struct MockModelPool(u64);
impl ModelPool for MockModelPool {
    fn available_vram(&self) -> u64 {
        self.0
    }
}

fn pool(vram: u64) -> InProcessAgentPool<SeededRng> {
    InProcessAgentPool::new(
        AgentPoolConfig::default(),
        SeededRng::new(42),
        Arc::new(MockModelPool(vram)),
    )
}

#[test]
fn contract_spawn_returns_handle_with_role() {
    let mut p = pool(u64::MAX);
    let h: AgentHandle = p.spawn(AgentRole::Executor, "t".into()).unwrap();
    assert_eq!(h.role, AgentRole::Executor);
    assert_eq!(h.status, AgentStatus::Spawning);
}

#[test]
fn contract_spawn_vram_zero_returns_resource_error() {
    let mut p = pool(0);
    match p.spawn(AgentRole::Executor, "t".into()) {
        Err(AgentPoolError::ResourceInsufficient(_)) => {}
        other => panic!("CONTRACT: expected ResourceInsufficient, got {other:?}"),
    }
}

#[test]
fn contract_all_8_roles_constructible() {
    let roles = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ];
    assert_eq!(roles.len(), 8, "CONTRACT: must have exactly 8 AgentRole variants");
    let mut p = pool(u64::MAX);
    for role in &roles {
        let h = p.spawn(*role, "t".into()).unwrap();
        assert_eq!(h.role, *role);
    }
}

#[test]
fn contract_spawn_parallel_returns_all() {
    let mut p = pool(u64::MAX);
    let tasks = vec![
        (AgentRole::Executor, "a".into()),
        (AgentRole::Auditor, "b".into()),
    ];
    let handles = p.spawn_parallel(tasks).unwrap();
    assert_eq!(handles.len(), 2, "CONTRACT: spawn_parallel must return all handles");
    assert_ne!(handles[0].agent_id, handles[1].agent_id);
}

#[test]
fn contract_designer_grc_config_differs_from_executor() {
    let mut p = pool(u64::MAX);
    let designer = p.spawn(AgentRole::Designer, "t".into()).unwrap();
    let executor = p.spawn(AgentRole::Executor, "t".into()).unwrap();
    assert_ne!(
        designer.grc_config.role, executor.grc_config.role,
        "CONTRACT: Designer and Executor must have different role names"
    );
    assert_ne!(
        designer.grc_config.required_steps, executor.grc_config.required_steps,
        "CONTRACT: Designer and Executor must have different required_steps"
    );
}

#[test]
fn contract_max_concurrent_returns_at_least_one() {
    // VRAM-aware concurrency: max_concurrent must always return >= 1
    let p = pool(u64::MAX);
    assert!(
        p.max_concurrent() >= 1,
        "CONTRACT: max_concurrent must return at least 1"
    );

    // Zero VRAM: fallback to 1 (sequential), never 0
    let p_zero = pool(0);
    assert_eq!(
        p_zero.max_concurrent(),
        1,
        "CONTRACT: max_concurrent must return 1 when VRAM is zero"
    );
}
