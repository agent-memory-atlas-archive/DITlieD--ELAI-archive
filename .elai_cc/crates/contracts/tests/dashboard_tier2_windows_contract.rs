#![cfg(any())] // disabled: depends on removed elai-dashboard crate (s-1430428 workspace-manifest-fix)
// Dashboard Tier 2 Track B — backend-extending windows contract tests.
//
// Covers the data contract additions introduced by the Tier 2 Track B
// surface (4 windows + 8 tauri commands + 3 SsbViewer methods + 1 dev-server
// registry + 1 explore-result cache). The compile-time imports double as a
// surface-presence check: if any of the 8 commands or 3 SsbViewer methods
// are missing or renamed the test fails to build, mirroring the Tier 1
// `dashboard_tier1_chat_contract.rs` precedent.
//
// Runtime checks:
//   1. Each new DTO round-trips through serde_json with snake_case keys
//      matching the spec (frontend consumers depend on these names).
//   2. SsbViewerError gains a NotFound variant with `kind` + `id` fields
//      so commands can map "explore_id unknown" / "preview_id unknown" /
//      "baby_step_id unknown" to a single failure mode without bespoke
//      strings (the variant is justified by the four NotFound call sites
//      in commands.rs Tier 2 additions).
//   3. SsbViewer::recent_preview_available returns Ok(empty) on a fresh DB
//      (negative path for the runtime-join PreviewState design — no rows
//      means no DevServerRegistry lookup needed).
//
// Plan §13:832 forbids assuming a sidecar route exists; this test does NOT
// touch the sidecar. The /search re-route used by submit_explore_query is
// validated by tier2-windows-backend.mjs (regression guard) and exercised
// at runtime only when the dashboard is actually running.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_dashboard_lib::commands::{
    get_baby_step_detail, get_decomposition_tree, get_explore_result,
    get_monitor_snapshot, get_phase_detail, get_preview_state,
    stop_dev_server, submit_explore_query, BabyStepDetail,
    DecompositionTreeSnapshot, ExploreResult, MonitorSnapshot,
    PhaseDetail, PreviewState, PreviewStatus,
};
use elai_orchestrator::{Ssb, SsbConfig, SsbViewer, SsbViewerError};
use tempfile::TempDir;

fn fresh_ssb(tmp: &TempDir) -> Ssb {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "contract-project".into(),
        session_id: "contract-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Ssb::new(cfg, clock).expect("open fresh ssb")
}

#[test]
fn tier2_command_symbols_are_callable() {
    // Compile-time surface check: taking the address of each command
    // asserts the symbol exists at the expected crate path. The `let _ =`
    // bindings prevent dead-code warnings without coercing the function
    // through a fn-pointer type (which would brittlely depend on the
    // `tauri::State<'_, T>` lifetime + tauri::command codegen wrapping).
    let _ = &get_decomposition_tree;
    let _ = &get_phase_detail;
    let _ = &get_baby_step_detail;
    let _ = &get_monitor_snapshot;
    let _ = &submit_explore_query;
    let _ = &get_explore_result;
    let _ = &get_preview_state;
    let _ = &stop_dev_server;
}

// Tier 2 audit BLOCK 3 fix: tests that construct an `Ssb` need a tokio
// runtime because `Ssb::new` spawns the writer task. The previous `#[test]`
// annotation panicked at runtime with "no reactor running". Same fix
// applied to `baby_step_detail_phase_detail_unknown_id_returns_not_found`
// below. Mirrors the established multi-thread pattern in
// `crates/orchestrator/src/ssb_viewer.rs:1364`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ssb_viewer_tier2_methods_are_callable() {
    // Compile-time surface check on SsbViewer::{decomposition_tree,
    // phase_detail, baby_step_detail, recent_preview_available}.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    // decomposition_tree(None) on a fresh DB must return Ok(empty snapshot).
    let tree = viewer
        .decomposition_tree(None)
        .expect("decomposition_tree on empty DB");
    assert!(
        tree.concepts.is_empty(),
        "decomposition_tree on empty DB must return empty concepts; got {} entries",
        tree.concepts.len()
    );

    // recent_preview_available on empty DB must return Ok(empty Vec).
    let previews = viewer
        .recent_preview_available("contract-project", 16)
        .expect("recent_preview_available on empty DB");
    assert_eq!(previews.len(), 0, "no preview_available rows on fresh DB");
}

