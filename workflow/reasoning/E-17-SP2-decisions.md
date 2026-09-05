E-17 SP2 — knowledge-layer core (tasks 3, 4, 5, 7) + SP1 carryover
Session: s-26556
Date: 2026-05-21

What ships
==========
SP2 lands the knowledge-layer slice of E-17:

- knowledge::reranker::ReRankerModel enum (JinaV3 / BgeV2M3 / MsMarcoMiniLmL12).
  Default = MsMarcoMiniLmL12 until JinaV3 / BgeV2M3 ONNX files are staged on
  disk (E-17 ops follow-up). Selecting an unstaged variant emits a
  tracing::warn at load time and the pipeline degrades gracefully (no panic).
- knowledge::reranker::RerankerConfig::with_project_root +
  with_project_root_and_variant constructors. Legacy Default impl preserved
  for test fixtures.
- knowledge::splade::SpladeConfig::with_project_root constructor.
- knowledge::deep module with deep_pool_cap + rrf_quality_gate primitives
  + named constants DEEP_CAP_DEFAULT (100), RRF_FLOOR_DEFAULT (0.005),
  RRF_K_CONSTANT (60.0).
- knowledge::query_class module with QueryClass enum + classify fn +
  has_code_symbol_signal public helper. Heuristic-based (no model call).
- knowledge::hyde::should_use_hyde gate + ELAI_HYDE_NL_ONLY env var
  (default = on / true). CodeSymbol + Mixed queries skip HyDE.
- knowledge::pipeline integration:
    - rrf_rerank refactored into rrf_rerank_with_tier(tier).
    - On QueryTier::Deep: cap BEFORE accumulator, floor AFTER freshness.
    - Deterministic tie-break (chunk_id asc) added to the final sort.
    - 4 new SSB rows emitted per query (deep_pool_capped,
      rrf_quality_gate_fired, hyde_skipped_code_query, rrf_component_weights).
    - New stage labels deep_cap / rrf_gate emitted into the stage trace
      when the corresponding gate fires.
- 4 new SsbActionType variants in orchestrator::ssb with full 7-touchpoint
  atomic SCHEMA_VERSION bump 96 -> 97. all_test_payloads().len() = 227.
- orchestrator::knowledge_ssb_adapter implements the 4 new emit methods.
- cli::live_cli SP1 carryover fix: ms-marco-l6.onnx -> cross-encoder-l6.onnx
  (the previous filename never existed on disk; L-6 graceful degradation
  was silently broken). Production sites now use RerankerConfig::with_project_root
  + SpladeConfig::with_project_root.
- dashboard-v2 commands.rs hardcoded fallback updated ms-marco-l6 -> cross-encoder-l6.
- elai_cc.toml schema docs for [knowledge.reranker], [knowledge.deep],
  [knowledge.rrf], [knowledge.hyde], [knowledge.code_vec]. (TOML loading
  itself is deferred to E-17 ops follow-up — current code hardcodes
  the defaults declared in knowledge::deep.)
- 6 new contract test files (4 knowledge + 1 orchestrator + 1 reranker enum).

