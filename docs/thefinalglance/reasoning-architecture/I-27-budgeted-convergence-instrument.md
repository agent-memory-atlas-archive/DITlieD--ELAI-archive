# Budgeted convergence instrument (I-27)

- **ELAI ids:** I-27 / GD-5 / ID-5 / I-38 floor / landmine #11 / SCHEMA 265
- **Spec status (as designed):** DARK on the live I-4 per-node spend leg (`ELAI-FEATURE-MAP.md:2823-2827`; `WIRE_DARK_I27_SEARCH_NODE_COMPUTE`). CLI `elai bench convergence` + `ConvergenceCurveMeasured` beacon exist (`crates/cli/src/bench_convergence.rs`, `cli/src/main.rs:1634-1637`). Original plan WAVE 4 BUILD (`I-plan/I-27_budgeted_convergence_instrumentation.md:3-7`).
- **Crate / module (spec):** `crates/cli/src/bench_convergence.rs` (`ConvergenceBench` in the plan; live `run` / `run_inner`); SSB `ConvergenceCurveMeasured`; dashboard-v2 H-9 family tile (`ConvergencePanel`).
- **Purpose of this file:** enough architecture to rebuild the verified-outcome-rate vs compute curve. Not a status ledger.

## 1. What it is

The **budgeted convergence instrument** is the public measurement of the search bet: **verified-outcome rate** (W-1-grade, **not** pass@k) versus **total test-time compute** (nodes / tokens / $) per config, with a **Pareto cost/accuracy** view so an operator can pick an operating point. The curve is the **mean** rate with **bootstrap CIs** (paired seeds ≥5, I-38 floor). The **knee** (marginal gain collapses) is **named**. **Monotonicity is measured**: a dip is **flagged**, never smoothed (GD-5) (`I-plan/I-27_budgeted_convergence_instrumentation.md:4-20`, `I-plan/I-SUMMARY_INDEX.md:499-510`).

## 2. Why it exists

