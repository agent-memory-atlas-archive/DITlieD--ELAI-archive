# ARBOR + COMBENCH + DENOVOSWE STEAL-LIST — 2026-06-11

Purpose: the grounded port plan from the 4-paper ingest (2606.11926 Arbor / 2606.10479 ComBench /
2606.10728 DeNovoSWE / 2606.12191 Survey) into ELAI. Written for the NEXT session to transform into
plan files. Every steal item names the source files the implementer MUST read directly (W-3: a
summary-only port is a violation), the ELAI substrate it lands on, and the local-lane / token-cost
verdict. Provenance: 3 explorer agents (1 opus full-repo read of Arbor src/, 2 sonnet) run 2026-06-11
in session b2713f18; their raw reports are the grounding for every file:line cite below.

## SOURCES ON DISK

Arbor repo (Apache-2.0, RUC-NLPIR/Arbor, the only full implementation):
  F:\.ELAI_workflow\docs\.source\2606-11926-arbor-hypothesis-tree-refinement\
  core algorithm ~1,800-2,200 LOC out of ~23K total; the rest is CLI/webui/events packaging.
ComBench repo: F:\.ELAI_workflow\docs\.source\2606-10479-combench\repo\  (paper html one dir up)
DeNovoSWE: F:\.ELAI_workflow\docs\.source\2606-10728-denovoswe\2606.10728.html (paper IS the content;
  repo/ is README-only, data is HF-hosted)
Survey: no code (NO-CODE.md); skip — nothing lands on an ELAI plan.
Interface maps: F:\.ELAI_workflow\workflow\papers\2606-*-interface.md (4 files)
Raw paper text: F:\.ELAI_workflow\.claude\papers\2606.*.{pdf,html}

## META-FINDINGS (read before any plan authoring; these change the port shape)

M-1 ARBOR'S ALGORITHM IS MOSTLY A PROMPT. The paper's "Six-Step Coordinator Loop / frontier control
    policy / UCT-like selection" does not exist as code. The implementation is ONE persistent ReAct
    agent (max 500 turns) whose steps are prompt instructions (coordinator/prompts.py ~770 LOC +
    skills/idea_drafting.md ~245 LOC) plus tools the LLM chooses to call. No scheduler, no UCT, no
    automatic decide step (orchestrator.py:96-103 docstring confirms). Hard-coded control logic is
    only: cycle counting, the held-out merge gate, insight propagation, worktree management,
    convergence nudges, context compaction. Consequence: ELAI does NOT port the architecture; it
    ports the 4 hard-coded mechanisms + selected prompt content, and keeps its FSM. The FSM with
    deterministic selection is a strictly better fit for the 7B lane than a 500-turn strategic loop.

M-2 COMBENCH'S DEMOTION RULE IS NOT IMPLEMENTED. The 7->6 / 6->1 proof-score demotion on
    construction failure is paper-prose only; the harness averages normalized proof + construction
    (evaluator.py:274-305 _build_case_result). Anything ELAI builds must implement demotion
    explicitly; never assume their harness is the reference implementation.

M-3 DENOVOSWE IS SFT-ONLY AND CLOUD-SCALE. 131K-token training context, no RL/DPO/GRPO anywhere.
    Its mechanisms feed I-30 (SFT exhaust) and H-47 (eval firewall); they do NOT supply DPO-pair or
    GRPO-reward construction (I-28/I-31 keep their own designs). Loss-masking is asserted, never
    ablated — cannot ground a claim.

M-4 ARBOR'S TOKEN COST (20-43M/run) IS STRUCTURAL, NOT MODEL-PRICED. Dominant driver: each executor
    is a fresh 50-turn agent that re-discovers the codebase by Read/Grep from scratch, full history
    resent every turn. ELAI's existing assets (elai.map, codegraph, CatRAG/repo-map, FSM-side
    selection, deterministic parsers, SSB) attack every driver. The 10x cut is ELAI's existing plan
    tree, not new invention — see S-5.

================================================================================
## THE STEAL ITEMS (ranked by net-new value to ELAI)
================================================================================

### S-1 INSIGHT BACKPROPAGATION [the -27pp mechanism; the ONE genuinely new capability]

