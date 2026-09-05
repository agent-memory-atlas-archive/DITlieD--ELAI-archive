# DEEP-B-002 exhaustive feature discovery

## Scope, method, and evidence semantics

- Canonical scope is exactly the eight originals listed under <code>### DEEP-B-002</code> in <code>DEEP-SOURCE-MANIFEST.md</code>. All eight were read from line 1 through EOF; no B-plan original from another chunk was opened.
- Orientation-only context was <code>02-B-PLAN-SUMMARY.discovery.md</code>, read line 1 through EOF. It is non-canonical: this ledger retains canonical-file details and explicitly records summary additions, omissions, and contradictions.
- <strong>[E]</strong> means explicit in the named canonical source. <strong>[I]</strong> means a conservative cross-file inference, never an independent promise. <strong>[A]</strong> means the source contains an unresolved ambiguity, stale requirement, or direct conflict. <strong>[SΔ]</strong> denotes a delta against the orientation summary.
- Line anchors are one-based canonical-source lines. Historical V1 requirements remain recorded when the same source later supersedes them; the later rebaseline/closure is treated as the current statement only where the file explicitly says so.
- “Atomic item” below means one uniquely identified ledger bullet. Values deliberately remain together only when they form one schema, ordered workflow, exact acceptance command, or inseparable choice.

## Manifest verification and exact processed-path/hash record

All canonical manifest rows match the files byte-for-byte. “Lines” are <code>wc -l</code> logical lines and “Bytes” are <code>wc -c</code>.

| Role | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| Canonical original | 218 | 9,817 | <code>44e91be8da1ea255ff353df2cada61ac79c23723e83315906fb0822e1827286c</code> | <code>docs/thefinalglance/B-plan/B-1_path_unification.md</code> | exact match |
| Canonical original | 245 | 11,543 | <code>8c7cfdac376353b926a43f521ca9f3568ebefc285e8318e0ff37f0f767728c5b</code> | <code>docs/thefinalglance/B-plan/B-2_fsm_phasetransition_wiring.md</code> | exact match |
| Canonical original | 338 | 16,931 | <code>a7c5f0acde93adce4e4fba95e958a83ef0ae4e99fb4636781b815afe323b38b2</code> | <code>docs/thefinalglance/B-plan/B-4_knowledge_producer_wiring.md</code> | exact match |
| Canonical original | 194 | 8,689 | <code>bdc5721c92765943a58047c47943aa5d193c8ff0637a67a977a573591ad74ec1</code> | <code>docs/thefinalglance/B-plan/B-12_shell_vs_workspace_decision.md</code> | exact match |
| Canonical original | 165 | 6,493 | <code>061c55e89a7018b322450a0b65646e1c02497f45baa8d70dbbe3cc09a1a2a61c</code> | <code>docs/thefinalglance/B-plan/B-24_phase_4I_media_validator.md</code> | exact match |
| Canonical original | 465 | 22,365 | <code>16e2210616db9b83d8b6b95cbfbf98b5b6f89f03b05923eb099e9671d403a174</code> | <code>docs/thefinalglance/B-plan/B-31_resumable_indexing_lock_lifecycle.md</code> | exact match |
| Canonical original | 402 | 19,213 | <code>f42601ae4f2501b2b2f309ba2141002d821b1b3fb2bc871072867ee369651289</code> | <code>docs/thefinalglance/B-plan/B-34_entry_point_detection.md</code> | exact match |
| Canonical original | 560 | 28,185 | <code>712dcaa6bb4bd4242b87f9ebbc526d173875cda07a68a7d3f6beb4face1dc612</code> | <code>docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md</code> | exact match |
| <strong>Canonical total</strong> | <strong>2,587</strong> | <strong>123,236</strong> | — | 8 files | exact manifest total |
| Orientation only | 836 | 81,884 | <code>f9e8200b33a905b37f518b38f101ec0b9fe39c3a0e903afc30a3bade199a3189</code> | <code>docs/thefinalglance/feature-discovery/02-B-PLAN-SUMMARY.discovery.md</code> | read completely |
| Scope locator only | 1,430 | 165,971 | <code>4aafb23fa4ec917813cc3b629f13cea84049194f4a3b91d340a545826ac2aa93</code> | <code>docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md</code> | DEEP-B-002 table located; not treated as an original |

## Per-file exhaustive ledgers

### B-1 — Path unification

- **B1-001 [E]** Objective: eliminate the split database path that leaves PageRank, CatRAG, A-RAG, Telescope/blast-radius consumers, and <code>DkgQuerierAdapter</code> empty even though the DKG is populated; estimated one 2–4 hour session, no prerequisite, highest-leverage B-plan fix (lines 1-8).
- **B1-002 [E]** The writer path is <code>.elai_cc/indexes/dkg.db</code>, built from <code>ELAI_DATA_DIR_NAME = ".elai_cc"</code> and the indexing service; two CLI consumers historically hard-coded <code>.elai/dkg.db</code>, while the dashboard already used the canonical constants (lines 15-20).
- **B1-003 [E]** Failure state: CLI PageRank/CatRAG/A-RAG/Telescope/blast-radius see an empty DKG; <code>DkgQuerierAdapter</code> silently and permanently falls through to <code>EmptyDkgQuerier</code>; PageRank runs over zero nodes, CatRAG pushes empty results, and <code>AragOrchestrator</code> retains an empty corpus (lines 22-29).
- **B1-004 [E]** A rule-engine reader independently uses the wrong <code>.elai/sessions/dkg.db</code> path; it must converge on <code>.elai_cc/indexes/dkg.db</code> (lines 31-32).
- **B1-005 [E]** Audit the complete <code>.elai_cc/</code> tree for literal <code>\.elai/</code>, excluding legitimate <code>.elai_cc/</code>; the plan expected roughly 5–10 hits in CLI/examples/tests/docs (lines 51-56).
- **B1-006 [E]** Create the cross-crate path authority at <code>crates/core/src/paths.rs</code> with <code>ELAI_DATA_DIR_NAME</code> and <code>INDEXES_DIR_NAME</code>; <code>dkg_db_path(project_root)</code> joins project root, data dir, indexes dir, and <code>dkg.db</code> (lines 58-66).
- **B1-007 [E]** The same authority exports <code>bm25_dir_path</code>, <code>vectors_db_path</code>, <code>sessions_dir_path</code>, <code>metrics_dir_path</code>, <code>models_dir_path</code>, and <code>rules_db_path</code> (lines 67-69; 102-109).
- **B1-008 [E]** <code>indexing_service.rs</code> must import the core constants/helper without changing its already-correct behavior (lines 71-72).
- **B1-009 [E]** Canonical live consumers are <code>crates/cli/src/runtime_builder.rs:224</code>, <code>crates/cli/src/repl_command.rs:625</code>, and <code>crates/cli/src/live_cli.rs:3255</code>, each using <code>dkg_db_path</code>; the latter paths supersede older monolithic <code>main.rs</code> anchors (lines 74-79; 110-116; 185-187).
- **B1-010 [E]** Every rule-engine DKG reader must call <code>core::paths::dkg_db_path</code> (lines 80-81).
- **B1-011 [E]** The CLI existence gate must test the canonical path so it fires in production (lines 83-85).
- **B1-012 [E]** Replace <code>.elai/models/</code> literals with <code>models_dir_path</code>; the required model location is <code>.elai_cc/models/onnx/</code> (lines 87-89).
- **B1-013 [E]** CLI bootstrap must either find <code>.elai_cc/</code> or raise a clear “uninitialized project” error; silent <code>EmptyDkgQuerier</code> fallback is forbidden (lines 91-93).
- **B1-014 [E]** When legacy <code>.elai/</code> and canonical <code>.elai_cc/</code> coexist, warn that the former is deprecated, prefer the latter, and never auto-delete the former because another tool may own data there (lines 95-98).
- **B1-015 [E]** Reachability must include CLI <code>main</code>, dashboard-v2 <code>run</code> and <code>submit_task</code>, and <code>TaskRunner::drive_task</code>; <code>submit_task</code> was still a frontend stub assigned to B-36 at this source snapshot (lines 117-124).
- **B1-016 [E]** Stub bans: no <code>\.elai/</code> literal in non-test source and no literal <code>\.elai_cc/dkg</code>; the latter must always go through a helper (lines 125-128).
- **B1-017 [E]** Declared observable gate runs <code>elai.exe bench canary-regression --fixtures .elai_cc/benchmarks/canary/fixtures</code>, expects SSB <code>canary_bench_run_completed</code>, and expects artifact <code>BENCH-CANARY PASS</code> (lines 129-133).
- **B1-018 [E]** Static deterministic gate: <code>rg</code> over non-test/non-example Rust must return zero <code>.elai/</code> literals (lines 134-140).
- **B1-019 [E]** Helper-use gate expects at least four <code>core::paths::dkg_db_path</code> references—three CLI readers and the indexing service (lines 141-143).
- **B1-020 [E]** Runtime smoke gate launches the REPL on an indexed fixture, queries “list rust files,” and requires real paths from <code>.elai_cc/indexes/dkg.db</code>, not an empty result (lines 145-147).
- **B1-021 [E]** SQLite evidence queries <code>dkg_nodes</code> and expects a count matching <code>phase_1A.toml</code>, approximately 8,000+ (lines 149-153).
- **B1-022 [E]** Acceptance requires all helpers to exist and be exported from core (lines 156-160).
- **B1-023 [E]** CLI startup must log canonical DKG, BM25, vectors, sessions, and models paths (line 160).
- **B1-024 [E]** CLI DKG results must be non-empty and match Phase-1A metrics; dashboard KnowledgeDrawer counts must remain non-zero after the refactor (lines 161-163).
- **B1-025 [E]** PageRank must demonstrably fire; add <code>tracing::info</code> for processed node count (lines 164-165).
- **B1-026 [E]** CatRAG overlay must be non-empty during a query (line 166).
- **B1-027 [E]** <code>AragOrchestrator::add_chunk</code> must be wired into <code>IndexingService</code> so the A-RAG corpus fills while files index (lines 167-168).
- **B1-028 [E]** Core path tests cover every helper under Windows and Unix separators (lines 170-173).
- **B1-029 [E]** CLI integration test spawns the REPL on a fixture and asserts real file references (lines 174-175).
- **B1-030 [E]** Indexing-service contract proves writes target <code>dkg_db_path</code>, not a literal (lines 176-177).
- **B1-031 [E]** Repository contract fails if a <code>\.elai/</code> literal returns (lines 178-179).
- **B1-032 [E]** File lifecycle: create <code>core/src/paths.rs</code> and <code>workflow/reasoning/B-1-decisions.md</code>; export from <code>core/src/lib.rs</code>; modify the three CLI readers, indexing service, all rule-engine DKG readers, and all vector loaders with the old model path (lines 181-192).
- **B1-033 [E]** Explicitly unlocked behaviors are real Phase-1D symbols, populated Phase-1E CatRAG and A-RAG, a non-falling-through Phase-3J adapter, real Telescope counts, and non-empty future surgical blast radius; roughly five audits move from PARTIAL/DEAD to WIRED (lines 194-205).
- **B1-034 [A]** The decision record must resolve helper placement (<code>core/paths.rs</code> versus direct <code>core/lib.rs</code>), legacy-directory migration policy, and old fixture handling even though the task body already strongly selects <code>core/paths.rs</code> and a warn/prefer/no-delete migration behavior (lines 207-212).
- **B1-035 [E]** B-2 may run in parallel; B-3 depends on B-1 so real readers can replace stubs (lines 214-218).
- **B1-036 [E]** Mandatory evidence inputs are the aggregate dual-path audit, Phase 1A/1D/1E/3J audits, the indexing-service constants, and both original CLI literal sites (lines 34-49).

