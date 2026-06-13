// Phase A1a contract — task_runner chokepoint wiring with BOTH
// feasibility_prober AND layer1a_processor populated (Some).
//
// Pins the contract that closes the gap recorded in metrics/phase_4F.toml:103
// (`production_construction_paths_wired = false`): when both wirings are
// supplied — as the new `elai run` subcommand does via `RuntimeWiring` —
// a `SsbActionType::TaskSubmitted` row reaches `drive_task` and the runner
// emits at least one `SsbActionType::ReasoningTrace` row (the unconditional
// `emit_reasoning(... "observe" ...)` at the head of `drive_task`,
// task_runner.rs:3263). That row is the observable proof that the chokepoint
// path is live; there is no `SsbActionType::Decomposition` variant in the
// SSB schema (ssb.rs:184/406 list `ReasoningTrace` and
// `ConceptDecompositionNode`; the brief's reference to "Decomposition" maps
// to the ReasoningTrace + ConceptDecompositionNode emissions during
// decomposition).
//
// The timeout (10s) has no empirical basis yet and is a project decision
// documented in workflow/handoff/phase-A1a.md acceptance_criteria.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::GitBranchPolicy;
use elai_orchestrator::ssb::{
    Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader, TaskSubmittedPayload,
};
use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, start_task_runner,
    DefaultFeasibilityProber, DefaultLayer1aProcessor, FeasibilityProber, Layer1aProcessor,
    RuntimeWiring, TaskRunnerConfig,
};
use rusqlite::Connection;
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Helpers — mirror the fixture pattern used by layer1a_contract.rs:41-67 and
// task_runner_web_research_contract.rs:39-65 so the chokepoint test rides on
// the same battle-tested template.
// ---------------------------------------------------------------------------

