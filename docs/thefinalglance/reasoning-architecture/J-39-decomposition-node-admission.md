# Decomposition node-admission gate (J-39)

- **ELAI ids:** J-39 / WAVE DD-1 / NodeAdmissionGate / NodeAdmissionEventKind
- **Spec status (as designed):** PLANNED BUILD (upgrade of 4E sizer). Live cousin: `node_admission.rs` T1 observe-only ceiling + `derive_cone_token_floor` + `NodeAdmissionEvent` contract — **not** the full EXP-1-gated cone-vs-ceiling disposer (`J-plan/J-39_decomposition_node_admission_gate.md:1-4,47`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:175-179`).
- **Crate / module (spec):** `crates/orchestrator/src/node_admission.rs` (named live); wraps `phase_sizing_splitter.rs::DefaultPhaseSizingSplitter`; consumes B-29 `compute_impact_radius`; escalates via I-7.
- **Purpose of this file:** enough architecture to rebuild bound-and-escalate node sizing. Not a status ledger.

## 1. What it is

The **decomposition node-admission gate** is a deterministic harness check that **disposes** proposed decomposition-node size. The model **proposes** a decomposition; the gate measures each node’s **INPUT lower-bound** (B-29 transitive import/call cone, tokenized and summed) against a **measured per-(model, lane) ceiling** and returns `Admitted` | `SplitForced` | `EscalatedToCloud`. Pre-execution **TOTAL** token cost is unpredictable (Pearson r<0.15, 30× same-task variance). Pre-execution **INPUT** is exactly countable. This is **bound-and-escalate, never predict-precisely**. It upgrades 4E `DefaultPhaseSizingSplitter`; it does not replace the proposal scorer (`J-plan/J-39_decomposition_node_admission_gate.md:4,21-22`, `J-plan/J-SUMMARY_INDEX.md:1062-1078`).

Same DD-wave law as J-40/J-41/J-42: push sizing out of model discretion; leave the model generation/selection (`J-plan/J-SUMMARY_INDEX.md:57-64`).

## 2. Why it exists

Today the model decides babystep size and count. That decision is unreliable, and a precise total-cost predictor is **excluded** (r<0.15) (`J-plan/J-39_decomposition_node_admission_gate.md:4,11`). LongCodeBench: coding accuracy 29%→3% from 32K→256K, non-monotonic — ceilings must be **measured per model**, not inferred from short context (`J-plan/J-39_decomposition_node_admission_gate.md:35`). If this organ is missing, oversized nodes execute on the local ~25–40K lane and blow the window. If it is faked (LLM “this will fit”, or a total-cost regressor), the r<0.15 result is ignored.

Honest residue (SWEnergy): sizing makes a small local model **FIT** the context, not **CAPABLE** of the task (4B ~0% agentic resolution). Local lane = bounded leaves; hard leaves escalate. The gate is **necessary, not sufficient** (`J-plan/J-39_decomposition_node_admission_gate.md:22,1078`).

EXP-1 (observe-only) decides authority: cone vs actual spend on 50 SWE-bench Verified. r>0.5 → real admission controller; r<0.2 → **floor-only** (still blocks provably-oversized nodes). **No default-ON until FP/over-split rate bounded on ≥3 real runs** (`J-plan/J-39_decomposition_node_admission_gate.md:7,47`).

## 3. How it works (runtime)

1. **Who calls it.** 4E phase-sizing splitter call site on the pipeline-factory L2 decomposition path; CLI decompose/plan entrypoint (`J-plan/J-39_decomposition_node_admission_gate.md:27`). Live seam: `DefaultPhaseSizingSplitter::estimate_complexity` already calls `admission_ceiling_threshold` (`phase_sizing_splitter.rs:199-207`).
2. **Proposal.** Model (or existing 4E heuristic) proposes a node: entry files + intent. `estimate_complexity` / `estimate_tokens` stay as the **proposal scorer** (`J-plan/J-39_decomposition_node_admission_gate.md:21`, `phase_sizing_splitter.rs:150-185,200-207`).
3. **Cone floor (deterministic, no LLM).** `derive_cone_token_floor`: from entry files, B-29 transitive import/call cone, tokenize each file, **sum = exact input lower-bound** (`J-plan/J-39_decomposition_node_admission_gate.md:8`). Live: `compute_impact_radius` + `estimate_tokens_cached` (`node_admission.rs:86-116`). Plan stub_ban: **missing cone data = FAIL-closed**, not pass (`J-plan/J-39_decomposition_node_admission_gate.md:28`). Live cousin returns 0 on empty files and **swallows** traversal/read errors (`node_admission.rs:96-115`) — that silent 0 is **not** the rebuild organ.
4. **Ceiling.** Prefer J-34 per-(model, lane) `fill_degradation_curve` / `calibrated_trim_fraction`. **Fallback** (not blocked on J-34): measured baseline **3B ~40K / 7B ~25K** (`metrics/baseline_findings_and_actions.md:150-151,216` as cited) (`J-plan/J-39_decomposition_node_admission_gate.md:9,18`). Live `ModelCapabilityProfile` **now has** `calibrated_trim_fraction: Option<f64>` (`context_budget.rs:299-340`) — plan said it did not (`J-plan/J-39_decomposition_node_admission_gate.md:17`); consume J-34 when `Some`, else baseline. Also compare against `ELAI_LOCAL_MAX_CONTEXT` / `effective_context_window` (`J-plan/J-39_decomposition_node_admission_gate.md:15`). Live T1: `(effective_context_window(Local) * budget_ratio)` else `DEFAULT_TOKEN_BUDGET=4096` (`node_admission.rs:23-36`, `phase_sizing_splitter.rs:52`). `budget_ratio` default is 4E `SPLIT_RATIO=0.6` (`phase_sizing_splitter.rs:13,162`). Plan name `admit_fraction` is **not** proven equal to 0.6 (§16).
5. **Admit (deterministic).** Admit to **LOCAL** iff `lower-bound <= admit_fraction * measured_ceiling`. Else:
   - **SplitForced** — re-split at a **graph community** boundary (Louvain / classic partitioning; textbook) (`J-plan/J-39_decomposition_node_admission_gate.md:9,37`).
   - **EscalatedToCloud** — I-7 cascading escalation (`J-plan/J-39_decomposition_node_admission_gate.md:9,21`).
6. **Cross-cutting escape (mandatory).** Atomic intent with a huge cone (e.g. “add audit logging to all mutations”): gate sizes the scope **SUPERSET** (files that **could** be touched); LLM still selects the change **SUBSET**. If the node carries an **atomic-intent marker** and a huge cone, **do not** force a pointless split — route to cloud **or** flag for human; never deadlock on the cone (`J-plan/J-39_decomposition_node_admission_gate.md:10`).
7. **OUTPUT-token cap = circuit breaker, never a precise gate.** Output is fat-tailed (P95/P50>5x), uncontrollable pre-exec. Wire as a breaker in the 3C/4G supervision style (`J-plan/J-39_decomposition_node_admission_gate.md:9,19`).
8. **Depth cap.** Existing `MAX_SPLIT_DEPTH = 2`. A node that fails admission **and** cannot split **escalates**, never infinite-split (SeqCV) (`J-plan/J-39_decomposition_node_admission_gate.md:48-49`, `phase_sizing_splitter.rs:15,147-148`).
9. **EXP-1 first.** Observe-only correlation; no default-ON until it passes and FP bounded (`J-plan/J-39_decomposition_node_admission_gate.md:7`).
10. **Commit.** One `NodeAdmissionEvent` per decomposed node `{decision, cone_token_floor, lane_ceiling, lane}` (`J-plan/J-39_decomposition_node_admission_gate.md:29`).
11. **Fail-closed.** No unwrap in lib; no placeholder ceiling constant; missing cone ≠ silent admit (`J-plan/J-39_decomposition_node_admission_gate.md:28`).

The model may propose nodes, mark atomic intent, and select the change subset. It may **not** set the ceiling, override a SplitForced/Escalated decision, or predict output tokens as the admit signal.

## 4. Types, enums, objects

Copy from `J-plan/J-39_decomposition_node_admission_gate.md:25-30`. Do not invent fields.

```
NodeAdmissionGate
fn derive_cone_token_floor(entry_files, ...) -> cone_token_floor   // INPUT lower-bound

NodeAdmissionEventKind { Admitted, SplitForced, EscalatedToCloud }  // Pattern-2 3-arm

NodeAdmissionEvent {
  decision: NodeAdmissionEventKind,   // plan observable names "decision"
  cone_token_floor,
  lane_ceiling,
  lane,
}
```

**Live payload (cousin — extra fields, different names):** `NodeAdmissionEventPayload { kind, step_id, token_estimate, ceiling_threshold, details_json, ts_ms }` (`node_admission.rs:53-59`). Rebuild the **plan** observable `{decision, cone_token_floor, lane_ceiling, lane}`; do not treat `token_estimate` of the 4E summary heuristic as the cone floor.

**Consumed:**

```
DefaultPhaseSizingSplitter { max_sub_tasks, max_depth, budget_ratio }
  DEFAULT_TOKEN_BUDGET = 4096
  SPLIT_RATIO = 0.6
  MAX_SPLIT_DEPTH = 2
fn estimate_tokens / estimate_complexity / should_split     // 4E proposal
ModelCapabilityProfile { ..., calibrated_trim_fraction: Option<f64>, effective_ceiling, ... }
compute_impact_radius(cache, entry, cone_depth)             // B-29 live name
EscalationCascade                                           // I-7
```

**Excluded designs (named, do not build):** pre-execution OUTPUT-token prediction; ACONIC CSP-treewidth decomposition (`J-plan/J-39_decomposition_node_admission_gate.md:11,36`).

**Do not cite as load-bearing:** claimed arXiv:2606.00953 (no canonical page at authoring) (`J-plan/J-39_decomposition_node_admission_gate.md:38`).

## 5. Control flow / state machine

**Owner:** orchestrator admission gate between `estimate_complexity` and `should_split` (`J-plan/J-39_decomposition_node_admission_gate.md:21`).

```
model proposes node {entry files, intent, optional atomic-intent marker}
4E estimate_complexity = proposal scorer (unchanged role)
     │
     ▼
floor = derive_cone_token_floor(B-29 cone, tokenize, sum)
        missing cone → FAIL-closed (re-grep / reject), not admit
ceiling = J-34 curve if present else baseline 3B~40K / 7B~25K
     │
     ├─ atomic-intent marker AND huge cone
     │      → EscalatedToCloud OR human flag   // never pointless split
     ├─ floor <= admit_fraction * ceiling
     │      → Admitted (LOCAL)
     ├─ over ceiling AND split legal (depth < MAX_SPLIT_DEPTH)
     │      → SplitForced at community boundary
     └─ over ceiling AND cannot split
            → EscalatedToCloud (I-7)           // never loop
emit NodeAdmissionEvent
OUTPUT breaker may trip later; it does not decide this admit
```

**Forced:** EXP-1 observe-only before gating authority. Depth 2 cap. Escalate on non-decomposable over-ceiling.

## 6. Grounding, receipts, gates

This organ grounds **size**, not claim truth. The cone sum is a **receipt** (files on disk + graph). Model “this is small enough” is not a receipt.

**Never allowed as an admit signal:** total-cost predictor; LLM self-size; output-token forecast; window-as-capability (SWEnergy: FIT ≠ CAPABLE).

Selector hierarchy for **answers** remains receipt > vote (J-86) > LLM self-eval banned. J-39 must not introduce “the sizer is sure.”

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Fallback local ceiling | 3B ~40K / 7B ~25K (measured 12GB card, 100% pass on that bench) | per lane (`J-plan/J-39_decomposition_node_admission_gate.md:18`) |
| 4E `DEFAULT_TOKEN_BUDGET` | 4096 | live fallback when window=0 (`phase_sizing_splitter.rs:52`) — **not** the measured 25K/40K |
| `SPLIT_RATIO` | 0.6 | 4E proposal threshold (`phase_sizing_splitter.rs:13`) |
| `MAX_SPLIT_DEPTH` | 2 | per tree (`phase_sizing_splitter.rs:15`) |
| `admit_fraction` | named, numeric **UNCERTAIN** | vs ceiling |
| EXP-1 | 50 SWE-bench Verified; r>0.5 vs r<0.2 | observe-only authority |
| Default-ON | **banned** until EXP-1 + ≥3-run FP bound | GD |
| SSB | +NodeAdmissionEvent; snapshot 268/577 at plan authoring | W-9 |
| Output P95/P50 | >5× fat tail | breaker, not gate |

**Do not confuse with:** 2A GRC 5/800; J-84 effort tier; 3E slots; J-42 within-step reset.

## 8. Roles and neighbors

Role-agnostic sizing. Local leaves vs cloud escalation.

**Consumes:** 4E splitter; B-29 cone; J-34 curve (optional); I-7; measured baselines.

**Produces:** admit/split/escalate decision + `NodeAdmissionEvent`.

**Must not fork:** a second splitter that still uses only `summary.len()/4` as the **gate**.

**Next in DD wave:** J-40 (research compression) (`J-plan/J-39_decomposition_node_admission_gate.md:52-53`).

## 9. Config / flags / defaults

- Preferred: default-ON **observe-only emit**; wire-dark-allowlist row **only if** gated (`J-plan/J-39_decomposition_node_admission_gate.md:30`). Conflicts with “no default-ON until EXP-1” in open questions (`:47`) — see §16.
- Fallback ceilings must be the **measured** 40K/25K, not a placeholder 4096, when claiming this organ (`stub_ban` no placeholder ceiling).
- Frontend: not specified beyond SSB row; W-4 **UNCERTAIN**.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `NodeAdmissionGate` / `derive_cone_token_floor` / `NodeAdmissionEvent` | plan produces_symbols |
| `phase_sizing_splitter.rs:150` `DefaultPhaseSizingSplitter` | wrap, inject before `should_split` |
| `estimate_complexity` `:200` / `estimate_tokens` `:173` | proposal scorer |
| `context_budget.rs:299` `ModelCapabilityProfile` | ceiling consumer |
| `context_policy.rs:70` `CLOUD_TRIM_FRACTION=0.85` | cited neighbor |
| `elai_knowledge::symbol_graph::traversal::compute_impact_radius` | live B-29 (`node_admission.rs:17-19`) |
| `node_admission.rs` | live T1 cousin |
| `j39_node_admission_event_contract.rs` | live T2 emit contract |
| `metrics/baseline_findings_and_actions.md:150-151,216` | fallback ceilings |

Plan W-7: original grounding pass did **not** confirm B-29 file:line (`J-plan/J-39_decomposition_node_admission_gate.md:16,50`). Live consumer is `compute_impact_radius`.

## 11. Journal / SSB events

Plan (`J-plan/J-39_decomposition_node_admission_gate.md:29-30`):

```
NodeAdmissionEvent {     // Pattern-2, 3-arm Kind
  kind: Admitted | SplitForced | EscalatedToCloud
  cone_token_floor, lane_ceiling, lane
}
```

Live already emits Kind in payload (`node_admission.rs:53-59`; contract `j39_node_admission_event_contract.rs`). Rebuild: **one row per decomposed node** with the plan fields. Do not rewind schema to 268/577.

Live emit is best-effort (“missed SSB never a correctness failure”, `node_admission.rs:40-41`). Plan observable is the row — whether miss is FAIL is **UNCERTAIN** vs that cousin.

## 12. Tests and observables

Plan observable: one `NodeAdmissionEvent` per node with `{decision, cone_token_floor, lane_ceiling, lane}` (`J-plan/J-39_decomposition_node_admission_gate.md:29`).

Implied cases from the spec (not a numbered T-table in the plan):

| Case | Assert |
|---|---|
| EXP-1 | 50-task correlation recorded; authority follows r cut |
| cone ≤ ceiling | `Admitted` |
| cone > ceiling, split legal | `SplitForced` at community boundary |
| cone > ceiling, depth maxed | `EscalatedToCloud`, no loop |
| atomic-intent + huge cone | not pointless split |
| missing cone | FAIL-closed, not admit |
| output fat tail | breaker, not admit flip |

Live contract today tests **emit** of Admitted/SplitForced kinds (`j39_node_admission_event_contract.rs:57-105`), not cone-floor vs ceiling. That is necessary but not sufficient for this organ.

Adversarial: using `summary.len()/4` as the **gate**; silent 0 cone; placeholder 4096 as the measured ceiling.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Model proposes nodes; a **deterministic** gate disposes on **countable INPUT cone**, not predicted total cost.
- Cone = transitive deps via an impact graph, tokenized, summed. Missing graph **fails closed**.
- Ceiling is **measured per (model, lane)** (curve or documented baseline 40K/25K class), not a 4K placeholder.
- Outcomes `{Admitted, SplitForced, EscalatedToCloud}`; cannot-split escalates; depth cap exists.
- Atomic huge-cone intent has a non-deadlock escape (cloud or human), not a forced nonsense split.
- Output cap is a **breaker**, not the admit predicate.
- EXP-1-style observe-only before treating the cone as a full controller.
- 4E-style proposal scorer is wrapped, not deleted.
- One journal row per node.

A “split if summary looks long” heuristic, ACONIC CSP, or live T1 window×0.6 **without** fail-closed cone floors is a cousin.

## 14. Non-goals / banned designs

- Pre-execution OUTPUT-token prediction (`J-plan/J-39_decomposition_node_admission_gate.md:11`).
- ACONIC CSP-treewidth as the open-ended coding decomposer (`:11,36`).
- Precise total-cost predictor (`:4`).
- Silent SKIP / admit on missing cone (`:28`).
- Placeholder ceiling constant (`:28`).
- Default-ON gating before EXP-1 + FP bound (`:7,47`).
- Infinite split loop (SeqCV) (`:49`).
- Claiming FIT = CAPABLE (SWEnergy) (`:22`).
- Load-bearing cite of unverified 2606.00953 (`:38`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-39_decomposition_node_admission_gate.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:57-64,1062-1078`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:29-31,175-179`. Live: `node_admission.rs`, `phase_sizing_splitter.rs:13-15,52,150-207`, `context_budget.rs:299-340`, `context_policy.rs:70`. Doctrine: `docs/strategy/ELAI-DETERMINISM-DOCTRINE.md` (as cited).

arXiv **[EXTERNAL, WHY]:** 2604.22750 (r<0.15); 2505.07897 LongCodeBench; 2510.07772 ACONIC **excluded** (`J-plan/J-39_decomposition_node_admission_gate.md:33-36`). VERIFIED 2026-06-16 in-plan.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric `admit_fraction` vs live `SPLIT_RATIO=0.6` vs J-34 `calibrated_trim_fraction`.
- Atomic-intent marker type/schema.
- Community-detection algorithm parameters (Louvain vs other).
- Human-flag channel for the escape hatch.
- EXP-1 r cut behavior in (0.2, 0.5).
- Observe-only-emit default-ON (`:30`) vs no default-ON until EXP-1 (`:47`).
- Live silent-0 cone vs stub_ban fail-closed — which wins if the cousin stays.
- `cone_depth` default for `compute_impact_radius` (`node_admission.rs:91-95`).
- Whether missed SSB emit is FAIL (plan observable) or best-effort (live).
- Frontend tile.
- Exact B-29 production tool name beyond `compute_impact_radius` (plan left file:line unconfirmed).
