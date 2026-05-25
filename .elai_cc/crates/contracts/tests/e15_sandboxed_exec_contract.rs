// E-15: Sandboxed-exec wiring contract test.
//
// Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-15_sandboxed_exec.md
// Decisions: F:\.ELAI_workflow\workflow\reasoning\E-15-decisions.md
// Cluster: s-25120 completion-drive session 2 (plan 28 of 28).
//
// This contract proves the wiring-level surface of the E-15 minimum-viable
// slice (M1..M5 + panel + payload extension) shipped this session. All
// assertions are platform-agnostic at compile time and never spawn
// firecracker / runc / iptables — live runtime checks are RUNPOD-PENDING and
// gated by `#[ignore]` in tests/agent_isolation.rs (Linux + Windows).
//
// Contracts checked here:
//   1. FirecrackerHotPool::spawn_from_snapshot has at least one caller in
//      task_runner.rs (closes the 1 NOT-WIRED finding from the RECHECK
//      report — backend used to ship as dead code).
//   2. All 6 SSB payload structs for E-15 exist in ssb.rs at the offsets
//      cited by the integration-explorer (14331..14377 region).
//   3. AgentPoolBabyStepExecutor exposes the `with_firecracker_pool`
//      setter and the field is wired through the constructor (Option<Arc>
//      default None preserves backward compatibility).
//   4. core::sandbox::SandboxConfig surfaces the 3 new sub-table fields
//      (firecracker, gvisor, appcontainer) and the four toml structs
//      (FirecrackerConfigToml, GvisorConfigToml, AppContainerConfigToml,
//      ResearcherNetConfigToml) are publicly reachable.
//   5. The Tauri command `get_sandbox_status` returns a JSON shape carrying
//      the 6 new E-15 keys mirrored in the TypeScript SandboxStatus
//      interface (verified at the source-level — the keys are present in
//      the json!({...}) literal in commands.rs).
//   6. The TS SandboxStatus interface in types.ts lists every E-15 field
//      so that the dashboard build does not type-error.
//   7. SandboxBackendPanel.svelte exists in lib/components/ and the
//      OrchestrationDrawer.svelte imports + mounts it.

use std::sync::Arc;

use elai_core::sandbox::{
    AppContainerConfigToml, FirecrackerConfigToml, GvisorConfigToml, ResearcherNetConfigToml,
    SandboxConfig,
};

// ---------------------------------------------------------------------------
// Contract 1: spawn_from_snapshot is actually called by task_runner.rs.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_firecracker_pool_wired_into_task_runner() {
    let task_runner_src = include_str!("../../orchestrator/src/task_runner.rs");
    let call_count = task_runner_src.matches("spawn_from_snapshot").count();
    assert!(
        call_count >= 1,
        "task_runner.rs has zero callers of FirecrackerHotPool::spawn_from_snapshot; \
         E-15 NOT-WIRED regression. Was wired via with_firecracker_pool() setter + \
         opt-in invocation inside AgentPoolBabyStepExecutor::execute."
    );
    let has_field = task_runner_src.contains("firecracker_pool: Option<Arc<crate::sandbox::FirecrackerHotPool>>");
    assert!(
        has_field,
        "AgentPoolBabyStepExecutor::firecracker_pool field signature drifted; \
         expected `firecracker_pool: Option<Arc<crate::sandbox::FirecrackerHotPool>>`"
    );
    let has_setter = task_runner_src.contains("pub fn with_firecracker_pool");
    assert!(
        has_setter,
        "AgentPoolBabyStepExecutor::with_firecracker_pool setter missing; \
         callers (session_manager + integration tests) need the builder method."
    );
}

