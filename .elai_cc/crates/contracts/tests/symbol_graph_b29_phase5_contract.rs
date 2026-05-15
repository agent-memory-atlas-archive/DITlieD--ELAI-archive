// B-29 Phase 5 — Contract tests for dispatcher tools, SSB SymbolQuery variant,
// auditor IMPACT_SUMMARY injection, and dashboard Tauri command.
//
// Locks:
//   - TOOL_SYMBOL_IMPACT/FLOW/CONTEXT/LIST constant values in ipc-types
//   - SymbolImpactRequest/SymbolFlowRequest/SymbolContextRequest/SymbolListRequest
//     field names and postcard round-trip
//   - SsbActionType::SymbolQuery variant exists (compile-time check)
//   - SsbPayload::SymbolQuery variant exists (compile-time check)
//   - SymbolQueryPayload fields (tool, target, hop_count, total_files, latency_ms)
//   - SCHEMA_VERSION_PUBLIC == 37
//   - format_impact_summary function signature (B-29 Phase 5 IMPACT_SUMMARY block)
//   - DISPATCH_CONTEXT_BASELINE_TOOLS additive check (symbol_* names present)

use elai_ipc_types::{
    SymbolContextRequest, SymbolFlowRequest, SymbolImpactRequest, SymbolListRequest,
    TOOL_SYMBOL_CONTEXT, TOOL_SYMBOL_FLOW, TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST,
};
use elai_orchestrator::ssb::{
    SsbActionType, SsbPayload, SymbolQueryPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Contract C1: TOOL_SYMBOL_* constant string values are locked.
// ---------------------------------------------------------------------------

#[test]
fn tool_symbol_constant_values_are_locked() {
    assert_eq!(TOOL_SYMBOL_IMPACT, "symbol_impact");
    assert_eq!(TOOL_SYMBOL_FLOW, "symbol_flow");
    assert_eq!(TOOL_SYMBOL_CONTEXT, "symbol_context");
    assert_eq!(TOOL_SYMBOL_LIST, "symbol_list");
}

// ---------------------------------------------------------------------------
// Contract C2: Request structs have the correct fields (compile-time check).
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _assert_request_fields() {
    // SymbolImpactRequest
    let _: fn() -> SymbolImpactRequest = || SymbolImpactRequest {
        project_root: String::new(),
        target: String::new(),
        depth: 2,
    };
    // SymbolFlowRequest
    let _: fn() -> SymbolFlowRequest = || SymbolFlowRequest {
        project_root: String::new(),
        entrypoint_id: String::new(),
        depth: 3,
    };
    // SymbolContextRequest
    let _: fn() -> SymbolContextRequest = || SymbolContextRequest {
        project_root: String::new(),
        name: String::new(),
        file_hint: None,
    };
    // SymbolListRequest
    let _: fn() -> SymbolListRequest = || SymbolListRequest {
        project_root: String::new(),
        file: None,
        query: None,
        limit: 100,
    };
}

// ---------------------------------------------------------------------------
// Contract C3: Postcard round-trip for SymbolImpactRequest.
// ---------------------------------------------------------------------------

#[test]
fn symbol_impact_request_postcard_round_trip() {
    let req = SymbolImpactRequest {
        project_root: "/projects/my_crate".to_string(),
        target: "crates/foo/src/lib.rs".to_string(),
        depth: 2,
    };
    let bytes = postcard::to_allocvec(&req).expect("serialize");
    let decoded: SymbolImpactRequest = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded, req);
}

// ---------------------------------------------------------------------------
// Contract C4: Postcard round-trip for SymbolFlowRequest.
// ---------------------------------------------------------------------------

#[test]
fn symbol_flow_request_postcard_round_trip() {
    let req = SymbolFlowRequest {
        project_root: "/projects/my_crate".to_string(),
        entrypoint_id: "crates/foo/src/lib.rs::compute_impact_radius".to_string(),
        depth: 3,
    };
    let bytes = postcard::to_allocvec(&req).expect("serialize");
    let decoded: SymbolFlowRequest = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded, req);
}

// ---------------------------------------------------------------------------
// Contract C5: Postcard round-trip for SymbolContextRequest.
// ---------------------------------------------------------------------------

#[test]
fn symbol_context_request_postcard_round_trip() {
    let req = SymbolContextRequest {
        project_root: "/projects/my_crate".to_string(),
        name: "compute_impact_radius".to_string(),
        file_hint: Some("crates/knowledge/src/symbol_graph/traversal.rs".to_string()),
    };
    let bytes = postcard::to_allocvec(&req).expect("serialize");
    let decoded: SymbolContextRequest = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded, req);
}

// ---------------------------------------------------------------------------
// Contract C6: Postcard round-trip for SymbolListRequest.
// ---------------------------------------------------------------------------

#[test]
fn symbol_list_request_postcard_round_trip() {
    let req = SymbolListRequest {
        project_root: "/projects/my_crate".to_string(),
        file: Some("crates/foo/src/lib.rs".to_string()),
        query: Some("compute".to_string()),
        limit: 50,
    };
    let bytes = postcard::to_allocvec(&req).expect("serialize");
    let decoded: SymbolListRequest = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded, req);
}

// ---------------------------------------------------------------------------
// Contract C7: SsbActionType::SymbolQuery variant exists and maps to string.
// ---------------------------------------------------------------------------