I-4 / I-26 spend compute on the claim “more test-time compute → more verified outcomes.” Without this organ the claim is folklore. If it is faked (pass@k, SWE-bench headline, smoothed curve, `--skip-verification` inflating `verified_count`), the cockpit lies (`:18-20,64-68,76-77`; landmine #11 `I-plan/I-0_INDEX.md:571-572`). The harness must **not** be editable by the session it measures (R8) (`:27`).

## 3. How it works (runtime)

1. **Who calls it.** Operator / CI via **`elai bench convergence --suite <fixture>`** (`:61,75`). I-26 collapse may **trigger** a curve emit (`I-plan/I-26_dynamic_thinking_budgets.md:17`) — I-27 **owns** the row shape. The measured agent does not write the beacon.
2. **Input.** Fixture suite at increasing compute envelopes. Plan: drive `elai run` and read **journal** verified-outcome counts + I-4 `SearchNodeEvent` spend (`:10-12,58`). Live Tier-1: fixture TOML supplies `verified` bools per task per seed per level — **no live inference** (`bench_convergence.rs:17-21`).
3. **Floor (deterministic, fail-closed).** `seed_count >= 5` and **paired** tasks across levels (I-38). Violation → **no compute**, `FloorViolation`, **zero** SSB rows (`:12-13,83`; live `floor_check` `:192`, contract C3).
4. **Sweep (deterministic given fixture).** For each compute level L, count W-1 verified fraction. Mean rate + bootstrap 95% CI (`BOOTSTRAP_N_RESAMPLES = 500`, tails 0.025/0.975 — live fill mirroring `bench_qdelta`) (`bench_convergence.rs:60-71`).
5. **Pareto (deterministic).** Second axis: **cost per additional verified task** = `dollar_per_task / mean_rate` when rate > 0 (`:13-14`; live `ConvergenceCurvePoint.cost_per_verified_task`).
6. **Knee (deterministic).** First adjacent pair whose marginal gain collapses; named in the row (`:14-15,76`). Live: first `gain < 0.01` → previous level; else last label (`detect_knee`, `KNEE_GAIN_THRESHOLD`) (`bench_convergence.rs:73-75,334-345`).
7. **Non-monotonic (deterministic, GD-5).** If rate[i] < rate[i-1], append that level to `non_monotonic_at` and report `"non-monotonic at [knee]; investigate."` **Do not smooth** (`:15-17,65`).
8. **Commit.** One **flat** `ConvergenceCurveMeasured` beacon per clean run (`:17,62`). Timestamped TOML report `workflow/benchmarks/convergence_<fixture>_<date>.toml` (`:77`). Cockpit tile **reads** the beacon (`:52,71-72`).
9. **Fail-closed.** `--skip-verification` broken arm that raises `verified_count` without W-1 **FAILS** R2 (`:77`, contract C5). Underpowered seeds: abort, no row.
10. **The model may generate** none of the curve. It may **not** choose the metric, drop a dip, or headline SWE-bench.

## 4. Types, enums, objects

Copy from `I-plan/I-27_budgeted_convergence_instrumentation.md:50-52,75-77` and live payload. Do not invent.

```
// Plan producer name
ConvergenceBench   // fixture sweep + curve + SSB
// Live CLI: pub(crate) fn run / pub fn run_inner / pub fn floor_check

ConvergenceConfig {
  suite: String,
  seed_count: u32,          // floor >= 5
  paired: bool,             // must be true
  levels: Vec<ConvergenceLevel>,
}
ConvergenceLevel {
  label: String,            // e.g. "1x","2x","4x"
  dollar_per_task: Option<f64>,  // E-49 $/CTS
  outcomes: Vec<Vec<bool>>,      // seed-major W-1 verified
}
ConvergenceTaskRow { verified: bool }

ConvergenceCurvePoint {
  compute_level: String,
  mean_rate: f64,           // [0,1]
  ci_low: f64,
  ci_high: f64,
  cost_per_verified_task: Option<f64>,
}

ConvergenceCurveMeasuredPayload {   // FLAT beacon, no Kind
  fixture_suite: String,
  seed_count: u32,
  task_count: u32,
  curve_points: Vec<ConvergenceCurvePoint>,
  knee_compute_level: String,
  non_monotonic_at: Vec<String>,
  seed: u64,                // ELAI_SEED for bootstrap
  fixture_blake3: String,   // live provenance pin
  runpod_pending: bool,     // live: Tier-1 fixture vs live I-4 axis
  ts_ms: i64,
}

BenchConvergenceError {
  FloorViolation { seed_count, paired },
  TooFewLevels { level_count },  // need >= 2
  EmptyLevel { label },
  SeedMismatch { ... },
  // ... (live)
}

FLOOR_MIN_SEED_COUNT: u32 = 5
BOOTSTRAP_N_RESAMPLES: usize = 500          // live; plan: inherit I-38
KNEE_GAIN_THRESHOLD: f64 = 0.01             // live fill; not in plan body
```

Plan **excludes** a Kind discriminator (`:17` “flat beacon”). Later K-18/K-15 riders prefer-reuse `+Kind` arms (`ELAI-FEATURE-MAP.md:1936-1939`) — those are **other** organs. Rebuild I-27 as **flat**.

## 5. Control flow / state machine

**Owner of the loop:** CLI bench, **off-agent**. Measured session cannot patch the harness (`:27`).

```
load TOML suite
  → floor_check (seeds≥5 AND paired) else ABORT, 0 rows
  → require ≥2 compute levels
  → per level: mean verified rate + bootstrap CI + cost/verified
  → detect_knee, detect_non_monotonic (no smoothing)
  → write TOML report
  → emit ONE ConvergenceCurveMeasured
  → cockpit reads beacon
```

**Forced:**

- Live I-4 `SearchNodeEvent` spend axis is **forward-compat** via opaque `compute_level` labels until I-4-promotion (`bench_convergence.rs:23-28`, `wire-dark-allowlist.toml:1939-1956`).
- Live (non-batch) refresh and cross-config overlay = Tier-2 (`:39-41`).
- Tile is **W-6 required**, not deferrable (`:47,71-72`).

## 6. Grounding, receipts, gates

Y-axis is **W-1 verified-outcome rows only**. Compute spend from I-4 `SearchNodeEvent` (plan) or fixture labels (Tier-1). Cost denominator is **E-49 $/CTS**; pass definition **STABLE-Pass** — import, do not redefine (`:23,55,87`).

**Never allowed as the curve metric:** pass@k (`:18,65`); SWE-bench / landmine #11 (`:18-19,68`); LLM-judge “looks verified”; paper efficiency numbers as targets (ID-5) (`:67`); smoothed / interpolated monotone (`:16-17,65`); `--skip-verification` counts (`:77`).

This organ is a **measurement artifact, not a gate** (`:27`). I-38 is the **enforcement** instrument for the statistical floor (`:28,83,91`). Do not re-implement bootstrap; import or **mirror** `bench_qdelta` (`:91`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| paired seeds | **≥5** (I-38) | per run |
| bootstrap | live **500** resamples, 95% CI | per point |
| levels | **≥2** | per suite |
| SSB | **one** beacon per clean run | per invocation |
| module size (plan) | ~120 LOC (`:32`) | advisory |
| SCHEMA (live) | **265** | `ConvergenceCurveMeasured` |

**Overflow / abort:** floor fail → `CONVERGENCE ABORT` + exit 1 (`bench_convergence.rs:365-366`). No fabricated point to fill a dip.

I-26 uses this curve to **calibrate** depth-factor (ID-5); this organ does **not** set I-26 gates.

## 8. Roles and neighbors

Operator-facing. No agent role “runs” the bench on itself as proof (HD-4 cousin: off-agent).

**Consumes:** E-49 `$/CTS` + STABLE-Pass; E-9 `include_str!` fixture runner **shape**; H-9 cockpit family; I-4 `SearchNodeEvent` spend (`:54-58,86-90`).

**Produces:** `ConvergenceCurveMeasured`; TOML report; cockpit tile. K-15 reasoning-budget bench is a **sibling** (x-axis = reason-budget, same y) (`ELAI-FEATURE-MAP.md:2828-2834`) — do not fork this runner. N-1 later unifies benches — cite. J-84 `U_lambda` effort toggle reads this + H-9 (`ELAI-FEATURE-MAP.md:662`).

**Must not fork:** a second bootstrap; a second cockpit; re-instrumenting I-4 (`:90`).

## 9. Config / flags / defaults

- CLI: `elai bench convergence --suite <fixture-set> --seeds 5` (`:75`; live also `--seed`, `--json` — `bench_convergence.rs:1`).
- Live dark flag name: **`WIRE_DARK_I27_SEARCH_NODE_COMPUTE`** (I-4 spend axis), **not** a kill-switch for the CLI (`wire-dark-allowlist.toml:1950-1956`).
- Decision doc: `workflow/reasoning/I-27-decisions.md` (`:37`).
- Frontend: H-9 family; `cross_lang_diff.py` required (`:47`).
- R8: harness not writable by the measured session (`:27`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/cli/src/bench_convergence.rs::ConvergenceBench` | `:51` (live: `run` / `run_inner`) |
| dashboard-v2 convergence tile / `ConvergencePanel` | `:52`; FEATURE-MAP `:2826` |
| `elai bench convergence` | `:61`; `cli/src/main.rs:1634-1637` |
| E-49 / E-9 / H-9 / I-4 SearchNodeEvent | `:54-58` |
| `i27_convergence_instrumentation_contract.rs` | `:36` |
| `workflow/reasoning/I-27-decisions.md` | `:37` |
| live payload | `ssb/payloads.rs:20472-20530` |
| report path | `workflow/benchmarks/convergence_<fixture>_<date>.toml` (`:77`); live also `.elai_cc/benchmarks/convergence/` |

## 11. Journal / SSB events

Plan: **+ConvergenceCurveMeasured** (flat beacon, baseline+1 from 221/532) (`:17,84`).

Live SCHEMA **265**, sql_str `convergence_curve_measured` (contract C1). Payload §4. **ONE** row per clean run (`payloads.rs:20488-20489`).

GD-2: `curve_points` recompute **byte-identically** from the journal/fixture under `ELAI_SEED` (`:76`). Live also pins `fixture_blake3`.

## 12. Tests and observables

Named (`:64-77`, contract C1–C5, `I-plan/I-SUMMARY_INDEX.md:508-510`):

| Id | Assert |
|---|---|
| C1 | action_type round-trip + real-DB CHECK |
| C2 | same fixture+seed → byte-identical `curve_points` / CI (GD-2) |
| C3 | every point `ci_low ≤ mean_rate ≤ ci_high`; `seed_count≥5`; seeds=3 → zero rows |
| C4 | monotone fixture → non-empty `knee_compute_level`; dip → non-empty `non_monotonic_at` |
| C5 | `--skip-verification` inflation: broken `verified_count` > honest → FAIL R2 |
| W-1 only | pass@k absent from the aggregator |
| report | TOML contains curve + CI + Pareto + knee + non-monotonic flags |

Trigger: `elai bench convergence --suite <fixture-set> --seeds 5` (`:75`).

Adversarial: smoothing a dip; SWE-bench in the report title; paper FLOP/J as a target; agent editing `bench_convergence.rs` mid-run; counting unverified pass@k.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- CLI (or equivalent off-agent runner) sweeps **≥2** compute levels on a **paired** fixture with **≥5** seeds.
- Y-axis is **W-1 verified-outcome rate only** (not pass@k).
- Points carry **bootstrap CI**; curve recomputes under seed (GD-2).
- Knee is **named**; non-monotonic levels are **listed, not smoothed** (GD-5).
- Pareto cost-per-additional-verified-task is present when $ data exists (E-49).
- One `ConvergenceCurveMeasured` beacon per clean run; missing row = FAIL.
- `--skip-verification` (or equivalent) is a **failing** broken arm.
- No SWE-bench headline (landmine #11). No paper efficiency gate (ID-5).
- Harness is not writable by the measured session.

A pass@k plot, a smoothed “scaling law” chart, or K-15 without this W-1 y-axis is a **cousin**.

## 14. Non-goals / banned designs

- pass@k as the metric (`:18,65`).
- Fabricated monotonicity (`:19-20,65`).
- Paper compute-efficiency targets (`:67`).
- SWE-bench headline (`:18-19,68`; `I-plan/I-0_INDEX.md:571-572`).
- Redefining E-49 / forking E-9 runner / forking H-9 / re-instrumenting I-4 (`:86-90`).
- Re-implementing I-38 bootstrap (`:91`).
- Live multi-config overlay / streaming refresh in v1 (`:39-41`).
- Waiving the cockpit tile (`:47`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-27_budgeted_convergence_instrumentation.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:499-510`. Landmine #11: `I-plan/I-0_INDEX.md:571-572`. Feature map: `ELAI-FEATURE-MAP.md:2823-2827`. Live: `crates/cli/src/bench_convergence.rs`, `ssb/payloads.rs:20472-20530`, `crates/contracts/tests/i27_convergence_instrumentation_contract.rs`, `wire-dark-allowlist.toml:1939-1956`. Neighbors: I-26 collapse emit; K-15 sibling; I-38 floor.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Plan drives live `elai run`; live Tier-1 is fixture-bools only (`:10-12` vs `bench_convergence.rs:17-21`).
- Plan producer name `ConvergenceBench` vs live `run`/`run_inner` (`:51`).
- Numeric knee threshold (live 0.01) not in the plan (`:14-15`).
- Bootstrap resample count not in the plan (live 500).
- Report directory: plan `workflow/benchmarks/` vs live `.elai_cc/benchmarks/convergence/` (`:77`).
- `runpod_pending` / `fixture_blake3` live fields not in the plan payload list.
- Whether I-26 collapse **must** emit this beacon or only “may” (`I-26:17`).
- How “nodes / tokens / $” share one x-axis when I-4 spend lands (opaque label vs three series) (`:5-6,11`).
- `ConvergencePanel` vs “extension of H-9 cost tile” (`FEATURE-MAP:2826` vs `:45`).
- Whether `--skip-verification` is a real CLI flag or only a contract thought-experiment (plan names it; live C5 simulates via payload).
