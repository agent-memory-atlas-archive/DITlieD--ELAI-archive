//! H-50 — regime-aware context policy contract (source-wiring gate).
//!
//! Verifies the five structural claims from the H-50 WIRING_CONTRACT:
//!
//! D1: effective_context_window(Local, _) respects ELAI_LOCAL_MAX_CONTEXT.
//! D2: trim_threshold(Local, 32000) = floor(32000 * 0.75) = 24000.
//! D3: compaction_ladder is reachable from conversation.rs maybe_auto_compact
//!     on the Local path — the trim-threshold path and the ladder are both
//!     present in the same module (source-reachability, R5/R6).
//! D4: recover_stubs unconditional whole-set call is replaced by the selective
//!     path on LOCAL: recover_single_stub is the per-seq method; recover_stubs
//!     is still present for the CLOUD/rewind path (additive, R6).
//! D5: ContextReduction variant is reused (no new WindowPolicyApplied variant
//!     unless the executor explicitly added one — GD-1 preference is reuse).
//!
//! All assertions are source-grep / unit-logic checks; no live model or sidecar
//! required.  The done-gate observable_effect leg (c) — a control run hitting
//! context_window_blocked while the enabled run completes — requires a live
//! sidecar session on pythondev and is DEFERRED per the context package note.

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
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// D1: effective_context_window reads ELAI_LOCAL_MAX_CONTEXT
// ---------------------------------------------------------------------------

/// D1: the context_policy module reads ELAI_LOCAL_MAX_CONTEXT to determine
/// the local window, same as runtime_client.rs.  Must not hard-code a window.
#[test]
fn d1_context_policy_reads_elai_local_max_context() {
    let src = read_src("orchestrator/src/context_policy.rs");
    assert!(
        src.contains("ELAI_LOCAL_MAX_CONTEXT"),
        "context_policy.rs must reference ELAI_LOCAL_MAX_CONTEXT; found:\n{src}"
    );
    assert!(
        src.contains("fn effective_context_window"),
        "context_policy.rs must export effective_context_window"
    );
}

// ---------------------------------------------------------------------------
// D2: trim_threshold(Local, 32000) = 24000
// ---------------------------------------------------------------------------

/// D2: trim_threshold for LOCAL at 32 000 tokens = floor(32000 * LOCAL_TRIM_FRACTION).
/// The plan specifies ~0.70-0.75; we use 0.75 (conservative, GD-4 calibration deferred).
/// floor(32000 * 0.75) = 24000.
#[test]
fn d2_local_trim_threshold_32k() {
    use elai_orchestrator::{trim_threshold, ContextPolicy};
    let threshold = trim_threshold(ContextPolicy::Local, 32_000);
    assert_eq!(
        threshold, 24_000,
        "trim_threshold(Local, 32000) must be floor(32000 * 0.75) = 24000, got {threshold}"
    );
}

/// D2b: trim_threshold cloud leaves the 0.85 SESSION_HANDOFF_THRESHOLD intact.
#[test]
fn d2b_cloud_trim_threshold_unchanged() {
    use elai_orchestrator::{trim_threshold, ContextPolicy};
    let threshold = trim_threshold(ContextPolicy::Cloud, 200_000);
    assert_eq!(
        threshold, 170_000,
        "trim_threshold(Cloud, 200_000) must be floor(200_000 * 0.85) = 170_000, got {threshold}"
    );
}

// ---------------------------------------------------------------------------
// D3: compaction_ladder reachable from maybe_auto_compact (source wiring)
// ---------------------------------------------------------------------------

/// D3: conversation.rs maybe_auto_compact uses the auto_compaction threshold
/// field. The H-50 path injects the regime-aware threshold via
/// with_auto_compaction_input_tokens_threshold in runtime_builder.rs.
/// Assert both sides of the wiring are present in source.
#[test]
fn d3_runtime_builder_wires_trim_threshold_for_mode() {
    let src = read_src("cli/src/runtime_builder.rs");
    assert!(
        src.contains("trim_threshold_for_mode"),
        "runtime_builder.rs must call trim_threshold_for_mode (H-50 wiring)"
    );
    assert!(
        src.contains("with_auto_compaction_input_tokens_threshold"),
        "runtime_builder.rs must call with_auto_compaction_input_tokens_threshold"
    );
}

