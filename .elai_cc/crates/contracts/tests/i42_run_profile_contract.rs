//! I-42 critique-only operating-profile enforcement-plane contract.
//!
//! `RunProfile { Orchestrate (default), CritiqueOnly }` is threaded through the
//! `elai run` surface and consulted at the single executor-install chokepoint
//! `runtime_setup::apply_task_runner_wiring`. In `CritiqueOnly` the resolved
//! production `baby_step_executor` is forced to `None` BEFORE install, so no
//! disk-mutating `AgentPoolBabyStepExecutor` is ever installed — structurally
//! refusing the executor/fixer agent spawn (one gate covers both, since the
//! fixer loop derives from the same field). FORK-1 = A (controller): the
//! observable is the SPAWN GATE FIRING (config state), not an absent FileWrite
//! SSB row (no live producer). FORK-2 = a: gate at INSTALL TIME in
//! apply_task_runner_wiring.
//!
//! Arms (all fail-closed; missing evidence = FAIL):
//!   (a) GATE-FIRED (R4, real production seam, NOT a direct gate-fn call): a
//!       wiring carrying a REAL executor + `RunProfile::CritiqueOnly`, passed
//!       through `apply_task_runner_wiring`, yields a config whose
//!       `baby_step_executor` is `None` — the executor AND fixer spawn is
//!       refused at the install chokepoint.
//!   (b) DIFFERENTIATOR (R2, proves the gate is not a no-op): the SAME wiring
//!       under `RunProfile::Orchestrate` (the default) installs the real
//!       executor (`baby_step_executor.is_some()`). The profile is the
//!       differentiator.
//!   (c) FROM-CLI-ARGS: `RunProfile::from_cli_args` parses `"orchestrate"` /
//!       `"critique-only"` and defaults to `Orchestrate` on `None`.
//!
//! R7 note: the wiring fn under assertion (`apply_task_runner_wiring`) is the
//! PRODUCTION chokepoint reached by `cli::run_subcommand::run_async`; this
//! contract drives it directly (not the new gate fn in isolation), so it is a
//! structural assertion on the REAL production config state, not a DTO
//! serialize. The production caller of `RunProfile::from_cli_args` is
//! `crates/cli/src/run_subcommand.rs` (`wiring.run_profile = Some(...)`).
//!
//! Spec references: docs/thefinalglance/I-plan/I-42_critique_only_profile.md
//! Reads (mandatory): wiring-enforcement.md R1/R2/R4/R7; .claude/rules/production-caller-gate.md

use std::sync::Arc;

use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, BabyStepExecutor, RunProfile, RuntimeWiring,
    SyntheticBabyStepExecutor,
};

/// Build the canonical production wiring bundle, then install a REAL
/// baby-step executor on it so the spawn-gate has something concrete to
/// refuse. `SyntheticBabyStepExecutor` stands in for the production
/// `AgentPoolBabyStepExecutor` here: the gate keys off `Some(_)` vs `None`,
/// not the concrete type, so a synthetic executor is a faithful proxy for
/// "an executor is installed". The cloud_config stays at its
/// build-default (`None`), so `apply_task_runner_wiring` plumbs the executor
/// through verbatim except for the I-42 gate.
async fn wiring_with_executor(root: &std::path::Path) -> RuntimeWiring {
    let mut wiring: RuntimeWiring = build_task_runner_wiring(root, "test-i42-run-profile")
        .await
        .expect("build_task_runner_wiring");
    let exec: Arc<dyn BabyStepExecutor> = Arc::new(SyntheticBabyStepExecutor);
    wiring.baby_step_executor = Some(exec);
    wiring
}

// ---------------------------------------------------------------------------
// (a) GATE-FIRED: CritiqueOnly forces baby_step_executor to None at install.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn i42_a_critique_only_refuses_executor_install() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let cursor_path = root.join("cursor.json");

    let mut wiring = wiring_with_executor(&root).await;
    // Sanity: the executor IS present on the wiring before apply.
    assert!(
        wiring.baby_step_executor.is_some(),
        "fixture must install a real executor on the wiring before apply"
    );

    wiring.run_profile = Some(RunProfile::CritiqueOnly);
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    // R1/R4: the structural spawn gate fired at the REAL production
    // chokepoint — no executor (and therefore no fixer, which derives from
    // the same field) is installed. Missing this = FAIL.
    assert!(
        cfg.baby_step_executor.is_none(),
        "RunProfile::CritiqueOnly must force baby_step_executor to None at the \
         install chokepoint (executor AND fixer spawn refused); got Some"
    );
}

// ---------------------------------------------------------------------------
// (b) DIFFERENTIATOR: Orchestrate (default) installs the real executor.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn i42_b_orchestrate_installs_executor() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let cursor_path = root.join("cursor.json");

    let mut wiring = wiring_with_executor(&root).await;
    wiring.run_profile = Some(RunProfile::Orchestrate);
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    // R2: proves the gate is the differentiator, not a no-op — the SAME
    // fixture under Orchestrate keeps the real executor installed.
    assert!(
        cfg.baby_step_executor.is_some(),
        "RunProfile::Orchestrate must keep the real executor installed \
         (the gate is the differentiator, not a no-op); got None"
    );
}

// ---------------------------------------------------------------------------
// (b') None profile preserves the legacy posture (executor installed).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn i42_b2_none_profile_preserves_legacy_install() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let cursor_path = root.join("cursor.json");

    let mut wiring = wiring_with_executor(&root).await;
    wiring.run_profile = None;
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    assert!(
        cfg.baby_step_executor.is_some(),
        "run_profile = None must preserve the legacy executor install"
    );
}

// ---------------------------------------------------------------------------
// (c) FROM-CLI-ARGS: the selector parses both labels and defaults correctly.
// ---------------------------------------------------------------------------

#[test]
fn i42_c_from_cli_args_parses_and_defaults() {
    assert_eq!(RunProfile::from_cli_args(None), RunProfile::Orchestrate);
    assert_eq!(
        RunProfile::from_cli_args(Some("orchestrate")),
        RunProfile::Orchestrate
    );
    assert_eq!(
        RunProfile::from_cli_args(Some("critique-only")),
        RunProfile::CritiqueOnly
    );
    // Unrecognised defaults to Orchestrate (parse-time validation rejects
    // bad values earlier at the CLI surface).
    assert_eq!(
        RunProfile::from_cli_args(Some("bogus")),
        RunProfile::Orchestrate
    );
    // The structural predicate the chokepoint consults.
    assert!(RunProfile::CritiqueOnly.refuses_executor_spawn());
    assert!(!RunProfile::Orchestrate.refuses_executor_spawn());
}