// ---------------------------------------------------------------------------
// Contract 2: all 6 E-15 SSB payload structs exist.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_six_ssb_payload_structs_present() {
    let ssb_src = include_str!("../../orchestrator/src/ssb.rs");
    let required_payloads = [
        "pub struct FirecrackerRestoreSlowPayload",
        "pub struct SnapshotUniquenessRegeneratedPayload",
        "pub struct SnapshotEvictedPayload",
        "pub struct SandboxBackendSelectedPayload",
        "pub struct AgentForkedFromSnapshotPayload",
        "pub struct ResearcherNetEgressBlockedPayload",
    ];
    for marker in &required_payloads {
        assert!(
            ssb_src.contains(marker),
            "ssb.rs missing payload struct: `{marker}` — E-15 enum variant ships \
             without a matching Payload struct; SCHEMA bump regression."
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 3: SandboxConfig + 4 toml structs publicly reachable.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_sandbox_config_extended_with_four_subtables() {
    // Default-construct via the public API to prove the type is reachable.
    let cfg = SandboxConfig::default();
    assert!(cfg.firecracker.is_none());
    assert!(cfg.gvisor.is_none());
    assert!(cfg.appcontainer.is_none());

    // The 4 toml structs are constructible via Default.
    let _f: FirecrackerConfigToml = FirecrackerConfigToml::default();
    let _g: GvisorConfigToml = GvisorConfigToml::default();
    let _a: AppContainerConfigToml = AppContainerConfigToml::default();
    let _r: ResearcherNetConfigToml = ResearcherNetConfigToml::default();
}

#[test]
fn e15_contract_firecracker_toml_roundtrips_serde_json() {
    // E-15 elai_cc.toml [sandbox.firecracker] table values.
    let json_input = serde_json::json!({
        "enabled": true,
        "snapshot_dir": ".elai_cc/sessions/{session_id}/snapshots",
        "restore_budget_ms": 200_u64,
        "max_snapshots": 64_u32,
        "max_age_secs": 86_400_u64,
        "uniqueness_mitigation": true,
    });
    let parsed: FirecrackerConfigToml = serde_json::from_value(json_input)
        .expect("FirecrackerConfigToml: must round-trip elai_cc.toml-shaped JSON");
    assert_eq!(parsed.enabled, Some(true));
    assert_eq!(parsed.restore_budget_ms, Some(200));
    assert_eq!(parsed.max_snapshots, Some(64));
    assert_eq!(parsed.max_age_secs, Some(86_400));
    assert_eq!(parsed.uniqueness_mitigation, Some(true));
    assert_eq!(
        parsed.snapshot_dir.as_deref(),
        Some(".elai_cc/sessions/{session_id}/snapshots")
    );
}

// ---------------------------------------------------------------------------
// Contract 4: FirecrackerHotPool can be constructed and handed to the
// executor (compile-time test — no live KVM required).
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_firecracker_pool_constructible_and_arc_wrappable() {
    use std::path::PathBuf;
    let pool = elai_orchestrator::sandbox::FirecrackerHotPool::with_defaults(PathBuf::from(
        "/tmp/elai-e15-contract-snapshots",
    ));
    // Just prove it is Arc-wrappable — task_runner wires it as Arc<...>.
    let _arc: Arc<elai_orchestrator::sandbox::FirecrackerHotPool> = Arc::new(pool);
}

// ---------------------------------------------------------------------------
// Contract 5: get_sandbox_status JSON shape includes 6 new E-15 keys.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_get_sandbox_status_returns_new_keys() {
    // Source-level assertion: the Tauri command body builds json!({...})
    // with the 6 new keys + the 4 B-18 keys. We grep the source file
    // because invoking the live Tauri command requires a Tauri runtime
    // bootstrap that is out of scope for a unit contract test.
    let commands_src = include_str!("../../dashboard-v2/src-tauri/src/commands.rs");
    let required_keys = [
        // B-18 (preserved):
        "\"tier\"",
        "\"profile_count\"",
        "\"allowed_paths_per_agent\"",
        "\"last_visibility_filter_hit_ms\"",
        // E-15 additive:
        "\"backend_selected\"",
        "\"restore_p50_ms\"",
        "\"snapshot_count\"",
        "\"uniqueness_regenerations_count\"",
        "\"researcher_net_allowed_hosts\"",
        "\"researcher_net_blocked_attempts\"",
    ];
    // Slice to the get_sandbox_status function body so the assertion does
    // not get false positives from neighbouring commands.
    let body_start = commands_src
        .find("pub fn get_sandbox_status")
        .expect("commands.rs missing pub fn get_sandbox_status");
    // Find the function-body closing brace. The Tauri command body ends with
    // `\n}\n` at column 0; if multiple braces appear (e.g. nested closures)
    // the FIRST one is the function close because we sliced from the
    // signature. Falls back to end-of-file defensively.
    let suffix = &commands_src[body_start..];
    let rel_end = suffix.find("\n}\n").unwrap_or(suffix.len());
    let body = &suffix[..rel_end];
    for key in &required_keys {
        assert!(
            body.contains(key),
            "get_sandbox_status missing JSON key `{key}`; \
             dashboard SandboxStatus TS interface would silently lose the field. \
             Edit commands.rs::get_sandbox_status to add it back."
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 6: TS SandboxStatus interface mirrors the Rust json! shape.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_ts_sandbox_status_has_new_fields() {
    let types_src = include_str!("../../dashboard-v2/src/lib/api/types.ts");
    let required_fields = [
        "backend_selected: SandboxBackendKind",
        "restore_p50_ms: number | null",
        "snapshot_count: number",
        "uniqueness_regenerations_count: number",
        "researcher_net_allowed_hosts: string[]",
        "researcher_net_blocked_attempts: SandboxResearcherNetBlockedAttempt[]",
    ];
    for field in &required_fields {
        assert!(
            types_src.contains(field),
            "types.ts SandboxStatus missing field declaration `{field}`; \
             cross_lang_diff.py would flag drift between commands.rs json!() \
             and the TS interface. Atomic edit required."
        );
    }
    // The new enum + helper interface also need to be exported.
    assert!(
        types_src.contains("export type SandboxBackendKind"),
        "types.ts missing SandboxBackendKind union"
    );
    assert!(
        types_src.contains("export interface SandboxResearcherNetBlockedAttempt"),
        "types.ts missing SandboxResearcherNetBlockedAttempt interface"
    );
}

// ---------------------------------------------------------------------------
// Contract 7: SandboxBackendPanel.svelte exists and is mounted.
// ---------------------------------------------------------------------------

#[test]
fn e15_contract_sandbox_backend_panel_exists_and_mounted() {
    let panel_src = include_str!(
        "../../dashboard-v2/src/lib/components/SandboxBackendPanel.svelte"
    );
    assert!(panel_src.contains("sandbox-backend-panel"),
        "SandboxBackendPanel.svelte body changed shape; expected .sandbox-backend-panel class root");
    assert!(panel_src.contains("backendLabel"),
        "SandboxBackendPanel.svelte missing backendLabel helper");

    let drawer_src = include_str!(
        "../../dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte"
    );
    assert!(
        drawer_src.contains("import SandboxBackendPanel"),
        "OrchestrationDrawer.svelte does not import SandboxBackendPanel"
    );
    assert!(
        drawer_src.contains("<SandboxBackendPanel"),
        "OrchestrationDrawer.svelte does not mount <SandboxBackendPanel ..>"
    );
}
