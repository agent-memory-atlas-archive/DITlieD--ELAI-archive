# J-108: the multi-hop / long-horizon retrieval DEPTH operator — HippoRAG-PPR primary + a constrained RetrievalNavigator fallback, both abstain-on-budget (the measured RLM "merge", never free-Python RLM)

STATUS: PLANNED (WAVE RLM-2; M; EXTENSION(J-42 + J-86 + J-38) — the DEPTH sibling to the J-86 WIDTH ladder; local-lane default GATED on the J-107 measurement; depends J-107, ships AFTER it)

Objective: the measured answer to "ELAI, RLM, or both?" (rlm-recursive-agents-grc-2026-06-26.md PARTS 7-9) is BOTH, in one specific shape: ELAI deterministic retrieval is the SPINE (wins lexical long-context outright, RG-3), and the multi-hop / long-horizon case — where single-shot retrieval STRUCTURALLY fails (RG-4) — is served by a DEPTH operator that iterates deterministically and ABSTAINS rather than fabricates. The measured ceiling (RG-5) reframes the build: the path to ~100% is DETERMINISM, not model-navigation tuning (ORACLE traversal = 8/8; model-driven nav PLATEAUS ~25%; a deterministic graph-hop hybrid made a weak 8B hit 87%). So J-108 ships, in priority order: (1) PRIMARY — route the multi-hop case through the SHIPPED HippoRAG PPR retriever (hippo_rag.rs::retrieve_notes, RS-4) + an atomic model extraction; (2) FALLBACK — a constrained RetrievalNavigator (a FIXED NavOp vocabulary over the query_with_tier candidate set + the adaptive_tool journaled named-op substrate, sibling to the SearchLadder WIDTH template) for cases with NO clean entity linkage, abstain-on-budget. Hard boundaries: NO free-Python RLM (RG-4: 0% + hallucinated + recursion never engaged even on a 120B — REJECTED every lane); NO model-CHOSEN orchestration (RG-1 G6 anti-goal — the op set is FIXED, the executor deterministic, the model only picks WHICH op / reads ONE atomic entity); NO routing the local lane to general CodeAct (J-55 keystone); the local-lane default-ON is contingent on the J-107 measurement clearing. GD-1 already-shipped, do NOT re-implement: hippo_rag.rs PPR (RS-4), query_with_tier (RS-1), adaptive_tool ToolDispatcher (RS-2), the SearchLadder struct/handle/journal idiom (RS-3), J-86 receipt selection, J-38 mid-task loop, J-42 within-step distill-reset, SessionDag/H-49 backtrack.

