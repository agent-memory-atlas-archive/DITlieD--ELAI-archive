//! J-25 T4 — graded pressure ladder contract (source-wiring gate).
//!
//! Verifies the structural claims from the J-25 WIRING_CONTRACT for the T4
//! slice (graded `PressureLevel` trigger input for `maybe_auto_compact`):
//!
//! D1: `pressure_level` fn + `PressureLevel` enum exist in
//!     crates/orchestrator/src/context_policy.rs and are re-exported from
//!     lib.rs (module reachable from the crate root).
//! D2: `pressure_level` has a non-test production caller in
//!     conversation.rs::maybe_auto_compact (R6/R7 production reachability).
//! D3: grade boundaries correct at 0% / 49% / 50% / 70% / 85% / 100% of
//!     usable (unit-logic, each boundary).
//! D4: the flat `auto_compaction_input_tokens_threshold` check is NOT the
//!     sole gate at the tick site — the grade is read and branches behaviour
//!     (Normal short-circuits before the flat check).
//! D5: composes with ContextSlot floors — `pressure_level` source does NOT
//!     redeclare `FLOOR_SSB_ENTRIES` / `FLOOR_GRC_HISTORY` /
//!     `FLOOR_OUTPUT_RESERVE` (no parallel floor system).
//! D6: kill-switch path — when `ELAI_CONTEXT_POLICY_DISABLE=1`,
//!     `pressure_level` collapses to the flat-legacy behaviour
//!     (`Critical` if `used >= LEGACY_FLAT_THRESHOLD`, else `Normal`).
//!
//! The NEGATIVE leg (grade fn never read at the tick => DARK) is locked by
//! D2 + D4 together: the fn is called inside `maybe_auto_compact` AND the
//! grade drives the branch (Normal returns early, the flat check is
//! secondary). The unit-logic tests in context_policy.rs::tests cover the
//! boundary arithmetic; this contract test locks the structural wiring.

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
// D1: pressure_level + PressureLevel exist + re-exported from lib.rs
// ---------------------------------------------------------------------------

/// D1a: context_policy.rs exports the fn + enum.
#[test]
fn d1a_pressure_level_symbols_exist() {
    let src = read_src("orchestrator/src/context_policy.rs");
    assert!(
        src.contains("pub enum PressureLevel"),
        "context_policy.rs must export pub enum PressureLevel"
    );
    assert!(
        src.contains("pub fn pressure_level"),
        "context_policy.rs must export pub fn pressure_level"
    );
    assert!(
        src.contains("pub fn usable_context_tokens"),
        "context_policy.rs must export pub fn usable_context_tokens"
    );
    assert!(
        src.contains("pub const DEFAULT_MAX_OUTPUT_CAP"),
        "context_policy.rs must export pub const DEFAULT_MAX_OUTPUT_CAP"
    );
}

/// D1b: lib.rs re-exports the fn + enum + const + helper.
#[test]
fn d1b_pressure_level_reexported_from_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    let block_start = src.find("pub use context_policy::{");
    assert!(block_start.is_some(), "lib.rs must have a context_policy pub use block");
    let block = &src[block_start.unwrap()..];
    let block_end = block.find("};").expect("pub use block terminates");
    let block = &block[..block_end];
    assert!(block.contains("PressureLevel"), "lib.rs must re-export PressureLevel");
    assert!(block.contains("pressure_level"), "lib.rs must re-export pressure_level");
    assert!(block.contains("usable_context_tokens"), "lib.rs must re-export usable_context_tokens");
    assert!(block.contains("DEFAULT_MAX_OUTPUT_CAP"), "lib.rs must re-export DEFAULT_MAX_OUTPUT_CAP");
}

// ---------------------------------------------------------------------------
// D2: non-test production caller in conversation.rs::maybe_auto_compact
// ---------------------------------------------------------------------------

