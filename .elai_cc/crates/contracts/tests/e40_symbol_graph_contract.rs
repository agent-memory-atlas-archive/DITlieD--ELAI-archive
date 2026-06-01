//! E-40 Tranche B-2 -- SymbolGraph DSL + Dataflow + Refine+Filter +
//! 5th-list RRF + 2 ToolDefinitions contract tests.
//!
//! Pins the behaviour of the new substrate end-to-end:
//!   * `SCHEMA_VERSION_PUBLIC` reflects the E-40 Tranche B-2 bump
//!     (192 -> 193).
//!   * `SsbActionType::SymbolGraphEvent` round-trips via
//!     `ssb_action_type_from_str_public`.
//!   * `SymbolGraphEventKind` 4-arm discriminator covers
//!     symbol_dsl_executed / dataflow_impact_computed /
//!     agent_impact_claim_refuted / scope_graph_resolved.
//!   * `parse_dsl` accepts the v1 grammar (DEF / USE / IMPL / CALL).
//!   * `DataflowAnalyzer::impact_analysis` honours `depth_cap`.
//!   * `RefineStage::default` uses the two-iteration fixed-weight
//!     schedule (alpha=0.6, beta=0.4).
//!   * 5th-list RRF is wire-dark by default (env gate off).
//!   * The two new ToolDefinition wire IDs match the canonical strings.
//!
//! NO cargo test execution by the main LLM; structural shape mirrors
//! crates/contracts/tests/e47_lora_ship_gate_contract.rs.

use elai_ipc_types::{
    SymbolDataflowImpactRequest, SymbolDslQueryRequest, TOOL_SYMBOL_DATAFLOW_IMPACT,
    TOOL_SYMBOL_DSL_QUERY,
};
use elai_orchestrator::ssb::{
    SymbolGraphEventKind, SymbolGraphEventPayload, ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV,
};
use elai_orchestrator::symbol_graph::{
    parse_dsl, refine, DataflowAnalyzer, DslFilter, DslSelector, RefineStage,
    SymbolGraphCache, SymbolNode,
};
use elai_orchestrator::symbol_graph::query::SymbolMatch;
use elai_orchestrator::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Contract -- SCHEMA_VERSION reflects the E-40 Tranche B-2 bump.
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_193_after_e40_tranche_b2() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 193,
        "schema only drifts forward; this contract was written at version 193. \
         CONTRACT: SCHEMA_VERSION_PUBLIC must be >= 193 after the \
         E-40 Tranche B-2 SymbolGraphEvent bump (192 -> 193). Update \
         ssb.rs ledger comments + \
         action_type_lock.golden together on the next bump."
    );
}

// ---------------------------------------------------------------------------
// Contract -- the new SsbActionType variant round-trips via from_str.
// ---------------------------------------------------------------------------

#[test]
fn contract_symbol_graph_event_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("symbol_graph_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::SymbolGraphEvent,
        "CONTRACT: ssb_action_type_from_str_public(\"symbol_graph_event\") \
         must yield SsbActionType::SymbolGraphEvent"
    );
}

