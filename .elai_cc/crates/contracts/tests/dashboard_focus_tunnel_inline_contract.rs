//! B-28 follow-up — focus-tunnel inline chips on PlanWindow baby-step.
//!
//! Replaces `dashboard_focus_tunnel_panel_contract.rs` (the dedicated
//! FocusTunnelPanel side-panel surface). The user's mental model is that
//! the focus tunnel is middleware — the dashboard's surface for it should
//! be inline chips/badges on the baby-step that triggered each beacon,
//! not a sibling panel in MonitorWindow.
//!
//! Locks the single replacement Tauri command
//! `query_focus_tunnel_per_baby_step` (commands.rs + lib.rs registration)
//! and the orchestrator-side filter helper this command relies on
//! (`SsbViewer::focus_tunnel_payloads` step_id filtering, exercised
//! end-to-end against a synthetic SSB).
//!
//! Drops the 5 panel-tier commands from the surface assertion — if they
//! re-appear in commands.rs the dashboard has regressed the refactor.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::focus_tunnel::{
    mode_distribution_for_payloads, GenerationMode,
};
use elai_orchestrator::{
    GenerationModeSelectedPayload, MicroStepCreatedPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SsbViewer, StreamingEarlyTerminatedPayload,
};
use tempfile::TempDir;

// `include_str!` bakes the file content at compile time so a missing or
// stripped file fails the build. The string-search assertions below then
// catch partial strips that still compile.
const DASHBOARD_LIB_RS: &str =
    include_str!("../../dashboard-v2/src-tauri/src/lib.rs");
const DASHBOARD_COMMANDS_RS: &str =
    include_str!("../../dashboard-v2/src-tauri/src/commands.rs");

/// The single replacement command. Frontend invokes it once per visible
/// baby-step with `{ babyStepId: <u64> }` and renders the result as
/// inline chips: Mode, Early-term, Micro-steps, Deps.
const REQUIRED_COMMAND: &str = "query_focus_tunnel_per_baby_step";

/// The 5 panel-era commands that MUST be removed by this refactor. If any
/// re-appears, this test fails — preventing accidental reintroduction.
const REMOVED_COMMANDS: &[&str] = &[
    "query_focus_tunnel_mode_distribution",
    "query_focus_tunnel_streaming_stats",
    "query_focus_tunnel_pre_assembly_stats",
    "query_focus_tunnel_dep_resolution",
    "query_focus_tunnel_micro_steps",
];

// ---------------------------------------------------------------------------
// Surface assertions — Tauri command registration + body presence.
// ---------------------------------------------------------------------------

#[test]
fn per_baby_step_command_registered_in_invoke_handler() {
    let needle = format!("commands::{REQUIRED_COMMAND}");
    assert!(
        DASHBOARD_LIB_RS.contains(&needle),
        "lib.rs invoke_handler must register '{REQUIRED_COMMAND}'; missing token '{needle}'"
    );
}

#[test]
fn per_baby_step_command_body_exists_in_commands_rs() {
    let needle = format!("pub async fn {REQUIRED_COMMAND}(");
    let alt_needle = format!("pub fn {REQUIRED_COMMAND}(");
    assert!(
        DASHBOARD_COMMANDS_RS.contains(&needle)
            || DASHBOARD_COMMANDS_RS.contains(&alt_needle),
        "commands.rs must define '{REQUIRED_COMMAND}' as a Tauri command"
    );
}

#[test]
fn old_panel_commands_are_removed() {
    for cmd in REMOVED_COMMANDS {
        let registration = format!("commands::{cmd}");
        assert!(
            !DASHBOARD_LIB_RS.contains(&registration),
            "lib.rs must NOT still register the panel-era command '{cmd}'; \
             token '{registration}' found — refactor reverted?"
        );
        let body = format!("pub async fn {cmd}(");
        let alt = format!("pub fn {cmd}(");
        assert!(
            !DASHBOARD_COMMANDS_RS.contains(&body) && !DASHBOARD_COMMANDS_RS.contains(&alt),
            "commands.rs must NOT still define the panel-era command '{cmd}'; \
             body found — refactor reverted?"
        );
    }
}

