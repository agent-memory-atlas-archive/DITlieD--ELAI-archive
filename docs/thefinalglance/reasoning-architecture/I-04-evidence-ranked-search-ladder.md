# Evidence-ranked search ladder (I-4)

- **ELAI ids:** I-4 / IC-6 / ID-1 / ID-2 / ID-4 / IN-1 / J-11 (S-2/S-3 merge-gate + worktree lifecycle) / SCHEMA 261
- **Spec status (as designed):** DARK (`ELAI_SEARCH_LADDER_ENABLE`; `ELAI-FEATURE-MAP.md:835-839`, `I-plan/I-SUMMARY_INDEX.md:91-107`). Original plan is WAVE 4 KEYSTONE, BLOCKING for I-22/I-28/I-30 (`I-plan/I-4_evidence_ranked_search_ladder.md:3`).
- **Crate / module (spec):** `crates/orchestrator/src/search/` — `ladder.rs` (v1), `mod.rs`; `search/tournament.rs` and `search/mcts.rs` are **gated rungs**, not v1. Production hook: `task_runner::dispatch_search_ladder` → `SearchLadder::run_best_of_n`.
- **Purpose of this file:** enough architecture to rebuild receipt-ranked best-of-N (and the gated later rungs). Not a status ledger.

## 1. What it is

The **evidence-ranked search ladder** is the orchestrator-owned selector that replaces LLM-judge branch picking. The generator proposes **K divergent candidates** (C-26 temperature schedules). The orchestrator owns the tree. **Selection is by receipts**: I-17 tri-layer verdict first, I-20 mutation kill-rate second, progress score third. A neural scorer may **order survivors**; it may never **resurrect** a vetoed branch (`I-plan/I-4_evidence_ranked_search_ladder.md:4-21`, `I-plan/I-0_INDEX.md:235-239`). Three rungs: **v1 best-of-N** (ships first), **v2 tournament**, **v3 PUCT over SessionDag with virtual loss**. v2/v3 code is banned until v1 shows a measured win on ≥3 real runs (GD-4) (`I-plan/I-4_evidence_ranked_search_ladder.md:11-21,40-51`).

## 2. Why it exists

