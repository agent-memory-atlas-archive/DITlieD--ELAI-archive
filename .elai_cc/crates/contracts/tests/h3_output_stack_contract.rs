//! H-3 — output-token stack contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-3-decisions.md): activate the cleanly-tractable
//! output-token levers. STREAMING early-abort (dark on ALL paths: streaming_validator
//! was None, RuntimeClient had no Ssb handle) is activated (gated on
//! ELAI_FOCUS_TUNNEL_EXECUTE, GD-4) + its existing StreamingEarlyTerminated SSB row is
//! emitted. DIFF SELECTION (BLOCKER-1) is lifted by relaxing the cloud Rule-1 override
//! so cloud EditExisting can select diff. DIFF REPLAY (BLOCKER-2: needs a net-new prod
//! FileReadWrite + buffer_editor threading) is a documented Tier-2 follow-on. try_edit_tiers
//! (B-20) is REUSED, never rebuilt. Tier-2 (test exec + live early-abort/diff run +
//! the GD-4 flag flip) is DEFERRED.

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

/// D1: the streaming early-abort is activated — RuntimeClient now carries an Ssb
/// handle and the abort site emits the StreamingEarlyTerminated off-agent row.
#[test]
fn streaming_early_abort_emits_off_agent_row() {
    let rc = read_src("cli/src/runtime_client.rs");
    assert!(
        rc.contains("streaming_ssb"),
        "RuntimeClient must carry an Ssb handle (streaming_ssb) for the early-abort SSB emit"
    );
    assert!(
        rc.contains("StreamingEarlyTerminated"),
        "the early-abort site must emit a StreamingEarlyTerminated SSB row (the off-agent artifact)"
    );
}

/// D1: the validator is constructed and gated on the flag (default-OFF until the
/// over-abort rate is bounded, GD-4 — never an unconditional default-ON).
#[test]
fn streaming_validator_constructed_and_gated() {
    let rb = read_src("cli/src/runtime_builder.rs");
    assert!(
        rb.contains("StreamingValidator::new"),
        "runtime_builder must construct a real StreamingValidator (no longer hardcoded None)"
    );
    assert!(
        rb.contains("ELAI_FOCUS_TUNNEL_EXECUTE"),
        "the validator must be GATED on ELAI_FOCUS_TUNNEL_EXECUTE (GD-4 default-OFF until measured)"
    );
}

/// D2: BLOCKER-1 lifted — cloud EditExisting can fall through to diff selection.
#[test]
fn cloud_rule1_relaxed_for_edit_existing() {
    let m = read_src("orchestrator/src/focus_tunnel/mode.rs");
    assert!(
        m.contains("rule_1_cloud_execution") && m.contains("EditExisting"),
        "Rule 1 (cloud override) must be narrowed so cloud EditExisting falls through to diff selection"
    );
}

/// GD-1: the B-20 3-tier matcher is REUSED, never re-implemented (exactly one def).
#[test]
fn try_edit_tiers_single_definition() {
    let d = read_src("tools/src/dispatcher.rs");
    let defs = d.matches("fn try_edit_tiers").count();
    assert_eq!(defs, 1, "exactly one try_edit_tiers definition (GD-1 reuse), found {defs}");
}

/// R9: ELAI_FOCUS_TUNNEL_EXECUTE (the gate H-3 relies on) is registered in the
/// wire-dark allowlist with a promotion path (no default-off graveyard).
#[test]
fn focus_tunnel_flag_registered_in_allowlist() {
    // crates_dir() is .../.elai_cc/crates; the allowlist is one level up at .elai_cc/.
    let elai_cc = crates_dir()
        .parent()
        .expect(".elai_cc is the parent of crates/")
        .to_path_buf();
    let path = elai_cc.join("wire-dark-allowlist.toml");
    let a = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        a.contains("ELAI_FOCUS_TUNNEL_EXECUTE"),
        "ELAI_FOCUS_TUNNEL_EXECUTE must be registered in wire-dark-allowlist.toml (R9)"
    );
}
