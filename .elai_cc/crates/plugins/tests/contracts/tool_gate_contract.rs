//! C-35 contract tests: verify ToolGate interface shape + stub_ban enforcement.
//! WIRING_CONTRACT: render_gated_catalog MUST NOT silently fall through to full catalog.
use elai_plugins::mcp::gate::{GatedCatalog, ToolGate, ToolGateConfig, ToolGateStateConfig, load_gate};
use elai_plugins::McpToolSpec;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

fn sample_catalog() -> Vec<McpToolSpec> {
    vec![
        McpToolSpec { name: "tool_a".into(), description: "Tool A".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "tool_b".into(), description: "Tool B".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "tool_c".into(), description: "Tool C".into(), input_schema: json!({"type":"object"}) },
    ]
}

#[test]
fn tool_gate_contract_interface_shape() {
    let mut states = HashMap::new();
    states.insert("planning".to_string(), ToolGateStateConfig {
        eager: vec!["tool_a".into()],
        gated: vec!["tool_b".into()],
        denied: vec!["tool_c".into()],
    });
    let gate = ToolGate::from_config(ToolGateConfig { states, max_eager_schemas: 4 });
    let catalog = sample_catalog();
    let result: GatedCatalog = gate.filter_for_state("planning", &catalog);
    assert_eq!(result.eager_schemas.len(), 1, "one eager schema");
    assert_eq!(result.eager_schemas[0].name, "tool_a");
    assert_eq!(result.gated_summaries.len(), 1, "one gated summary");
    assert_eq!(result.gated_summaries[0].name, "tool_b");
    assert_eq!(result.gated_summaries[0].summary, "Tool B");
    assert!(result.denied_names.contains("tool_c"), "tool_c must be denied");
}

#[test]
fn tool_gate_contract_stub_ban_no_silent_full_catalog() {
    let absent_path = PathBuf::from("/tmp/definitely_does_not_exist_for_c35.toml");
    let gate = load_gate(&absent_path).expect("load_gate must not fail on absent path");
    let catalog = sample_catalog();
    let result: GatedCatalog = gate.filter_for_state("planning", &catalog);
    assert_eq!(result.eager_schemas.len(), 0, "safe default: no eager schemas");
    assert_eq!(result.denied_names.len(), 0, "safe default: no denied tools");
    assert_eq!(result.gated_summaries.len(), catalog.len(), "safe default: all tools gated");
    assert_eq!(gate.reason(), "manifest_absent", "safe default must report manifest_absent");
}

#[test]
fn tool_gate_contract_denied_is_err_not_panic() {
    let mut states = HashMap::new();
    states.insert("executing".to_string(), ToolGateStateConfig {
        eager: vec![],
        gated: vec![],
        denied: vec!["dangerous_tool".into()],
    });
    let gate = ToolGate::from_config(ToolGateConfig { states, max_eager_schemas: 4 });
    let result = gate.check_tool_allowed("executing", "dangerous_tool");
    assert!(result.is_err(), "denied tool must Err");
    let ok = gate.check_tool_allowed("executing", "safe_tool");
    assert!(ok.is_ok(), "unlisted tool defaults to allowed (gated)");
}