### B-2 — FSM <code>PhaseTransition</code> wiring

- **B2-001 [E]** Objective: a roughly ten-line, 30-minute producer fix writes a <code>PhaseTransition</code> SSB row from <code>Fsm::transition_with_metrics</code>, activating the StatusBar state badge, EXECUTING pulse, Monitor “currently” text, and the historical Activity renderer; no prerequisite (lines 1-8).
- **B2-002 [E]** V2 replaces the V1 SSB viewer with an AppState/SSB-backed B-36 command, moves StatusBar and MonitorWindow to dashboard-v2 paths, and replaces textual Activity entries with an orbit halo plus a bubble-up strip event (lines 10-38).
- **B2-003 [E]** Producer work is dashboard-agnostic and must land as written; V2 consumers use <code>get_fsm_state</code>/<code>get_context_state</code> and emitted <code>phase_transition</code> events (lines 40-44).
- **B2-004 [E]** Existing polling reads the newest <code>action_type='phase_transition'</code> every 1,000 ms, but production previously had no writer: only test fixtures and the DST model constructed the payload, while FSM state was persisted solely to a per-session MessagePack file (lines 57-62).
- **B2-005 [E]** Observable pre-fix failures were a permanently IDLE badge, dead ten-state color/pulse switch, no EXECUTING animation, stale Monitor text, and no historical feed row (lines 64-70).
- **B2-006 [E]** The existing web-research-only producer used incompatible <code>fsm_transition</code>; the StatusBar did not read it (lines 72-75).
- **B2-007 [E]** Confirm exact payload schema before coding: <code>from</code>, <code>to</code>, <code>ts_ms</code>, and optional <code>reason</code> (lines 93-97).
- **B2-008 [E]** If FSM lacks access to a writer, inject <code>ssb_writer: Option&lt;Arc&lt;dyn SsbWriter&gt;&gt;</code>; optionality preserves tests that construct an FSM without SSB (lines 98-101).
- **B2-009 [E]** Emit only after successful state-update logic and before returning success, use <code>self.clock.now_ms()</code>, clone/default the reason, and write <code>SsbPayload::PhaseTransition(payload)</code> with sequence argument 0 and an empty attachment vector (lines 103-125).
- **B2-010 [E]** SSB write failure is observational, never a decision input: warn with target <code>fsm</code> and continue the state transition (lines 116-129).
- **B2-011 [E]** Canonical action type is singular <code>phase_transition</code>, because that is what the consumer query reads (lines 131-132).
- **B2-012 [E]** Migrate the partial <code>fsm_transition</code> producer to the canonical name rather than widening the query; recommendation is a single source of truth (lines 134-139).
- **B2-013 [E]** Delete a redundant <code>emit_fsm_transition</code>, or retain it only for genuinely external transitions and make it use the canonical action type (lines 141-143).
- **B2-014 [E]** Pass the writer through every <code>Fsm::new</code> call site (lines 145-146).
- **B2-015 [E]** Find direct state assignments that bypass <code>transition_with_metrics</code> and refactor them through the metrics method (lines 148-150).
- **B2-016 [E]** Produced anchors are <code>fsm.rs::transition_with_metrics</code> and <code>ssb/payloads.rs::PhaseTransitionPayload</code> (lines 152-157).
- **B2-017 [E]** Consumers are <code>ssb_viewer.rs::latest_fsm_state</code>, V2 <code>get_context_state</code>/<code>get_fsm_state</code>, and <code>ContextStatePanel.svelte::get_context_state</code> (lines 157-162).
- **B2-018 [E]** Reachability paths include LiveCli <code>run_async → set_ssb_writer → transition_with_metrics</code>, TaskRunner <code>build_orchestrator → with_ssb_writer</code>, runtime setup, and dashboard <code>submit_task → TaskRunner → FSM</code> (lines 162-166).
- **B2-019 [E]** Stub bans require a real non-test FSM writer and forbid a permanently IDLE badge during a running task (lines 167-170).
- **B2-020 [E]** Contract trigger is <code>cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb -- --nocapture</code>; evidence is an SSB <code>phase_transition</code> plus visible PLANNING/EXECUTING/etc. badge changes and pulse (lines 171-177).
- **B2-021 [E]** Static checks require at least one <code>phase_transition</code> write in <code>fsm.rs</code> and at least one production <code>SsbPayload::PhaseTransition</code> outside tests/DST/SSB fixtures (lines 179-190).
- **B2-022 [E]** Manual task smoke expects the StatusBar to transition within two seconds (lines 191-194).
- **B2-023 [E]** SQL evidence reads the five newest <code>phase_transition</code> rows from the session SSB and requires from/to state fields (lines 196-200).
- **B2-024 [E]** Acceptance: emit on every successful transition and keep all 94 FSM tests passing (lines 203-206).
- **B2-025 [E]** Visual acceptance covers badge change, EXECUTING pulse, and Monitor text (lines 207-209).
- **B2-026 [A]** Acceptance still requires a V1 <code>ActivityEntry</code> feed row although the same source’s V2 rebaseline says there is no direct equivalent; current V2 evidence should be orbit halo/bubble strip instead (lines 27-34 versus 210).
- **B2-027 [E]** FSM must succeed without a writer; failure is log-and-continue, never panic/block (line 211).
- **B2-028 [E]** Unit test with a mock writer asserts the exact emitted payload (lines 213-217).
- **B2-029 [E]** Unit test without a writer asserts transition success and no panic; existing constructor tests are updated as needed (lines 218-222).
- **B2-030 [E]** Optional end-to-end Tauri test proves <code>get_fsm_state</code> returns the latest transition (lines 223-224).
- **B2-031 [E]** Files changed are FSM, the partial TaskRunner producer, all FSM constructors, and <code>B-2-decisions.md</code> (lines 226-233).
- **B2-032 [A]** Decision record retains two design choices—FSM-owned optional writer versus caller emission, and one action type versus preserving the web-research name—while explicitly fixing the failure mitigation as non-blocking (lines 235-241).
- **B2-033 [E]** Mandatory evidence inputs are dashboard orchestration audit, full transition implementation, SSB payload definition/constructors, consumer query/renderer, and the partial TaskRunner producer (lines 77-91).
- **B2-034 [E]** B-3 is next and B-4 through B-7 can proceed in parallel (lines 243-245).

### B-4 — Knowledge-engine producer wiring

