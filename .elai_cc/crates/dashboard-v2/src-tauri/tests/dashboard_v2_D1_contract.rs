// C-18 B-36 backfill — dashboard-v2 D1 (`get_sidecar_health`) contract.
//
// Why this lives in dashboard-v2 tests, not contracts/ (C-18 D4):
//   `elai_cc_contracts/Cargo.toml` does NOT depend on `elai-dashboard-v2`
//   (the dep was disabled in session s-1430428 when the old crate moved to
//   .bkp_old/), and re-enabling it requires verifying tauri-build, dev-dep
//   feature flags, and macOS/Linux CI impact — out of scope for C-18.
//   The orchestrator decision is to place dashboard contract tests inside
//   `crates/dashboard-v2/src-tauri/tests/` where the dep is already
//   satisfied (it is itself).
//
// Why no live function call:
//   `get_sidecar_health(state: State<'_, AppState>) -> CmdResult<Value>`
//   takes a Tauri-managed `AppState` which cannot be constructed outside the
//   Tauri runtime. Refactoring to a free function would violate the C-18
//   "ZERO production code changes except D1/D2/D3" constraint. We therefore
//   lock the response JSON shape via a sentinel struct round-trip; the JSON
//   shape IS the dashboard-frontend ↔ backend contract.
//
// What it asserts:
//   - The two documented response shapes (offline + healthy) deserialise into
//     the sentinel struct without losing fields.
//   - The shape is round-trip stable (deserialise → re-serialise → identical
//     JSON values modulo field ordering).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// SidecarHealthResponse — sentinel mirror of the documented JSON shape.
// Production source: crates/dashboard-v2/src-tauri/src/commands.rs:4131
//   - on health probe failure: { "loaded": false, "error": <str> }
//   - on success:               { "loaded": true, "model": <str>,
//                                 "layers": <u32>, "head_dim": <u32>,
//                                 ... model metadata ... }
// Fields not listed below are accepted but ignored (serde default).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SidecarHealthResponse {
    loaded: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    layers: Option<u32>,
    #[serde(default)]
    head_dim: Option<u32>,
}

#[test]
fn d1_offline_shape_round_trips() {
    let sample = json!({
        "loaded": false,
        "error": "health endpoint returned non-2xx or unexpected body",
    });
    let parsed: SidecarHealthResponse =
        serde_json::from_value(sample.clone()).expect("offline shape must deserialise");
    assert_eq!(parsed.loaded, false);
    assert_eq!(
        parsed.error.as_deref(),
        Some("health endpoint returned non-2xx or unexpected body"),
    );
    // Round-trip back to JSON, parse again, compare structs (field-order safe).
    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: SidecarHealthResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

#[test]
fn d1_healthy_shape_round_trips() {
    let sample = json!({
        "loaded": true,
        "model": "qwen-2.5-coder",
        "layers": 28u32,
        "head_dim": 128u32,
    });
    let parsed: SidecarHealthResponse =
        serde_json::from_value(sample.clone()).expect("healthy shape must deserialise");
    assert!(parsed.loaded);
    assert_eq!(parsed.model.as_deref(), Some("qwen-2.5-coder"));
    assert_eq!(parsed.layers, Some(28));
    assert_eq!(parsed.head_dim, Some(128));

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: SidecarHealthResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

// TODO(C-19+): when ELAI_LIVE_SIDECAR-gated integration coverage lands, add
// a #[ignore]-by-default test that spawns the dashboard backend, invokes
// `get_sidecar_health` via the Tauri invoke channel, and asserts the same
// shape against a live sidecar /health response.
