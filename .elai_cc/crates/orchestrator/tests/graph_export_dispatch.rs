//! C-24 — graph-export dispatch contract.
//!
//! The `TOOL_EXPORT_GRAPH_MERMAID` IPC tool arm
//! (crates/orchestrator/src/production_agent_rpc.rs:2841) builds a
//! `VizData` via the knowledge-crate view builders, renders it with
//! `elai_knowledge::visualization::mermaid::generate_mermaid(&data, cap)`,
//! and returns `{"mermaid": "..."}`. These four assertions pin the
//! load-bearing render + envelope contract of that dispatch path WITHOUT a
//! live index (the view builders need a real DKG on disk; the render step
//! and the JSON envelope are the deterministic, index-independent half the
//! dispatch arm relies on).
//!
//! Grounding:
//!   - mermaid.rs:40-52 (header `flowchart LR`, `%% empty graph` on empty)
//!   - types.rs:201-222 (`VizData::empty(name, mode)`)
//!   - production_agent_rpc.rs:2889-2894 (cap + `{"mermaid": ...}` envelope)

use elai_knowledge::visualization::mermaid::generate_mermaid;
use elai_knowledge::visualization::types::{SymbolMode, VizData};

/// Mirror of the dispatch arm's render + envelope step
/// (production_agent_rpc.rs:2889-2894): render the VizData and wrap the
/// string in the `{"mermaid": ...}` JSON object the agent parses.
fn export_envelope(data: &VizData, cap: usize) -> serde_json::Value {
    let mermaid = generate_mermaid(data, cap);
    serde_json::json!({ "mermaid": mermaid })
}

/// Assertion 1: the export dispatch yields an Ok JSON object carrying a
/// string `mermaid` field (the contract the agent decodes).
#[test]
fn export_yields_mermaid_field() {
    let data = VizData::empty("proj".to_string(), SymbolMode::Full);
    let value = export_envelope(&data, 200);
    let mermaid = value
        .get("mermaid")
        .and_then(|v| v.as_str())
        .expect("export envelope must carry a string `mermaid` field");
    assert!(
        !mermaid.is_empty(),
        "generate_mermaid never returns an empty string (mermaid.rs:38)"
    );
}

/// Assertion 2: the rendered diagram starts with a Mermaid graph header
/// (`flowchart` per mermaid.rs:43; the C-24 detail accepts `graph ` or
/// `flowchart`).
#[test]
fn export_starts_with_graph_header() {
    let data = VizData::empty("proj".to_string(), SymbolMode::Full);
    let value = export_envelope(&data, 200);
    let mermaid = value
        .get("mermaid")
        .and_then(|v| v.as_str())
        .expect("mermaid field present");
    assert!(
        mermaid.starts_with("flowchart ") || mermaid.starts_with("graph "),
        "mermaid output must open with a Mermaid graph header, got: {:?}",
        mermaid.lines().next()
    );
}

/// Assertion 3: the render is deterministic — two calls over the same
/// VizData produce byte-identical output (mermaid.rs sorts files/edges for
/// reproducibility; the dispatch path must not leak run-to-run nondeterminism).
#[test]
fn export_is_deterministic_across_two_calls() {
    let data = VizData::empty("proj".to_string(), SymbolMode::Full);
    let first = export_envelope(&data, 200);
    let second = export_envelope(&data, 200);
    assert_eq!(
        first, second,
        "two exports over identical VizData must be byte-identical"
    );
}

/// Assertion 4: the `VizData::empty()` path renders the `%% empty graph`
/// stub-ban marker (mermaid.rs:48-51) rather than a silent blank body, so a
/// no-index dispatch surfaces a valid, explicitly-empty diagram.
#[test]
fn export_empty_vizdata_renders_empty_graph_marker() {
    let data = VizData::empty("proj".to_string(), SymbolMode::Full);
    let value = export_envelope(&data, 200);
    let mermaid = value
        .get("mermaid")
        .and_then(|v| v.as_str())
        .expect("mermaid field present");
    assert!(
        mermaid.contains("%% empty graph"),
        "empty VizData must render the `%% empty graph` marker, got: {mermaid:?}"
    );
}