#[test]
fn ssb_viewer_error_has_not_found_variant() {
    // The Track B commands map "unknown id" failures (explore_id,
    // preview_id, baby_step_id, phase_id) onto SsbViewerError::NotFound.
    // Constructing the variant here pins its public shape; the discriminant
    // pattern-match below pins the variant name + payload field names.
    let err = SsbViewerError::NotFound {
        kind: "baby_step".into(),
        id: "baby_step_missing".into(),
    };
    match err {
        SsbViewerError::NotFound { kind, id } => {
            assert_eq!(kind, "baby_step");
            assert_eq!(id, "baby_step_missing");
        }
        other => panic!("expected SsbViewerError::NotFound, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn baby_step_detail_phase_detail_unknown_id_returns_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");

    // Negative-path: phase_detail with an id that doesn't exist must NOT
    // panic and must surface NotFound (the Tauri command then maps to
    // an Err string).
    match viewer.phase_detail("phase_does_not_exist") {
        Err(SsbViewerError::NotFound { .. }) => {}
        Err(other) => panic!("expected NotFound, got {other:?}"),
        Ok(_) => panic!("expected Err on unknown phase_id"),
    }
    match viewer.baby_step_detail("baby_step_does_not_exist") {
        Err(SsbViewerError::NotFound { .. }) => {}
        Err(other) => panic!("expected NotFound, got {other:?}"),
        Ok(_) => panic!("expected Err on unknown baby_step_id"),
    }
}

#[test]
fn decomposition_tree_snapshot_serde_snake_case() {
    let snapshot = DecompositionTreeSnapshot::default();
    let v = serde_json::to_value(&snapshot).expect("serialize");
    let map = v.as_object().expect("object root");
    assert!(map.contains_key("concepts"), "DecompositionTreeSnapshot must expose snake_case 'concepts'");
    assert!(
        map.contains_key("generated_at_ms"),
        "DecompositionTreeSnapshot must expose snake_case 'generated_at_ms'"
    );
}

#[test]
fn phase_detail_optional_fields_serde_shape() {
    let detail = PhaseDetail::empty("phase-x".to_string());
    let v = serde_json::to_value(&detail).expect("serialize");
    let map = v.as_object().expect("object root");
    for k in ["phase_id", "description", "expected_outcome", "dependencies", "estimated_steps_remaining"] {
        assert!(map.contains_key(k), "PhaseDetail missing snake_case key '{k}'");
    }
}

#[test]
fn baby_step_detail_serde_shape() {
    let detail = BabyStepDetail::empty("baby_step-y".to_string());
    let v = serde_json::to_value(&detail).expect("serialize");
    let map = v.as_object().expect("object root");
    for k in ["baby_step_id", "task", "estimated_tokens", "concept_name", "phase_name", "step_name", "tdd", "acceptance"] {
        assert!(map.contains_key(k), "BabyStepDetail missing snake_case key '{k}'");
    }
}

#[test]
fn monitor_snapshot_empty_default_serde_shape() {
    let snap = MonitorSnapshot::empty();
    let v = serde_json::to_value(&snap).expect("serialize");
    let map = v.as_object().expect("object root");
    for k in ["identity", "progress", "status", "interventions", "timeline"] {
        assert!(map.contains_key(k), "MonitorSnapshot missing snake_case key '{k}'");
    }
}

#[test]
fn explore_result_serde_shape() {
    let r = ExploreResult {
        status: "ready".into(),
        answer: "n/a".into(),
        sources: Vec::new(),
        citations: Vec::new(),
    };
    let v = serde_json::to_value(&r).expect("serialize");
    let map = v.as_object().expect("object root");
    for k in ["status", "answer", "sources", "citations"] {
        assert!(map.contains_key(k), "ExploreResult missing snake_case key '{k}'");
    }
}

#[test]
fn preview_state_serde_shape_with_status_enum() {
    let s = PreviewState { slots: Vec::new() };
    let v = serde_json::to_value(&s).expect("serialize");
    let map = v.as_object().expect("object root");
    assert!(map.contains_key("slots"), "PreviewState missing snake_case 'slots'");

    // PreviewStatus is a snake_case-tagged enum so the frontend can match on
    // the literal strings "starting" | "ready" | "error" | "stopped".
    let starting = serde_json::to_value(&PreviewStatus::Starting).expect("ser starting");
    let ready = serde_json::to_value(&PreviewStatus::Ready).expect("ser ready");
    let err = serde_json::to_value(&PreviewStatus::Error).expect("ser error");
    let stopped = serde_json::to_value(&PreviewStatus::Stopped).expect("ser stopped");
    assert_eq!(starting, serde_json::Value::String("starting".into()));
    assert_eq!(ready, serde_json::Value::String("ready".into()));
    assert_eq!(err, serde_json::Value::String("error".into()));
    assert_eq!(stopped, serde_json::Value::String("stopped".into()));
}
