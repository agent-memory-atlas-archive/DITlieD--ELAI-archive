E-44 Tranche 7B — madsim Deterministic Simulation Testing decisions

Author: executor (Tranche 7B sub-phase)
Date: 2026-05-31
Status: SHIPPED
Scope: dev-dependency wiring + DST harness test file only. No production code touched.

DECISION 1 — Why madsim

madsim is the established deterministic-concurrency simulator for Rust (github.com/madsim-rs/madsim, Apache-2.0/MIT, used in production by RisingWave). It replaces tokio's scheduler with a deterministic one under `#[cfg(madsim)]` so concurrency-sensitive invariants (restart ceilings, replay determinism, ordering) can be asserted bit-exactly across runs. The E-44 supervision tree's 3-restart-per-window ceiling is exactly the class of invariant DST exists to cover.

DECISION 2 — Why dev-dependency + #![cfg(madsim)] at file head

Two independent gates so the default build pays zero cost:
(a) `[dev-dependencies]` keeps madsim out of the production graph; `cargo build` / `cargo check` (library) never see it.
(b) `#![cfg(madsim)]` at the head of tests/agent_pool_dst.rs means the entire file is excluded from compilation under the default cfg set. `cargo test` does NOT compile the DST harness.

Activation is one-line opt-in via RUSTFLAGS. Per metrics/phase_3I.toml line 31-34, prior madsim cfg-gated compile cost was 83 seconds — acceptable for a dedicated nightly / on-demand check.

DECISION 3 — Version pin rationale

madsim 0.2.31 + madsim-tokio 0.2.30 (verified live on crates.io 2026-05-31):
- madsim 0.2.31 published 2024-10-18, not yanked, 161k+ downloads
- madsim-tokio 0.2.30 published 2024-10-04, not yanked, 5.1M+ downloads
- both under MIT/Apache-2.0 dual license (workspace-compatible)
- anti-slopsquatting: name matches upstream repo (github.com/madsim-rs/madsim) and the canonical crates.io entry; no typo-squat siblings of concern

Newer point-release 0.2.34 exists (2025-10-11) but the user-locked plan pins 0.2.31 per the context-package sub-phase brief; 0.2.31 is the minor that adds the API surface the test harness exercises. We honor the lock; bumping is a separate decision tracked as RISK-LOW.

DECISION 4 — Asserting existing surface, not adding new policy

The DST harness imports SupervisionManager / SupervisionPolicy / SupervisionOutcome / EscalationAction / AgentDeathClassification and exercises them through their existing handle_death entry point at supervision.rs:145. No new types. No modification to supervision.rs or agent_pool.rs. This honors the Tranche 7B constraint that 7B is BACKEND DST TEST infrastructure only.

DECISION 5 — Why SeededClock pairing

madsim provides deterministic async runtime + simulated time, but SupervisionManager is generic over ClockProvider and the policy clock advances independently of the runtime clock. Pairing madsim with SeededClock (elai-core::clock::SeededClock) means BOTH clocks are deterministic, so the restart_history vector embedded in SupervisionOutcome::Escalated is bit-equal across runs. Without SeededClock the test would be flaky on the timestamp dimension even under madsim.

DECISION 6 — Deterministic Uuids for replay

Uuid::new_v4() is not deterministic even under madsim (it draws from getrandom, not the runtime). We hard-code Uuid bytes per test (deterministic_agent_id helper) so the same input bytes produce the same Uuid each replay. This is the canonical workaround documented across the madsim ecosystem.

DECISION 7 — Test coverage rationale (4 tests, ~250 LOC)

1. dst_supervisor_honors_3_restart_ceiling — asserts the cardinal invariant: 3 transient crashes restart, 4th escalates with the configured Terminate action.
2. dst_deterministic_seed_replay — proves the same seed + same events yield bit-equal Debug traces across two runs. Foundation for any upstream replay reasoning.
3. dst_compensation_replay_idempotent — proves three independent drives (original + 2 replays) reach the same Escalated outcome. Stands in for the saga.rs Tranche 7A compensation-replay invariant E2E coverage (Tranche 7D will close that loop end-to-end).
4. dst_budget_exceeded_does_not_consume_restart_quota — negative-space invariant: BudgetExceeded is a separate code path (ConvergenceInjected) and MUST NOT decrement the restart budget reserved for genuine crashes. Catches the "budget death stole my restart quota" regression.

Out of scope: fault-injection on `AgentPool::launch_agent_process` directly. Per the exploration report (tranche7-E44-wiring.md GAP A), no `supervise` method currently exists on AgentPool — extracting one is a Tranche 7A or later concern. The current harness asserts the policy layer (SupervisionManager) which is the layer the 3-restart ceiling actually lives in.

DECISION 8 — Activation invocation

```
RUSTFLAGS="--cfg madsim" cargo test -p elai-orchestrator --test agent_pool_dst
```

Default `cargo test` skips the file silently. CI can opt in via a dedicated job that sets RUSTFLAGS; budget per metrics/phase_3I.toml line 34: madsim_compile_time_sec = 83.0 (one-time per workspace check; subsequent test compiles are incremental).

OPEN QUESTION (deferred)

- madsim 0.2.34 bump: newer point-release exists; not adopted in 7B per the locked plan. Bump can be a follow-up if a future Tranche surfaces an API the 0.2.31 line lacks.
- AgentPool::supervise extraction: GAP A from the exploration report. Out of scope for 7B; if Tranche 7A or later adds a `supervise` method, a follow-on DST test can target it directly.

GROUNDING REFERENCES
- crates.io madsim 0.2.31 (verified 2026-05-31, not yanked)
- crates.io madsim-tokio 0.2.30 (verified 2026-05-31, not yanked)
- github.com/madsim-rs/madsim (canonical upstream, Apache-2.0/MIT)
- workflow/sessions/s-21636/context-package-tranche7-E44.md (the locked plan)
- workflow/sessions/s-21636/exploration/tranche7-E44-wiring.md (GAP A, RISK D-E)
- .elai_cc/crates/orchestrator/src/supervision.rs (the surface being asserted)
- .elai_cc/crates/orchestrator/src/recovery.rs (AgentDeathClassification source)
- .elai_cc/crates/core/src/clock.rs (SeededClock source)
- .elai_cc/metrics/phase_3I.toml (madsim_compile_time_sec=83.0 baseline)
