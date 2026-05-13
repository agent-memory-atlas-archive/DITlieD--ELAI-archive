// Phase A1c — end-to-end chokepoint integration test.
//
// Asserts the full `TaskSubmitted -> drive_task -> ReasoningTrace |
// ConceptDecompositionNode` path with a production `RuntimeWiring` bundle
// built via `elai_orchestrator::build_task_runner_wiring`. The test sits in
// the orchestrator crate's `tests/` directory (not in `crates/contracts/`)
// because it exercises the in-crate wiring helpers against a fresh tempdir
// and is the canonical regression guard for the dashboard-CLI shared-bucket
// contract pinned by phase A1b.
//
// Tests/handle:
//   - Builds a `RuntimeWiring` rooted in a `tempfile::TempDir` so the
//     canonical SSB path is `<tmp>/.elai_cc/sessions/ssb.db`.
//   - Applies the wiring to a `TaskRunnerConfig::new_minimal(...)` via
//     `apply_task_runner_wiring` (NOT a hand-rolled `TaskRunnerConfig { ... }`
//     literal — that would bypass the chokepoint we are guarding).
//   - Spawns `start_task_runner(writer, cfg)` and writes a synthetic
//     `SsbPayload::TaskSubmitted` row via the same `Arc<Ssb>` handle the
//     runner subscribed to (broadcast attachment guarantee, mirrors
//     contracts/tests/task_runner_chokepoint_contract.rs:101-148).
//   - Waits up to 10s for at least one `SsbActionType::ReasoningTrace` OR
//     `SsbActionType::ConceptDecompositionNode` row. The "Decomposition-family"
//     wording in workflow/handoff/phase-A1c.md acceptance maps to these
//     variants (the SSB schema has no `Decomposition` enum literal — see
//     ssb.rs:184/406 and task_runner_chokepoint_contract.rs:11-15).
//   - Cancels and awaits `JoinHandle` with a 5s graceful-shutdown timeout.
//
// Stub-ban:
//   - `.expect(...)` is acceptable here per CLAUDE.md ("acceptable ONLY in
//     test code (#[cfg(test)]/tests/) ..."). No `.unwrap()` is used outside
//     the `tempfile::tempdir().expect("tempdir")` literal — every fallible
//     production-shaped call propagates via `.expect("...")` with a context
//     message describing what failed.
//   - No `#[ignore]`. No raw `tokio::time::sleep` masquerading as a wait
//     loop — the real wait is gated by `tokio::time::timeout` plus a
//     polling helper that breaks the moment a matching row appears.
//
// Acceptance: cargo test -p elai-orchestrator --test e2e_chokepoint passes
// with at least one matching SSB row observed before the 10s timeout.

use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload, SsbReader, TaskSubmittedPayload};
use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, start_task_runner, RuntimeWiring,
};

/// Poll the SSB until at least one row matching ANY of `targets` is present,
/// or `timeout` elapses. Returns the first variant that matched (so the
/// failure assertion can report which family fired) or `None` on timeout.
async fn wait_for_any_action(
    writer: &Arc<elai_orchestrator::Ssb>,
    targets: &[SsbActionType],
    timeout: Duration,
) -> Option<SsbActionType> {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        for action in targets {
            let entries = writer.query_by_type(*action).unwrap_or_default();
            if !entries.is_empty() {
                return Some(*action);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
async fn e2e_chokepoint_taskpayload_drives_reasoning_or_decomposition() {
    // 1. Fresh project_root inside a TempDir. The TempDir guard is held in
    //    scope until the end of the test so cleanup happens after the runner
    //    has been cancelled and joined.
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let session_id = "test-a1c-e2e";

    // 2. Build the production wiring bundle. This is the same call the
    //    dashboard Tauri `setup()` and `elai run` use in production. Errors
    //    here would indicate a regression in the chokepoint helpers (A1a/A1b
    //    contracts) — surface them with a precise message rather than a bare
    //    unwrap so a CI failure points at the right phase.
    let wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
        .await
        .expect("build_task_runner_wiring(project_root) succeeds");

    // The canonical project_id derived from the same path the wiring used.
    // After the A1c swap, runtime_setup.rs computes this exact value via
    // elai_core::project_id_from_path(project_root). Mirroring it here means
    // the synthetic TaskSubmitted row lands in the bucket the runner reads.
    let project_id = elai_core::project_id_from_path(&project_root);

    // 3. Apply the wiring to a fresh TaskRunnerConfig via the production
    //    chokepoint helper. cursor_path lives inside the project's
    //    `.elai_cc/` directory (build_task_runner_wiring already created it).
    let cursor_path = project_root.join(".elai_cc").join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);
    assert!(
        cfg.feasibility_prober.is_some(),
        "apply_task_runner_wiring must populate feasibility_prober"
    );
    assert!(
        cfg.layer1a_processor.is_some(),
        "apply_task_runner_wiring must populate layer1a_processor"
    );

    // 4. Spawn the runner against the same `Arc<Ssb>` we just wrote.
    let writer = wiring.ssb.clone();
    let (handle, cancel) = start_task_runner(writer.clone(), cfg);

    // 5. Give the runner a moment to subscribe to the broadcast before the
    //    write fires. The polling loop below would also catch this row even
    //    if subscribed late (`replay_backlog` re-reads the table), but the
    //    short delay shaves variance off the assertion latency.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 6. Synthetic TaskSubmitted row — plain non-research, non-scaffold task
    //    so `drive_task` takes the default branch and emits the unconditional
    //    `observe` ReasoningTrace at task_runner.rs:3263.
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "Phase A1c e2e: build a hello-world CLI binary".to_string(),
        project_id: project_id.clone(),
        submitted_at_ms: 0,
    });
    writer
        .write_payload(&project_id, 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted to SSB");

    // 7. Wait up to 10s for ReasoningTrace OR ConceptDecompositionNode.
    //    `wait_for_any_action` polls every 50ms so a fast machine sees the
    //    row in well under the budget; the 10s upper bound matches the
    //    acceptance criterion in workflow/handoff/phase-A1c.md.
    let targets = [
        SsbActionType::ReasoningTrace,
        SsbActionType::ConceptDecompositionNode,
    ];
    let matched = tokio::time::timeout(
        Duration::from_secs(10),
        wait_for_any_action(&writer, &targets, Duration::from_secs(10)),
    )
    .await
    .expect("wait_for_any_action future must complete within outer 10s timeout");
    assert!(
        matched.is_some(),
        "expected at least one ReasoningTrace OR ConceptDecompositionNode row \
         within 10s after writing TaskSubmitted; drive_task never ran (chokepoint regression). \
         project_root={} project_id={}",
        project_root.display(),
        project_id
    );

    // 8. Graceful shutdown. cancel() flips the CancelHandle; the runner's
    //    `tokio::select!` wakes on the notify and returns. The 5s timeout
    //    is a safety net — a properly wired runner shuts down in <50ms.
    cancel.cancel();
    let join_result = tokio::time::timeout(Duration::from_secs(5), handle)
        .await
        .expect("task runner JoinHandle did not complete within 5s of cancel()");
    let runner_result = join_result.expect("task runner panicked");
    runner_result.expect("task runner returned an error after cancel()");
}
