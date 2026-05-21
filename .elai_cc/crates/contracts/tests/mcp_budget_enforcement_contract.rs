//! Contract: MCP per-tool token-budget enforcement (E-24 Task 3).

use elai_orchestrator::ssb::SsbActionType;

/// Contract 1: SSB action type has stable SQL string.
#[test]
fn contract_budget_ssb_type_stable() {
    assert_eq!(
        SsbActionType::McpBudgetEnforced.as_sql_str(),
        "mcp_budget_enforced"
    );
}

// Budget enforcement uses elai_plugins::mcp_budget which is mcp-feature-gated.
#[cfg(feature = "mcp")]
mod gated_budget {
    use elai_plugins::mcp_budget::{enforce, BudgetedTool, TokenBudget};

    /// Contract 2: 20-tool fixture with budget=2000 tokens drops the lowest-priority tail.
    #[test]
    fn contract_20_tool_budget_drops_tail() {
        // Each tool schema is 1000 chars -> 250 tokens. Budget 2000 -> keep 8 tools.
        let mut tools: Vec<BudgetedTool> = (0..20)
            .map(|i| BudgetedTool {
                name: format!("tool_{i:02}"),
                schema_json: "x".repeat(1000),
                priority: u32::MAX,
            })
            .collect();

        let budget = TokenBudget {
            budget_tokens: 2000,
            priority_order: (0..20).map(|i| format!("tool_{i:02}")).collect(),
        };

        let report = enforce(&mut tools, &budget);
        assert_eq!(tools.len(), 8, "expected 8 kept tools");
        assert_eq!(report.dropped.len(), 12, "expected 12 dropped tools");
        assert!(report.kept_tokens <= 2000, "kept_tokens must not exceed budget");

        // Dropped tools must be the tail (indices 8-19)
        for dropped in &report.dropped {
            let idx: usize = dropped.trim_start_matches("tool_").parse().unwrap();
            assert!(idx >= 8, "only tail tools should be dropped, got {dropped}");
        }
    }

    /// Contract 3: priority order is respected -- high-priority tool kept over low-priority.
    #[test]
    fn contract_priority_respected() {
        let mut tools = vec![
            BudgetedTool {
                name: "critical_tool".to_string(),
                schema_json: "x".repeat(500), // 125 tokens
                priority: u32::MAX,
            },
            BudgetedTool {
                name: "optional_tool".to_string(),
                schema_json: "x".repeat(500), // 125 tokens
                priority: u32::MAX,
            },
        ];

        let budget = TokenBudget {
            budget_tokens: 130, // room for only one tool
            priority_order: vec!["critical_tool".to_string()],
        };

        let report = enforce(&mut tools, &budget);
        assert!(tools.iter().any(|t| t.name == "critical_tool"), "high-priority tool must be kept");
        assert!(report.dropped.contains(&"optional_tool".to_string()), "low-priority tool must be dropped");
    }

    /// Contract 4: no drops when budget is sufficient.
    #[test]
    fn contract_no_drops_when_under_budget() {
        let mut tools: Vec<BudgetedTool> = (0..5)
            .map(|i| BudgetedTool {
                name: format!("tool_{i}"),
                schema_json: "x".repeat(100), // 25 tokens each = 125 total
                priority: u32::MAX,
            })
            .collect();

        let budget = TokenBudget {
            budget_tokens: 1000,
            priority_order: vec![],
        };

        let report = enforce(&mut tools, &budget);
        assert!(report.dropped.is_empty(), "no drops when under budget");
        assert_eq!(tools.len(), 5);
    }
}
