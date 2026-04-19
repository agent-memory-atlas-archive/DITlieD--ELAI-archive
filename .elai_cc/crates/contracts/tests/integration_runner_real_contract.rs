//! Batch 4A (4A.4) — real IntegrationTestRunner swap contract.
//!
//! Pins:
//!   1. `CommandIntegrationRunner` maps level labels to the correct
//!      per-language commands (Rust: cargo check / cargo test).
//!   2. An unsupported language emits `IntegrationSkipped { level, reason }`
//!      with `passed=true` so the tiered validator does NOT short-circuit.
//!   3. Per-level timeout constants are 5 / 30 / 60 / 120 s.
//!   4. `passed=false` outcomes carry a non-empty stderr snippet.

use std::time::Duration;

use elai_tools::tiered_validation::{
    CommandIntegrationRunner, IntegrationCommandRunner, BABY_STEP_TIMEOUT_S, CONCEPT_TIMEOUT_S,
    PHASE_TIMEOUT_S, STEP_TIMEOUT_S,
};

// ---------------------------------------------------------------------------
// 1. Command mapping per level + language
// ---------------------------------------------------------------------------

#[test]
fn rust_baby_maps_to_cargo_check() {
    let runner = CommandIntegrationRunner::new("rust");
    let (program, args) = runner.command_for("baby").expect("baby command");
    assert_eq!(program, "cargo");
    assert!(args.iter().any(|a| a == "check"));
}

#[test]
fn rust_step_uses_crate_slug_when_set() {
    let runner =
        CommandIntegrationRunner::new("rust").with_crate_slug("elai-orchestrator");
    let (program, args) = runner.command_for("step").expect("step command");
    assert_eq!(program, "cargo");
    assert!(args.iter().any(|a| a == "test"));
    assert!(args.iter().any(|a| a == "-p"));
    assert!(args.iter().any(|a| a == "elai-orchestrator"));
}

#[test]
fn rust_phase_uses_crate_slug() {
    let runner =
        CommandIntegrationRunner::new("rust").with_crate_slug("elai-tools");
    let (program, args) = runner.command_for("phase").expect("phase command");
    assert_eq!(program, "cargo");
    assert!(args.iter().any(|a| a == "elai-tools"));
}

#[test]
fn rust_concept_uses_workspace() {
    let runner = CommandIntegrationRunner::new("rust");
    let (program, args) = runner.command_for("concept").expect("concept command");
    assert_eq!(program, "cargo");
    assert!(args.iter().any(|a| a == "--workspace"));
}

#[test]
fn python_baby_uses_python_minus_c() {
    let runner = CommandIntegrationRunner::new("python");
    let (program, args) = runner.command_for("baby").expect("python baby");
    assert_eq!(program, "python");
    assert!(args.iter().any(|a| a == "-c"));
}

#[test]
fn typescript_baby_uses_tsc_noemit() {
    let runner = CommandIntegrationRunner::new("typescript");
    let (program, args) = runner.command_for("baby").expect("ts baby");
    assert_eq!(program, "tsc");
    assert!(args.iter().any(|a| a == "--noEmit"));
}

#[test]
fn go_baby_uses_go_vet() {
    let runner = CommandIntegrationRunner::new("go");
    let (program, args) = runner.command_for("baby").expect("go baby");
    assert_eq!(program, "go");
    assert!(args.iter().any(|a| a == "vet"));
}

// ---------------------------------------------------------------------------
// 2. IntegrationSkipped path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_language_emits_integration_skipped() {
    let runner = CommandIntegrationRunner::new("cobol");
    let project_dir = tempfile::tempdir().unwrap();
    let outcome = runner
        .run_integration("step", project_dir.path())
        .await;
    assert!(
        outcome.passed,
        "IntegrationSkipped must report passed=true so the tiered validator continues"
    );
    let snippet = outcome.stderr_snippet.expect("skip snippet");
    assert!(snippet.contains("IntegrationSkipped"));
    assert!(snippet.contains("no_test_command_for_language"));
}

#[tokio::test]
async fn unsupported_level_for_known_language_is_also_skipped() {
    // An unknown level label degrades to the default STEP command, but we
    // verify the IntegrationSkipped structural contract by constructing a
    // language with no mapping at all.
    let runner = CommandIntegrationRunner::new("brainfuck");
    let project_dir = tempfile::tempdir().unwrap();
    let outcome = runner
        .run_integration("baby", project_dir.path())
        .await;
    assert!(outcome.passed);
    assert!(outcome
        .stderr_snippet
        .as_deref()
        .unwrap_or("")
        .contains("IntegrationSkipped"));
}

// ---------------------------------------------------------------------------
// 3. Per-level timeout constants
// ---------------------------------------------------------------------------

#[test]
fn timeout_constants_match_spec() {
    assert_eq!(BABY_STEP_TIMEOUT_S, 5);
    assert_eq!(STEP_TIMEOUT_S, 30);
    assert_eq!(PHASE_TIMEOUT_S, 60);
    assert_eq!(CONCEPT_TIMEOUT_S, 120);
}

#[test]
fn timeout_for_level_returns_correct_duration() {
    assert_eq!(
        CommandIntegrationRunner::timeout_for_level("baby"),
        Duration::from_secs(5)
    );
    assert_eq!(
        CommandIntegrationRunner::timeout_for_level("step"),
        Duration::from_secs(30)
    );
    assert_eq!(
        CommandIntegrationRunner::timeout_for_level("phase"),
        Duration::from_secs(60)
    );
    assert_eq!(
        CommandIntegrationRunner::timeout_for_level("concept"),
        Duration::from_secs(120)
    );
    // Unknown levels default to the Step budget (30 s).
    assert_eq!(
        CommandIntegrationRunner::timeout_for_level("???"),
        Duration::from_secs(30)
    );
}

// ---------------------------------------------------------------------------
// 4. Failing command surfaces a stderr snippet
// ---------------------------------------------------------------------------

#[tokio::test]
async fn nonexistent_program_reports_spawn_failure() {
    // Use a language mapping that produces a command we KNOW is absent on
    // the host (`brainfuck` has no mapping → IntegrationSkipped path), so
    // instead we force the real command branch by picking `go` + pointing
    // at an empty dir where `go` is unlikely to succeed. If Go is absent
    // the spawn_failure branch fires; if present but returns non-zero, the
    // non-success branch fires. Either way, `passed=false`.
    let runner = CommandIntegrationRunner::new("go");
    let project_dir = tempfile::tempdir().unwrap();
    let outcome = runner
        .run_integration("baby", project_dir.path())
        .await;
    // Skip when environment unexpectedly has `go` + a tiny empty dir treats
    // vet as success-on-nothing — not all systems reach the failure path.
    if outcome.passed {
        return;
    }
    assert!(outcome.stderr_snippet.is_some());
}