- **B4-001 [E]** Objective: expose every knowledge-engine signal in the dashboard, repair audited runtime degradations, and stage missing ONNX models; estimate 2–3 sessions; requires B-1 canonical paths and is parallel-safe with B-5/B-6 (lines 1-8).
- **B4-002 [E]** V2 keeps producer work unchanged, relocates Tauri commands to dashboard-v2, renders through <code>KnowledgeDrawer.svelte</code>, and consumes SSB through AppState-backed commands (lines 10-23).
- **B4-003 [A]** V2 stub names are <code>get_query_trace_recent</code>, <code>get_search_mode</code>, <code>get_embedding_breakers</code>, <code>get_recall_metrics</code>, and <code>get_dkg_stats</code> and are supposed to stay 1:1, but later task/contract text uses <code>get_query_trace</code> and <code>get_circuit_breaker_states</code> as alternate names (lines 25-27; 110-119; 225-228).
- **B4-004 [E]** Closure rebaseline maps the old <code>NoopSsbNotifier</code> anchor to <code>SsbDegradationNotifier</code> in <code>live_cli.rs</code>; semaphore shutdown and the targeted boxed errors had already shipped through B-16A/anti-stale work (lines 29-46).
- **B4-005 [E]** Metrics citations were stale: current STANDARD MRR is 0.6000 and DEEP MRR 0.5017, still a qualitative monotonicity failure; command output must read the TOML at runtime rather than literals, and the drawer already paints a red warning (lines 48-53).
- **B4-006 [E]** B-4 absorbed C-10’s <code>CanaryDrift</code> row: schema 98→99, the seven-site “QUINTUPLE-UPDATE,” payload/test-bijection 231→232, startup producer, and 30-second heartbeat; C-10 may drop that work (lines 55-60).
- **B4-007 [E]** Audit baseline says roughly 91% of knowledge signals were invisible and model/path/wiring shortcuts caused degraded stages (lines 62-67).
- **B4-008 [E]** Stage <code>cross-encoder-l12.onnx</code> plus <code>tokenizer.json</code> under the core model path; setup shell/batch scripts download and blake3-verify it, and <code>elai_cc.toml [models.cross_encoder]</code> documents it; absence degrades reranking to RRF-only (lines 80-84).
- **B4-009 [E]** Stage GraphCodeBERT and Granite ONNX models at <code>.elai_cc/models/onnx/</code>, fix the old <code>.elai/models/</code> mismatch, and change the canary input from <code>embed("test")</code> to <code>embed("fn main()")</code> (lines 86-89).
- **B4-010 [E]** Shipped SearchMode integration uses <code>pipeline.search_mode</code> plus <code>set_search_mode()</code>; the caller drives the cascade externally instead of storing the originally proposed <code>Arc&lt;DegradationCascade&gt;</code> inside the pipeline (lines 91-100).
- **B4-011 [E]** <code>SsbDegradationNotifier</code> is constructed by <code>spawn_cascade_evaluator_loop</code>, has its dead-code allowance removed, and writes <code>knowledge_degradation</code> rows on mode transitions (lines 97-100).
- **B4-012 [E]** <code>QueryTrace::new</code> records the active <code>SearchMode</code>, and cascade evaluation runs in a spawned Tokio loop every <code>CASCADE_EVAL_INTERVAL_SECS = 30</code> (lines 101-104).
- **B4-013 [E]** Runtime retrieval must query the code-vector cache and fuse BM25 + <code>code_vec</code> + <code>text_vec</code> as three-ranker RRF, replacing two-ranker behavior (lines 106-108).
- **B4-014 [E]** Query-trace command returns a per-stage timeline/latency; drawer “Last query” shows BM25, vector, PageRank, CatRAG, A-RAG, rerank, and token-fit bars with fired/skipped/degraded state (lines 110-114).
- **B4-015 [E]** Circuit-breaker command returns per-model <code>state ∈ {Closed, HalfOpen, Open}</code>, <code>last_failure</code>, <code>last_success</code>, and <code>attempts</code>; drawer presents a five-row grid (lines 116-119).
- **B4-016 [E]** <code>SidecarHydeClient</code> emits <code>hyde_query</code> carrying hypothesis text, cache hit/miss, and latency; the latest hypothesis appears in the QueryTrace inspector (lines 121-124).
- **B4-017 [E]** Shipped live recall values are FAST 0.6000/0.7000/0.3908, STANDARD 0.8000/0.8000/0.6000, and DEEP 0.8000/0.8000/0.5017 for R@5/R@10/MRR (lines 126-130).
- **B4-018 [E]** Canonical V2 recall command is explicitly <code>get_recall_metrics</code>, not the original <code>get_retrieval_quality_metrics</code>; it reads <code>phase_3K_rag.toml</code> with <code>read_to_string</code>/<code>toml::from_str</code> and a probe-path fallback (lines 130-136).
- **B4-019 [A]** DEEP monotonicity is only PARTIAL: an original-query anchor fix shipped, UI exposes the remaining 0.5017&lt;0.6000 gap, and further improvement is outside B-4; the stale acceptance box still says DEEP must be at least STANDARD, while the deterministic check deliberately asserts DEEP remains lower (lines 138-146; 285-289; 302-315).
- **B4-020 [E]** <code>get_pagerank_top</code> must return 5–10 non-zero importance-ranked symbols after B-1 (lines 148-150).
- **B4-021 [E]** Shipped freshness command calls <code>SsbViewer::freshness_state()</code>, polls every five seconds, renders <code>decay_rate</code>, <code>access_boost</code>, and <code>last_recalc_ms</code>, and returns <code>known:false</code>/“no freshness snapshot yet” before any producer runs (lines 152-158).
- **B4-022 [E]** Shipped vector-cache telemetry is emitted after every successful query; <code>get_vector_cache_hit_rate</code> reads the latest row and renders <code>hit_rate_pct</code>, <code>hits/total</code>, and session ID, with <code>known:false</code> fallback (lines 160-167).
- **B4-023 [E]** Settings must expose current index schema version, last migration, and pending <code>UpgradeAction</code> status (lines 169-170).
- **B4-024 [E]** <code>CanaryDriftPayload</code> fields are <code>similarity</code>, <code>drift_threshold</code>, <code>passed</code>, <code>reference_present</code>, and <code>ts_ms</code> (lines 172-176).
- **B4-025 [E]** Canary producer emits the actual startup verdict and then every 30 seconds; <code>get_canary_drift</code> reads the newest row; Settings renders <code>cosine X / thr 0.85</code>, red on failure, with recompute affordance (lines 176-183).
- **B4-026 [E]** Correct drift threshold is absolute cosine similarity at least 0.85; the old 0.05 literal meant a delta, not the actual pass threshold (lines 183-186).
- **B4-027 [E]** Drawer must expose indexed and failed-file counts (example: 47 indexed/3 parse failures) and allow drilling into the failure list (lines 188-189).
- **B4-028 [E]** Shipped stage timing captures <code>self.clock.now()</code> and <code>elapsed_since</code> around synthetic <code>deep_cap</code>/<code>rrf_gate</code>; total latency is calculated before trace construction as a real sum-of-stage-latencies upper bound (lines 191-200).
- **B4-029 [E]** Timing preserves entropy quarantine: only injected <code>self.clock</code> is used, never raw <code>std::time::Instant</code> (lines 201-202).
- **B4-030 [E]** Semaphore closure no longer panics: acquisition failure logs a warning and exits gracefully; the old <code>expect("semaphore closed")</code> is absent (lines 204-209).
- **B4-031 [E]** Targeted knowledge modules have zero <code>Box&lt;dyn Error&gt;</code>; current types are <code>KnowledgeError</code>, <code>VectorError</code>, and <code>Bm25Error</code> (lines 211-215).
- **B4-032 [E]** Exact payload symbols are <code>KnowledgeDegradationPayload</code>, <code>HydeQueryPayload</code>, <code>QueryTracePayload</code>, <code>CircuitBreakerStateChangePayload</code>, and <code>VectorCacheHitRatePayload</code> (lines 217-224).
- **B4-033 [E]** Declared command symbols are <code>get_query_trace</code>, <code>get_embedding_breakers</code>, <code>get_recall_metrics</code>, <code>get_pagerank_top</code>, <code>get_freshness_state</code>, and <code>get_canary_drift</code> (lines 225-230).
- **B4-034 [E]** Consumers are KnowledgeDrawer for all knowledge rows and Settings for canary/index version; reachability runs through RAG bench, LiveCli local knowledge-stack construction, and the V2 invoke handler (lines 231-237).
- **B4-035 [E]** Stub bans require no production <code>NoopSsbNotifier</code>, no semaphore <code>expect</code>, and nonzero per-stage latency (lines 238-242).
- **B4-036 [E]** Observable contract runs STANDARD RAG bench on “find auth handlers,” expects <code>query_trace</code>, and requires the drawer timeline (lines 243-247).
- **B4-037 [E]** Deterministic model filenames are <code>cross-encoder-l12.onnx</code>, <code>graphcodebert.onnx</code>, and <code>granite-embed.onnx</code> (lines 250-256).
- **B4-038 [E]** Schema check requires 99; CanaryDrift must appear at least seven times; typed payload bijection test must have 232 entries (lines 258-268).
- **B4-039 [E]** Static checks enforce live notifier construction, absence of semaphore panic, nonzero gate latencies, command registration, and frontend wrappers for canary/freshness/vector cache (lines 270-300).
- **B4-040 [E]** Acceptance retains model staging with blake3, live degradation rows, three-ranker code vectors, visible trace/breaker/HyDE/recall/failed-file/canary surfaces, no semaphore panic, and no public boxed errors (lines 302-315).
- **B4-041 [E]** Touch set spans pipeline, reranker, BM25, vector indexer, PageRank, HyDE, CLI notifier, dashboard command/viewer/drawer, staging scripts, and <code>B-4-decisions.md</code> (lines 317-329).
- **B4-042 [A]** Script naming drifts: task text creates <code>stage_models.sh/.bat</code>, the deterministic check cites <code>stage_onnx_models.{sh,bat,py}</code>, and the file list returns to <code>stage_models.sh/.bat</code> (lines 80-83; 250-253; 328).
- **B4-043 [E]** Decision record chooses first-run download versus manual staging and TypeScript binding strategy, assuming B-3 supplied <code>ts-rs</code> (lines 331-334).
- **B4-044 [E]** Mandatory declared inputs are the knowledge-wiring audit, Phase 1A–1I/3K audits and summaries, QueryPipeline, and reranker failure anchor (lines 69-76).
- **B4-045 [A]** Source status is granular rather than simply DONE: tasks 3, 8, 11, 12, 14, 16–18 say shipped; task 9 says PARTIAL; code-vector query, trace, breaker, HyDE, PageRank, index-upgrade status, and failed-file UI have no shipped marker in this file (lines 78-215).
- **B4-046 [E]** Frontend and producer are intentionally paired in one B-step; next B-5/B-6 work is parallel-safe (lines 242; 336-338).

### B-12 — Shell versus Workspace decision

- **B12-001 [E]** Original objective was to make the spec Workspace primary and retain Tier-0 Shell as an optional power-user mode; estimate ~30 minutes, with B-3/B-7/B-11 prerequisites (lines 1-9).
- **B12-002 [E]** The 2026-05-07 V2 rebaseline says that decision is already executed differently: dashboard-v2 has one canonical Workspace and retires Tier-0 Shell entirely (lines 11-24).
- **B12-003 [E]** Actual entry flow is <code>main.ts</code> using Svelte 5 <code>mount(App, { target })</code>, then <code>ProjectSelector.svelte</code> on first run, then <code>Workspace.svelte</code> (lines 13-20).
- **B12-004 [E]** Workspace owns <code>AppShell</code>, <code>Sidebar</code>, <code>Panel</code>, <code>PanelDivider</code>, <code>StatusBar</code>, <code>WindowHost</code>, <code>Drawer</code>, <code>CommandPalette</code>, <code>Rail</code>, and orbit canvas (lines 20-22).
- **B12-005 [E]** Rebaseline work item verifies StatusBar belongs inside Workspace and moves it if it is an App sibling (lines 26-30).
- **B12-006 [E]** ProjectSelector must ask only for the project, never the retired Tier-0/Workspace choice (lines 31-33).
- **B12-007 [A]** Rebaseline initially asks to verify five 48px sidebar icons, but the later July closure explicitly says the active three-button 48px <code>IconRail</code> and four-question-tab information architecture supersede that five-mode model; controller resolution is deferred to B-26 (lines 34-36 versus 165-170).
- **B12-008 [E]** Remove the obsolete <code>elai_cc.toml [dashboard] primary_shell</code> knob because V2 has only one shell (lines 37-38).
- **B12-009 [E]** Exact V1→V2 mapping retires <code>Shell.svelte</code>/<code>WindowRegistry.ts</code>, moves Workspace/Sidebar/StatusBar to V2 screen/layout paths, and uses <code>main.ts</code> Svelte mount (lines 40-51).
- **B12-010 [E]** Historical spec called for one Workspace, internal StatusBar, five mode icons, and primary/secondary panels; the old Tier-0 implementation had nine windows, ten drawers, autonomous UI, and ProposalsPanel built ahead of post-MVP scope (lines 53-65).
- **B12-011 [A]** Historical task 1 asks for a <code>Ctrl+Shift+W</code> Tier-0 toggle, persisted <code>primary_shell = workspace|tier0</code>, and a first-run preference question; all three are superseded by the single-shell rebaseline (lines 76-85 versus 11-38 and 157-174).
- **B12-012 [E]** Historical detailed sidebar spec was 48px/icon-only with terminal, layers, play, search, settings; footer displayed LOCAL teal/CLOUD purple mode and user session (lines 86-92).
- **B12-013 [E]** Historical six spec windows were COMMAND, PLAN, ACTIVITY, REVIEW, EXPLORE, MONITOR; Preview/Research/Settings were to be hidden as post-MVP unless flagged (lines 94-99).
- **B12-014 [E]** Historical defaults were INTERACTIVE = ACTIVITY primary/COMMAND secondary and AUTONOMOUS = MONITOR primary/ACTIVITY secondary (lines 101-106).
- **B12-015 [E]** Historical drawer plan consolidated ten named drawers into Settings tabs or hid them behind power-user mode while keeping individual surfaces in Tier-0 (lines 108-112).
- **B12-016 [E]** <code>Ctrl+.</code> command palette remains; window history/tabs defer to B-26 (lines 114-117).
- **B12-017 [E]** Wiring contract’s produced symbol is V2 <code>Workspace.svelte</code>, reachable from <code>main.ts::mount</code>; its stub ban says Tier-0 cannot be default (lines 119-130).
- **B12-018 [A]** Observable contract still expects Tier-0 keybinding access, contradicting retirement and the July closure’s explicit deferral (lines 131-135 versus 157-160).
- **B12-019 [A]** Deterministic checks still inspect the retired V1 frontend and expect at least one <code>primary_shell</code> hit, directly conflicting with the V2 instruction to delete that knob (lines 137-147 versus 37-38).
- **B12-020 [E]** July 2 closure verifies Workspace is shipped as the default: <code>App.svelte</code> routes <code>session.screen</code> to Workspace through the default else branch (lines 149-156).
- **B12-021 [E]** Tier-0 keybinding is deferred to B-26 and intentionally not implemented because adding a nonexistent shell <code>WindowType</code> creates a three-site synchronization risk (lines 157-160).
- **B12-022 [E]** Internal StatusBar is shipped: <code>WorkflowShell.svelte</code> renders <code>ShellStatusBar</code> in row 3 of its three-row grid, below App scope (lines 161-164).
- **B12-023 [E]** Sidebar/icon remap is deferred, not a remaining B-12 done gate; active IA supersedes the original five icons (lines 165-170).
- **B12-024 [E]** Layout-preference wizard is deferred and declared meaningless under a single shell; ProjectSelector asks only for a project (lines 171-174).
- **B12-025 [E]** Drawer consolidation is deferred because V2 has roughly 60 drawer panels, materially beyond the assumed ten (lines 175-179).
- **B12-026 [E]** Historical file-touch list covered V1 main/App, Workspace, Shell, Sidebar, StatusBar, a new FirstRunWizard, TOML schema, and <code>B-12-decisions.md</code>; it is not the current V2 implementation map (lines 181-190).
- **B12-027 [E]** Mandatory evidence references Phase-4K/dashboard audits, dashboard-design Workspace/windows flows, and the former V1 Shell/Workspace/registry (lines 67-75).
- **B12-028 [A]** Current status is “single Workspace shipped; StatusBar shipped; Tier-0, five-icon remap, wizard, and drawer consolidation deferred/superseded,” not a blanket requirement to backfill all original tasks (lines 149-179).
- **B12-029 [E]** B-13 is the declared next step (lines 192-194).

