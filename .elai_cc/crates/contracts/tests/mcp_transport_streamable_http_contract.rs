//! Contract: MCP Streamable HTTP transport (E-24 Task 4).

use elai_orchestrator::ssb::SsbActionType;

/// Contract 1: SSB action type for legacy SSE has stable SQL string.
#[test]
fn contract_legacy_sse_ssb_type_stable() {
    assert_eq!(
        SsbActionType::McpLegacySseTransport.as_sql_str(),
        "mcp_legacy_sse_transport"
    );
}

/// Contract 2: StreamableHttp is the default transport mode.
#[cfg(feature = "mcp")]
#[test]
fn contract_streamable_http_is_default() {
    use elai_plugins::mcp_bridge::McpTransportMode;
    let default_mode = McpTransportMode::default();
    assert_eq!(default_mode, McpTransportMode::StreamableHttp);
}

/// Contract 3: SseLegacy transport mode is distinct from StreamableHttp.
#[cfg(feature = "mcp")]
#[test]
fn contract_sse_legacy_distinct_from_streamable() {
    use elai_plugins::mcp_bridge::McpTransportMode;
    assert_ne!(McpTransportMode::SseLegacy, McpTransportMode::StreamableHttp);
}
