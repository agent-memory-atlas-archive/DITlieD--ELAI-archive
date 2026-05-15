// B-33 Phase B — visualization::types unit tests (TESTING_FIRST gate).
//
// Locks the public surface required by:
//   - get_graph_view_data Tauri command (commands.rs)
//   - GraphWindow.svelte JSON consumer (api/types.ts)
//   - generate_mermaid (visualization::mermaid)
//
// Stub-ban regression: any non-Full SymbolMode MUST carry a populated reason.

use std::collections::BTreeMap;

use elai_knowledge::symbol_graph::types::Confidence;
use elai_knowledge::visualization::types::{
    SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject, VizSymbol,
    VizSymbolEdge, MAX_VIZ_EDGES, MAX_VIZ_SYMBOLS, MAX_VIZ_SYMS_PER_FILE,
};

fn fixture_data() -> VizData {
    let mut symbols_by_file = BTreeMap::new();
    symbols_by_file.insert(
        "src/lib.rs".to_string(),
        vec![VizSymbol {
            id: "src/lib.rs::run".to_string(),
            file: "src/lib.rs".to_string(),
            name: "run".to_string(),
            qualified_name: "lib::run".to_string(),
            kind: "Function".to_string(),
            line: 1,
        }],
    );

    VizData {
        project: VizProject {
            name: "demo".to_string(),
            files: 2,
            edges: 1,
            symbols: Some(1),
            call_edges: Some(0),
            built_at: Some(1_700_000_000_000),
        },
        files: vec![
            VizFile {
                id: "src/lib.rs".to_string(),
                label: "lib.rs".to_string(),
                language: "rust".to_string(),
                deps: 1,
                dependents: 0,
                symbol_count: 1,
            },
            VizFile {
                id: "src/main.rs".to_string(),
                label: "main.rs".to_string(),
                language: "rust".to_string(),
                deps: 0,
                dependents: 1,
                symbol_count: 0,
            },
        ],
        file_edges: vec![VizFileEdge {
            source: "src/main.rs".to_string(),
            target: "src/lib.rs".to_string(),
            kind: VizEdgeKind::Import,
            cyclic: false,
        }],
        symbols: vec![VizSymbol {
            id: "src/lib.rs::run".to_string(),
            file: "src/lib.rs".to_string(),
            name: "run".to_string(),
            qualified_name: "lib::run".to_string(),
            kind: "Function".to_string(),
            line: 1,
        }],
        symbol_edges: vec![VizSymbolEdge {
            source: "src/main.rs::main".to_string(),
            target: "src/lib.rs::run".to_string(),
            confidence: Confidence::Unique,
        }],
        symbols_by_file,
        symbol_mode: SymbolMode::Full,
    }
}

#[test]
fn viz_data_round_trips_through_serde_json() {
    let data = fixture_data();
    let json = serde_json::to_value(&data).expect("serialize VizData");
    let back: VizData = serde_json::from_value(json).expect("deserialize VizData");
    assert_eq!(data, back, "VizData must round-trip via serde_json");
}

#[test]
fn symbol_mode_capped_carries_reason() {
    let mode = SymbolMode::Capped {
        reason: "edge count exceeds 60K cap".to_string(),
    };
    let json = serde_json::to_value(&mode).expect("serialize");
    // Externally tagged: { "Capped": { "reason": "..." } }
    let inner = json
        .get("Capped")
        .expect("Capped variant external tag missing");
    let reason = inner
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("Capped.reason must be a string");
    assert!(!reason.is_empty(), "Capped::reason must be non-empty");
}

#[test]
fn symbol_mode_omitted_carries_reason() {
    let mode = SymbolMode::Omitted {
        reason: "no index yet — run /index".to_string(),
    };
    let json = serde_json::to_value(&mode).expect("serialize");
    let inner = json
        .get("Omitted")
        .expect("Omitted variant external tag missing");
    let reason = inner
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("Omitted.reason must be a string");
    assert!(!reason.is_empty(), "Omitted::reason must be non-empty");
}

#[test]
fn symbol_mode_full_serializes_as_string_or_unit() {
    // Externally-tagged unit variant -> JSON string "Full".
    let mode = SymbolMode::Full;
    let json = serde_json::to_value(&mode).expect("serialize");
    let s = json
        .as_str()
        .expect("SymbolMode::Full must serialize as a JSON string");
    assert_eq!(s, "Full");
}

#[test]
fn symbols_by_file_iterates_sorted() {
    let mut map: BTreeMap<String, Vec<VizSymbol>> = BTreeMap::new();
    map.insert("zeta.rs".to_string(), vec![]);
    map.insert("alpha.rs".to_string(), vec![]);
    map.insert("mu.rs".to_string(), vec![]);
    let keys: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
    assert_eq!(
        keys,
        vec!["alpha.rs", "mu.rs", "zeta.rs"],
        "BTreeMap keys must iterate in lexical order (C2 BTREEMAP gotcha)"
    );
}

#[test]
fn constants_match_spec() {
    assert_eq!(MAX_VIZ_SYMBOLS, 20_000, "B-33 plan caps symbols at 20K");
    assert_eq!(MAX_VIZ_EDGES, 60_000, "B-33 plan caps edges at 60K");
    assert_eq!(
        MAX_VIZ_SYMS_PER_FILE, 200,
        "B-33 plan caps per-file symbols at 200"
    );
}

#[test]
fn viz_data_empty_helper_produces_omitted_with_reason() {
    let data = VizData::empty(
        "nope".to_string(),
        SymbolMode::Omitted {
            reason: "no index yet — run /index".to_string(),
        },
    );
    assert_eq!(data.files.len(), 0);
    assert_eq!(data.file_edges.len(), 0);
    match data.symbol_mode {
        SymbolMode::Omitted { reason } => assert!(reason.contains("/index")),
        other => panic!("expected Omitted, got {:?}", other),
    }
}

#[test]
fn viz_edge_kind_serializes_externally_tagged() {
    let json = serde_json::to_value(VizEdgeKind::Import).expect("serialize");
    assert_eq!(
        json.as_str(),
        Some("Import"),
        "VizEdgeKind::Import must serialize as JSON string \"Import\""
    );
    let json = serde_json::to_value(VizEdgeKind::DynamicImport).expect("serialize");
    assert_eq!(json.as_str(), Some("DynamicImport"));
    let json = serde_json::to_value(VizEdgeKind::ReExport).expect("serialize");
    assert_eq!(json.as_str(), Some("ReExport"));
}
