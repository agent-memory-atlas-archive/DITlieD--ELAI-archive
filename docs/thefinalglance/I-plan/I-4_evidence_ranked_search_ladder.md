# I-4: evidence-ranked search ladder — receipts replace LLM-judge selection, orchestrator owns the tree

- Track 13 — EVIDENCE-RANKED SEARCH | lever SMARTER-ORCH | grounding STRONG | nature BUILD | effort XL | WAVE 4 | depends I-2 + I-17 | BLOCKING for I-22/I-28/I-30 | binds ID-1/ID-2/ID-4

- Premise: nothing public ships tree search — what ships is wide parallelism plus an LLM-judge selector.
  ELAI's verifier stack lets selection be BY RECEIPTS instead (IC-6). The LLM is demoted to a stateless
  hypothesis generator; the orchestrator owns the state tree; failed branches are pruned from context
  (the vividness trap is real and must be MEASURED before it is claimed — I-5 instruments it). The MCTS XL
  is a tiered ladder so every rung ships value independently and does not block Tracks 14-15.

## Steal/build
IC-6 replaces the original consolidated MCTS-first design with a three-rung ladder: v1 best-of-N — K
divergent candidates in isolated workspaces (I-2) each processed by the tri-layer stack (I-17), budgeted
mutation (I-20), and progress scores; the orchestrator RANKS by receipts and the SSB records WHY each branch
won or lost. v2 extends to tournament brackets among ranked survivors. v3 adds a PUCT tree over SessionDag
(H-49 branch store) with virtual loss for safe parallel expansion. The generator proposes K candidates via
C-26 temperature schedules; it never owns the state tree. Selection is purely evidence-ranked per ID-4 — a
neural scorer may ORDER surviving branches, never RESURRECT a vetoed one (ID-4 hard rule). EXCLUDED: an
LLM-judge-only selector that ignores verifier verdicts (banned explicitly, stub_ban below); batch-API-based
parallel expansion (impossible on live loops — IC-1; confined to offline lanes in I-7). DECLINED: shipping
v3 tree code before v1 best-of-N has demonstrated measured win (GD-4 sequencing rule). FULL-FORM EXPANSION
REQUIRED before EXECUTING (I-0 reading order rule).

## Existing ground truth
- IN-1: no mcts*.rs module under orchestrator/src; "mcts" text hits are comments and AB-MCTS config keys.
  I-4 is a genuine BUILD — confirmed absent this grounding session.
- IS-6: context_budget.rs ContextSlot :1079 (nine slots), update_breakpoints :1507 (blake3 prefix stability),
  adaptive_reduce :1551 — the branch context seams I-5 must preserve when rebuilding per-branch prompts.
- IS-8: metamorphic_testing.rs RelationLibrary :181 + mutation_testing.rs MutationRunner :354 +
  EquivalentMutantCache (blake3-keyed, 30-day TTL) — the verifier components I-17 stacks and I-20 extends;
  I-4 consumes their verdicts as receipts.
- IS-14: production_agent_rpc.rs build_tool_catalog :657 (the 3-arm convert/dispatch pattern) — the
  dispatch seam the ladder hooks into at the baby-step dispatch site.
- IS-9: bsp/engine.rs recover_from_checkpoint :958 (EXISTS with test :1242; IC-15 corrects the stale
  "stub" framing) — the resume primitive the branch store inherits.
- ID-2 (I-0): parallel branches must never share mutable scratch space; I-4 depends on I-2's workspaces
  for isolation correctness.
- SessionDag and H-49 undo-tree: PLAN-asserted; executor re-greps per W-7 (H-49 is a Wave-2 plan file).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/search/ladder.rs — best-of-N dispatcher (K candidates, I-2 workspace spawn per
  branch, collect receipts from I-17 stack, rank, emit SearchNodeEvent rows, return winner)
- SearchNodeEvent action_type registration (ssb.rs + action_type_lock.rs, baseline+1)
- Stub-safe seam at the task_runner baby-step dispatch replacing the linear retry ONLY behind the
  ladder's measured first-win (GD-4; linear retry stays until the ladder proves itself on >=3 real runs)