Public systems ship **wide parallelism + an LLM-judge selector**. That is optimizer-grades-itself on the search tree (IC-6). If this organ is missing, a fluent compile-fail branch can win because a model “liked” it. If it is faked (prompt “pick the best patch”, or a neural score that un-vetoes L1 HardVeto), ID-4 is dead (`I-plan/I-4_evidence_ranked_search_ladder.md:4-8,79-83`, `I-plan/I-0_INDEX.md:222-226,235-239`). Failed siblings must stay out of later prompts (vividness trap); I-5 **measures** that tax — this organ **calls** the pristine filter per branch and must not re-accumulate raw tracebacks (`I-plan/I-4_evidence_ranked_search_ladder.md:7-8,58-59`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator, at the **task_runner baby-step dispatch** that today is linear retry. The ladder **replaces that retry only after a measured first-win**; until then linear retry stays (`I-plan/I-4_evidence_ranked_search_ladder.md:44-46,76-77`). Live hook name: `task_runner::dispatch_search_ladder` (`crates/orchestrator/src/lib.rs:884-889`, `task_runner.rs:14060-14082`). The model does **not** call `run_best_of_n`.
2. **Input.** K `Branch` candidates (I-17) plus optional per-branch receipts: mutation kill outcomes (I-20) and a progress score (I-22 / rank-3). Isolated scratch per branch from I-2 (`I-plan/I-4_evidence_ranked_search_ladder.md:12-17,41-47,70`).
3. **Per-branch expand (deterministic host, generator already proposed source):**
   - Spawn / bind an I-2 `BranchWorkspace` (ID-2: **no shared mutable scratch**) (`:70,85-86`).
   - Rebuild per-branch context via I-5 pristine filter; preserve `context_budget.rs::update_breakpoints` (IS-6) (`:74-75`). Failed siblings + raw tracebacks are **hard-excluded** (I-5 owns the filter; I-4 calls it) (`:58-59`, `I-plan/I-SUMMARY_INDEX.md:110-124`).
   - Run `TriLayerStack::evaluate` / `run_verdict` (I-17) (`:71`, `I-plan/I-17_tri_layer_evaluation_stack.md:67`).
   - Score mutation via `MutationScheduler::score` / I-20 Wilson-CI kill-rate (`:72`, `I-plan/I-20_budgeted_mutation_scheduler.md:18-20`).
   - Emit `SearchNodeEvent` **Expanded** (`:67-68`).
4. **Prune (deterministic, terminal).** L1 HardVeto **or** flake-quarantined oracle → `SearchNodeEvent` **Pruned**. The branch is **dead**. Ranker must not see it as a candidate for promotion (`:81-83`, `I-plan/I-17_tri_layer_evaluation_stack.md:10-12,70-73`).
5. **Rank survivors (deterministic lexicographic key — first match wins the order):**
   1. tri-layer verdict / score (I-17; veto already stripped)
   2. mutation kill-rate (I-20 Wilson-CI point)
   3. progress score
   (`I-plan/I-4_evidence_ranked_search_ladder.md:47`, live fill of the same key: `search/ladder.rs:277-291`). A neural / PPM score may **only** break order **among** survivors; it is **not** a fourth key that can beat a veto (`I-plan/I-0_INDEX.md:235-238`).
6. **Promote.** Emit `SearchNodeEvent` **Promoted** with **verifier-derived** fields (mutation kill-rate, progress score, tri-layer tag). A Promoted row that carries only a neural score **fails the done-gate** (`:79-81,93-100`).
7. **Merge (J-11 S-2, this organ’s merge step — not I-17’s ranker).** Winner merge runs **B_test in a fresh `git worktree add --detach` worktree from a SEPARATE pool**. Executor never sees B_test. LLM-reported scores are logged and **never trusted**. No improvement vs trunk → reject. Merge `--no-ff`; conflict → abort+restore. Eval commands live in the **frozen gate-manifest**, never agent-writable meta (`:127-150`).
8. **Abstain / fail-closed.** Every candidate vetoed → **no Promoted row**, emit **BudgetCollapsed**, caller keeps linear retry (`I-plan/I-SUMMARY_INDEX.md:105-107`; live `SearchLadderOutcome.winner = None` at `ladder.rs:240-248`). Missing held-out eval cmd = **BLOCK**, never fall back to LLM `test_score` (Arbor F/O-1 refused) (`:144-146`). Score parse is **deterministic only** (Arbor F/O-3 refused) (`:151-154`).
9. **v2 / v3.** Tournament among ranked survivors; then PUCT + virtual loss on SessionDag (H-49). **Not present in v1 binary** (`rg mcts|tournament` on the v1 artifact must be 0 until each gate opens) (`:50-51,99-101`).
10. **The model may generate** candidate source and (advisory) SELECT diversity text (`:180-183`). It may **not** choose the winner, un-veto, rewrite eval cmds, parse its own score as evidence, or expand via Batch API on a live loop (IC-1; batch is I-7 offline only) (`:19-20,84`).

## 4. Types, enums, objects

Copy from the plan’s wiring contract plus live DTOs that name the same organ. Do not invent fields.

```
SearchLadder::run_best_of_n(...) -> SearchLadderOutcome
  // v1 entry; orchestrator-owned; no LLM inside the ranker

SearchNodeEvent.Kind {
  Expanded,          // candidate generated + (pristine context +) tri-layer run
  Pruned,            // HardVeto or flake-quarantine; TERMINAL
  Promoted,          // best-of-N winner; MUST carry verifier receipts
  BudgetCollapsed,   // no survivor; honest no-winner beacon
}

// Live payload (SCHEMA 261) — named evidence fields the Promoted row must carry
SearchNodeEventPayload {
  kind: SearchNodeEventKind,
  branch_id: String,
  tri_layer_verdict: String,   // e.g. hard_veto | scored_survivor; empty on collapse
  tri_layer_score: f64,        // [0,1]; literal 0.0 on veto
  mutation_kill_rate: f64,     // I-20 Wilson-CI point; 0.0 if no receipt
  progress_score: f64,         // [0,1] rank-3
  ts_ms: i64,
}

// Live DTOs that implement the plan ranker (do not rebuild a cousin)
BranchInput { branch: Branch, mutation_outcomes: Vec<bool>, progress_score: f64 }
RankedBranch { verdict: VerdictRow, mutation_kill_rate: f64, progress_score: f64 }
PrunedBranch { verdict: VerdictRow, progress_score: f64 }   // progress preserved for I-28 DPO
SearchLadderOutcome {
  winner: Option<RankedBranch>,
  pruned_count: usize,
  pruned: Vec<PrunedBranch>,
  escalated: bool,             // I-17 L3 over the survivor SET
  ranked: Vec<RankedBranch>,   // best-first
}
SearchLadderError { PristineContext { branch_id, source }, Verifier(...) }

fn search_ladder_enabled() -> bool
  // truthy: 1 | true | yes | on (case-insensitive); else false
ELAI_SEARCH_LADDER_ENABLE_ENV = "ELAI_SEARCH_LADDER_ENABLE"
BEST_OF_N_DEFAULT_WIDTH: usize = 4   // live [DESIGN] ID-5; NOT a measured gate
```

**Plan ranking (authoritative):** “deterministic tri-layer verdict first, mutation kill-rate second, progress score third” (`I-plan/I-4_evidence_ranked_search_ladder.md:47`). Live `rank_survivors` is desc `verdict.score`, then kill-rate, then progress, then **branch_id** (`ladder.rs:277-291`). The branch_id last key is a **live fill** for GD-2 stability; the plan does not name it.

**I-17 `rank_candidates`:** I-17’s wiring contract names `ladder.rs::rank_candidates` as the production caller of `run_verdict` (`I-plan/I-17_tri_layer_evaluation_stack.md:67`). Live method is `rank_survivors` (private) inside `run_best_of_n`. Rebuild: **a host ranker that only sees non-vetoed rows**. Do not require the identifier `rank_candidates`.

**v2/v3 types:** plan names `search/tournament.rs` and `search/mcts.rs` + PUCT + virtual loss (`:50-51`). Field-level PUCT structs are **not** in I-4. Do not invent `c_puct`.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `SearchLadder`. Generator proposes K candidates; engine ranks, prunes, promotes, merges. No token-level MCTS in v1 (`:16-17,46-47`).

**v1 lifecycle (must be unit-tested):**

| From | Event | To |
|---|---|---|
| (candidate in) | I-5 context + I-17 verdict | Expanded |
| Expanded | HardVeto **or** flake-quarantine | Pruned (terminal) |
| Expanded | survivor | ranked set |
| ranked set | lexicographic receipts | Promoted (exactly one) |
| all pruned | no survivor | BudgetCollapsed; linear retry remains |
| Promoted | J-11 held-out B_test in detached worktree | merge `--no-ff` **or** reject |

**Forced transitions:**

- HardVeto → Pruned. No L2/L3/neural un-veto (`:81-83`).
- Flag off / no measured win → **do not** replace linear retry (`:45-46`).
- v2 blocked until v1 pass-rate delta on a real held-out fixture (`:50`).
- v3 blocked until v2 delta; never blocks Tracks 14–15 (`:51`).
- Live interactive path: Batch API **structurally unreachable** (I-7 / IC-1) (`:84`).

**Worktree lifecycle (J-11 S-3, compose I-2 pool — do not restate I-2):** start-point = **current trunk HEAD**; finalize-commit **filters generated artifacts**; worktree **removed**, **branch preserved** for merge; merge-eval in a **separate detached pool**; stale worktrees force-removed at create (`:156-173`). Arbor’s Unix-only non-pooled worktrees are **not** the I-2 implementation (`:171-173`).

## 6. Grounding, receipts, gates

This organ **selects by receipts**. It does not re-implement I-17 checkers, I-20 mutation, or F-4 cant-lie.

**Selector hierarchy (search, not GroundingLevel):**

| Rank | Signal | May promote? |
|---|---|---|
| veto | I-17 L1 HardVeto / flake quarantine | **never** (Pruned) |
| 1 | tri-layer survivor score | yes, among survivors |
| 2 | I-20 mutation kill-rate (Wilson-CI point) | tie-break |
| 3 | progress score (deterministic φ; I-22 owns the scorer) | tie-break |
| advisory | C-26 SELECT prompt / neural order | **order survivors only** |

**Never allowed as a confidence signal / winner chooser:** LLM-judge-only selection; LLM-reported `test_score` when eval cmd is missing; LLM parse of eval stdout; PPM / L2 score resurrecting a veto; window fill; “the model preferred branch 2”; agent-writable `eval_cmd_test` in tree meta (`:79-86,144-154`).

**Held-out merge gate (this organ, J-11):** B_dev is for iteration; **B_test is gate-owned** in a detached worktree, off-agent. Frozen gate-manifest owns eval cmds (R8). Absent cmd = BLOCK (R1) (`:139-150`).

**I-5:** outbound per-branch payload contains **zero bytes** of prior traceback; ≤30-token contrastive constraint is I-5’s job (`I-plan/I-SUMMARY_INDEX.md:121-124`). I-4 must **call** that filter, not inline a second mask.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| v1 width K | plan fixture fans **3** candidates (`I-plan/I-SUMMARY_INDEX.md:105`); live `BEST_OF_N_DEFAULT_WIDTH = 4` labelled `[DESIGN]` (`ladder.rs:25-30`) | per run |
| I-17 halt | 15s empirical bound (shared with I-8; do not duplicate timer) (`I-plan/I-17_tri_layer_evaluation_stack.md:12-13`) | per deterministic check |
| Promoted rows | exactly one per successful best-of-N; zero on collapse | per run |
| GD-4 promotion | measured win on **≥3 real runs** before replacing linear retry **and** before v2/v3 (`:46,87`) | organ |
| SSB bump (plan) | `SearchNodeEvent` baseline+1 from LIVE 221/532 as of 2026-06-10 (`:114-115`) | plan-time |
| Live schema | SCHEMA_VERSION **261** (`ssb/payloads.rs:17771-17774`) | implementation |
| Branch context window (live fill) | `DEFAULT_BRANCH_CONTEXT_WINDOW_TOKENS = 32_000` (`ladder.rs:32-36`) | per branch rebuild |

**Overflow / collapse:** all vetoed → `BudgetCollapsed`, `winner=None`, linear retry. Plan does **not** name a token cap for the ladder itself (that is I-26 / E-39). Do not treat context-window size as the search budget (H-42 / RD-4 neighbor).

**K is not a measured gate.** ID-5: do not freeze 3 or 4 as an acceptance threshold (`I-plan/I-0_INDEX.md:240-244`, `ladder.rs:25-30`).

## 8. Roles and neighbors

I-4 is **role-agnostic at selection**. Generator (any coding role) proposes; orchestrator ranks.

**Consumes:** I-2 `BranchWorkspace::spawn` (isolation); I-17 `TriLayerStack`; I-20 mutation receipts; H-49 `SessionDag::fork_branch` (branch store — **v3 tree**, and merge/undo; v1 must still not share scratch); I-5 `pristine_path_filter`; IS-6 `update_breakpoints`; IS-14 `build_tool_catalog` 3-arm dispatch **shape** (`I-plan/I-4_evidence_ranked_search_ladder.md:69-75`); C-26 temps; D-3 adapters **via** I-17 (do not restack).

**Produces:** ranked winner + `SearchNodeEvent` rows; merge of the winner after held-out eval. I-22 `search/score.rs` **feeds** rank-3 / PUCT value (`I-plan/I-22_progress_shaped_rewards.md:45-59`) — I-4 does not re-derive φ. I-28 reads `pruned` + preserved `progress_score` for DPO negatives (`I-plan/I-28_dag_credit_assignment.md:91`). I-7 escalates local failure via I-11 BeliefState — **does not** Batch-API expand this ladder (`I-plan/I-SUMMARY_INDEX.md:141-154`). I-8 abort / dead-end hash prunes isomorphic retries **before** an LLM call (`I-plan/I-SUMMARY_INDEX.md:156-169`). J-86 later **activates** this ladder as GRC-v3’s receipt-gated candidate path (`ELAI-FEATURE-MAP.md:835-839` neighbor; do not author J-86 here).

**Must not fork:** a second selector inside the agent; LLM-judge-only arm as the production path; I-2 isolation “optional because v1 is stateless”; a cousin MCTS that ships before v1 measured win; Arbor fail-opens (LLM score fallback, mutable eval cmds, LLM score parse).

**Live v1 fill (do not rebuild as a license to skip ID-2):** `ladder.rs` ranks **statelessly** and “does NOT touch the H-49 SessionDag (FORK-B — the DAG tree store enters at v3)” (`search/mod.rs:43-47`). Isolation remains **mandatory** for parallel candidate execution (`:85-86`). A rebuild that runs K generators on one mutable dir is a cousin.

## 9. Config / flags / defaults

- Original plan observable trigger: `elai run --task fixtures/search/simple_fix.toml --search-ladder` (`:94`).
- Live / FEATURE-MAP: **`ELAI_SEARCH_LADDER_ENABLE`** default-OFF, R9 experiment, promotion `I-4-promotion`, expiry in allowlist (`ELAI-FEATURE-MAP.md:835`, `wire-dark-allowlist.toml:1829-1850`). Truthy literals: `1` / `true` / `yes` / `on` (`ladder.rs:396-408`).
- Two-gate no-op (live fill): also requires plumbed `checker_runtime` (I-17 L1 registry) (`wire-dark-allowlist.toml:1835-1837`).
- Linear retry **stays** until measured win on ≥3 real runs (GD-4) (`:45-46`).
- Decision doc: `workflow/reasoning/I-4-decisions.md` (`:48`).
- Frontend: H-15 branch board consumes `SearchNodeEvent`; **v1 may ship without the Svelte surface** (W-6) (`:89-92`).
- No batch-API flag on the live loop (`:84`).

Rebuild the organ as **gated best-of-N with linear fallback**. An always-on ladder that deletes linear retry before the ≥3-run measurement is a cousin.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/search/ladder.rs::SearchLadder::run_best_of_n` | `I-plan/I-4_evidence_ranked_search_ladder.md:66` |
| `crates/orchestrator/src/search/mod.rs` public re-export | `:67` |
| `search/tournament.rs` (v2, gated) | `:50` |
| `search/mcts.rs` + PUCT + virtual loss (v3, gated) | `:51` |
| `SearchNodeEvent` + `action_type_lock.rs` | `:68,114-115` |
| `task_runner` baby-step / `dispatch_search_ladder` | `:44-46,76-77`; live `task_runner.rs:14060+` |
| `I-2 BranchWorkspace::spawn` | `:70` |
| `I-17 TriLayerStack::evaluate` / `run_verdict` | `:71`; I-17 `:56,67` |
| `I-20 MutationScheduler::score` | `:72` |
| `session_dag.rs::SessionDag::fork_branch` | `:73` |
| `context_budget.rs::update_breakpoints` | `:74` |
| `production_agent_rpc.rs::build_tool_catalog` (3-arm **shape**) | `:75` |
| `crates/contracts/tests/i4_search_ladder_contract.rs` | live tests A–F |
| `workflow/reasoning/I-4-decisions.md` | `:48` |
| Arbor prior art (cite, do not copy fail-opens) | `src/coordinator/tools/git_ops.py:282-520` as cited in `:128` |

**Live neighbors (implementation truth, not a second organ):** `search/mod.rs`, `search/ladder.rs`, `search/score.rs` (I-22), `ssb/payloads.rs:17771-17852`, `task_runner.rs:14060-14170`, `wire-dark-allowlist.toml` flag `ELAI_SEARCH_LADDER_ENABLE`.

## 11. Journal / SSB events

Plan: **+SearchNodeEvent** Kind `{Expanded, Pruned, Promoted, BudgetCollapsed}` (`:68,114-115`).

Live SCHEMA **261** payload: `SearchNodeEventPayload` in §4. Wire `sql_str`:

```
search_node_expanded
search_node_pruned
search_node_promoted
search_node_budget_collapsed
```

(`ssb/payloads.rs:17800-17810`). Round-trip required (`i4_search_ladder_contract.rs` arm F).

**Live emit policy (fill):** SSB write is **best-effort** — write failure logs and **does not abort ranking** (`ladder.rs:343-388`). Plan is silent. Rebuild: **ranking is load-bearing**; treating a missing Promoted row as success still **fails R1** on the contract tests (query_by_type against real sqlite) (`i4_search_ladder_contract.rs:3-12`).

H-15 comparison view consumes these rows (`:89-92`). Do not invent extra action_types for v1.

## 12. Tests and observables

Named requirements (`I-plan/I-4_evidence_ranked_search_ladder.md:79-101`, `I-plan/I-SUMMARY_INDEX.md:105-107`, live contract A–F):

| Id | Assert |
|---|---|
| production dispatch | gated run lands ≥1 Expanded, ≥1 Promoted with verifier fields, ≥1 Pruned on a compile-fail / vetoed candidate |
| rank-2 | equal tri-layer score → higher mutation kill-rate wins (`i4_a`) |
| rank-1 | tri-layer score outranks kill-rate (`i4_b` intent; plan: verdict first) |
| ID-4 veto | HardVeto → Pruned, **zero** Promoted for that branch; score 0.0 (`i4_c`) |
| quarantine | flake-quarantined oracle dropped even if L1 would pass (`i4_c2`) |
| GD-4 default-OFF | `search_ladder_enabled()` false when env unset (`i4_d`) |
| collapse | all vetoed → BudgetCollapsed, winner None, zero Promoted (`i4_e`) |
| env literal | `ELAI_SEARCH_LADDER_ENABLE` matches allowlist (`i4_f`) |
| `--llm-judge-only` broken arm | picks vetoed/compile-fail branch → **non-zero exit**, **zero** Promoted (R2/ID-4) (`:99-100`) |
| v2/v3 absent | `rg 'mcts\|tournament' target/` = 0 until gates open (`:100-101`) |
| merge F/O-1 | missing held-out cmd **BLOCK**, never LLM `test_score` (`:144-146`) |
| merge F/O-2 | eval cmds not agent-writable (`:147-150`) |
| score parse | deterministic parser only (`:151-154`) |
| GD-2 | Promoted ranking recomputable under `ELAI_SEED` (`:97`) |

Trigger (plan): `cargo run --bin elai -- run --task fixtures/search/simple_fix.toml --search-ladder` (`:94`). Live smoke: `i4_a_production_dispatch_search_node_rows_land` (`wire-dark-allowlist.toml:1850`).

Adversarial: Promoted without kill-rate / tri-layer tag; un-veto via L2; shared scratch across K; Batch API on interactive; Arbor LLM fallback; shipping `mcts.rs` in the v1 crate.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Orchestrator-owned `run_best_of_n` (or identical host entry) ranks K isolated candidates **with no LLM call inside the ranker**.
- Closed `SearchNodeEvent` kinds `{Expanded, Pruned, Promoted, BudgetCollapsed}`.
- Ranking key is **tri-layer receipt, then mutation kill-rate, then progress**. Neural score cannot promote a vetoed branch.
- Every Promoted row carries verifier-derived evidence (tri-layer tag + real kill-rate / progress). LLM-judge-only is a **failing broken arm**, not a mode.
- HardVeto / flake-quarantine is **terminal Pruned**.
- All-vetoed emits BudgetCollapsed and **does not** invent a winner.
- Parallel candidates **do not share mutable scratch** (ID-2 / I-2).
- v1 ships **before** tournament/PUCT; v2/v3 absent until measured v1 win on ≥3 real runs.
- Winner merge (when merge is in scope) re-runs held-out eval in a **detached worktree from a separate pool**; missing eval cmd BLOCKs; eval cmds are frozen-manifest, not LLM-writable; score parse is deterministic.
- Default-OFF / linear-retry fallback until GD-4 measurement.
- Batch API is **not** on the live expansion path.

A prompt “pick the best of these diffs”, an MCTS-first tree, or Arbor’s LLM `test_score` fallback is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- LLM-judge-only selector (`:18-19,79-81`).
- Resurrecting HardVeto (`:81-83`; I-17 `:21-22`).
- Shipping v3 tree code before v1 measured win (`:20-21,87`).
- Batch-API parallel expansion on live loops (`:19-20,84`) — offline only in I-7.
- Shared mutable scratch (`:85-86`).
- Absorbing I-17 / I-20 / I-2 / H-49 / I-5 into `ladder.rs` (GD-1 compose) (`:119-125`).
- Arbor fail-opens: LLM score when eval unset; LLM-writable eval cmds; LLM score extraction (`:143-154`).
- Persistent-ReAct 500-turn loop, premature-stop nudges, LLM-chosen node selection (`:177-179`).
- Cross-language verifier packs (I-17 Tier-2; not v1) (`:52`).
- Treating window size as the search budget (H-42 / I-26 / RD-4).
- Fabricated measurements, blake2/sha2, `.unwrap()` on production paths (`:88`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-4_evidence_ranked_search_ladder.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:91-107`. Doctrine: `I-plan/I-0_INDEX.md:221-239` (ID-1/ID-2/ID-4). Neighbors: I-17 (`I-plan/I-17_tri_layer_evaluation_stack.md`), I-20, I-22 (`search/score.rs`), I-5 / I-7 / I-8 summaries in `I-SUMMARY_INDEX.md:110-169`. Feature map: `ELAI-FEATURE-MAP.md:835-839`, `:3433-3438`. Live (implementation truth, not a second spec): `crates/orchestrator/src/search/{mod,ladder}.rs`, `ssb/payloads.rs:17771-17852`, `task_runner.rs:14060+`, `crates/contracts/tests/i4_search_ladder_contract.rs`, `wire-dark-allowlist.toml:1829-1850`. Arbor citations stay inside the I-4 plan (`git_ops.py:282-520` as quoted there).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact default K: plan observable uses 3; live `[DESIGN]` width is 4; neither is a measured gate (`I-SUMMARY_INDEX.md:105` vs `ladder.rs:25-30`).
- Whether `run_best_of_n` **spawns** I-2 workspaces internally or the caller binds them. Plan wiring lists `BranchWorkspace::spawn` as a consume (`:70`); live v1 ranks statelessly (`search/mod.rs:43-47`).
- Whether v1 **must** emit Expanded **before** prune, or prune without Expanded. Live emits Pruned **instead of** Expanded on veto (`ladder.rs:203-210`) — plan Kind list does not order them.
- J-11 held-out merge: specified in I-4, **not** implemented in live `ladder.rs` (rank + SSB only). Rebuild-now vs defer-to-promotion is unspecified.
- `rank_candidates` (I-17 name) vs `rank_survivors` (live).
- `--search-ladder` CLI vs `ELAI_SEARCH_LADDER_ENABLE` only (`:94` vs FEATURE-MAP `:835`).
- Progress-score producer for v1 before I-22: plan rank-3 exists; I-22 depends **on** I-4 (`I-22:3`). Who writes `BranchInput.progress_score` in v1 is unspecified.
- PUCT hyperparameters, virtual-loss formula, tournament bracket shape (`:16,50-51`).
- Whether BudgetCollapsed consumes I-26 budget or is only the no-survivor beacon (name collision with I-26 collapse).
- SSB emit best-effort vs fail-closed if the Promoted write fails (`ladder.rs:343-388` vs R1 missing-row FAIL).
- `escalated: bool` on `SearchLadderOutcome` — live L3 over survivor set; plan v1 text does not name L3 inside I-4 (I-17 owns it).
- Exact baby-step dispatch line in `task_runner.rs` (plan: PLAN-asserted, re-grep W-7) (`:56-57`).