// ---------------------------------------------------------------------------
// Behaviour assertion — drive synthetic SSB rows through the orchestrator
// helper the new command uses, verify per-baby-step filtering produces the
// expected per-step chip data (mode, early_term, micro_step count).
//
// Note: DependencyResolvedPayload has no step_id field (verified in
// ssb.rs:2614). Per-step dep stats are therefore returned as task-level
// fallback counts; this contract pins the per-step types that DO carry a
// step identifier and ignores deps in the synthetic fixture.
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "dashboard-inline".into(),
        session_id: "session-A".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn per_baby_step_filter_isolates_step_specific_rows() {
    let (_dir, path) = fresh_db();
    let writer = open_writer(&path).await;

    // Two baby steps. Step 7 picks FIM with one early-term row + 2 micro-steps.
    // Step 8 picks DIFF, no early-term, no micro-steps.
    let mode_7 = SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
        step_id: "baby_step_7".into(),
        mode: "fim".into(),
        rule_fired: "fim_all_conditions_met".into(),
        reason: "rule_3_fim".into(),
        preferred_was: None,
        supports_fim: true,
        exec_mode: "local".into(),
    });
    let mode_8 = SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
        step_id: "baby_step_8".into(),
        mode: "diff".into(),
        rule_fired: "multi_region_diff".into(),
        reason: "rule_4_diff".into(),
        preferred_was: None,
        supports_fim: true,
        exec_mode: "local".into(),
    });
    let term_7 = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "baby_step_7".into(),
        mode: "fim".into(),
        reason: "lazy_pattern".into(),
        reason_detail: Some("pattern_id=todo".into()),
        token_count_at_term: 42,
        total_tokens_received: 42,
        language_hint: "rust".into(),
    });
    let ms_7a = SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
        parent_step_id: 7,
        created_path: "src/foo.rs".into(),
        type_hint: "Function".into(),
        reason: "undefined_symbol".into(),
        count: 1,
    });
    let ms_7b = SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
        parent_step_id: 7,
        created_path: "src/bar.rs".into(),
        type_hint: "Struct".into(),
        reason: "undefined_symbol".into(),
        count: 1,
    });

    for payload in [mode_7, mode_8, term_7, ms_7a, ms_7b] {
        writer
            .write_payload("dashboard", 0, payload, vec![])
            .await
            .expect("ssb write");
    }
    drop(writer);

    let viewer = SsbViewer::open(&path).expect("viewer open");

    // Mode rows — fetch all GenerationModeSelected then filter caller-side
    // by step_id (the per-baby-step command's intended logic).
    let mode_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::GenerationModeSelected],
            None,
            500,
        )
        .expect("mode fetch");
    let step_7_mode: Vec<&SsbPayload> = mode_rows
        .iter()
        .filter(|p| match p {
            SsbPayload::GenerationModeSelected(m) => m.step_id == "baby_step_7",
            _ => false,
        })
        .collect();
    assert_eq!(step_7_mode.len(), 1, "step 7 must have exactly 1 mode row");
    let dist = mode_distribution_for_payloads(&step_7_mode);
    assert_eq!(dist.get(&GenerationMode::Fim).copied().unwrap_or(0), 1);
    assert_eq!(dist.get(&GenerationMode::Diff).copied().unwrap_or(0), 0);

    // Early-term rows — same filter pattern, scoped to step 7.
    let term_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::StreamingEarlyTerminated],
            None,
            500,
        )
        .expect("term fetch");
    let step_7_term_count = term_rows
        .iter()
        .filter(|p| match p {
            SsbPayload::StreamingEarlyTerminated(t) => t.step_id == "baby_step_7",
            _ => false,
        })
        .count();
    let step_8_term_count = term_rows
        .iter()
        .filter(|p| match p {
            SsbPayload::StreamingEarlyTerminated(t) => t.step_id == "baby_step_8",
            _ => false,
        })
        .count();
    assert_eq!(step_7_term_count, 1, "step 7 must have 1 early-term row");
    assert_eq!(step_8_term_count, 0, "step 8 must have 0 early-term rows");

    // Micro-step rows — parent_step_id filter, summed via `count` field.
    let micro_rows = viewer
        .focus_tunnel_payloads(&[SsbActionType::MicroStepCreated], None, 500)
        .expect("micro fetch");
    let step_7_micro_total: u32 = micro_rows
        .iter()
        .filter_map(|p| match p {
            SsbPayload::MicroStepCreated(m) if m.parent_step_id == 7 => Some(u32::from(m.count)),
            _ => None,
        })
        .sum();
    let step_8_micro_total: u32 = micro_rows
        .iter()
        .filter_map(|p| match p {
            SsbPayload::MicroStepCreated(m) if m.parent_step_id == 8 => Some(u32::from(m.count)),
            _ => None,
        })
        .sum();
    assert_eq!(step_7_micro_total, 2, "step 7 must have 2 micro-steps");
    assert_eq!(step_8_micro_total, 0, "step 8 must have 0 micro-steps");
}

#[test]
fn old_focus_tunnel_panel_svelte_file_removed() {
    // Spec marker: the dedicated panel component must be deleted by this
    // refactor. We probe relative to CARGO_MANIFEST_DIR (the contracts
    // crate) so the test is path-stable across local + CI checkouts.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let panel_path = std::path::Path::new(manifest_dir)
        .join("..")
        .join("dashboard-v2")
        .join("src")
        .join("lib")
        .join("components")
        .join("FocusTunnelPanel.svelte");
    assert!(
        !panel_path.exists(),
        "FocusTunnelPanel.svelte must be deleted by the inline-chips refactor; \
         still present at {}",
        panel_path.display()
    );
}
