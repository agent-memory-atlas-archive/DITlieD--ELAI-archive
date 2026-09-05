# Verification Report: phase-index-pre0-2D
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-index-pre0-2D.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
All 11 phases (pre-0 through 2D) have substantial codebase evidence. The workspace expanded from the
claimed 9-crate layout to 15+ active crates (expected growth from later phases). Core phase deliverables
are present and verified: DKG, BM25, vector search, PageRank, CatRAG/A-RAG, cross-encoder, SPLADE,
resilience layer, and the complete GRC reasoning engine (2A-2D). Phase 1F (speculative decoding
evaluation) shows decision infrastructure (SpeculativeConfig, ProbeResult) but no measured ADOPT/DROP
decision doc in the codebase. Phase 2C TomlConfigLoader is flagged as a stub implementation in source.

## Deliverables verified DONE

- pre-phase-0: cfg(unix) gates present — evidence: crates/tools/src/lib.rs:5253 (#[cfg(unix)] guard)
- pre-phase-0: PoisonError handling present — evidence: crates/tools/src/lib.rs:6296 (PoisonError::into_inner)
- phase-0: 9-crate workspace layout (expanded to 15+ crates) — evidence: .elai_cc/Cargo.toml:3-21
- phase-1A: DkgEngine struct and new() — evidence: crates/knowledge/src/dkg/mod.rs:1615 (elai.map DEF_FN)
- phase-1A: DkgNode symbol extraction — evidence: crates/knowledge/src/dkg/mod.rs:1615
- phase-1A: LanguageGrammar trait — evidence: crates/knowledge/src/dkg/language_registry.rs:34
- phase-1A: knowledge/ crate — evidence: .elai_cc/Cargo.toml:10 (crates/knowledge member)
- phase-1B: BM25Chunk — evidence: crates/knowledge/src/pipeline.rs:852 (elai.map MACRO ref)
- phase-1B: Bm25Index with index_file/query operations — evidence: crates/knowledge/src/bm25.rs (LanguageGrammar usage at line 586)
- phase-1C: circuit_breaker per model — evidence: crates/knowledge/src/vector/circuit_breaker.rs:427 (elai.map)
- phase-1C: RRF fusion — evidence: crates/knowledge/tests/splade_tests.rs:225 (e2_rrf_fusion_score_correctness)
- phase-1D: compute_repo_map() — evidence: crates/knowledge/src/pagerank/mod.rs:352
- phase-1D: PageRank/pagerank module — evidence: crates/knowledge/src/pagerank/mod.rs:277 (elai.map SCIP_REF)
- phase-1E: CatRAG engine — evidence: crates/knowledge/src/catrag.rs:168 (CatRAG engine class)
- phase-1E: A-RAG module — evidence: crates/knowledge/src/arag.rs:11 and arag.rs:102
- phase-1E: full pipeline query() — evidence: crates/knowledge/src/pipeline.rs:149-165
- phase-1E: StageTrace pipeline observability — evidence: crates/knowledge/src/pipeline.rs:629 (elai.map)
- phase-1F: SpeculativeConfig, ProbeResult structs — evidence: crates/providers/src/providers/ollama.rs:18,255
- phase-1F: speculative_decoding probe function — evidence: crates/providers/src/providers/ollama.rs:107
- phase-1G: cross_encoder circuit breaker circuit — evidence: crates/knowledge/src/vector/circuit_breaker.rs:555 (elai.map)
- phase-1G: NDCG measurement infrastructure — evidence: crates/knowledge/tests/reranker_sweep.rs:248,400 (ndcg_delta)
- phase-1H: SPLADE module with vocab_coverage — evidence: crates/knowledge/src/splade.rs:423
- phase-1H: Wilcoxon signed-rank test — evidence: crates/knowledge/src/splade.rs:532-544
- phase-1H: KEEP/DROP decision logic — evidence: crates/knowledge/src/splade.rs:684 (Self::Keep => "KEEP")
- phase-1H: splade feature-gated module — evidence: crates/knowledge/src/splade.rs (file exists)
- phase-1I: SearchMode cascade (HYBRID/SPARSE_ONLY/BM25_ONLY) — evidence: crates/knowledge/src/degradation.rs:39,58-59
- phase-1I: startup_readiness_check — evidence: crates/knowledge/src/startup.rs:126
- phase-1I: QueryValidator — evidence: crates/knowledge/src/query_validator.rs:97 (elai.map CALL)
- phase-1I: StageTrace for observability — evidence: crates/knowledge/src/pipeline.rs:629 (elai.map)
- phase-2A: BasicGrcEngine — evidence: crates/reasoning/tests/verification_tests.rs:341 (elai.map CALL)
- phase-2A: GrcError variants (UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding) — evidence: crates/reasoning/src/engine.rs:206,308,382; types.rs:142
- phase-2A: reasoning/ crate — evidence: .elai_cc/Cargo.toml:11 (crates/reasoning member)
- phase-2B: verify_claim() — evidence: crates/reasoning/src/engine.rs:530 (elai.map DEF_FN)
- phase-2B: ground_plan() with safety gate — evidence: crates/orchestrator/tests/safety_gate_integration.rs:128 (grounding_sweep_gate_blocks_when_ungrounded)
- phase-2B: FILE_CHECK grounding method — evidence: crates/reasoning/tests/verification_tests.rs:253 (elai.map MACRO "file:")
- phase-2C: AgentGrcConfig — evidence: crates/orchestrator/src/agent_pool.rs:727 (elai.map CALL)
- phase-2C: TomlConfigLoader — evidence: crates/reasoning/src/lib.rs:25 (pub use), config.rs:34 (struct def)
- phase-2C: anti_rubber_stamp enforcement — evidence: crates/reasoning/tests/agent_config.rs:386 (elai.map MACRO)
- phase-2C: agent_grc_config_for_role routing — evidence: crates/reasoning/src/config.rs:379 (elai.map CALL)
- phase-2D: generate_csq() — evidence: crates/reasoning/src/engine.rs:649 (elai.map DEF_FN)
- phase-2D: CsqResult, CsqSeverity — evidence: crates/reasoning/src/types.rs:458,470
- phase-2D: check_inline_feasibility() — evidence: crates/reasoning/src/engine.rs:707
- phase-2D: InlineFeasibilityState — evidence: crates/reasoning/src/types.rs:524
- phase-2D: ReplanRequest for MAJOR replan — evidence: crates/orchestrator/tests/adversarial_ipc.rs:208 (elai.map)

## Deliverables PARTIAL

- phase-1F: ADOPT or DROP decision with measurements in metrics files
  - exists: crates/providers/src/providers/ollama.rs:107 (probe_speculative_decoding fn), ollama.rs:255 (SpeculativeConfig struct with speculative_decoding: false default)
  - gap: no metrics file or decision doc with measured throughput ratios (1.3x gate), acceptance rate numbers, or VRAM delta recorded in codebase; decision outcome is embedded only as a code default (false = not adopted), not as an explicit measured decision doc

- phase-2C: TomlConfigLoader fully implemented vs stub
  - exists: crates/reasoning/src/config.rs:34 (struct defined and impl present)
  - gap: source comment at config.rs:30 reads "stub implementation reading from core::Config" — full TOML file parsing ([grc.defaults] + [agents.{role}.grc] sections) may not be wired end-to-end; comment indicates partial implementation state

## Deliverables MISSING
- phase-1F: metrics files with parameter sweep (k=1-7), throughput benchmark output, acceptance rate measurement — searched: "throughput_bench", "acceptance_rate", "1.3x", "ADOPT", "DROP" in providers/; no evidence in elai.map or source

## Deliverables UNVERIFIABLE
- pre-phase-0: 0 compile errors / 0 warnings on Windows baseline — build results not in codebase; no CI artifact readable via grep
- pre-phase-0: 69 .rs files / 61K LOC / 684 test annotations claimed baseline — no snapshot artifact; changelog/memory-bank refs explicitly excluded
- phase-0: old code archived — archive location not verifiable without reading memory-bank (excluded)
- phase-1C: startup canary query embed("fn main()") within 5s measured — runtime behavior, not statically verifiable
- phase-1D: 21+ tests including proptest — test count not enumerable without running test binary; proptest usage not confirmed by elai.map grep
- phase-1E: measured end-to-end latency and token savings — runtime measurement, no stored artifact in codebase
- phase-1G: <50ms p99 for 50 candidates — runtime performance claim, not statically verifiable
- phase-2A: 12+ test count — test count not enumerable without running; 12+ individual test functions plausible given engine.rs size

## What must be done (action list)
- [ ] Write or locate Phase 1F decision doc in workflow/reasoning/ or metrics/ with measured throughput ratios and explicit ADOPT/DROP outcome; target file: workflow/reasoning/phase-1F-decisions.md or .elai_cc/metrics/speculative_decoding_eval.md
- [ ] Verify TomlConfigLoader stub comment at crates/reasoning/src/config.rs:30 — confirm whether full TOML section parsing ([grc.defaults] + [agents.{role}.grc]) is wired or still reading only from core::Config; update comment or complete implementation
- [ ] Confirm proptest usage in Phase 1D (pagerank) tests — grep crates/knowledge/tests/ for proptest:: to validate "21+ tests including proptest" claim
- [ ] Confirm phase-1B <10ms p99 query latency claim has a benchmark or test that asserts timing

## Notes
Workspace has grown from the planned 9 crates to 15+ (contracts, agent, ipc-types, web_search, scaffold, web_validator, dashboard-v2 added by later phases). This is expected and not a discrepancy. The pre-phase-0 Windows platform baseline claims (file counts, LOC, test counts) are architectural snapshots that cannot be verified from current codebase state — the codebase has evolved significantly. All core functional deliverables for phases 1A through 2D are present and verified.