### B-24 — Phase 4I media validator

- **B24-001 [E]** Objective is conditional Phase 4I: deterministic CPU Tier 1 under 100 ms plus GPU/VLM advisory Tier 2 around 2–5 seconds, CLAP audio, external media tools, and a Render-Capture-Evaluate loop; estimate 2–3 sessions (lines 1-7).
- **B24-002 [E]** Requires B-22 autonomous iteration and B-7 <code>PipelineSpec</code> media-step surfaces; parallel-safe with B-23/B-25/B-26/B-27/B-28 (lines 8-10).
- **B24-003 [E]** Entire module is instantiated only when <code>PipelineSpec</code> contains media-producing steps/<code>MediaSpec</code>; lazy loading is mandatory (lines 14-18; 59-63).
- **B24-004 [E]** Tier 1 checks format/resolution with ImageMagick, SSIM/LPIPS with ONNX Runtime CPU EP, 3D vertex/triangle counts with Blender CLI, audio sample rate with FFmpeg, and optional expected-text OCR with Tesseract (lines 20-23; 65-71).
- **B24-005 [E]** GPU SSIM/LPIPS is explicitly forbidden as nondeterministic; every Tier-1 check targets under 100 ms (lines 21-24; 65-71).
- **B24-006 [E]** Tier-2 Mode A is <code>Qwen3.5-VL-30B-A3B</code> through Ollama for one image, approximately 17 GB Q4 (lines 24-26; 73-76).
- **B24-007 [E]** Tier-2 Mode B is <code>InternVL3.5-38B</code> through SGLang for sequences, approximately 38 GB at 8-bit (lines 24-27; 73-77).
- **B24-008 [E]** Mode selection is deterministic (single image versus sequence), and constrained decoding must produce the <code>VlmValidationOutput</code> JSON schema (lines 27; 73-78).
- **B24-009 [E]** CLAP is roughly 200M parameters/0.4 GB FP16, remains loaded whenever audio is active, and compares audio/text embeddings by cosine similarity (lines 29-30; 80-82).
- **B24-010 [E]** External-tool failures degrade gracefully; concurrency is bounded at four Tier-1 jobs, one Tier-2 job, with queue depth 64 (lines 32-33).
- **B24-011 [E]** Verdict precedence is fail-closed for deterministic checks: Tier-1 failure → <code>Fail</code>; Tier-1 pass plus Tier-2 disagreement → <code>HumanReview</code>; unavailable VLM → <code>Degraded</code> (lines 35-36; 84-85).
- **B24-012 [E]** Tier 2 is advisory and cannot veto/override Tier 1 (lines 24; 84-85; 116-119).
- **B24-013 [E]** <code>MediaRenderAdapter::render(spec) → Vec&lt;RenderedImage&gt;</code> abstracts render-to-image (lines 38-45; 87-90).
- **B24-014 [E]** Five adapters are Playwright/web, Blender CLI/3D, FFmpeg/animation, direct screenshot read, and ComfyUI/generated media (lines 40-45; 87-90).
- **B24-015 [E]** RCE workflow is execute → render → VLM evaluation → structured feedback → executor iteration (lines 46-47; 92-94).
- **B24-016 [E]** Iteration limit is configurable with default 5; exceeding it escalates, and unchanged issue sets stop convergence early (lines 47-48; 95-96).
- **B24-017 [E]** Dashboard <code>MediaValidationPanel</code> shows per-step Tier-1 results, Tier-2 advisory JSON, preview thumbnails, iteration count, and last issues (lines 98-100).
- **B24-018 [E]** Produced types/surfaces are <code>MediaValidator</code>, <code>Tier1Validator</code>, <code>Tier2Validator</code>, five adapter implementations, and the feedback-loop runner (lines 102-107).
- **B24-019 [E]** Exact six SSB actions are <code>media_validation_started</code>, <code>media_tier1_complete</code>, <code>media_tier2_complete</code>, <code>render_complete</code>, <code>visual_feedback_iteration</code>, and <code>media_human_review_required</code> (lines 108-110).
- **B24-020 [E]** Reachability is TaskRunner with a media-producing <code>PipelineSpec</code>; consumer is the media panel and thumbnail UI (lines 111-120).
- **B24-021 [E]** Observable task submits an image-producing job and requires tier-1/tier-2/render SSB rows inside the iteration budget plus dashboard thumbnails/results (lines 121-125).
- **B24-022 [E]** Stub bans prohibit GPU deterministic metrics, Tier-2 veto, and any violation of bounded concurrency (lines 116-119).
- **B24-023 [E]** Deterministic gate searches for <code>fn new_if_media_steps</code> and requires at least 28 media tests (lines 127-135).
- **B24-024 [E]** Test target is at least 28 total: 20 contract/functional and 8 adversarial (line 50).
- **B24-025 [E]** Acceptance covers lazy absence, per-check Tier-1 latency, advisory-only Tier 2, CLAP, all five adapters, convergence, 4/1 concurrency, human-review action, thumbnails/results, and 28+ passing tests (lines 137-148).
- **B24-026 [E]** File lifecycle adds <code>media_validator.rs</code>, five <code>media_adapters</code>, panel component, media setup script, decisions document, and confirmed dependencies; modifies SSB and V2 Tauri wiring (lines 150-161).
- **B24-027 [E]** New ONNX/image/etc. dependencies require confirmation, and VLM model download occurs only when the user opts in (lines 159-160).
- **B24-028 [E]** Panel is a new V2 component mounted from SettingsWindow or MonitorWindow; no existing V2 equivalent is assumed (lines 155-158).
- **B24-029 [A]** This canonical file contains a conditional plan with unchecked acceptance boxes and no shipped closure/status declaration; production client names, four Tauri commands, and 71/21/19/2 test ledgers reported by the orientation summary are not present here (lines 1-165).
- **B24-030 [E]** Mandatory sources are Phase-4I, media-validator integration spec, integration summary, and Phase-4 summary (lines 52-57).
- **B24-031 [E]** B-25 Empire integration is the declared next step (lines 163-165).

### B-31 — Resumable indexing, cross-process locks, and startup lifecycle

- **B31-001 [E]** Current header explicitly corrects the old marker to <strong>SHIPPED</strong> and identifies production implementations for metadata store, lock/scope, auto-resume/shutdown, five SSB payloads, three <code>IndexResult</code> variants, cancellation command, and IndexingProgressPanel (lines 1-10).
- **B31-002 [E]** <code>ELAI_DISABLE_INDEXING</code> is registered in <code>wire-dark-allowlist.toml</code> under R9; decisions and a dated plan audit are named as evidence (lines 8-10).
- **B31-003 [E]** Objective: a 50K-file index survives crash via per-batch checkpoints, a concurrent process cannot double-index a project, and dashboard/MCP restart auto-resumes without user action (lines 11-15).
- **B31-004 [E]** Estimate was 10–14 hours across two sessions; requires B-1 canonical paths and is parallel-safe with B-29/B-30/B-32/B-33/B-34/B-35 (lines 16-19).
- **B31-005 [E]** Failure baseline included restart-from-zero after <code>kill -9</code>, concurrent duplicate/corrupt writes or SQLite busy errors, and partial data misclassified as complete/missing because no per-batch progress existed (lines 21-38).
- **B31-006 [E]** Ported design combines 50-file content-hash checkpoints, atomic project lock with PID/liveness/staleness, and startup resume/watcher catch-up with a 60-second shutdown deadline (lines 40-63).
- **B31-007 [E]** Existing <code>mcp_session_claim</code> remains the FSM/session ownership layer; new locks sit below it at indexing/watcher service scope (lines 65-68).
- **B31-008 [E]** Metadata table fields are <code>project_id</code> PK, <code>project_path</code>, <code>indexed_files=0</code>, <code>total_files=0</code>, <code>indexing_status</code>, <code>last_checkpoint_at</code>, <code>batch_size=50</code>, <code>batches_processed=0</code>, optional <code>batches_total</code>, and <code>schema_version=1</code> default (lines 93-109).
- **B31-009 [E]** Status domain is exactly <code>in-progress | completed | cancelled</code> and must never be absent at runtime (lines 103; 290-292).
- **B31-010 [E]** <code>file_hashes</code> stores project ID, relative path, blake3 content hash, and last-indexed timestamp under composite PK <code>(project_id, relative_path)</code> (lines 110-118).
- **B31-011 [A]** The migration is named “v2” although the sample table’s internal <code>schema_version</code> default is 1; the source does not explain that numbering distinction (lines 108-119).
- **B31-012 [E]** <code>ProjectMetadataStore</code> methods are <code>load</code>, idempotent <code>save</code>, <code>update_status</code>, <code>get_file_hashes</code> returning a BTreeMap, <code>set_file_hash</code>, <code>delete_file_hash</code>, and cascading <code>delete_project</code> (lines 120-129).
- **B31-013 [E]** Build-index Phase A scans indexable files; Phase B hashes content with blake3, loads saved hashes, and skips unchanged files in resume mode (lines 130-137).
- **B31-014 [E]** Phase C processes changed/new files in batches of 50; each batch embeds chunks, upserts BM25/vector/DKG, updates file hashes/counters/time, and persists <code>in-progress</code> (lines 138-145).
- **B31-015 [E]** Phase D marks <code>completed</code> only after all batches finish (line 145).
- **B31-016 [E]** Cancellation is checked at every batch boundary; it persists <code>cancelled</code> and returns <code>IndexResult::Cancelled { batches_completed }</code> (lines 146-149).
- **B31-017 [E]** Incremental update reuses batching only for changed hashes and deletes hashes/chunks for files missing from the latest scan (lines 151-155).
- **B31-018 [E]** <code>cancel_indexing(project_id) → bool</code> sets a flag for the next boundary and is exposed both as an orchestrator tool and a Tauri command (lines 156-159).
- **B31-019 [E]** One-time <code>migrateAbsolutePathKeys</code> strips old project-root prefixes from file-hash keys and is idempotent/safe twice (lines 161-165).
- **B31-020 [A]** Lock primitive choice is conditional: use <code>fs2</code> only if already present; otherwise propose hand-rolled <code>O_EXCL + PID</code> and ask before adding a dependency. The shipped header does not state which branch landed (lines 167-175).
- **B31-021 [E]** <code>IndexLock::acquire(project_root, LockScope)</code> returns <code>IndexLockGuard</code>; <code>LockScope</code> is <code>Index</code> or <code>Watch</code> (lines 177-180).
- **B31-022 [E]** Lock files are <code>&lt;project&gt;/.elai_cc/.index.lock</code> or <code>.watch.lock</code> and contain the holder PID (lines 181-182).
- **B31-023 [E]** PID liveness is <code>kill(pid, 0)</code>/SIG-0 semantics on POSIX and <code>OpenProcess</code> on Windows; a dead PID is reclaimable (lines 183-184).
- **B31-024 [E]** Holder refreshes mtime every 30 seconds; stale threshold is two minutes (lines 185-186).
- **B31-025 [E]** <code>IndexLockGuard</code> is RAII and releases on Drop; the fs2 branch uses <code>try_lock_exclusive</code> (lines 187-188).
- **B31-026 [A]** Context description mentions stale reclaim with SIGTERM/<code>taskkill /F</code>, while implementation tasks describe liveness check and reclaim without requiring process termination; do not infer that the shipped lock kills holders (lines 48-53 versus 177-188).
- **B31-027 [E]** <code>build_index</code>/<code>update_index</code> acquire Index scope; contention logs “Another process is already indexing this project, skipping” and returns <code>AlreadyLocked { holder_pid }</code>; RAII releases on both success and error (lines 190-195).
- **B31-028 [E]** Watch scope permits multiple instances to observe, but only one actively patches; secondaries report “active (watched by another process)” (lines 196-199).
- **B31-029 [E]** Diagnostic <code>IndexLock::is_locked_by_other(project_root, scope) → Option&lt;u32&gt;</code> reports foreign holder PID (lines 201-204).
- **B31-030 [E]** <code>auto_resume_indexed_projects()</code> is invoked in CLI after lock initialization/before dispatch and in dashboard after Tauri setup/before its first window (lines 206-214).
- **B31-031 [E]** Startup discovers every project with metadata, supporting multi-project deployments (lines 215-217).
- **B31-032 [E]** For <code>in-progress</code>, startup tries Index scope and calls <code>build_index(..., resume=true)</code>; if another live PID owns it, startup logs and skips (lines 218-223).
- **B31-033 [E]** For <code>completed</code>, startup acquires Watch scope and immediately runs incremental catch-up (lines 224-227).
- **B31-034 [E]** For <code>cancelled</code>, startup does nothing; only an explicit user re-index restarts it (lines 228-229).
- **B31-035 [E]** Each project resumes in a fire-and-forget Tokio task; failures are logged and never fail startup (lines 230-231).
- **B31-036 [E]** <code>graceful_shutdown</code> accepts SIGINT, SIGTERM, or dashboard stdin EOF, sets global <code>shutting_down</code>, and prevents new work (lines 233-237).
- **B31-037 [E]** Shutdown cancels every active index and stops all watchers/releasing Watch locks (lines 238-240).
- **B31-038 [E]** Shutdown waits at most 60 seconds for batch checkpoint and lock release, then force-releases lock files and logs timestamp/PID (lines 241-247).
- **B31-039 [E]** CLI signal handler and Tauri exit handler must both reach graceful shutdown (lines 246-247).
- **B31-040 [E]** <code>IndexCheckpoint</code> fields are project ID, processed/total batches, processed files, and last-checkpoint time; it emits every batch (lines 249-251).
- **B31-041 [E]** <code>IndexResume</code> fields are project ID, resumed batch, and files skipped; it emits once per resume (line 251).
- **B31-042 [E]** <code>IndexCancel</code> fields are project ID, completed batches, and indexed files; lock observability adds <code>LockAcquired</code>/<code>LockContested</code> (lines 252-253).
- **B31-043 [E]** Dashboard progress panel replaces static progress with live checkpoints and shows exact resume and lock-contention/read-only banners including batch/file savings and holder PID (lines 255-261).
- **B31-044 [E]** Produced contract includes all metadata methods, lock types, startup functions, five SSB variants, cancellation, and <code>IndexResult::{Cancelled, AlreadyLocked, Resumed}</code> (lines 263-274).
- **B31-045 [E]** Consumers/reachability cover CLI start/signal, dashboard run/exit, dashboard cancellation, watcher Watch scope, and both indexing writers’ Index scope (lines 275-289).
- **B31-046 [E]** Stub bans require a valid status, Drop release, a 60-second nonzero deadline, and startup registration from both CLI and dashboard (lines 290-296).
- **B31-047 [E]** Crash contract kills an index, restarts it, requires <code>IndexResume</code> from a nonzero batch, byte/ID-equivalent chunks with no duplicates, and released lock (lines 298-303).
- **B31-048 [E]** Deterministic evidence checks schema tables, completed counters, crash-resume node count, concurrent rejection with PID, auto-resume within five seconds, process exit within 65 seconds, and lock-file cleanup (lines 305-349).
- **B31-049 [E]** Acceptance requires automatic migration, 50-file checkpointing, boundary cancellation, blake3 skip, lock coverage, PID-visible contention, both startup calls, signal/exit shutdown, five visible SSB events, duplicate-free crash identity, path-key migration, and no unconfirmed dependency (lines 351-375).
- **B31-050 [E]** Test suite covers metadata CRUD/migration idempotence, kill/resume identity, contention/stale reclaim/Drop, startup resume, shutdown/cancel SSB, and a type-level lock-required resilience contract (lines 376-392).
- **B31-051 [E]** File lifecycle creates metadata SQL/module, lock/startup modules, panel, and decisions; modifies indexing service, CLI lifecycle, V2 Tauri run/exit/cancel, and five action types (lines 394-418).
- **B31-052 [A]** Decision doc retains choices for primitive, fixed/adaptive batches, 2m/5m stale timeout, blake3/sha256, session-claim interaction, 60s/120s shutdown, and immediate/natural stale release even though the header says decisions and implementation already exist (lines 420-439).
- **B31-053 [E]** Explicit side benefits are large-repo resilience, B-25 readiness, live progress, incremental B-29 graph builds, sane user cancellation, cross-session corruption prevention, and restart without manual <code>/index</code> (lines 441-456).
- **B31-054 [E]** B-35 depends on B-31 lock primitives; B-32 touches the same indexer; B-29/B-30 are parallel (lines 458-465).
- **B31-055 [E]** Mandatory inputs include the full SocratiCode indexer/lock/startup patterns, watcher interplay, full ELAI indexing service, multi-session rules, and Phase-1A spec (lines 70-90).

