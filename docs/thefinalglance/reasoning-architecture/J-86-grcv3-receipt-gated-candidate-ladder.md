# GRC-v3 receipt-gated candidate ladder (J-86)

- **ELAI ids:** J-86 / GS-6 / GS-9 / GS-10 / SL-1..SL-5 / WAVE GRC-3 / IC-6 / DC-3 / IMPL 7/8/9/13/14/15/27
- **Spec status (as designed):** PLANNED **ACTIVATION** of I-4 + D-2 + I-24 + I-26 (DARK substrates exist). Sequence: after J-83, then J-84/J-85 (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:1-5`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:40-44,48-52`, `ELAI-FEATURE-MAP.md:681-692`).
- **Crate / module (spec):** `crates/orchestrator/src/search/ladder.rs` (activate + extend); D-2 PPM port 8101; I-24 canary; SessionDag + H-49 backtrack. Does **not** add a new tree-search engine.
- **Purpose of this file:** enough architecture to rebuild receipt-first candidate selection over the J-83 carrier. Not a status ledger.

## 1. What it is

The **receipt-gated candidate ladder** is the user’s “candidate system” as an **activation** of the dark I-4 evidence-ranked search ladder **over the J-83 BeliefState carrier**. Branch candidate beliefstates on the SessionDag; close each with a D-10-routed **domain receipt**; the receipt-passing candidate **advances** (commit-and-freeze); the rest stay or **backtrack**. Selector hierarchy, triply grounded: **deterministic RECEIPT > vote over INDEPENDENT samples (receipt-absent fallback ONLY, never recall) > LLM self-eval (BANNED)**. Early-stop on convergence (never fixed-N). D-2 PRM is **Tier-2 under the Tier-1 receipt**, guarded by I-24. No MCTS-first (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:5,13-19`, `ELAI-FEATURE-MAP.md:681-692`).

GRC-v3 is UNIFICATION + ACTIVATION of existing pieces plus **one** net-new organ (J-85). J-86 is **activation**, not that new organ (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:8-11`).

## 2. Why it exists

“Sample many reasonings and pick the confident one” reintroduces optimizer-grades-itself. Majority vote **fails** for factual claims (shared training bias); a deterministic checker is strictly superior when one exists (a wrong answer in 30/40 samples still fails the checker) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:11`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:136-138`). ToT’s durable primitive is **backtracking** (~60% of its gain); ELAI already has SessionDag + H-49 — so **no** bespoke tree-search engine (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:11,19`).

If this organ is missing, J-84’s Width tier has nowhere to branch, and I-4 stays a linear-retry cousin. If it is faked (vote while a receipt exists, LLM path-grade, PRM as Tier-1, MCTS-first, vote-for-recall), IC-6/DC-3 are void (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:54-58`).

