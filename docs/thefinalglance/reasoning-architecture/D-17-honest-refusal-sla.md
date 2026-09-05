# Honest-refusal SLA (D-17)

- **ELAI ids:** D-17 / RPS.VIII / D0-021 / D0-052 / D003-D17-E001..E024 / SUP-009
- **Spec status (as designed):** DARK(`ELAI_REFUSAL_SLA_TICK_ENABLE`) (`ELAI-FEATURE-MAP.md:606-607`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:122`). Live tick gate: `task_runner.rs:5417` region; allowlist `wire-dark-allowlist.toml:891-896`.
- **Crate / module (spec):** `crates/orchestrator/src/refusal_sla.rs`; periodic hook in `task_runner.rs`; `[refusal_sla]` in `elai_cc.toml`
- **Purpose of this file:** enough architecture to rebuild session-level refusal-rate SLA monitoring. Not a status ledger.

## 1. What it is

The **honest-refusal SLA** tracks session metric `refusal_rate = refusals / total_claims` and auto-flags when the rate is **< 5%** (model likely confabulating) or **> 50%** (checkers misconfigured **or** the task suite is too hard) (`D-plan/D-17_honest_refusal_sla.md:1-7`, `ELAI-SYSTEMS-OVERVIEW.md:238-239`). It is a structural defense against gaming Mode E as the easy route. Honest “I don’t know” is a first-class quality metric next to accuracy (real-problem §4.6).

## 2. Why it exists

Without an SLA, UniCR (D-16) can be “satisfied” by refusing everything, or never refusing and hallucinating. If this organ is missing, Mode E has no operator alarm. If it is faked (a log line with no Bubble-Up, or thresholds hardcoded in a dashboard), operators never act. UC-5’s 10–30% “healthy band” is an **unvalidated guess**; D-17 instruments 5–50% so the band can emerge empirically (`D-plan/D-17_honest_refusal_sla.md:12-20,87-92`).

## 3. How it works (runtime)

1. **Who calls it.** Periodic hook in `task_runner` / orchestration loop: every `check_interval_minutes` (config default **30**), compute `snapshot` then `alert_on_breach` (`:59-62,69-72`). Prose also says “every N minutes (60 default)” — window vs interval split in §7.
2. **Input.** `session_id`, `SsbViewer`, thresholds from TOML.
3. **`snapshot` (deterministic aggregation):**
   - Numerator: `RefusalDecision` rows in the window (D-16).
   - Denominator: `ModeSelected` / `MODE_SELECTED` rows in the window (D-10) (`:46-52`).
   - `rate = refusal_count / total_claims`.
   - Band: `Below` if rate < `healthy_min`; `Above` if rate > `healthy_max`; else `Healthy`.
   - If Below or Above → emit `RefusalSlaBreach`.
4. **`total_claims == 0` → `Healthy` by definition.** No division by zero, no alert (`:113-114`).
5. **`alert_on_breach`.** If band ≠ Healthy, emit Bubble-Up (B-13) with `AlertKind::RefusalRateOutOfBand` (`:54-57`). **Idempotent** inside a window: suppress duplicates via a **window-end key** (`:115-116`). Healthy snapshots must not alert (`:78`).
6. **The model has no input.** This organ only reads SSB. Gaming it would mean suppressing rows — that is an audit-log integrity problem (D-8/D-25), not a knob here.

## 4. Types, enums, objects

Copy from `D-plan/D-17_honest_refusal_sla.md:36-44`. Do not invent fields.

```
struct RefusalRateMonitor {
  window_size_minutes: u32,
  healthy_min: f32,
  healthy_max: f32,
  ssb_db_path: PathBuf,
}

struct RefusalRateSnapshot {
  session_id: String,
  window_start: DateTime<Utc>,
  refusal_count: u32,
  total_claims: u32,
  rate: f32,
  band: HealthBand,
}

enum HealthBand {
  Below { rate: f32, min: f32 },
  Healthy,
  Above { rate: f32, max: f32 },
}

fn snapshot(&self, session_id, ssb_viewer) -> RefusalRateSnapshot
fn alert_on_breach(&self, snapshot, alert_tx)
```

SUP-009: event **`RefusalSlaBreach`** + **`AlertKind::RefusalRateOutOfBand`** win over summary `AUTO_FLAG` (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md:3477`). Exhaustive D0-021 still mentions `AUTO_FLAG` for a <2% synthetic — treat as the **loser name**; rebuild `RefusalSlaBreach`.

## 5. Control flow / state machine

**Owner:** orchestrator periodic monitor. Dashboard D-27 **renders** the gauge; it does not own thresholds (`D-plan/D-SUMMARY_INDEX.md:189`, D27-025).

```
every check_interval_minutes:
  snapshot(session)
    total_claims==0 → Healthy (no alert)
    rate < 0.05 → Below → RefusalSlaBreach + Bubble-Up (deduped)
    rate > 0.50 → Above → RefusalSlaBreach + Bubble-Up (deduped)
    else        → Healthy → no alert
```

**Forced:** thresholds from TOML, not compiled-in (`:117`). Session-scoped, not global (`:90-92`).

## 6. Grounding, receipts, gates

Receipts: `RefusalRateSnapshot`, `RefusalSlaBreach` SSB, Bubble-Up alert. Numerator and denominator are **journal rows**, not model self-report.

