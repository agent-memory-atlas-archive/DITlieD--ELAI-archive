# Observable effect typed (D-18)

- **ELAI ids:** D-18 / RPS Phase 4F extension / D0-022 / D0-053 / D004-D18-001..044
- **Spec status (as designed):** DARK(`ELAI_OBSERVABLE_EFFECT_*` ×3) (`ELAI-FEATURE-MAP.md:608-609`). The three flags are `ELAI_OBSERVABLE_EFFECT_ENABLE`, `ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE`, `ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE` (`wire-dark-allowlist.toml:900-925`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:128`).
- **Crate / module (spec):** `crates/orchestrator/src/observable_effect.rs`; C-22 `FeasibilityProber` return-type extension; Mode D dispatch in `grounding_mode.rs`
- **Purpose of this file:** enough architecture to rebuild Mode D’s typed multi-effect sandbox predicate. Not a status ledger.

## 1. What it is

**ObservableEffect** is the typed Mode D upgrade of C-22 `FeasibilityProber`: each claim carries an `EffectSet` of **≥2 orthogonal** `ObservableEffect`s, every observation is labeled **Sandbox(tier)** vs **Production**, and sandbox success **never** auto-promotes to a production claim (`D-plan/D-18_observable_effect_typed.md:1-8,21-27,88-91`, `ELAI-SYSTEMS-OVERVIEW.md:234-239`). This realizes Voyager skill-with-effect, Eureka reward-as-falsifier, and Coscientist confound documentation as load-bearing primitives (real-problem §4.4).

## 2. Why it exists

Three named failure modes (`:21-27`):

1. **Goodhart on test design** — a single effect is a proxy the optimizer will climb. Defense: ≥2 orthogonal effects.
2. **Simulator-reality gap** — Minecraft ≠ field. Defense: `SandboxObservation ≠ ProductionObservation`; no production grounding from sandbox alone.
3. **Confounded experiment** — log environment; `confound_flags` surface known risks.

If this organ is missing, Mode D is “we ran something.” If it is faked (one effect, or sandbox billed as production), it is exactly those failure modes.

## 3. How it works (runtime)

1. **Who calls it.** D-10 `dispatch_by_mode` `OperationalFalsification` → sandbox `run_observable_effect` (`D-plan/D-10_mode_routing_predicate.md:107-113`; D-18 `:160-163`). Also `DefaultFeasibilityProber.probe` (C-22) now returns `Vec<ObservationResult>` (`:93-97`).
2. **Author-time.** Probe construction **must** include `EffectSet` with `effects.len() >= target_orthogonality` (default **2**) and `validate_orthogonality` (`:96-97`).
3. **`validate_orthogonality` (deterministic):**
   - Reject `len < target_orthogonality`.
   - Effects must differ on at least one of `(trigger.kind, expected_observation variant)`. Same-kind effects **count as one** for orthogonality (`:72-77`).
4. **`check_goodhart_risk`.** If `len < target_orthogonality` → `Some(GoodhartRiskFlag)` with reason `"single-effect probe vulnerable to Goodhart"` (`:99-102`). Integration/cloud **require** a 1-effect probe to **run and emit** that flag (`:128,210-211`) while author-time validation **forbids** constructing it (D004-D18-036) — conflict in §16.
5. **Run effects.** D-11 checkers run individual effects (`:157-158`). Each result is an `ObservationResult` with `ObservationContext::Sandbox(tier)` or `Production`, outcome, timing, `EnvironmentSnapshot`.
6. **No auto-promotion.** `auto_promote_sandbox_to_production = false` (L3). Sandbox pass may emit `ProductionObservationProposal { operator_required: true }` only as a **proposal**; operator ratification is required. Production-claim CLI is **deferred**; v1 is sandbox-only (`:88-91,236-239`).
7. **Commit.** `ObservableEffectDispatched` + per-effect `SandboxObservation`; `GoodhartRiskFlag` on insufficient effects; persist `workflow/sessions/{id}/observations/{claim_id}.toml` with snapshot + confounds (`:169-175`).
8. **The model may generate** trigger payloads / expected observations. It may **not** drop to one effect, relabel sandbox as production, or flip auto-promote.

## 4. Types, enums, objects

Copy from `D-plan/D-18_observable_effect_typed.md:54-91`. Do not invent fields.

```
struct ObservableEffect {
  trigger: TriggerSpec,
  expected_observation: TypedObservation,
  success_predicate: Predicate,
  bounded_runtime: Duration,
  sandbox_tier: SandboxTier,
  confound_flags: Vec<ConfoundFlag>,
}

struct TriggerSpec { kind: TriggerKind, payload: BoundedString<4096> }
enum TriggerKind { Subprocess, HttpRequest, FileWrite, SandboxedRun }

enum TypedObservation {
  StringMatch(BoundedString<2048>),
  FileContent { path: BoundedString<512>, expected_blake3: Option<Blake3Hash> },
  ExitCode(i32),
  MetricValue { metric_name: String, range: (f64, f64) },
}

enum SandboxTier { TierA, TierB, TierC }   // B-18 / C-11

enum ConfoundFlag { ColdCache, ColdNetwork, GpuRecentlyIdle, ConcurrentTasks, Other(String) }

struct EffectSet {
  effects: Vec<ObservableEffect>,
  claim_id: ClaimId,
  target_orthogonality: u8,   // default ≥2
}

fn validate_orthogonality(set: &EffectSet) -> Result<(), OrthogonalityError>
fn check_goodhart_risk(set: &EffectSet) -> Option<GoodhartRiskFlag>

enum ObservationContext { Sandbox(SandboxTier), Production }

struct ObservationResult {
  effect: ObservableEffect,
  context: ObservationContext,
  outcome: ObservationOutcome,
  timing_ms: u64,
  environment_snapshot: EnvironmentSnapshot,
}

enum ObservationOutcome {
  Confirmed,
  Refuted { evidence: BoundedString<2048> },
  Inconclusive { reason: String },
}

struct EnvironmentSnapshot {
  hostname: String,
  hardware_class: String,
  active_confounds: Vec<ConfoundFlag>,
}
```

`Predicate` is named on `ObservableEffect.success_predicate` and never defined — **UNCERTAIN** (§16).

## 5. Control flow / state machine

**Owner:** orchestrator Mode D + FeasibilityProber. Operator owns production promotion.

```
Mode D selected
  → EffectSet (≥2 orthogonal) at author time
  → validate_orthogonality
       fail → OrthogonalityError and/or GoodhartRiskFlag (§16)
  → run each effect in declared SandboxTier (typically C)
  → ObservationContext::Sandbox(tier) on every sandbox result
  → persist observations TOML
  → ProductionObservationProposal only as operator-gated proposal
  → NEVER auto Production claim
```

**Forced:** sandbox label persists through `ObservationResult` (`:164-168`). Same-kind pair is not orthogonal.

## 6. Grounding, receipts, gates

Receipts: typed expected vs actual `ObservationOutcome`, environment snapshot, confound flags, SSB, observations TOML. Checker JSON from D-11 may implement an individual effect; it does not erase the sandbox/production label.

**Never allowed as a confidence signal:** “it passed in the sandbox so production is grounded”; single-effect pass; LLM-written success_predicate without running it.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `min_effects_per_claim` | **2** | per claim |
| `target_orthogonality` | **2** | EffectSet / TOML |
| trigger payload | `BoundedString<4096>` | per effect |
| StringMatch / Refuted evidence | `BoundedString<2048>` | |
| FileContent path | `BoundedString<512>` | |
| `auto_promote_sandbox_to_production` | **false** | L3 |
| SCHEMA_VERSION (plan) | 63 → 64 | SSB |
| unit tests | ≥10 | acceptance |
| observable | 1 dispatch + 2 SandboxObservation; 0 Goodhart; 0 ProductionProposal until operator | happy path |

`bounded_runtime` is data only — no runner/timeout behavior specified (D004-D18-038).

## 8. Roles and neighbors

Consumes C-22 FeasibilityProber, D-11 CheckerRegistry, B-18/C-11 Tier C, D-10 Mode D. Parallel-safe with D-17. D-11 checkers are **effect runners**, not a substitute for orthogonality.

**Must not fork:** C-22 single-result probe as Mode D; auto-promote; reliability_guard as sandbox (D-11 ban still applies).

## 9. Config / flags / defaults

Copy **all three** flags:

| Flag | Role (allowlist) |
|---|---|
| `ELAI_OBSERVABLE_EFFECT_ENABLE` | emit gate (`runtime_config` ~2428; `task_runner.rs:3846`) |
| `ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE` | real-effect executor (`observable_effect.rs` ~549) |
| `ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE` | legacy emit path (`task_runner.rs` ~11981) |

(`wire-dark-allowlist.toml:900-925`, `ELAI-FEATURE-MAP.md:608-609`)

- **L3-locked `[observable_effect]`:**

```toml
[observable_effect]
min_effects_per_claim = 2
target_orthogonality = 2
auto_promote_sandbox_to_production = false
```

(`D-plan/D-18_observable_effect_typed.md:114-117`). The false auto-promote default is safety-critical.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `observable_effect.rs` | types, validate, goodhart |
| `feasibility/src/lib.rs` | `probe` → `Vec<ObservationResult>` |
| `grounding_mode.rs` | Mode D runner |
| `ssb.rs` 63→64 | four variants |
| `elai_cc.toml` | `[observable_effect]` |
| `lib.rs` | `pub mod observable_effect` |
| `orchestrator/tests/observable_effect_integration.rs` | 1-effect / 2-orthogonal / no auto-promote |
| `workflow/sessions/{id}/observations/{claim_id}.toml` | artifact |
| papers | voyager-2305.16291, eureka-2310.12931, coscientist-2304.05332 |

## 11. Journal / SSB events

Plan SCHEMA_VERSION **63 → 64**:

```
ObservableEffectDispatched { claim_id, effect_count: u32, orthogonality_target: u8, context: String }
GoodhartRiskFlag { claim_id, effect_count: u32, reason: String }
SandboxObservation { claim_id, effect_id: u32, outcome: String, tier: String }
ProductionObservationProposal { claim_id, sandbox_evidence: String, operator_required: bool }
```

(`:104-112`). D-SUMMARY/D-0 highlight `GOODHART_RISK_FLAG` (`D-plan/D-SUMMARY_INDEX.md:132`, D0-053). `effect_id` appears in SSB but not on `ObservableEffect` (D004-D18-043).

## 12. Tests and observables

| Case | Assert |
|---|---|
| 1 effect | OrthogonalityError |
| 2 same-kind | OrthogonalityError |
| 2 different-kind | Ok |
| insufficient effects | `check_goodhart_risk` Some |
| Sandbox context | preserved on ObservationResult |
| integration 1 effect | GoodhartRiskFlag row |
| integration 2 orthogonal | Dispatched + 2 SandboxObservation |
| sandbox success | **no** production claim (auto_promote=false) |
| cloud A | 1 effect → GoodhartRiskFlag |
| cloud B | 2 orthogonal → dispatch + 2 sandbox rows; labels preserved |
| cloud C | sandbox pass → no automatic production claim |

≥10 tests. Three paper interfaces. Decision doc: ≥2 anti-Goodhart; explicit sandbox/production; no auto-promote (`:134-142`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Typed `ObservableEffect` with trigger, expected observation, predicate, bounded_runtime, sandbox_tier, confound_flags.
- **≥2** effects per claim; orthogonality = differ on trigger kind **or** observation variant; same-kind counts as one.
- Every sandbox result labeled `ObservationContext::Sandbox(tier)`; never claim production from sandbox alone.
- `auto_promote_sandbox_to_production` defaults **false** and is L3-locked.
- Single-effect path emits `GoodhartRiskFlag` (however author-time vs runtime is resolved — see §16; both behaviors are specified).
- C-22 `probe` returns `Vec<ObservationResult>`.
- All three `ELAI_OBSERVABLE_EFFECT_*` flags exist; default-OFF isolation allowed.
- Observations TOML persisted with environment snapshot + confounds.

A single untyped “we ran the sandbox” cousin is not this organ.

## 14. Non-goals / banned designs

- Production-claim CLI (deferred; sandbox-only v1).
- Per-domain effect template library (deferred).
- Automated confound detection (deferred; manual flags) (`:236-241`).
- Auto-promotion.
- Treating sandbox Confirmed as production grounding.
- Averaging a single Goodhart-able effect into a pass (ties to D-11 demotion spirit, not a D-18 numeric cap).

## 15. Source citations

Primary: `D-plan/D-18_observable_effect_typed.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:128-132`. Feature map: `ELAI-FEATURE-MAP.md:608-609`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:234-239`. Exhaustive: D0-022, D0-053, D004-D18-001..044. Live flags: `wire-dark-allowlist.toml:900-925`, `task_runner.rs:3846`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Author-time reject of 1-effect vs runtime/cloud that must emit GoodhartRiskFlag (D004-D18-036).
- For target>2, how many independent groups satisfy “same-kind count as one” (D004-D18-037).
- `bounded_runtime` unused by any runner (D004-D18-038).
- HttpRequest / FileWrite allowlists, credentials, production auth (D004-D18-039).
- “Log all environmental variables” vs snapshot of hostname/hardware/confounds only (D004-D18-040).
- Operator ratification API / identity (D004-D18-041). CLI deferred.
- Precedence among `min_effects_per_claim`, `target_orthogonality`, `EffectSet.target_orthogonality` (D004-D18-042).
- `effect_id` identity; observation TOML atomicity/retention (D004-D18-043).
- Adapter from TriggerSpec/TypedObservation/Predicate → D-11 Claim/Candidate (D004-D18-044).
- `Predicate` type undefined.
- MetricValue range inclusive/exclusive.
- Three live flags vs one plan section — when ENABLE is on but EXECUTOR is off.
- Schema 64 vs live schema ≫ 64.