- v1 ranking: deterministic tri-layer verdict first, mutation kill-rate second, progress score third
- decision doc workflow/reasoning/I-4-decisions.md
DEFER (Tier-2 / gated):
- search/tournament.rs (v2) — gated behind ladder v1 measured pass-rate delta on a real held-out fixture
- search/mcts.rs + PUCT + virtual loss (v3) — gated behind v2 delta; never blocks Track 14/15
- cross-language verifier pack (mentioned in I-17 as Tier-2; not part of v1)

## Integration
New modules: crates/orchestrator/src/search/ladder.rs, crates/orchestrator/src/search/mod.rs.
Existing seams: task_runner baby-step dispatch (the linear retry site — PLAN-asserted; executor re-greps per
W-7); SessionDag (H-49 branch store — compose, never restate); IS-14 build_tool_catalog dispatch pattern
(the 3-arm shape ladder.rs mirrors); IS-8 MutationRunner verdicts consumed as receipts; IS-6 context_budget
branch-rebuild path (I-5 owns the pristine filter; I-4 calls it per branch).
frontend_pair: branch board — the H15-1 comparison view is the intended consumer of SearchNodeEvent rows
(PLAN-asserted for H-15; executor re-greps per W-7; cross_lang_diff.py PASS required when the Svelte surface
lands).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/search/ladder.rs::SearchLadder::run_best_of_n (the v1 entry point)
  - crates/orchestrator/src/search/mod.rs (public re-export)
  - SearchNodeEvent (ssb.rs + action_type_lock.rs — Kind: Expanded/Pruned/Promoted/BudgetCollapsed)
consumes_symbols:
  - crates/orchestrator/src/search/ladder.rs calls I-2 BranchWorkspace::spawn — GD-1 (workspace isolation)
  - crates/orchestrator/src/search/ladder.rs calls I-17 TriLayerStack::evaluate — GD-1 (the ranker)
  - crates/orchestrator/src/search/ladder.rs calls I-20 MutationScheduler::score — GD-1 (mutation receipts)
  - session_dag.rs::SessionDag::fork_branch — GD-1 (branch store, H-49)
  - context_budget.rs::update_breakpoints — GD-1 IS-6 (bp stability preserved across branch rebuild)
  - production_agent_rpc.rs::build_tool_catalog — GD-1 IS-14 (the 3-arm dispatch pattern mirrored)
reachable_from:
  - task_runner.rs baby-step dispatch site -> SearchLadder::run_best_of_n  [PRODUCTION task-runner]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO LLM-judge-only selection: a branch ranked SOLELY by neural score with no verifier receipt FAILS the
    done-gate (ID-4 hard rule; the SSB row for every Promoted branch must carry verifier-derived evidence)
  - NO resurrecting a branch the tri-layer stack vetoed (HardVeto is terminal — ID-4; the ladder may not
    un-veto a branch regardless of neural score)
  - NO batch-API parallel expansion on any live search loop path (IC-1; Batch API offline-only)
  - NO sharing mutable scratch between parallel branches (ID-2; I-2 workspace isolation is mandatory,
    not optional)
  - NO shipping v2/v3 code before v1 measured win on >=3 real runs (GD-4; premature tree code is banned)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated measurements
frontend_pair:
  H-15 branch-board Svelte surface (comparison view consuming SearchNodeEvent rows) — PLAN-asserted for H-15;
  cross_lang_diff.py PASS required when the dashboard surface ships (W-4). Justified: the Svelte surface is
  deferred to H-15; v1 ladder ships without it (W-6 justified for v1).
observable_effect:
  trigger: "cargo run --bin elai -- run --task fixtures/search/simple_fix.toml --search-ladder"
  expect_ssb: "SearchNodeEvent rows land: at least one Pruned (the compile-fail branch), at least one
    Promoted carrying verifier receipt fields (mutation kill-rate, progress score) — Kind discriminators
    Expanded/Pruned/Promoted present; recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "the winning branch's merge commit is journal-verified as the highest-receipt ranked
    survivor; an --llm-judge-only broken arm that picks the compile-fail branch FAILS with a non-zero exit
    and zero Promoted rows (R2/ID-4); v2/v3 code absent from the binary (rg 'mcts\\|tournament' target/ MUST
    be 0 until v2 gate opens)"

