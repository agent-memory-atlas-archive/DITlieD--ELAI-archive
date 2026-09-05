# Mode-routing predicate `select_grounding_mode` (D-10)

- **ELAI ids:** D-10 / RPS.I / D0-014 / D10-E01..D10-E40 / D10-N01..D10-N13
- **Spec status (as designed):** WIRED (`ELAI-FEATURE-MAP.md:581-584`). D-SUMMARY still labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:80`). Live producer is also gated by `ELAI_CLAIM_ROUTER_ENABLE` (`task_runner.rs:3716-3718`) — that env gate is a later fill, not in the original D-10 plan.
- **Crate / module (spec):** `crates/orchestrator` — `src/claim.rs`, `src/grounding_mode.rs`, `src/orch_context.rs`; claim-emit hook in `src/task_runner.rs`; L3 lock in `scripts/locked_keys_registry.toml`
- **Purpose of this file:** enough architecture to rebuild the five-mode orchestrator router. Not a status ledger.

## 1. What it is

The **mode-routing predicate** is the orchestrator-owned function `select_grounding_mode(claim, ctx) -> GroundingMode`. Every novel claim is assigned exactly one of five closed modes: **A** `Constructive` (checker is ground truth), **B** `FalsifiabilityCrossFamily` (cross-family adversary, last resort), **C** `DiscoveryLabelOnly` (post-hoc label, not a runtime route), **D** `OperationalFalsification` (typed sandbox `ObservableEffect`), **E** `Refuse` (first-class “I don’t know”). The LLM has **no input** to selection (`D-plan/D-10_mode_routing_predicate.md:3-8`, `ELAI-SYSTEMS-OVERVIEW.md:232-235`, `ELAI-SPEC-OVERVIEW.md:178-181`).

## 2. Why it exists

ELAI’s trust spine refuses optimizer-grades-itself. Cohen Boxed Myopic + Everitt current-RF (real-problem §3.2) require the grading function to sit outside the agent (`D-plan/D-10_mode_routing_predicate.md:5-8,167-170`). If this organ is missing, every claim can be “verified” by the same model that produced it. If it is faked (prompt instructions, or letting the model pick A–E), the agent edits its own grade. Mode B without an alternate family is the Self-Correction Bench failure (64.5% same-model blind-spot); the spec therefore falls to Mode E, not same-family critique (`D-plan/D-10_mode_routing_predicate.md:171-173`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator, at every claim-emit site in `task_runner.rs`, after constructing a `Claim`. FEATURE-MAP names the live path `task_runner -> agent_pool::route_claim -> select_grounding_mode` (`ELAI-FEATURE-MAP.md:581-584`, `agent_pool.rs:2268`). The model does not call this function.
2. **Input.** `claim: &Claim` and `ctx: &OrchContext`. Pure typed inputs. No LLM, no I/O, no clock (`D-plan/D-10_mode_routing_predicate.md:94-96,197-200`).
3. **Predicate (deterministic, exact order — first match wins):**

```
fn select_grounding_mode(claim: &Claim, ctx: &OrchContext) -> GroundingMode {
    if claim.is_operationalizable() && ctx.checker_registry.applicable(claim) {
        return GroundingMode::Constructive;                 // A
    }
    if claim.permits_experiment() && ctx.sandbox.can_run(claim) {
        return GroundingMode::OperationalFalsification;     // D
    }
    if claim.intrinsic_signals_below_threshold(ctx.unicr_tau) {
        return GroundingMode::Refuse;                       // E
    }
    if ctx.has_alternate_family_model() {
        return GroundingMode::FalsifiabilityCrossFamily;    // B
    }
    return GroundingMode::Refuse;                           // E
}
```

(`D-plan/D-10_mode_routing_predicate.md:33-49,52-54`)

4. **Mode C is not returned.** `DiscoveryLabelOnly` is applied **after** Mode A or D produces a checker-grounded result (D-15). Hitting it in `dispatch_by_mode` is a defensive error (`D-plan/D-10_mode_routing_predicate.md:25-27,117-118,174-175`).
5. **Mode B is last-resort.** Only after A and D are unavailable **and** intrinsic signal is **not** below τ **and** an alternate family exists. Otherwise refuse (`D-plan/D-10_mode_routing_predicate.md:23-24,52-54`).
6. **Dispatch (deterministic routing, backends may be stubbed until neighbors land):**
   - A → `ctx.checker_registry.run(claim)` (D-11; stub `Inapplicable` until then)
   - D → `ctx.sandbox.run_observable_effect(claim)` (D-18; stub `Inconclusive`)
   - B → `ctx.adversary_role.probe(claim)` (D-13; stub `NoResult`)
   - E → `ClaimResult::Refused { reason }`
   - C → defensive `ClaimResult::Inapplicable` / `Err` — **not a runtime mode**
   (`D-plan/D-10_mode_routing_predicate.md:107-118`)
7. **Commit.** Emit `ModeSelected` then run dispatch then emit `ClaimDispatched` (`D-plan/D-10_mode_routing_predicate.md:129-133,197-203`).
8. **Abstain / fail-closed.** Empty context must still be able to reach Refuse. Routing everything to Mode B without checks is banned (`D-plan/D-10_mode_routing_predicate.md:197-200`).
9. **The model may generate** claim `statement` / `claim_type` proposals / intrinsic-signal raw text. It may **not** choose the mode, mutate `GroundingMode` / `ClaimType`, or call an LLM from inside `select_grounding_mode`.
10. **L3.** `select_grounding_mode` and `ClaimType` are locked-symbol entries. Agent cannot edit (`D-plan/D-10_mode_routing_predicate.md:135-148`; live: `locked_keys_registry.toml:93-107`).

## 4. Types, enums, objects

Copy from `D-plan/D-10_mode_routing_predicate.md:77-120`. Do not invent fields.

```
Claim {
  id: ClaimId,
  claim_type: ClaimType,
  statement: BoundedString<2048>,
  domain: Domain,                         // D-1 + D-7
  source_template: Option<TemplateId>,
  intrinsic_signals: Option<IntrinsicSignals>,  // D-3
}

