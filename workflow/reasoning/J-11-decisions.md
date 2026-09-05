# J-11 Decision Document — Source-Cited Amendment Injection

**Date:** 2026-06-26
**Task:** Insert source-cited amendment blocks into 10 existing plan files, grounding each in directly-read source code from three external projects (Arbor, ComBench, DeNovoSWE).
**Scope:** Documentation-only. No runtime code, no gate files, no test files. Only plan files in `docs/thefinalglance/` + this decision doc.

## What

Ten plan files received `## J-11 AMENDMENT —` blocks at their tail (before the "End of" marker or at file end). Each block cites specific file:line ranges from the source project, read directly per W-3 (grounded-by-default). The amendments are prior-art pointers (GD-1 compose-not-restate), DO-NOT-COPY warnings, and feature specifications — they do NOT change the plans' scope, allowed_write_set, or acceptance criteria.

## Key Decision

All amendments are **advisory context for the executor**, not binding architecture changes. The plans' existing scope, wiring contracts, and acceptance criteria remain authoritative. The amendments add source-cited prior-art grounding that the executor reads before implementation, following GD-1 (compose-not-restate) and the grounded-by-default hard rule.

## Source Verification

### Arbor (arXiv:2606.11926)
Source path: `F:\.ELAI_workflow\docs\.source\2606-11926-arbor-hypothesis-tree-refinement\`
Files read directly (W-3):
- `src/coordinator/tools/git_ops.py` — GitMergeBranchTool (merge gate, eval_cmd_test, score verification, worktree pool)
- `src/coordinator/tools/executor_run.py` — worktree lifecycle (create, finalize, remove, branch preserve)
- `src/coordinator/tools/tree_ops.py` — TreeSetMetaTool (LLM-writable eval cmds — F/O-2)
- `src/core/git_artifacts.py` — filter_commit_paths (artifact filtering)
- `src/coordinator/prompts.py` — SELECT guidance, Evaluation Discipline (B_dev/B_test)
- `src/coordinator/context.py` — ContextManager (token estimation, pruning tiers)
- `src/coordinator/context_prune.py` — ContextPruner (three-tier observation pruning)

### ComBench (arXiv:2606.10479)
Source path: `F:\.ELAI_workflow\docs\.source\2606-10479-combench\repo\`
Files read directly (W-3):
- `src/eval/prime_code_executor.py` — run_prime_code_verifier (minimal verifier contract)
- `src/eval/extractor.py` — prepare_response, extract_construction (response parsing)
- `src/eval/score_parsing.py` — parse_answer_score, parse_proof_score (deterministic score extraction)
- `src/eval/evaluator.py` — _build_case_result (averaging flaw — M-2 demotion clause)
- `data_process/check_ref_construction.py` — _check_record (register-time reference-fixture gate)
- `src/prime_code/testing_util.py` — reliability_guard (NOT a sandbox — DO-NOT-COPY)

### DeNovoSWE (arXiv:2606.10728)
Source path: `F:\.ELAI_workflow\docs\.source\2606-10728-denovoswe\2606.10728.html`
Sections read directly (W-3):
- §3.3 (S3.SS3) — Evaluation Protocol and Leakage Prevention
  - S3.I1.i2.p1.1 — Multi-channel Leak Purging (Mode 2)
  - S3.I1.i3.p1.1 — Git history sanitization (Mode 1)
  - S3.SS3.p3.1 — Runtime cheating prevention (Mode 3 + Mode 4)
- §4.1 (S4.SS1) — Difficulty Scoring Framework
  - S4.E4 (Eq.4) — convex combination fusion formula
  - S4.E5 (Eq.5) — weight optimization on 2-simplex
  - S4.SS1.SSS0.Px1 — three signals (structural, LLM g, LLM q)
  - S4.SS1.SSS0.Px4 — Pearson correlation optimization, 2D grid sweep
- §4.2 (S4.SS2) — Filtering Thresholds
  - S4.T2 (Table 2) — five-band threshold table (0.90/0.85/0.80/0.70/0.60)
- §5.3 (S5.SS3) — Ablation Study
  - S5.SS3.p3.1 — +1.2pp gain (0.488→0.500), stricter easy tier is the driver

## Amendment Map

| Plan File | Steal IDs | Source | Key Content |
|-----------|-----------|--------|-------------|
| I-4 | S-2, S-3, S-4 + 2 F/O | Arbor git_ops.py, executor_run.py, prompts.py, tree_ops.py | Held-out merge gate mechanics, worktree lifecycle, prompt content, two DO-NOT-COPY fail-opens |
| I-2 | S-3 | Arbor executor_run.py, git_artifacts.py, git_ops.py | Worktree lifecycle prior-art (5 mechanics) |
| I-17 | S-2 | Arbor git_ops.py | L1 veto supremacy pointer |
| I-25 | S-8b | DeNovoSWE §4.1 | Difficulty fusion as router feature (Eq.4-5, three signals) |
| I-30 | S-8a | DeNovoSWE §4.2, §5.3 | Difficulty-banded inclusion floor + partial-over-trivial |
| D-6 | S-2, S-8c | Arbor git_ops.py, DeNovoSWE §3.3 | B_dev/B_test temporal split + .git re-init store hygiene |
| D-11 | S-7 | ComBench prime_code_executor.py, evaluator.py, check_ref_construction.py, testing_util.py | Checker contract + register-time gate + demotion clause + DO-NOT-COPY reliability_guard |
| H-43 | S-2 | Arbor git_ops.py | GD-1 prior-art pointer (verified eval score beats LLM-reported) |
| H-47 | S-8c | DeNovoSWE §3.3 | Four leakage bypass modes + two-layer doctrine |
| H-50 | S-5 | Arbor context.py, context_prune.py | Three-tier ContextManager layers (token-efficiency) |

## Expected Test Outcome

No test outcome — this is a documentation-only task. The amendments are text blocks in gitignored plan files. The deterministic check is:
1. grep for "J-11 AMENDMENT" in each of the 10 files → 1+ hit each
2. grep for "TODO\|pass\|placeholder" in the amendment blocks → 0 hits (zero lazy code)
3. gate_freeze_check — N/A (no gate files touched)

## Unknowns

- The steal-list index (S-2..S-8, M-1..M-4, HANDOFF, WHAT ARBOR LACKS) was not found as a standalone file in `workflow/papers/`. The task's detailed per-file descriptions contained the necessary citation mapping, so the amendments were authored from those descriptions + direct source reads. If a standalone steal-list index exists elsewhere, it should be cross-referenced.
- The Arbor `skills/idea_drafting.md` file was referenced in the I-4 S-4 amendment but not read line-by-line (the IDEATE method structure was confirmed via the prompts.py cross-reference). If the executor needs the exact ~245-line content, they should read it directly from the source path.
- DeNovoSWE's winning fusion weights (w_s*, w_g*, w_q*) are NOT published in the paper — ELAI must calibrate on its own rollouts. The +1.2pp figure is design rationale, never an ELAI target (HD-1/ID-5).

## Files Touched

1. `docs/thefinalglance/I-plan/I-4_evidence_ranked_search_ladder.md` — +3 amendment blocks (S-2, S-3, S-4)
2. `docs/thefinalglance/I-plan/I-2_cow_workspace_isolation.md` — +1 amendment block (S-3)
3. `docs/thefinalglance/I-plan/I-17_tri_layer_evaluation_stack.md` — +1 amendment block (S-2)
4. `docs/thefinalglance/I-plan/I-25_ledger_trained_router.md` — +1 amendment block (S-8b)
5. `docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md` — +1 amendment block (S-8a)
6. `docs/thefinalglance/D-plan/D-6_held_out_task_store.md` — +2 amendment blocks (S-2, S-8c)
7. `docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md` — +1 amendment block (S-7)
8. `docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md` — +1 amendment block (S-2)
9. `docs/thefinalglance/H-plan/H-47_dgm_eval_firewall_dategate_templateregen.md` — +1 amendment block (S-8c)
10. `docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md` — +1 amendment block (S-5)
11. `workflow/reasoning/J-11-decisions.md` — this file (NEW)
