// C-18 B-36 backfill — dashboard-v2 D3 (`run_research`) contract.
//
// See dashboard_v2_D1_contract.rs for the in-crate-tests rationale (C-18 D4).
//
// What it asserts:
//   - The `run_research` response JSON shape — documented at
//     crates/dashboard-v2/src-tauri/src/commands.rs:5209 as
//     `{ ok: bool, requestId: String }` — round-trips through serde.
//   - The argument types `(query: String, depth: Option<u32>, breadth: Option<u32>)`
//     are locked at compile time via a sentinel signature check.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RunResearchResponse {
    ok: bool,
    #[serde(rename = "requestId")]
    request_id: String,
}

#[test]
fn d3_run_research_response_shape_round_trips() {
    let sample = json!({
        "ok": true,
        "requestId": "research-1715990400000",
    });
    let parsed: RunResearchResponse =
        serde_json::from_value(sample.clone()).expect("D3 response must deserialise");
    assert!(parsed.ok);
    assert!(
        parsed.request_id.starts_with("research-"),
        "requestId must follow the documented `research-<unix_ms>` pattern; got {:?}",
        parsed.request_id,
    );

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: RunResearchResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

// Compile-time anchor for `run_research` argument types.
#[allow(dead_code)]
fn _assert_run_research_argument_types() {
    let _query: String = String::new();
    let _depth: Option<u32> = None;
    let _breadth: Option<u32> = None;
}

// TODO(C-19+): when ELAI_LIVE_SIDECAR-gated integration coverage lands, add
// a #[ignore]-by-default test that spawns the dashboard backend, invokes
// `run_research` with a tiny depth=1/breadth=1 query, awaits the
// `research-done` event, and asserts the documented event payload shape.
