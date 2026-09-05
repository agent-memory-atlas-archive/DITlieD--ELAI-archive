# J-RLM-GROUNDING-MAP — the RLM / recursive-reasoning wave (J-107..J-108)
Wave: RLM-RECURSIVE-REASONING. Authored 2026-06-26 (main LLM, in-band, paper-to-plan PHASE-5 over the
already-complete PHASE-1..4 carried in workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md PARTS 5-9 +
the two measured experiments). Run-id: s-rlm-jplan-2026-06-26.
Reliability of the paper side: HIGH for the two units' load-bearing facts (the units stand on OWN MEASURED
DATA + code JUST READ this session, not on the single-source-LOW RLM marketing numbers).

Status legend mirrors J-0_INDEX. Every file:line below grep-confirmed 2026-06-26 unless tagged PLAN-asserted.

================================================================================
RG — RLM-SIDE FINDINGS (paper + OWN measured experiment; the steal calculus)
================================================================================
RG-1  RLM CORE (arXiv:2512.24601, HIGH; workflow/papers/rlm-interface.md, ingested 2026-06-23). The empirical
      RLM result is NOT "recursion beats long-context" abstractly — it is a DSPy.RLM SCAFFOLD (a HARNESS, no
      fine-tuning) that makes SMALL open models competitive-to-SOTA on long-horizon reasoning by having the
      model WRITE PYTHON to slice/recurse over a context VARIABLE (REPL-as-context). The decisive conformance
      gate is rubric G6 (workflow/papers/rlm-rubric-recursive-coding-agents-interface.md): WHO chooses the
      decomposition — full-RLM = the LLM at run time; map-reduce / not-RLM = the orchestrator fixes it at
      design time. ELAI is plan-first / FSM-gated / TD-3-deterministic => DELIBERATELY Level-2/3 by G6. COROLLARY
      (binds both units): "be more RLM at the orchestration grain" is an ANTI-GOAL (it would violate TD-3, the
      Workflow-tool ban, HD-2). The only defensible steal is the narrow WITHIN-REASONING-STEP primitive + its
      measurement.
RG-2  LONGCOT BENCH (arXiv:2604.14140, HIGH; workflow/papers/longcot-bench-2604.14140-interface.md). The harness
      GitHub LongHorizonReasoning/longcot is MIT + the HF dataset LongHorizonReasoning/LongCoT is live =>
      PORTABLE, license-clean, ELAI CAN run it. This is the concrete buildable unit (J-107) and doubles as the
      RG-6 measurement instrument.
RG-3  MEASURED exp-1 — LEXICAL long-context recall, WEAK lane (OWN DATA; workflow/experiments/
      rlm-vs-elai-retrieval-2026-06-26.md; groq llama-3.1-8b, W=6000, bands 0.5/2/8W, n=12/band, deterministic
      exact-match). Single-needle accuracy 8xW: A flat 25% | B RLM-model-slicing 62% (erratic, 50-62% across
      bands) | C ELAI deterministic BM25 retrieval 100% | D merge 100% (cheapest tokens/correct). VERDICT: on
      lexical long-context, ELAI deterministic retrieval ALONE captures the whole RLM benefit; RLM did not beat
      it; the weak 8B NEVER engaged the rlm_query recursion primitive (RLM's headline mechanism didn't fire on
      the local lane).
RG-4  MEASURED exp-2 — MULTI-HOP (synonym needle the question never names), CAPABLE lane (OWN DATA;
      workflow/experiments/RLM-VS-ELAI-RESULTS-2026-06-26.md; cerebras gpt-oss-120b; 2-hop dept->person->room).
      Accuracy 8xW: A 12% | B 0% | C 0% | D 25%. THE SIGNAL IS THE FAILURE MODE: C (single-shot retrieval)
      structurally CANNOT multi-hop but ABSTAINS (empty, never fabricates) = fails SAFE; B (free-code RLM) is
      WORST — HALLUCINATED "42" on 7/8 tasks at 8xW, recursion never engaged even on a 120B = fails by
      FABRICATING (the trust-contract-breaking mode); D (deterministic iterative navigation) the ONLY arm >0% at
      8xW, correct-or-abstains, shortfall is step-budget (tunable) not capability.