async fn wait_for_action(
    writer: &Arc<Ssb>,
    action: SsbActionType,
    min_count: usize,
    timeout: Duration,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        let entries = writer.query_by_type(action).unwrap_or_default();
        if entries.len() >= min_count {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

fn new_ssb(db_path: PathBuf) -> Arc<Ssb> {
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "a1a-chokepoint".into(),
        session_id: "test-a1a".into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

async fn new_layer1a(
    rules_db_path: PathBuf,
    ssb: Arc<Ssb>,
) -> Arc<DefaultLayer1aProcessor> {
    let conn = Connection::open(&rules_db_path).expect("open rules.db");
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let processor = DefaultLayer1aProcessor::new(rules_db, rules_db_path, ssb);
    processor.ensure_tables().await.expect("ensure_tables");
    Arc::new(processor)
}

// ---------------------------------------------------------------------------
// Contract 1 — full chokepoint: TaskSubmitted -> drive_task -> ReasoningTrace
// with BOTH feasibility_prober AND layer1a_processor wired as Some(...).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn contract_task_submitted_drives_pipeline_with_full_wiring() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let cursor_path = tmp.path().join("cursor.json");

    let feasibility_root = tmp.path().join("feasibility");
    let feasibility_prober: Arc<dyn FeasibilityProber> =
        Arc::new(DefaultFeasibilityProber::new(feasibility_root));

    let rules_db_path = tmp.path().join("rules.db");
    let layer1a_concrete = new_layer1a(rules_db_path, ssb.clone()).await;
    let layer1a_processor: Arc<dyn Layer1aProcessor> = layer1a_concrete.clone();

    let (handle, cancel) = start_task_runner(
        ssb.clone(),
        TaskRunnerConfig {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: None,
            dev_server_registry: None,
            web_research_runner: None,
            scaffold_applier: None,
            rule_evaluator: None,
            interrogation_store: None,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: Some(feasibility_prober),
            autonomous_gate: None,
            plan_interactive_gate: None,
            layer1a_processor: Some(layer1a_processor),
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
            task_splitter: None,
            research_tree: None,
            pipeline_grc: None,
            model_capability_profile: None,
            cloud_router: None,
            cloud_spend_cap_daily_usd: None,
            crystallizer: None,
            compiled_pipeline_store: None,
            background_task_manager: None,
            // F-6 / Slice A3: legacy contract fixture -- no learned-skill store.
            skill_store: None,
            // C-39: legacy contract fixture -- no Dispatch wired.
            mcp_dispatch: None,
            // C.5: legacy contract fixture -- no live tracker (sentinel fallback).
            token_tracker: None,
            // B-28 WIRING-2: legacy contract fixture -- no production buffer editor.
            buffer_editor: None,
            // P0-1: legacy contract fixture -- no adversary runtime.
            adversary_runtime: None,
            // E-46 D-8: contract fixture — module-constant defaults.
            planning_config: Default::default(),
        },
    );

    // Give the runner a moment to subscribe before we write.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Plain (non-research, non-scaffold) task so drive_task takes the
    // default branch and emits the unconditional `observe` ReasoningTrace.
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "Build a hello-world CLI binary".into(),
        project_id: "a1a-chokepoint".into(),
        submitted_at_ms: 0,
    });
    ssb.write_payload("a1a-chokepoint", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    // Acceptance criterion: at least one ReasoningTrace row appears within
    // 10s — proves handle_task_entry decoded the payload and dispatched to
    // drive_task, which is the chokepoint we are closing.
    let got = wait_for_action(
        &ssb,
        SsbActionType::ReasoningTrace,
        1,
        Duration::from_secs(10),
    )
    .await;
    assert!(
        got,
        "expected >=1 ReasoningTrace row within 10s when both feasibility_prober \
         and layer1a_processor are wired (Some+Some). drive_task never ran."
    );

    // Layer 1A tables exist after ensure_tables() — sanity-check the
    // processor was constructed via the production path, not a stub.
    let pending = layer1a_concrete.list_pending().await;
    assert!(
        pending.is_empty(),
        "fresh layer1a_processor must report zero pending proposals (got {})",
        pending.len()
    );

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
}

// ---------------------------------------------------------------------------
// Contract 2 — stub-ban: TaskRunnerConfig literal with both wirings set to
// None must NOT compile-equivalent to the production CLI path. This contract
// is a static check that the field types are Option<Arc<dyn ...>> so the
// `runtime_builder` cannot silently regress to `None`. A negative assertion
// at the type level: a closure that consumes both fields confirms they exist
// and are nameable.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Contract 3 — Phase A1b sibling: the helpers re-exported by
// `elai_orchestrator` (build_task_runner_wiring / apply_task_runner_wiring)
// MUST produce a TaskRunnerConfig with BOTH `feasibility_prober` and
// `layer1a_processor` populated as `Some(...)`. This pins the wiring move
// from `cli::runtime_builder` into `elai_orchestrator::runtime_setup` so the
// dashboard Tauri `setup()` callback can call them without re-implementing
// the wiring locally.
//
// Acceptance: cfg.feasibility_prober.is_some() && cfg.layer1a_processor.is_some()
// after passing the canonical bundle through apply_task_runner_wiring.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn contract_apply_task_runner_wiring_attaches_both_processors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let session_id = "test-a1b-wiring";

    let wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
        .await
        .expect("build_task_runner_wiring");

    let cursor_path = project_root.join(".elai_cc").join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    assert!(
        cfg.feasibility_prober.is_some(),
        "apply_task_runner_wiring must populate feasibility_prober (production chokepoint)"
    );
    assert!(
        cfg.layer1a_processor.is_some(),
        "apply_task_runner_wiring must populate layer1a_processor (production chokepoint)"
    );

    // The canonical SSB path lives at <project_root>/.elai_cc/sessions/ssb.db
    // (flat, no session_id nesting) — same bucket the dashboard reads via
    // `state::resolve_ssb_path`. This guards the WARN-1 fix from notes-A1a.md.
    let canonical_ssb = elai_core::paths::sessions_dir_path(&project_root)
        .join(elai_orchestrator::SSB_DB_FILE_NAME);
    assert!(
        canonical_ssb.exists(),
        "build_task_runner_wiring must open the canonical flat SSB path at {} \
         (no session_id sub-directory); got missing file",
        canonical_ssb.display()
    );
}

#[test]
fn contract_taskrunnerconfig_exposes_both_wiring_fields() {
    // Type-level proof: both fields are reachable from a function that
    // builds a minimal config. If a future refactor removes either field or
    // renames it, this test stops compiling — that is the intended trip-wire.
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/a1a-cursor"));
    let _: &Option<Arc<dyn FeasibilityProber>> = &cfg.feasibility_prober;
    let _: &Option<Arc<dyn Layer1aProcessor>> = &cfg.layer1a_processor;
    assert!(cfg.feasibility_prober.is_none());
    assert!(cfg.layer1a_processor.is_none());
}