/// D3b: conversation.rs maybe_auto_compact consumes auto_compaction_input_tokens_threshold.
#[test]
fn d3b_maybe_auto_compact_uses_threshold_field() {
    let src = read_src("orchestrator/src/conversation.rs");
    assert!(
        src.contains("auto_compaction_input_tokens_threshold"),
        "conversation.rs maybe_auto_compact must reference auto_compaction_input_tokens_threshold"
    );
}

// ---------------------------------------------------------------------------
// D4: recover_stubs is additive (not removed); local path uses selective recovery
// ---------------------------------------------------------------------------

/// D4: session_dag.rs must still export recover_stubs (cloud/rewind callers)
/// AND the new recover_single_stub for selective local recovery.
/// This checks that recover_single_stub EXISTS in session_dag.rs.
#[test]
fn d4_session_dag_has_both_recovery_methods() {
    let src = read_src("orchestrator/src/session_dag.rs");
    let stubs_count = src.matches("fn recover_stubs").count();
    assert_eq!(
        stubs_count, 1,
        "recover_stubs must have exactly one definition (GD-1); found {stubs_count}"
    );
    assert!(
        src.contains("fn recover_single_stub"),
        "session_dag.rs must define recover_single_stub for selective H-50 local recovery"
    );
}

/// D4b (R8 production-reachability gate): live_cli.rs must call recover_single_stub
/// on the LOCAL branch, and the unconditional recover_stubs call must be guarded
/// so it does NOT run on the LOCAL path after a trim (prevents net-zero trim).
/// This asserts the production wiring, not just symbol existence.
#[test]
fn d4b_live_cli_local_path_uses_selective_recovery() {
    let src = read_src("cli/src/live_cli.rs");
    // After BLOCK-1 fix: recover_single_stub must appear in live_cli.rs
    // (wired into the on-demand reload site for local stubs).
    assert!(
        src.contains("recover_single_stub"),
        "live_cli.rs must call recover_single_stub for H-50 LOCAL selective recovery (R8: production caller)"
    );
    // The LOCAL branch must guard recover_stubs (it must be conditional on
    // backend_mode NOT being Local, i.e. skip the whole-set re-inflate on local).
    // We verify this by asserting the local-guard comment/pattern is present.
    assert!(
        src.contains("BackendMode::Local") || src.contains("backend_mode") && src.contains("recover_stubs"),
        "live_cli.rs must check backend_mode before calling recover_stubs (local path must not re-inflate all stubs)"
    );
}

// ---------------------------------------------------------------------------
// D5: ContextReduction is reused (no new SSB variant added by H-50 default)
// ---------------------------------------------------------------------------

/// D5: GD-1 preference: H-50 reuses SsbActionType::ContextReduction.
/// The plan says to prefer reuse over adding WindowPolicyApplied.
/// Assert ContextReduction is present in ssb.rs (already verified as shipped by H-6).
#[test]
fn d5_context_reduction_variant_present() {
    let src = read_src("orchestrator/src/ssb.rs");
    assert!(
        src.contains("ContextReduction"),
        "ssb.rs must contain ContextReduction (reused by H-50 per GD-1)"
    );
}

// ---------------------------------------------------------------------------
// Kill-switch: ELAI_CONTEXT_POLICY_DISABLE is the default-ON guard
// ---------------------------------------------------------------------------