#[test]
fn ssb_action_type_symbol_query_as_sql_str() {
    let action = SsbActionType::SymbolQuery;
    assert_eq!(action.as_sql_str(), "symbol_query");
}

// ---------------------------------------------------------------------------
// Contract C8: ssb_action_type_from_str round-trip for "symbol_query".
// ---------------------------------------------------------------------------

#[test]
fn ssb_action_type_from_str_symbol_query() {
    let action = elai_orchestrator::ssb::ssb_action_type_from_str_public("symbol_query")
        .expect("from_str must succeed for 'symbol_query'");
    assert_eq!(action, SsbActionType::SymbolQuery);
}

// ---------------------------------------------------------------------------
// Contract C9: SymbolQueryPayload fields (compile-time check).
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _assert_symbol_query_payload_fields() {
    let p = SymbolQueryPayload {
        tool: "symbol_impact".to_string(),
        target: "src/lib.rs".to_string(),
        hop_count: 2,
        total_files: 5,
        latency_ms: 10,
    };
    let _tool: &str = &p.tool;
    let _target: &str = &p.target;
    let _hop_count: u32 = p.hop_count;
    let _total_files: u32 = p.total_files;
    let _latency_ms: u32 = p.latency_ms;
}

// ---------------------------------------------------------------------------
// Contract C10: SsbPayload::SymbolQuery variant exists and action_type matches.
// ---------------------------------------------------------------------------

#[test]
fn ssb_payload_symbol_query_action_type_discriminant() {
    let payload = SsbPayload::SymbolQuery(SymbolQueryPayload {
        tool: "symbol_impact".to_string(),
        target: "src/lib.rs".to_string(),
        hop_count: 2,
        total_files: 3,
        latency_ms: 11,
    });
    assert_eq!(payload.action_type(), SsbActionType::SymbolQuery);
}

// ---------------------------------------------------------------------------
// Contract C11: SCHEMA_VERSION_PUBLIC == 37.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_37() {
    // B-29 Phase 5 bumps SCHEMA_VERSION from 36 to 37 for SymbolQuery variant.
    // Subsequent B-steps (B-32 MinifiedFileChunked at 38) may push the value
    // higher; assert the floor, not exact equality, so this test does not
    // break when a sibling batch lands first.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 37,
        "B-29 Phase 5 requires SCHEMA_VERSION >= 37 (SymbolQuery variant); got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// Contract C12: format_impact_summary produces the IMPACT_SUMMARY block shape.
// ---------------------------------------------------------------------------

#[test]
fn format_impact_summary_block_shape() {
    use elai_orchestrator::task_runner::format_impact_summary;
    use elai_knowledge::symbol_graph::traversal::ImpactResult;
    use std::collections::BTreeMap;

    let mut files_by_depth = BTreeMap::new();
    files_by_depth.insert(
        0u32,
        vec!["crates/foo/src/lib.rs".to_string()],
    );
    files_by_depth.insert(
        1u32,
        vec![
            "crates/bar/src/main.rs".to_string(),
            "crates/baz/src/util.rs".to_string(),
        ],
    );
    files_by_depth.insert(
        2u32,
        vec!["crates/qux/src/api.rs".to_string()],
    );

    let result = ImpactResult {
        target: "crates/foo/src/lib.rs".to_string(),
        resolved_files: vec![
            "crates/foo/src/lib.rs".to_string(),
            "crates/bar/src/main.rs".to_string(),
            "crates/baz/src/util.rs".to_string(),
            "crates/qux/src/api.rs".to_string(),
        ],
        files_by_depth,
        total_files: 4,
        max_depth_reached: 2,
        truncated: false,
    };

    let summary = format_impact_summary("crates/foo/src/lib.rs", &result);
    assert!(summary.contains("IMPACT_SUMMARY"), "must start with IMPACT_SUMMARY header");
    assert!(summary.contains("Changed: crates/foo/src/lib.rs"), "must name the changed file");
    assert!(summary.contains("Hop 1:"), "must include Hop 1 band");
    assert!(summary.contains("Hop 2:"), "must include Hop 2 band");
    assert!(summary.contains("Truncated: false"), "must show truncation status");
}

// ---------------------------------------------------------------------------
// Contract C13: DISPATCH_CONTEXT_BASELINE_TOOLS additive — symbol_* names
// present in the tool list used by task_runner for SSB tools_allowed count.
// ---------------------------------------------------------------------------

#[test]
fn dispatch_context_baseline_tools_includes_symbol_tools() {
    use elai_orchestrator::task_runner::DISPATCH_CONTEXT_BASELINE_TOOLS_PUBLIC;

    let names = DISPATCH_CONTEXT_BASELINE_TOOLS_PUBLIC;
    assert!(
        names.contains(&"symbol_impact"),
        "DISPATCH_CONTEXT_BASELINE_TOOLS must include 'symbol_impact'"
    );
    assert!(
        names.contains(&"symbol_flow"),
        "DISPATCH_CONTEXT_BASELINE_TOOLS must include 'symbol_flow'"
    );
    assert!(
        names.contains(&"symbol_context"),
        "DISPATCH_CONTEXT_BASELINE_TOOLS must include 'symbol_context'"
    );
    assert!(
        names.contains(&"symbol_list"),
        "DISPATCH_CONTEXT_BASELINE_TOOLS must include 'symbol_list'"
    );
}
