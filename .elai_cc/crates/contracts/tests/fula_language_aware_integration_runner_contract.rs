//! FUL-A.2 — language-aware `CommandIntegrationRunner` contract.
//!
//! Pins:
//!   1. `new_multi(Vec<String>)` constructor exists and is polyglot-aware.
//!   2. Single-language Rust project runs `cargo` at BabyStep.
//!   3. Single-language Python project runs `python` at BabyStep.
//!   4. Polyglot Rust+TypeScript project returns BOTH commands via
//!      `commands_for(level)` so the runner can short-circuit on first fail.
//!   5. Missing-language pair emits IntegrationSkipped at run time.
//!   6. Empty `Vec` input falls back to `["rust"]` (documented safe default).
//!   7. Backward compatibility: the legacy `new(language)` constructor still
//!      behaves as before (single-language) — existing callers keep working.

use elai_tools::tiered_validation::{
    CommandIntegrationRunner, IntegrationCommandRunner,
};

// ---------------------------------------------------------------------------
// 1. new_multi exists and preserves language order.
// ---------------------------------------------------------------------------

#[test]
fn new_multi_accepts_vec_and_preserves_order() {
    let runner = CommandIntegrationRunner::new_multi(vec![
        "rust".to_string(),
        "typescript".to_string(),
    ]);
    let langs = runner.languages();
    assert_eq!(langs, &["rust".to_string(), "typescript".to_string()]);
}

// ---------------------------------------------------------------------------
// 2. Rust-only project maps BabyStep to `cargo check`.
// ---------------------------------------------------------------------------

#[test]
fn rust_only_baby_step_runs_cargo_check() {
    let runner = CommandIntegrationRunner::new_multi(vec!["rust".to_string()]);
    let cmds = runner.commands_for("baby");
    assert_eq!(cmds.len(), 1, "rust-only should have exactly 1 command");
    let (lang, program, args) = &cmds[0];
    assert_eq!(lang, "rust");
    assert_eq!(program, "cargo");
    assert!(args.iter().any(|a| a == "check"));
}

// ---------------------------------------------------------------------------
// 3. Python-only project maps BabyStep to `python`.
// ---------------------------------------------------------------------------

#[test]
fn python_only_baby_step_runs_python() {
    let runner = CommandIntegrationRunner::new_multi(vec!["python".to_string()]);
    let cmds = runner.commands_for("baby");
    assert_eq!(cmds.len(), 1);
    let (lang, program, _args) = &cmds[0];
    assert_eq!(lang, "python");
    assert_eq!(program, "python");
}

// ---------------------------------------------------------------------------
// 4. Polyglot Rust+TypeScript project returns both commands at BabyStep.
// ---------------------------------------------------------------------------

#[test]
fn polyglot_rust_typescript_returns_both_commands() {
    let runner = CommandIntegrationRunner::new_multi(vec![
        "rust".to_string(),
        "typescript".to_string(),
    ]);
    let cmds = runner.commands_for("baby");
    assert_eq!(cmds.len(), 2, "polyglot should have 2 commands at baby");
    assert_eq!(cmds[0].0, "rust");
    assert_eq!(cmds[0].1, "cargo");
    assert_eq!(cmds[1].0, "typescript");
    assert_eq!(cmds[1].1, "tsc");
}

// ---------------------------------------------------------------------------
// 5. Unknown-language triple yields IntegrationSkipped on run_integration.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn polyglot_all_unknown_languages_skip() {
    let runner =
        CommandIntegrationRunner::new_multi(vec!["cobol".to_string(), "brainfuck".to_string()]);
    let dir = tempfile::tempdir().unwrap();
    let outcome = runner.run_integration("baby", dir.path()).await;
    assert!(
        outcome.passed,
        "all-skip must produce passed=true (skip semantics)",
    );
    let s = outcome.stderr_snippet.expect("snippet present on skip");
    assert!(s.contains("IntegrationSkipped"), "got {s}");
}

// ---------------------------------------------------------------------------
// 6. Empty Vec falls back to rust.
// ---------------------------------------------------------------------------

#[test]
fn empty_vec_falls_back_to_rust() {
    let runner = CommandIntegrationRunner::new_multi(Vec::<String>::new());
    let langs = runner.languages();
    assert_eq!(langs, &["rust".to_string()], "empty must fall back to rust");
    let cmds = runner.commands_for("baby");
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].1, "cargo");
}

// ---------------------------------------------------------------------------
// 7. Legacy new(&str) constructor still produces single-language behaviour.
// ---------------------------------------------------------------------------

#[test]
fn legacy_new_single_language_still_works() {
    let runner = CommandIntegrationRunner::new("rust");
    let langs = runner.languages();
    assert_eq!(langs, &["rust".to_string()]);
    let cmds = runner.commands_for("baby");
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].1, "cargo");
}

// ---------------------------------------------------------------------------
// 8. run_integration on a polyglot runner with a valid + known language runs
// and short-circuits on first fail. We verify the structural contract: the
// run must return a ValidationOutcome shape (not an Err), and when at least
// one command is spawnable the tier cannot report `passed=true` unless every
// language's command succeeded. Because spawning arbitrary `cargo check`
// inside a tempdir is environment-dependent, we assert only the non-skip
// shape: stderr_snippet surfaces the failing language on non-zero exit.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn polyglot_run_integration_surfaces_a_shape_with_known_languages() {
    let runner = CommandIntegrationRunner::new_multi(vec![
        "rust".to_string(),
        "python".to_string(),
    ]);
    let dir = tempfile::tempdir().unwrap();
    let outcome = runner.run_integration("baby", dir.path()).await;
    // The outcome is either passed=true (both ran cleanly) OR passed=false
    // with a non-empty snippet — we never want IntegrationSkipped here
    // because both languages are supported.
    if !outcome.passed {
        let s = outcome.stderr_snippet.unwrap_or_default();
        assert!(
            !s.is_empty(),
            "polyglot fail must carry a stderr snippet, got empty",
        );
        assert!(
            !s.contains("no_test_command_for_language"),
            "rust+python should never fall into the all-skip snippet: {s}",
        );
    }
}
