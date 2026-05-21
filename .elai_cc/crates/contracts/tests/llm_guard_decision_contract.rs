//! Contract: LLM-guard PreToolUse hook (E-24 Task 5).

use elai_orchestrator::ssb::SsbActionType;

/// Contract 1: SSB action type for guard block has stable SQL string.
#[test]
fn contract_llm_guard_blocked_ssb_type_stable() {
    assert_eq!(
        SsbActionType::McpLlmGuardBlocked.as_sql_str(),
        "mcp_llm_guard_blocked"
    );
}

/// Contract 2: Default config is fail-closed.
#[cfg(feature = "mcp")]
#[test]
fn contract_default_config_fail_closed() {
    use elai_plugins::llm_guard::LlmGuardConfig;
    let cfg = LlmGuardConfig::default();
    assert!(cfg.fail_closed);
    assert_eq!(cfg.sidecar_url, "http://localhost:8100");
}

/// Contract 3: GuardDecision variants are correctly typed.
#[cfg(feature = "mcp")]
#[test]
fn contract_guard_decision_types() {
    use elai_plugins::llm_guard::GuardDecision;
    let allow = GuardDecision::Allow;
    let deny = GuardDecision::Deny { reason: "rm -rf detected".to_string() };
    let ask = GuardDecision::Ask { reason: "ambiguous".to_string() };

    // Verify each variant is distinct
    assert_ne!(allow, deny);
    assert_ne!(allow, ask);
    assert_ne!(deny, ask);

    // Verify deny reason is preserved
    if let GuardDecision::Deny { reason } = &deny {
        assert_eq!(reason, "rm -rf detected");
    } else {
        panic!("expected Deny variant");
    }
}
