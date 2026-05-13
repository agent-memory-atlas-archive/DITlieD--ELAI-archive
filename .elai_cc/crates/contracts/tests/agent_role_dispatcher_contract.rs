//! Phase A5 — AgentRoleDispatcher contract.
//!
//! Pins:
//!   1. `AgentRoleDispatcher::from_value` parses the AppState `model_routing`
//!      JSON shape `[{role, target, fallback?}, ...]` without panic.
//!   2. `pick_role` finds a role by hint, normalising kebab/snake forms
//!      so "prompt-engineer" and "prompt_engineer" resolve identically.
//!   3. `build_dispatch` returns a `RoleDispatch` whose `provider_kind`
//!      matches the prefix of the configured target ("anthropic:"/"claude*"
//!      -> Anthropic, "local:"/Ollama-family -> Ollama, "xai:"/"grok*"
//!      -> Xai).
//!   4. Three distinct roles bound to three distinct provider variants
//!      resolve to three distinct `ProviderKind` values, exercising the
//!      core dispatch axis the plan calls out (cloud / local / third).
//!   5. Unknown role returns `RoleDispatchError::RoleNotFound`, not a
//!      silent fallback.

use elai_orchestrator::agent_role_dispatcher::{
    AgentRoleDispatcher, RoleDispatchError,
};
use elai_providers::ProviderKind;

fn routing_fixture() -> serde_json::Value {
    serde_json::json!([
        { "role": "executor",         "target": "anthropic:claude-sonnet-4-6" },
        { "role": "auditor",          "target": "local:qwen-2.5-coder-7b" },
        { "role": "prompt-engineer",  "target": "xai:grok-3" },
    ])
}

#[test]
fn dispatcher_parses_appstate_routing_shape() {
    let dispatcher = AgentRoleDispatcher::from_value(&routing_fixture())
        .expect("routing JSON must parse");
    assert_eq!(dispatcher.role_count(), 3);
}

#[test]
fn pick_role_normalises_kebab_and_snake() {
    let dispatcher = AgentRoleDispatcher::from_value(&routing_fixture())
        .expect("routing JSON must parse");
    // Configured as kebab-case ("prompt-engineer") but caller may hand
    // the snake_case form coming from `AgentRole` serde.
    let by_snake = dispatcher
        .pick_role("prompt_engineer")
        .expect("snake_case hint must resolve");
    let by_kebab = dispatcher
        .pick_role("prompt-engineer")
        .expect("kebab-case hint must resolve");
    assert_eq!(by_snake.role, by_kebab.role);
}

#[test]
fn three_roles_resolve_to_three_distinct_providers() {
    let dispatcher = AgentRoleDispatcher::from_value(&routing_fixture())
        .expect("routing JSON must parse");

    let exec = dispatcher.pick_role("executor").expect("executor present");
    let aud = dispatcher.pick_role("auditor").expect("auditor present");
    let pe = dispatcher
        .pick_role("prompt-engineer")
        .expect("prompt-engineer present");

    let dispatch_exec = dispatcher
        .build_dispatch(exec)
        .expect("executor dispatch builds");
    let dispatch_aud = dispatcher
        .build_dispatch(aud)
        .expect("auditor dispatch builds");
    let dispatch_pe = dispatcher
        .build_dispatch(pe)
        .expect("prompt-engineer dispatch builds");

    assert_eq!(dispatch_exec.provider_kind, ProviderKind::Anthropic);
    assert_eq!(dispatch_exec.model, "claude-sonnet-4-6");

    assert_eq!(dispatch_aud.provider_kind, ProviderKind::Ollama);
    assert_eq!(dispatch_aud.model, "qwen-2.5-coder-7b");

    assert_eq!(dispatch_pe.provider_kind, ProviderKind::Xai);
    assert_eq!(dispatch_pe.model, "grok-3");

    // Core axis: 3 distinct provider variants observed.
    let variants = [
        dispatch_exec.provider_kind,
        dispatch_aud.provider_kind,
        dispatch_pe.provider_kind,
    ];
    let mut sorted: Vec<&'static str> = variants
        .iter()
        .map(|k| match k {
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Ollama => "ollama",
            ProviderKind::Xai => "xai",
            ProviderKind::OpenAi => "openai",
        })
        .collect();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 3, "three roles must yield three distinct providers");
}

#[test]
fn unknown_role_returns_role_not_found_not_silent_fallback() {
    let dispatcher = AgentRoleDispatcher::from_value(&routing_fixture())
        .expect("routing JSON must parse");
    let err = dispatcher
        .pick_role("nonexistent_role")
        .expect_err("unknown role must error, never silently fall back");
    assert!(matches!(err, RoleDispatchError::RoleNotFound(_)));
}

#[test]
fn invalid_routing_value_returns_typed_error() {
    // The model_routing slot is Arc<Mutex<serde_json::Value>>; a non-array
    // payload must surface as a typed error rather than panic.
    let bad = serde_json::json!({ "not": "an array" });
    let err = AgentRoleDispatcher::from_value(&bad)
        .expect_err("non-array routing must error");
    assert!(matches!(err, RoleDispatchError::InvalidRoutingShape(_)));
}
