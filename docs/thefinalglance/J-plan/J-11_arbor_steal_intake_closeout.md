# J-11: arbor / combench / denovoswe steal intake closeout — land the amendments as grounded pointer + contract insertions

STATUS: PLANNED (WAVE 2; EXTENSION — discharges the steal-list's S-2/S-3/S-4/S-5/S-7/S-8 as
amendments into existing I/H/D plan files; zero SSB, zero runtime code. The direct analog of J-7.)

Objective: the Arbor/ComBench/DeNovoSWE steal-list found ONE genuinely-new build (S-1, now J-10) and
SIX confirm-or-amend items that land as pointer lines + contract blocks into already-authored plan
files (steal-list HANDOFF #2-#6). Building parallel sessions for them would violate GD-1 (they
execute EXISTING plans). J-11 discharges them as one gated documentation unit: each target plan file
gains the named source-file pointers + the specific mechanics the steal-list verified from the repo
read, plus the two Arbor FAIL-OPENS ELAI must explicitly refuse to copy. Hard boundary: J-11 INSERTS
into the named plan files; it never re-authors them and never writes runtime code (W-3 binds every
insertion to the original source paths — a summary-only port is a violation).

Estimated effort: ~1 session (~4-6 h): ~6 amendment insertions, each a re-read of the named source
files + a contract/pointer block. No code, no SSB, no build.
Pre-requisites: none. Parallel-safe with all J sessions (pure plan-file edits on disjoint targets).
Source: ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md S-2..S-8 + the HANDOFF section + the
Arbor/ComBench source files named below (W-3: each amendment reads its source directly). Doctrine: J-0_INDEX.md.

## Context
The steal-list's grounded recommendation (3 explorer reads, one a full opus repo read): one new
session + amendments; do NOT build a parallel HTR subsystem (S-6). J-10 is the new session. J-11 is
the amendment bundle. Each item below is mostly CONFIRMATION of existing ELAI doctrine — Arbor /
ComBench / DeNovoSWE are independent existence-proofs of R1-R9 / TD / GD items ELAI already holds —
plus a small net-new mechanic per item (the detached-worktree gate idiom, the register-time
reference-fixture gate, the difficulty-banded inclusion floor). The value is GROUNDING: every
amendment cites a real repo file:line so the consuming session implements against the source, not a
paraphrase.

## Existing state (why this is EXTENSION, not BUILD)
The six targets are authored plan files (I-2, I-4, I-17, I-25, I-30, D-6, D-11, H-43, H-47, H-50).
None of them currently cite the Arbor/ComBench/DeNovoSWE source files. J-11 adds the citations + the
verified mechanics + the explicit DO-NOT-COPY fail-opens. Statuses of the targets are STALE-by-
construction in the summary indexes; verify each at edit before inserting (SRC-4 discipline).

## Reads (mandatory before each insertion — W-3: read the SOURCE, not the steal-list summary)
0. ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md in full (the index + M-1..M-4 meta-findings +
   the "WHAT ARBOR LACKS" positioning block + the HANDOFF section).
1. Arbor source (under F:\.ELAI_workflow\docs\.source\2606-11926-arbor-hypothesis-tree-refinement\):
   - src\coordinator\tools\git_ops.py:282-520  GitMergeBranchTool (the full held-out gate: :348
     execute, :366 protected branches, :109 _run_eval_in_worktree, :402 None-score reject, :437
     is_improvement, :507 merge --no-ff, :465 protected_paths, :482 required_outputs, :413 the
     F/O-1 fallback-to-LLM-score, :62 _parse_eval_score LLM fallback, :130 the separate merge-eval
     worktree pool)
   - src\coordinator\tools\executor_run.py:116-210 (_compute_branch_name / _create_worktree /
     _finalize_worktree / _remove_worktree), :432 after_executor snapshot, :248/:266 B_dev-only
     injection
   - src\coordinator\core\git_artifacts.py    filter_commit_paths (generated-artifact filtering)
   - src\coordinator\prompts.py:396 SELECT guidance / :442 Evaluation Discipline block
   - src\coordinator\skills\idea_drafting.md  the IDEATE method (~245 lines; cloud-lane only)
   - src\coordinator\executor\prompts.py      UNDERSTAND->IMPLEMENT->VALIDATE->ITERATE->EVALUATE->REPORT
   - src\coordinator\core\context.py:269 the 4-layer ContextManager / src\coordinator\context_prune.py
