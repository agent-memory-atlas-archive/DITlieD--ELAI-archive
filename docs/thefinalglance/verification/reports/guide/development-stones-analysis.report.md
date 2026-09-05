# Verification Report: development-stones-analysis
Source: F:\.ELAI_workflow\docs\specs\guide\development-stones-analysis.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The analysis document identifies 12 "stones" (risks) with specific claims about spec mechanisms and implementation targets. The majority of mechanisms cited as "already addressed in spec" are verifiably implemented in .elai_cc/. The core anti-thrashing pipeline (MistakeBook, anti-spiral, 6-layer reasoning, temperature profile, AB-MCTS), the knowledge stack (BM25/tantivy, DKG, PageRank, SPLADE, ONNX reranker), cloud mode (privacy redactor, HYBRID/CLOUD routing, air-gapped mode), and rules lifecycle (PROPOSED/CANDIDATE/SHADOW/ACTIVE, dep_audit, WAL) are all implemented. Key gaps: progressive indexing (cold start) is not implemented in the staged BM25-first/DKG-async form described; no evidence of BYOE (bring-your-own-endpoint) for enterprise trust gap; hardware auto-detection on first launch as a mode-selection routing mechanism is unverifiable from current evidence. Stone D (testing bootstrap for legacy code) mechanisms are not implemented.

## Mechanisms verified DONE

- MistakeBook (Blake3 hash of failed diffs, forbidden diff injection, eviction) — evidence: crates/core/src/mistake_book.rs:318, crates/orchestrator/src/mistake_book.rs:76
- FailureTaxonomy regex classification — evidence: crates/tools/src/failure_taxonomy.rs:245
- Anti-spiral detection (3x same step escalation, SpiralDetected SSB payload) — evidence: crates/orchestrator/src/anti_spiral.rs:229, crates/contracts/tests/anti_spiral_runner_contract.rs:110
- AB-MCTS per-VRAM-tier branch limits (L4 reasoning layer) — evidence: crates/core/src/vram_tier.rs:103 (MAX_MCTS_BRANCHES_12GB), crates/contracts/tests/vram_tier_contract.rs:73
- Temperature 1.0 for repair/generation profile — evidence: crates/orchestrator/src/context_budget.rs:2757 (assert temperature == 1.0)
- 6-layer reasoning layers tracked via VramTierConfig.available_layers — evidence: crates/core/src/vram_tier.rs:143 (assert available_layers.len() == 6)
- Cloud mode routing (LOCAL/CLOUD/HYBRID) with privacy redactor — evidence: crates/orchestrator/src/cloud_mode.rs:171, crates/orchestrator/src/privacy_redactor.rs:126 (redact_context)
- Air-gapped mode (AirGappedConflict error, AIR_GAPPED_ENV var, focus_tunnel blocking) — evidence: crates/orchestrator/src/cloud_mode.rs:498, crates/contracts/tests/focus_tunnel_e6_contract.rs:447
- WAL mode for SSB (concurrent reads, single-writer) — evidence: crates/orchestrator/src/ssb.rs:4268 (pragma journal_mode WAL), crates/orchestrator/src/ssb.rs:6084 (test_single_writer)
- WAL mode for DKG — evidence: crates/knowledge/src/dkg/mod.rs:188 (PRAGMA journal_mode=WAL)
- ModelPool with VRAM availability tracking and predictive preload — evidence: crates/orchestrator/src/agent_pool.rs:473, crates/orchestrator/src/preload.rs (PreloadManager, trigger_preload_background)
- PerRoleRuntime and RuntimeOrchestrator (B-37, multi-runtime backend abstraction) — evidence: crates/orchestrator/src/per_role_runtime.rs:137, crates/orchestrator/src/runtime_orchestrator.rs:172
- DKG implemented with multiple languages (Rust, Python, JS, TS, Vue, Scala, PHP, Swift, Bash) — evidence: crates/knowledge/tests/multilang_extraction.rs:1 (B-30 7 new language extractors), crates/contracts/tests/dkg_multilang_contract.rs:230
- BM25/tantivy indexing operational — evidence: crates/knowledge/src/bm25.rs:990, crates/knowledge/src/pipeline.rs:949 (bm25_ok assertion)
- PageRank on DKG graph — evidence: crates/knowledge/src/pagerank/mod.rs:277, crates/contracts/tests/dashboard_tier3_drawers_contract.rs:154 (pagerank_top)
- ONNX reranker (cross-encoder-l12) — evidence: crates/knowledge/src/pipeline.rs:839
- SPLADE sparse keyword expansion — evidence: crates/knowledge/src/splade.rs:702, crates/cli/src/live_cli.rs:1025
- Startup readiness gate with BM25 canary query — evidence: crates/knowledge/src/startup.rs:425 (startup_readiness_check), crates/knowledge/src/pipeline.rs:847 (BM25 canary)
- IndexingService with resumable index build — evidence: crates/orchestrator/src/indexing_service.rs:358 (build_index_resumable), crates/orchestrator/tests/indexing_resume.rs:56
- Rules lifecycle PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE — evidence: crates/rules/src/lifecycle.rs:682-788
- deprecated_patterns table in rules.db — evidence: crates/rules/src/schema.rs:912 (query_deprecated_patterns)
- Dep audit (crates.io, PyPI fetching, major-version-only outdated detection) — evidence: crates/cli/src/dep_audit.rs:383 (query_crates_io), crates/cli/src/dep_audit.rs:396 (query_pypi), crates/cli/src/dep_audit.rs:821 (is_outdated major-bump only)
- PrivacyConfig / redact_context integrated into cloud mode path — evidence: crates/orchestrator/src/cloud_mode.rs:435 (privacy_redactor::redact_context called in cloud dispatch)
- VramTier detection via NVML — evidence: crates/orchestrator/src/system_probe.rs:298 (probe_gpu_shell_out), crates/contracts/tests/provider_signals_contract.rs:196 (detected_via nvml)
- Ollama as fallback provider with degraded mode — evidence: crates/cli/src/repl.rs:133 ("Ollama detected. Entering degraded mode."), crates/providers/src/runtime/ollama.rs
- Context Telescope (collapse to budget) — evidence: crates/orchestrator/src/telescope_caller.rs:161 (collapse_to_budget), crates/contracts/tests/a46_context_telescope_contract.rs:25
- SPARSE_ONLY and BM25_ONLY graceful degradation modes — evidence: crates/knowledge/src/reranker.rs:2008, crates/knowledge/src/vector/mod.rs:309 (graceful_degradation)
- Evidence gates (gate checks, gate2/gate3/gate7 in evidence.rs) — evidence: crates/orchestrator/src/evidence.rs:767-1030
- AgentRole-based dispatch (Auditor, Executor, etc.) — evidence: crates/orchestrator/src/agent_pool.rs:887, crates/orchestrator/src/auditor_mode.rs:84
- Project Intelligence multi-language detection — evidence: crates/knowledge/src/project_profile.rs (framework detection, DKG extraction with fallback)
- Cloud mode audit trail (SSB comment noting audit trail for cloud escalation) — evidence: crates/orchestrator/src/ssb.rs:1897 (doc comment: "audit trail for why did this step go to the cloud")