// ---------------------------------------------------------------------------
// Contract -- SymbolGraphEventKind serde round-trip + as_str labels.
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_graph_event_kind_round_trip() {
    let cases = [
        (SymbolGraphEventKind::SymbolDslExecuted, "symbol_dsl_executed"),
        (
            SymbolGraphEventKind::DataflowImpactComputed,
            "dataflow_impact_computed",
        ),
        (
            SymbolGraphEventKind::AgentImpactClaimRefuted,
            "agent_impact_claim_refuted",
        ),
        (
            SymbolGraphEventKind::ScopeGraphResolved,
            "scope_graph_resolved",
        ),
    ];
    for (kind, expected_wire) in &cases {
        assert_eq!(
            kind.as_str(),
            *expected_wire,
            "CONTRACT: SymbolGraphEventKind::{:?}.as_str() must equal {:?}",
            kind,
            expected_wire
        );
        let json = serde_json::to_string(kind).expect("serialize");
        let expected_json = format!("\"{}\"", expected_wire);
        assert_eq!(
            json, expected_json,
            "CONTRACT: SymbolGraphEventKind::{:?} JSON must equal {:?}",
            kind, expected_json
        );
        let decoded: SymbolGraphEventKind =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            decoded, *kind,
            "CONTRACT: SymbolGraphEventKind JSON must round-trip"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract -- SymbolGraphEventPayload round-trips via serde_json.
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_graph_event_payload_round_trip() {
    let payload = SymbolGraphEventPayload {
        kind: SymbolGraphEventKind::DataflowImpactComputed,
        session_id: Some("s-e40-b2".into()),
        agent_id: Some("symbol_graph_tool".into()),
        details_json: serde_json::json!({
            "symbol": "foo",
            "depth_cap": 2,
            "impacted_count": 3
        }),
        ts_ms: 1_700_000_193_001,
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: SymbolGraphEventPayload =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.kind, payload.kind);
    assert_eq!(decoded.session_id, payload.session_id);
    assert_eq!(decoded.ts_ms, payload.ts_ms);
}

// ---------------------------------------------------------------------------
// Contract -- DSL parser accepts the v1 grammar.
// ---------------------------------------------------------------------------

#[test]
fn test_dsl_parser_basic() {
    let ast = parse_dsl("DEF(foo)").expect("DEF(foo) parses");
    assert_eq!(ast.selector, DslSelector::Def("foo".into()));
    assert!(ast.filters.is_empty());

    let ast = parse_dsl("USE(bar) | lang(rust) | depth(2)").expect("filters parse");
    assert_eq!(ast.selector, DslSelector::Use("bar".into()));
    assert_eq!(ast.filters.len(), 3);
    assert!(matches!(ast.filters[2], DslFilter::Depth(2)));

    let ast = parse_dsl("CALL(handle_request)").expect("CALL parses");
    assert_eq!(ast.selector, DslSelector::Call("handle_request".into()));

    let ast = parse_dsl("IMPL(MyTrait)").expect("IMPL parses");
    assert_eq!(ast.selector, DslSelector::Impl("MyTrait".into()));

    assert!(parse_dsl("").is_err(), "CONTRACT: empty DSL is rejected");
    assert!(
        parse_dsl("NOTHING(x)").is_err(),
        "CONTRACT: unknown selector is rejected"
    );
}

// ---------------------------------------------------------------------------
// Contract -- DataflowAnalyzer BFS honours depth_cap.
// ---------------------------------------------------------------------------

fn build_test_cache() -> SymbolGraphCache {
    let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty cache");
    cache.index_node(SymbolNode {
        id: "src/a.rs:caller:10".into(),
        name: "caller".into(),
        file: "src/a.rs".into(),
        line: 10,
        kind: "function".into(),
        calls: vec!["src/b.rs:mid:20".into()],
    });
    cache.index_node(SymbolNode {
        id: "src/b.rs:mid:20".into(),
        name: "mid".into(),
        file: "src/b.rs".into(),
        line: 20,
        kind: "function".into(),
        calls: vec!["src/c.rs:leaf:30".into()],
    });
    cache.index_node(SymbolNode {
        id: "src/c.rs:leaf:30".into(),
        name: "leaf".into(),
        file: "src/c.rs".into(),
        line: 30,
        kind: "function".into(),
        calls: vec![],
    });
    cache
}

#[test]
fn test_dataflow_analyzer_bfs_depth_cap() {
    let mut cache = build_test_cache();
    let mut a = DataflowAnalyzer::new(&mut cache, 1);
    let out_depth_1 = a.impact_analysis("leaf");
    assert!(
        out_depth_1.iter().any(|s| s.contains("mid")),
        "CONTRACT: depth=1 surfaces direct caller (mid)"
    );
    assert!(
        !out_depth_1.iter().any(|s| s.contains("caller")),
        "CONTRACT: depth=1 must NOT reach 2-hop ancestor (caller)"
    );
    let mut cache = build_test_cache();
    let mut a = DataflowAnalyzer::new(&mut cache, 2);
    let out_depth_2 = a.impact_analysis("leaf");
    assert!(
        out_depth_2.iter().any(|s| s.contains("caller")),
        "CONTRACT: depth=2 surfaces 2-hop ancestor (caller)"
    );
}

// ---------------------------------------------------------------------------
// Contract -- RefineStage uses the two-iteration fixed-weight schedule.
// ---------------------------------------------------------------------------

fn sm(id: &str, name: &str) -> SymbolMatch {
    SymbolMatch {
        id: id.into(),
        name: name.into(),
        file: format!("{name}.rs"),
        line: 1,
        kind: "function".into(),
    }
}

#[test]
fn test_refine_stage_two_iter() {
    let stage = RefineStage::default();
    assert_eq!(
        stage.iters, 2,
        "CONTRACT: RefineStage::default().iters must equal 2 per the \
         RANGER two-step empirical schedule. Bumping this value requires \
         updating workflow/reasoning/E-40-tranche-B2-decisions.md."
    );
    assert!(
        (stage.alpha - 0.6).abs() < f64::EPSILON,
        "CONTRACT: alpha = 0.6"
    );
    assert!(
        (stage.beta - 0.4).abs() < f64::EPSILON,
        "CONTRACT: beta = 0.4"
    );
    let init = vec![sm("a:1", "alpha"), sm("b:2", "beta")];
    let dsl = vec![sm("a:1", "alpha"), sm("c:3", "gamma")];
    let out = refine(init, dsl);
    assert_eq!(
        out[0].id, "a:1",
        "CONTRACT: a-symbol appears in BOTH lists so co-occurrence bonus \
         must push it to the top"
    );
    assert_eq!(out.len(), 3, "CONTRACT: refined union has 3 distinct ids");
}

// ---------------------------------------------------------------------------
// Contract -- 5th-list RRF is wire-dark (env gate off) by default.
// ---------------------------------------------------------------------------

#[test]
fn test_5th_rrf_disabled_by_default() {
    let prior = std::env::var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV).ok();
    std::env::remove_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV);
    let gate_on =
        elai_knowledge::pipeline::symbol_graph_5th_rrf_enabled_from_env();
    assert!(
        !gate_on,
        "CONTRACT: ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE unset must yield wire-dark \
         (false) from symbol_graph_5th_rrf_enabled_from_env"
    );
    if let Some(v) = prior {
        std::env::set_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV, v);
    }
}

