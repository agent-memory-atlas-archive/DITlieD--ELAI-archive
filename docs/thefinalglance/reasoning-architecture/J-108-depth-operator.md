# Multi-hop DEPTH operator (HippoRAG-PPR primary + RetrievalNavigator fallback)

- **ELAI ids:** J-108 / RN-1..RN-5 / RG-3..RG-6 / RS-1..RS-4 / RC-1..RC-2 / WAVE RLM-2
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:161-180`). Depends J-107; ships AFTER it. Local-lane default-ON is **gated** on J-107’s lift + no-collapse artifact. Default-OFF `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` + wire-dark-allowlist (R9).
- **Crate / module (spec):** `.elai_cc/crates/orchestrator/src/search/navigator.rs` (new); HippoRAG routing + `task_runner.rs` retrieval-seam wiring; consumes `crates/memory/src/hippo_rag.rs::retrieve_notes`, `crates/knowledge/src/pipeline.rs::query_with_tier`, `crates/tools/src/adaptive_tool.rs`, `crates/orchestrator/src/search/ladder.rs`
- **Purpose of this file:** enough architecture to rebuild the DEPTH sibling of the J-86 WIDTH ladder — deterministic multi-hop that abstains rather than fabricates. Not a status ledger.

## 1. What it is

The **multi-hop / long-horizon DEPTH operator** is a sequential retrieval loop. SearchLadder (J-86 / I-4) is **WIDTH** — branch N candidates in parallel, pick the best by receipt. DEPTH iterates: each step’s op is informed by the last, to **chain hops**. Priority order: **RN-1 PRIMARY** route a multi-hop query through the shipped HippoRAG Personalized-PageRank retriever (`hippo_rag.rs::retrieve_notes`) plus an **atomic** model extraction; **RN-2 FALLBACK** a constrained `RetrievalNavigator` with a **FIXED** `NavOp` vocabulary over `query_with_tier`’s candidate set, only when there is no clean entity linkage; **RN-3** on `max_steps` exhaustion **abstain** (`BudgetCollapsed` / `winner:None`), never guess (`J-plan/J-108_retrieval_navigator_depth_operator.md:1-5,15-20`, `ELAI-FEATURE-MAP.md:161-180`). The model proposes **one** named op or reads **one** atomic entity. A deterministic executor adjudicates (TD-3).

## 2. Why it exists

Single-shot retrieval structurally cannot multi-hop. ELAI experiments: lexical long-context is already won by deterministic retrieval (RG-3: BM25 100% vs free-RLM 62% erratic). Multi-hop (synonym needle the question never names) is the hole: Arm C (single-shot) abstains-safe but 0%; Arm B (free-code RLM) **fabricated "42" on 7/8** at 8×W and never engaged recursion even on a 120B; Arm D (deterministic iterative nav) was the only arm >0% and correct-or-abstains (RG-4). Ceiling measurement: ORACLE pure-deterministic traversal = 8/8; model-driven nav **plateaus ~25%**; a graph-hop hybrid (deterministic hops, model only for atomic read) on a weak 8B = 87% (RG-5) (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:27-47`). If this organ is missing, multi-hop either silent-fails (single-shot) or fabricates (free-Python RLM). If it is faked (model writes code over context, open op set, guess on budget exhaustion, navigator as primary), the trust contract that C/D preserved is broken (`J-plan/J-108_retrieval_navigator_depth_operator.md:12-13,59-63`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator, at the **same** `task_runner` retrieval dispatch seam where SearchLadder is constructed (`PLAN-asserted ~task_runner.rs:12869+`, re-grep W-7). Arc-cloned at boot. `navigate()` / `route` **must** be reached from that path in the **same changeset** (D-030). A contracts caller never satisfies this (R7) (`J-plan/J-108_retrieval_navigator_depth_operator.md:42,55-57`).
2. **Input.** A multi-hop / long-horizon query (a needle the single-shot top-k misses), plus lane/model identity for the J-107 knee. Flag `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` must be on for the navigator fallback; HippoRAG primary is the shipped retriever (`J-plan/J-108_retrieval_navigator_depth_operator.md:42`, `ELAI-FEATURE-MAP.md:175-177`).
3. **Router (deterministic — RN-1 vs RN-2).** Reuse J-84’s TYPE axis / D-10 / the DKG entity graph’s **edge presence**. Clean entity linkage → RN-1. No clean linkage → RN-2. The model does **not** choose the route. A runtime-heuristic that could drift is a known risk, not a license to let the model pick (`J-plan/J-108_retrieval_navigator_depth_operator.md:39,117`).
4. **RN-1 PRIMARY (deterministic graph-hop + atomic read).** `hippo_rag.rs::retrieve_notes` — one PPR pass, query → bridge entity → all hops. Then an **ATOMIC** model extraction: read the entity / regex the number off the single grepped hop chunk, optional **3-sample vote for the atomic read only**. The model **never navigates**. Do **not** re-implement PPR (GD-1, RS-4, RC-2) (`:16,39`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:68-71,94-98`).
5. **RN-2 FALLBACK (constrained navigator).** `navigator.rs` bounded loop over `query_with_tier` candidates. FIXED `NavOp::{Grep(pat), Read(ids), Expand(id), SubQuery{ids,q} (depth<=2), Answer(s)}`. Model **proposes** one named op. Deterministic executor **runs** it. Every op is an SSB row via adaptive_tool markers (RS-2). Caps ~25–37% on hard multi-hop — the safety net, never the primary (`:17,40`, `ELAI-FEATURE-MAP.md:168-172`).
6. **RN-4 SubQuery bound.** SubQuery is a capped child call (`depth<=2`) over shipped orchestrate/BSP sub-agent machinery, **not** an open REPL. Sits below the J-55 code-mode collapse threshold (RG-6: constrained named-op held 100% on the weak lane) (`:19,40`).
7. **RN-5 step budget.** `max_steps` reads the J-107-measured per-(model,lane) knee (ID-5). Fixture default now; live knee when J-107’s local run lands (W-10). **Never** a paper constant (`:20,41`).
8. **RN-3 abstain-on-budget.** On `max_steps` exhaustion emit the SearchLadder `BudgetCollapsed` equivalent and return `answer:None`. **Never a guess.** Measured trust contract: B fabricated, D abstained (RG-4) (`:18,41,63`).
9. **Hop backtrack (optional substrate).** SessionDag branch/merge + H-49 undo-tree for a failed hop. No bespoke tree-search (`:28,54`).
10. **Commit.** HippoRAG path rides `query_with_tier` existing rows. Navigator ops reuse `SearchNodeEvent` (`Promoted` / `BudgetCollapsed`) + adaptive_tool markers (`ToolCallStart/Read/Branch/Finding/Converge/Escalate/ToolCallEnd`). Net 0..+1 Kind, lockstep W-9 (`:68-69`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:120-121`).
11. **Fail-closed.** Out-of-set op rejected. No code-execution path in `navigator.rs` (`rg` of `exec|eval|python|interpret|run_code|compile` MUST be 0). Corrupt/empty candidate set → abstain, never fabricate. Guess on exhaustion → FAIL the negative arm (`:59-63,99`).

The model may **generate** one `NavOp` proposal or the atomic entity/number read. It may **not** navigate the corpus, author a new op, execute code over the context, choose RN-1 vs RN-2, or emit an answer after budget collapse.

## 4. Types, enums, objects

Copy from `J-plan/J-108_retrieval_navigator_depth_operator.md:16-20,39-41` and `I-plan/I-4_evidence_ranked_search_ladder.md:68`. Do not invent fields.

```
NavOp {                              // FIXED vocabulary; closed
  Grep(pat),
  Read(ids),                         // dispatch_file_read_outline IS the Read op (RS-2)
  Expand(id),
  SubQuery { ids, q },               // capped child-spawn, depth<=2; NOT an open REPL
  Answer(s),
}

DepthRoute {                         // deterministic; model does not pick
  HippoRagPrimary,                   // RN-1: clean entity linkage
  NavigatorFallback,                 // RN-2: no clean entity linkage
}

RetrievalNavigator {                 // navigator.rs; copy SearchLadder struct/handle/journal idiom
  // bounded loop over query_with_tier candidate set
  max_steps,                         // RN-5: J-107 knee, not a paper constant
  // deterministic executor
}

DepthOutcome {
  Some(answer),                      // only from RN-1 atomic extract or NavOp::Answer under budget
  None,                              // RN-3: BudgetCollapsed equivalent; NEVER a guess
}

SearchNodeEvent Kind {               // I-4 / J-86; REUSE
  Expanded, Pruned, Promoted, BudgetCollapsed
  // J-108 uses Promoted / BudgetCollapsed (plan); other Kinds owned by WIDTH
}

AdaptiveToolMarkers {                // RS-2; REUSE per NavOp
  ToolCallStart, Read, Branch, Finding, Converge, Escalate, ToolCallEnd
}

fn retrieve_notes(...)               // hippo_rag.rs:57; ACTIVATE, do not re-implement
fn query_with_tier(...)              // pipeline.rs:581; RN-2 candidate set
fn run_best_of_n(...)                // ladder.rs:173; idiom to COPY, not call as WIDTH
fn search_ladder_enabled()           // ladder.rs:382
// BudgetCollapsed at ladder.rs:350  // abstain-on-budget pattern to COPY
```

**Atomic-read 3-sample vote** is scoped to the **fuzzy entity/number read only**. It is never the multi-hop selector (IC-6 / K-16 RR-2) (`J-plan/J-108_retrieval_navigator_depth_operator.md:120`).

**`has clean entity linkage`:** plan says reuse J-84 TYPE axis / DKG edge presence. Exact predicate type is UNCERTAIN (§16).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `RetrievalNavigator` / HippoRAG router. The model proposes one op; the executor accepts, rejects, or abstains. No token-level navigator. No REPL.

**Decision table (must be contract-tested):**

| Condition | Path | Notes |
|---|---|---|
| multi-hop **and** clean entity linkage | RN-1 HippoRAG-PPR + atomic extract | one deterministic pass; model never navigates |
| multi-hop **and** no entity linkage | RN-2 `RetrievalNavigator` | FIXED `NavOp`; model proposes one op |
| `max_steps` exhausted | RN-3 `answer:None` + `BudgetCollapsed` | never a number |
| SubQuery | child, `depth<=2` | not open recursion |
| out-of-set op | reject | G6 / TD-3 negative arm |
| empty/corrupt candidates | abstain | R1; never fabricate |
| flag off | today’s single-shot / HippoRAG-only | loudly; not a silent cousin |
| local lane, J-107 gate not cleared | HippoRAG + single-shot only; navigator capable-lane only | honest residue GD-5 |

**Forced transitions:** first retrieval on a linked multi-hop query = HippoRAG, not navigator (RC-2). Navigator fires **only** on no-entity-linkage (`:40,83-84`). WIDTH ladder (J-86) is orthogonal and coexists (`:13`).

**Local-lane default-ON:** authored **only if** J-107’s arm cleared (lift + no J-55 collapse). Else navigator is cloud/capable-lane only (`:45,87,119`).

## 6. Grounding, receipts, gates

DEPTH **navigates by determinism**. Confidence that a hop is correct comes from HippoRAG’s graph pass / tool receipts / `BudgetCollapsed`, not from the model’s narration of the hop chain (K-16 RR-1). Candidate **selection** among hops is not a vote over independent samples except the optional 3-sample **atomic read** (RR-2 scoped exception, plan-explicit).

**Selector hierarchy here:** deterministic PPR / named-op receipts > (atomic-read 3-sample vote, fuzzy read **only**) > LLM self-eval of “I found it” **BANNED**.

**Never allowed as a confidence signal / navigator:** model-written Python over the context, open/dynamic op set, “the model said hop 2 is right,” paper step-budget, window fill as depth, WIDTH `run_best_of_n` used as if it were DEPTH (`:59-63`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:94-98`).

**J-107 is the calibration + promotion gate**, not a runtime receipt for a single query.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `NavOp` set | exactly `{Grep, Read, Expand, SubQuery, Answer}` | per navigator |
| SubQuery depth | `<=2` | per child; uncapped = ban |
| `max_steps` | J-107 per-(model,lane) knee; fixture default until W-10 | per query |
| experiment D@18 | OWN DATA: 10→18 recovered abstained-but-reachable cases | **not** the production default (ID-5) (`:20`) |
| navigator fallback cap | ~25–37% on hard multi-hop (RG-5) | DESIGN-rationale, not a done-gate number |
| ORACLE / hybrid | 8/8 and 87% | OWN DATA ceiling; never a shipped SLA |
| flag | `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` default-OFF | process |
| SSB | reuse `SearchNodeEvent` + adaptive_tool markers; net 0..+1 | per op / outcome |
| navigator.rs (plan) | ~210 LOC | new file |
| task_runner wire | ~40 LOC | D-030 same changeset |
| contract tests | ~110 LOC | `j108_retrieval_navigator_contract.rs` |

**Overflow:** `max_steps` exhaustion → `None`, not a longer loop. SubQuery deeper than 2 → reject. Paper-constant `max_steps` → FAIL contract (h): must read the J-107 artifact (`:99`).

Do not confuse with: GRC `GrcBudget` (2A), serial-depth H-42, WIDTH ladder K, context-fill J-34.

## 8. Roles and neighbors

Role-agnostic at the retrieval seam. J-84 classifier routes a query to single-shot vs depth (`:8,54`). I-7 / I-8 escalate/abstain compose (`:8,54`).

**Consumes:** `retrieve_notes` (RS-4, activate); `query_with_tier` (RS-1); `ToolDispatcher` + `dispatch_file_read_outline` + markers (RS-2); `ladder.rs` struct/handle/journal + `BudgetCollapsed` idiom (RS-3, **copy**, do not re-implement WIDTH); SessionDag / H-49; J-84; J-107 knee + gate; I-7/I-8; J-42 / J-38 / J-86 as the DEPTH × WIDTH axes this extends (`:49-54`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:58-81`).

**Produces:** HippoRAG-primary answers, navigator hop chains (replayable SSB), `answer:None` on exhaustion, `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` allowlist row, `workflow/reasoning/J-108-decisions.md`.

**Must not fork:** a second PPR; a second candidate cascade; a free-Python RLM REPL; model-chosen orchestration; local-lane CodeAct; an external trace store (SSB rows **are** the record, GD-2).

**Not parallel-safe on sequence with J-107** (reads the gate). Parallel-safe on files (`:9`).

## 9. Config / flags / defaults

- `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` default-OFF, kill-switch posture (W-2), **not** a terminal default-OFF. Promotion gated on J-107 (`:90`).
- `wire-dark-allowlist.toml` row: `type=experiment`, owner, `expiry_date`, `promotion_plan_id=J-108`, `smoke_test` (R9) (`:42,104`).
- Local-lane default-ON **only** after J-107 clears; else capable-lane only + local HippoRAG/single-shot, and **say so** (GD-5) (`:119`).
- Decision doc: HippoRAG-primary/navigator-fallback ordering; fixed-NavOp/deterministic-executor TD-3; abstain-on-budget; J-107-gates-the-local-lane; free-Python-RLM / model-chosen-orchestration non-goals (`:44`).
- Cloud/capable lane may ship without waiting on J-107 (`:8`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/search/navigator.rs` | `NavOp` + loop + executor + abstain + HippoRAG routing (`:48,102`) |
| `task_runner.rs` retrieval seam ~12869 | PRODUCTION_HOOK, D-030 (`:56,103`) — PLAN-asserted, re-grep W-7 |
| `crates/memory/src/hippo_rag.rs::retrieve_notes` `:57` | RN-1 primary (RS-4) |
| `crates/knowledge/src/pipeline.rs::query_with_tier` `:581` | RN-2 candidates (RS-1) |
| `crates/tools/src/adaptive_tool.rs` `:295` | op journaling (RS-2) |
| `search/ladder.rs` `:173` / `:350` / `:382` | idiom + `BudgetCollapsed` (RS-3) |
| `wire-dark-allowlist.toml` | `ELAI_RETRIEVAL_NAVIGATOR_ENABLE` (`:104`) |
| `crates/contracts/tests/j108_retrieval_navigator_contract.rs` | (a)–(i) (`:99,106`) |
| `workflow/reasoning/J-108-decisions.md` | five decisions (`:44,107`) |
| `ssb.rs` + `.golden` | only if +Kind (`:105`) |

**Live cousins (do not rebuild as this organ):** using WIDTH `run_best_of_n` as multi-hop; calling HippoRAG from a test-only harness without the `task_runner` seam (D-030 dark = BLOCKED) (`:118`).

## 11. Journal / SSB events

No new action_type family (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:120-121`):

```
SearchNodeEvent { Promoted, BudgetCollapsed, ... }   // RS-3; reuse
adaptive_tool markers                                 // one set per NavOp
query_with_tier existing rows                         // HippoRAG / candidate path
```

Round-trip: op stream replays identically (GD-2, contract (c)). Do not add an external trace store (`:63`).

## 12. Tests and observables

Named contract `j108_retrieval_navigator_contract.rs` (`:99`):

| Id | Assert |
|---|---|
| (a) linked → HippoRAG | multi-hop WITH entity linkage routes to PPR, **not** navigator |
| (b) unlinked → navigator | no-entity-linkage routes to `RetrievalNavigator` |
| (c) GD-2 | every `NavOp` emits an SSB row; stream replays identically |
| (d) closed set | out-of-set op rejected (G6/TD-3) |
| (e) no code path | no execution path in navigator (RG-4) |
| (f) SubQuery cap | `depth<=2` |
| (g) abstain | `max_steps` → `answer:None` + `BudgetCollapsed`; emits **no number** |
| (h) knee | `max_steps` reads J-107 artifact, not a literal constant |
| (i) R1 | corrupt/empty candidates → abstain, never fabricate |

Integration smoke R10 (`:67-69`): drive the **real** `task_runner` retrieval entrypoint on (a) linked multi-hop → HippoRAG, (b) no linkage → navigator, (c) unreachable → abstain.

`rg` hygiene (`:74-77`): `exec|eval|python|...` = 0; `self.choose.*control.flow|model.*authors.*op|dynamic.*op|open.*op.set` = 0; allowlist row present.

Negative arm that **must FAIL:** a model executing code over the context, a fabricated answer on budget exhaustion, or an op not in `NavOp` (`:69`).

W-1: the 3-case smoke on a hard long-context input, not file-exists (`:91`). production_caller 0-DARK this changeset (`:88`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Closed `NavOp` with exactly `{Grep, Read, Expand, SubQuery, Answer}`.
- PRIMARY multi-hop path is deterministic graph-hop (HippoRAG-PPR or equivalent **already-shipped** entity-graph PPR) + atomic model read. The model never navigates.
- FALLBACK navigator runs **only** when entity linkage is absent. Executor is deterministic; model proposes one op.
- SubQuery `depth<=2`. No REPL, no `eval`, no model-authored ops.
- Budget exhaustion returns `None` + a `BudgetCollapsed`-equivalent row and **no** guessed number.
- `max_steps` is a measured per-(model,lane) knee (J-107), not a paper constant.
- Wired from the production retrieval dispatch (same changeset, not test-only).
- Default-OFF flag + allowlist; local-lane default-ON only after the LongCoT scaffold arm clears lift + no-collapse.
- Op journal is the product journal, not a side file.

A free-Python RLM, an open tool-loop, WIDTH best-of-N used as multi-hop, or “the model hops until it feels done” is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Free-Python RLM / model-written code over its own context (RG-4: 0% + hallucinated; REJECTED every lane) (`:59`).
- Model-CHOSEN orchestration / open `NavOp` set (RG-1 G6, TD-3) (`:60`).
- Model navigation as the multi-hop **PRIMARY** (RC-2) (`:61`).
- Guess on budget exhaustion (`:62`).
- Uncapped SubQuery (`:62`).
- Local-lane general CodeAct (J-55) (`:63`).
- Paper-constant step budget (`:63`).
- Re-implementation of hippo_rag / `query_with_tier` / ladder / adaptive_tool (GD-1) (`:63`).
- External trace store (`:63`).
- `.unwrap` on prod (`:63`).
- Shipping `navigator.rs` dark (D-030) (`:118`).
- Authoring local-lane default-ON before J-107 clears (`:119`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-108_retrieval_navigator_depth_operator.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:2231-2249`. Feature map: `ELAI-FEATURE-MAP.md:161-180`. Grounding map: `J-plan/_grounding/J-RLM-GROUNDING-MAP.md` RG-3..RG-6, RS-1..RS-7, RC-1..RC-2. Design sketch: `workflow/experiments/RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md`. Own measured failure modes: `workflow/experiments/RLM-VS-ELAI-RESULTS-2026-06-26.md`. Steal analysis: `workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md` PART 5 H1 + PART 6 + PARTS 7/8/9. WIDTH sibling types: `I-plan/I-4_evidence_ranked_search_ladder.md:43-68`. ArXiv (on-disk maps, RC-3): RLM 2512.24601, HippoRAG 2405.14831. Neighbor organs: J-107, J-86, J-84, J-42, J-38, J-55, I-4, I-7, I-8.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact `has clean entity linkage` predicate (J-84 TYPE vs DKG edge presence vs both) (`:117`). Plan requires it be deterministic and structurally foreclose model-chosen routing; the boolean’s inputs are not a typed struct here.
- `task_runner.rs:12869` is PLAN-asserted; live line may have moved (W-7) (`:8,56,118`).
- Fixture numeric `max_steps` before J-107’s knee exists (`:41,86`). Experiment 10→18 is **not** automatically the fixture.
- Whether `SearchNodeEvent` Kind needs a DEPTH discriminator (net 0..+1, decide at edit) (`:68,96`).
- `NavOp` field types (`pat`, `ids`, `q`, `s`) — named, not typed (`:17`).
- Atomic-extract API (regex vs model read vs 3-sample vote as required vs optional) (`:16,39,120`).
- Whether RN-1 HippoRAG routing lives inside `navigator.rs` or a sibling module (`:102` puts routing in `navigator.rs`; `:39` also names “a deterministic router”).
- Frontend tile vs reuse vs N/A (W-4) (`:65`).
- How J-84 “single-shot vs depth” composes with RN-1 vs RN-2 (two routers stacked; interaction unspecified).
- LIVE schema number at implement time (re-derive W-9).