RG-5  MEASURED ceiling — "CAN WE GET 100% ON 8xW MULTI-HOP?" (OWN DATA; RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md
      §ceiling). ORACLE (pure deterministic traversal, ZERO model calls) = 8/8 = 100% (the answer is ALWAYS
      deterministically reachable; the gap is navigation reliability, NOT reachability). D@18 (model-driven
      navigator, more budget) PLATEAUS: 2xW 25->37%, 8xW 25->25% FLAT (per-step model error compounds over the
      hop chain — you do NOT reach 100% by giving the model more navigation latitude). GRAPH-HOP HYBRID
      (deterministic hops, model ONLY for atomic entity+number read) on a WEAK 8B = 7/8 = 87% at 8xW (the one
      miss was a final number mis-read, not navigation). LEVER = DETERMINISM, not model-nav tuning.
RG-6  J-55 TENSION RESOLVED-BY-MEASUREMENT (the F4 open question). J-55 KEYSTONE forbids routing the sub-14B local
      lane to general CodeAct (~12% multi-tool code-mode collapse <14B, WebSearch-verified 2026-06-20). The RLM
      results show 4B-27B succeeding WITH the scaffold. exp-1/exp-2 measured the resolution: the CONSTRAINED
      named-op path (D) held 100% on the weak lane and never collapsed; the FREE-code path (B) was erratic with
      code-errors and fabricated. So a constrained single-string-slice + capped-child-spawn sits BELOW the
      general code-mode collapse threshold; free-Python RLM is REJECTED on every lane.

================================================================================
RS — ELAI SUBSTRATE ANCHORS (shipped/planned code the units COMPOSE; GD-1)
================================================================================
RS-1  query_with_tier  crates/knowledge/src/pipeline.rs:581  [WIRED]  — Stage-1 deterministic candidate set
      (BM25 + vector + rerank; the real retrieval entrypoint). J-108's candidate set; production-grade superset
      of exp-1's BM25-only Arm C.
RS-2  ToolDispatcher + dispatch_file_read_outline  crates/tools/src/adaptive_tool.rs:295  [WIRED]  — the
      journaled named-op substrate (SSB markers ToolCallStart/Read/Branch/Finding/Converge/Escalate/ToolCallEnd).
      One journaled named op per turn == J-108's Stage-2 shape; replay falls out of the SSB rows for free.
RS-3  SearchLadder::run_best_of_n  crates/orchestrator/src/search/ladder.rs:173  [DARK(ELAI_SEARCH_LADDER_ENABLE),
      search_ladder_enabled() :382]  — the WIDTH template (branch K candidates, rank by receipts, prune, on empty
      survivor set emit BudgetCollapsed -> winner:None at :350). J-86 activates it; J-108 is its DEPTH sibling
      (copy the struct/handle/journal idiom, do NOT re-implement it).
RS-4  hippo_rag.rs::retrieve_notes  crates/memory/src/hippo_rag.rs:57  [WIRED]  — HippoRAG Personalized-PageRank
      over the entity/note graph (Gutiérrez et al. NeurIPS 2024, arXiv:2405.14831 — in-code citation). One PPR
      pass propagates dept-query -> shared person entity -> BOTH hops in a SINGLE deterministic retrieval. This
      is the PRODUCTION form of the RG-5 graph-hop hybrid = J-108's PRIMARY multi-hop path.
RS-5  K-15 `elai bench reasoning-budget` + ConvergenceCurveMeasured + bench_convergence + ConvergencePanel (I-27)
      [PLANNED(K-15) / I-27 DARK]  feature-map 1.13:2266-2287 — the bench RUNNER SHAPE J-107 reuses (bootstrap-CI,
      ELAI_SEED, receipts on the y-axis). J-107 is its LongCoT/recursive-scaffold arm (x-axis = imposed
      attention/recursion budget), GD-1 do-NOT-fork-the-runner.
RS-6  J-42 within-step distill-reset [PLANNED], J-38 mid-task retrieval loop [PLANNED(J-38)], J-86 receipt-gated
      WIDTH ladder [PLANNED], I-38 QDELTA quality bench [PLANNED], J-34 empirical context calibration
      [WIRED/PARTIAL] — the DEPTH(J-42/J-38) x WIDTH(J-86) axes + the measurement siblings J-108/J-107 extend.