2. ComBench source (under F:\.ELAI_workflow\docs\.source\2606-10479-combench\repo\):
   - src\eval\evaluator.py (EvaluationSuite + 3 evaluators; :274 the averaging that PROVES demotion
     is absent — M-2), src\eval\extractor.py (<construct> extraction + independent failure semantics),
     src\eval\prime_code_executor.py (verifier invocation contract), src\eval\score_parsing.py (the
     4-string rubric parse), src\prime_code\testing_util.py:676 reliability_guard ("NOT a security
     sandbox" — DO-NOT-COPY), data_process\check_ref_construction.py (register-time validation),
     examples\example.jsonl (full record schema)
3. DeNovoSWE: docs\.source\2606-10728-denovoswe\2606.10728.html (sections 3.3 leakage, 4.1 / Eq.4-5
   difficulty fusion + the banding table; the paper IS the content — repo is README-only).
4. The interface maps workflow/papers/2606-{11926,10479,10728}-*-interface.md (orientation only).
5. Each TARGET plan file before inserting (re-read its current state; verify status at edit, SRC-4).

## Tasks (one insertion per item; each names its source file:line)
1. AMEND I-4 + H-43 + D-6 + I-17 — S-2 HELD-OUT MERGE-GATE MECHANICS. Insert into I-4's merge step:
   the gate ITSELF re-runs the held-out eval (B_test) on the candidate branch in a fresh DETACHED
   worktree, off-agent; executor only ever sees B_dev; LLM-reported scores logged but NEVER trusted;
   no improvement vs trunk/baseline (metric_direction-aware) => reject; merge --no-ff; conflict =>
   abort+restore; protected_paths fnmatch guard; required_outputs checked (cite git_ops.py:282-520
   line-by-line). MANDATORY DO-NOT-COPY block (both violate wiring-enforcement):
     - F/O-1 (git_ops.py:413): eval_cmd_test unset => Arbor falls back to the LLM-reported score.
       ELAI: absent held-out cmd = BLOCK (R1, missing evidence = hard FAIL).
     - F/O-2: Arbor's eval commands live in MUTABLE tree.meta (LLM-writable via TreeSetMeta). ELAI:
       eval cmds in the FROZEN gate-manifest, never agent-writable state (R8).
     - score extraction: Arbor falls back to an LLM call (_parse_eval_score:62). ELAI: deterministic
       parser ONLY (TD-3, the LLM is never the gate).
   Add a GD-1 prior-art pointer to H-43 (Arbor is the working existence proof its convergence claim
   cites), D-6 (B_dev/B_test split = D-6's temporal split, simpler), I-17 (L1 veto supremacy). The
   net-new bit: the gate-runs-eval-itself-in-detached-worktree idiom for the I-4 merge step.
2. AMEND I-2 + I-4 — S-3 WORKTREE LIFECYCLE. Insert GD-1 prior-art pointer lines (cite
   executor_run.py:116-210, :432, core/git_artifacts.py, git_ops.py:130) with the 5 stealable
   mechanics: (1) branch start-point = CURRENT trunk HEAD (picks up prior merges); (2) finalize-
   commit filters generated artifacts out; (3) worktree removed but BRANCH PRESERVED for later
   merge; (4) merge-eval runs in its own separate detached worktree pool; (5) stale worktrees force-
   removed at create, collision => timestamp suffix. NOTE: Arbor has NO pooling and is Unix-only
   (os.getuid()); ELAI's I-2 warm pooled Windows-safe worktrees (IC-9) are an IMPROVEMENT — keep
   them. No new session.
3. AMEND I-4 ranker prompt + the executor/eval discipline — S-4 PROMPT CONTENT (port phrases, NOT
   the architecture). DO-NOT-PORT (cite M-1): the persistent-ReAct shape, the 500-turn loop, the
   premature-stop nudges, LLM-chosen node selection — ELAI's FSM + deterministic leaf selection
   replaces all of it. DO-PORT as advisory prompt text WITH source cite: prompts.py:396 SELECT
   guidance => I-4 ranker tie-break (advisory only); prompts.py:442 Evaluation Discipline (B_dev vs
   B_test) => the executor/eval prompt; skills/idea_drafting.md IDEATE method => the babystep-author
   / scope-planner prompt material on the CLOUD lane ONLY (7B produces weaker hypotheses — quality
   lives here, mechanics don't care); executor/prompts.py UNDERSTAND..REPORT => diff against ELAI's
   executor prompt, steal deltas only. NOTE: the do-not-re-propose-same-hidden-assumption wording
   (idea_tree.py:397) is already carried by J-10's CONSTRAINTS BLOCK — cross-reference, do not
   duplicate.
4. AMEND H-50 (or note in its authoring) — S-5 TOKEN-EFFICIENCY. Insert Arbor's ContextManager
   layers 1-3 (core/context.py:269: truncate-15K / snip-old-tool-results / drop-redundant-reads of
   the SAME file) as prior art for the local ContextPolicy — Arbor's L3 redundant-READ dedup is a
   distinct dedup ELAI's H-6 turn-masking does not list. Layer-4 (the 20K LLM-summarize call) is
   EXPLICITLY REJECTED at the 32K local ceiling (H-51 topic-digest + I-14 deterministic compression
   replace it). Add the context_prune.py pointer (elide IDEATE scratch after each commit). Record
   the D1-D5 cost-driver -> ELAI-asset map (S-5) as positioning, NOT work items (every driver is
   already attacked by elai.map/codegraph/CatRAG/FSM-selection/deterministic-parsers/H-1/H-2/I-10).
5. AMEND D-11 — S-7 COMBENCH CHECKER CONTRACT. Insert the minimal reusable contract their CODE
   implements (not the paper prose), each clause with its source file: verifier = self-contained
   script, payload on stdin, ast.literal_eval parse, prints "True" on pass (prime_code_executor.py);
   answer-format contract lives IN THE PROMPT, harness never validates format — verifier rejects
   malformed payloads; extraction via exactly-one <construct>...</construct> regex, proof and
   construction sections parsed INDEPENDENTLY so one errors while the other scores (extractor.py);
   per-component result dict {status, raw_score, normalized_score, reason} kept separate end-to-end
   (evaluator.py); the 4-string judge rubric <points>N out of 7</points>, N in {0,1,6,7}, parse
   failure = error never a guessed score (score_parsing.py). NET-NEW the register-time gate: ComBench
   runs the REFERENCE answer through the verifier at dataset-build time and fails the build if it
   doesn't pass (data_process/check_ref_construction.py) — ELAI analog: at checker REGISTRATION the
   registry runs the checker against its reference fixture; register FAILS CLOSED. DO-NOT-COPY (cite):
   the reliability_guard "NOT a security sandbox" (testing_util.py:676 — ELAI uses Tier-C), the
   stdout=="True" string protocol (ELAI: JSON CheckerResult already in D-11), ast.literal_eval-only
   payloads (ELAI: JSON+schema), no checker versioning (ELAI: blake3 content-address per checker).
   EXPLICIT DEMOTION (M-2: ComBench does NOT implement it — evaluator.py:274 averages, never
   demotes): D-11 must implement demotion explicitly — construction-fail demotes the advisory-judge
   proof score monotonically, final<=raw. The demotion pattern generalizes to ANY ELAI gate that
   accepts prose + a checkable witness (HD-5/VD-1 doctrine). Confirmatory cite for I-38 pass^k / G-6
   Avg-not-Best and TD-3 (construction verdict deterministic, proof score judge-dependent).
6. AMEND I-30 + H-47 + I-25 + D-6 — S-8 DENOVOSWE MECHANISMS (cite paper sections 3.3 / 4.1):
   (a) I-30: adaptive difficulty-banded thresholds (their only real ablation, +1.2pp; bands
       [0,.2)/.9 [.2,.4)/.85 [.4,.6)/.8 [.6,.8)/.7 [.8,1]/.6; the gain came from the STRICTER easy
       tier). Amendment: no global pass/fail gate on SFT exhaust; the difficulty band sets the
       inclusion floor; a PARTIAL on a hard task is worth more than a PASS on a trivial one.
   (b) I-25 + I-30: difficulty FUSION d = w_s*struct + w_g*judge1 + w_q*judge2, weights grid-searched
       on the simplex to maximize |Pearson rho| vs empirical pass rate (Eq.4-5). Winning weights NOT
       published — ELAI calibrates on its OWN rollouts; the structural signal ("executable Python
       lines") needs a domain substitute (SCIP function count / cross-language bindings touched /
       SSB variants added). => I-25 router feature + I-30 banding input.
   (c) H-47 + D-6: leakage — static command-blocking alone CONFIRMED insufficient (an agent with a
       shell re-downloads); two-layer = block + LLM trace audit. Concrete bypass modes for H-47's
       test suite: git clone of origin, pip install/download via pypi fuzzy match, curl/wget of
       source, git reflog + loose-object recovery (re-init .git entirely). => H-47 threat-case
       enumeration + D-6 store hygiene (re-init .git in held-out fixtures). No leak-rate number
       published — design rationale only.
   (d) DECLINE recorded in-file: the DCR doc loop (S-8d) is offline-only, no session. CAVEATS pinned
       (M-3): SFT-only, 131K context => cloud-lane-only for any trajectory training, loss-masking
       unablated, the repo-selection funnel is offline-pipeline reference for I-30's dataset card.
7. RECORD positioning (not a work item): the "WHAT ARBOR LACKS" block (no wiring gate, fail-open
   merge, no auditor stage, mutable evaluator config, informal event log, no cost hard-stop, no bash
   sandbox, no crash recovery, single scalar objective) — each is an ELAI R1-R9/TD/GD item Arbor
   independently proves the need for. Insert as a one-line GD-1 positioning pointer into the relevant
   strategy/positioning doc IF one is being touched; otherwise leave in the steal-list. NOT a build.
8. Decision doc: workflow/reasoning/J-11-decisions.md (which targets were amended, the verified
   status of each at edit, the two fail-opens named, the demotion composition, the declines).

## WIRING_CONTRACT
produces_symbols:
  - amendment blocks (source-cited) inserted into I-4, I-2, I-17, I-25, I-30, D-6, D-11, H-43, H-47,
    H-50 (the held-out gate idiom + the two DO-NOT-COPY fail-opens + the worktree mechanics + the
    checker contract + the register-time gate + the difficulty banding + the leakage threat cases)
  - workflow/reasoning/J-11-decisions.md
consumes_symbols:
  - the named Arbor/ComBench source files (read directly per W-3; the citations point AT them)
  - the DeNovoSWE paper sections 3.3 / 4.1
  - the target plan files (amended in place; never re-authored — GD-1)
reachable_from:
  - the consuming sessions (I-4, D-11, I-30, H-47, ...) read their amended plan file before
    executing; the amendment is the grounding the implementer reads [JD-1 harness evidence: the
    pointer line is present in the target file, grep-provable]
stub_ban:
  - NO summary-only port: every amendment names the real source file:line; an amendment that
    paraphrases without the source path is a W-3 violation
  - NO copying the two Arbor fail-opens (F/O-1 LLM-score fallback, F/O-2 mutable eval config) — they
    are inserted as explicit DO-NOT-COPY, never as patterns to follow (R1/R8)
  - NO new runtime code, NO new SSB action_type (this is a documentation/amendment unit, JC-3)
  - NO re-authoring a target plan file (GD-1); insertion only
frontend_pair:
  N/A — plan-file amendments; justified (no runtime surface).
observable_effect:
  trigger: "grep each target plan file after the amendment lands"
  expect_artifact: "each amendment block is present in its target file with its source file:line
    citation (grep-provable per item: I-4 carries the held-out gate idiom + both DO-NOT-COPY
    fail-opens; D-11 carries the checker contract + the register-time reference-fixture gate + the
    explicit demotion clause; I-30 carries the banding table + partial-inclusion floor; H-47 carries
    the four leakage bypass modes). NEGATIVE (R2): an amendment lacking its source file:line, or one
    that presents F/O-1/F/O-2 as a pattern to copy rather than refuse, FAILS review."

## DETERMINISTIC_CHECK
```bash
# each amendment present in its target, source-cited (re-grep target paths at edit, W-7):
rg -n "git_ops.py:282|detached worktree|F/O-1|F/O-2" docs/thefinalglance/I-plan/I-4_*.md
rg -n "executor_run.py:116|branch.*trunk HEAD|BRANCH PRESERVED" docs/thefinalglance/I-plan/I-2_*.md
rg -n "check_ref_construction|register-time|demotion|final<=raw" docs/thefinalglance/D-plan/D-11_*.md
rg -n "difficulty.*band|inclusion floor|0\.9|0\.85" docs/thefinalglance/I-plan/I-30_*.md
rg -n "git clone of origin|reflog|re-init .git|trace audit" docs/thefinalglance/H-plan/H-47_*.md
rg -n "context.py:269|layer-4|REJECTED at.*32K|redundant-READ" docs/thefinalglance/H-plan/H-50_*.md
test -f workflow/reasoning/J-11-decisions.md
```

## ACCEPTANCE
- [ ] every amendment names its real source file:line (W-3); zero paraphrase-only insertions.
- [ ] both Arbor fail-opens (F/O-1 LLM-score fallback, F/O-2 mutable eval config) inserted as
      explicit DO-NOT-COPY into I-4, mapped to R1 / R8.
- [ ] D-11 carries the register-time reference-fixture gate (fail-closed) AND the explicit demotion
      clause (M-2: ComBench does not implement it; ELAI must).
- [ ] I-30 carries the difficulty-banded inclusion floor + the partial-over-trivial rule; the
      structural-signal domain substitute is named (SCIP function count etc.).
- [ ] H-47 carries the four concrete leakage bypass modes + the two-layer (block + trace audit)
      doctrine; D-6 carries the .git re-init store hygiene.
- [ ] no runtime code, no SSB, no target plan file re-authored (GD-1); declines (S-8d, Survey)
      recorded in J-11-decisions.md.
- [ ] every external figure (-27pp, +1.2pp, leak rates) stays design-rationale, never a target
      (HD-1/ID-5).

## Tests to add
- None (documentation/amendment unit; the DETERMINISTIC_CHECK greps ARE the test — each amendment
  must be grep-present in its target with its source citation, JD-1 harness evidence). The negative
  arm is a review check: an uncited amendment or a fail-open-as-pattern insertion fails review.

## Files touched
- MODIFY: docs/thefinalglance/I-plan/{I-4,I-2,I-17,I-25,I-30}*.md (source-cited amendment blocks)
- MODIFY: docs/thefinalglance/D-plan/{D-6,D-11}*.md
- MODIFY: docs/thefinalglance/H-plan/{H-43,H-47,H-50}*.md
- CREATE: workflow/reasoning/J-11-decisions.md
- (all docs are gitignored; no .elai_cc change — this unit ships no runtime code)

## Citations / Grounding
- ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md S-2/S-3/S-4/S-5/S-7/S-8 + the HANDOFF section +
  "WHAT ARBOR LACKS" + M-1/M-2/M-3/M-4 (the index; the SOURCE files in Reads are ground truth, W-3).
- Arbor / ComBench source files named in Reads #1-#2 (read directly). DeNovoSWE paper sections
  3.3 / 4.1 in Reads #3.
- ELAI ground truth: the target plan files' verified status at edit (SRC-4: summary indexes are
  STALE-by-construction). Hard rules invoked: W-3 (no summary-only port), R1/R8 (the two fail-opens),
  TD-3 (LLM never the gate), M-2 (demotion absent — implement explicitly), GD-1 (compose, never
  re-author), HD-1/ID-5 (measured-only).

## Open questions / risks
- target-file status drift: I-4/D-11/I-30/H-47/H-50 statuses in the summary indexes are stale; verify
  each at edit (git log + FINISH master index) before inserting, and skip+note any target already
  carrying the amendment (idempotent).
- I-4 carries S-2 + S-3 + S-4 (three insertions into one file): keep them as three separately-anchored
  blocks (merge-gate / worktree / prompt-content) so each is independently grep-checkable.
- the "WHAT ARBOR LACKS" positioning block: only insert if a strategy/positioning doc is being
  touched this session; otherwise it stays in the steal-list (not a forced write).

## Next
Closes the Arbor/ComBench/DeNovoSWE intake: J-10 built the one net-new capability (insight backprop),
J-11 grounded the rest into the existing plan corpus with source-cited amendments. No parallel HTR
subsystem (S-6). The consuming sessions (I-4 merge step, D-11 checker, I-30 SFT exhaust, H-47 eval
firewall) now implement against real Arbor/ComBench source, not a paraphrase.

End of J-11.
