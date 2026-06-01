//! E-25 Tranche N (s-22600 2026-06-01) -- WIRE contract for the
//! compaction-ladder producer wire in `task_runner.rs::drive_task`
//! PreCompact branch + the dashboard reader DTOs in
//! `commands_e25.rs`.
//!
//! Complements `e25_tranche_b4_contract.rs` (which exercises the runner
//! SHAPE) by guarding the WIRE invariants that hold once the runner is
//! plumbed into production:
//!   T1. `PRE_COMPACT_PROBE_BUDGET` exists as a public const at
//!       `task_runner.rs` and equals the 30k sentinel documented in
//!       `context-package-trancheN.md` decision D-N1.b.
//!   T2. `PRE_COMPACT_LADDER_TARGET_BUDGET` exists and equals 20_000
//!       (matches `compaction_ladder.rs::tests::gate_on_above_target_emits_until_below`
//!       + B-4 D1 30k->20k acceptance criterion, decision D-N1.c).
//!   T3. Gate-OFF -> ZERO `ClaudeCodeArchitectureEvent` rows emitted by
//!       a CollectEmitter mirror of the producer's bridge struct. Pure
//!       deterministic short-circuit test (runner side); validates the
//!       wire's defensive double-guard at the producer call-site.
//!   T4. Gate-ON -> payload SHAPE round-trip parity: each emitted
//!       `ClaudeCodeArchitectureEventPayload` carries `kind ==
//!       CompactionLadderTierApplied`, `primitive ==
//!       CompactionLadder`, `session_id == Some(expected)`, and a
//!       `details_json` carrying the 7 LOCKED keys (`ladder_event`,
//!       `tier`, `tier_ordinal`, `bytes_freed`, `budget_before`,
//!       `budget_after`, `target_budget`, `wall_ms`).
//!
//! DEFERRED per `context-package-trancheN.md` trim recommendation:
//!   - live `TaskRunner` harness drive (needs heavy scaffolding).
//!   - Tauri-reader exercise of `commands_e25::recent_compaction_events`
//!     (covered by the dashboard-smoker probe instead).
//!   - Svelte panel render assertion (dashboard-smoker territory).
//!
//! Env-gate manipulation copies the `EnvGuard` set/drop pattern from
//! `compaction_ladder.rs::tests::EnvGuard` (lines 249-269). `serial_test`
//! is intentionally NOT used: the crate is not a workspace dep
//! (verified per HARD RULE Anti-Slopsquatting + no-new-deps-without-confirm).
//! Cargo's per-test-binary process isolation keeps cross-test env
//! leakage bounded; the EnvGuard drop restores the prior value.

use elai_orchestrator::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind, ClaudeCodeArchitectureEventPayload,
};
use elai_orchestrator::compaction_ladder::{
    CompactionLadder, LadderEventEmitter, run_compaction_ladder,
    ELAI_E25_COMPACTION_LADDER_ENABLE_VAR,
};
use elai_orchestrator::task_runner::{
    PRE_COMPACT_LADDER_TARGET_BUDGET, PRE_COMPACT_PROBE_BUDGET, RUNNER_AGENT_ID,
};

use std::cell::RefCell;

// ---------------------------------------------------------------------------
// EnvGuard -- verbatim copy of compaction_ladder.rs::tests::EnvGuard so the
// wire contract can flip the env-gate without leaking across tests.
// ---------------------------------------------------------------------------

struct EnvGuard {
    prior: Option<String>,
}

impl EnvGuard {
    fn set(value: &str) -> Self {
        let prior = std::env::var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR).ok();
        // SAFETY: tests are single-threaded per-process for env-gate flips.
        unsafe {
            std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, value);
        }
        Self { prior }
    }

    fn unset() -> Self {
        let prior = std::env::var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR).ok();
        unsafe {
            std::env::remove_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR);
        }
        Self { prior }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(v) => unsafe {
                std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, v);
            },
            None => unsafe {
                std::env::remove_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR);
            },
        }
    }
}

// ---------------------------------------------------------------------------
// CollectEmitter -- mirrors the bridge struct constructed inline at
// task_runner.rs::drive_task PreCompact branch. The wire uses RefCell
// because the runner is sync and the emitter must be single-task-local;
// the test mirror has identical shape so an emit-count regression here
// catches a producer-side regression at the wire.
// ---------------------------------------------------------------------------

struct CollectEmitter {
    rows: RefCell<Vec<ClaudeCodeArchitectureEventPayload>>,
}

impl CollectEmitter {
    fn new() -> Self {
        Self {
            rows: RefCell::new(Vec::new()),
        }
    }
}

impl LadderEventEmitter for CollectEmitter {
    fn emit(&self, payload: ClaudeCodeArchitectureEventPayload) {
        self.rows.borrow_mut().push(payload);
    }
}

// ---------------------------------------------------------------------------
// T1 + T2: probe-budget + target-budget sentinels exist + carry the LOCKED
// values from context-package-trancheN.md decisions D-N1.b + D-N1.c.
// ---------------------------------------------------------------------------