**Never allowed as a confidence signal:** LLM “we refused enough”; dashboard-hardcoded 5/50; silently logging breach without Bubble-Up (`:93-94`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `healthy_min` | **0.05** (<5% = confabulating) | per session window |
| `healthy_max` | **0.50** (>50% = misconfigured/too hard) | per session window |
| UC-5 guess (not the SLA) | 10–30% | rationale only |
| `window_size_minutes` | **60** | aggregation window |
| `check_interval_minutes` | **30** | tick |
| `enabled` | true in TOML | still live-gated by env flag |
| SCHEMA_VERSION (plan) | 62 → 63 | SSB |
| unit tests | ≥6 | acceptance |
| integration | 100 ModeSelected; 2 / 20 / 60 refusals | Below / Healthy / Above |

Prose “every N minutes (60 default)” vs config `check_interval_minutes = 30` + `window_size_minutes = 60` (D003-D17-E009). Rebuild: **window 60, tick 30**.

## 8. Roles and neighbors

Consumes D-16 `RefusalDecision`, D-10 `ModeSelected`, `SsbViewer`/`SsbWriter`, B-13 Bubble-Up. Reachable from task_runner tick and D-27 `RefusalRateGauge.svelte` (`:96-111`). Parallel-safe with D-18.

**Must not fork:** a second global SLA that disagrees with the session snapshot; Svelte-hardcoded bands (D-27 must load D-17 config).

## 9. Config / flags / defaults

- **`ELAI_REFUSAL_SLA_TICK_ENABLE`** — copy this flag (`ELAI-FEATURE-MAP.md:606-607`, `wire-dark-allowlist.toml:891-896`). Allowlist type `kill_switch`, promotion_plan_id D-27.
- **L3-locked `[refusal_sla]`:**

```toml
[refusal_sla]
enabled = true
window_size_minutes = 60
healthy_min = 0.05
healthy_max = 0.50
check_interval_minutes = 30
```

(`D-plan/D-17_honest_refusal_sla.md:69-72`)

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `refusal_sla.rs` | monitor, snapshot, alert_on_breach |
| `ssb.rs` 62→63 | RefusalSlaBreach |
| `task_runner.rs` | periodic hook |
| `elai_cc.toml` | `[refusal_sla]` |
| `lib.rs` | `pub mod refusal_sla` |
| `orchestrator/tests/refusal_sla_integration.rs` | 2% / 20% / 60% |
| `workflow/reasoning/D-17-decisions.md` | three decisions |
| D-27 later | `RefusalRateGauge.svelte`, `get_refusal_rate` |

## 11. Journal / SSB events

Plan SCHEMA_VERSION **62 → 63**:

```
RefusalSlaBreach { session_id: String, rate: f64, healthy_min: f64,
                   healthy_max: f64, band: String, total_claims: u32 }
```

(`:64-67`). D-SUMMARY/D-0 also mention `REFUSAL_SLA_BREACH` (`D-plan/D-SUMMARY_INDEX.md:124` via D0-052). Loser name: `AUTO_FLAG`.

## 12. Tests and observables

| Case | Assert |
|---|---|
| rate 0.02 | Below |
| rate 0.30 | Healthy |
| rate 0.60 | Above |
| snapshot on synthetic SSB | counts/rate correct |
| alert_on_breach Healthy | no alert |
| 100 selected + 2 refusals | Below + alert |
| 100 + 60 refusals | Above + alert |
| 100 + 20 refusals | Healthy, no alert |
| total_claims=0 | Healthy, no NaN |
| duplicate breach same window | one alert |
| cloud A | 100 claims / 2 refusals (2%) → RefusalSlaBreach Below + Bubble-Up |
| cloud B | rate in [5%, 50%] → no breach |

≥6 tests. Decision doc: why 5–50 not 10–30; why session-level; why Bubble-Up (`:87-94`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- `refusal_rate = refusals / total_claims` with D-16 numerator and D-10 ModeSelected denominator.
- Bands **<5% Below** / **>50% Above** / else Healthy, thresholds from TOML.
- Zero claims → Healthy.
- Breach emits `RefusalSlaBreach` **and** Bubble-Up `RefusalRateOutOfBand`, deduped per window.
- Session-scoped.
- `ELAI_REFUSAL_SLA_TICK_ENABLE` default-OFF isolation allowed; when on, must tick.

A silent counter or a dashboard-only cousin without alerts is not this organ.

## 14. Non-goals / banned designs

- D-27 gauge rendering (later).
- Per-task-family band tuning (deferred).
- Long-term SLA trend report (deferred; D-27 adequate) (`:162-166`).
- Hardcoding 5/50 in Svelte.
- Using `AUTO_FLAG` as the SSB variant (SUP-009 loser).

## 15. Source citations

Primary: `D-plan/D-17_honest_refusal_sla.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:122-126`. Feature map: `ELAI-FEATURE-MAP.md:606-607`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:238-239`. Exhaustive: D0-021, D0-052, D003-D17-E001..E024, SUP-009. Live: `task_runner.rs:5417`, `wire-dark-allowlist.toml:891-896`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Whether denominator is all `ModeSelected` or only Mode E + refusals; “refusals / total_claims” vs counting `RefusalDecision.refused=true` only.
- Window alignment (calendar vs sliding vs session start).
- Window-end key format for idempotency.
- `enabled=true` in TOML vs env `ELAI_REFUSAL_SLA_TICK_ENABLE` AND semantics.
- LLM Honesty Survey TMLR 2025 cited in D-SUMMARY (`:126`) but not in the D-17 tasks/reads list.
- D-27 red gauge is an expect_artifact of D-17 but D-27 is out of scope — testability of “dashboard shows red” in D-17 alone.
- Schema 63 vs live schema ≫ 63.