/// D2: conversation.rs::maybe_auto_compact calls pressure_level. The call
/// must be inside the fn body (not only in a test module).
#[test]
fn d2_maybe_auto_compact_calls_pressure_level() {
    let src = read_src("orchestrator/src/conversation.rs");
    let fn_start = src.find("fn maybe_auto_compact");
    assert!(fn_start.is_some(), "conversation.rs must define maybe_auto_compact");
    let body = &src[fn_start.unwrap()..];
    // Find the end of the fn (next `fn ` at the same indent or end of file).
    // For a robust check, just assert the call appears after the fn start and
    // before the next `fn ` definition.
    let next_fn = body[1..].find("\n    fn ").map(|i| i + 1).unwrap_or(body.len());
    let body = &body[..next_fn];
    assert!(
        body.contains("pressure_level("),
        "maybe_auto_compact must call pressure_level inside its body"
    );
    assert!(
        body.contains("PressureLevel::Normal"),
        "maybe_auto_compact must branch on PressureLevel::Normal"
    );
}

// ---------------------------------------------------------------------------
// D3: grade boundaries at 0 / 49 / 50 / 70 / 85 / 100% of usable
// ---------------------------------------------------------------------------

/// D3: exercise pressure_level directly via the orchestrator crate's public
/// API at each grade boundary. Uses `ContextPolicy::Cloud` and a window of
/// 200_000 so `usable = 200_000 - 20_000 = 180_000`. Boundaries:
///   0%   -> Normal
///   49%  -> Normal
///   50%  -> Moderate
///   70%  -> High
///   85%  -> Critical
///   100% -> Critical
#[test]
fn d3_grade_boundaries() {
    use elai_orchestrator::{pressure_level, ContextPolicy, PressureLevel};

    // Serialize env-mutating assertions: the kill-switch must be OFF for the
    // graded path to be exercised.
    let prior = std::env::var("ELAI_CONTEXT_POLICY_DISABLE").ok();
    std::env::remove_var("ELAI_CONTEXT_POLICY_DISABLE");

    let window: u32 = 200_000;
    // usable = 200_000 - 20_000 = 180_000
    let usable: u32 = 180_000;
    let policy = ContextPolicy::Cloud;

    // Integer ratios so 70% and 85% land exactly on the grade boundaries
    // (f64 * 0.70 then `as u32` truncates 125999.999... to 125999 = 69.999%,
    // which lands in Moderate, not High). usable*70/100 = 126000 exactly.
    let cases = [
        (0u32, PressureLevel::Normal),
        (usable * 49 / 100, PressureLevel::Normal),
        (usable * 50 / 100, PressureLevel::Moderate),
        (usable * 69 / 100, PressureLevel::Moderate),
        (usable * 70 / 100, PressureLevel::High),
        (usable * 84 / 100, PressureLevel::High),
        (usable * 85 / 100, PressureLevel::Critical),
        (usable, PressureLevel::Critical),
    ];

    for (used, expected) in cases {
        let got = pressure_level(used, window, policy);
        assert_eq!(
            got, expected,
            "used={used} usable={usable} ({}%): expected {expected:?}, got {got:?}",
            (used as f64 / usable as f64) * 100.0
        );
    }

    // Restore prior env state.
    if let Some(v) = prior {
        std::env::set_var("ELAI_CONTEXT_POLICY_DISABLE", v);
    }
}

/// D3b: usable == 0 collapses to Critical (no division by zero, no false
/// Normal).
#[test]
fn d3b_usable_zero_is_critical() {
    use elai_orchestrator::{pressure_level, ContextPolicy, PressureLevel};

    let prior = std::env::var("ELAI_CONTEXT_POLICY_DISABLE").ok();
    std::env::remove_var("ELAI_CONTEXT_POLICY_DISABLE");

    // window = 10_000 < DEFAULT_MAX_OUTPUT_CAP (20_000) => usable = 0
    let level = pressure_level(0, 10_000, ContextPolicy::Cloud);
    assert_eq!(
        level,
        PressureLevel::Critical,
        "usable==0 must collapse to Critical, not Normal"
    );

    if let Some(v) = prior {
        std::env::set_var("ELAI_CONTEXT_POLICY_DISABLE", v);
    }
}

// ---------------------------------------------------------------------------
// D4: flat threshold is NOT the sole gate — grade drives entry
// ---------------------------------------------------------------------------