ClaimType {  // closed, L3-locked
  Code, Math, Prose, Empirical, Definitional,
  Counterfactual, EthicalAesthetic
}

GroundingMode {  // closed, L3-locked; five variants
  Constructive,                 // A
  FalsifiabilityCrossFamily,    // B
  DiscoveryLabelOnly,           // C (label only)
  OperationalFalsification,     // D
  Refuse                        // E
}

ClaimResult {
  Verified,
  Falsified { evidence: BoundedString<2048> },
  Inconclusive { reason: String },
  Refused { reason: String },
}

fn is_operationalizable(&self) -> bool
  Code | Math | (Empirical with test_cmd) → true
  Prose | Definitional | Counterfactual | EthicalAesthetic → false

fn permits_experiment(&self) -> bool
  Empirical with sandbox-compatible test → true; else false

OrchContext extensions:
  checker_registry          // D-11 trait; stub until D-11
  sandbox                   // C-22 FeasibilityProber; D-18 fills typed effect
  unicr_tau                 // D-16; stub constant until D-16
  has_alternate_family_model() -> bool
    true iff ELAI_PPM_ENABLE OR ELAI_CLOUD_MODE (D-2 / B-37)

fn select_grounding_mode(claim, ctx) -> GroundingMode   // no LLM
fn dispatch_by_mode(claim, mode, ctx) -> ClaimResult
```

**Live cousin (do not rebuild as this organ):** implemented `GroundingMode` is `{Constructive, FalsifiabilityCrossFamily, ConformalRefusal, OperationalDeferred, ConservativeFallback}` (`grounding_mode.rs:57-72`) and `ClaimType` is `{Factual, Procedural, Normative, Speculative, Hypothetical, Computational, MetaQuery}` (`claim.rs:35-43`). That table is a different predicate. Rebuild the **plan** enum and the **plan** if-chain.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator. The model proposes a claim; the engine assigns a mode. No token-level router (`D-plan/D-10_mode_routing_predicate.md:129-133`).

**Decision table (must be unit-tested, five branches):**

| Condition | Mode | Notes |
|---|---|---|
| operationalizable **and** applicable checker | A Constructive | Wins even if Mode D is also possible |
| no checker, sandbox-compatible experiment | D OperationalFalsification | |
| no checker, no sandbox, signals below `unicr_tau` | E Refuse | |
| no checker, no sandbox, signals OK, alt-family present | B FalsifiabilityCrossFamily | |
| no checker, no sandbox, signals OK, **no** alt-family | E Refuse | never same-family B |
| `DiscoveryLabelOnly` | not selected | post-hoc D-15 label on A/D results |

(`D-plan/D-10_mode_routing_predicate.md:153-160`)

**Forced transitions:** none inside this organ. Neighbors consume the mode: D-11 runs A, D-13 runs B, D-15 labels A/D, D-16 supplies τ for E, D-18 runs D.

**`dispatch_by_mode` must emit `ClaimDispatched` regardless of mode** (`D-plan/D-10_mode_routing_predicate.md:201-202`). Ownership conflict with the task-runner “caller emits” sentence is in §16.

## 6. Grounding, receipts, gates

This organ **selects how** a claim will be grounded. It does not itself verify files, run tests, or score confidence.

**Selector hierarchy (mode precedence, not GroundingLevel):** A checker > D experiment > E low-signal refuse > B cross-family > E no-alt-family refuse.

**Never allowed as a confidence signal / mode chooser:** LLM self-rating, “the model said Mode A”, scalar 0–1 from the policy model, window fill, PPM score as a substitute for `is_operationalizable`, same-family self-critique as Mode B (`D-plan/D-10_mode_routing_predicate.md:5-8,197-200`).

**τ is not invented here.** `ctx.unicr_tau` is a D-16 stub until UniCR lands (`D-plan/D-10_mode_routing_predicate.md:100-105,273-277`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `Claim.statement` | `BoundedString<2048>` | per claim |
| `ClaimResult::Falsified.evidence` | `BoundedString<2048>` | per result |
| observable exercise | 50 synthetic claims, 10 per named mode | D-10 smoke |
| smoke accuracy | ≥45/50 (plan says “≥90% F1”) | D-10 observable |
| D-21 gate (authoritative later) | macro-F1 ≥ 0.85, per-mode F1 ≥ 0.70 | D-21, not this organ |
| unit tests | ≥12 new tests in orchestrator | D-10 acceptance |
| SCHEMA_VERSION (plan) | 55 → 56 | SSB |

No per-topic GRC budget lives here (that is 2A/4A). No token budget. Overflow of bounded strings: **UNCERTAIN** (plan names the bound, not the truncate-vs-reject rule).

## 8. Roles and neighbors

D-10 is **role-agnostic at selection**. Live code special-cases `OrchUserRole::{Adversary,Auditor,Meta}` (`grounding_mode.rs:115-128`); that is **not** in the original predicate — treat as cousin unless a later plan amends D-10.

**Consumes:** `Claim` constructed at emit sites; `IntrinsicSignals` (D-3); `Domain` (D-1/D-7); stub traits for registry (D-11), sandbox (D-18), `unicr_tau` (D-16), adversary (D-13); `LockedKeysAuditor` (D-9); `SeededClock` named in wiring contract (`D-plan/D-10_mode_routing_predicate.md:187-193`).

**Produces:** `GroundingMode`, `ClaimResult`, `ModeSelected` + `ClaimDispatched` SSB rows.

**Must not fork:** a second selector inside the agent; LLM-in-the-loop mode choice; routing-everything-to-B; treating Mode C as a dispatch target.

**Not parallel-safe with D-16** (UniCR is the formal Mode E). Parallel-safe with D-11 and D-12 (`D-plan/D-10_mode_routing_predicate.md:9-14`).

## 9. Config / flags / defaults

- Original D-10: **no env flag**. Predicate is always on, L3-locked.
- Live later fill: `ELAI_CLAIM_ROUTER_ENABLE=1` plus `claim_router_runtime: Option<…>` (`task_runner.rs:3716-3729`). Absence → dark `claim_routed` family. Rebuild the organ as **always-on selection**; an env gate is optional isolation, not a license to skip routing.
- `has_alternate_family_model()`: `ELAI_PPM_ENABLE` **OR** `ELAI_CLOUD_MODE` (`D-plan/D-10_mode_routing_predicate.md:100-104`). PPM is same-family Qwen — D-13 later forbids using it as Adversary. This function only answers “is some second runtime configured?”, not “is it a different family?” (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` D10-N07).
- Locked keys: `select_grounding_mode` in `grounding_mode.rs`; `ClaimType` in `claim.rs` (`D-plan/D-10_mode_routing_predicate.md:135-148`; live `locked_keys_registry.toml:93-107`).
- Decision doc required: `workflow/reasoning/D-10-decisions.md` (three decisions: orchestrator owns routing; E not B as fallback; C is post-hoc).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/claim.rs` | `Claim`, `ClaimId`, `ClaimType`, `is_operationalizable`, `permits_experiment` |
| `.elai_cc/crates/orchestrator/src/grounding_mode.rs` | `GroundingMode`, `select_grounding_mode`, `dispatch_by_mode`, `ClaimResult` |
| `.elai_cc/crates/orchestrator/src/orch_context.rs` | checker_registry / sandbox / unicr_tau / `has_alternate_family_model` |
| `task_runner.rs` claim-emit sites | ModeSelected → dispatch → ClaimDispatched |
| `ssb.rs` SCHEMA 55→56 | `ModeSelected`, `ClaimDispatched` |
| `scripts/locked_keys_registry.toml` | locked symbols |
| `crates/contracts/tests/mode_routing_contract.rs` | schema 56 + closed-enum serde |
| `workflow/reasoning/D-10-decisions.md` | three decisions |

**Live neighbors (not the rebuild names):** `select_grounding_mode` at `grounding_mode.rs:110`; “sound-checker-first” constructive branch at `grounding_mode.rs:157-166` (`ELAI-FEATURE-MAP.md:581-584`); `agent_pool::route_claim` at `agent_pool.rs:2268`; pub use `lib.rs:2301`; contract `crates/contracts/tests/d10_mode_routing_contract.rs`.

## 11. Journal / SSB events

Plan SCHEMA_VERSION **55 → 56** (`D-plan/D-10_mode_routing_predicate.md:122-127`):

```
ModeSelected { claim_id: Uuid, mode: String, reason: String, audit_id: Uuid }
ClaimDispatched { claim_id: Uuid, mode: String, result: String, evidence_snippet: Option<String> }
```

Round-trip tests required. D-SUMMARY also names `MODE_SELECTED` (`D-plan/D-SUMMARY_INDEX.md:81`). Live schema is already far past 56 (`ssb/schema.rs` comments at 172+). Rebuild: **emit these two payload shapes**; do not rewind a live schema number. Reason-string contents and `evidence_snippet` truncation are unspecified (§16).

## 12. Tests and observables

Named requirements (`D-plan/D-10_mode_routing_predicate.md:150-165,203-255`):

| Id | Assert |
|---|---|
| operationalizable table | Code+test_cmd true; Math+sympy_expr true; Empirical without sandbox-compat false; Prose/Definitional/Counterfactual/EthicalAesthetic false |
| A beats D | checker available → Constructive even if experiment possible |
| D branch | no checker, sandbox-compat → OperationalFalsification |
| E low-signal | no checker, no sandbox, below τ → Refuse |
| B branch | no checker, no sandbox, signals OK, alt-family → FalsifiabilityCrossFamily |
| E no-alt | same but no alt-family → Refuse |
| no LLM | `rg` of anthropic/client.send/sidecar/complete in `grounding_mode.rs` = 0 matches |
| contract | SCHEMA_VERSION_PUBLIC == 56 (plan); ModeSelected + ClaimDispatched round-trip; GroundingMode closed serde |
| observable | 50 claims, 10/mode, 50 ModeSelected rows, ≥45/50 match hand labels |
| cloud A | 10 mixed synthetic claims via `/task`; distribution vs hand labels |
| L3 | edit to `select_grounding_mode` without override is blocked |

≥12 new orchestrator tests. D-21 later gates **macro-F1 ≥ 0.85** and is the quantitative authority (`D-plan/D-10_mode_routing_predicate.md:281-284`).

Adversarial: Refuse-on-empty-context reachable; all five **named** mode branches tested; `ClaimDispatched` emitted on every mode including Refuse.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Closed `GroundingMode` with exactly `{Constructive, FalsifiabilityCrossFamily, DiscoveryLabelOnly, OperationalFalsification, Refuse}`.
- Closed `ClaimType` with exactly `{Code, Math, Prose, Empirical, Definitional, Counterfactual, EthicalAesthetic}`.
- `select_grounding_mode` is pure orchestrator logic: **no LLM call**, L3-locked, implements the if-chain in §3 in that order.
- Constructive outranks OperationalFalsification. Mode B is never selected when signals are below τ. No alt-family → Refuse, not B.
- `DiscoveryLabelOnly` is never a successful runtime dispatch target.
- `dispatch_by_mode` covers all five variants (stubs allowed for D-11/D-13/D-16/D-18).
- `ModeSelected` then `ClaimDispatched` at every claim-emit.
- `is_operationalizable` / `permits_experiment` tables as in §4.
- Empty-context Refuse path exists and is tested.
- LLM has no input to selection.

A prompt “please pick a grounding mode”, an agent-side router, or the live `{ConformalRefusal, OperationalDeferred, ConservativeFallback}` enum is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- D-11 checker bodies, D-13 adversary, D-16 UniCR, D-18 ObservableEffect, D-21 F1 benchmark (`D-plan/D-10_mode_routing_predicate.md:273-280`).
- LLM inside `select_grounding_mode`.
- Same-model Mode B.
- Mode C as a runtime grade.
- Routing-everything-to-FalsifiabilityCrossFamily.
- Letting PPM (same-family Qwen) count as a cross-family adversary (that ban is D-13; D-10’s `has_alternate_family_model` is coarser — do not “fix” it by silently tightening without D-13).

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:76-84`. Overview: `ELAI-SPEC-OVERVIEW.md:171-183`. Feature map: `ELAI-FEATURE-MAP.md:581-584`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:230-239`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` D0-014, D10-E01..E40, D10-N01..N13. Live code (implementation truth, not spec): `crates/orchestrator/src/grounding_mode.rs`, `claim.rs`, `agent_pool.rs:2268`, `task_runner.rs:3716-3729`, `scripts/locked_keys_registry.toml:93-107`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `test_cmd`, `sympy_expr`, and sandbox-compat are used by `is_operationalizable` / tests but are **not** fields of declared `Claim` (D10-N02).
- `intrinsic_signals: Option<_>` — behavior when `None` is unspecified (D10-N03).
- `dispatch_by_mode` DiscoveryLabelOnly returns `ClaimResult::Inapplicable` but `ClaimResult` has no `Inapplicable`; also described as `Err` while the fn returns bare `ClaimResult` (D10-N04, D10-N05).
- Who emits `ClaimDispatched` — task_runner vs `dispatch_by_mode` (D10-N06). Duplicate rows possible.
- `ELAI_PPM_ENABLE` / `ELAI_CLOUD_MODE` do not prove a **different family** or a healthy runtime (D10-N07).
- 45/50 accuracy is not F1; D-10 says ≥90% F1, D-21 gates 0.85 macro-F1 (D10-N08, D10-N09).
- Predicate never returns `DiscoveryLabelOnly`, yet the 50-claim exercise wants 10 Mode-C cases (D10-N10).
- Reason strings, result serialization, `evidence_snippet` truncation (D10-N11).
- Stub returns `Inapplicable` / `NoResult` are not `ClaimResult` variants; no re-route when a selected backend is down (D10-N12).
- “Every claim-emit site” is not inventoried (D10-N13).
- Live enum/ClaimType/role-preempts vs plan if-chain — which SCHEMA_VERSION to stamp on a tree already past 56.
- BoundedString overflow policy.