## Grounding
- IC-6 (I-GROUNDING-MAP): the ladder redesign — both external reviews converge on best-of-N first; source
  Consolidated draft Part II + Review_ELAI-Competitive + Review_ELAI-Trust-Harness.
- IN-1 (I-GROUNDING-MAP): no MCTS module confirmed absent — I-4 is a genuine BUILD.
- IS-8 (I-GROUNDING-MAP): MutationRunner + EquivalentMutantCache live — consume as receipts, not rebuild.
- IS-14 (I-GROUNDING-MAP): build_tool_catalog 3-arm pattern — mirror, not reinvent.
- ID-4 (I-0_INDEX): evidence-ranked selection doctrine — neural scorer orders, never resurrects vetoed.
- ID-1 (I-0_INDEX): stateless-generator / trajectory-pruning — generator proposes, orchestrator owns state.
- ID-2 (I-0_INDEX): branch-workspace-isolation — parallel branches never share mutable scratch (depends I-2).
- VD-1 (H-0_INDEX): execution-grounded fitness binds I-4 — verifier receipts are the fitness signal.
- RV-1 (H-0_INDEX): divergence guard on branch restore semantics — I-4 composes H-49 divergence guard.
- SSB: +SearchNodeEvent (Kind: Expanded/Pruned/Promoted/BudgetCollapsed) — baseline+1 from LIVE 221/532
  as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).
- arXiv IDs for MCTS / AB-MCTS / PUCT if cited in the decision doc: WebSearch-confirm before headline
  (feedback_paper_citation_verify; no raw training-memory IDs in gate thresholds).

## COMPOSES-WITH
- SessionDag / H-49 undo-tree (the branch store): compose fork/rollback, do NOT re-implement branch history.
- I-17 tri-layer stack (Wave 3): the ranker; I-4 depends on it (I-17 must ship before I-4 executes).
- I-20 mutation scheduler (Wave 3): mutation receipts fed into the ranking; compose, not rebuild.
- I-2 branch workspace isolation (Wave 2): provides the isolated scratch dirs; I-4 depends on it.
- C-26 temperature schedules (shipped): the divergent-K generation mechanism; compose, not reinvent.
- D-3 verifier adapters (shipped): the verifier consensus trait; I-17 stacks these; I-4 consumes verdicts.

## J-11 AMENDMENT — S-2 HELD-OUT MERGE-GATE MECHANICS (source-cited)
Grounding: Arbor `src/coordinator/tools/git_ops.py:282-520` (GitMergeBranchTool), read directly per W-3.
Arbor's merge gate is a working existence proof of the held-out merge-gate I-4's search ladder needs. The gate
ITSELF re-runs the held-out eval (B_test) on the candidate branch in a fresh DETACHED worktree, off-agent
(`git_ops.py:348` execute, `:109` _run_eval_in_worktree, `:130` separate `merge-eval-worktrees-{uid}` pool,
`:145` `git worktree add --detach`). The executor only ever sees B_dev (`prompts.py:415` "evaluate on B_dev
ONLY — never on B_test"). LLM-reported scores are logged but NEVER trusted (`:407` score comparison log,
`:412` verified_score used). No improvement vs trunk/baseline => reject (`:437` is_improvement,
`:438` metric_direction-aware). Merge is `--no-ff` (`:507`); conflict => abort+restore (`:514` merge --abort,
`:517` checkout original). Protected paths fnmatch guard (`:465` plugin.protected_paths). Required outputs
checked (`:482` plugin.required_outputs, `:484` git show).

The net-new bit for I-4: the **detached-worktree held-out eval idiom** — the I-4 merge step runs B_test in a
fresh `git worktree add --detach` worktree from a SEPARATE pool, not in the executor's worktree. The executor
never touches B_test; the gate owns the eval.

### DO-NOT-COPY — two Arbor FAIL-OPENS ELAI must explicitly refuse (R1/R8)
- **F/O-1** (`git_ops.py:413`): when `eval_cmd_test` is unset, Arbor falls back to the LLM-reported
  `test_score` (`:416` "falling back to LLM-reported test_score ... NOT independently verified"). ELAI: absent
  held-out cmd = **BLOCK** (R1, missing evidence = hard FAIL). Never fall back to an LLM-reported score.
