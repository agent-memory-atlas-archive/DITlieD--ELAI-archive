//! H-6 — observation masking + memory read-back contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-6-decisions.md): ACTIVATION. Un-gate the E-25
//! compaction ladder (default-ON kill-switch) and replace the hardcoded "[stubbed]"
//! placeholder with a real recover_stubs call (blake3-verified SSB recovery). No schema
//! bump (ContextReduction reuse). Deferred: the adaptive_reduce-direct ContextReduction
//! caller (overlaps the ladder), get_relevant_facts (FactStore-handle gap), and the
//! live-trim :1276 placeholder. Tier-2 (test exec + the ladder no-regression + the live
//! long-session masking/recovery + the GD-4 over-reduction bound) is DEFERRED.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// D1: the E-25 compaction ladder is un-gated — default-ON with the env as a
/// kill-switch (W-2: default-ON, no default-OFF graveyard, no allowlist row needed).
#[test]
fn compaction_ladder_default_on_killswitch() {
    let src = read_src("orchestrator/src/compaction_ladder.rs");
    // The gate fn must default to true (env unset => ON) — the marker is unwrap_or(true).
    assert!(
        src.contains("unwrap_or(true)"),
        "compaction_ladder_enabled() must default-ON (unwrap_or(true)); env is the kill-switch"
    );
}

/// D2: the live rehydration path recovers REAL archived content via recover_stubs,
/// no longer substituting the hardcoded placeholder on the snapshot branch.
#[test]
fn live_cli_recovers_stubs_not_placeholder() {
    let src = read_src("cli/src/live_cli.rs");
    assert!(
        src.contains("recover_stubs"),
        "the rehydration path must call recover_stubs (real content), not only the placeholder"
    );
}

/// GD-1: recover_stubs is REUSED (single definition in session_dag.rs), not re-built.
#[test]
fn recover_stubs_single_definition() {
    let src = read_src("orchestrator/src/session_dag.rs");
    let defs = src.matches("fn recover_stubs").count();
    assert_eq!(defs, 1, "exactly one recover_stubs definition (GD-1 reuse), found {defs}");
}