/// D4: inside maybe_auto_compact, the `PressureLevel::Normal` branch must
/// return BEFORE the flat `auto_compaction_input_tokens_threshold` check.
/// This locks the "grade drives entry; threshold is secondary" invariant.
#[test]
fn d4_grade_drives_entry_not_flat_threshold() {
    let src = read_src("orchestrator/src/conversation.rs");
    let fn_start = src.find("fn maybe_auto_compact").expect("maybe_auto_compact exists");
    let body = &src[fn_start..];
    let next_fn = body[1..].find("\n    fn ").map(|i| i + 1).unwrap_or(body.len());
    let body = &body[..next_fn];

    // Search for the flat-threshold COMPARISON, not the earlier comment that
    // mentions the field name. The real gate is
    // `input_tokens < self.auto_compaction_input_tokens_threshold`.
    let normal_idx = body.find("PressureLevel::Normal").expect("Normal branch present");
    let flat_idx = body
        .find("input_tokens < self.auto_compaction_input_tokens_threshold")
        .expect("flat threshold comparison present");
    assert!(
        normal_idx < flat_idx,
        "PressureLevel::Normal branch must come BEFORE the flat threshold comparison so the grade drives entry"
    );
    // The Normal branch must return early.
    let normal_branch = &body[normal_idx..flat_idx];
    assert!(
        normal_branch.contains("return None"),
        "Normal branch must short-circuit with `return None` before the flat check"
    );
}

// ---------------------------------------------------------------------------
// D5: composes with ContextSlot floors — does NOT redeclare them
// ---------------------------------------------------------------------------

/// D5: context_policy.rs must NOT redeclare the floor constants that
/// context_budget.rs owns. T4 composes with the existing floors; it does
/// not build a parallel floor system.
#[test]
fn d5_no_parallel_floor_redeclaration() {
    let src = read_src("orchestrator/src/context_policy.rs");
    // The floor constants live in context_budget.rs; redeclaring them here
    // would be the parallel-floor smell S1 §4 warns about.
    assert!(
        !src.contains("FLOOR_SSB_ENTRIES"),
        "context_policy.rs must not redeclare FLOOR_SSB_ENTRIES (owned by context_budget.rs)"
    );
    assert!(
        !src.contains("FLOOR_GRC_HISTORY"),
        "context_policy.rs must not redeclare FLOOR_GRC_HISTORY (owned by context_budget.rs)"
    );
    assert!(
        !src.contains("FLOOR_OUTPUT_RESERVE"),
        "context_policy.rs must not redeclare FLOOR_OUTPUT_RESERVE (owned by context_budget.rs)"
    );
    // The floor constants must still exist in context_budget.rs (compose, not
    // delete).
    let budget = read_src("orchestrator/src/context_budget.rs");
    assert!(budget.contains("FLOOR_SSB_ENTRIES"), "context_budget.rs must still define FLOOR_SSB_ENTRIES");
    assert!(budget.contains("FLOOR_GRC_HISTORY"), "context_budget.rs must still define FLOOR_GRC_HISTORY");
    assert!(budget.contains("FLOOR_OUTPUT_RESERVE"), "context_budget.rs must still define FLOOR_OUTPUT_RESERVE");
}

// ---------------------------------------------------------------------------
// D6: kill-switch path collapses to flat-legacy behaviour
// ---------------------------------------------------------------------------

/// D6: with `ELAI_CONTEXT_POLICY_DISABLE=1`, pressure_level returns Critical
/// when `used >= LEGACY_FLAT_THRESHOLD` and Normal otherwise — matching the
/// `trim_threshold` kill-switch semantics.
#[test]
fn d6_kill_switch_collapses_to_flat_legacy() {
    use elai_orchestrator::{
        pressure_level, ContextPolicy, PressureLevel, LEGACY_FLAT_THRESHOLD,
    };

    let prior = std::env::var("ELAI_CONTEXT_POLICY_DISABLE").ok();
    std::env::set_var("ELAI_CONTEXT_POLICY_DISABLE", "1");

    // Kill-switch: grade collapses, window/policy are ignored.
    let below = pressure_level(LEGACY_FLAT_THRESHOLD - 1, 200_000, ContextPolicy::Cloud);
    assert_eq!(
        below,
        PressureLevel::Normal,
        "kill-switch ON, used < LEGACY_FLAT_THRESHOLD => Normal"
    );
    let at = pressure_level(LEGACY_FLAT_THRESHOLD, 200_000, ContextPolicy::Cloud);
    assert_eq!(
        at,
        PressureLevel::Critical,
        "kill-switch ON, used >= LEGACY_FLAT_THRESHOLD => Critical"
    );
    let above = pressure_level(LEGACY_FLAT_THRESHOLD + 50_000, 200_000, ContextPolicy::Cloud);
    assert_eq!(
        above,
        PressureLevel::Critical,
        "kill-switch ON, used > LEGACY_FLAT_THRESHOLD => Critical"
    );

    if let Some(v) = prior {
        std::env::set_var("ELAI_CONTEXT_POLICY_DISABLE", v);
    } else {
        std::env::remove_var("ELAI_CONTEXT_POLICY_DISABLE");
    }
}

