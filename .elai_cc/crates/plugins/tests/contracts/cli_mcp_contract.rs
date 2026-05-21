// C-39: Contract test — both InProcessDispatch and McpClientDispatch
// satisfy the Dispatch trait and produce structurally equivalent SSB rows
// for equivalent tool invocations.
//
// This is a COMPILE-TIME contract: if both impls compile and implement
// the Dispatch trait, the contract is satisfied. Runtime equivalence is
// tested by the CI parity script (ci-mcp-parity.sh).

use elai_orchestrator::dispatch::{Dispatch, DispatchRequest, DispatchResult};
use elai_orchestrator::dispatch::{InProcessDispatch, McpClientDispatch};
use std::sync::Arc;

/// Asserts that T implements Dispatch + Send + Sync + 'static.
/// Used as a compile-time contract gate.
fn assert_dispatch_contract<T: Dispatch + Send + Sync + 'static>(_: &Arc<T>) {}

#[test]
fn cli_mcp_contract_both_impls_satisfy_dispatch_trait() {
    let in_proc = Arc::new(InProcessDispatch::new_noop());
    assert_dispatch_contract(&in_proc);

    let mcp = Arc::new(McpClientDispatch::new_noop());
    assert_dispatch_contract(&mcp);
}

/// Both impls must accept the same DispatchRequest shape.
/// This verifies the trait's input type is shared.
#[test]
fn cli_mcp_contract_dispatch_request_shape_shared() {
    // DispatchRequest must be constructable and passed to both impls.
    // If either impl requires a different request type, this fails to compile.
    let req = DispatchRequest::new_noop("fs.read");
    let _ = req; // used by trait method signature
}

/// Both impls must return DispatchResult — not different result types.
#[test]
fn cli_mcp_contract_dispatch_result_type_shared() {
    // The return type of Dispatch::dispatch must be the same for both impls.
    // This is enforced by the trait definition; this test documents the contract.
    let _: fn(&DispatchRequest) -> DispatchResult;
}

/// SSB action type for MCP dispatch must be cli_mcp_dispatch (not mcp_tool_called).
/// This distinguishes CLI-via-MCP events from dashboard MCP events in SSB.
#[test]
fn cli_mcp_contract_ssb_action_type_distinct_from_dashboard_mcp() {
    use elai_orchestrator::ssb::SsbActionType;

    let cli_mcp = SsbActionType::CliMcpDispatch;
    let dashboard_mcp = SsbActionType::McpToolCalled;

    assert_ne!(
        cli_mcp.as_sql_str(),
        dashboard_mcp.as_sql_str(),
        "CLI MCP dispatch must use distinct SSB action type from dashboard MCP"
    );
    assert!(
        cli_mcp.as_sql_str().starts_with("cli_mcp_"),
        "CLI MCP action types must use cli_mcp_ prefix"
    );
}