WHAT: after every experiment/audit, (a) extract a 1-3 sentence insight from the raw report via a
small LLM call; (b) re-synthesize EVERY ancestor's insight from its children's insights ("under 200
words", root = global research memory, rebuilt every experiment); (c) inject downward at two points:
ancestor-lineage insights into child executor prompts, and a CONSTRAINTS BLOCK into every ideation
turn carrying ROOT INSIGHT (priors) + PRUNED LESSONS ("do NOT re-propose any idea that shares the
same hidden assumption or mechanism class without explicitly explaining how it counters the lesson")
+ VALIDATED FINDINGS ("part of the trunk's working assumptions. Build on them; don't re-derive").
Ablation: -27pp without it — more than the tree structure itself (-18pp).

SOURCE FILES (implementer MUST read):
  src/coordinator/tools/tree_ops.py:518        propagate_insights (the upward walk + synthesis call)
  src/coordinator/tools/executor_run.py:368    _parse_executor_report (extraction prompt, verbatim
                                               JSON-fields contract, 12K-char head+tail truncation)
  src/coordinator/tools/executor_run.py:226    _gather_ancestor_insights (downward injection, exec side)
  src/coordinator/idea_tree.py:358             get_constraints_block (downward injection, ideation side;
                                               :397 pruned-lesson wording, :415 validated-findings wording)
  src/coordinator/idea_tree.py:28-93           Node schema (insight + result + score + code_ref fields)

ELAI LANDING: net-new session (next free I/J slot), nature BUILD, composes-with I-5 (pristine
context's "compressed contrastive constraint" IS Arbor's pruned-lesson in miniature — Arbor
generalizes it to a full constraints discipline with a measured ablation), I-11 (BeliefState is the
cross-handoff sibling; insight tree is the cross-SIBLING/cross-ROUND memory — distinct scope, shares
the digest idea), I-4 (per-branch receipts get an insight field so losers teach), I-12 (root insight
is the runtime precursor of an evidence-gated playbook entry), H-49/SessionDag (the tree store).
Concretely: insight field on decomposition-tree nodes + SSB InsightPropagated rows + synthesis-up on
every audit verdict + constraints-block rendering into executor/ideation prompts.

LOCAL-LANE VERDICT: fully 7B-capable. Both LLM calls are short-input/short-output summarization —
the opus agent explicitly verified no frontier dependency in the mechanism. Cost: ~depth extra small
calls per experiment; on ELAI, depth<=4 and the calls fit the local sidecar.

ANTI-GAMING NOTE: Arbor's insight field is LLM-written and ungated. ELAI should bind each insight to
the SSB row(s) it was extracted from (provenance hash) so a fabricated lesson is detectable — the
ID-3 receipt discipline applied to memory.

### S-2 HELD-OUT MERGE-GATE MECHANICS [confirms H-43/VD-4/D-6; steal the mechanics, fix 2 fail-opens]

WHAT: admission = the GATE ITSELF re-runs the held-out eval (B_test) on the candidate branch in a
fresh DETACHED worktree, off-agent; executor only ever sees B_dev; LLM-reported scores logged but
never trusted; no improvement vs trunk/baseline (metric_direction-aware) = rejected; merge --no-ff;
conflict = abort + restore. protected_paths fnmatch guard keeps eval/data files unmodifiable;
required_outputs checked.

SOURCE FILES:
  src/coordinator/tools/git_ops.py:282-520     GitMergeBranchTool (full gate: :348 execute, :366
                                               protected branches, :109 _run_eval_in_worktree,
                                               :402 None-score reject, :437 is_improvement, :507 --no-ff,
                                               :465 protected_paths, :482 required_outputs)
  src/coordinator/prompts.py:442               "Evaluation Discipline" block (B_dev vs B_test doctrine)
  src/coordinator/tools/executor_run.py:248/266 B_dev-only injection into executor prompts

TWO ARBOR FAIL-OPENS ELAI MUST NOT COPY (both violate wiring-enforcement):
  F/O-1 git_ops.py:413 — eval_cmd_test unset => gate falls back to the LLM-REPORTED score with a
        warning. Violates R1 (missing evidence = hard FAIL). ELAI: absent held-out cmd = BLOCK.
  F/O-2 eval commands live in MUTABLE tree.meta writable by the LLM via TreeSetMeta at any time.
        Violates R8 (gate frozen, read-only to the executing run). ELAI: eval cmds in the frozen
        gate-manifest set, never in agent-writable state.
  Also: score extraction falls back to an LLM call (_parse_eval_score :62) after the JSON regex —
  ELAI replaces with a deterministic parser only (TD-3: LLM is never the gate).

ELAI LANDING: amendment/grounding-pointer lines into I-4 (the ranker's merge step), H-43
(execution-grounded fitness — Arbor is the working existence proof the convergence claim cites),
D-6 (held-out store; B_dev/B_test split = D-6's temporal split, simpler), I-17 (L1 veto supremacy).
Mostly CONFIRMATION; the net-new bit is the gate-runs-eval-itself-in-detached-worktree idiom for the
I-4 merge step.

### S-3 WORKTREE LIFECYCLE DETAILS -> I-2 [confirms IC-9; steal 5 mechanics]

SOURCE: src/coordinator/tools/executor_run.py:116-210 (_compute_branch_name, _create_worktree,
_finalize_worktree, _remove_worktree), :432 after_executor snapshot hook; core/git_artifacts.py
(filter_commit_paths); git_ops.py:130 (separate merge-eval worktree pool).

STEAL: (1) branch start-point = CURRENT trunk HEAD so every dispatch picks up prior merges; (2)
finalize-commit filters generated artifacts (caches/logs) out of the commit; (3) worktree removed
but BRANCH PRESERVED for later merge; (4) merge-eval runs in its own separate detached worktree pool,
never the executor's; (5) stale worktrees force-removed at create time, collision => timestamp
suffix. Arbor has NO pooling (fresh worktree per dispatch) — ELAI's planned warm pool (IC-9) is an
improvement; keep it. Arbor is Unix-only here (os.getuid()); ELAI's I-2 already plans Windows-safe
pooled worktrees.

ELAI LANDING: GD-1 prior-art pointer lines in I-2 + I-4; no new session.

### S-4 COORDINATOR PROMPT CONTENT [port phrases, not the architecture]

DO NOT PORT: the persistent-ReAct shape, the 500-turn loop, the premature-stop nudge heuristics, the
LLM-chosen node selection. ELAI's FSM + deterministic leaf selection replaces all of it (cheaper AND
7B-viable).

DO PORT (into ELAI agent prompt templates / skill files, cite source):
  coordinator/prompts.py:396       SELECT guidance ("prefer high-impact with clear evidence; consider
                                   diversity") — seed text for I-4's ranker tie-break prompt, advisory only
  idea_tree.py:397                 the do-not-re-propose-same-hidden-assumption wording (S-1 block)
  prompts.py:442                   Evaluation Discipline block (B_dev/B_test)
  skills/idea_drafting.md          the IDEATE method (245 lines) — raw material for the babystep-author
                                   / scope-planner prompts on the CLOUD lane only (7B produces weaker
                                   hypotheses; quality lives here, mechanics don't care)
  executor/prompts.py              UNDERSTAND->IMPLEMENT->VALIDATE->ITERATE->EVALUATE->REPORT block —
                                   compare against ELAI's executor prompt, steal deltas only

### S-5 TOKEN-EFFICIENCY: THE 10x MAP [Arbor cost drivers -> ELAI assets; answers the
    "make Arbor+ELAI token-efficient" question]

Driver D1 (DOMINANT): executors re-discover the codebase every dispatch via Read/Grep loops, 50
turns, history resent per turn. ELAI kill: inject prebuilt context — elai.map / codegraph slice /
Phase-1D repo-map / CatRAG — into executor dispatch (B-10 profiles + 3E slots already budget this).
Arbor executors get NO code context at all; ELAI's knowledge engine is the single biggest lever.
Driver D2: k-wide fan-out with zero cross-executor sharing. ELAI kill: S-1 insight loop + I-11
BeliefState make siblings share distilled understanding; I-4 receipts prune early.
Driver D3: LLM-based extraction everywhere (report->fields, eval->score, children->synthesis, 20K
summarize-compaction). ELAI kill: deterministic parsers for report + score (structured executor
report format, TD-3); keep only the two short S-1 calls.
Driver D4: ~3K coordinator + ~2K executor static prompts resent every turn, cached only on Claude.
ELAI kill: H-1/H-2/I-10 cloud cache discipline; sidecar PrefixKVCache un-bypass (H-50 optional leg).
Driver D5: coordinator deliberation tokens for selection. ELAI kill: FSM picks the next pending leaf
algorithmically; zero tokens.
Compaction: Arbor's 4-layer ContextManager (core/context.py:269; truncate-15K / snip-old-tool-results
/ drop-redundant-reads / LLM-summarize-20K). Layers 1-3 are cheap deterministic code — steal into
H-50's local ContextPolicy if not already covered (H-6 masks turns; Arbor L3 drop-redundant-READS of
the same file is a distinct dedup ELAI does not list). Layer 4 (20K summarize call) is INFEASIBLE at
the 32K local ceiling — H-51 topic-digest + I-14 deterministic compression replace it.
Also steal: coordinator/context_prune.py (elide IDEATE scratch after each commit — a smart targeted
pruner). Cost instrument: AgentStats process-wide token counter (agent.py:56) + run_stats.json — ELAI
already has H-9/SpendAccumulator; nothing to port.

### S-6 LOCAL-LANE HTR HYBRID [the user's core question: can Arbor-class search run on ELAI's lanes]

Verdict: YES, as a composition of already-planned ELAI sessions plus S-1. Arbor's frontier dependency
lives ONLY in (a) the 500-turn prompt-driven coordinator (replaced by FSM, S-4) and (b) hypothesis
QUALITY at ideation (route to cloud). Mechanisms 2/3/4/5/7 of the agent map are model-agnostic.
The hybrid, in existing plan vocabulary:
  ideation/architect  = CLOUD (H-7 architect role; Arbor idea_drafting method as prompt material)
  tree + selection    = FSM-owned, deterministic (I-4 ladder; v1 best-of-N FIRST, exactly as planned)
  executors           = LOCAL 7B on small baby-steps with prebuilt context (D1 kill), bounded turns,
                        structured report format; I-2 isolated workspaces
  ranking/admission   = deterministic receipts (I-17 tri-layer) + held-out gate (S-2 mechanics,
                        fail-closed); never LLM-judge-only (ID-4)
  cross-branch memory = S-1 insight loop (7B-capable) + I-11 BeliefState on escalation (I-7 cascade:
                        local-first, condense, escalate once)
  cost control        = E-39 hard ceiling (Arbor has NO budget hard-stop — its cap is a prompt
                        suggestion; ELAI's is a real gate, keep supremacy)
Net: "Arbor on local" is NOT a new architecture for ELAI; it is I-4+I-2+I-7+I-17+H-7 with S-1 added
and S-2's gate idiom on the merge step. The next-session plan should frame it that way — one new
session (S-1) + amendments, not a parallel HTR subsystem.

### S-7 COMBENCH CHECKER CONTRACT -> D-11

WHAT: the minimal reusable contract their code actually implements (not the paper prose):
  - verifier = self-contained script; payload on stdin; ast.literal_eval parse; prints "True" on
    pass, anything else = fail description. Harness wraps in multiprocessing with timeout (15s
    default) + kill.
  - answer format contract lives IN THE PROMPT (per-problem instruction field tells the model the
    exact payload shape); harness never validates format — the verifier rejects malformed payloads.
  - extraction via ONE structural tag: <construct>...</construct>, exactly-one enforced by regex;
    proof section and construction section parsed independently so one can error while the other scores.
  - per-component result dict {status, raw_score, normalized_score, reason} kept separate end-to-end.
  - CI frozen-gate analog: data_process/check_ref_construction.py runs the REFERENCE answer through
    the verifier at dataset-build time; build fails if it doesn't pass. ELAI: at checker REGISTRATION
    time, the registry runs the checker against its reference fixture; register fails closed.
  - judge rubric: forced 4-string output (<points>N out of 7</points>, N in {0,1,6,7}); parse failure
    = error, never a guessed score.

SOURCE FILES:
  repo\src\eval\evaluator.py            EvaluationSuite + the 3 evaluators (+ :274 the averaging that
                                        PROVES demotion is absent — M-2)
  repo\src\eval\extractor.py            <construct> extraction + independent failure semantics
  repo\src\eval\prime_code_executor.py  verifier invocation contract
  repo\src\eval\score_parsing.py        the 4-string rubric parse
  repo\src\prime_code\testing_util.py   :676 reliability_guard ("NOT a security sandbox" — do-not-copy)
  repo\data_process\check_ref_construction.py   the register-time validation pattern
  repo\examples\example.jsonl           full record schema, one annotated toy
  ..\pipeline\prompt_inputs\step3\verify_code.py  the one standalone verifier example (9x9 IMO grid)

DO-NOT-COPY (their weaknesses; ELAI already does better): Python-level reliability_guard sandbox
(ELAI: Tier-C); stdout=="True" string protocol with no envelope (ELAI: JSON verdict per CheckerResult
typed return already in D-11); ast.literal_eval-only payloads (ELAI: JSON + schema); no checker
versioning (ELAI: blake3 content-address each checker at registration); demotion absent (ELAI
implements explicitly: construction-fail demotes advisory-judge score monotonically, final<=raw).

ELAI LANDING: amend D-11 plan file with this contract + the source paths + the register-time
validation gate. The demotion pattern generalizes to ANY ELAI gate that accepts prose + a checkable
witness (HD-5/VD-1 doctrine, now with a concrete reference shape). Confirmatory evidence for I-38
pass^k / G-6 Avg-not-Best framing and TD-3 (their construction verdict deterministic, proof score
judge-dependent) — citation lines only.

### S-8 DENOVOSWE MECHANISMS -> I-30 / H-47 / I-25 / D-6

(a) ADAPTIVE DIFFICULTY-BANDED THRESHOLDS (their only real ablation; +1.2pp over best global):
    bands [0,.2)/.9, [.2,.4)/.85, [.4,.6)/.8, [.6,.8)/.7, [.8,1]/.6 — trajectory kept iff pass-rate
    >= threshold(difficulty). Gain came from the STRICTER easy tier, not looser hard tiers.
    -> I-30 amendment: no global pass/fail gate on SFT exhaust; difficulty band sets the inclusion
    floor; a PARTIAL on a hard task is worth more than a PASS on a trivial one.
(b) DIFFICULTY FUSION: d = w_s*struct + w_g*judge1 + w_q*judge2, weights grid-searched on the simplex
    to maximize |Pearson rho| vs empirical pass rate on a calibration sub-pool. Winning weights NOT
    published — ELAI must calibrate on its own rollouts; structural signal ("executable Python
    lines") needs a domain substitute (SCIP function count / cross-language bindings touched / SSB
    variants added). -> I-25 router feature + I-30 banding input. Section 4.1 / Eq. 4-5 of the paper.
(c) LEAKAGE: static command-blocking alone CONFIRMED insufficient (their words: agent with shell can
    re-download); two-layer = block + LLM trace audit. Concrete bypass modes for H-47's test suite:
    git clone of origin, pip install/download via pypi_name fuzzy match, curl/wget of source,
    git reflog + loose-object recovery (they destroy and re-init .git entirely). No leak-rate number
    published. -> H-47 threat-case enumeration + D-6 store hygiene (re-init .git in held-out task
    fixtures). Section 3.3.
(d) DCR doc loop (draft->critic with rule-based coverage check->repair, validated only by downstream
    pass rate) — offline-only; mild interest for E-48/H-51 digest quality checks; no session.
(e) CAVEATS pinned: SFT-only (M-3); 131K context => cloud-lane-only for any trajectory training;
    loss-masking unablated; repo-selection funnel (>=90% test pass, >=50% coverage, permissive
    license, 4818 instances -> ~34K rollouts -> ~11K kept) is offline pipeline reference for I-30's
    dataset card.

================================================================================
## WHAT ARBOR LACKS (ELAI's moat, verified by the repo read — use in positioning, not as work items)
================================================================================
No static reachability/wiring gate (admission = metric-went-up only; wire-dark code passes). Fail-open
merge path (F/O-1). No auditor/adversarial stage — executor self-certifies via the metric. Mutable
evaluator config (F/O-2 vs ELAI R8). Event log informal (no append-only ledger, no schema lock, no
action_type discipline vs SSB). No token/cost hard-stop (vs E-39). No sandboxing of executor bash
(vs Tier-C). No mid-executor crash recovery (re-run from scratch; vs ELAI checkpoints). Single scalar
objective. Every one of these is an ELAI R1-R9/TD/GD doctrine item Arbor independently proves the
need for — strong citation material.

================================================================================
## HANDOFF TO THE PLAN-AUTHORING SESSION
================================================================================
Recommended shape (one new session + amendments; do NOT build a parallel HTR subsystem — S-6):
1. NEW session "insight-backprop tree memory" (S-1): nature BUILD, composes I-5/I-11/I-4/I-12/H-49,
   +1 SSB variant (InsightPropagated, Pattern-2), frontend pair = insight column on the plan/branch
   view. Wave: after I-4 v1 lands (it injects into the ladder's prompts) but the tree-node insight
   field + audit-time extraction can ship independently before.
2. AMEND I-4: GD-1 prior-art pointers to Arbor coordinator/executor + the S-2 gate idiom for the
   merge step + S-4 SELECT prompt text. AMEND I-2: S-3 pointer lines.
3. AMEND D-11: S-7 checker contract + source paths + register-time reference-fixture gate + explicit
   demotion composition.
4. AMEND I-30: S-8a/b banding + partial-trajectory inclusion + calibration recipe. AMEND H-47: S-8c
   threat cases. Optional I-25 line for the difficulty router feature.
5. AMEND H-50 (or note in its authoring): Arbor compaction layers 1-3 as prior art for the local
   ContextPolicy; layer-4 explicitly rejected at 32K (S-5).
6. Per W-3/SRC discipline: every implementing session reads the named Arbor/ComBench source files
   directly (paths above); this file is the index, not the source.
Declines recorded: Arbor coordinator architecture (M-1/S-4); Arbor layer-4 compaction (S-5); ComBench
sandbox + stdout protocol + literal-eval payloads (S-7); DeNovoSWE DCR loop as a session (S-8d);
Survey (no code, no landing).
