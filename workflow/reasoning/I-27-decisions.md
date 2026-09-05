I-27 DECISIONS — budgeted convergence instrumentation
Session: s-107b022a / I-27-exec  Date: 2026-06-14  HEAD: 95278ed

KEY DECISIONS

D-1  Compute-spend axis (fork A185): use Option A — proxy from cost_per_task_cents
     (the $/CTS cost denominator already in coding_metrics::types). The I-4
     SearchNodeEvent per-node leg is absent (I-4 UNBUILT). The fixture sweep
     steps compute envelopes at the task level (nodes = task_count * seeds).
     The SearchNodeEvent-resolution leg is registered WIRE-DARK[I-27] + stubs.md.
     Rationale: plan §REVISED scope explicitly lists "nodes/tokens/$" as the axis;
     cost_per_task_cents is the live $/CTS denominator (E-49 SHIPPED).

D-2  bootstrap_ci: mirrored from bench_qdelta.rs (xorshift64, same algorithm as
     safedelta_runner::bootstrap_ci). Cannot import (private fn). Mirror declared
     in COMPOSES-WITH. The CI operates on verified_count booleans per task.

D-3  Verified-outcome row (W-7 gap): W-1-grade source. The sweep drives run_inner
     over fixture tasks and counts results where verified=true per task per seed.
     At Tier-1 (fixture leg), tasks carry a `verified` bool field directly (no
     live inference). This is the same shape as QdeltaTaskRow — the fixture
     supplies ground-truth outcomes. The live leg (task_runner result manifests)
     is deferred (Tier-2/PENDING-on-I-4 production runner).

D-4  Flat beacon: ONE ConvergenceCurveMeasured row per `elai bench convergence`
     run, carrying curve_points[] inline in the msgpack payload. The payload is
     bounded by the fixture task count and seed count (both small at Tier-1).

D-5  Monotonicity: if mean_rate at compute level L_i < mean_rate at L_{i-1},
     the non_monotonic_at field names level L_i. Nothing is smoothed (GD-5).

D-6  Knee detection: the knee is the level where the marginal gain
     (rate[i] - rate[i-1]) drops below KNEE_GAIN_THRESHOLD (0.01). If none,
     knee_compute_level = the last level.

D-7  Fixture format: mirrors QdeltaConfig with compute_levels[] array instead of
     task-level on/off booleans. Each compute level carries a Vec<bool> verified.

D-8  SSB schema: ConvergenceCurveMeasured is a flat beacon (no Kind discriminator,
     same as QdeltaMeasured). One action_type string: "convergence_curve_measured".
     +1 entry in SSB_ACTION_TYPES => SCHEMA auto-derives to 265.

D-9  Production wiring: CLI subcommand BenchConvergence {suite, seeds, json}
     dispatched from main.rs, exactly mirrors BenchQdelta pattern.

D-10 Wire-dark allowlist row: none needed for the bench itself (fully wired).
     The per-node SearchNodeEvent resolution leg carries WIRE-DARK[I-27] marker
     in a comment inside bench_convergence.rs; backed by a stubs.md entry.

EXPECTED TEST OUTCOME
- cargo check --workspace --tests: EXIT 0
- action_type_lock golden regen: +1 entry (convergence_curve_measured)
- i27_convergence_instrumentation_contract: all arms PASS (authored)
- e45 cross-language drift contract: PASS

UNKNOWNS AT SHIP TIME
- Live fixture-to-DB path (E-49 coding_metrics) not exercised at Tier-1;
  contract uses inline fixture data. Deferred to Tier-2 (post-I-4).
- Svelte tile cross-lang pair: authored below as ConvergencePanel.svelte
  extending CostPanel.svelte family. svelte-check = auditor responsibility.