### B-34 — Deterministic entry-point detector

- **B34-001 [E]** Objective: deterministic orphan/name/framework-route detection exposed as a dispatcher tool, grounding Designer recursive decomposition and Researcher <code>/research</code> starting points; estimate 5–7 hours (lines 1-8).
- **B34-002 [E]** Requires B-29 <code>SymbolGraphCache</code>/per-file payloads and is parallel-safe with B-30/B-31/B-32/B-33/B-35 (lines 9-11).
- **B34-003 [E]** Existing Designer <code>EXISTING_PROJECT</code> flow otherwise hallucinates structure or mistakes filename matches for genuine entry points; Researcher has only file/symbol search and lacks routes/CLI starts (lines 13-30).
- **B34-004 [E]** Algorithm is LLM-free and attaches a reason string, so consumers see why each entry was classified (lines 32-42).
- **B34-005 [E]** <code>entry_point_names_per_language()</code> returns an ordered BTreeMap for 14 languages (lines 61-86).
- **B34-006 [E]** Exact lowercase-<code>main</code> languages are JavaScript, TypeScript, Python, Go, Rust, Java, Kotlin, Scala, C, C++, Swift, Ruby, and PHP; C# uses uppercase <code>Main</code>; Python separately recognizes <code>__main__</code> (lines 66-86).
- **B34-007 [E]** <code>EntryPoint</code> fields are symbol/file ID, qualified/basename, relative file, optional line, and typed reason (lines 88-97).
- **B34-008 [E]** <code>EntryPointReason</code> variants are <code>Orphan</code>, <code>WellKnownName(String)</code>, <code>FrameworkRoute(String)</code>, and <code>Test</code> (lines 99-105).
- **B34-009 [E]** <code>detect_entry_points(file_graph, payloads) → Vec&lt;EntryPoint&gt;</code> consumes <code>CodeGraph</code> and <code>SymbolGraphFilePayload</code> slices (lines 108-115).
- **B34-010 [E]** Orphan heuristic considers only files with no dependents and includes them only when their payload has at least one outgoing call; emitted ID/name/file are relative path, line is absent, reason is <code>Orphan</code> (lines 118-125).
- **B34-011 [E]** Well-known-name heuristic excludes <code>&lt;module&gt;</code>, is case-sensitive, uses per-language names, and emits symbol ID, qualified name, file/line, and the matched name (lines 126-134).
- **B34-012 [E]** HTTP framework callees are case-normalized <code>get</code>, <code>post</code>, <code>put</code>, <code>delete</code>, <code>patch</code>, <code>head</code>, <code>options</code>, <code>all</code>, and <code>use</code>, producing <code>framework:http-&lt;name&gt;</code> (lines 135-140).
- **B34-013 [E]** Callee <code>route</code> produces exact reason <code>framework:route</code> (lines 139-140).
- **B34-014 [E]** Framework hits are attributed to the enclosing symbol found by line-range containment at the call site (lines 141-145).
- **B34-015 [E]** Test heuristic excludes <code>&lt;module&gt;</code>, matches <code>^test[_A-Z]</code>, <code>^Test[A-Z_]</code>, or exact <code>it</code>/<code>describe</code>, and emits reason <code>Test</code> (lines 146-151).
- **B34-016 [E]** Dedup key is ordered <code>(id, reason)</code>; the same symbol may appear under different reasons but never twice under the same reason (lines 153-155).
- **B34-017 [E]** <code>find_symbol_at</code> sorts once by start line, checks inclusive <code>start ≤ target ≤ end</code>, and returns the deepest enclosure (highest start) (lines 157-162).
- **B34-018 [E]** Dispatcher tool name is <code>entry_points</code>; it gets project cache/file graph, lazily collects candidates, calls detector, and returns JSON <code>{ entries }</code> (lines 164-177).
- **B34-019 [E]** p99 target is under 500 ms on 10K files by loading payloads only for orphans and conventional filenames such as <code>main.*</code>, <code>index.*</code>, <code>__main__.py</code>, and <code>Main.kt</code> (lines 179-185).
- **B34-020 [E]** Files that are neither orphan candidates nor conventionally named must not cause payload loads (line 185).
- **B34-021 [E]** On <code>TechStackDecision::ExistingProject</code>, <code>research_decompose</code> calls the tool and injects <code>EXISTING_ENTRY_POINTS</code> into Designer context (lines 187-191).
- **B34-022 [E]** Designer instruction grounds every Phase in exactly one entry-point group and every Step in a code path traced from one entry point (lines 192-195).
- **B34-023 [E]** <code>/research</code> pre-queries the tool and injects <code>PROJECT_STARTING_POINTS</code> during Oracle formulation; tree expansion uses these as seeds (lines 197-201).
- **B34-024 [E]** <code>EntryPointsDetected</code> payload is <code>{ project_id, count, by_reason: BTreeMap&lt;String,u32&gt; }</code> and emits once when Designer or Researcher invokes the tool (lines 203-205).
- **B34-025 [E]** Produced symbols are detector, enclosure helper, entry types, name table, SSB variant, and dispatcher registration (lines 207-216).
- **B34-026 [E]** Consumers are research decomposition, CLI research formulation, and dashboard <code>list_entry_points</code>; reachability includes CLI research, dispatcher calls, and TaskRunner Designer briefing (lines 217-228).
- **B34-027 [E]** Empty vector is the defined “no entries” result; detector must not return <code>Result&lt;Vec, Error&gt;</code> (lines 229-232).
- **B34-028 [E]** Stub/contract invariants are p99 budget and impossibility of duplicate <code>(id, reason)</code>; B-33 graph/decomposition mode may highlight the results (lines 232-236).
- **B34-029 [E]** Observable research fixture expects <code>EntryPointsDetected</code> populated with at least GET, POST, and test reasons and an Oracle answer beginning at real files (lines 237-243).
- **B34-030 [E]** CLI deterministic output fixture includes Rust/Go <code>main</code>, C# <code>Main</code>, orphan <code>index.ts</code>, tests, and Express GET routes (lines 245-252).
- **B34-031 [E]** Manual performance check targets under 500 ms; Designer and Researcher logs/output must show injected blocks/routes (lines 254-268).
- **B34-032 [E]** Payload-selectivity evidence on 10K files requires <code>N &lt;&lt; 1000</code>, concretely under 10% (lines 276-280).
- **B34-033 [E]** Acceptance adds a stricter small-repo target under 100 ms, all four heuristics, 14-language table plus Python special case, deepest enclosure, CLI/IPC registration, both injections, event emission, selective loads, dedup, zero production unwraps, and no unconfirmed crates (lines 283-302).
- **B34-034 [E]** Orphan tests distinguish depended-on A, outgoing-call B, and inert C (lines 304-310).
- **B34-035 [E]** Name tests cover every language and case sensitivity: Rust <code>Main</code> is rejected; C# <code>Main</code> is accepted (lines 310-314).
- **B34-036 [E]** Framework tests cover <code>app.get</code>, <code>router.post</code>, and <code>Route::get</code> with correct enclosing attribution (lines 315-318).
- **B34-037 [E]** Test-name fixture expects <code>test_foo</code>, <code>Test_bar</code>, <code>it</code>, and <code>describe</code> but not <code>regular_func</code> (lines 319-321).
- **B34-038 [E]** Dedup test requires two records for one symbol under Orphan and WellKnownName, neither merged nor duplicated (lines 322-325).
- **B34-039 [E]** Performance/selectivity tests use synthetic 10K files and a recording cache; slow-CI timing may be skipped locally, while dispatcher contract freezes JSON shape (lines 326-333).
- **B34-040 [E]** File lifecycle creates four entry-point module files and decisions, exports them, registers dispatcher/SSB/UI/CLI, and modifies Designer/Researcher integration (lines 335-358).
- **B34-041 [E]** V2 <code>list_entry_points</code> can feed a B-33 graph or, if its alternate placement wins, orbit MAP mode (lines 351-355).
- **B34-042 [A]** Project <code>[entry_points]</code> append-only customization is only a decision-doc recommendation, not a settled task; the orientation summary overstates it as an implemented/firm configuration behavior (lines 360-370).
- **B34-043 [A]** Potential <code>subscribe</code>/<code>on</code>/<code>register</code> framework patterns are discussion-only; default is parity with the exact HTTP/route set (lines 362-366).
- **B34-044 [A]** <code>include_tests: bool</code> is an unresolved decision; recommendation is default include and consumer-side filtering (lines 378-380).
- **B34-045 [E]** Side benefits include accurate decomposition/research, B-33 highlighting, B-25 discovery, possible B-23 table growth, and user-facing <code>/entry_points</code> exploration (lines 382-395).
- **B34-046 [E]** Mandatory inputs are the full SocratiCode detector/table, B-29 symbol graph, Designer research tree, and Oracle implementation-order source (lines 44-57).
- **B34-047 [A]** The source has no SHIPPED header and all acceptance boxes remain open; “PLANNED” comes from the orientation summary rather than this canonical file (lines 1-402).
- **B34-048 [E]** B-29 is required; B-33 consumes highlights; B-35 is independent but co-feeds Designer context (lines 397-402).

