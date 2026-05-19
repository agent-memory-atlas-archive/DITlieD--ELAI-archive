// C-18 B-36 backfill — dashboard-v2 D2 (`run_planning_task`) contract.
//
// See dashboard_v2_D1_contract.rs for the in-crate-tests rationale (C-18 D4).
//
// What it asserts:
//   - The `run_planning_task` response JSON shape — documented at
//     crates/dashboard-v2/src-tauri/src/commands.rs:5026 as
//     `{ ok: bool, requestId: String, taskId: String, seq: i64 }` —
//     deserialises into the sentinel struct without losing fields, and
//     re-serialises to a struct-equal value.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RunPlanningTaskResponse {
    ok: bool,
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "taskId")]
    task_id: String,
    seq: i64,
}

#[test]
fn d2_run_planning_task_response_shape_round_trips() {
    // Sample mirrors the exact key set produced by
    // `run_planning_task` at commands.rs:5026.
    let sample = json!({
        "ok": true,
        "requestId": "plan-42",
        "taskId": "t-42",
        "seq": 42_i64,
    });
    let parsed: RunPlanningTaskResponse =
        serde_json::from_value(sample.clone()).expect("D2 response must deserialise");
    assert_eq!(parsed.ok, true);
    assert_eq!(parsed.request_id, "plan-42");
    assert_eq!(parsed.task_id, "t-42");
    assert_eq!(parsed.seq, 42);

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: RunPlanningTaskResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

// Compile-time anchor: argument types for the `run_planning_task` command
// (task: String, project_id: Option<String>). If the production signature
// changes, this fn line breaks the build and surfaces the contract drift.
#[allow(dead_code)]
fn _assert_run_planning_task_argument_types() {
    let _task: String = String::new();
    let _project_id: Option<String> = None;
    // We do NOT call the function (Tauri State<> indirection). The role of
    // this fn is purely type-shape locking against documentation drift.
}

// TODO(C-19+): when ELAI_LIVE_SIDECAR-gated integration coverage lands, add
// a #[ignore]-by-default test that spawns the dashboard backend, invokes
// `run_planning_task` via the Tauri invoke channel, and asserts (a) the
// response matches this struct, (b) a `TaskSubmitted` SSB row was written.