The receipt-absent **vote rung** is the genuine **addition** (IMPL 8): it stops a silent fall-back to the banned self-judge when receipts run out (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:5`).

## 3. How it works (runtime)

1. **Who calls it.** J-84 **WIDTH** tier → branch candidates over J-83. Candidate-step receipt FAIL → backtrack. Production loop, not a contract-test caller (R7). Full branching sits **high** on the J-84 ladder (hard tier), only when the classifier sees a branching space **and** lower rungs failed receipts; mass stays at no-chain / single-chain (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:11,50-52`).
2. **Input.** K candidate `BeliefState`s (J-83 entries); D-10 mode; optional D-2 step scores; I-24 canary fixtures; I-26 / J-84 budget cap. Generator proposes candidate **continuations**. Orchestrator owns the tree (`I-04-evidence-ranked-search-ladder.md:10-12`).
3. **SL-1 selector (deterministic).** Close each candidate with the D-10-routed domain receipt (F-4 / D-11 / J-1 / H-trading). **When a receipt exists it is the selector, full stop.** Receipt-passing candidate **commits+freezes** (J-83 CV-2/CV-3). Others **pruned-never-promoted** (I-4 invariant, `i4_search_ladder_contract.rs`) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:15,77`). Live substrate: `SearchLadder::run_best_of_n` at `ladder.rs:184`; prune emit `:209`; rank survivors `:282-291` (grounding-map pin `:237/277` is **stale** vs current function layout). Flag `ELAI_SEARCH_LADDER_ENABLE` (`ladder.rs:396-402`).
4. **SL-2 receipt-absent vote (the ADD).** Only when **no** deterministic receipt exists **and** the task is independent-error **and** TYPE is **not** recall. Statistical consensus over **independent** samples. **Never** LLM self-eval. **Never** for factual-recall (J-84 TYPE already sent recall to retrieval — verify this is **structural**, not a bypassable runtime check) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:16,110`).
5. **SL-3 early-stop.** Stop branching when a receipt confirms one **OR** confidence converges **OR** the J-84 depth/width budget caps. **Never** fixed best-of-N. CGES ~69% sample cut at equal accuracy and “gains saturate ~20 samples” are **[EXTERNAL]** — not hard-coded N (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:17,79`).
6. **SL-4 PRM Tier-2-under-receipt.** Activate D-2 PPM sidecar (port **8101**) + I-22 `ProgressRewardScored` for **intermediate** steps that have **no deterministic receipt yet**. **Always under** Tier-1 receipt. Guarded by I-24 isomorphic canary (`CanaryInjector` / `PrmCanaryEvent`). Search over-optimizes past **~400** samples (ADD-6 / Cobbe) — cap, do not keep sampling. Ladder must **not depend** on PPM being up: receipts are self-sufficient; live PRM is a separately-gated W-10 checkbox (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:18,88,111`, `D-02-ppm-sidecar.md:10-14,16-18`). PPM is verifier-only (`PpmAdapter::is_verifier_only() = true`); same-model critic is banned.
7. **SL-5 backtrack.** Candidate-step receipt **FAILS** → backtrack to the last **receipt-passing** J-83 entry over SessionDag branch/merge + H-49 reversible edits. **No** new tree-search engine (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:19,39`).
8. **Corrupt receipt.** Degrade to **abstain/escalate**, never silent self-eval (R1) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:94`).
9. **v2/v3 rungs.** I-4 already bans tournament/PUCT in v1 until measured win. J-86 does **not** make MCTS-first; mass stays at bottom rungs; full tree-search rare and **classifier-gated** (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:5`, `I-04-evidence-ranked-search-ladder.md:10,36`). `rg mcts|monte.carlo.*first` in `ladder.rs` MUST be 0 as a **first** selector (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:71`).
10. **Commit.** Reuse `SearchNodeEvent` / `ProgressRewardScored` / `PrmCanaryEvent` — **net 0** SSB (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:63`). Selection recomputable from `SearchNodeEvent` rows (GD-2).

The model may generate candidate continuations. It may **not** pick the winner, vote on recall, grade its own paths, or use PRM as the commit gate.

## 4. Types, enums, objects

Copy from `J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:13-19` plus I-4 live DTOs. Do not invent fields.

```
SelectorRung {
  Receipt,                         // SL-1; D-10-routed; when present, full stop
  VoteIndependentSamples,          // SL-2; receipt-ABSENT + independent-error + NOT recall
  LlmSelfEval,                     // BANNED — must not be reachable
}

fn select_candidate(...) -> Advance | Prune | Vote | Retrieve | Backtrack | AbstainEscalate

// Early-stop conditions (any):
  receipt_confirms_one | confidence_converges | j84_budget_cap
  // never fixed N
```

**I-4 substrate (ACTIVATE, do not re-author)** — `I-04-evidence-ranked-search-ladder.md:42-75`, `ladder.rs:184-291`:

```
SearchLadder::run_best_of_n(...) -> SearchLadderOutcome
SearchNodeEventKind { Expanded, Pruned, Promoted, BudgetCollapsed }
SearchLadderOutcome { winner: Option<RankedBranch>, pruned_count, pruned, escalated, ranked }
fn is_pruned(verdict) -> bool     // HardVeto or flake-quarantine; terminal
fn rank_survivors                 // lex: tri-layer score, mutation kill-rate, progress
                                  // neural/PPM may order survivors, NEVER resurrect veto
```

**D-2 / I-22 / I-24 (ACTIVATE as Tier-2):**

```
PpmClient::score_step(...) -> PpmScore { score: f32 in [0,1], confidence, raw_output }
  endpoint 127.0.0.1:8101
ProgressRewardScored            // I-22
CanaryInjector / PrmCanaryEvent // I-24; frozen fixtures R8
```

**Live cousin (do not treat as J-86 done):** I-4 v1 already receipt-ranks via I-17/I-20/I-22 **without** J-83 carrier branching, without the SL-2 vote rung, without early-stop-vs-fixed-N, without PRM-under-receipt placement. Un-darking I-4 **alone** is not this organ.

## 5. Control flow / state machine

**Owner:** orchestrator ladder. J-84 decides **whether** Width is legal. J-85 may **stop** a drifting call. J-83 **is** the candidate object. SessionDag/H-49 **is** backtrack.

```
J-84 tier == Width
     │
     ▼
branch K candidate BeliefStates on SessionDag
     │
     ▼
for each candidate (until early-stop):
     D-10 routes domain receipt
     │
     ├─ receipt PASS → ADVANCE (J-83 commit+freeze); prune rest (never promote)
     ├─ receipt FAIL → BACKTRACK to last receipt-passing entry; new continuation
     ├─ no receipt, TYPE=Retrieve/recall → RETRIEVAL, not vote
     ├─ no receipt, independent-error → VOTE over independent samples
     └─ corrupt receipt → ABSTAIN / I-7 escalate (never self-eval)
     │
     intermediate no-receipt-yet steps:
        D-2 PRM score UNDER Tier-1 (optional; PPM down ≠ fail the ladder)
        I-24 canary; stop width past ~400-sample over-optimization cap
     │
     early-stop if receipt confirms OR confidence converges OR J-84 cap
```

**Forced:** vote cannot run when a receipt exists; vote cannot run on recall; PRM cannot promote a vetoed/failed-receipt candidate; MCTS is not v1 first rung.

## 6. Grounding, receipts, gates

**Selector hierarchy (wave-wide, this organ is the candidate grain):**

| Rank | Signal | Legal? |
|---|---|---|
| 1 | Deterministic **external receipt** (D-10-routed) | **always, when it exists** |
| 2 | Vote over **independent** samples | receipt-**absent** + independent-error **only**; **never recall** |
| 3 | LLM self-eval / ToT value-head / “I prefer path A” | **BANNED** |

(`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:136-138`, `ELAI-FEATURE-MAP.md:685-687`, `J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:15`)

I-4 ranker already: tri-layer > mutation kill-rate > progress; neural/PPM **orders survivors only** (`I-04-evidence-ranked-search-ladder.md:27-31`). J-86 **does not weaken** that: PRM stays below the receipt.

**Never allowed:** vote-when-receipt-exists; vote-for-recall; LLM path grade; PRM as Tier-1; un-veto via score; fixed-N as the stop; MCTS-first.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| I-4 flag | `ELAI_SEARCH_LADDER_ENABLE` DARK | process (`ladder.rs:396-402`) |
| D-2 flag | `ELAI_PPM_ENABLE` + toml `[runtime.ppm]` | sidecar 8101 (`D-02-ppm-sidecar.md:18`) |
| Over-optimization cap | **~400** samples (ADD-6 / Cobbe) **[EXTERNAL magnitude]** — I-24 cap, not a paper paste into routing | search width |
| CGES / saturate ~20 | **EXTERNAL WHY** — early-stop exists; do not hard-code N=20 | SL-3 |
| PPM retries | max 3 (D-2) | sidecar (`D-02-ppm-sidecar.md:26`) |
| SSB | reuse three event families; **net 0** | W-9 |
| W-10 | live PRM remote-GPU is a **separate checkbox**; fixture leg can ship first | SL-4 |

**Do not confuse with:** J-84 Width **decision** (this organ **runs** Width); J-85 token ceiling (during a call); I-4 v2 tournament / v3 PUCT (gated until v1 measured win).

## 8. Roles and neighbors

Triggered by J-84 hard/Width. Verifier stronger-than-generator (VD-4). Cross-family adversary remains D-13 — same-model critique is not Mode B and not this selector.

**Consumes:** J-83 candidates; D-10 receipts; I-4 `run_best_of_n`; D-2/I-22/I-24; SessionDag/H-49; I-26 early-stop budget; I-7/I-8.

**Produces:** Advanced frozen BeliefState; pruned siblings; backtrack; optional PRM step scores under receipt; reused SSB rows.

**Must not fork:** a second ladder; MCTS engine; vote selector while checkers exist.

**Not parallel-safe in-wave:** layers on J-83; **reads** J-84 Width + J-85 guard; **ship last** (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:8,112`).