### B-39 — Chat REFLECT pipeline

- **B39-001 [E]** Objective: opt-in per-message <code>TRIAGE → REASONER → AUDITOR → SYNTHESIZE</code> so nontrivial turns are grounded, sampled at N=4, audited, and rendered with existing GRC primitives and no new model weights; estimate two sessions (lines 1-7).
- **B39-002 [E]** Prerequisites are B-8 production <code>PipelineGrc</code>, B-14 streaming, and B-5 executor/auditor role configuration; starting SSB schema was 28 (lines 8-12).
- **B39-003 [E]** Parallel-safe set is B-37, B-38, B-22b, and B-23 because crate scopes differ (line 12).
- **B39-004 [E]** Agentless research anchor motivates isolated model calls/no shared KV between inference stages (lines 14-19).
- **B39-005 [E]** Verifier research sets best-of-N at N=4 for 3B–7B models and requires the auditor to score traces rather than re-solve (lines 20-26).
- **B39-006 [A]** Research premise says grounded test/tool/file evidence is mandatory because evidence-free auditing degenerates to resampling, but stage implementation allows <code>evidence = empty</code> when tool output is inaccessible (lines 27-30 versus 228-233).
- **B39-007 [E]** RefineBench is the basis for limiting refinement, while the 25K local ceiling yields a 20K pipeline cap and at least 5K nominal headroom (lines 31-35).
- **B39-008 [E]** AISLE threat model motivates stage isolation so user prompt injection cannot pollute verifier context (lines 36-39).
- **B39-009 [E]** Integration point is the top of <code>LiveCli::run_turn</code> before <code>runtime.run_turn</code>; <code>ConversationRuntime</code> is a field type and has no <code>send_message</code> method (lines 41-48).
- **B39-010 [E]** Baseline chat was pure user→sidecar→streamed reply with no reasoning/grounding/audit; B-39 adds the toggle, pipeline, four SSB actions, and ReasoningDrawer state (lines 43-59).
- **B39-011 [E]** Production role lookup must be added to <code>agent_grc_config_for_role</code>; changing only <code>TomlConfigLoader::load_config</code> silently misses the live path (lines 75-85).
- **B39-012 [E]** <code>chat_reasoner</code> config is <code>GrcBudget::new(4, 8192)</code>, steps Observe/Hypothesize/PlanAction, grounding High, no CSQ, and allowed FileCheck/SourceCheck (lines 87-95).
- **B39-013 [E]** <code>GroundingLevel::Medium</code> does not exist; High is intentional because the role reads freshly loaded code (lines 92-95).
- **B39-014 [E]** <code>chat_auditor</code> config is <code>GrcBudget::new(4, 5120)</code>, Observe/Verify, grounding High, no CSQ, allowed FileCheck/SourceCheck/TestCheck, and transition override Observe→Verify (lines 97-101).
- **B39-015 [E]** Role work reuses <code>AgentGrcConfig</code>, <code>GrcBudget</code>, <code>GroundingLevel</code>, <code>VerificationMethod</code>, and <code>GrcState</code>; no new role types (lines 103-104).
- **B39-016 [E]** Add four SSB variants and bump schema 28→29, including DDL CHECK strings and exhaustive test-payload arms in the same change (lines 106-136).
- **B39-017 [E]** <code>ReflectTriagedPayload</code> fields are turn ID, class (<code>Chat|Problem|CodeFix</code>), token count, and <code>skipped_to_synthesize</code> (lines 111-114; 140-141).
- **B39-018 [E]** <code>ReflectCandidatePayload</code> fields are turn ID, candidate index 0..N-1, token count, and summary truncated to at most 200 chars (lines 116-119; 142-143).
- **B39-019 [E]** <code>ReflectAuditVerdictPayload</code> fields are turn ID, best index, f32 confidence, u8 refine iteration, and evidence-source strings (lines 121-124; 144-146).
- **B39-020 [E]** <code>ReflectRefinedPayload</code> fields are turn ID, total tokens, four-element stage-token array in pipeline order, and remaining budget (lines 126-130; 147-149).
- **B39-021 [E]** All four payloads derive Serialize, Deserialize, Debug, and Clone (line 151).
- **B39-022 [E]** Original design places <code>ChatReflectPipeline</code> in <code>reasoning/src/chat_reflect.rs</code> and rejects placement in CLI as dispersing reasoning (lines 153-162).
- **B39-023 [A]** Original design adds an <code>elai-providers</code> dependency and accepts <code>&amp;dyn LlmProvider</code>, but the later wiring contract records the shipped divergence: local <code>ChatInferenceClient</code> in reasoning, <code>SidecarChatClient</code> in CLI, no providers dependency, and a CLI→reasoning dependency instead (lines 157-162; 195-208; 366-381).
- **B39-024 [E]** <code>TurnClass</code> exact values are <code>Chat</code>, <code>Problem</code>, and <code>CodeFix</code> (line 166).
- **B39-025 [E]** <code>ChatReflectConfig</code> fields/defaults are <code>enabled=false</code>, <code>n_candidates=4</code> constrained 1..=8, <code>budget_cap_tokens=20_000</code>, and originally <code>max_refine_iterations=2</code> (lines 168-174).
- **B39-026 [E]** <code>ReflectOutput</code> returns final reply, class, token summary, and <code>skipped</code>; Chat classification marks skipped and bypasses stages 2–4 (lines 176-181).
- **B39-027 [E]** <code>ReflectTokenSummary</code> stores four stage counts plus total (lines 183-186).
- **B39-028 [E]** Pipeline state contains config, separate reasoner/auditor <code>PipelineGrc</code>, and optional <code>Arc&lt;Ssb&gt;</code> for testability (lines 188-193).
- **B39-029 [E]** Public API is <code>new(config, ssb)</code> plus async <code>run(turn_id, user_message, inference/provider)</code> returning <code>ReflectOutput</code> or <code>ReflectError</code> (lines 195-208, with the dependency divergence in B39-023).
- **B39-030 [E]** TRIAGE is one roughly 1.5K-token call with only turn content and a fixed prompt returning exact lowercase <code>chat|problem|code-fix</code> (lines 210-217).
- **B39-031 [E]** Chat classification sets skipped, uses the user message unchanged for the local synth/pass path, avoids candidate/auditor work, and emits one <code>ReflectTriaged</code> row (lines 212-217).
- **B39-032 [E]** REASONER performs one isolated call per candidate under <code>chat_reasoner</code>, with no prior history, collects responses, and emits one <code>ReflectCandidate</code> per result (lines 219-224).
- **B39-033 [E]** Candidate budget guard stops launching remaining candidates when running total exceeds 85% of cap and continues with partial results (lines 225-226).
- **B39-034 [A]** The estimate “N=4 candidates, ~8K each” cannot fit a 20K cap; the contract explicitly tolerates overshoot by at most one candidate, so 20K is a launch guard rather than a hard never-exceed bound (lines 219-226; 335-337).
- **B39-035 [E]** AUDITOR receives all collected candidates, available grounded evidence, and verifier scoring instructions in a clean context; it returns best index and confidence (lines 228-233).
- **B39-036 [E]** Original refinement rule retries one REASONER using rejected candidates as negative examples when confidence is below 0.6, then re-audits and emits a verdict after each audit (lines 234-237).
- **B39-037 [A]** Most task/test/acceptance text caps refinement at 2, but the final decision section is an explicit later C-33 reconciliation saying implementation shipped <code>REFINE_CAP=3</code> and a contract asserts 3; current canonical value is therefore 3, while 2 remains historical/stale (lines 31-32; 172; 228-237; 333-334; 464-465 versus 543-546).
- **B39-038 [E]** SYNTHESIZE is local <code>candidates[best_index]</code> string selection, never another inference call, and emits <code>ReflectRefined</code> with token summary (lines 239-243; 387-388).
- **B39-039 [A]** The research intro says all four stages are separate <code>/v1/messages</code> calls, but SYNTHESIZE is explicitly local and no call; “KV reset per stage” applies only to model-calling stages/candidate calls (lines 16-19 versus 239-243).
- **B39-040 [E]** Every stage failure becomes <code>ReflectError::StageFailed { stage, source }</code>; LiveCli falls through to ordinary chat, so REFLECT cannot brick chat (lines 245-247).
- **B39-041 [E]** Production pipeline forbids <code>.unwrap()</code> and uses propagation/context mapping (line 249).
- **B39-042 [E]** Export <code>pub mod chat_reflect</code> from reasoning; crate-root re-export is optional only if build layout needs it (lines 251-255).
- **B39-043 [E]** Session state gains <code>reflect_enabled=false</code> and a config; slash parser adds exactly one <code>reason</code> spec with <code>&lt;on|off&gt;</code>, validating the argument in the handler (lines 257-271).
- **B39-044 [E]** Two <code>reason</code> specs are forbidden because first-word parsing makes the second unreachable (lines 267-271).
- **B39-045 [E]** Enabled gate intercepts before normal runtime; non-skipped success uses REFLECT output, while skipped or error falls through unchanged (lines 272-276).
- **B39-046 [E]** Slash-command spec-count assertion changes exactly 146→147 (lines 277-278).
- **B39-047 [E]** ReasoningDrawer adds a REFLECT panel beneath existing GRC gauges (lines 280-284).
- **B39-048 [E]** Toggle row says “Chat Reasoning (REFLECT),” displays teal when active, and dispatches the same slash command; no toggle-specific Tauri command (lines 285-288).
- **B39-049 [E]** Triage UI shows turn ID, class badge colors CHAT gray/PROBLEM amber/CODE-FIX blue, and skip state (lines 289-290).
- **B39-050 [E]** Candidate UI shows count and four proportional token bars; audit UI shows best index, confidence, and refine count (lines 291-294).
- **B39-051 [E]** Token UI has four stacked TRIAGE/REASONER/AUDITOR/SYNTHESIZE cells and a remaining/20K gauge red below 10% (lines 295-298).
- **B39-052 [E]** UI must use Svelte 5 <code>$state/$derived/$effect</code>, with no legacy reactive labels (lines 300-300).
- **B39-053 [E]** <code>get_reflect_state()</code> typed shape is enabled plus nullable last triage, verdict, and summary; four TypeScript payload aliases mirror Rust (lines 301-316).
- **B39-054 [A]** In this source, <code>get_reflect_state</code> remains mock/stub data and live Tauri wiring is deferred; any summary claim that a live Tauri command already exposes it is later evidence not contained here (lines 301-304; 531-532).
- **B39-055 [E]** Contract suite requires at least 12 deterministic mock-provider tests and no live sidecar (lines 318-322; 349-351).
- **B39-056 [E]** Triage tests prove Chat makes stage-token slots 1..3 zero, Problem runs all stages with nonzero tokens, and CodeFix runs all stages (lines 324-329).
- **B39-057 [E]** Candidate/auditor tests prove exactly N candidate rows, in-range best index, refinement cap, and budget abort behavior (lines 330-337).
- **B39-058 [E]** Accounting test proves total equals the four stage-token sum (lines 338-339).
- **B39-059 [E]** SSB tests require exactly one triage, at least one audit verdict per problem, and exactly one refined row per run (lines 340-345).
- **B39-060 [E]** Failure test returns <code>StageFailed</code> on REASONER HTTP 500 without panic/unwrap (lines 346-347).
- **B39-061 [E]** Produced contract includes pipeline/config/output/class/token/error, both production role lookups, four schema-29 actions, local inference trait/adapter, slash spec, drawer, and state API (lines 353-370).
- **B39-062 [E]** Consumers are PipelineGrc, BasicGrcEngine, role config/types, SSB, and <code>LiveCli::run_turn</code>; reachability is enabled turns and <code>/reason on</code> (lines 371-380).
- **B39-063 [E]** Stub bans forbid placeholder/TODO stage bodies, production mock data, production unwraps, and inference-based synthesis (lines 382-389).
- **B39-064 [E]** Observable timing expects triage within 500 ms, four candidates within 30s, audit within 60s, refined within 65s, plus a populated drawer (lines 390-397).
- **B39-065 [E]** Deterministic checks verify module/config/actions/schema, zero unwrap, one slash spec, 12 tests, clean three-crate check, state API, and five-run fixed-seed/mock triage stability under <code>ELAI_SEED=42</code> (lines 399-442).
- **B39-066 [E]** Acceptance requires no placeholders/TODOs, exact role budgets/methods, schema/DDLs/test-payload arms, single reason spec/count 147, Chat bypass, Problem full-stage accounting, all four bus actions, runes-only UI, typed state API, 12/0/0 tests, clean cargo check, and no new Svelte errors (lines 444-473).
- **B39-067 [E]** Unit tests additionally cover class parsing, config defaults, token-sum invariant, and non-panicking error Display (lines 475-482).
- **B39-068 [E]** File lifecycle creates <code>chat_reflect.rs</code>, contract tests, and decisions; modifies reasoning config/lib/dev dependency, SSB, LiveCli/CLI deps/commands, drawer, stub, and frontend types (lines 484-507).
- **B39-069 [E]** Deferred AISLE/Mythos adversarial suite waits for the 3090×2 box; per-stage isolation is already the mitigation (lines 509-519).
- **B39-070 [E]** Deferred AB-MCTS replaces flat best-of-N only after production measurement and costs roughly 3× (lines 520-523).
- **B39-071 [E]** Deferred <code>/think hard</code> budget forcing requires a REASONER prefix and waits for baseline stability (lines 524-526).
- **B39-072 [E]** Deferred prefix caching could save roughly 30% but requires verified sidecar <code>cache_control</code> support (lines 527-530).
- **B39-073 [E]** Live Tauri state and final UI placement (drawer pill versus Settings section) are explicitly deferred/user-review decisions in this source (lines 531-535).
- **B39-074 [E]** Decision doc must justify KV isolation and N=4, record the current shipped cap=3 reconciliation, and document triage false-skip, partial-candidate budget abort, and schema-reader break risks (lines 537-552).
- **B39-075 [A]** Source “Next” assigns live state wiring and future AB-MCTS to B-40/B-39b, while the orientation summary instead names B-39D/E/F and reports B-39C answer substitution; those later subplans are absent from this canonical file (lines 554-560).
- **B39-076 [A]** The canonical file has no top-level DONE marker, but its final decision text explicitly says cap=3 “implementation shipped”; status is at least partially post-implementation, while the orientation summary supplies the broader DONE/B-39C status (lines 1-560).
- **B39-077 [E]** Mandatory inputs are reasoning engine/config/types, orchestrator GRC/SSB, LiveCli dispatch, ReasoningDrawer/stub, and B-8/B-14/B-5 plan references (lines 61-73).