## Mechanisms PARTIAL

- Cold start / progressive indexing (Stone A)
  - exists: BM25 indexing via IndexingService (build_index_resumable at indexing_service.rs:358); DKG built at end of index build (indexing_service.rs:354 comment: "DKG: built once at the end after all BM25 batches complete"); startup_readiness_check with BM25-only fallback mode (startup.rs:530 "implies_bm25_only")
  - gap: No dedicated "Phase 1 BM25 in <5s, Phase 2 DKG imports-only, Phase 3 hot-files-only, Phase 4 background" staged rollout. The BM25-first-then-DKG ordering is internal sequencing within one index pass, not a progressive user-visible state machine. No git-aware file prioritization or on-demand per-task pre-indexing found.

- Self-discovery rule proposal (Stone 6 — self-mod 3+ violations -> auto-propose rule)
  - exists: SelfModBridge, SelfModOrchestrator, DataTable::FailureTaxonomy proposals (crates/orchestrator/src/self_mod_bridge.rs:238), self_mod_layer1a with shadow testing (crates/orchestrator/src/self_mod_layer1a.rs:1464)
  - gap: Spec claims "when auditor catches same violation 3+ times, auto-proposes a new rule." The failure_taxonomy_threshold mechanism sums counts across records (self_mod_bridge_contract.rs:183), but the exact 3+ trigger tied specifically to FailureTaxonomy miss rates auto-generating new deprecated_patterns rules was not confirmed end-to-end.