// ---------------------------------------------------------------------------
// D7: maybe_auto_compact uses context_backend_mode / regime_policy /
//     effective_context_window, NOT hardcoded BackendMode::Api /
//     ContextPolicy::Cloud as the sole window source.
// ---------------------------------------------------------------------------

/// D7: the tick site must thread the runtime's `context_backend_mode` through
/// `regime_policy` and `effective_context_window` so Local sessions grade
/// against the local window. The fn body must NOT contain
/// `BackendMode::Api` or `ContextPolicy::Cloud` as the sole window source
/// (those constants as the only source is the verifier-reject bug).
#[test]
fn d7_uses_context_backend_mode_not_hardcoded_cloud() {
    let src = read_src("orchestrator/src/conversation.rs");
    let fn_start = src.find("fn maybe_auto_compact").expect("maybe_auto_compact exists");
    let body = &src[fn_start..];
    let next_fn = body[1..].find("\n    fn ").map(|i| i + 1).unwrap_or(body.len());
    let body = &body[..next_fn];

    assert!(
        body.contains("self.context_backend_mode"),
        "maybe_auto_compact must read self.context_backend_mode so the regime is runtime-threaded"
    );
    assert!(
        body.contains("regime_policy("),
        "maybe_auto_compact must derive the policy via regime_policy(mode)"
    );
    assert!(
        body.contains("effective_context_window("),
        "maybe_auto_compact must derive the window via effective_context_window(mode)"
    );
    assert!(
        !body.contains("BackendMode::Api"),
        "maybe_auto_compact must NOT hardcode BackendMode::Api as the window source"
    );
    assert!(
        !body.contains("ContextPolicy::Cloud"),
        "maybe_auto_compact must NOT hardcode ContextPolicy::Cloud as the policy source"
    );
}

/// D7b: ConversationRuntime carries a `context_backend_mode` field and a
/// `with_context_backend_mode` builder so the regime can be threaded from
/// the construction site without changing the default (back-compat for
/// tests that use `new` / `new_with_features`).
#[test]
fn d7b_runtime_carries_context_backend_mode_field_and_builder() {
    let src = read_src("orchestrator/src/conversation.rs");
    assert!(
        src.contains("context_backend_mode: BackendMode"),
        "ConversationRuntime must declare a context_backend_mode: BackendMode field"
    );
    assert!(
        src.contains("pub fn with_context_backend_mode"),
        "ConversationRuntime must expose a with_context_backend_mode builder"
    );
    // Default in new_with_features is BackendMode::Api (back-compat). Assert
    // against the whole source: a slice ending at the first `    }\n` would
    // truncate at the usage_tracker block close, before the Self { ... } init
    // where the default actually lives.
    assert!(
        src.contains("context_backend_mode: BackendMode::Api"),
        "new_with_features must default context_backend_mode to BackendMode::Api for back-compat"
    );
}

/// D7c: runtime_builder.rs wires `with_context_backend_mode(backend)` at the
/// production construction site so Local sessions grade against the local
/// window (the H-50 flat-threshold wire-in neighbor).
#[test]
fn d7c_runtime_builder_wires_context_backend_mode() {
    let src = read_src("cli/src/runtime_builder.rs");
    assert!(
        src.contains("with_context_backend_mode(backend)"),
        "runtime_builder.rs must call .with_context_backend_mode(backend) at the production wire site"
    );
}