## Cross-file interactions

- **X-001 [E]** B-1 is the explicit prerequisite for both B-4 model/vector placement and B-31 checkpoint metadata placement; it also declares B-2 parallel-safe (B-1 lines 6-8; B-4 lines 5-8; B-31 lines 16-19).
- **X-002 [E]** B-1’s canonical helper surface is the shared authority for CLI/orchestrator/dashboard reachability; B-4 and B-31 must consume the same <code>.elai_cc</code> root rather than introducing feature-local literals (B-1 lines 58-69, 110-127).
- **X-003 [E]** B-2’s V2 StatusBar/Monitor consumers live inside the single Workspace shell that B-12 declares canonical; the phase writer is orchestrator-side and therefore independent of shell reorganization (B-2 lines 10-44; B-12 lines 11-24, 161-164).
- **X-004 [E]** B-2, B-4, B-24, B-31, B-34, and B-39 all define SSB observability, but with different control semantics: B-2 writes are best-effort; B-24 Tier 1 can veto; B-31 events evidence lifecycle; B-34 logs discovery; B-39 failures fall through (their respective wiring/failure sections).
- **X-005 [E]** B-4’s model staging explicitly relies on B-1’s <code>models_dir_path</code>; B-1 independently bans old <code>.elai/models/</code> literals (B-1 lines 87-89; B-4 lines 80-89).
- **X-006 [E]** B-31’s idempotent <code>migrateAbsolutePathKeys</code> is explicitly a post-B-1 compatibility migration, tying resume identity to relative canonical paths (B-31 lines 161-165).
- **X-007 [E]** B-31 and B-34 are declared parallel-safe, but both name B-29: B-31 supplies resilient/incremental index lifecycle while B-34 consumes B-29’s symbol cache/payload contract (B-31 lines 17-19, 449-451; B-34 lines 9-11).
- **X-008 [E]** B-31 explicitly enables B-35 artifact indexing to coexist with code indexing through shared lock primitives (B-31 lines 462-465).
- **X-009 [E]** B-12 depends on B-3 registration, B-7 decomposition, and B-11 tokens; B-24 depends on B-7 media-bearing PipelineSpec and B-22 autonomous iteration; none of those prerequisites is silently supplied by this chunk (B-12 lines 7-9; B-24 lines 8-10).
- **X-010 [E]** B-39 depends on B-5 role configuration, B-8 PipelineGrc production wiring, and B-14 streaming; its passthrough fallback deliberately preserves B-14 behavior when disabled/skipped/failing (B-39 lines 8-12, 245-278).
- **X-011 [I]** Once B-1 and B-31 compose, checkpoint/file-hash/lock state and the DKG/BM25/vector data they protect resolve beneath one canonical project root; neither source authorizes a second metadata root.
- **X-012 [I]** B-2’s phase events become visible only after both producer wiring and a live V2 read path; B-12 establishes where the StatusBar lives, while B-36 (outside this chunk) remains the named backend bridge.
- **X-013 [I]** B-31’s crash-safe batching protects the indexes queried by B-4; therefore duplicate-free resume is upstream data-integrity evidence for query traces/recall, even though neither file defines a direct B-31→B-4 API.
- **X-014 [I]** B-34 is read-only over graph/cache payloads; B-31’s Index/Watch locks govern mutation. A coherent implementation can let entry-point reads continue against the last completed snapshot during contention, but this behavior is not promised by either source.
- **X-015 [I]** V2 commands are the common bridge for B-2 state, B-4 knowledge, B-24 media, B-31 cancellation/progress, and B-34 entry-point display; exact command registration is repeatedly delegated to B-36 rather than owned by B-12’s shell.
- **X-016 [I]** Failure policies intentionally differ by risk: observation failure logs/continues (B-2), retrieval/model issues degrade or warn (B-4), advisory absence degrades while deterministic media failure vetoes (B-24), startup resume errors are nonfatal but lock contention prevents writes (B-31), no entries is an empty value (B-34), and chat failure passes through (B-39).
- **X-017 [I]** Determinism forms a shared boundary: B-1 centralizes paths; B-2 and B-4 use injected clocks; B-24 bans GPU SSIM/LPIPS and selects mode deterministically; B-31 uses blake3/relative IDs/RAII; B-34 uses BTree collections/case-exact rules; B-39’s contract mocks and fixed seed stabilize triage.
- **X-018 [I]** Concurrency mechanisms are orthogonal: B-24 bounds within-process media jobs (4/1/64), whereas B-31 arbitrates cross-process index/watch ownership; neither substitutes for the other.
- **X-019 [I]** Four activation gates coexist without conflict: project initialization (B-1), media-step presence (B-24), indexing-disable environment flag (B-31), and per-session <code>/reason</code> toggle default-off (B-39).
- **X-020 [I]** Source chronology matters: B-4 includes a May closure, B-31 a June shipped correction, B-12 a July closure, and B-39 a later C-33 reconciliation; the orientation snapshot predates at least some of these embedded updates.

## Orientation-summary additions, corrections, and omissions

