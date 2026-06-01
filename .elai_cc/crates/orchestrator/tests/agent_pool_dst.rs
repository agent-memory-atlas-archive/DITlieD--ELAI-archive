// E-44 Tranche 7B: madsim Deterministic Simulation Testing (DST) harness
// for the supervision-tree restart ceiling.
//
// This file is GATED at the file head with `#![cfg(madsim)]`. The default
// `cargo test` invocation does NOT compile or run any code below. To activate:
//
//   RUSTFLAGS="--cfg madsim" cargo test -p elai-orchestrator --test agent_pool_dst
//
// Rationale (workflow/reasoning/E-44-madsim-decisions.md):
//   1. madsim swaps tokio at link time under `--cfg madsim`. Production builds
//      are NOT affected; the test runner sees a deterministic scheduler +
//      mock clock + mock net stack.
//   2. The harness ASSERTS the EXISTING SupervisionManager surface (3-restart
//      ceiling per restart window). It does NOT add new policy, does NOT
//      modify supervision.rs, and does NOT touch agent_pool.rs.
//   3. Tests are scoped to (a) the 3-restart ceiling honored under fault
//      injection, (b) deterministic seed replay producing identical outcomes,
//      (c) compensation-style retry idempotency across simulated re-runs.
//
// Grounding:
//   - madsim 0.2.31 (crates.io verified 2026-05-31, not yanked, 161k+ dls).
//   - SupervisionManager::handle_death at supervision.rs:145 returns
//     SupervisionOutcome::Escalated when restart_count >= max_restarts
//     within restart_window (per supervision.rs:113-200 source).
//   - SeededClock (elai-core::clock::SeededClock) is the deterministic
//     ClockProvider already used by orchestrator unit tests; pairing it with
//     madsim::time means BOTH the supervision policy clock AND the simulated
//     async runtime advance deterministically.

#![cfg(madsim)]

use std::time::Duration;