## 9. Config / flags / defaults

- Un-dark I-4/D-2 behind **kill-switches** with **wire-dark-allowlist** rows (type/owner/expiry/promotion_plan_id/smoke). Not a terminal default-OFF (W-2/R9) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:85-86,99`).
- Frontend: existing search-ladder / I-27 ConvergencePanel. Decide tile at edit (W-4); justified N/A (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:59-60`).
- Decisions doc: `workflow/reasoning/J-86-decisions.md` (hierarchy + vote-absent-only + PRM-under-receipt) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:40`).
- I-24 canary fixtures frozen (R8).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `search/ladder.rs` `run_best_of_n` | ACTIVATE + SL-1..SL-5 ~180 LOC (`ladder.rs:184`) |
| `task_runner::dispatch_search_ladder` | I-4 production hook (`I-04-evidence-ranked-search-ladder.md:18`) |
| D-2 `ppm_client.rs` / port 8101 | SL-4 |
| I-22 `ProgressRewardScored` | SL-4 |
| I-24 `CanaryInjector` | SL-4 guard |
| SessionDag branch/merge + H-49 | SL-5 |
| `wire-dark-allowlist.toml` | I-4/D-2 un-dark rows |
| `j86_candidate_ladder_contract.rs` | CREATE ~90 LOC |
| `workflow/reasoning/J-86-decisions.md` | CREATE ~30 LOC |
| `i4_search_ladder_contract.rs` | existing prune-never-promoted |

## 11. Journal / SSB events

Reuse, **net 0** (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:63`):