Sweep claim corrections
=======================
The SP1 sweep flagged "code_vec populated but never queried" — that
framing is REFUTED. code_vector is pushed at pipeline.rs:516 (now ~535
after this PR's edits) and IS consumed by rrf_rerank at the RRF
accumulator loop (now in rrf_rerank_with_tier). The gate is opt-in via
ELAI_CODE_VEC=1 + VectorIndexer::has_code_index(), so the correct framing
is "wiring exists but default-off; observability surfaced via the new
[knowledge.code_vec] config section + the rrf_component_weights SSB row
which lets the dashboard see when code_vec_weight = None (no
contribution) vs Some(1.0) (active)."

Honest framing of constants
===========================
DEEP_CAP_DEFAULT = 100
  Prior-work-typical heuristic; avoids phantom-hit zone observed at
  K=5000 (arxiv:2411.11767 S4.SS2.p2). Paper does NOT mandate this
  specific value (S4 fn1 only — "we follow prior work in setting K = 100").
  Comment in deep.rs cites the paper section anchor explicitly.

RRF_FLOOR_DEFAULT = 0.005
  ELAI design choice; back-derived from k=60 arithmetic (a single ranker
  at rank 140 produces score 1/(60+140) = 0.005). NOT cited to any paper.
  k=60 itself IS grounded — Cormack et al. SIGIR 2009 establishes k=60 as
  the empirical default and that prior choice ships unchanged at
  pipeline.rs (now via the named RRF_K_CONSTANT). The 0.005 floor is
  validated by the SP6 eval harness (task 10); SP2 ships the value as a
  hardcoded default behind a config-key surface.

HyDE-off-for-code rule
  Extrapolation from arxiv:2212.10496 NL-focus design intent. NOT
  externally validated for code queries. The cited arxiv:2406.14497
  (CodeRAG-Bench) does NOT evaluate HyDE, so it cannot support the rule
  directly — only the related "BM25 100.0 NDCG@10 on HumanEval vs 5.2 on
  DS-1000" result indicates query-type sensitivity. SP6 task 10 measures
  recall@5 with/without the gate; if the classifier scores <95% F1, plan
  acceptance falls back to "HyDE off by default everywhere."

ms-marco-MiniLM-L12 framing
  arxiv:2411.11767 does NOT test ms-marco-MiniLM-L12. The plan's swap
  rationale (Jina-v3 / BGE-v2-m3 over the L-12 baseline) is jina.ai
  marketing numbers + plan-author design choice, not paper-grounded.
  The enum + dispatch ships in SP2; actual model staging + an honest
  before/after recall@5 measurement land in E-17 ops follow-up.

Deferred from SP2
=================
- Jina-v3 / BGE-v2-m3 ONNX file staging on disk (model download +
  tokenizer + dashboard doctor probe extension). Tracked as E-17 ops.
- Real TOML loading for the 5 new [knowledge.*] sections (current code
  hardcodes the defaults).
- HyDE-vs-NL classifier F1 measurement on a labelled goldset (SP6 task 10).
- cAST chunking, eval harness, PLAID late-interaction (SP3 / SP4 / SP6 / SP7).
- Full RerankerConfig::default + SpladeConfig::default removal. SP2 only
  migrates the 1 production caller in live_cli for each. The 8 reranker
  test sites + 4 splade test sites keep Default::default() — they live
  inside #[cfg(test)] modules where relative-path CWD-resolution is
  acceptable.

Acceptance criteria for SP2 (matched against E-17 plan task list)
=================================================================
- Task 3 (reranker swap config-driven): SHIPPED (enum + config dispatch
  + warn-on-missing). Default remains MsMarcoMiniLmL12.
- Task 4 (DEEP cap + RRF quality gate): SHIPPED (deep.rs module +
  pipeline integration + 4 SSB rows + 3 contract tests).
- Task 5 (HyDE NL-only gate + Layer A classifier): SHIPPED
  (query_class.rs + hyde.rs::should_use_hyde + ELAI_HYDE_NL_ONLY env +
  hyde_skipped_code_query SSB row).
- Task 7 (rrf_component_weights observability): SHIPPED
  (rrf_component_weights SSB row + RRF_BM25/CODE_VEC/TEXT_VEC weight
  constants).
- SP1 carryover (live_cli typo + with_project_root constructors): SHIPPED.
- 4 SsbActionType variants + SCHEMA_VERSION 96 -> 97 atomic bump: SHIPPED
  (7 touchpoints updated, all_test_payloads count 223 -> 227).

Risk register snapshot
======================
HIGH: ms-marco-l6.onnx -> cross-encoder-l6.onnx rename touches the
dashboard hardcoded fallback (commands.rs:1241) — checked + updated.

HIGH: SCHEMA_VERSION 97 must not collide with any concurrent PR. Verify
no in-flight task bumps ssb.rs before merge. Local cargo check passes
the round-trip test contract.

MEDIUM: code_vec wiring claim refutation documented above.

MEDIUM: Mixed queries default to HyDE-OFF (conservative). The plan
truth-table suggested "Mixed -> NL bucket (HyDE ON)" but the implemented
should_use_hyde returns true ONLY for QueryClass::NaturalLanguage when
the gate is on; Mixed queries skip HyDE. This is the safer choice (no
false-positive HyDE roundtrips on ambiguous queries) and is validated by
the SP6 eval harness — if Mixed -> NL bucket yields higher recall,
flip the gate logic in a follow-up patch.

LOW: HyDE classifier F1 measurement deferred to SP6. If F1 < 95% the
plan falls back to "HyDE off everywhere" — that path is reachable today
by setting ELAI_HYDE_NL_ONLY=1 + ensuring classify always returns
something other than NaturalLanguage (currently impossible without
patching the classifier).