- **F/O-2** (`git_ops.py:374` + `tree_ops.py:358` TreeSetMetaTool): Arbor's eval commands live in MUTABLE
  `tree.meta`, LLM-writable via `TreeSetMeta` (`tree_ops.py:391` eval_cmd_test is a settable key). ELAI: eval
  cmds live in the **FROZEN gate-manifest**, never agent-writable state (R8). An agent that can rewrite its own
  eval command can always pass.
- **Score extraction** (`git_ops.py:62` _parse_eval_score): Arbor falls back to an LLM call
  (`:76` `provider.create(...)`) to parse the eval score when JSON parsing fails (`:190`). ELAI: deterministic
  parser ONLY (TD-3, the LLM is never the gate). A score the LLM extracted from its own eval output is not
  evidence.

## J-11 AMENDMENT — S-3 WORKTREE LIFECYCLE PRIOR ART (source-cited)
Grounding: Arbor `src/coordinator/tools/executor_run.py:116-210`, `src/core/git_artifacts.py:54`, read directly per W-3.
Five stealable mechanics for the I-4 candidate-branch lifecycle (GD-1 prior-art pointer, compose not restate):
1. Branch start-point = CURRENT trunk HEAD (`executor_run.py:136` _create_worktree, `:162` start_ref from
   trunk — picks up prior merges, not the original repo HEAD).
2. Finalize-commit filters generated artifacts out (`:187` _finalize_worktree, `:194` filter_commit_paths;
   `git_artifacts.py:54` splits commit-worthy from artifact paths via fnmatch patterns).
3. Worktree removed but BRANCH PRESERVED for later merge (`:210` _remove_worktree, `:211` "the branch is
   preserved for later merging").
4. Merge-eval runs in its own SEPARATE detached worktree pool (`git_ops.py:130`
   `merge-eval-worktrees-{uid}`, distinct from the executor pool at `executor_run.py:147`
   `coordinator-worktrees-{uid}`).
5. Stale worktrees force-removed at create (`executor_run.py:154` if exists -> `git worktree remove --force`;
   `git_ops.py:138` same in eval pool); collision => timestamp suffix (`executor_run.py:168-171`).

NOTE: Arbor has NO worktree pooling and is Unix-only (`os.getuid()` at `:147`/`git_ops.py:130`). ELAI's I-2
warm pooled Windows-safe worktrees (IC-9) are an IMPROVEMENT — keep them. This amendment grounds the
lifecycle semantics; I-2 owns the pool implementation.

## J-11 AMENDMENT — S-4 PROMPT CONTENT (advisory text, NOT architecture — source-cited)
Grounding: Arbor `src/coordinator/prompts.py:396-450`, `src/coordinator/skills/idea_drafting.md`,
`src/coordinator/executor/prompts.py`, read directly per W-3.
DO-NOT-PORT (cite M-1): the persistent-ReAct shape, the 500-turn loop, the premature-stop nudges, LLM-chosen
node selection — ELAI's FSM + deterministic leaf selection replaces all of it.
DO-PORT as advisory prompt text WITH source cite:
- `prompts.py:396` SELECT guidance ("Prefer high-impact ideas with clear evidence, implementation path, and
  recoverable failure modes; Consider diversity — don't over-commit to one direction") => I-4 ranker tie-break
  (advisory only; the deterministic scorer is authoritative).
- `prompts.py:442` Evaluation Discipline ("B_dev is for iteration ... B_test is for milestones only ... If
  B_test performance diverges significantly from B_dev, investigate") => the executor/eval prompt.
- `skills/idea_drafting.md` IDEATE method (~245 lines) => the babystep-author / scope-planner prompt material
  on the CLOUD lane ONLY (7B produces weaker hypotheses — quality lives here, mechanics don't care).
- `executor/prompts.py` UNDERSTAND->IMPLEMENT->VALIDATE->ITERATE->EVALUATE->REPORT => diff against ELAI's
  executor prompt, steal deltas only.
NOTE: the `<REDACTED_TOKEN>` wording (`idea_tree.py:397`) is already carried by J-10's CONSTRAINTS BLOCK —
cross-reference, do not duplicate.

End of I-4.
