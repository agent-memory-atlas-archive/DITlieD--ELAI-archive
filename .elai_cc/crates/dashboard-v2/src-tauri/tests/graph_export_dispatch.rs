// C-18 B-33 backfill — export_graph_mermaid_impl dispatch contract.
//
// The plan named this `graph_export_dispatch.rs` and placed it under
// `crates/orchestrator/tests/`. The context-package audit (C-18 §4 DRIFT)
// flagged that `export_graph_mermaid` takes `State<'_, AppState>` and the
// orchestrator crate cannot depend on `elai-dashboard-v2`. The orchestrator
// decision (C-18 D2) was to extract the Tauri-free
// `export_graph_mermaid_impl(project_root, project_id, mode, max_nodes,
//   decomposition_data) -> Result<String>` free function from the command
// body; this test calls that impl directly from the dashboard crate's own
// tests directory (re-exported via `elai_dashboard_v2_lib`).
//
// What it asserts:
//   - For an empty / not-yet-indexed project_root, the "file" mode still
//     returns Ok and produces a non-empty Mermaid string whose header is one
//     of `flowchart` / `graph`. (B-33's mermaid generator emits a "no nodes"
//     placeholder block when input is empty; the string length is always > 10.)
//   - Two consecutive calls return byte-identical strings (determinism
//     contract — the Mermaid renderer must not pick up clock/random state).

use elai_dashboard_v2_lib::export_graph_mermaid_impl;
use tempfile::TempDir;

#[test]
fn graph_export_dispatch_file_mode_returns_mermaid_header() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c18-graph-export";

    let out = export_graph_mermaid_impl(project_root, project_id, "file", Some(50), None)
        .expect("export_graph_mermaid_impl(file, no-index) must succeed");

    assert!(
        out.contains("flowchart") || out.contains("graph"),
        "Mermaid output must start with the `flowchart` or `graph` header; got: {:?}",
        // Limit printed snippet to 200 chars so test output stays scannable.
        out.chars().take(200).collect::<String>(),
    );
    assert!(
        out.len() > 10,
        "Mermaid output must be non-trivial (>10 chars); got len={}",
        out.len(),
    );
}

#[test]
fn graph_export_dispatch_is_deterministic_across_two_calls() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c18-graph-export";

    let first =
        export_graph_mermaid_impl(project_root, project_id, "file", Some(50), None)
            .expect("first call must succeed");
    let second =
        export_graph_mermaid_impl(project_root, project_id, "file", Some(50), None)
            .expect("second call must succeed");

    assert_eq!(
        first, second,
        "export_graph_mermaid_impl must produce byte-identical output across consecutive \
         calls with identical inputs (determinism contract)",
    );
}

#[test]
fn graph_export_dispatch_unknown_mode_returns_err() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();

    let result =
        export_graph_mermaid_impl(project_root, "c18-graph-export", "not-a-mode", None, None);
    assert!(
        result.is_err(),
        "unknown mode must yield an Err; got Ok({:?})",
        result.ok(),
    );
}

// ----------------------------------------------------------------------
// C-24 — additional assertions on top of the C-18 file.
//
// (a) empty-graph file mode includes a `classDef` header (the
//     `knowledge::visualization::mermaid` generator emits one even on
//     the empty VizData path so callers can pipe directly to a Mermaid
//     renderer without checking for a "no nodes" placeholder).
// (b) symbol mode on an unindexed project_root falls through to file
//     mode via `build_symbol_view` (its file_view delegate short-circuits
//     on missing dkg.db) and produces the same valid Mermaid header.
// (c) max_nodes=0 still produces a non-empty Mermaid string (the cap
//     applies after the header is emitted).
// (d) C-24 explicit test for the "no decomposition_data" Err path.
// ----------------------------------------------------------------------

#[test]
fn graph_export_dispatch_empty_graph_has_classdef() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();

    let out = export_graph_mermaid_impl(project_root, "c24-empty", "file", Some(50), None)
        .expect("file mode on empty project must succeed");
    // The B-33 mermaid generator emits the `classDef` block as part of
    // the header even when no files are present — this is the stub_ban
    // contract from the plan (no "empty placeholder" branch silently
    // dropping the header).
    assert!(
        out.contains("classDef"),
        "empty-graph Mermaid output must still contain `classDef` header; got: {:?}",
        out.chars().take(400).collect::<String>(),
    );
}

#[test]
fn graph_export_dispatch_symbol_mode_no_index_returns_mermaid_header() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();

    let out = export_graph_mermaid_impl(project_root, "c24-symbol", "symbol", Some(50), None)
        .expect("symbol mode on empty project must succeed");
    assert!(
        out.contains("flowchart") || out.contains("graph"),
        "symbol mode (no index) must still produce a Mermaid header; got: {:?}",
        out.chars().take(200).collect::<String>(),
    );
}

#[test]
fn graph_export_dispatch_decomposition_without_data_is_err() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();

    // C-24: decomposition mode REQUIRES `decomposition_data: Some(...)`
    // — the impl errors loudly so the caller knows the Tauri command's
    // AppState resolution is the only way to render decomposition view.
    let result =
        export_graph_mermaid_impl(project_root, "c24-decomp", "decomposition", Some(50), None);
    assert!(
        result.is_err(),
        "decomposition mode without decomposition_data must yield an Err; got Ok({:?})",
        result.ok(),
    );
}

#[test]
fn graph_export_dispatch_zero_max_nodes_still_non_empty() {
    // C-24: max_nodes=Some(0) is a pathological caller input. The
    // mermaid generator must still produce at least the header — the
    // cap applies to node emission after the header. Callers that
    // genuinely want zero should pass an explicit empty VizData; this
    // assertion guards against the generator silently returning "".
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();

    let out =
        export_graph_mermaid_impl(project_root, "c24-cap-zero", "file", Some(0), None)
            .expect("file mode with cap=0 must succeed");
    assert!(
        out.len() > 10,
        "Mermaid output with max_nodes=0 must still be non-empty (header survives); got len={}",
        out.len(),
    );
}
