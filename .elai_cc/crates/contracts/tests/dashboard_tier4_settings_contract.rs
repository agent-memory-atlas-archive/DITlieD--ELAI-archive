#![cfg(any())] // disabled: depends on removed elai-dashboard crate (s-1430428 workspace-manifest-fix)
// Dashboard Tier 4 Settings reorg — contract tests
// (dashboard-wiring-plan §8).
//
// Surface-presence + round-trip assertions for the 6 new Tauri commands
// (set_execution_mode, get_execution_mode, set_vram_budget, get_cloud_spend,
// trigger_reindex, trigger_dep_audit), the new SsbPayload variant
// (ExecutionModeChanged), and the two DashboardState fields added by this
// tier (execution_mode, vram_budget_mb). Pairs with tier4-settings-backend.mjs
// on the frontend / invoke-handler side.
//
// Same discipline as Tier 3: compile-time imports assert the public surface;
// serde round-trip pins JSON field shapes; no new crates introduced (I3C-9).
//
// Intentional non-goals:
//   - no mount-style Tauri state harness (DEF-ALL-1 still open);
//   - no CLOUD/HYBRID dispatch assertions (§9 group 20 deferral);
//   - no disk-persistence checks on vram_budget_mb (runtime-only state by
//     design — D2-1 runtime-join precedent).

use elai_dashboard_lib::commands::{
    get_cloud_spend, get_execution_mode, set_execution_mode, set_vram_budget,
    trigger_dep_audit, trigger_reindex, CloudSpendSnapshot, ProviderSpend,
};
use elai_orchestrator::{ExecutionModeChangedPayload, SsbPayload};

#[test]
fn tier4_command_symbols_are_callable() {
    // Compile-time surface check — each of the 6 Tauri commands must exist
    // at the expected crate path. Rename or removal breaks the build, same
    // pattern as Tier 3 contracts.
    let _ = &set_execution_mode;
    let _ = &get_execution_mode;
    let _ = &set_vram_budget;
    let _ = &get_cloud_spend;
    let _ = &trigger_reindex;
    let _ = &trigger_dep_audit;
}

#[test]
fn cloud_spend_snapshot_round_trips_through_serde_json() {
    // The Tauri IPC encoder uses serde_json. Tier 4 ships the snapshot as a
    // permanent graceful-empty stub (`tracked: false`), so pinning the
    // field names here guards against a silent rename on the frontend
    // side in Settings.svelte.
    let stub = CloudSpendSnapshot {
        total_usd: 0.0,
        last_24h_usd: 0.0,
        per_provider: Vec::new(),
        tracked: false,
    };
    let encoded = serde_json::to_string(&stub).expect("encode CloudSpendSnapshot");
    assert!(encoded.contains("\"total_usd\":0.0"));
    assert!(encoded.contains("\"last_24h_usd\":0.0"));
    assert!(encoded.contains("\"per_provider\":[]"));
    assert!(encoded.contains("\"tracked\":false"));

    let populated = CloudSpendSnapshot {
        total_usd: 12.34,
        last_24h_usd: 5.67,
        per_provider: vec![ProviderSpend {
            provider: "anthropic".to_owned(),
            total_usd: 11.00,
        }],
        tracked: true,
    };
    let encoded = serde_json::to_string(&populated).expect("encode populated");
    assert!(encoded.contains("\"tracked\":true"));
    assert!(encoded.contains("\"provider\":\"anthropic\""));
    assert!(encoded.contains("\"total_usd\":11"));
}

#[test]
fn execution_mode_changed_payload_round_trips() {
    // SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload { .. })
    // must round-trip through the same msgpack-backed persistence layer
    // already covered by ssb.rs's prop-test. This test only asserts the
    // struct fields are what Settings.svelte expects to read back via
    // SsbViewer (from_mode / to_mode / project_id / changed_at_ms).
    let payload = ExecutionModeChangedPayload {
        from_mode: "LOCAL".to_owned(),
        to_mode: "CLOUD".to_owned(),
        project_id: "proj-1".to_owned(),
        changed_at_ms: 1_713_300_001_000,
    };
    let encoded = serde_json::to_string(&payload).expect("encode payload");
    assert!(encoded.contains("\"from_mode\":\"LOCAL\""));
    assert!(encoded.contains("\"to_mode\":\"CLOUD\""));
    assert!(encoded.contains("\"project_id\":\"proj-1\""));
    assert!(encoded.contains("\"changed_at_ms\":1713300001000"));

    let decoded: ExecutionModeChangedPayload =
        serde_json::from_str(&encoded).expect("decode payload");
    assert_eq!(decoded.from_mode, "LOCAL");
    assert_eq!(decoded.to_mode, "CLOUD");
    assert_eq!(decoded.project_id, "proj-1");
    assert_eq!(decoded.changed_at_ms, 1_713_300_001_000);
}

#[test]
fn ssb_payload_variant_discriminant_is_stable() {
    // Wrap the payload in the enum variant actually persisted to SSB. If
    // the variant name ever changes, the enum rename would silently drop
    // pre-existing msgpack rows; this assertion keeps the variant pinned
    // to ExecutionModeChanged.
    let payload = SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
        from_mode: "HYBRID".to_owned(),
        to_mode: "LOCAL".to_owned(),
        project_id: "p".to_owned(),
        changed_at_ms: 0,
    });
    let encoded = serde_json::to_string(&payload).expect("encode variant");
    assert!(
        encoded.contains("ExecutionModeChanged"),
        "expected variant tag in serialised SsbPayload; got: {encoded}"
    );
}
