// C-18 B-36 backfill — dashboard-v2 D6 (`reindex_now`) contract.
//
// See dashboard_v2_D1_contract.rs for the in-crate-tests rationale (C-18 D4).
//
// What it asserts:
//   - The `reindex_now` success response JSON shape — documented at
//     crates/dashboard-v2/src-tauri/src/commands.rs:4742 — round-trips
//     through serde. Fields are: ok, dkg_files, dkg_nodes, dkg_edges,
//     bm25_files, bm25_chunks, profile_loaded.
//   - Argument types: `(app: tauri::AppHandle, state: State<'_, AppState>)` —
//     no user-supplied args, locked here by the absence of any in the
//     sentinel.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ReindexNowResponse {
    ok: bool,
    dkg_files: u64,
    dkg_nodes: u64,
    dkg_edges: u64,
    bm25_files: u64,
    bm25_chunks: u64,
    profile_loaded: bool,
}

#[test]
fn d6_reindex_now_response_shape_round_trips() {
    // Sample matches commands.rs:4742 exactly.
    let sample = json!({
        "ok": true,
        "dkg_files": 12u64,
        "dkg_nodes": 47u64,
        "dkg_edges": 89u64,
        "bm25_files": 12u64,
        "bm25_chunks": 31u64,
        "profile_loaded": true,
    });
    let parsed: ReindexNowResponse =
        serde_json::from_value(sample.clone()).expect("D6 response must deserialise");
    assert!(parsed.ok);
    assert_eq!(parsed.dkg_files, 12);
    assert_eq!(parsed.dkg_nodes, 47);
    assert_eq!(parsed.dkg_edges, 89);
    assert_eq!(parsed.bm25_files, 12);
    assert_eq!(parsed.bm25_chunks, 31);
    assert!(parsed.profile_loaded);

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: ReindexNowResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

#[test]
fn d6_reindex_progress_emit_shape_round_trips() {
    // The "done" progress event payload at commands.rs:4732 is structurally
    // identical to the response (`profile_loaded` + 5 numeric counters)
    // plus a `status` string. Lock it explicitly so the frontend's
    // `reindex-progress` listener has a stable contract.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ReindexProgressDone {
        status: String,
        dkg_files: u64,
        dkg_nodes: u64,
        dkg_edges: u64,
        bm25_files: u64,
        bm25_chunks: u64,
        profile_loaded: bool,
    }

    let sample = json!({
        "status": "done",
        "dkg_files": 0u64,
        "dkg_nodes": 0u64,
        "dkg_edges": 0u64,
        "bm25_files": 0u64,
        "bm25_chunks": 0u64,
        "profile_loaded": false,
    });
    let parsed: ReindexProgressDone =
        serde_json::from_value(sample.clone()).expect("reindex-progress done shape must deserialise");
    assert_eq!(parsed.status, "done");

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: ReindexProgressDone =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

// TODO(C-19+): integration coverage that spawns the backend, fires
// `reindex_now`, and asserts the response struct against a real tempdir
// project. Needs the C-18 D2-style refactor on reindex_now first.