// ---------------------------------------------------------------------------
// Contract -- the two new ToolDefinition wire IDs match the canonical strings.
// ---------------------------------------------------------------------------

#[test]
fn test_new_tool_wire_ids_match_canonical_strings() {
    assert_eq!(
        TOOL_SYMBOL_DSL_QUERY, "symbol_dsl_query",
        "CONTRACT: TOOL_SYMBOL_DSL_QUERY wire ID must equal \"symbol_dsl_query\""
    );
    assert_eq!(
        TOOL_SYMBOL_DATAFLOW_IMPACT, "symbol_dataflow_impact",
        "CONTRACT: TOOL_SYMBOL_DATAFLOW_IMPACT wire ID must equal \"symbol_dataflow_impact\""
    );
}

// ---------------------------------------------------------------------------
// Contract -- request/response postcard round-trip.
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_dsl_query_request_postcard_round_trip() {
    let req = SymbolDslQueryRequest {
        project_root: "/tmp/proj".into(),
        dsl: "DEF(foo) | lang(rust)".into(),
        depth_cap: 2,
    };
    let bytes = postcard::to_allocvec(&req).expect("encode");
    let decoded: SymbolDslQueryRequest = postcard::from_bytes(&bytes).expect("decode");
    assert_eq!(decoded.project_root, req.project_root);
    assert_eq!(decoded.dsl, req.dsl);
    assert_eq!(decoded.depth_cap, req.depth_cap);
}

#[test]
fn test_symbol_dataflow_impact_request_postcard_round_trip() {
    let req = SymbolDataflowImpactRequest {
        project_root: "/tmp/proj".into(),
        symbol: "foo".into(),
        depth_cap: 3,
    };
    let bytes = postcard::to_allocvec(&req).expect("encode");
    let decoded: SymbolDataflowImpactRequest =
        postcard::from_bytes(&bytes).expect("decode");
    assert_eq!(decoded.symbol, req.symbol);
    assert_eq!(decoded.depth_cap, req.depth_cap);
}