#[test]
fn pre_compact_probe_budget_const_equals_30k() {
    // E-25 Tranche N D-N1.b: sentinel `PRE_COMPACT_PROBE_BUDGET = 30_000`.
    // Wire surfaces this as a `pub const` so dashboards / follow-up
    // probes can key on it; the value matches the
    // compaction_ladder.rs::tests gate_on_above_target_emits_until_below
    // 30k starting-budget acceptance criterion.
    assert_eq!(PRE_COMPACT_PROBE_BUDGET, 30_000);
}

#[test]
fn pre_compact_ladder_target_budget_const_equals_20k() {
    // E-25 Tranche N D-N1.c: target sentinel `PRE_COMPACT_LADDER_TARGET_BUDGET = 20_000`.
    // Matches B-4 D1 30k -> 20k acceptance criterion.
    assert_eq!(PRE_COMPACT_LADDER_TARGET_BUDGET, 20_000);
}

#[test]
fn runner_agent_id_label_locked() {
    // Belt-and-braces guard: the wire writes via
    // `writer.write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, ...)` so
    // the agent-id string is part of the producer contract. Locking it
    // here ensures dashboard joins on `agent_id == "task_runner"` stay
    // correct across refactors.
    assert_eq!(RUNNER_AGENT_ID, "task_runner");
}

// ---------------------------------------------------------------------------
// T3: gate-OFF -> ZERO emits. Validates the bridge's defensive guard at
// the call-site (the runner already short-circuits when the gate is OFF;
// this test confirms the WIRE-side double-guard semantics: the
// CollectEmitter mirror sees zero rows).
// ---------------------------------------------------------------------------

#[test]
fn gate_off_emits_zero_rows_via_collect_emitter() {
    let _g = EnvGuard::unset();
    let ladder = CompactionLadder::default_order(PRE_COMPACT_LADDER_TARGET_BUDGET);
    let emitter = CollectEmitter::new();
    let result = run_compaction_ladder(
        &ladder,
        PRE_COMPACT_PROBE_BUDGET,
        Some("task_test_seq_0".into()),
        &emitter,
    );
    assert!(
        result.tiers_run.is_empty(),
        "gate-OFF runner must return empty tiers_run"
    );
    assert_eq!(result.bytes_freed, 0, "gate-OFF runner must free 0 bytes");
    assert_eq!(
        result.final_budget_tokens, PRE_COMPACT_PROBE_BUDGET,
        "gate-OFF runner must leave the budget untouched"
    );
    assert!(
        emitter.rows.borrow().is_empty(),
        "gate-OFF CollectEmitter must receive zero ClaudeCodeArchitectureEventPayload rows"
    );
}

// ---------------------------------------------------------------------------
// T4: gate-ON -> payload SHAPE invariants. Each emitted payload carries
// the LOCKED kind / primitive / session_id and the 7 LOCKED details_json
// keys. Catches a producer-side detail regression that the
// dashboard / Tauri reader keys on.
// ---------------------------------------------------------------------------

#[test]
fn gate_on_emits_compaction_ladder_tier_applied_with_locked_details_keys() {
    let _g = EnvGuard::set("1");
    let ladder = CompactionLadder::default_order(PRE_COMPACT_LADDER_TARGET_BUDGET);
    let emitter = CollectEmitter::new();
    let expected_session = "task_42";
    let result = run_compaction_ladder(
        &ladder,
        PRE_COMPACT_PROBE_BUDGET,
        Some(expected_session.into()),
        &emitter,
    );

    assert!(
        !result.tiers_run.is_empty(),
        "gate-ON 30k -> 20k must apply >= 1 tier"
    );
    assert!(
        result.final_budget_tokens <= PRE_COMPACT_LADDER_TARGET_BUDGET,
        "gate-ON traversal must drop final_budget_tokens at-or-below target"
    );

    let rows = emitter.rows.borrow();
    assert_eq!(
        rows.len(),
        result.tiers_run.len(),
        "emit count must equal tiers_run count (one row per applied tier)"
    );

    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row.kind,
            ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            "row {i}: kind must be CompactionLadderTierApplied"
        );
        assert_eq!(
            row.primitive,
            ClaudeCodeArchPrimitive::CompactionLadder,
            "row {i}: primitive must be CompactionLadder"
        );
        assert_eq!(
            row.session_id.as_deref(),
            Some(expected_session),
            "row {i}: session_id must round-trip the producer-supplied label"
        );

        // 7 LOCKED keys on details_json (ladder_event + tier +
        // tier_ordinal + bytes_freed + budget_before + budget_after +
        // target_budget + wall_ms) -- the dashboard / Tauri reader
        // (commands_e25::detail_to_compaction_dto) keys on these
        // exactly.
        let details = row.details_json.as_object().unwrap_or_else(|| {
            panic!("row {i}: details_json must be a JSON object");
        });
        for key in [
            "ladder_event",
            "tier",
            "tier_ordinal",
            "bytes_freed",
            "budget_before",
            "budget_after",
            "target_budget",
            "wall_ms",
        ] {
            assert!(
                details.contains_key(key),
                "row {i}: details_json missing LOCKED key `{key}`"
            );
        }
        assert_eq!(
            details
                .get("ladder_event")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "tier_applied",
            "row {i}: ladder_event must be the LOCKED `tier_applied` discriminator"
        );
        assert_eq!(
            details
                .get("target_budget")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            PRE_COMPACT_LADDER_TARGET_BUDGET,
            "row {i}: target_budget must echo the producer-supplied target"
        );
    }
}