Estimated effort: ~1.5-2 sessions (~M). The HippoRAG routing decision + atomic-extraction wrap is small (the retriever is shipped); the RetrievalNavigator fallback (navigator.rs, the NavOp loop + journaling + abstain-on-budget) is the bulk. The wiring at the task_runner retrieval seam + the env gate + the allowlist row + the contract.
Pre-requisites (re-grep, W-7): J-107 (the gate — author the local-lane default-ON ONLY if its arm cleared; the cloud/capable lane may ship without waiting); hippo_rag.rs::retrieve_notes (RS-4 :57); query_with_tier (RS-1 pipeline.rs:581); adaptive_tool.rs ToolDispatcher + dispatch_file_read_outline (RS-2 :295) + the SSB markers; search/ladder.rs run_best_of_n (RS-3 :173) + search_ladder_enabled (:382) + BudgetCollapsed (:350) — the idiom to COPY; SessionDag branch/merge + H-49 (the backtrack substrate); the task_runner retrieval dispatch seam where SearchLadder is constructed (PLAN-asserted ~task_runner.rs:12869+, executor re-greps, W-7); J-84 (the classifier that routes a query to single-shot vs depth) + I-7/I-8 (escalate/abstain).
Parallel-safe with: J-107 (disjoint files; but J-108 reads J-107's gate). Depends J-107; ships AFTER it.
Source: rlm-recursive-agents-grc-2026-06-26.md PART 5 H1 + PART 6 (constrained within-step recursive-context primitive = EXTENSION(J-42/J-86), local-lane GATED) + PARTS 7/8/9 (the measured merge + the HippoRAG-primary ceiling) + the full design sketch in workflow/experiments/RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md; _grounding/J-RLM-GROUNDING-MAP.md RG-3/RG-4/RG-5/RG-6 + RS-1..RS-7 + RC-1/RC-2. Doctrine: J-0_INDEX.md (W/GD/HD/TD/ID inherited; ADDS no delta family — TD-3 deterministic-executor, IC-6 receipts-not-votes, G6 anti-goal all cover it).

## Context
Why this matters: SearchLadder (J-86) is WIDTH — branch N candidates in parallel, pick the best by receipt. The multi-hop / long-horizon retrieval problem is DEPTH — iterate sequentially, each step's op informed by the last, to CHAIN hops (the J-38 mid-task-retrieval-loop / J-42 within-step-distill grain). They are orthogonal and coexist (the GRC-v3 doc already names both axes: J-86 width vs J-84 serial depth). The two ELAI experiments settled the design: free-code RLM lost every cell and FABRICATED on hard tasks (RG-4); single-shot retrieval abstains-safe but can't multi-hop (RG-4); the deterministic iterative merge is the only multi-hop-capable arm that preserves the abstain-not-fabricate trust contract (RG-4); and the ROUTE TO ~100% is to minimise the model to the genuinely-fuzzy step (entity read) and make find/hop/extract deterministic (RG-5) — which is exactly what the SHIPPED HippoRAG PPR retriever already does in ONE pass. So D is NOT new infrastructure: it is three shipped pieces (RS-1 candidate set, RS-2 journaled ops, RS-4 HippoRAG) composed by one new bounded loop + a routing decision.

THE OPERATOR (priority order):
  RN-1 PRIMARY — DETERMINISTIC GRAPH-HOP: route a multi-hop query through hippo_rag.rs::retrieve_notes (PPR over the entity graph, one deterministic pass propagating query -> bridge entity -> all hops) + an ATOMIC model extraction (read the entity / regex the number off the single grepped hop chunk, or a 3-sample vote). The model NEVER navigates; the entity graph IS the traversal (RG-5; RC-2).
  RN-2 FALLBACK — CONSTRAINED RetrievalNavigator: for queries with NO clean entity linkage (no graph edge to propagate along), a FIXED NavOp loop over the query_with_tier candidate set: NavOp::{Grep(pat), Read(ids), Expand(id), SubQuery{ids,q} (a CAPPED depth<=2 child-spawn, NOT an open REPL), Answer(s)}. The model PROPOSES one named op (TD-3 proposes); a deterministic executor RUNS it (TD-3 adjudicates); every op is an SSB row (RS-2; replay falls out free). Caps ~25-37% on hard multi-hop (RG-5) — the safety net, never the primary.
  RN-3 ABSTAIN-ON-BUDGET == SearchLadder's BudgetCollapsed -> winner:None (RS-3 :350): on max_steps exhaustion the operator returns "no answer", NEVER a guess (the measured trust contract; B fabricated "42" x7, D abstained — RG-4).
  RN-4 BOUNDED RECURSION: SubQuery is a capped child call (depth<=2) over the SHIPPED orchestrate/BSP sub-agent machinery, not arbitrary REPL recursion — sits BELOW the J-55 code-mode collapse threshold (RG-6: the constrained named-op path held 100% on the weak lane where free-code was erratic).
  RN-5 STEP BUDGET = the J-107-MEASURED knee (ID-5): max_steps is the K-15-class per-(model,lane) calibration the J-107 bench sizes (the experiment showed 10->18 recovers abstained-but-reachable cases; the bench sizes it per model). NEVER a paper constant.

## Existing state (B/C/D/E/F/G/H/I/J coverage; RS-* in the grounding map)
- HIPPORAG MULTI-HOP RETRIEVER [WIRED] crates/memory/src/hippo_rag.rs::retrieve_notes (:57) — Personalized-PageRank over the entity/note graph (arXiv:2405.14831, in-code citation); the deterministic ONE-pass multi-hop retriever = RN-1 primary (RS-4). Do NOT re-implement.
- STAGE-1 CANDIDATE SET [WIRED] crates/knowledge/src/pipeline.rs::query_with_tier (:581) — BM25+vector+rerank; the RN-2 candidate set (RS-1). Production superset of exp-1's BM25-only Arm C.
- JOURNALED NAMED-OP SUBSTRATE [WIRED] crates/tools/src/adaptive_tool.rs ToolDispatcher + dispatch_file_read_outline (:295) + the SSB markers (ToolCallStart/Read/Branch/Finding/Converge/Escalate/ToolCallEnd) — RN-2's op-journaling + replay (RS-2). dispatch_file_read_outline IS the Read op.
- WIDTH LADDER TEMPLATE [DARK(ELAI_SEARCH_LADDER_ENABLE)] crates/orchestrator/src/search/ladder.rs run_best_of_n (:173) + search_ladder_enabled (:382) + BudgetCollapsed (:350) — the struct/handle/journal idiom J-108 COPIES for the DEPTH sibling (RS-3; J-86 activates the width side). navigator.rs is its sibling file.
- SESSION DAG [WIRED] branch/merge + H-49 [PARTIAL] undo-tree — the optional backtrack substrate for a failed hop (no bespoke tree-search).
- J-42 within-step distill-reset [PLANNED] + J-38 mid-task retrieval loop [PLANNED] + J-86 receipt-gated WIDTH ladder [PLANNED] + J-84 classifier [PLANNED] + I-7/I-8 escalate/abstain — the axes/router this extends/composes (RS-6). J-107 (this wave) = the GATE.

## Reads (mandatory before coding)
1. _grounding/J-RLM-GROUNDING-MAP.md (RG-3/RG-4/RG-5/RG-6, RS-1..RS-7, RC-1/RC-2 — especially RC-2: HippoRAG primary, navigator fallback).
2. workflow/experiments/RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md IN FULL (the design sketch: the NavOp enum, the three-pieces-already-shipped framing, the HippoRAG-primary revision, the wiring/gate/non-goals) + RLM-VS-ELAI-RESULTS-2026-06-26.md (the measured failure modes that justify abstain-on-budget).
3. search/ladder.rs IN FULL (run_best_of_n + the BudgetCollapsed/winner:None idiom to copy) + hippo_rag.rs::retrieve_notes + query_with_tier + adaptive_tool.rs ToolDispatcher + the task_runner retrieval dispatch seam — re-grep live (W-7); a wrong seam is a material fork (STOP + handoff).
4. J-107 (the gate artifact — read its knee + lift + collapse-check before authoring the local-lane default) + J-84 (the classifier that routes to single-shot vs depth) + J-86 (the width sibling) + I-7/I-8.
5. CLAUDE.md: TD-3 (the executor is deterministic, the model proposes one op / reads one entity), IC-6 (receipts-not-votes), G6/RG-1 (no model-chosen orchestration), J-55 (no local-lane CodeAct), W-2/R9 (kill-switch + allowlist for the default-OFF gate), D-030 production-caller, no .unwrap on prod.

## Tasks (WAVE RLM-2 — the DEPTH operator; ship AFTER J-107)
1. RN-1 HippoRAG routing (~50 LOC): a deterministic router (reuse J-84's TYPE axis / D-10) sends a multi-hop/long-horizon query to hippo_rag.rs::retrieve_notes (PPR, one pass) + an ATOMIC model extraction wrap (read the bridge entity / regex the answer off the single hop chunk; optional 3-sample vote for the atomic read only). The model never navigates (RC-2). Do NOT re-implement PPR (GD-1).
2. RN-2 RetrievalNavigator fallback (~140 LOC) crates/orchestrator/src/search/navigator.rs: the FIXED NavOp enum + the bounded loop over query_with_tier's candidate set, each op executed by a DETERMINISTIC executor + journaled via the adaptive_tool markers (RS-2). Mirror the SearchLadder struct/handle/journal idiom (RS-3); SubQuery capped depth<=2 (RN-4). Fires ONLY when RN-1 has no clean entity linkage.
3. RN-3 abstain-on-budget (~20 LOC): on max_steps exhaustion emit the BudgetCollapsed-equivalent + return answer:None — NEVER a guess (RG-4 trust contract; the negative arm of the gate).
4. RN-5 step budget from J-107 (~15 LOC): max_steps reads the J-107-measured per-(model,lane) knee (ID-5); a fixture default now, the live knee when J-107's local run lands (W-10).
5. WIRE at the task_runner retrieval seam (~40 LOC): construct the operator at the SAME dispatch seam where SearchLadder is built (Arc-cloned at boot); invoke it on the multi-hop/long-context path (a needle the single-shot top-k misses). Env gate default-OFF ELAI_RETRIEVAL_NAVIGATOR_ENABLE (kill-switch posture per W-2) + a wire-dark-allowlist.toml row (type=experiment, owner, expiry_date, promotion_plan_id=J-108, smoke_test) per R9. PRODUCTION_HOOK: the navigate()/route call MUST be reached from the task_runner retrieval path in the SAME changeset (D-030); off-agent production_caller_check.py confirms.
6. The local-lane default DECISION (~10 LOC + the decisions doc): author the local-lane default-ON ONLY if J-107's arm cleared (lift + no J-55 collapse); else local lane stays single-shot/HippoRAG-only and the navigator is cloud/capable-lane only (RG-6/J-55). Record the decision.
7. workflow/reasoning/J-108-decisions.md (HippoRAG-primary/navigator-fallback ordering; the fixed-NavOp/deterministic-executor TD-3 boundary; abstain-on-budget; the J-107-gates-the-local-lane rule; the free-Python-RLM/model-chosen-orchestration non-goals).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/search/navigator.rs (the RetrievalNavigator: fixed NavOp loop + deterministic executor + abstain-on-budget) + the RN-1 HippoRAG routing decision + the task_runner retrieval-seam wiring + the ELAI_RETRIEVAL_NAVIGATOR_ENABLE gate
consumes_symbols:
  - crates/memory/src/hippo_rag.rs::retrieve_notes (the RN-1 primary multi-hop pass; ACTIVATE, do NOT re-implement; RS-4, GD-1)
  - crates/knowledge/src/pipeline.rs::query_with_tier (the RN-2 candidate set; RS-1)
  - crates/tools/src/adaptive_tool.rs ToolDispatcher + dispatch_file_read_outline + the SSB markers (the RN-2 op-journaling; RS-2)
  - crates/orchestrator/src/search/ladder.rs run_best_of_n / BudgetCollapsed / search_ladder_enabled (the struct/handle/journal idiom + the abstain-on-budget pattern to COPY; RS-3)
  - SessionDag branch/merge + H-49 (the optional hop-backtrack substrate); J-84 (the classifier router) + J-107 (the step-budget knee + the gate) + I-7/I-8 (escalate/abstain)
reachable_from:
  - the task_runner retrieval dispatch path -> route(multi-hop query) -> HippoRAG primary or RetrievalNavigator fallback [PRODUCTION retrieval, task_runner.rs ~12869 PLAN-asserted, re-grep W-7]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO free-Python RLM / model-written code over its own context (RG-4: 0% + hallucinated; REJECTED every lane)
  - NO model-CHOSEN orchestration / open NavOp set (FIXED vocabulary, deterministic executor, model proposes one op only; RG-1 G6, TD-3)
  - NO model navigation as the multi-hop PRIMARY (HippoRAG-PPR is primary; the navigator is the no-entity-linkage fallback; RC-2)
  - NO guess on budget exhaustion (abstain-on-budget == BudgetCollapsed/winner:None; RG-4); NO uncapped SubQuery recursion (depth<=2; RN-4)
  - NO routing the local lane to general CodeAct (J-55); NO paper-constant step budget (J-107-measured knee, ID-5); NO re-implementation of hippo_rag/query_with_tier/ladder/adaptive_tool (GD-1); NO .unwrap on prod; NO external trace store for the op journal (the SSB rows ARE the record, GD-2)
frontend_pair:
  the existing search/ConvergencePanel surfaces the hop chain + the selected/abstained outcome + the op stream — decide a tile vs reuse at edit (W-4); justified N/A if it rides the existing I-4/I-27 surfaces.
observable_effect:
  trigger: "drive the REAL task_runner retrieval entrypoint on (a) a multi-hop query WITH clean entity linkage [-> HippoRAG primary], (b) a multi-hop query with NO entity linkage [-> RetrievalNavigator fallback], (c) an unreachable-answer query [-> abstain] (integration smoke, R10, on a HARD long-context input)"
  expect_ssb: "reuse SearchNodeEvent (Promoted / BudgetCollapsed, RS-3) + the adaptive_tool op markers (RS-2) per NavOp + query_with_tier's existing rows for the HippoRAG/candidate path — net 0..+1 from LIVE ~268/577 (a +Kind decided at edit, lockstep, W-9)"
  expect_artifact: "(a) HippoRAG returns the multi-hop-linked facts in one pass + the atomic extraction answers; (b) the navigator chains hops via fixed named ops, each an SSB row (replayable); (c) on budget exhaustion the operator ABSTAINS (answer:None, a BudgetCollapsed-equivalent row) and emits NO number — the NEGATIVE arm that must FAIL: a model writing/executing code over the context, a fabricated answer on budget exhaustion, or an op the model authored that is not in the fixed NavOp set"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-108_retrieval_navigator_depth_operator.md --run-trigger
rg "NavOp|abstain|BudgetCollapsed|hippo|retrieve_notes|max_steps" .elai_cc/crates/orchestrator/src/search/navigator.rs   # the RN-1..RN-5 legs
rg -c "exec|eval|python|interpret|run_code|compile" .elai_cc/crates/orchestrator/src/search/navigator.rs   # MUST be 0 (no free-code RLM; RG-4, the model never executes code)
rg -c "self.choose.*control.flow|model.*authors.*op|dynamic.*op|open.*op.set" .elai_cc/crates/orchestrator/src/search/navigator.rs   # MUST be 0 (fixed NavOp set; G6/TD-3)
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/search/navigator.rs   # 0-DARK (wired at the task_runner seam this changeset; D-030)
rg "ELAI_RETRIEVAL_NAVIGATOR_ENABLE" .elai_cc/wire-dark-allowlist.toml   # the R9 row exists
test -f workflow/reasoning/J-108-decisions.md
```

## ACCEPTANCE
- [ ] RN-1: the multi-hop PRIMARY is HippoRAG-PPR (hippo_rag.rs reused) + atomic model extraction; the model never navigates the corpus (RC-2).
- [ ] RN-2: the RetrievalNavigator is a FIXED NavOp set + a deterministic executor (model proposes one op); fires only on no-entity-linkage; SubQuery depth<=2 (RN-4); no free code (RG-4).
- [ ] RN-3: abstain-on-budget (answer:None, BudgetCollapsed-equivalent) — never a guess (RG-4 trust contract; the negative arm).
- [ ] RN-5: max_steps = the J-107-measured per-(model,lane) knee (ID-5); fixture default now, live knee W-10-gated.
- [ ] J-55 + the gate: the local-lane default-ON is authored ONLY if J-107's arm cleared (lift + no collapse); else navigator is capable-lane only. Default-OFF ELAI_RETRIEVAL_NAVIGATOR_ENABLE with a complete wire-dark-allowlist row (R9).
- [ ] GD-1/D-030: hippo_rag/query_with_tier/ladder/adaptive_tool reused not re-implemented; navigate() reached from the task_runner seam THIS changeset (production_caller 0-DARK); cargo check GREEN; auditor PASS -p orchestrator (+memory/knowledge as touched).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the 3-case integration smoke (HippoRAG / navigator-fallback / abstain) run-trigger on a hard long-context input, not file-exists.
- W-2/R9: ELAI_RETRIEVAL_NAVIGATOR_ENABLE ships with a wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=J-108, smoke); kill-switch posture, not a terminal default-OFF (promotion gated on J-107's measurement).
- W-3: EXTENSION — composes hippo_rag/query_with_tier/ladder/adaptive_tool; the navigator loop + the routing decision are the net-new (a DEPTH sibling, not a summary-port).
- W-4: rides the I-4/I-27 ConvergencePanel (tile vs reuse at edit) or justified N/A. W-5/W-10: in-process; the live local-lane run + the per-lane knee are RunPod-gated (the separate W-10 checkbox).
- W-6: existing search/convergence panel or justified N/A. W-7: navigator seam (task_runner) + hippo_rag + query_with_tier + ladder + adaptive_tool re-grepped at edit (the task_runner line is PLAN-asserted).
- W-9: reuse SearchNodeEvent + adaptive_tool markers + query_with_tier rows; net 0..+1 (a Kind decided at edit, lockstep). TD-3: deterministic executor, fixed op set. IC-6: receipts not votes (the atomic-read 3-sample vote is for the FUZZY read only, never the selector). GD-2: the op stream + the answer recomputable from the SSB rows.

## Tests to add
- crates/contracts/tests/j108_retrieval_navigator_contract.rs (~110 LOC): (a) a multi-hop query WITH entity linkage routes to HippoRAG-PPR (one deterministic pass), not the navigator (RN-1); (b) a no-entity-linkage multi-hop query routes to the RetrievalNavigator (RN-2); (c) every NavOp emits an SSB row and the op stream replays identically (GD-2); (d) the model can ONLY pick an op from the fixed NavOp set — an out-of-set op is rejected (G6/TD-3 negative arm); (e) no code execution path exists in the navigator (RG-4 negative arm); (f) SubQuery is capped at depth<=2 (RN-4); (g) on max_steps exhaustion the operator returns answer:None + a BudgetCollapsed-equivalent, emits NO number (RN-3 trust contract); (h) max_steps reads the J-107 knee artifact, not a literal constant (RN-5/ID-5); (i) corrupt/empty candidate set degrades to abstain, never a fabricated answer (R1).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/search/navigator.rs (~210 LOC: NavOp + the bounded loop + deterministic executor + abstain-on-budget + the HippoRAG routing decision)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (~40 LOC: construct + invoke at the retrieval dispatch seam; the PRODUCTION_HOOK, D-030)
- MODIFY: .elai_cc/wire-dark-allowlist.toml (the ELAI_RETRIEVAL_NAVIGATOR_ENABLE row: type/owner/expiry/promotion_plan_id=J-108/smoke)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs + the .golden ONLY IF a +Kind is added (lockstep, W-9)
- CREATE: crates/contracts/tests/j108_retrieval_navigator_contract.rs (~110 LOC)
- CREATE: workflow/reasoning/J-108-decisions.md (~30 LOC)

## Citations / Grounding
- rlm-recursive-agents-grc-2026-06-26.md PART 5 H1 (constrained within-step recursive primitive = EXTENSION(J-42/J-86), capable-lane default, local-lane gated) + PART 6 (the verdict) + PARTS 7/8/9 (the measured merge + the HippoRAG-primary ceiling) — the steal analysis.
- workflow/experiments/RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md (the full design sketch: NavOp enum, three-pieces-already-shipped, HippoRAG-primary revision, wiring/gate/non-goals) + RLM-VS-ELAI-RESULTS-2026-06-26.md (the measured failure modes — B fabricated, C abstained, D the only multi-hop arm; the ORACLE/hybrid ceiling) — OWN MEASURED DATA.
- ELAI ground truth (RS-1/RS-2/RS-3/RS-4): query_with_tier (pipeline.rs:581), adaptive_tool ToolDispatcher (:295), search/ladder.rs run_best_of_n (:173)/BudgetCollapsed(:350)/search_ladder_enabled(:382), hippo_rag.rs::retrieve_notes (:57, arXiv:2405.14831 in-code) — re-grepped (W-7); the task_runner seam PLAN-asserted (re-grep at edit).
- arXiv [grounded via on-disk maps, RC-3]: RLM 2512.24601 (workflow/papers/rlm-interface.md HIGH; G6 rubric in rlm-rubric-recursive-coding-agents-interface.md), HippoRAG 2405.14831 (NeurIPS 2024, hippo_rag.rs in-code citation).
- CLAUDE.md: TD-3 (deterministic executor, model proposes one op), IC-6 (receipts not votes), J-55 (no local-lane CodeAct), W-2/R9 (kill-switch + allowlist), D-030 (production-caller), HD-2/GD-2 (the SSB rows are the record, no external trace store).

## Open questions / risks
- The "has clean entity linkage" routing decision (RN-1 vs RN-2) must be deterministic (reuse J-84's TYPE axis / the DKG entity graph's edge presence) — verify it forecloses model-chosen routing structurally (TD-3), not by a runtime heuristic that could drift.
- The task_runner retrieval seam is the load-bearing wiring (PLAN-asserted ~12869); verify the dark SearchLadder construction shape before driving the sibling — a wrong seam is a material fork (STOP + handoff, R-4). Do NOT ship navigator.rs dark (D-030 — wire it this changeset or BLOCKED).
- The local-lane default is contingent on J-107: do NOT author local-lane default-ON before J-107's arm clears (lift + no J-55 collapse). If J-107 slips, ship cloud/capable-lane only + the local lane on HippoRAG-PPR + single-shot, and SAY so (honest residue, GD-5).
- The atomic-read 3-sample vote (RN-1) is for the FUZZY entity/number read ONLY — never the multi-hop selector (IC-6; the hop traversal is deterministic HippoRAG). Keep the vote scoped to the single atomic extraction.

## Next
J-108 lands the measured "both": ELAI deterministic retrieval as the spine, the SHIPPED HippoRAG-PPR as the deterministic multi-hop primary (the path to ~100%, RG-5), and a constrained abstain-on-budget RetrievalNavigator as the no-entity-linkage fallback — free-Python RLM rejected, model-chosen orchestration an anti-goal. It is the DEPTH sibling to the J-86 WIDTH ladder, gated by J-107's measurement, and closes the RLM wave: the corpus's ~15% genuinely-actionable surface, built ELAI's way (the harness navigates, the model only reads).

End of J-108.