- Hardware auto-selection on first launch (Stone 1 solution direction 5)
  - exists: VramTier detection via NVML (system_probe.rs:298); VramTierConfig per tier (vram_tier.rs); cloud/local/hybrid mode routing (cloud_mode.rs)
  - gap: No evidence of an explicit first-launch wizard that probes GPU/CUDA/VRAM and automatically selects cloud vs ollama vs sidecar mode as the UX entry point.

## Mechanisms MISSING

- BYOE (Bring Your Own Endpoint) for enterprise trust gap (Stone F solution direction 1) — searched: BYOE, bring_own_endpoint, custom_endpoint, user_endpoint; no evidence in elai.map
- Observation-based / characterization test generation for legacy code (Stone D solution direction 1) — searched: characterization_test, observation.*test, instrument.*entry, capture_io; no evidence. Stone D mechanisms (weaker evidence gates for zero-test codebases, human-assisted test bootstrapping, observation-based recording) are not implemented.
- Adaptive futility threshold per error category (Stone 5 solution direction 1) — searched: adaptive.*futility, success_rate.*category, historical.*success; no evidence of per-category retry budget tracking
- Community rule registry / language pack format for sharing (Stone E solution direction 3, Stone 6 solution direction 1) — searched: community_rule, rule_registry, language_pack, rule_pack; no evidence of a published community contribution mechanism
- Tiered LSP-backed DKG for dynamic language semantics (Stone 3 solution direction 1 and Stone E solution direction 1) — searched: LSP.*DKG, pyright, tsserver, gopls, language.*server.*dkg; no evidence. DKG is tree-sitter-only (confirmed via knowledge/src/dkg/mod.rs and multilang_extraction.rs)

## Mechanisms UNVERIFIABLE

- "50K inserts/sec ceiling documented" for SSB — SSB WAL confirmed but the specific throughput figure is a spec claim; no benchmark fixture found via elai.map grep
- DoLa HARMFUL on 7B (-15% code quality), steering vectors NEUTRAL — measurement claim from A-1 baseline harness; memory context only, not verifiable from code alone
- 3B 72% pass@1 / 7B 90% pass@1 baseline measurements — same as above; empirical harness results, memory-bank only

## What must be done (action list)

- [ ] Implement staged progressive indexing: expose BM25_READY status before DKG completes; gate UI on BM25_READY not full COMPLETE (crates/orchestrator/src/indexing_service.rs + crates/knowledge/src/startup.rs)
- [ ] Implement git-aware file prioritization in IndexingService.build_index_resumable (crates/orchestrator/src/indexing_service.rs:358) — index recently-modified files first
- [ ] Add BYOE support: allow user-configured base_url override in cloud_mode CloudConfig (crates/orchestrator/src/cloud_mode.rs) to point at self-hosted vLLM/Ollama server
- [ ] Wire adaptive futility threshold: track per-error-category retry success rates in SSB and expose to repair-loop escalation logic (crates/orchestrator/src/task_runner.rs)
- [ ] Implement weaker evidence gates for zero-test codebases: detect test count via ProjectIntelligence and select relaxed gate variant in crates/orchestrator/src/evidence.rs
- [ ] First-launch mode auto-selection: add startup UX that reads VramTier detection result and recommends LOCAL/CLOUD/HYBRID mode (crates/dashboard-v2/ + system_probe.rs)
- [ ] Verify/implement 3+ FailureTaxonomy miss -> deprecated_patterns auto-proposal end-to-end in SelfModBridge (crates/orchestrator/src/self_mod_bridge.rs)

## Notes
Stone 4 (VRAM swapping) and Stone 5 (thrashing) are assessed LOW risk in the document; both are verifiably well-addressed. Stone B (3B quality cliff) is a configuration/deployment decision (7B as default on 12GB); the runtime infrastructure (VramTier, PerRoleRuntime, ModelPool) that enables this choice is implemented. Stone 2 (ecosystem matrix) is partially addressed: Rust and Python dep_audit exist; JSON-mode parsing is used selectively; community parsers and versioned parser modules are not implemented.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\development-stones-analysis.report.md | status=MOSTLY_DONE | done=31 | partial=3 | missing=5 | unverifiable=3