- **SD-001 [SΔ correction]** The summary’s single status snapshot cannot override newer canonical in-file closures. Most materially, B-31 explicitly says SHIPPED rather than PLANNED (B-31 lines 1-10).
- **SD-002 [SΔ addition]** B-1’s summary-level DONE status is not stated in B-1 itself; the canonical source is still a task/acceptance document with unchecked gates.
- **SD-003 [SΔ omission]** B-1 summary omits legacy coexistence behavior: warn, prefer <code>.elai_cc</code>, and never auto-delete <code>.elai</code> (B-1 lines 95-98).
- **SD-004 [SΔ omission]** B-1 summary omits the model-path migration, canonical-path startup logging, exact three current CLI reader anchors, canary-bench observable contract, and four named regression tests (B-1 lines 87-93, 110-133, 156-192).
- **SD-005 [SΔ addition]** B-2 summary-level DONE status is external to B-2; this canonical file contains no completion marker.
- **SD-006 [SΔ correction]** B-2 summary correctly adopts orbit halo/bubble-strip V2 behavior, but any requirement for a textual ActivityEntry renderer is historical and conflicts with the source’s V2 rebaseline (B-2 lines 27-34, 203-211).
- **SD-007 [SΔ omission]** B-2 summary omits exact payload fields, one-second polling, optional-writer constructor behavior, direct-state-mutation audit, two-second visual target, and SQL/94-test done gates.
- **SD-008 [SΔ addition]** B-4 summary’s <code>ELAI_CODE_VEC=1</code> rollout flag is not present anywhere in this canonical B-4 file; it may come from another record and must not be attributed to this source.
- **SD-009 [SΔ correction]** B-4 summary says to correct DEEP MRR monotonicity, but canonical current state is PARTIAL: anchor fix shipped, measured DEEP remains below STANDARD, UI warns, and further improvement is out of B-4 scope (B-4 lines 138-146).
- **SD-010 [SΔ correction]** B-4 canonical error types are <code>KnowledgeError</code>, <code>VectorError</code>, and <code>Bm25Error</code>; the summary’s exact <code>PagerankError</code> name is not supported here (B-4 lines 211-215).
- **SD-011 [SΔ correction]** B-4 does not “remove” entropy quarantine; it explicitly preserves it through injected <code>self.clock</code> (B-4 lines 191-202).
- **SD-012 [SΔ omission]** B-4 summary omits the shipped C-10 absorption: <code>CanaryDrift</code>, schema 99, 232-payload bijection, 0.85 threshold correction, startup/30s producer, command, and Settings UI (B-4 lines 55-60, 172-186).
- **SD-013 [SΔ omission]** B-4 summary omits exact recall values, <code>known:false</code> freshness/cache fallbacks, failed-file drilldown, index upgrade state, script-name drift, and granular shipped/partial/open status.
- **SD-014 [SΔ ambiguity]** B-4 summary uses <code>get_query_trace</code>/<code>get_circuit_breaker_states</code>; canonical V2 naming also mandates <code>get_query_trace_recent</code>/<code>get_embedding_breakers</code>. Implementation audit must resolve the 1:1 API contract (B-4 lines 25-27, 110-119, 225-230).
- **SD-015 [SΔ correction]** B-12 summary frames five icons, Tier-0 preference, and drawer consolidation as verification/backfill; July canonical closure says they are deferred or superseded, while only Workspace default and internal StatusBar are shipped (B-12 lines 149-179).
- **SD-016 [SΔ correction]** B-12’s active sidebar is three buttons at 48px under a four-question-tab IA, not a currently accepted five-icon target (B-12 lines 165-170).
- **SD-017 [SΔ correction]** No layout-choice wizard or <code>primary_shell</code> setting should be restored under the single-shell rebaseline; the old deterministic check expecting that key is stale (B-12 lines 31-38, 137-147, 171-174).
- **SD-018 [SΔ omission]** B-12 summary does not report that V2 has roughly 60 drawers, making the original ten-drawer task materially larger and deferred (B-12 lines 175-179).
- **SD-019 [SΔ addition]** B-24 summary reports DONE status, concrete Mock/Ollama/SGLang clients, RCE crate/wiring, four Tauri commands, 71/21/19/2 passing-test ledgers, an ignored <code>cloud_mode_live</code> test, and a pinned Claude model; none appears in the canonical B-24 source.
- **SD-020 [SΔ correction]** B-24’s canonical source resolves the previously noted “six unnamed SSB variants” exactly: <code>media_validation_started</code>, <code>media_tier1_complete</code>, <code>media_tier2_complete</code>, <code>render_complete</code>, <code>visual_feedback_iteration</code>, and <code>media_human_review_required</code>.
- **SD-021 [SΔ omission]** B-24 summary omits queue depth 64, deterministic single-image/sequence mode choice, configurable default-five iteration cap, exact human-review action, and user-opt-in model download.
- **SD-022 [SΔ correction]** Canonical B-24 verification target is only “28+” (20 functional/contract + 8 adversarial); larger shipped ledgers in the summary require a different evidence source.
- **SD-023 [SΔ correction]** B-31 is SHIPPED, with exact implementation paths and production reachability; the summary’s PLANNED label is stale (B-31 lines 1-10).
- **SD-024 [SΔ omission]** B-31 summary omits <code>ELAI_DISABLE_INDEXING</code>, R9 allowlist registration, <code>IndexResult::{Cancelled,Resumed,AlreadyLocked}</code>, explicit cancellation tool/Tauri API, and the exact read-only contention banner.
- **SD-025 [SΔ omission]** B-31 summary compresses away exact metadata columns/defaults, store CRUD signatures, incremental deleted-file cleanup, event payload fields, stdin-EOF shutdown signal, and type-level lock-required contract.
- **SD-026 [SΔ ambiguity]** B-31’s source says the lock implementation is shipped but retains the fs2-versus-O_EXCL branch as a decision; the summary repeats the planned conditional and does not identify the selected production primitive.
- **SD-027 [SΔ correction]** B-31’s internal metadata table default <code>schema_version=1</code> coexists with a “migration v2”; the summary’s simple “metadata schema v2” should not erase this unexplained distinction.
- **SD-028 [SΔ omission]** B-34 summary omits the exact 14-language table, full HTTP callee set, test regexes, deepest-enclosure algorithm, empty-vector no-result semantics, <code>/entry_points</code>, <code>list_entry_points</code>, and stricter under-100ms small-repo target.
- **SD-029 [SΔ correction]** B-34 <code>[entry_points]</code> append-only override is only a recommendation awaiting the decision document, not a settled configuration behavior (B-34 lines 367-370).
- **SD-030 [SΔ ambiguity]** B-34’s include-tests flag and expanded event/plugin patterns are unresolved choices; canonical default recommendation is include tests and keep SocratiCode HTTP/route parity.
- **SD-031 [SΔ addition]** B-34’s PLANNED status is supplied by the orientation summary; the canonical source itself has no status line and leaves acceptance unchecked.
- **SD-032 [SΔ correction]** B-39 summary’s refine cap of 2 is stale against the canonical final C-33 reconciliation: implementation shipped cap 3 and contract asserts 3 (B-39 lines 543-546).
- **SD-033 [SΔ correction]** B-39 summary says every stage receives a separate model call/KV reset; canonical SYNTHESIZE is local selection and explicitly forbids another call (B-39 lines 239-243, 387-388).
- **SD-034 [SΔ addition]** B-39C answer substitution, <code>TerminalRenderer::render_markdown</code>, runtime bypass, and delegates <code>push_user_text</code>/<code>push_assistant_text</code>/<code>record_pipeline_usage</code> do not appear in this B-39 canonical source.
- **SD-035 [SΔ correction]** B-39’s canonical dashboard API is mock/stub with live Tauri wiring deferred; the summary’s <code>ssb_viewer.get_reflect_state</code>, live Tauri command, and “seventh ReasoningDrawer panel” are later external evidence absent here (B-39 lines 301-304, 531-532).
- **SD-036 [SΔ omission]** B-39 summary’s deferred list omits canonical AB-MCTS, <code>/think hard</code>, prefix caching, state-command wiring, and UI-placement review; conversely its B-39D/E/F items are not in this source.
- **SD-037 [SΔ confirmation/addition]** The summary correctly reflects the shipped dependency divergence—<code>ChatInferenceClient</code> in reasoning plus <code>SidecarChatClient</code> in CLI—rather than the earlier rejected providers dependency; its additional <code>ReflectEventSink</code> identifier is not present in this canonical file (B-39 lines 366-381).
- **SD-038 [SΔ omission]** B-39 summary omits exact chat role budgets/methods, event payload schemas, <code>n_candidates</code> range, 0.6 refine threshold, spec count 147, timing gates, 12 named contract behaviors, and the mandatory-evidence-versus-empty-evidence conflict.
- **SD-039 [SΔ ambiguity]** B-39’s overall DONE status comes from the summary; the canonical file only explicitly says the cap-3 implementation shipped and otherwise retains unchecked acceptance.
- **SD-040 [SΔ correction]** Summary-level “any pipeline error falls back” is accurate, but Chat classification also intentionally skips and falls through; default <code>reflect_enabled</code> is false, so not every chat turn runs all four logical stages.

## Zero-omission, duplicate, and accounting audit

### Per-file accounting

| Canonical source | Lines read | Bytes | Atomic ledger items | ID range | Section coverage |
|---|---:|---:|---:|---|---|
| B-1 | 218 | 9,817 | 36 | B1-001…B1-036 | objective/context, mandatory reads, all 10 tasks, wiring, deterministic checks, acceptance, tests, files, unlocks, decisions, next |
| B-2 | 245 | 11,543 | 34 | B2-001…B2-034 | V2 rebaseline, context, all 8 tasks, wiring, deterministic checks, acceptance, tests, files, decisions, next |
| B-4 | 338 | 16,931 | 46 | B4-001…B4-046 | both rebaseline/closure notes, context/reads, all 18 tasks, wiring, checks, acceptance, files, decisions, next |
| B-12 | 194 | 8,689 | 29 | B12-001…B12-029 | V2 rebaseline, historical context/8 tasks, wiring/checks, July closure acceptance, files, next |
| B-24 | 165 | 6,493 | 31 | B24-001…B24-031 | context/reads, all 8 tasks, wiring, checks, acceptance, files/dependencies, next |
| B-31 | 465 | 22,365 | 55 | B31-001…B31-055 | shipped header, context/reads, all 15 numbered tasks and four phases, wiring, checks, acceptance, tests, files, decisions, benefits, next |
| B-34 | 402 | 19,213 | 48 | B34-001…B34-048 | context/reads, all 8 tasks/four heuristics, wiring, checks, acceptance, tests, files, decisions, benefits, next |
| B-39 | 560 | 28,185 | 77 | B39-001…B39-077 | references/context/reads, all 8 tasks, divergence, wiring, checks, acceptance, tests/files, deferred work, final reconciliation, next |
| <strong>Total canonical</strong> | <strong>2,587</strong> | <strong>123,236</strong> | <strong>356</strong> | 8 disjoint prefixes | all feature-bearing sections represented |

### Mechanical and semantic audit

- Canonical ledger IDs are contiguous and unique within every prefix: 36 + 34 + 46 + 29 + 31 + 55 + 48 + 77 = <strong>356</strong>; no missing number and no duplicate ledger label.
- Cross-file interaction records are <strong>20</strong> unique items (<code>X-001…X-020</code>): ten direct/dependency compositions and ten explicitly labeled inferences.
- Summary-delta records are <strong>40</strong> unique items (<code>SD-001…SD-040</code>). They distinguish unsupported summary additions, canonical corrections, canonical omissions from the summary, and unresolved ambiguity.
- Total uniquely labeled analytical records are <strong>416</strong> (356 canonical atoms + 20 interactions + 40 summary deltas). Manifest rows and audit prose are not counted as feature atoms.
- Every original was read line 1 through its manifest EOF count. Section coverage was checked against each heading sequence: no task, payload/action identifier, command/surface, workflow state, schema/default, integration, permission/privacy constraint, failure/fallback, determinism/concurrency rule, observability gate, test/acceptance item, dependency/status/deferred item, or in-file contradiction was intentionally omitted.
- Repeated requirements (for example media 4/1 concurrency, B-39 cap 2, B-31 60-second shutdown) are represented once as a canonical atom and cross-referenced in ambiguity/delta items rather than double-counted as independent features.
- Explicit and inferred claims are segregated. No inferred item changes status, selects an unresolved implementation branch, or converts a recommendation into a requirement.
- Scope audit: originals opened = exactly 8/8 DEEP-B-002 rows; originals opened from any other chunk = 0. Orientation files used = exactly the permitted summary plus the DEEP-B-002 manifest locator.
