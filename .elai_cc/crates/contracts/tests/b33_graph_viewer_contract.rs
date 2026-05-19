// B-33 — Graph viewer wire-format contract.
//
// Pins the JSON shape produced by `elai_knowledge::visualization::types::*`
// and the empty-input contract for `generate_mermaid`. Both are consumed by
// the dashboard-v2 GraphWindow, so any silent shape break in Rust would
// surface as broken Svelte deserialization.
//
// What this test guards:
//   - VizData JSON keys (project, files, file_edges, symbols, symbol_edges,
//     symbols_by_file, symbol_mode).
//   - SymbolMode is externally tagged ({"Capped":{"reason":"..."}}, {"Omitted":...}).
//   - export_graph_mermaid (via generate_mermaid) returns a String beginning
//     with "flowchart" even on empty input (stub_ban regression).
//   - VizEdgeKind serialises as a JSON string.

use std::collections::BTreeMap;

use elai_knowledge::visualization::mermaid::generate_mermaid;
use elai_knowledge::visualization::types::{
    SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject,
    MAX_VIZ_EDGES, MAX_VIZ_SYMBOLS, MAX_VIZ_SYMS_PER_FILE,
};

#[test]
fn vizdata_json_shape_pinned() {
    let data = VizData {
        project: VizProject {
            name: "demo".to_string(),
            files: 1,
            edges: 0,
            symbols: Some(1),
            call_edges: Some(0),
            built_at: Some(1),
        },
        files: vec![VizFile {
            id: "lib.rs".to_string(),
            label: "lib.rs".to_string(),
            language: "rust".to_string(),
            deps: 0,
            dependents: 0,
            symbol_count: 1,
            // C-24 (DRIFT-7 Path A).
            level: None,
            status: None,
        }],
        file_edges: vec![],
        symbols: vec![],
        symbol_edges: vec![],
        symbols_by_file: BTreeMap::new(),
        symbol_mode: SymbolMode::Full,
    };

    let json = serde_json::to_value(&data).expect("serialize");
    let obj = json.as_object().expect("VizData must be a JSON object");

    // Every locked key must be present (frontend types.ts depends on them).
    for key in [
        "project",
        "files",
        "file_edges",
        "symbols",
        "symbol_edges",
        "symbols_by_file",
        "symbol_mode",
    ] {
        assert!(
            obj.contains_key(key),
            "VizData JSON missing key `{key}`. Full: {json}"
        );
    }

    // Field-level: VizFile shape.
    let file = obj["files"]
        .as_array()
        .and_then(|a| a.first())
        .expect("first file");
    for key in [
        "id",
        "label",
        "language",
        "deps",
        "dependents",
        "symbol_count",
    ] {
        assert!(
            file.get(key).is_some(),
            "VizFile JSON missing key `{key}`: {file}"
        );
    }
}

#[test]
fn symbolmode_external_tag_serialization() {
    let capped = SymbolMode::Capped {
        reason: "edge cap".to_string(),
    };
    let json = serde_json::to_value(&capped).expect("serialize");
    let inner = json
        .get("Capped")
        .expect("Capped must be externally tagged: {\"Capped\": {\"reason\": ...}}");
    assert_eq!(
        inner
            .get("reason")
            .and_then(|v| v.as_str())
            .expect("reason"),
        "edge cap"
    );

    let full = SymbolMode::Full;
    let json = serde_json::to_value(&full).expect("serialize");
    assert_eq!(
        json.as_str(),
        Some("Full"),
        "SymbolMode::Full must serialise as JSON string \"Full\""
    );
}

#[test]
fn export_graph_mermaid_returns_string_starting_with_flowchart_on_empty() {
    let empty = VizData::empty("nope".to_string(), SymbolMode::Full);
    let out = generate_mermaid(&empty, 200);
    assert!(
        out.starts_with("flowchart"),
        "Mermaid must always begin with `flowchart` (empty-input stub_ban regression). Got: {out}"
    );
    assert!(
        out.contains("classDef"),
        "Mermaid must always emit classDef block (legend stays consistent on empty input). Got: {out}"
    );
}

#[test]
fn viz_edge_kind_serialises_as_string() {
    let v = serde_json::to_value(VizEdgeKind::Import).expect("ser");
    assert_eq!(v.as_str(), Some("Import"));
    let v = serde_json::to_value(VizEdgeKind::DynamicImport).expect("ser");
    assert_eq!(v.as_str(), Some("DynamicImport"));
    let v = serde_json::to_value(VizEdgeKind::ReExport).expect("ser");
    assert_eq!(v.as_str(), Some("ReExport"));
}

#[test]
fn b33_caps_match_dashboard_assumptions() {
    assert_eq!(MAX_VIZ_SYMBOLS, 20_000);
    assert_eq!(MAX_VIZ_EDGES, 60_000);
    assert_eq!(MAX_VIZ_SYMS_PER_FILE, 200);
}

#[test]
fn fileedge_has_cyclic_field() {
    // Frontend renders cyclic edges in red dotted style; the field MUST be
    // present + serialised as a JSON bool. Locking it here guards against
    // a refactor that flattens VizFileEdge.
    let edge = VizFileEdge {
        source: "a".to_string(),
        target: "b".to_string(),
        kind: VizEdgeKind::Import,
        cyclic: true,
    };
    let v = serde_json::to_value(&edge).expect("ser");
    assert_eq!(
        v.get("cyclic").and_then(|x| x.as_bool()),
        Some(true),
        "VizFileEdge.cyclic must serialise as JSON bool"
    );
}