```
SearchNodeEvent { Expanded, Pruned, Promoted, BudgetCollapsed }   // I-4
ProgressRewardScored                                              // I-22
PrmCanaryEvent                                                    // I-24
```

Kind add only if edit-time W-9 requires it; 9-tuple lockstep. Do not rewind 268/577.

## 12. Tests and observables

Named contract `j86_candidate_ladder_contract.rs` (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:93-94`):

| Id | Assert |
|---|---|
| (a) | receipt-passing advances; rest **pruned-never-promoted** |
| (b) | receipt-absent independent-error → independent-sample **vote** |
| (c) | recall → **retrieval**, never a vote |
| (d) | LLM-self-eval selection **rejected** |
| (e) | early-stop on receipt convergence (**not** fixed-N) |
| (f) | D-2 PRM scores intermediate **only under** Tier-1; I-24 canary |
| (g) | failed candidate **backtracks** to last receipt-passing entry |
| (h) | over-width past cap **stopped** (ADD-6) |
| (i) | corrupt receipt → abstain/escalate, **never** silent self-eval |

**W-1 smoke:** hard branching **with** receipt; open-ended **without**; recall query. Negative: LLM self-grade MUST NOT select; PRM never Tier-1 (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:61-64,85`).

**DETERMINISTIC_CHECK:** SL-1..SL-5 strings present; self-eval / vote-recall count 0; mcts-first count 0; production_caller 0 DARK; `J-86-decisions.md` exists (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:66-73`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Candidates are **typed beliefstates** (J-83 shape), not raw traces.
- Selector: receipt (when it exists) **beats** everything; vote only if receipt-absent **and** independent-error **and** not recall; LLM self-eval unreachable.
- Receipt-pass → commit-and-freeze + prune rest (never promote pruned).
- Receipt-fail → backtrack to last receipt-passing node on an existing DAG/undo substrate — **no** new MCTS engine as the first selector.
- Early-stop on receipt or confidence convergence or budget cap — **not** fixed-N.
- Process-reward / PRM, if present, scores **intermediate no-receipt** steps **under** Tier-1 and is canary-capped; ladder works if PRM is down.
- Width is classifier-gated (hard tier), not default for every task.
- I-4/D-2/I-24/H-49 activated, not rewritten.

Un-darking best-of-N without the vote-rung / recall-ban / PRM-under-receipt / J-83 carrier is a cousin, not this organ.

## 14. Non-goals / banned designs

- Vote as selector when a receipt exists (IC-6, DC-3).
- LLM self-eval / ToT value-head as selector (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:55`).
- Vote for factual-recall (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:56`).
- PRM as Tier-1 gate (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:57`).
- Uncapped width / ignoring I-24 ~400-sample over-opt (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:57`).
- MCTS-first / bespoke tree-search (IC-6) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:58`).
- Re-implementing I-4 / D-2 / I-24 / H-49 (GD-1).
- Ladder that **requires** PPM up (W-10) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:111`).
- Shipping before J-83 (and before J-84 Width + J-85 in-wave sequence) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:112`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md` (full file). Grounding map: `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md` GS-6/GS-9/GS-10, DC-3. Index: `J-plan/J-SUMMARY_INDEX.md:1901-1915`. Feature map: `ELAI-FEATURE-MAP.md:681-692`. Neighbors: `reasoning-architecture/I-04-evidence-ranked-search-ladder.md`, `D-02-ppm-sidecar.md`. Live: `search/ladder.rs:184-291,396-402`. Design: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` §3/§9 + IMPLICATION 7/8/9/13/14/15/27 + ROUND-3 ADD-6 (as cited by the J-86 plan).

arXiv **[EXTERNAL, WHY]:** self-consistency 2203.11171; “Consensus is Not Verification” 2603.06612; ToT 2305.10601; test-time-compute 2408.03314; CGES 2511.02603; Cobbe over-optimization (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:106`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Structural vs runtime foreclosure of vote-for-recall (plan prefers J-84 TYPE / D-10 so SL-2 **never sees** recall; verify at authoring) (`J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md:110`).
- Vote aggregation (majority, weighted, k-of-n) — plan says “statistical consensus over independent samples”, not the estimator.
- “Confidence converges” numeric rule (ties to H-39? I-26 plateau? K-15?).
- Whether ~400 is an I-24 **measured** cap or a cited Cobbe magnitude to re-measure (ID-5 vs ADD-6).
- How I-4’s existing lex rank (tri-layer, mutation, progress) composes with J-83 domain receipts when both fire.
- Exact SessionDag API for “last receipt-passing entry”.
- Promotion plan ids for allowlist un-dark rows.
- Grounding-map `run_best_of_n` `:237/277` vs live `:184` / rank `:282`.
