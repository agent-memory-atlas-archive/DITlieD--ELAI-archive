// C-24 (Task 9, DECISION-10) — Tauri command smoke tests.
//
// Exercise the C-24-extracted impl-fns `get_graph_view_data_impl` and
// `get_impact_overlay_impl` (re-exported from
// crates/dashboard-v2/src-tauri/src/lib.rs) — the same code path the
// Tauri commands run after they resolve the State<>-bound inputs.
//
// Why not the Tauri commands themselves: `State<'_, AppState>` cannot be
// constructed outside a Tauri runtime. C-18 D2 set the precedent of
// extracting a Tauri-free `*_impl(project_root, project_id, ...)` free
// function and calling it from integration tests. C-24 DECISION-10
// applies the same pattern to `get_graph_view_data` + `get_impact_overlay`.
//
// Assertions:
//   (a) file mode VizData on an empty / unindexed project returns Ok
//       (placeholder VizData with `SymbolMode::Omitted{reason}`); files
//       array may be empty but the call MUST NOT panic.
//   (b) impact_overlay on an unindexed project returns Ok with the
//       documented empty shape `{files:[], by_depth:{}, total_files:0, ...}`.
//   (c) symbol mode on an unindexed project still returns Ok (delegates
//       through file view which short-circuits on missing dkg.db).
//   (d) all three calls compose without panic on an empty project fixture.

use elai_dashboard_v2_lib::{get_graph_view_data_impl, get_impact_overlay_impl};
use tempfile::TempDir;

#[test]
fn graph_commands_file_mode_returns_ok_on_empty_project() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let data = get_graph_view_data_impl(project_root, project_id, "file", None)
        .expect("file mode must succeed on an unindexed project (returns placeholder)");
    // The placeholder is built via `views::empty_viz(...)` which calls
    // `VizData::empty(...)` — its files vec is empty by construction.
    // We assert the project_id round-trips and the symbol_mode carries a
    // reason (the stub_ban contract from the plan).
    assert_eq!(
        data.project.name, project_id,
        "VizData.project.name must round-trip the project_id"
    );
    // SymbolMode::Omitted carries a reason string — assert via JSON
    // serialisation so we don't pull in the enum's private discriminants.
    let json =
        serde_json::to_value(&data).expect("VizData must serialize for smoke-test inspection");
    let symbol_mode = json
        .get("symbol_mode")
        .expect("VizData must publish symbol_mode field");
    // The Omitted variant serialises as {"Omitted": {"reason": "..."}}.
    let omitted = symbol_mode
        .get("Omitted")
        .expect("symbol_mode must be Omitted on an unindexed project");
    let reason = omitted
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("Omitted.reason must be a populated string");
    assert!(
        !reason.is_empty(),
        "SymbolMode::Omitted.reason must be non-empty on an unindexed project"
    );
}

#[test]
fn graph_commands_impact_overlay_empty_shape_on_unindexed() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let overlay = get_impact_overlay_impl(project_root, project_id, "any_symbol", 3)
        .expect("impact_overlay must return Ok (empty shape) on an unindexed project");
    // Documented empty shape per get_impact_overlay_impl source.
    let obj = overlay
        .as_object()
        .expect("impact overlay must be a JSON object");
    assert!(obj.contains_key("files"));
    assert!(obj.contains_key("by_depth"));
    assert!(obj.contains_key("total_files"));
    assert!(obj.contains_key("max_depth_reached"));
    assert!(obj.contains_key("truncated"));
    assert_eq!(
        obj.get("total_files").and_then(|v| v.as_u64()),
        Some(0),
        "unindexed project must yield total_files == 0"
    );
}

#[test]
fn graph_commands_symbol_mode_returns_ok_on_empty_project() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let data = get_graph_view_data_impl(project_root, project_id, "symbol", None)
        .expect("symbol mode must succeed on an unindexed project");
    assert_eq!(
        data.project.name, project_id,
        "VizData.project.name must round-trip the project_id for symbol mode too"
    );
}

#[test]
fn graph_commands_three_calls_no_panic_on_empty_fixture() {
    // C-24: defense-in-depth — run all three Tauri-free calls back-to-back
    // on the same empty fixture so no shared-state regression panics one
    // call after another succeeds.
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let _file = get_graph_view_data_impl(project_root, project_id, "file", None)
        .expect("file mode must succeed");
    let _symbol = get_graph_view_data_impl(project_root, project_id, "symbol", None)
        .expect("symbol mode must succeed");
    let _overlay = get_impact_overlay_impl(project_root, project_id, "target_xyz", 3)
        .expect("impact_overlay must succeed");
}

#[test]
fn graph_commands_get_graph_view_data_impl_decomposition_requires_data() {
    // C-24 (DECISION-10): decomposition mode is a Tauri-only surface
    // because it needs AppState's DecompositionTreeSnapshot. Calling the
    // impl directly without supplying decomposition_data MUST error.
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let result =
        get_graph_view_data_impl(project_root, project_id, "decomposition", None);
    assert!(
        result.is_err(),
        "decomposition mode without decomposition_data must yield Err; got Ok({:?})",
        result.ok().map(|d| d.project.name),
    );
}

#[test]
fn graph_commands_get_graph_view_data_impl_unknown_mode_is_err() {
    let dir = TempDir::new().expect("project tempdir");
    let project_root = dir.path();
    let project_id = "c24-graph-commands";

    let result =
        get_graph_view_data_impl(project_root, project_id, "not-a-mode", None);
    assert!(
        result.is_err(),
        "unknown mode must yield Err; got Ok({:?})",
        result.ok().map(|d| d.project.name),
    );
}