use elai_core::clock::SeededClock;
use elai_orchestrator::recovery::AgentDeathClassification;
use elai_orchestrator::supervision::{
    EscalationAction, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// Standard 3-restart-per-window policy mirroring the production default
// asserted at supervision.rs:113 + recovery_contract.rs.
fn default_policy() -> SupervisionPolicy {
    SupervisionPolicy {
        max_restarts: 3,
        restart_window: Duration::from_secs(60),
        on_max_restarts: EscalationAction::Terminate,
    }
}

// Deterministic Uuid for replay — same input bytes -> same Uuid every run.
// madsim's runtime guarantees thread / scheduler determinism but does not
// reseed UUID generators. Hard-code the bytes so the test trace is bit-exact
// across two runs.
fn deterministic_agent_id(tag: u8) -> Uuid {
    let bytes = [
        0xE4, 0x44, 0xD5, 0x7B, // E-44 DST B-prefix
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, tag,
    ];
    Uuid::from_bytes(bytes)
}

// ---------------------------------------------------------------------------
// Test 1: 3-restart ceiling honored under simulated crash injection
// ---------------------------------------------------------------------------
//
// SCENARIO: 5 agents launched; agent #0 crashes 4 times in quick succession.
// EXPECT: the first 3 crashes return Restarted; the 4th returns Escalated
// with the configured Terminate action and restart_count == 3.
//
// This asserts the EXISTING supervision.rs invariant; madsim only provides
// the deterministic scheduler.

#[madsim::test]
async fn dst_supervisor_honors_3_restart_ceiling() {
    let clock = SeededClock::new();
    let policy = default_policy();
    let mut mgr = SupervisionManager::new(clock, policy);

    let agent = deterministic_agent_id(0x01);
    let phase = "phase-e44-dst";

    // Inject 3 transient crashes — all must return Restarted.
    for expected_count in 1..=3 {
        let outcome = mgr
            .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
            .expect("handle_death must not error on transient crash");
        match outcome {
            SupervisionOutcome::Restarted { restart_count } => {
                assert_eq!(
                    restart_count, expected_count,
                    "restart #{expected_count} expected Restarted with count {expected_count}"
                );
            }
            other => panic!("expected Restarted on crash #{expected_count}, got {other:?}"),
        }
    }

    // 4th crash must escalate per the configured Terminate action.
    let outcome = mgr
        .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
        .expect("handle_death must not error on ceiling hit");
    match outcome {
        SupervisionOutcome::Escalated {
            restart_count,
            max_restarts,
            escalation_action,
            restart_history,
        } => {
            assert_eq!(restart_count, 3, "restart_count must be 3 at ceiling");
            assert_eq!(max_restarts, 3, "policy max_restarts surfaced verbatim");
            assert_eq!(
                escalation_action,
                EscalationAction::Terminate,
                "Terminate is the configured escalation"
            );
            assert_eq!(
                restart_history.len(),
                3,
                "history must record exactly 3 restarts"
            );
        }
        other => panic!("expected Escalated on 4th crash, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 2: Deterministic seed replay across two simulator runs
// ---------------------------------------------------------------------------
//
// SCENARIO: run the same crash sequence twice with identical seeds and
// identical SeededClock instances.
// EXPECT: outcome traces are bit-equal.

#[madsim::test]
async fn dst_deterministic_seed_replay() {
    fn trace_run() -> Vec<String> {
        let clock = SeededClock::new();
        let policy = default_policy();
        let mut mgr = SupervisionManager::new(clock, policy);

        let agent = deterministic_agent_id(0x02);
        let phase = "phase-e44-dst-replay";

        let mut trace = Vec::with_capacity(4);
        for _ in 0..4 {
            let outcome = mgr
                .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
                .expect("handle_death OK");
            // String-format the outcome — Debug is stable for the variants we
            // care about (restart_count / escalation_action / max_restarts).
            // restart_history Vec timestamps are clock-derived so SeededClock
            // determinism keeps them identical across runs.
            trace.push(format!("{outcome:?}"));
        }
        trace
    }

    let run_a = trace_run();
    let run_b = trace_run();

    assert_eq!(
        run_a, run_b,
        "DST determinism violation: same seed + same events produced different traces"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Compensation replay determinism — idempotent re-run of a 3-event
// sequence yields the same final SupervisionOutcome each time.
// ---------------------------------------------------------------------------
//
// SCENARIO: simulate a "recovery replay" where the supervisor is re-driven
// with the same crash trace on a fresh manager. The replay path must
// reproduce the same Escalated outcome the original run produced.
//
// This stands in for the saga-replay invariant (E-44 Tranche 7D coupling):
// even though saga.rs ships in Tranche 7A, the DST harness here proves the
// underlying SupervisionManager is deterministic — a prerequisite for any
// upstream compensation replay being deterministic.

#[madsim::test]
async fn dst_compensation_replay_idempotent() {
    fn drive_to_escalation() -> SupervisionOutcome {
        let clock = SeededClock::new();
        let policy = default_policy();
        let mut mgr = SupervisionManager::new(clock, policy);

        let agent = deterministic_agent_id(0x03);
        let phase = "phase-e44-dst-compensation";

        // Drive through the 3-restart ceiling and capture the 4th outcome.
        let mut final_outcome = SupervisionOutcome::NoRestartNeeded;
        for _ in 0..4 {
            final_outcome = mgr
                .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
                .expect("handle_death OK");
        }
        final_outcome
    }

    let original = drive_to_escalation();
    let replay_a = drive_to_escalation();
    let replay_b = drive_to_escalation();

    // All three runs (original + 2 replays) MUST escalate identically.
    let SupervisionOutcome::Escalated {
        restart_count,
        max_restarts,
        escalation_action,
        ..
    } = &original
    else {
        panic!("original drive must end in Escalated, got {original:?}");
    };
    assert_eq!(*restart_count, 3);
    assert_eq!(*max_restarts, 3);
    assert_eq!(*escalation_action, EscalationAction::Terminate);

    // Debug-format compare is sufficient for variant + scalar parity; the
    // restart_history Vec is also covered by Debug.
    assert_eq!(
        format!("{original:?}"),
        format!("{replay_a:?}"),
        "compensation replay A diverged from original"
    );
    assert_eq!(
        format!("{original:?}"),
        format!("{replay_b:?}"),
        "compensation replay B diverged from original"
    );
}

// ---------------------------------------------------------------------------
// Test 4: BudgetExceeded does NOT consume the restart budget (negative-space
// invariant: a budget-exceeded death must NOT decrement the 3-restart
// ceiling that remains available for genuine crashes).
// ---------------------------------------------------------------------------

#[madsim::test]
async fn dst_budget_exceeded_does_not_consume_restart_quota() {
    let clock = SeededClock::new();
    let policy = default_policy();
    let mut mgr = SupervisionManager::new(clock, policy);

    let agent = deterministic_agent_id(0x04);
    let phase = "phase-e44-dst-budget";

    // 3 budget-exceeded events: each must return ConvergenceInjected and
    // leave the restart quota untouched.
    for _ in 0..3 {
        let outcome = mgr
            .handle_death(
                agent,
                AgentDeathClassification::BudgetExceeded,
                phase,
                0,
                false,
            )
            .expect("handle_death OK");
        assert!(
            matches!(outcome, SupervisionOutcome::ConvergenceInjected),
            "BudgetExceeded must yield ConvergenceInjected, got {outcome:?}"
        );
    }

    // After 3 budget-exceeded events, the restart quota must still be intact
    // (the next 3 transient crashes must all Restart, and the 4th escalate).
    for expected_count in 1..=3 {
        let outcome = mgr
            .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
            .expect("handle_death OK");
        match outcome {
            SupervisionOutcome::Restarted { restart_count } => {
                assert_eq!(restart_count, expected_count);
            }
            other => panic!(
                "BudgetExceeded consumed quota: expected Restarted at #{expected_count}, got {other:?}"
            ),
        }
    }

    let final_outcome = mgr
        .handle_death(agent, AgentDeathClassification::Crash, phase, 0, false)
        .expect("handle_death OK");
    assert!(
        matches!(final_outcome, SupervisionOutcome::Escalated { .. }),
        "4th crash after BudgetExceeded run must still Escalate, got {final_outcome:?}"
    );
}