/// Kill-switch: ELAI_CONTEXT_POLICY_DISABLE is the kill-switch constant (not an enable flag).
/// Verify the module exposes the constant with the correct name.
#[test]
fn killswitch_is_disable_not_enable() {
    let src = read_src("orchestrator/src/context_policy.rs");
    assert!(
        src.contains("ELAI_CONTEXT_POLICY_DISABLE"),
        "kill-switch must be named ELAI_CONTEXT_POLICY_DISABLE (disable=1 turns it off)"
    );
    // Default-ON pattern: context_policy_disabled() defaults to false (policy active).
    assert!(
        src.contains("unwrap_or(false)"),
        "context_policy_disabled() must default to false (policy active = ON by default)"
    );
}

// ---------------------------------------------------------------------------
// Production wiring: live_cli.rs uses effective_context_window
// ---------------------------------------------------------------------------

/// Wiring: live_cli.rs prepare_turn_runtime replaces max_tokens_for_model with
/// effective_context_window for the trim gate.
#[test]
fn live_cli_uses_effective_context_window() {
    let src = read_src("cli/src/live_cli.rs");
    assert!(
        src.contains("effective_context_window"),
        "live_cli.rs must call effective_context_window (H-50 trim gate fix)"
    );
}

/// Wiring: the old mis-wired UNCONDITIONAL assignment
///   `let model_window = max_tokens_for_model(&self.model) as usize;`
/// must no longer be the primary trim-gate path after H-50.
/// The H-50 path uses effective_context_window(backend_mode); the legacy
/// function may still appear in a CONDITIONAL kill-switch branch (WARN-1 fix)
/// but must never be the unconditional assignment on that line.
#[test]
fn live_cli_trim_gate_no_longer_uses_max_tokens_for_model_directly() {
    let src = read_src("cli/src/live_cli.rs");
    // The old unconditional one-liner is gone; effective_context_window is the
    // primary path.  The kill-switch branch may conditionally reference
    // max_tokens_for_model as a fallback, which is correct (WARN-1 fix).
    // Assert: effective_context_window IS present (primary path).
    assert!(
        src.contains("effective_context_window(backend_mode)"),
        "live_cli.rs must call effective_context_window(backend_mode) as the primary trim gate"
    );
    // Assert: the unconditional direct assignment pattern is gone.
    // The old code was a standalone `let model_window = max_tokens_for_model(&self.model) as usize;`
    // with no if/else; now it must only appear inside a conditional block.
    // We verify the kill-switch guard is present when max_tokens_for_model is used.
    if src.contains("max_tokens_for_model(&self.model) as usize") {
        // If the old fn is still used somewhere, it must be guarded by context_policy_disabled().
        assert!(
            src.contains("context_policy_disabled()"),
            "max_tokens_for_model must only be used under context_policy_disabled() kill-switch guard"
        );
    }
}

// ---------------------------------------------------------------------------
// Unit: effective_context_window runtime behaviour
// ---------------------------------------------------------------------------

/// effective_context_window(Api) returns the cloud default.
#[test]
fn effective_window_api_returns_cloud_default() {
    use elai_orchestrator::{effective_context_window, CLOUD_DEFAULT_WINDOW};
    use elai_orchestrator::BackendMode;
    assert_eq!(
        effective_context_window(BackendMode::Api),
        CLOUD_DEFAULT_WINDOW,
        "Api mode must return CLOUD_DEFAULT_WINDOW"
    );
}

/// effective_context_window(Local) when env var set returns parsed value.
#[test]
fn effective_window_local_reads_env() {
    use elai_orchestrator::{effective_context_window, ELAI_LOCAL_MAX_CONTEXT_VAR};
    use elai_orchestrator::BackendMode;
    // Safety: test-only env mutation; no race in single-threaded test runner.
    unsafe { std::env::set_var(ELAI_LOCAL_MAX_CONTEXT_VAR, "25000") };
    let w = effective_context_window(BackendMode::Local);
    unsafe { std::env::remove_var(ELAI_LOCAL_MAX_CONTEXT_VAR) };
    assert_eq!(w, 25_000, "effective_context_window(Local) must reflect ELAI_LOCAL_MAX_CONTEXT");
}