RS-7  J-55 model-strength modality router [PLANNED, KEYSTONE]  — the lane guard: the Qwen-7B local lane is NEVER
      routed to CodeAct. J-108's NavOp set is fixed + the executor deterministic; the local lane runs the
      named-op + HippoRAG path only (text, no free code), capable-lane default for any recursive SubQuery.
      LIVE SCHEMA baseline at authoring: ~268 / ~577 (DERIVED, crates/orchestrator/src/ssb.rs:1508 =
      SCHEMA_VERSION_DDL_EPOCH(-309) + len(SSB_ACTION_TYPES); NEVER hard-code, re-derive at edit, W-9).

================================================================================
RC — DESIGN-DOC CORRECTIONS (use THESE, not the older map line / the naive framing)
================================================================================
RC-1  SUPERSEDES-BUT-SCOPES the feature-map line "RLM REPL-as-context: DEFER (only _compact_history taken)"
      (ELAI-FEATURE-MAP.md ~3016, J-76). The defer is REVISITED ONLY as a CONSTRAINED within-reasoning-step
      primitive (a single string-slice + one capped rlm_query-style child-spawn over the SHIPPED orchestrate/BSP
      sub-agent machinery), capable-lane DEFAULT, local-lane GATED on the J-107 measurement. The defer on
      "RLM-as-everything / free-Python REPL / model-chosen orchestration" STANDS (G6 anti-goal, RG-1). J-108 is
      an EXTENSION(J-42/J-86/J-38), not a new architecture.
RC-2  CORRECTS the naive "the model-driven navigator gets you to 100%". Per RG-5 model-driven navigation
      PLATEAUS ~25% at 8xW; the path to ~100% is DETERMINISTIC traversal. Therefore J-108 PRIMARY = HippoRAG
      (RS-4, already shipped) + atomic model extraction; the model-driven RetrievalNavigator (NavOp loop) is the
      FALLBACK for cases with NO clean entity linkage (caps ~25-37%, the safety net). The model NEVER navigates
      by writing code (RG-4 free-code = 0% + hallucinated).
RC-3  ARXIV IDS GROUNDED VIA ON-DISK INTERFACE MAPS (not training memory): RLM 2512.24601 =
      workflow/papers/rlm-interface.md (HIGH); LongCoT 2604.14140 = workflow/papers/longcot-bench-2604.14140-
      interface.md (HIGH); HippoRAG 2405.14831 = NeurIPS 2024, in-code citation at hippo_rag.rs. The supporting
      blog/raw.works items (MGH / code-execution-as-reasoning / RLMs-new-reasoning) are MED-LOW reliability =
      CONFIRMATION grounding only, never a unit (banked as the Section-4 verdict block + a K-1 [EXTERNAL] row).

================================================================================
CARRIED WEB LANDMINES (external numbers — design rationale, NEVER targets; HD-1/ID-5)
================================================================================
- RLM headline numbers (Qwen3-4B 0->100 on MRCRv2, Qwen3.5-9B/27B beat GPT-5.2 on LongCoT, Codex 72.5% /
  Opus-4.7-Mini 75.4%) are SINGLE-SOURCE LOW reliability. They MOTIVATE the local-lane wedge; the J-107/J-108
  done-gates produce ELAI-MEASURED deltas per-(model,lane), never these figures.
- the RLM "Math/Chemistry ~0" prediction was REFUTED by the raw.works follow-up (a Claude-Code prompt
  improvement closed it) — reinforces "the scaffold not the model is the lever," not a target.

================================================================================
CANONICAL CONSTANTS + SSB DELTAS (both units PREFER-REUSE)
================================================================================
LIVE ~268/577 DERIVED (ssb.rs:1508; re-derive at edit, W-9).
  J-107  NO new action_type — reuse ConvergenceCurveMeasured (I-27) +Kind LongCotArm if a discriminator is
         needed (decide at edit, lockstep). baseline+0..+1.
  J-108  NO new action_type — reuse SearchNodeEvent (I-4/J-86; Promoted / BudgetCollapsed) + the ToolDispatcher
         markers (RS-2) for each NavOp; HippoRAG retrieval rides query_with_tier's existing rows. baseline+0..+1.

End of J-RLM-GROUNDING-MAP.
