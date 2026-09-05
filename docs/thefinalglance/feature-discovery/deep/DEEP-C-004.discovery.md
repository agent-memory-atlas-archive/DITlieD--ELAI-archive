# DEEP-C-004 — exhaustive feature and behavior discovery

## Scope, method, and evidence labels

This ledger is a complete deep read of the exact `DEEP-C-004` manifest assignment. The only non-source document consulted was `03-C-SUMMARY_INDEX.discovery.md`, and it was used only to orient the read and to compute the explicit summary deltas near the end of this document. No original source assigned to another deep-read chunk was read.

Evidence labels:

- **[E] explicit** — directly required, reported, prohibited, deferred, or accepted by the cited source lines.
- **[I] inferred** — a bounded consequence of two or more explicit statements; it is not itself promised by the source.
- **[A] ambiguous** — the source leaves a material value, shape, lifecycle, or implementation choice unspecified.
- **[C] conflicting** — two explicit statements cannot both be read literally without reconciliation.

An **atomic item** below is one independently testable feature, behavior, status claim, schema, gate, or constraint. A schema is counted once even when all of its explicitly named fields are preserved in that item. Source reads/background paths and file-touch inventories are captured where they establish dependencies, implementation surfaces, or status, but repeated mentions of the same feature are not recounted as new atomic items.

## Exact processed manifest and integrity verification

Manifest section: `DEEP-SOURCE-MANIFEST.md` lines 403–418 (`### DEEP-C-004`). Manifest header totals and every row were independently checked with `sha256sum` and `wc -l -c`; all values match byte-for-byte.

| # | Class | Logical lines | Bytes | SHA-256 (manifest = observed) | Exact absolute path |
|---:|---|---:|---:|---|---|
| 1 | numbered/index original | 572 | 33447 | `7d83f7fc29b7f0ab7176a8d414e9528b10800082870c064b784316bd7182cdd4` | `docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md` |
| 2 | numbered/index original | 291 | 16011 | `a3682aef7400c0ea72f0e869ffc14c4cdae08044ce75cd9dad222d46e0ad1f93` | `docs/thefinalglance/C-plan/C-17_knowledge_indexing_resilience.md` |
| 3 | numbered/index original | 263 | 14337 | `574bdaa442252088850291dea69a4a207ed91f7407f73fa1b65ec3b2a755d75c` | `docs/thefinalglance/C-plan/C-21_self_mod_anti_stale_automation.md` |
| 4 | numbered/index original | 302 | 17106 | `1dbf30473c2c81d68ea1654fdb857619519dc7884def6bd44531ac9e123189fb` | `docs/thefinalglance/C-plan/C-25_compiled_pipeline_template_crystallizer.md` |
| 5 | numbered/index original | 317 | 17743 | `e91e5293d81da1d29c6363d88d0b18c802feba879c83661089d035e51576fcac` | `docs/thefinalglance/C-plan/C-26_constrained_decoding_retry_strategy.md` |
| 6 | numbered/index original | 209 | 11120 | `838141d7c9c0b6de8c2e9856aac30b6efb9967b9edc781e4d51c045d64b81daf` | `docs/thefinalglance/C-plan/C-38_pagerank_production_contract_aider_spec_fix.md` |
| 7 | numbered/index original | 147 | 6545 | `57d6d3add2451979a2f69e3f2526f256872c52ed53dcf948b42cc6c65b9b7120` | `docs/thefinalglance/C-plan/C-46_uno_orchestra_parsimonious_routing.md` |
| 8 | numbered/index original | 133 | 5675 | `d1f5ae104bb65eb05ee11ed7d9e96e25c8ea460980d5ed0f1da962f318320ba6` | `docs/thefinalglance/C-plan/C-47_case_based_calibration_tool_use.md` |
| 9 | numbered/index original | 169 | 7150 | `b722a8dc3544fd53ec0475ce8795215c757dd76b9f1c86b67e855c1737d8818a` | `docs/thefinalglance/C-plan/C-52_dafny_formal_verification_spike.md` |
| 10 | numbered/index original | 248 | 11790 | `d5c1efcb7bff65299413816e9a1f15fbb9bf9f4b45b58e2b01a63fd9a8bc8acc` | `docs/thefinalglance/C-plan/C-54_swe_bench_verified_run_three_configs.md` |
| **Total** |  | **2651** | **140924** | **10/10 hashes matched** | **10 files** |

## Per-file exhaustive ledgers

### C-8 — Viewer Stub Eradication: Wiring Remaining Producers

#### Status, scope, dependencies, and lifecycle

- **F-C8-001 [E]** C-8 is explicitly **CLOSED** at the 2026-05-23 rebaseline; all 12 producer-wiring tasks shipped in session `s-32952`, and session `s-25120` performed the completion-drive recheck. [C-8 L1-L10]
- **F-C8-002 [E]** The recheck changed two prior `PARTIAL` findings to `STALE-SHIPPED-ON-V2`, two prior `MISSING` findings to `PHANTOM-CLAIM-RESOLVED`, applied six `SPEC_DRIFT` fixes, and refined four deterministic checks. [C-8 L5-L27]
- **F-C8-003 [E]** Four contract-test files—three added and one extended—pin the wiring witnesses; the reported audit result is 14/14 passing. [C-8 L9-L10, L389-L395, L453-L483]
- **F-C8-004 [E]** The preserved objective replaces live-data mocks/hardcoded producer stubs across ambiguity scoring, Cosmos scheduling/events/stage integration, artifact indexing/RPC, reranking notification, A-RAG semantic scoring, pipeline latency, degradation/search-mode state, and dashboard unwired flags. [C-8 L33-L43]
- **F-C8-005 [E]** C-8 requires C-1 for `QueryTrace`/`CosmosRefreshStarted` SSB variants and C-2 for confirmed `SsbNotifier` injection paths; it is declared parallel-safe with C-3 and C-4. [C-8 L46-L50]
- **F-C8-006 [E]** The decision record is `workflow/reasoning/C-8-decisions.md`, containing DRIFT-A..M, R1–R4, T1–T4, and A1–A4 rationale/amendments. [C-8 L29-L31, L554-L557]

#### Capabilities, actions, surfaces, schemas, and workflows

- **F-C8-007 [E]** `AmbiguityScorer::score(&plan_text).await` is called from `task_runner.rs` only when constructing `TransitionContext` for the `PLANNING -> TESTING_FIRST` transition, before `GroundingSweepGate`; it is not called on every step. [C-8 L94-L98, L265-L267, L307-L309]
- **F-C8-008 [E]** Ambiguity scoring failure logs `tracing::warn!`, stores `ambiguity_score: None`, and intentionally preserves the B-17 fallback behavior; success stores `Some(score)`. [C-8 L94-L99, L400-L401]
- **F-C8-009 [E]** The live viewer method is `SsbViewer::latest_ambiguity_scored()`, not the originally proposed `last_ambiguity_score()`; the shipped name follows sibling `recent_*`/`latest_*` conventions. [C-8 L100-L113]
- **F-C8-010 [E]** `get_ambiguity_state` queries the newest `ambiguity_scored` row, returns the real score structure, and omits the former `"mock"` field. [C-8 L112-L116, L330-L332, L402]
- **F-C8-011 [E]** The Cosmos hourly scheduler is hosted in `cli/src/live_cli.rs` within `build_local_knowledge_stack`, not in the per-task `task_runner.rs`; this gives it orchestrator-process lifetime. [C-8 L118-L138, L334-L340]
- **F-C8-012 [E]** Scheduler bootstrap occurs only when an `Arc<Ssb>` is present and the manifest-present branch succeeds; CLI cold-start paths without an SSB writer remain unaffected. [C-8 L121-L130]
- **F-C8-013 [E]** The interval is `DEFAULT_TICK_INTERVAL_SECS = 3600`; each cycle calls `cosmos_tick_emit_loop(&ssb_for_tick, &worker_for_tick).await`. [C-8 L130-L134, L403-L406]
- **F-C8-014 [E]** The wrapper emits one `CosmosRefreshStarted` SSB row and one `CosmosArtifactUpdated` row for each refreshed artifact name; `CosmosWorker::tick()` stays synchronous and storage-only, with no `SsbNotifier` field added. [C-8 L131-L136, L268-L271]
- **F-C8-015 [E]** `IndexingService::build_index_resumable` invokes the artifact hook after code-phase completion through `maybe_index_artifacts`; artifact failure is non-fatal, warns, and allows core indexing to continue. [C-8 L140-L153, L407-L410]
- **F-C8-016 [E]** Successful artifact indexing emits `ArtifactsIndexed`; the originally proposed `ArtifactStale` rows are explicitly dropped because staleness is private to `index_artifact` and is represented in returned `ArtifactIndexState`, partitioned into `indexed[]` and `up_to_date[]`. [C-8 L140-L153]
- **F-C8-017 [E]** `TOOL_INDEX_ARTIFACTS` replaces the prior `Err("full indexing deferred to Phase C")` stub, reusing the pre-existing `ArtifactsIndexed` action/payload from B-34 rather than adding a duplicate schema variant. [C-8 L155-L160]
- **F-C8-018 [E]** The public `IndexArtifactsRequest` remains singular for IPC wire compatibility; the handler internally wraps it as a one-element `Vec<IndexArtifactsRequest>` to ease a future bulk-frame decode upgrade without current public API churn. [C-8 L161-L166]
- **F-C8-019 [E]** The RPC opens the manifest and store, calls `index_artifact` per specification, emits exactly one `ArtifactsIndexed` row, and returns JSON with `project_id`, `artifact_count`, `chunk_count`, `elapsed_ms`, `indexed[]`, and `up_to_date[]`. [C-8 L166-L174]
- **F-C8-020 [E]** Artifact RPC errors propagate as `AgentRpcError::ToolError`; panic is forbidden. [C-8 L168-L170]
- **F-C8-021 [E]** Production `CrossEncoderReranker` construction receives the real `Arc<Ssb>` coerced to `Arc<dyn SsbNotifier>`; production construction of `NoopSsbNotifier::{new,default}` is banned. [C-8 L176-L179, L352-L357, L415-L417]
- **F-C8-022 [E]** Search-mode state crosses the CLI/dashboard process boundary through a new `SearchModeChanged` SSB row and `SearchModeChangedPayload`, replacing the infeasible in-memory `AppState Arc<RwLock<SearchMode>>` design. [C-8 L181-L189]
- **F-C8-023 [E]** The CLI emits search mode once at startup and on every `DegradationCascade` transition. [C-8 L185-L188, L418-L420]
- **F-C8-024 [E]** Dashboard `get_search_mode` reads `SsbViewer::latest_action_of_type("search_mode_changed")`; if no row exists, it gracefully returns lowercase `"hybrid"`, not a hardcoded uppercase `"HYBRID"` production answer. [C-8 L187-L190, L359-L363, L421-L422, L527-L529]
- **F-C8-025 [E]** A new dashboard-side `set_search_mode` Tauri command emits a `SearchModeChanged` row, allowing reviewers to pin the diagnostic mode. [C-8 L190-L191, L527-L530]
- **F-C8-026 [E]** Adding the cross-process search-mode event atomically raises `SCHEMA_VERSION` 49→50 and the action/payload count assertion 101→102, including enum, payload, and `from_str` arms. [C-8 L192-L196, L278-L281, L506-L509]
- **F-C8-027 [E]** Dashboard `unwired.searchMode` is switched to `false`, and TypeScript `SearchMode` accepts lowercase wire tags in addition to the legacy uppercase enum. [C-8 L196-L197, L423, L531-L534]
- **F-C8-028 [E]** `AragOrchestrator::semantic_search` receives an optional injected ONNX `OrtSession` via `with_embedder()` and uses query/document dot product when available. [C-8 L199-L202, L424-L425, L487-L490]
- **F-C8-029 [E]** If the ONNX embedder fails, semantic search logs a warning and retains the old norm-proxy behavior as a fallback. [C-8 L282-L283, L424-L425, L487-L490]
- **F-C8-030 [E]** `QueryPipeline` gains `clock: Arc<dyn ClockProvider>` and a `with_clock()` builder; production installs the process `OsClock`, while deterministic tests can supply `SeededClock`. [C-8 L204-L213, L426-L429]
- **F-C8-031 [E]** All 17 ordinary `PipelineStage` latency literals plus the two synthetic `deep_cap`/`rrf_gate` emit sites compute milliseconds using `clock.elapsed_since(stage_start).as_millis() as u64`; literal `latency_ms: 0` count must be zero. [C-8 L212-L221, L365-L371, L426-L429, L498-L503]
- **F-C8-032 [E]** Raw `std::time::Instant` is prohibited for pipeline instrumentation by entropy quarantine; the sole scoped exemption is per-RPC `TOOL_INDEX_ARTIFACTS.elapsed_ms`, avoiding a public constructor expansion. [C-8 L204-L227, L283-L285]
- **F-C8-033 [E]** C-8 creates a new `CosmosStage { worker: Arc<CosmosWorker> }` with stable `const fn name() -> "cosmos"` and `run(&self, query) -> Vec<RetrievalResult>`. [C-8 L229-L234]
- **F-C8-034 [E]** `CosmosStage` is inserted between `ArtifactStage` and `ARagStage` for `STANDARD` and `DEEP` retrieval tiers only; `FAST` omits it. [C-8 L234-L236, L373-L377, L430-L432]
- **F-C8-035 [E]** Cosmos routing lowercases and alphanumeric-splits the query and artifact `name`/`research_query`, drops tokens shorter than three characters, and matches by token overlap. [C-8 L236-L239]
- **F-C8-036 [E]** Each matching Cosmos artifact becomes one `RetrievalResult` with curated score `0.75` and a 1,024-character snippet cap; downstream RRF supplies relative ranking. [C-8 L239-L242]
- **F-C8-037 [E]** The proposed `unwired.ambiguityState` flag is a phantom: it never existed. `GatesPanel.svelte` and `OrchestrationDrawer.svelte` invoke `get_ambiguity_state` directly, so there is nothing to flip. [C-8 L244-L257, L433-L436]
- **F-C8-038 [E]** `trigger_cosmos_refresh` remains an uncalled per-task helper in `task_runner.rs`; follow-up must either wire it from a task-entry hook or document its removal. [C-8 L536-L542, L561-L565]
- **F-C8-039 [E]** A frontend per-mode `SearchModeBadge` picker that calls `set_search_mode` remains pending, although its backend command is complete. [C-8 L566-L567]
- **F-C8-040 [E]** Several stale exact-schema-version contract tests still need migration to the C-7 `>=` floor pattern, and a platform-aware `std::time` guard is still needed for a pre-existing artifacts clock-overflow failure. [C-8 L568-L572]

#### Reachability, observability, failure rules, and verification

- **F-C8-041 [E]** Ambiguity, Cosmos retrieval, and reranking are reachable from `elai /research --mode cloud`; artifact indexing is reachable through the agent in `elai /task --mode cloud`; the indexing hook is reachable through `reindex_now`; search mode is reachable through Tauri commands. [C-8 L301-L306]
- **F-C8-042 [E]** Stub bans require: ambiguity scoring only at its transition gate, real Cosmos SSB writes rather than tracing alone, an `Ok` artifact RPC, no production no-op reranker notifier, no literal-zero pipeline latency, and no hardcoded search-mode result. [C-8 L307-L313]
- **F-C8-043 [E]** The primary observable research trigger expects `ambiguity_scored` with non-null score and no `mock` key, positive `query_trace.total_latency_ms`, and `cosmos_refresh_started` when configured/ticked. [C-8 L314-L321]
- **F-C8-044 [E]** The primary artifact observable requires `TOOL_INDEX_ARTIFACTS` to return `Ok` with a present `artifact_count` (which may be zero). [C-8 L320-L321]
- **F-C8-045 [E]** Twelve deterministic checks cover source symbols/absence patterns, scheduler host, artifact hook, stub removal, notifier construction, function-scoped search-mode hardcoding, all latency sites, Cosmos stage, dashboard flag, workspace compilation, and four contract-test invocations. [C-8 L323-L396]
- **F-C8-046 [E]** `production_agent_rpc_contract` test F asserts a result with parseable `artifact_count: u32` plus `project_id`, `chunk_count`, `elapsed_ms`, `indexed[]`, and `up_to_date[]`; the suite reports 6/6. [C-8 L170-L174, L455-L459]
- **F-C8-047 [E]** `ambiguity_task_runner_contract` has three cases: qualifying score+streak allows through both gate APIs, a high `overall_score` blocks with an ambiguity-tagged reason, and `None` preserves streak-only allow; it reports 3/3. [C-8 L460-L465]
- **F-C8-048 [E]** `cosmos_tick_wiring_contract` has three cases: one-artifact emits queued/total 1, empty manifest emits one unconditional started row with zeros, and empty manifest yields zero updates/refreshed count; it reports 3/3. [C-8 L466-L475]
- **F-C8-049 [E]** The Cosmos contract tests the helper rather than the live scheduler host; host construction remains a separate CLI integration witness. [C-8 L472-L475]
- **F-C8-050 [E]** `indexing_service_artifact_hook_contract` proves absent manifest is a documented no-op and malformed TOML is caught/warned without failing core indexing; it reports 2/2. [C-8 L476-L481]
- **F-C8-051 [E]** A prior six-assertion `c8_stub_eradication_contract` pins schema floor, SQL-string mapping, `from_str` round-trip, payload round-trip, `ArtifactsIndexed` round-trip, and history ordering. [C-8 L544-L547]
- **F-C8-052 [E]** Workspace compilation and all 12 deterministic checks are marked passing; the rebaseline acceptance checklist is otherwise fully checked or explicitly resolved. [C-8 L398-L440]
- **F-C8-053 [E]** The live Cloud-Sonnet smoke is explicitly deferred to C-9+ because it needs Sonnet plus configured `[[cosmos.artifacts]]`; the deterministic T1–T4 contracts are the present closure evidence. [C-8 L441-L451]

#### Ambiguities, conflicts, and bounded inference

- **F-C8-054 [A]** `reachable_from` still glosses `get_search_mode` as an `AppState.search_mode live read`, while the rebaseline explicitly replaced that design with an SSB viewer read; the latter is the authoritative shipped design. [C-8 L181-L197 versus L301-L306]
- **F-C8-055 [A]** Task 4 retains the phrase “wire `index_all_artifacts`,” but the rebaseline says the implementation calls per-artifact `index_artifact` through `maybe_index_artifacts`; exact implementation naming must follow the rebaseline. [C-8 L140-L153, L342-L346]
- **F-C8-056 [I]** Because the cloud smoke is deferred while all deterministic contracts pass, `CLOSED` here means source-level/contract closure, not proof of the named live cloud scenario. [C-8 L3-L10, L437-L451]

### C-17 — Knowledge Indexing Resilience

#### Status, scope, dependencies, and capabilities

- **F-C17-001 [E]** C-17 is a forward implementation plan with every acceptance box unchecked; it aims to close B-30, B-31, B-31.5, B-34, and phase-3K indexing completeness, fidelity, and lifecycle gaps. [C-17 L1-L8, L233-L268]
- **F-C17-002 [E]** It requires C-15 clock/entropy stability and C-14 dead-code stripping, and is declared parallel-safe with C-16 and C-18–C-20. [C-17 L8-L10]
- **F-C17-003 [E]** `KotlinGrammar` mirrors `JavaGrammar`; it uses a verified `tree-sitter-kotlin` workspace dependency if available, otherwise reuses the Java parser while reporting `language_id = "kotlin"`. [C-17 L60-L64]
- **F-C17-004 [E]** Adding any Kotlin parser dependency is guarded by `cargo search tree-sitter-kotlin` to prevent slopsquatted crate adoption. [C-17 L62-L64]
- **F-C17-005 [E]** `"kt"` and `"kts"` are added to `ext_to_lang_id`, and `KotlinGrammar` is registered in `LanguageRegistry::default()` after an impact check of callers. [C-17 L66-L68]
- **F-C17-006 [E]** New `svelte_extraction.rs` tests require TypeScript reparse of `<script>` with `<file>.svelte::` qualified names, CSS `@import` isolation from script symbols, and no panic for minimal Svelte. [C-17 L70-L74]
- **F-C17-007 [E]** CLI startup calls `auto_resume_indexed_projects` after initialization and before task dispatch; both CLI and dashboard must remain callers. [C-17 L76-L79, L169-L172]
- **F-C17-008 [E]** CLI shutdown handles cross-platform `tokio::signal::ctrl_c` and Unix-only `SignalKind::terminate`, routing both to `graceful_shutdown`; dashboard `ExitRequested` remains another required reachability path. [C-17 L81-L84, L169-L172]
- **F-C17-009 [E]** A new `FileWatcherService` acquires `LockScope::Watch` through `IndexLock::try_acquire` on project start, releases it on stop, and is wired beside the completed-project watcher-start path in `startup.rs`. [C-17 L86-L90]
- **F-C17-010 [E]** A new `indexing_resilience_contract.rs` contains four assertions: `#[must_use]` regression for `IndexResult`, type-level `LockScope` requirement for `IndexLock`, empty-project auto-resume completion within five seconds, and graceful-shutdown deadline in `[30,120]` seconds. [C-17 L92-L98]
- **F-C17-011 [E]** `ChunkKind` gains `Heading` and `Paragraph`; non-code files first use `# `/`## ` heading boundaries and blank-line paragraph splitting before the 50-line fallback. [C-17 L100-L105]
- **F-C17-012 [E]** The explicit non-code route excludes `.rs`, `.py`, `.ts`, `.go`, `.java`, `.cs`, and `.rb`; Markdown acceptance specifically requires the new chunk kinds. [C-17 L103-L105, L246-L247]
- **F-C17-013 [E]** `AragOrchestrator::semantic_search` replaces the norm proxy with an existing dual-vector ONNX session’s query embedding dotted against chunk embeddings, while preserving `SEMANTIC_OVERSAMPLE_FACTOR = 3`. [C-17 L107-L113]
- **F-C17-014 [E]** The implementation must discover the actual `OnnxEmbeddingModel`/`EmbeddingSession` API before assuming its shape. [C-17 L107-L111]
- **F-C17-015 [E]** `/research` detects project entry points, formats `PROJECT_STARTING_POINTS`, prepends it to the research query, and emits `EntryPointsDetected`. [C-17 L114-L118]
- **F-C17-016 [E]** Entry-point pre-seeding is conditional on `project_dir.is_some()` and otherwise no-ops. [C-17 L114-L118]
- **F-C17-017 [E]** `run_tier_a.sh` parses real `batch_complete`-style tracing spans to calculate nonzero per-batch p50/p99 wall-clock values and writes them to `phase_b31_5_tier_a.toml`. [C-17 L120-L124]
- **F-C17-018 [E]** `run_all_tiers.sh` runs against a real `ELAI_SCALE_FIXTURE_ROOT` containing at least Linux kernel v6.7 and VS Code 1.94.2; Chromium and Postgres are stretch goals. [C-17 L126-L130]
- **F-C17-019 [E]** Scale acceptance runs `scale_resume_identity_contract`, `scale_lock_contention_contract`, and `scale_long_tail_memory_contract` against populated per-tier TOMLs. [C-17 L126-L132]
- **F-C17-020 [E]** Dashboard `ActivityFeed` subscribes to SSB, filters action type, and renders `IndexResume` as “Resumed indexing from batch N/M (saved X files)”. [C-17 L134-L139]
- **F-C17-021 [E]** The same SSB-driven UI renders `LockContested` as “Another ELAI process (PID X) is indexing; read-only mode”; polling alone is explicitly insufficient. [C-17 L134-L139, L256-L257]
- **F-C17-022 [E]** `C-17-decisions.md` records the Kotlin fallback, ONNX session access, watcher lifecycle, and per-batch timing parsing choices. [C-17 L141-L144]

#### Integration contracts, observability, failures, and done gates

- **F-C17-023 [E]** Produced surfaces include `KotlinGrammar`, `kt`/`kts` mapping, `ChunkKind::{Heading,Paragraph}`, `heading_boundary_chunks`, `paragraph_chunks`, ONNX dot-product search, project-starting context, `FileWatcherService`, two test files, and two SSB-driven banners. [C-17 L146-L156]
- **F-C17-024 [E]** Consumed interfaces are `LanguageGrammar`, the existing ONNX embedding session, `detect_entry_points`, startup resume/shutdown APIs, `IndexLock::try_acquire`, and `SsbActionType::{IndexResume,LockContested}`. [C-17 L157-L163]
- **F-C17-025 [E]** Primary reachability is CLI startup/shutdown, `/research`, watcher service, and dashboard ActivityFeed. [C-17 L164-L168]
- **F-C17-026 [E]** Stub bans prohibit single-entrypoint-only resume/shutdown, the semantic norm proxy, and filesystem watching that bypasses `LockScope::Watch` locking. [C-17 L169-L175]
- **F-C17-027 [E]** The end-to-end observable indexes Rust/Kotlin/Svelte/Markdown, interrupts mid-index, restarts, then researches `FooService`; required events include resumed batch >0, entry points, and Kotlin grammar loaded. [C-17 L176-L184]
- **F-C17-028 [E]** Required artifacts include Kotlin DKG entries, real dot-product query results, `PROJECT_STARTING_POINTS` context, and heading/paragraph chunks for Markdown. [C-17 L176-L184]
- **F-C17-029 [E]** Deterministic checks assert source symbols/files, no norm/proxy remnants, CLI lifecycle wiring, contract/test presence, scoped crate compilation/tests, and positive tier-A timing. [C-17 L185-L231]
- **F-C17-030 [E]** The cloud smoke additionally requires `files_skipped_on_resume > 0` (no full reindex) and a Sonnet 4.6 research execution over the fixture. [C-17 L258-L266]
- **F-C17-031 [E]** Completion requires a clean workspace build and the four-decision reasoning document in addition to all feature/test/smoke boxes. [C-17 L267-L268]
- **F-C17-032 [E]** The explicit edit inventory spans knowledge DKG/BM25/A-RAG, CLI main/live CLI, orchestrator watcher/startup, knowledge/contracts tests, scale scripts/metrics, ActivityFeed, and the decision artifact. [C-17 L270-L286]

#### Ambiguities, conflicts, and bounded inference

- **F-C17-033 [A]** The Java-parser Kotlin fallback guarantees a `kotlin` language identifier but does not specify how Kotlin-specific syntax that Java’s grammar rejects is recovered or scored. [C-17 L60-L64]
- **F-C17-034 [A]** “Non-code files” is broader than the acceptance text’s Markdown-only assertion; exact extensions routed to heading versus paragraph splitting, and precedence when both apply, are not specified. [C-17 L100-L105, L246-L247]
- **F-C17-035 [A]** The source asks to parse a “batch_complete span or similar” but does not freeze a tracing record schema, percentile algorithm, warm-up policy, or sample-count minimum. [C-17 L120-L124]
- **F-C17-036 [I]** All acceptance boxes remain unchecked, so the source specifies desired behavior and cannot by itself be used as shipped evidence. [C-17 L233-L268]

### C-21 — Self-Modification and Anti-Staleness Automation Closure

#### Status, permissions, automation, schema, and security

- **F-C21-001 [E]** C-21 is an unchecked forward plan to close B-16, B-23, phase-4H, and anti-staleness-enforcement wiring gaps; it requires C-15 and is declared parallel-safe with C-16–C-20. [C-21 L1-L11, L224-L237]
- **F-C21-002 [E]** Windows CI runs `bash scripts/check_anti_stale.sh` after the HashMap-ban step in strict mode; a nonzero exit fails the job. [C-21 L54-L58]
- **F-C21-003 [E]** B-16B Chunk C is a hard permission gate: prompt exactly for proceeding with roughly 12 `.unwrap()` sites, accept explicit `Y`, and do not modify those sites on `N` or silence. [C-21 L35-L36, L60-L64]
- **F-C21-004 [E]** If approved, Chunk C reads the recorded site list and converts the deterministic-auditor/DKG calls with `?` or `.ok_or()`; `.expect()` is prohibited. [C-21 L60-L64]
- **F-C21-005 [E]** `scripts/install_pre_commit_hook.sh` installs behavior into `.git/hooks/pre-commit`; the generated hook is local and not committed, while the installer is committed. [C-21 L66-L74]
- **F-C21-006 [E]** The hook inspects staged paths and blocks `elai_cc.toml` plus files matching `*safety_locks*`, `*auditor_config*`, `*cloud_monthly_budget*`, `*benchmark_harness*`, `*test_suite*`, `*score_weights*`, `*gate_definitions*`, `*lock_list*`, `*cloud_opus_allowed*`, `*cloud_escalation_threshold*`, or `*config_variants_schema*`. [C-21 L66-L72]
- **F-C21-007 [E]** A locked-key match exits 1 and names the path using `BLOCKED: commit modifies LOCKED_KEYS file: <path> (spec phase-4H.txt:201-204)`. [C-21 L68-L74, L166-L169]
- **F-C21-008 [E]** `dep_audit_to_stale_catches(&DepAuditReport) -> Vec<StaleApiCatch>` maps real dependency-audit output into `FullPipelineOutcome.stale_api_catches` at each builder site where a report exists; absence of an audit leaves the field empty. [C-21 L76-L82]
- **F-C21-009 [E]** A successful `DataTable::DeprecatedPatterns` promotion asynchronously spawns `metrics/compute_steering_vector.py`, logs the child PID, and emits `steering_vector_refresh_triggered`. [C-21 L84-L90]
- **F-C21-010 [E]** A missing steering-vector script warns and never panics; no process is spawned and no event is promised for an unsuccessful promotion. [C-21 L84-L90]
- **F-C21-011 [E]** Layer1A `DIVERGING` calls nonblocking `trigger_dep_audit_background()`, which starts detached `elai dep-audit --output-format toml` and writes `metrics/dep_audit_latest.toml`. [C-21 L92-L97]
- **F-C21-012 [E]** Dependency-audit spawning is rate-limited by `metrics/.dep_audit_last_trigger`; a sentinel modified less than 24 hours ago suppresses the trigger. [C-21 L92-L97]
- **F-C21-013 [E]** Open `AllowedConfigKey::Parameter(String)` is replaced by `AllowedConfigKey::Parameter(ParameterKey)` with no string fallback. [C-21 L99-L109, L166-L177]
- **F-C21-014 [E]** `ParameterKey` has exactly 12 variants: `EtcThreshold`, `FreshnessHalfLife`, `FreshnessDocHalfLife`, `FreshnessBoostNorm`, `BabyStepMaxLoc { vram_tier: u8 }`, `ContextSlotWeight { slot: u8 }`, `RetryBudget`, `AbMctsMaxBranches`, `AbMctsTemperatures`, `MicroReflectionMaxTokens`, `SteeringVectorCoefficient`, and `SelfModFrequency`. [C-21 L99-L105]
- **F-C21-015 [E]** Match arms and proposal target configuration types are migrated across `self_mod_layer1a`, `self_mod_safety`, `self_mod_bridge`, `self_mod_orchestrator`, and `self_mod_types`; constructing `Parameter("cloud_monthly_budget")` must fail compilation. [C-21 L105-L113]
- **F-C21-016 [E]** `ConstraintsBlock` queries `rules.db` for `deprecated_patterns` with `severity='BLOCK'` and creates `DO NOT USE {deprecated_name}. Use {replacement_name}.` entries. [C-21 L115-L122]
- **F-C21-017 [E]** Rule-derived constraints append to TOML overrides, de-duplicate on `deprecated_name`, and silently fall through if `rules.db` is unavailable. [C-21 L115-L122]
- **F-C21-018 [E]** The SWE-bench seed corpus contains `trigger_task.toml`, `task_01.toml`…`task_10.toml`, and one-paragraph `README.txt`; the 11 TOMLs use `id`, `description`, `expected_output`, `timeout_s`, `stage`, and `is_canary`. [C-21 L124-L131]
- **F-C21-019 [E]** The trigger task is a known-answer canary gating `StageReport.trigger_task_passed`; the ten tasks use SWE-bench Lite problem statements under MIT terms and include no solution patches. [C-21 L124-L131]
- **F-C21-020 [E]** Pre-tool block failures carry `T1_BLOCK`, `T2_BLOCK`, or `PRE_TOOL_BLOCK` according to the responsible hook so `record_mistake_on_failure` writes a categorized `MistakeEntry`; the RPC response is updated only if it exposes this tier. [C-21 L133-L140]

#### Reachability, observability, determinism, and acceptance

- **F-C21-021 [E]** Produced surfaces include the CI step, hook installer/behavior, dependency-audit mapper/background trigger, closed enum, rules-derived constraints, 11-TOML corpus, and block-tier discriminant. [C-21 L142-L152]
- **F-C21-022 [E]** Integration consumes the anti-stale exit contract, replaces both locked/known string lists, populates `FullPipelineOutcome`, hooks promotion/DIVERGING state, invokes the steering script/rules query, and feeds `record_mistake_on_failure`. [C-21 L153-L161]
- **F-C21-023 [E]** Operational reachability is a GitHub Actions PR, local `git commit`, Layer1A `DIVERGING`, deprecated-pattern promotion, and the task-runner failure path. [C-21 L162-L165]
- **F-C21-024 [E]** Stub bans require exactly 12 enum variants, hook exit 1, real audit data, and nonblocking audit launch. [C-21 L166-L170]
- **F-C21-025 [E]** Promotion observability requires the steering script to start and `modern_api.npz` mtime to change; acceptance tightens this to within 60 seconds and requires the refresh event. [C-21 L171-L179, L224-L227]
- **F-C21-026 [E]** Locked-file commit observability is the exact blocking message and exit 1; closed-parameter observability is a compile error. [C-21 L171-L177, L228-L231]
- **F-C21-027 [E]** DIVERGING observability requires `dep_audit_latest.toml` and nonempty `stale_api_catches` when stale dependencies exist. [C-21 L178-L179, L232-L233]
- **F-C21-028 [E]** Ten deterministic checks cover CI presence, all enum variants, absence of raw string construction, installer, audit functions, 11 fixtures, rules-query wiring, clean orchestrator build, and block discriminants. [C-21 L181-L222]
- **F-C21-029 [E]** Acceptance requires the anti-stale step to execute successfully in an actual PR and a scoped `parameter_key` test suite to pass. [C-21 L234-L235]
- **F-C21-030 [E]** Chunk C acceptance is conditional on prior approval and then requires zero production `.unwrap()` sites in `deterministic_auditor.rs` plus passing tests. [C-21 L236-L237]
- **F-C21-031 [E]** The file inventory explicitly marks deterministic-auditor/DKG edits conditional and creates `C-21-decisions.md`; no completion claims are checked in this source. [C-21 L239-L259]

#### Ambiguities, conflicts, and bounded inference

- **F-C21-032 [A]** “STRICT mode” is required, but the specified command shows no environment variable or CLI switch; the source does not identify how strict mode is activated. [C-21 L54-L57]
- **F-C21-033 [A]** The sentinel write/update lifecycle after a spawned audit, behavior after spawn failure, and cross-process race handling are not specified. [C-21 L92-L97]
- **F-C21-034 [A]** Silent `rules.db` fallthrough protects availability but supplies no event/metric, so operators cannot distinguish “no blocked patterns” from “database unavailable” using this contract alone. [C-21 L115-L122]
- **F-C21-035 [I]** The pre-commit protection is opt-in per clone because only an installer is committed; CI anti-stale enforcement is the repository-wide backstop. [C-21 L54-L58, L66-L74]

### C-25 — Compiled Pipeline and Template Crystallizer

#### Status, dependencies, accounting, background lifecycle, and schemas

- **F-C25-001 [E]** C-25 is an unchecked forward plan implementing compiled-pipeline Designs 1–3: compiled templates/execution, background task management, and cumulative token/cache accounting. [C-25 L1-L10, L238-L272]
- **F-C25-002 [E]** It requires C-9’s autonomous gate and `gate_decision` observation loop, is declared parallel-safe with C-21–C-24 and C-26, and estimates two to three sessions. [C-25 L10-L13]
- **F-C25-003 [E]** `CumulativeTokenTracker` has six `u64` fields: `prompt_tokens_total`, `completion_tokens_total`, `cache_read_tokens`, `cache_write_tokens`, `last_response_prompt`, and `last_response_completion`; it derives debug/clone/default/serde support. [C-25 L46-L54]
- **F-C25-004 [E]** `record_response(&ProviderUsage)` accumulates usage, and `recalibrate_budget(&mut ModelCapabilityProfile)` sets `attention_ceiling = max(4096, last_response_prompt + 2048)`. [C-25 L46-L54]
- **F-C25-005 [E]** Before adding a usage type, implementation must reuse an existing `ProviderUsage` if present; required usage fields are `input_tokens`, `output_tokens`, `cache_read_tokens`, and `cache_write_tokens`, all `u64`. [C-25 L52-L54]
- **F-C25-006 [E]** Cloud mode shares the tracker as `Arc<Mutex<CumulativeTokenTracker>>`, records after every `SpendAccumulator::record()`, and recalibrates before each outbound request without changing billing. [C-25 L56-L59]
- **F-C25-007 [E]** `BackgroundTaskStarted` carries `task_id`, `command_summary`, `soft_timeout_secs`, and `hard_timeout_secs`; `BackgroundTaskComplete` carries `task_id`, `exit_status`, `duration_secs`, `auto_detached`, and `timed_out`. [C-25 L61-L64]
- **F-C25-008 [E]** `BackgroundTaskManager` owns `active_tasks: HashMap<String, BackgroundTaskHandle>`, `max_concurrent: u8 = 3`, an SSB sender, and `poll_interval_secs: u32`. [C-25 L66-L68]
- **F-C25-009 [E]** `BackgroundTaskHandle` stores `task_id`, `spawned_at: Instant`, soft/hard timeouts, `JoinHandle<i32>`, and `auto_detached`. [C-25 L68-L70]
- **F-C25-010 [E]** `spawn_task()` emits `BackgroundTaskStarted` and returns a task ID; `poll_completed()` harvests finished handles and emits `BackgroundTaskComplete`. [C-25 L70-L72]
- **F-C25-011 [E]** `enforce_timeouts()` drops/detaches at the soft boundary with `auto_detached=true` and aborts at the hard boundary; `gated_spawn()` refuses when `autonomous_mode=false`. [C-25 L71-L74]
- **F-C25-012 [E]** Background polling backs off across 5, 10, then 30 seconds by doubling on idle cycles. [C-25 L74-L75]
- **F-C25-013 [E]** Lowering child-process priority is optional and may be skipped if neither an existing `nix` nor `winapi` dependency supports it; the choice must be documented. [C-25 L74-L76]

#### Compiled pipeline data, matching, persistence, execution, and CLI surfaces

- **F-C25-014 [E]** `TemplateVariable` has `name`, `region_start`, `region_end`, and `variable_type`; `VariableType` is `ConceptClass`, `TaskDescription`, `CodeTarget`, or `Other(String)`. [C-25 L78-L81]
- **F-C25-015 [E]** `CompiledStep` has `step_index: u8`, `template_prompt`, `variables`, `variable_region_ratio: f32`, and `l1_tool_only: bool`. [C-25 L81-L82]
- **F-C25-016 [E]** `CompiledPipeline` has UUID ID, concept class, `plan_skeleton_hash: u64`, steps, create/last-use times, `use_count: u32`, `first_try_success_rate: f32`, and `compilation_eligible: bool`. [C-25 L83-L86]
- **F-C25-017 [E]** A pipeline is eligible only when every step is L1-tool-only, maximum variable-region ratio is strictly below 0.3, and use count is at least three. [C-25 L84-L87]
- **F-C25-018 [E]** `TemplateId(Uuid)` and `PipelineOutcomeId(Uuid)` are newtypes; compiled types derive debug/clone/serde and persist through `rmp-serde`, after verifying both dependencies already exist. [C-25 L87-L89]
- **F-C25-019 [E]** `TemplateCrystallizer` keeps `observation_log: HashMap<(String,u64), Vec<ExecutionOutcome>>`, threshold 3, and an SSB sender. [C-25 L91-L93]
- **F-C25-020 [E]** `ExecutionOutcome` records success, concept class, skeleton hash, optional pipeline UUID, token count, and timestamp. [C-25 L93-L94]
- **F-C25-021 [E]** `record_execution()` appends to the concept/hash bucket; `check_crystallization()` returns a compiled pipeline only when the last N outcomes are consecutive successes and the candidate is compilation-eligible. [C-25 L94-L98]
- **F-C25-022 [E]** Crystallization emits `template_crystallized` with concept class, skeleton hash, pipeline ID, and `threshold_n`. [C-25 L96-L98]
- **F-C25-023 [E]** `CompiledExecutor` owns a shared read/write pipeline cache, consecutive-failure counts, confidence threshold 0.7, and an SSB sender. [C-25 L100-L103]
- **F-C25-024 [E]** `try_execute()` returns `Err(CacheMiss)` for an absent `(concept_class, plan_hash)` key. [C-25 L100-L104]
- **F-C25-025 [E]** Two consecutive failures or confidence below 0.7 emits `decompile_triggered` and returns `Err(Decompile)`; the caller falls back to LLM decomposition. [C-25 L103-L106]
- **F-C25-026 [E]** `CompiledPipelineStore` keeps a cache path, keyed entries, and default TTL `604800` seconds (seven days). [C-25 L108-L113]
- **F-C25-027 [E]** Disk load decodes MessagePack; corrupt input warns, returns an empty store, and never panics. [C-25 L108-L112]
- **F-C25-028 [E]** Persistence uses an atomic temporary-file-plus-rename sequence; the `.elai/cache/` directory is created if absent. [C-25 L111-L113]
- **F-C25-029 [E]** Expiry removes entries whose `now - last_used_at` exceeds TTL, and `upsert()` inserts/updates before running eviction. [C-25 L112-L113]
- **F-C25-030 [E]** `classify_task_for_template(&task)` heuristically returns `(concept_class, plan_skeleton_hash)` or `None`; it uses SeaHash only if already present, otherwise `DefaultHasher` with the choice documented. [C-25 L115-L118]
- **F-C25-031 [E]** Pipeline factory selection tries a compiled template first; success emits `plan_skeleton` with `source=Template`, while decompile falls through and the existing generated path emits `source=Llm`. [C-25 L115-L121]
- **F-C25-032 [E]** `PlanSkeletonSource { Template, Llm }` becomes a required field of `PlanSkeletonPayload`; omission is a wiring violation. [C-25 L119-L121, L178-L183]
- **F-C25-033 [E]** `ResearchTreeConfig` gains optional shared crystallizer/store handles; each completed baby step records/checks, then an emitted candidate is upserted and persisted. [C-25 L123-L127]
- **F-C25-034 [E]** CLI adds `SlashCommand::Pipeline(PipelineSubcommand)` with `List` and `Reset`. [C-25 L129-L132]
- **F-C25-035 [E]** `/pipeline list` prints concept class, skeleton hash, use count, first-try success rate, and creation time for every stored entry. [C-25 L129-L132]
- **F-C25-036 [E]** `/pipeline reset` clears the in-memory store and deletes the MessagePack file. [C-25 L129-L132]

#### Reachability, observability, tests, and closure gates

- **F-C25-037 [E]** Token tracking is reached from cloud response round trips; crystallization from ResearchTree post-step; compiled execution from pipeline factory entry; background management from autonomous Bash; and cache administration from CLI. [C-25 L162-L177]
- **F-C25-038 [E]** Stub bans require the 4,096 budget clamp, explicit cache miss, autonomous-only spawning, corrupt-cache recovery, and a populated plan-skeleton source. [C-25 L178-L183]
- **F-C25-039 [E]** The primary template smoke runs the same two-method-counter task four times: runs 1–3 use LLM skeletons, run 3 crystallizes at threshold three, and run 4 uses the template. [C-25 L134-L138, L184-L191]
- **F-C25-040 [E]** Run 4 must use fewer tokens than the mean of runs 1–3 and expose positive `cache_read_tokens` in `CloudSpendPayload`. [C-25 L134-L141, L261-L264]
- **F-C25-041 [E]** The cache smoke requires nonempty `.elai/cache/compiled_pipelines.msgpack` and at least one `/pipeline list` entry. [C-25 L184-L191, L261-L271]
- **F-C25-042 [E]** A long autonomous Bash smoke must emit started/complete events, cross the soft timeout, and complete with `auto_detached=true`. [C-25 L139-L141, L265-L266]
- **F-C25-043 [E]** Ten deterministic check groups verify accounting, events, manager functions, compiled types/store/TTL, try-template wiring, source schema, CLI commands, workspace/scoped tests, and the cache artifact. [C-25 L193-L236]
- **F-C25-044 [E]** Acceptance requires at least eight new test functions across compiled-pipeline, background-task, and cumulative-token scopes, plus all pipeline contract tests. [C-25 L267-L270]
- **F-C25-045 [E]** `C-25-decisions.md` must justify the +2,048 attention margin, two-failure decompile threshold, and seven-day TTL. [C-25 L143-L145, L272]
- **F-C25-046 [E]** If C-9 has not shipped, it is explicitly the true blocker for fully verifying the crystallization smoke even if implementation work proceeds. [C-25 L297-L302]

#### Ambiguities, conflicts, and bounded inference

- **F-C25-047 [C]** Soft timeout is described as dropping the join handle (which detaches execution) while the smoke expects a `BackgroundTaskComplete` record for the detached task; after dropping the only handle, the stated manager has no specified mechanism to observe eventual exit or enforce the later hard timeout. [C-25 L66-L75, L139-L141]
- **F-C25-048 [A]** The source does not define confidence computation, initial value, or update rules for the 0.7 decompile threshold. [C-25 L100-L106]
- **F-C25-049 [A]** The concept-class heuristic and plan-skeleton canonicalization/hash stability are intentionally unspecified beyond a hasher preference. [C-25 L115-L118]
- **F-C25-050 [A]** Atomic rename gives crash-safe replacement on one filesystem, but the source does not specify fsync/durability, cross-process locking, or concurrent writer conflict handling. [C-25 L108-L113]
- **F-C25-051 [C]** C-25 calls C-26 “parallel-safe” while C-26 explicitly makes C-25 a prerequisite for its measurement smoke; code edits may be orthogonal, but completion ordering is not parallel. [C-25 L11-L13; C-26 L12-L15]
- **F-C25-052 [I]** All C-25 acceptance boxes are unchecked, so values and behaviors are requirements rather than shipped observations. [C-25 L238-L272]

### C-26 — Constrained Decoding, Retry, and Futility Primitives

#### Status, dependencies, decoding schemas, defaults, and provider integration

- **F-C26-001 [E]** C-26 is an unchecked forward plan closing token-efficiency and update-order gaps with provider decoding configuration, retry strategy, two futility paths, provider wiring, fixtures/measurements, grammars, CRANE, and role defaults. [C-26 L1-L15, L266-L293]
- **F-C26-002 [E]** It requires C-10 types for similarity/persona logic and C-25 token tracking for smoke measurements; it is declared parallel-safe with C-21–C-24. [C-26 L12-L15]
- **F-C26-003 [E]** `DecodingMode` has four variants: `JsonSchema { schema_json: String }`, `Grammar { ebnf_path: PathBuf, crane_adaptive: bool }`, `Regex { pattern: String }`, and `FreeText`, with debug/clone/equality/serde. [C-26 L44-L47]
- **F-C26-004 [E]** `ConstrainedDecodingConfig` has `mode`, `max_retries_on_invalid: u8 = 3`, and `strict: bool = true`; its intended default mode is `FreeText`. [C-26 L48-L50]
- **F-C26-005 [E]** `crane_adaptive_eligible()` returns true only for `Grammar` with `crane_adaptive=true`. [C-26 L51-L52]
- **F-C26-006 [E]** A local `RoleDecoding` enum avoids a providers→agent circular dependency. [C-26 L53-L57]
- **F-C26-007 [E]** Five role defaults are fixed: designer `FreeText`, executor `Grammar(function_signature.ebnf, crane_adaptive=true)`, critic `JsonSchema`, auditor `JsonSchema`, researcher `FreeText`. [C-26 L53-L57]
- **F-C26-008 [E]** `RetryStrategy` has temperature schedule default `[0.0, 0.2, 0.5]`, same-error threshold 3, and `decompose_on_failure=false`, with debug/clone/equality/serde. [C-26 L60-L64]
- **F-C26-009 [E]** `BubbleUpPolicy` gains `retry_strategy: RetryStrategy`, and every literal construction is migrated with struct-update syntax without changing existing retry-count logic. [C-26 L64-L70]
- **F-C26-010 [E]** `temperature_for_attempt()` saturates at the final schedule element and returns 0.0 rather than panicking for an empty schedule. [C-26 L67-L69]
- **F-C26-011 [E]** For each baby step, same-error futility stores the last N `FailureRecord.reason` strings in a capacity-N `VecDeque`; N identical strings emits `futility_detected { error_class, consecutive_count, level="baby_step" }` and returns `BubbleUpAction::Escalate`. [C-26 L72-L77]
- **F-C26-012 [E]** Output-stagnation futility computes Jaccard similarity through `GoalDriftCanary::compute(prev,curr)`; score strictly greater than 0.90 emits `futility_output_stagnation { similarity, threshold:0.90, level }`. [C-26 L78-L81]
- **F-C26-013 [E]** Output stagnation updates `navigator.suggested_persona` using `suggest_persona_for_pattern(StagnationPattern::NoDrift)`. [C-26 L78-L82]
- **F-C26-014 [E]** New futility code must contain no `.unwrap()`; if C-10’s module is absent, conditional-use blocks compile the C-10-dependent paths to a no-op. [C-26 L81-L84, L195-L200]
- **F-C26-015 [E]** Sidecar outbound requests accept optional `ConstrainedDecodingConfig`; JSON Schema writes `response_format={type:"json_object",schema:...}`, Grammar writes `grammar_path`, and FreeText adds nothing. [C-26 L86-L94]
- **F-C26-016 [E]** A sidecar `constrained_decoding_unavailable` response logs `tracing::warn!` and retries/falls back as FreeText without panic; silent swallowing is banned. [C-26 L91-L93, L195-L200]
- **F-C26-017 [E]** Cloud `JsonSchema` is encoded as an Anthropic tool named `structured_output` with `input_schema`, plus forced `tool_choice` for that tool. [C-26 L96-L104]
- **F-C26-018 [E]** Cloud Grammar and FreeText make no request changes; Grammar is explicitly sidecar-only. [C-26 L99-L104]
- **F-C26-019 [E]** `ModelFeatures.supports_grammar` profiles are a consumed patch surface for Grammar-mode availability. [C-26 L181-L188]

#### Benchmark assets, grammars, tests, events, and observability

- **F-C26-020 [E]** The model-capability benchmark must locate and fix its actual fixture-path bug rather than assume a path from the report. [C-26 L106-L114]
- **F-C26-021 [E]** It adds valid `fixtures/templates.json` with three template entries and `fixtures/padding.txt` with 100 lines. [C-26 L106-L118]
- **F-C26-022 [E]** The benchmark is rerun, and actual nonzero measurements 3/4/5 replace zeros in `model_baseline.toml`; fabricated floats are forbidden. [C-26 L112-L118, L195-L202]
- **F-C26-023 [E]** `import_block.ebnf` covers a common subset of Rust `use` and Python import/from-import forms. [C-26 L120-L124]
- **F-C26-024 [E]** `function_signature.ebnf` covers Rust function signatures and Python `def` signature lines. [C-26 L124-L126]
- **F-C26-025 [E]** `error_type.ebnf` covers CamelCase identifiers ending in `Error`, `Err`, or `Exception`, including qualified paths such as `std::io::Error`. [C-26 L126-L129]
- **F-C26-026 [E]** Grammar files must be nonempty and resolve `ebnf_path` according to a documented cwd-relative versus embedded strategy. [C-26 L128-L132, L195-L202]
- **F-C26-027 [E]** `constrained_decoding_contract.rs` defines 12 named cases covering mode defaults/serialization, CRANE eligibility, retry temperature, policy inclusion, both futility paths, and three role defaults. [C-26 L134-L149]
- **F-C26-028 [E]** `benchmark_fixture_contract.rs` defines three cases for valid templates JSON, nonempty padding, and nonzero measurements 3/4/5. [C-26 L150-L155]
- **F-C26-029 [E]** Produced event types explicitly include `futility_detected` and `futility_output_stagnation`. [C-26 L162-L179]
- **F-C26-030 [E]** The observable happy path activates critic JSON Schema, expects `constrained_decoding_used` with `mode=JsonSchema`, `validation=Pass`, no retry, and no futility event. [C-26 L204-L209]
- **F-C26-031 [E]** The adversarial observable uses an always-rejecting regex and expects after three equal failures `futility_detected(error_class=constrained_decode_reject,count=3)` plus a persona-switch row. [C-26 L209-L213, L285-L290]
- **F-C26-032 [E]** The benchmark observable requires nonzero deltas for measurements 3/4/5 and the updated baseline on disk. [C-26 L212-L213, L291-L292]
- **F-C26-033 [E]** Eleven deterministic check groups verify types/default tables, retry/policy, events, provider wiring, assets, nonzero baselines, 15 tests, and clean provider/orchestrator/contracts compilation. [C-26 L215-L264]
- **F-C26-034 [E]** Acceptance calls for all 15 contract cases and the three-part cloud/regex/benchmark smoke to be proven. [C-26 L266-L293]
- **F-C26-035 [E]** `C-26-decisions.md` records local-role typing, grammar path resolution, temperature rationale, warned FreeText fallback, and fixture placeholder rationale. [C-26 L156-L160, L293]
- **F-C26-036 [E]** C-10 absence may ignore the similarity contract, but the source’s final note says the persona-switch smoke cannot close until C-10 ships. [C-26 L143-L145, L313-L317]

#### Ambiguities, conflicts, and bounded inference

- **F-C26-037 [C]** Ordinary Rust `#[derive(Default)]` would yield numeric/bool defaults of 0/false, conflicting with the explicit `max_retries_on_invalid=3` and `strict=true`; the intended contract requires a manual/custom default even though “derive Default” is requested. [C-26 L48-L50]
- **F-C26-038 [A]** `strict` is named and defaulted but its validation, retry, and fallback semantics are never defined. [C-26 L48-L50]
- **F-C26-039 [A]** `Regex` is a declared mode and drives the adversarial smoke, but neither sidecar request encoding nor cloud behavior for Regex is specified. [C-26 L44-L47, L86-L104, L204-L213]
- **F-C26-040 [C]** The same-error path explicitly escalates but does not switch persona, while the always-failing-regex acceptance expects a persona switch from three equal errors; persona switching is only specified for the separate >0.90 output-similarity path. [C-26 L72-L82, L209-L213, L285-L290]
- **F-C26-041 [A]** `constrained_decoding_used` and a persona-switch SSB row are required observables, but neither is added to the produced-event list or assigned a payload schema/action-type task. [C-26 L162-L179, L204-L213]
- **F-C26-042 [A]** Same-error tracking stores full reason strings while the event calls the value `error_class`; normalization from reason text to class is not defined. [C-26 L73-L77]
- **F-C26-043 [A]** The source does not define schema-validation mechanics, handling of invalid `schema_json`, retry temperature application to provider calls, or whether the three-retry count includes the first attempt. [C-26 L44-L70, L86-L104]
- **F-C26-044 [C]** C-10 is listed as a prerequisite, yet missing C-10 is also supported through a no-op/ignored test; implementation can compile without it, but the named smoke and full acceptance cannot close. [C-26 L12-L14, L81-L84, L143-L145, L313-L317]
- **F-C26-045 [I]** All acceptance boxes are unchecked, so per-role defaults and benchmark values are intended rather than evidenced as shipped. [C-26 L266-L293]

### C-38 — PageRank Production Contract and Aider Spec Accuracy Fix

#### Status, source-truth workflow, production behavior, events, and evidence

- **F-C38-001 [E]** C-38 is an unchecked plan with two coupled objectives: prove PageRank runs on the production query path and correct a docs-truth error that called Aider’s repo map file-level. [C-38 L1-L7, L170-L185]
- **F-C38-002 [E]** It treats B-1 DKG runtime activation as shipped, is parallel-safe with C-35–C-37/C-39/C-40, and is not parallel-safe with another plan mutating `pipeline.rs` gates. [C-38 L9-L11]
- **F-C38-003 [E]** The source records prior-oracle claims that Aider uses symbol-level NetworkX nodes, definition-without-reference self-loop weight 0.1, and a 50× chat-context multiplier, but requires a fresh fetch rather than trusting that prior read. [C-38 L13-L23, L61-L67]
- **F-C38-004 [E]** The fresh comparison extracts graph node/edge types, weighting, self-loop policy, top-k selection, and chat-context multiplier from current `get_ranked_tags`/`get_ranked_tags_map` behavior. [C-38 L61-L67]
- **F-C38-005 [E]** `aider-elai-pagerank-diff.md` is a source-of-truth table with columns `attribute`, `Aider behavior`, `ELAI behavior`, `divergence-intent`, and `divergence-impact`. [C-38 L68-L71]
- **F-C38-006 [E]** `DETAILED_SUMMARY_01` section 10 must replace “file-level,” re-audit all three claimed divergences, and cite the comparison artifact. [C-38 L39-L50, L73-L77]
- **F-C38-007 [E]** The corrected section carries a footnote with the fresh-fetch date and exact comparison-artifact path. [C-38 L73-L77, L131-L132]
- **F-C38-008 [E]** The current session’s `mistakes.md` gains `stale-external-reference`: external dependency behavior was specified without fresh-fetch verification. [C-38 L79-L81, L166-L167]
- **F-C38-009 [E]** The test fixture contains at least 50 synthetic code files and one deliberate hub imported by 20 others; the hub must appear in PageRank’s top three. [C-38 L83-L86, L154-L156]
- **F-C38-010 [E]** The contract invokes the production prompt/query path rather than `compute_repo_map` directly and observes `pagerank_run` through the real SSB side effect, not a mock. [C-38 L83-L90, L127-L129]
- **F-C38-011 [E]** Starting with absent `.elai/dkg.db` must transparently rebuild rather than silently skip and emit both `dkg_rebuild_started` and `dkg_rebuild_complete`. [C-38 L90-L92]
- **F-C38-012 [E]** Identical fixture input produces deterministic top-k output across ten repeated production-path invocations. [C-38 L93-L94]
- **F-C38-013 [E]** `pagerank_run` is reused if already present; rebuild actions are added only if absent, and a schema-version bump occurs only for genuinely new actions. [C-38 L96-L98]
- **F-C38-014 [E]** Each PageRank/rebuild event row carries `fixture_or_repo_id` and `elapsed_ms`. [C-38 L96-L98]
- **F-C38-015 [E]** Production code must ensure or rebuild DKG before PageRank; an existing silent `if dkg.db exists` skip blocks C-38 closure and proves B-1’s claimed fix did not land. [C-38 L100-L105, L127-L130]
- **F-C38-016 [E]** If the silent gate survives, a separate B-1 follow-up bug is filed and C-38 remains open rather than hiding the production defect inside docs/test work. [C-38 L100-L105, L184-L185]
- **F-C38-017 [E]** External-claim hygiene is also added to an apps-workflow rule such as `.claude/rules/token-discipline.md`. [C-38 L107-L110]
- **F-C38-018 [E]** The production behavior is reachable from `/explore` and from the dedicated cargo contract target. [C-38 L120-L126]
- **F-C38-019 [E]** The observable contract expects one `pagerank_run` per invocation and rebuild rows only when beginning without the DKG database. [C-38 L134-L139]
- **F-C38-020 [E]** Seven deterministic checks cover diff/doc artifacts, passing production contract, fixture size, event types, removal/replacement of silent gating, and the mistakes entry. [C-38 L141-L168]
- **F-C38-021 [E]** Closure acceptance independently requires fresh-fetch timestamp, full diff, corrected/cited summary, mistake record, all 5.a–5.d tests, 50+ files, top-three hub, real event, absent-DKG rebuild, and ten-run determinism. [C-38 L170-L185]
- **F-C38-022 [E]** On closure, B-1 and phase-1D verification reports are annotated with the new production-path lock. [C-38 L197-L202]
- **F-C38-023 [E]** Until the docs correction and contract proof land, external claims that ELAI improves on Aider are explicitly treated as unverified. [C-38 L204-L209]

#### Ambiguities, conflicts, and bounded inference

- **F-C38-024 [A]** The plan says `pagerank_run` may already exist and should be confirmed; therefore “add/extend pagerank_run” is not a settled schema change. [C-38 L96-L98]
- **F-C38-025 [A]** Event implementation is alternately located in `orchestrator/src/ssb.rs` and `orchestrator/src/ssb/schema.rs`, and contract-test paths alternate between `tests/contracts/...` and `crates/orchestrator/tests/contracts/...`; exact repository placement requires inspection. [C-38 L83-L84, L96-L98, L151-L152, L187-L195]
- **F-C38-026 [A]** The exact FSM states that “expect symbol-graph context,” top-k size beyond the top-three assertion, and PageRank tie-breaking are not specified. [C-38 L30-L37, L83-L94]
- **F-C38-027 [I]** The Aider algorithm details are plan-recorded evidence pending the mandated fresh fetch, not facts newly verified by this deep read. [C-38 L13-L23, L61-L67]
- **F-C38-028 [I]** All acceptance boxes are unchecked; B-1 is reported shipped but C-38’s defining production proof is not yet evidenced in this source. [C-38 L9-L20, L170-L185]

### C-46 — Uno-Orchestra Parsimonious Agent Routing

#### Status, policy, configuration, events, fallback, and testing

- **F-C46-001 [E]** C-46 is an unchecked plan for an optional selective-delegation dispatcher mode that avoids the full agent chain on simple work. [C-46 L1-L7, L116-L127]
- **F-C46-002 [E]** Eight `AgentRole` implementations are prerequisite; C-45 credit attribution is recommended but not a strict gate; C-46 is declared parallel-safe with C-35–C-45 and C-47–C-53. [C-46 L9-L11]
- **F-C46-003 [E]** The design is grounded through a required paper-ingestor pass over arXiv:2605.05007, producing `workflow/papers/uno-orchestra-interface.md`. [C-46 L13-L17, L43-L46]
- **F-C46-004 [E]** `TaskDifficultyClassifier` features are task-description length, code-file count, repo-map centrality of affected files, and embedding cosine to a fixed easy/hard reference set. [C-46 L47-L51]
- **F-C46-005 [E]** The classifier emits a normalized difficulty score in `[0.0,1.0]`. [C-46 L47-L51]
- **F-C46-006 [E]** `SelectiveDelegator` combines difficulty with C-45 per-role credit weights when those weights are available and returns the roles to spawn. [C-46 L53-L55]
- **F-C46-007 [E]** For `difficulty < 0.3`, the default role set is exactly Executor plus Auditor. [C-46 L55-L58]
- **F-C46-008 [E]** For `0.3 <= difficulty < 0.7`, the role set adds ContextCompiler and PromptEngineer. [C-46 L55-L58]
- **F-C46-009 [E]** For `difficulty >= 0.7`, the full chain is selected. [C-46 L55-L58]
- **F-C46-010 [E]** An Auditor “insufficient context” result emits `selective_delegation_escalated` and re-spawns the full chain, consuming one retry under the existing bubble-up policy. [C-46 L60-L63]
- **F-C46-011 [E]** `[agent.selective_delegation]` config contains `enabled: bool` default false, `classifier_model: string` default `"trigram_centrality"`, and `reference_set_path: string`. [C-46 L65-L67]
- **F-C46-012 [E]** `selective_delegation_decision` is emitted per task with difficulty and chosen role set; it must occur before any spawn so routing is auditable. [C-46 L69-L71, L100-L105]
- **F-C46-013 [E]** Additional events are `selective_delegation_escalated` and `selective_delegation_cost_saved`; the cost-saved metric must be nonnegative. [C-46 L69-L71, L102-L105]
- **F-C46-014 [E]** TDD requires stable classifier scores under a deterministic seed and exact low/mid/high policy sets. [C-46 L73-L77]
- **F-C46-015 [E]** TDD also simulates auditor failure to prove full-chain re-spawn/events and compares spawn counts against a full-chain baseline for cost saved. [C-46 L78-L81]
- **F-C46-016 [E]** A dedicated `selective_delegation_contract.rs` and scoped agent test invocation are required. [C-46 L87-L88, L107-L114]
- **F-C46-017 [E]** The 100-task SWE-bench Verified pilot is deferred to RunPod; full-chain and selective arms target solve-rate delta within ±2 percentage points and cost reduction at least 30%. [C-46 L83-L85]
- **F-C46-018 [E]** The feature remains disabled by default and is flipped only after the pilot meets the target. [C-46 L124-L127]
- **F-C46-019 [E]** The planned file surface adds routing modules/tests/paper/pilot and modifies dispatcher, SSB schema, and `elai_cc.toml`. [C-46 L129-L140]

#### Ambiguities, conflicts, and bounded inference

- **F-C46-020 [C]** The context describes a current six-role “full chain,” while prerequisites require eight roles and the medium policy names two roles not in that six-role list; the authoritative full-chain membership/order is not reconciled. [C-46 L9-L10, L21-L24, L53-L58]
- **F-C46-021 [A]** Score calibration, feature normalization/weights, boundary behavior for NaN/out-of-range scores, and the fixed reference-set schema are not defined. [C-46 L47-L58]
- **F-C46-022 [A]** `reference_set_path` has no default, required/optional rule, missing-file fallback, or privacy/update policy. [C-46 L65-L67]
- **F-C46-023 [A]** The paper-backed cost-savings paragraph says “typically” 40–60% on an easy half before the plan’s own pilot; it is a motivating estimate, not an acceptance result. [C-46 L33-L35, L83-L85]
- **F-C46-024 [I]** All acceptance boxes are unchecked and the decisive pilot is deferred, so production enablement is intentionally not authorized by this source. [C-46 L83-L85, L116-L127]

### C-47 — Case-Based Calibration of Tool-Use Telemetry

#### Status, storage schema, retrieval, hints, privacy surface, and tests

- **F-C47-001 [E]** C-47 is an unchecked plan for a per-project tool-call memory loop recording call, response, and post-call audit/FSM outcome, then consulting prior cases before future MCP calls. [C-47 L1-L7, L105-L116]
- **F-C47-002 [E]** C-35 Tool Attention, C-42 zero-trust fuzzing, and Phase 4K MCP bridge must already be shipped; it is sequence-after C-35/C-42 and otherwise parallel-safe with C-36–C-46. [C-47 L9-L11]
- **F-C47-003 [E]** A required paper-ingestor pass over arXiv:2605.15041 produces `workflow/papers/case-based-calibration.md`. [C-47 L13-L16, L43-L46]
- **F-C47-004 [E]** The case store is filesystem-side, per project, and estimated around 100 KB; the design is deterministic retrieval, explicitly not fine-tuning. [C-47 L25-L35]
- **F-C47-005 [E]** `CaseBase` may use sled or SQLite, with sled preferred; its schema is `tool_name | timestamp | call_shape_hash | call_shape | response_kind | post_call_audit_ok | session_id`. [C-47 L47-L51]
- **F-C47-006 [E]** Storage is capped at 100 entries per tool and evicts the oldest entry. [C-47 L47-L51, L91-L94]
- **F-C47-007 [E]** `CaseRetriever` returns top-K prior cases using Jaccard similarity over argument-key sets plus Levenshtein similarity over string values. [C-47 L53-L56]
- **F-C47-008 [E]** Retrieval returns the nearest case plus `NEAR_MATCH` or `NO_MATCH`, and similarity ordering must be deterministic. [C-47 L53-L56, L66-L70]
- **F-C47-009 [E]** A hint is injected into the model’s next prompt only when a near match previously succeeded and exposes a known shape correction such as a missing/renamed key. [C-47 L58-L61]
- **F-C47-010 [E]** Config-based hint suppression is enforced at the event/emit point, not after a hint has been produced. [C-47 L66-L72, L91-L95]
- **F-C47-011 [E]** Events are `case_base_consulted`, `case_base_hint_emitted`, and `case_base_call_logged`; every event must carry non-null `tool_name`. [C-47 L63-L64, L105-L113]
- **F-C47-012 [E]** `case_base_call_logged` must emit only after the tool returns, preserving the response/audit trail. [C-47 L91-L93]
- **F-C47-013 [E]** Consultation occurs before every MCP dispatch when `[tool_use.case_base.enabled]` is active. [C-47 L78-L90]
- **F-C47-014 [E]** Five TDD cases prove persistent round-trip, deterministic ordering, rename-hint emission, oldest-first cap eviction, and disabled-config suppression. [C-47 L66-L72]
- **F-C47-015 [E]** A dedicated `case_base_contract.rs` and scoped plugin tests are required. [C-47 L73-L76, L96-L103]
- **F-C47-016 [E]** The pilot replays 50 existing ELAI sessions and must measure at least a 10% reduction in repeated tool-call errors. [C-47 L73-L75, L114-L115]
- **F-C47-017 [E]** Planned file changes add case store/tests/paper/pilot and modify MCP server pre-dispatch, SSB schema, and `[tool_use.case_base]` config. [C-47 L118-L127]

#### Ambiguities, conflicts, and bounded inference

- **F-C47-018 [C]** Task text permits sled or SQLite, but acceptance specifically requires persistence “via sled”; backend freedom and the done gate do not agree literally. [C-47 L47-L51 versus L107-L109]
- **F-C47-019 [C]** The store is called “LRU,” but eviction is explicitly oldest-by-timestamp rather than access-recency; the deterministic timestamp policy is the only fully specified behavior. [C-47 L47-L51, L66-L72, L91-L94]
- **F-C47-020 [A]** The context says the index holds the last N successful calls, while the schema and logging rule retain response/audit fields for every returned call; whether failed calls occupy the 100-entry cap is unspecified. [C-47 L25-L32, L47-L51, L91-L93]
- **F-C47-021 [A]** Top-K, the NEAR_MATCH threshold, Jaccard/Levenshtein weighting, tie-break order, canonical call-shape hashing, and non-string-value treatment are not defined. [C-47 L53-L61]
- **F-C47-022 [A]** The enabled flag’s default is not stated. [C-47 L88-L90, L125-L127]
- **F-C47-023 [A]** Stored call shapes, string values, and session IDs create a privacy/security surface, but no redaction, encryption, access control, retention duration, cross-project isolation test, or user-deletion workflow is specified. [C-47 L25-L35, L47-L51]
- **F-C47-024 [I]** All acceptance boxes are unchecked; the claimed repeat-error improvement remains a pilot target rather than observed evidence. [C-47 L105-L116]

### C-52 — Dafny Formal Verification Spike (Research Grade)

#### Status, scope, conditional lifecycle, integration, and evidence

- **F-C52-001 [E]** C-52 is explicitly research-grade, optional, opt-in, and narrow; closure may be either a successful three-module spike or a documented production infeasibility. [C-52 L1-L11, L38-L39]
- **F-C52-002 [E]** Phase 4F code completeness is required, C-50 contract coding is preferred but not strict, and a Dafny toolchain on the developer host is required for the success path. [C-52 L8-L10]
- **F-C52-003 [E]** It is declared parallel-safe with C-35–C-51 and C-53. [C-52 L10-L11]
- **F-C52-004 [E]** The motivating paper claim is a prior-oracle report of 90.91% Dafny verification success for Gemma 4-31B on NL2VC-60; a paper-ingestor task must produce `nl2vc-dafny-interface.md`. [C-52 L3-L6, L13-L16, L49-L52]
- **F-C52-005 [E]** The formal layer sits above mutation/metamorphic/wiring evidence and targets mathematical pre/postcondition proofs only where practical. [C-52 L18-L23]
- **F-C52-006 [E]** Candidate modules are FSM transition validation, C-48 `MemoryView` trust-tier filtering, and C-28 mutation-score aggregation. [C-52 L25-L31, L57-L60]
- **F-C52-007 [E]** File I/O, networking, and LLM dispatch are explicitly out of scope. [C-52 L33-L36]
- **F-C52-008 [E]** Each `.dfy` file is a specification mirroring claimed Rust pre/postconditions, not a one-to-one Rust source translation. [C-52 L62-L64]
- **F-C52-009 [E]** Toolchain installability is checked on Windows and Linux development hosts; if installation is nontrivial, concrete blockers are documented and work jumps to the infeasibility path. [C-52 L53-L55]
- **F-C52-010 [E]** The success path adds `formal/fsm_transition.dfy`, `formal/memory_view.dfy`, and `formal/mutation_aggregator.dfy`. [C-52 L91-L99]
- **F-C52-011 [E]** Feature-gated `VerificationHarness` exposes `cargo verify-dafny --module FSMTransition`, shells out to `dafny verify`, treats clean output as pass, and sends failure to SSB. [C-52 L66-L69]
- **F-C52-012 [E]** CI integration is optional/informational, never blocking, and runs when a PR touches one of the three spike modules. [C-52 L71-L72]
- **F-C52-013 [E]** If toolchain install, all three translations, or CI integration cannot land, `dafny-spike-infeasibility.md` with concrete blocker descriptions becomes the closure artifact. [C-52 L74-L77]
- **F-C52-014 [E]** Infeasibility cannot be a vague “too hard”; acceptable blockers include installation failure, a language-feature gap, or CI-runner incompatibility. [C-52 L109-L114]
- **F-C52-015 [E]** If only one of three spikes lands, a partial-success note is mandatory and full success must not be claimed. [C-52 L113-L114]
- **F-C52-016 [E]** Events are `dafny_verify_run`, `dafny_verify_pass`, and `dafny_verify_fail`. [C-52 L79-L80]
- **F-C52-017 [E]** TDD proves a nonzero Dafny exit surfaces `dafny_verify_fail` and all three known-good specifications verify cleanly. [C-52 L82-L86]
- **F-C52-018 [E]** Regardless of outcome, `.elai_cc/docs/formal-verification.md` documents the spike result. [C-52 L88-L89]
- **F-C52-019 [E]** Success is reachable from the cargo alias and optional CI; infeasibility is reachable through the explicit alternate artifact path. [C-52 L91-L108]
- **F-C52-020 [E]** Deterministic closure is disjunctive: either Dafny exists plus the FSM spec verifies, or an infeasibility file exists with at least one `blocker:` entry. [C-52 L116-L127]
- **F-C52-021 [E]** Success acceptance requires paper artifact, documented installability, all three specifications passing, reproducible harness, all three events, TDD, and informational CI. [C-52 L129-L140]
- **F-C52-022 [E]** Infeasibility acceptance requires at least one blocker, a documented revisit trigger, and C-52 closure as `DEFERRED` with an explicit re-evaluation condition. [C-52 L142-L146]
- **F-C52-023 [E]** The success file plan adds a new workspace crate, three Dafny files, harness tests, workflow, paper artifact, workspace membership, and SSB actions; the infeasibility plan adds the blocker and formal-verification docs. [C-52 L148-L161]
- **F-C52-024 [E]** A valid outcome explicitly leaves testing-trust as ELAI’s highest evidence tier if formal verification proves infeasible. [C-52 L163-L169]

#### Ambiguities, conflicts, and bounded inference

- **F-C52-025 [C]** Task 2 asks installation on both Windows and Linux, while success acceptance only requires at least one supported host; the minimum host-coverage gate is inconsistent. [C-52 L53-L55 versus L131-L134]
- **F-C52-026 [C]** Context says spike “3–5” modules, but task selection, produced files, tests, and acceptance all fix exactly three. [C-52 L33-L36 versus L57-L60, L93-L99, L134-L136]
- **F-C52-027 [A]** The plan verifies Dafny specifications but defines no refinement, translation-validation, or equivalence check proving the Rust implementations actually satisfy those specifications. [C-52 L62-L69]
- **F-C52-028 [A]** Events are listed among success-path symbols, but it is unclear whether an installability failure before the harness exists must emit any Dafny event. [C-52 L53-L55, L79-L80, L91-L102]
- **F-C52-029 [A]** The success file inventory omits `formal-verification.md` even though documentation is required for either outcome. [C-52 L88-L89, L148-L161]
- **F-C52-030 [I]** All acceptance boxes are unchecked, and a deferred infeasibility closure is first-class rather than evidence that verification shipped. [C-52 L129-L146]

### C-54 — SWE-bench Verified Run, Three Configurations (Flagship / RunPod Dependent)

#### Status, prerequisites, corpus, locked configurations, harness, and security

- **F-C54-001 [E]** C-54 is an unchecked, RunPod-dependent flagship plan to publish ELAI’s first SWE-bench Verified measurement; it explicitly rejects differentiation claims without measured evidence. [C-54 L1-L7, L202-L217]
- **F-C54-002 [E]** Prerequisites are C-21’s ten-task seed, shipped C-35, at least one C-37 re-evaluation run, shipped C-39/C-42, and an accessible RunPod 4090/A100/H100 instance. [C-54 L9-L24]
- **F-C54-003 [E]** C-44 Composer 2 is required for the named Config C; otherwise that arm is downgraded to Sonnet-only. C-51 is explicitly not required. [C-54 L19-L22]
- **F-C54-004 [E]** Benchmark work is parallel-safe with C-55–C-58 only as separate runs; configurations sharing one RunPod execute sequentially, and the production binary is locked against C-35–C-53 prompt-path edits at run start. [C-54 L26-L29]
- **F-C54-005 [E]** The current ten-task seed expands to the full 500-task SWE-bench Verified corpus under `benchmarks/swe_bench/verified/`. [C-54 L59-L72]
- **F-C54-006 [E]** `manifest.toml` pins the SWE-bench version SHA for reproducibility. [C-54 L69-L72]
- **F-C54-007 [E]** `SwebenchHarness` clones each target repository at its pre-issue SHA, presents issue+repository to ELAI, collects/applies the emitted patch, runs hidden tests, and reports pass/fail. [C-54 L74-L78]
- **F-C54-008 [E]** Harness execution reuses C-11 sandbox-completeness for task isolation. [C-54 L74-L78]
- **F-C54-009 [E]** Config A (`config-A-local-7B.toml`) uses backend `local_qwen_7b`, binding executor and auditor to the same local 7B model. [C-54 L80-L83]
- **F-C54-010 [E]** Config B (`config-B-local-7B-7B.toml`) uses two distinct local 7B instances for executor and auditor; one-GPU hosts run them sequentially. [C-54 L83-L85]
- **F-C54-011 [E]** Config C (`config-C-hybrid.toml`) uses `sonnet_46`, `composer2`, and `local_qwen_7b`, with Phase 4M default role routing and C-41 skill routing. [C-54 L85-L87]
- **F-C54-012 [E]** The three arms isolate infrastructure with one local model, the effect of a separate local auditor, and cloud-grade per-call quality under the same broader system. [C-54 L48-L57]

#### Run lifecycle, metrics, publication, events, recovery, and done gates

- **F-C54-013 [E]** Each configuration performs one complete sweep of all 500 tasks; a task is run once per configuration with no cherry-picking. [C-54 L89-L105]
- **F-C54-014 [E]** Per-task records are `solved`, `time_s`, `llm_calls_total`, `tokens_in`, `tokens_out`, `cost_usd`, post-solve `mutation_score`, `MR_violations`, and `wiring_contract_pass`. [C-54 L89-L94]
- **F-C54-015 [E]** Per-config aggregates are solve rate, mean cost, p50/p99 time, and mean mutation score. [C-54 L94-L95]
- **F-C54-016 [E]** Each arm emits `workflow/sessions/{id}/swe_bench_run_{config}/scorecard.json`. [C-54 L94-L95]
- **F-C54-017 [E]** Auto-generated methodology records the SWE-bench SHA, RunPod/GPU/RAM, full configuration snapshots, C-35 gating and C-37 decoding state, wall-clock/cost, and timeout/error versus honest-failure outliers. [C-54 L97-L105]
- **F-C54-018 [E]** Methodology is generated from machine-readable config/run metadata and may not contain a human-editable narrative section that can drift from the numbers. [C-54 L126-L133, L161-L164]
- **F-C54-019 [E]** Public `swe_bench_verified_results.md` presents the three solve rates and costs, links methodology, and states honest losses/caveats. [C-54 L107-L115]
- **F-C54-020 [E]** The publication compares Devin/OpenHands/mini-SWE-agent/Claude Code only with each competitor’s cited version, date, published number, and methodology. [C-54 L107-L115]
- **F-C54-021 [E]** Results must identify benchmark SHA, hardware, and total cost; a bare “ELAI scored X%” is explicitly incomplete. [C-54 L161-L168]
- **F-C54-022 [E]** The mandatory honesty statement declares single-attempt pass@1, version SHA/hardware/cost, no cherry-picking, fixed TOML hyperparameters, seeded Config-A reproducibility within ±0 tasks, and Config-C cloud stochasticity within ±1–3 tasks. [C-54 L116-L121, L161-L169]
- **F-C54-023 [E]** Events are `swe_bench_run_started`, `swe_bench_task_completed`, `swe_bench_scorecard_emitted`, and `swe_bench_methodology_locked`. [C-54 L123-L124]
- **F-C54-024 [E]** Five no-GPU TDD cases prove pre-issue checkout, a known-good hidden-test pass, deliberately bad patch failure, official scorecard conformance, and metadata-driven methodology generation. [C-54 L126-L133]
- **F-C54-025 [E]** The benchmark is excluded from normal CI because of cost and is launched manually as `elai benchmark swe-bench --config ... --runpod-id <id>`. [C-54 L135-L136]
- **F-C54-026 [E]** If RunPod eviction, network failure, or another error interrupts a sweep, its partial scorecard is discarded and the entire configuration restarts. [C-54 L138-L141]
- **F-C54-027 [E]** A publishable number must come from one completed sweep; partials may never be published. [C-54 L161-L166]
- **F-C54-028 [E]** Seven deterministic groups verify 500 files, pinned SHA, three config files, feature-gated harness build, four action types, three post-run scorecards, and methodology/results artifacts. [C-54 L171-L200]
- **F-C54-029 [E]** Acceptance separately requires corpus/config/harness/sanity completion, all three 500-task scorecards, generated methodology, authored results with honesty statement, all four events, and no partial/cherry-picked publication. [C-54 L202-L217]
- **F-C54-030 [E]** C-54 consumes C-28 mutation and C-29 metamorphic metrics, all three backend tiers, C-11 sandboxing, and the SSB bus. [C-54 L143-L160]
- **F-C54-031 [E]** File output includes corpus/manifest/configs, task-runner harness/test, three scorecards, methodology, results, CLI subcommand, and SSB schema. [C-54 L219-L234]
- **F-C54-032 [E]** Closure annotates the C-21 verification report for the full corpus and unlocks C-55/C-56 plus eventual C-59 aggregation. [C-54 L236-L248]

#### Ambiguities, conflicts, and bounded inference

- **F-C54-033 [C]** The prerequisite permits downgrading Config C to Sonnet-only when C-44 is unavailable, but the objective, config schema, consumed backend list, and acceptance require Sonnet+Composer2+local; it is unclear whether a downgraded run may close C-54 as “three configs.” [C-54 L3-L5, L19-L20, L85-L87, L154-L157, L212-L213]
- **F-C54-034 [A]** Config A says executor and auditor both bind to local 7B, while Config B’s distinction is two separate instances; process/model-instance sharing and context isolation in Config A are not defined. [C-54 L80-L85]
- **F-C54-035 [A]** Hidden-test secrecy/access, credential handling for cloud backends/RunPod, artifact redaction, and whether repository network access stays enabled during execution are not specified beyond C-11 sandbox reuse. [C-54 L74-L78, L97-L105]
- **F-C54-036 [A]** Timeout values, failed-vs-errored scoring, percentile estimator, cost accounting source, and stochastic seeds/temperatures are recorded in methodology but not fixed by this plan. [C-54 L89-L105, L116-L121]
- **F-C54-037 [A]** “Scorecard JSON conforms to SWE-bench official format” is required, but the added ELAI-only fields and mapping into the official schema are not defined. [C-54 L89-L95, L126-L133]
- **F-C54-038 [I]** “Three-run protocol” means one full sweep for each of three configurations, not three repetitions of each task/configuration. [C-54 L89-L105]
- **F-C54-039 [I]** All acceptance boxes are unchecked and RunPod is a hard external gate; the source contains no published score, spend, or completed run. [C-54 L23-L24, L202-L217]

## Cross-source interaction ledger

These are synthesized interactions, not additional atomic source features; each points back to already-counted ledger items.

- **X-001 — shipped/planned overlap:** C-8 reports the A-RAG ONNX dot-product path shipped with norm-proxy fallback, while C-17 still plans “replace the norm proxy” and bans it entirely. Before implementing C-17, reconcile whether its task is stale, whether C-8’s fallback violates C-17’s ban, or whether C-17 intends a stronger no-fallback contract. [F-C8-028–F-C8-029; F-C17-013, F-C17-026]
- **X-002 — indexing lifecycle:** C-8’s nonfatal artifact hook, production indexing RPC, Cosmos tier, and SSB events add live producer behavior that C-17’s resume/watch/signal/indexing work must preserve across interruption and restart. [F-C8-014–F-C8-020, F-C8-033–F-C8-036; F-C17-007–F-C17-010, F-C17-027]
- **X-003 — clock/determinism dependency:** C-8’s shipped `ClockProvider`/`SeededClock` latency behavior is consistent with C-17 and C-21 depending on C-15’s entropy baseline; future timing/audit work must not reintroduce raw entropy into pipeline instrumentation. [F-C8-030–F-C8-032; F-C17-002; F-C21-001]
- **X-004 — seed corpus to flagship corpus:** C-21 creates the canary plus ten SWE-bench Lite TOMLs; C-54 explicitly consumes that seed and expands it to the pinned 500-task Verified corpus. C-54 therefore cannot treat the seed as its final benchmark set. [F-C21-018–F-C21-019; F-C54-002, F-C54-005–F-C54-006]
- **X-005 — cumulative accounting chain:** C-25 defines cumulative prompt/cache accounting and budget recalibration; C-26 consumes it for measurement, and C-54 later records per-task token/cost metrics. Field/unit alignment across `ProviderUsage`, `CloudSpendPayload`, benchmark scorecards, and methodology is required. [F-C25-003–F-C25-006, F-C25-040; F-C26-002, F-C26-032; F-C54-014–F-C54-017]
- **X-006 — compilation/decoding ordering:** C-25 and C-26 are edit-parallel but not acceptance-parallel: C-26 requires C-25 for smoke accounting, and C-25 itself needs C-9 for crystallization proof. [F-C25-002, F-C25-046, F-C25-051; F-C26-002]
- **X-007 — decoding decision into benchmark lock:** C-26 proposes the per-role defaults that C-37 later re-evaluates; C-54 requires that re-evaluation to have run and freezes its result in methodology before the benchmark binary is locked. [F-C26-007; F-C54-002, F-C54-004, F-C54-017]
- **X-008 — closed configuration surface:** C-21’s `ParameterKey` makes self-modifiable parameters a closed set. C-25/C-26/C-46/C-47/C-54 add new configuration, but none explicitly adds those controls to `ParameterKey`; they should remain ordinary or locked config unless a later plan deliberately widens the enum. [F-C21-013–F-C21-015; F-C25-034–F-C25-036; F-C26-004–F-C26-007; F-C46-011; F-C47-013; F-C54-009–F-C54-011]
- **X-009 — selective delegation dependency:** C-46 may consume C-45 credit weights but has a complete difficulty-only default policy when they are unavailable; the integration must distinguish “no credit data” from a zero credit weight. [F-C46-002, F-C46-006–F-C46-010]
- **X-010 — routing evidence:** C-46’s 100-task pilot measures solve-rate/cost effects of pruning, while C-54’s three locked configurations measure full-system solve-rate/cost. If selective delegation is enabled in a C-54 arm, its state belongs in the full config snapshot even though C-54 only explicitly names C-35/C-37 state. [F-C46-017–F-C46-018; F-C54-013–F-C54-018]
- **X-011 — MCP security chain:** C-47 is sequenced after C-35 tool-attention and C-42 zero-trust hardening and is reached on every enabled MCP invocation. Its unaddressed call-shape/session privacy surface therefore sits inside, but is not solved by, the existing bridge security gate. [F-C47-002, F-C47-005, F-C47-013, F-C47-023]
- **X-012 — formal candidates are externally staged:** C-52’s MemoryView and mutation-aggregation proofs depend on C-48/C-28 candidate behavior even though only Phase 4F is called a strict prerequisite; proof work cannot faithfully mirror modules that have not stabilized. [F-C52-002, F-C52-006–F-C52-008]
- **X-013 — test-trust feeds public evidence:** C-54 consumes C-28 mutation and C-29 metamorphic/wiring evidence in every task record; C-52 is explicitly above that evidence layer and is not a substitute for benchmark hidden-test pass/fail. [F-C52-005; F-C54-014, F-C54-030]
- **X-014 — shared SSB schema:** C-8 already bumps SSB schema 49→50. C-21, C-25, C-26, C-38, C-46, C-47, C-52, and C-54 all add or reference actions; concurrent landing requires one rebased, atomic schema/count/parse migration rather than independent version assumptions. [F-C8-026; F-C21-009; F-C25-007, F-C25-022, F-C25-025; F-C26-029–F-C26-031; F-C38-013–F-C38-014; F-C46-012–F-C46-013; F-C47-011–F-C47-012; F-C52-016; F-C54-023]
- **X-015 — determinism layers differ:** C-8 uses injected clocks; C-38 repeats exact PageRank input ten times; C-46 seeds classification; C-47 fixes retrieval/eviction order; C-54 promises ±0 tasks only for seeded local Config A and explicitly allows hybrid variation. These are compatible but not interchangeable determinism claims. [F-C8-030–F-C8-031; F-C38-012; F-C46-014; F-C47-008, F-C47-019; F-C54-022]
- **X-016 — deferred evidence boundary:** C-8 is closed while its cloud smoke is deferred; C-46 defers the pilot; C-52 accepts deferred infeasibility; C-54 is entirely RunPod-gated. Status reports must name which evidence tier closed rather than collapsing all four into “done.” [F-C8-053, F-C8-056; F-C46-017–F-C46-024; F-C52-022, F-C52-030; F-C54-039]

## Additions and corrections to `03-C-SUMMARY_INDEX.discovery.md`

Each item below is a delta, not a duplicate feature count. “Addition” means explicit source detail omitted by the orientation summary; “correction” means the summary’s statement is superseded, overbroad, or materially misleading against the exact current source.

### C-8 deltas

- **D-C8-001 [correction]** C-8 is now explicitly `CLOSED`, with 12 tasks shipped and 14/14 contract tests passing; the summary presents it as undifferentiated planned production activation. [Summary L182-L199; C-8 L3-L10, L389-L440]
- **D-C8-002 [correction]** The hourly Cosmos scheduler is in long-lived `live_cli`, not `task_runner`; it is gated by SSB/manifest availability and emits events through an external wrapper. [Summary L191; C-8 L118-L138]
- **D-C8-003 [correction]** Artifact integration is `maybe_index_artifacts`/per-artifact `index_artifact`, nonfatal on error; `ArtifactStale` rows were dropped and `ArtifactsIndexed` was reused. [Summary L192-L193; C-8 L140-L174]
- **D-C8-004 [addition]** The artifact RPC keeps a singular wire request, returns the six-field JSON shape, emits one event, and maps errors to `AgentRpcError::ToolError`. [Summary L193; C-8 L155-L174]
- **D-C8-005 [correction]** Search mode is not stored in `AppState.search_mode`; it crosses processes through `SearchModeChanged`, with startup/transition emits, viewer lookup, lowercase fallback, diagnostic setter, and atomic schema 49→50. [Summary L195; C-8 L181-L197]
- **D-C8-006 [correction]** Pipeline latency uses injected `ClockProvider`, not raw `Instant::elapsed`; all 17 ordinary plus two synthetic sites are covered, with one explicit RPC-only `Instant` exemption. [Summary L197; C-8 L204-L227]
- **D-C8-007 [addition]** The ONNX path has an explicit warned norm-proxy fallback, so “replaces” is not an unconditional removal. [Summary L196; C-8 L282-L283, L424-L425, L487-L490]
- **D-C8-008 [addition]** `CosmosStage` had to be created; its token-overlap router, score 0.75, snippet cap 1,024, and exact stage ordering are omitted from the summary. [Summary L198; C-8 L229-L242]
- **D-C8-009 [addition]** `latest_ambiguity_scored` is the shipped method name, and the alleged `unwired.ambiguityState` flag is a resolved phantom. [Summary L190; C-8 L100-L116, L244-L257]
- **D-C8-010 [addition]** The named cloud smoke is deferred to C-9+, while four deterministic contract files are current closure evidence; an orphan Cosmos helper and frontend mode picker remain follow-ups. [Summary L199; C-8 L441-L483, L559-L570]

### C-17 deltas

- **D-C17-001 [addition]** All acceptance boxes remain unchecked; summary verbs such as “adds/wires/fixes” describe requirements, not shipped evidence. [Summary L372-L396; C-17 L233-L268]
- **D-C17-002 [addition]** Kotlin dependency selection has an anti-slopsquatting `cargo search` gate, and its Java-parser fallback only guarantees the Kotlin language ID. [Summary L380-L381; C-17 L60-L68]
- **D-C17-003 [addition]** The omitted Svelte test contract covers TypeScript-qualified names, CSS isolation, and no-panic parsing. [Summary L395; C-17 L70-L74]
- **D-C17-004 [addition]** Resume is ordered after initialization/before dispatch; shutdown must remain reachable from both CLI signal paths and dashboard exit. [Summary L382-L384; C-17 L76-L84, L169-L172]
- **D-C17-005 [addition]** Watch lifecycle includes lock acquisition on start and release on stop, and resilience tests pin a five-second empty resume plus a 30–120-second shutdown deadline. [Summary L385-L386, L395; C-17 L86-L98]
- **D-C17-006 [correction/addition]** Splitters apply to a broader non-code extension route before fallback, not merely Markdown; exact splitter precedence remains unspecified. [Summary L387-L388; C-17 L100-L105]
- **D-C17-007 [addition]** Scale runs name VS Code/Linux minimums, Chromium/Postgres stretch fixtures, and three specific scale contracts; banners have exact user-visible strings. [Summary L392-L396; C-17 L120-L139]

### C-21 deltas

- **D-C21-001 [addition]** The source is unchecked and Chunk C has a hard authorization boundary: explicit `Y` only, with `N` or silence prohibiting edits and `.expect()` banned. [Summary L474-L475; C-21 L35-L36, L60-L64]
- **D-C21-002 [addition]** The local pre-commit hook is installed but not committed and matches an explicit 12-pattern/path set with an exact failure message. [Summary L476-L477; C-21 L66-L74]
- **D-C21-003 [addition]** Steering refresh occurs only after successful promotion, logs PID, warns without panic when missing, and dependency audits use exact output/sentinel paths with a 24-hour mtime gate. [Summary L478-L482; C-21 L76-L97]
- **D-C21-004 [addition]** The 12 enum variants include typed `vram_tier`/`slot` payloads; rules-derived constraints append, dedupe, and silently fall through; SWE fixtures forbid solution patches. [Summary L483-L487; C-21 L99-L131]
- **D-C21-005 [ambiguity addition]** The summary omits that STRICT activation, sentinel race/failure semantics, and unavailable-rules observability are undefined. [Summary L474-L488; C-21 L54-L57, L92-L122]

### C-25 deltas

- **D-C25-001 [addition]** C-25 is wholly unchecked, and C-9 is the explicit blocker for final crystallization acceptance. [Summary L560-L598; C-25 L238-L272, L297-L302]
- **D-C25-002 [addition]** Tracker/background/compiled types have exact fields; eligibility requires all-L1 steps, ratio `<0.3`, and use count `>=3`. [Summary L568-L591; C-25 L46-L89]
- **D-C25-003 [addition]** Persistence recovers corrupt MessagePack to empty+warn, persists through temp+rename, evicts after 604800 seconds during upsert, and `/pipeline reset` deletes the file. [Summary L574-L575, L596; C-25 L108-L132]
- **D-C25-004 [addition]** Cache matching, confidence derivation, concurrency/durability, and concept/hash canonicalization are not fixed by the plan. [Summary L568-L575, L593-L595; C-25 L100-L127]
- **D-C25-005 [conflict addition]** Dropping the only join handle on soft detach is not reconciled with later completion observation/hard-timeout enforcement. [Summary L579-L584, L598; C-25 L66-L75, L139-L141]

### C-26 deltas

- **D-C26-001 [addition]** C-26 is unchecked; C-10 absence compiles/ignores one path, but full persona-switch smoke still requires C-10. [Summary L600-L630; C-26 L12-L15, L81-L84, L143-L145, L313-L317]
- **D-C26-002 [conflict correction]** “Derive Default” cannot produce the stated numeric/bool defaults; implementation needs a custom default for max retries 3 and strict true. [Summary L608-L610; C-26 L44-L50]
- **D-C26-003 [addition]** `strict` semantics and Regex provider encoding are absent even though Regex is a mode and drives acceptance. [Summary L608-L622, L629; C-26 L44-L50, L86-L104]
- **D-C26-004 [conflict addition]** Same-error futility only escalates, whereas the Regex smoke also expects persona switching; persona switching is specified only for the separate similarity path. [Summary L616-L619, L629; C-26 L72-L82, L209-L213]
- **D-C26-005 [addition]** `constrained_decoding_used` and the persona-switch row are acceptance observables without produced action/payload tasks. [Summary L618-L630; C-26 L162-L179, L204-L213]
- **D-C26-006 [addition]** Fixture sizes/content, grammar-language subsets, actual-measurement/no-fabrication rule, and all 15 named contract responsibilities are explicit in the source. [Summary L623-L630; C-26 L106-L160]

### C-38 deltas

- **D-C38-001 [addition]** C-38 remains unchecked, and closure is blocked if B-1’s silent DKG gate still exists. [Summary L953-L969; C-38 L100-L105, L170-L185]
- **D-C38-002 [addition]** Fresh fetch must extract six specific algorithm attributes and feed a five-column source-of-truth diff plus dated footnote/rule update. [Summary L957, L965-L967; C-38 L61-L81, L107-L110]
- **D-C38-003 [correction]** The source says confirm/reuse `pagerank_run` and add rebuild events only if absent; the summary’s “extends pagerank_run” is not a settled change. [Summary L968; C-38 L96-L98]
- **D-C38-004 [addition]** Every event carries repo/fixture ID and elapsed time; the production contract uses a 50-file/20-import hub and repeats top-k ten times. [Summary L963-L969; C-38 L83-L98]
- **D-C38-005 [addition]** External superiority claims remain explicitly unverified until both production proof and docs correction land. [Summary L953-L969; C-38 L204-L209]

### C-46 deltas

- **D-C46-001 [correction]** “Trigram” is the default classifier-model identifier, not one of the actual feature bullets; features are length, file count, centrality, and reference-task embedding cosine. [Summary L1115; C-46 L47-L51, L65-L67]
- **D-C46-002 [correction]** Auditor insufficiency re-spawns the **full chain** in one retry, not merely an unspecified “larger delegation tier.” [Summary L1121; C-46 L60-L63]
- **D-C46-003 [addition]** Config defaults disabled/`trigram_centrality`, emits decision before spawn, and requires nonnegative savings. [Summary L1113-L1123; C-46 L65-L71, L100-L105]
- **D-C46-004 [conflict addition]** Six-role context, eight-role prerequisite, and medium-tier ContextCompiler/PromptEngineer membership do not yield one unambiguous full-chain definition. [Summary L1110-L1120; C-46 L9-L10, L21-L24, L53-L58]
- **D-C46-005 [addition]** The pilot is explicitly deferred to RunPod and the feature must remain disabled until its ±2pp/≥30% gate passes. [Summary L1123; C-46 L83-L85, L124-L127]

### C-47 deltas

- **D-C47-001 [correction]** The task permits sled or SQLite (sled preferred) while acceptance demands sled; “adds a sled-backed CaseBase” overstates a conflicted choice. [Summary L1135; C-47 L47-L51, L107-L109]
- **D-C47-002 [correction]** Eviction is deterministically oldest-by-timestamp, not true access-based LRU as the summary says. [Summary L1136; C-47 L47-L51, L91-L94]
- **D-C47-003 [addition]** The seven-column schema, post-return logging order, success-only hint condition, emit-point suppression, and non-null `tool_name` event gate are omitted. [Summary L1133-L1140; C-47 L47-L72, L91-L95, L105-L113]
- **D-C47-004 [addition]** Top-K/threshold/weight/tie/hash behavior and whether failures occupy the cap are unspecified. [Summary L1135-L1138; C-47 L25-L32, L47-L61]
- **D-C47-005 [addition]** Per-project call shapes/string values/session IDs have no specified redaction, encryption, retention, access, isolation, or deletion policy. [Summary L1133-L1140; C-47 L25-L35, L47-L51]

### C-52 deltas

- **D-C52-001 [addition]** C-52 may validly close `DEFERRED`; partial success requires a note and cannot be called full success. [Summary L1219-L1233; C-52 L109-L114, L142-L146]
- **D-C52-002 [conflict addition]** Installability is requested on Windows+Linux but success accepts one host; context says 3–5 modules while the actual contract fixes three. [Summary L1221-L1229; C-52 L33-L36, L53-L60, L131-L136]
- **D-C52-003 [addition]** `.dfy` files specify claimed Rust behavior but the plan contains no proof/refinement bridge showing Rust conforms to them. [Summary L1226-L1229; C-52 L62-L69]
- **D-C52-004 [addition]** Informational CI is nonblocking, infeasibility needs a concrete blocker plus revisit trigger, and formal-verification documentation is required for either outcome. [Summary L1229-L1233; C-52 L71-L89, L142-L161]

### C-54 deltas

- **D-C54-001 [correction]** C-37 need only have been run, not necessarily shipped; C-44 absence permits a Sonnet-only downgrade, which conflicts with the later fixed Config-C acceptance. [Summary L1259; C-54 L14-L20, L85-L87, L212-L213]
- **D-C54-002 [addition]** Config A binds both roles to one local model, Config B uses distinct instances (sequential on one GPU), and Config C applies Phase-4M/C-41 routing. [Summary L1265-L1269; C-54 L80-L87]
- **D-C54-003 [addition]** Per-task and aggregate metric schemas, hardware/config/gating/outlier methodology fields, and once-per-config run policy are omitted. [Summary L1269-L1274; C-54 L89-L105]
- **D-C54-004 [correction]** Methodology is auto-generated from machine data; `results.md` is separately authored. The summary’s wording that methodology/results are generated from machine-readable inputs overextends the anti-drift rule. [Summary L1271-L1274; C-54 L97-L121, L161-L164, L214-L215]
- **D-C54-005 [addition]** Interrupted sweeps discard partials and restart fully; normal CI is excluded, and manual CLI launch is required. [Summary L1263-L1274; C-54 L135-L141, L161-L166]
- **D-C54-006 [addition]** The honesty statement fixes pass@1/version/hardware/cost/no-cherry-pick language and distinguishes ±0 seeded local from ±1–3 hybrid variation. [Summary L1272-L1274; C-54 L116-L121]
- **D-C54-007 [addition]** All acceptance remains unchecked; no score, cost, or completed RunPod run exists in the source. [Summary L1253-L1274; C-54 L23-L24, L202-L217]

## Zero-omission and zero-duplicate audit

### Full-read line coverage

Every logical line in every manifest source was read. The following section map proves contiguous start-to-end coverage; blank separator lines are included inside the adjacent ranges.

| Source | Complete source range | Section ranges covered |
|---|---:|---|
| C-8 | L1–L572 | L1–51 status/objective; L52–91 context/reads; L92–262 tasks; L263–322 wiring; L323–397 checks; L398–452 acceptance; L453–484 tests; L485–558 files; L559–572 next |
| C-17 | L1–L291 | L1–11 objective; L12–44 context; L45–57 reads; L58–145 tasks; L146–184 wiring; L185–232 checks; L233–269 acceptance; L270–287 files; L288–291 next |
| C-21 | L1–L263 | L1–12 objective; L13–37 context/gate; L38–51 reads; L52–141 tasks; L142–180 wiring; L181–223 checks; L224–238 acceptance; L239–260 files; L261–263 next |
| C-25 | L1–L302 | L1–14 objective; L15–31 context; L32–43 reads; L44–146 tasks; L147–192 wiring; L193–237 checks; L238–273 acceptance; L274–296 files; L297–302 next |
| C-26 | L1–L317 | L1–16 objective; L17–27 context; L28–41 reads; L42–161 tasks; L162–214 wiring; L215–265 checks; L266–294 acceptance; L295–312 files; L313–317 next |
| C-38 | L1–L209 | L1–24 objective/grounding; L25–52 context; L53–60 reads; L61–111 tasks; L112–140 wiring; L141–169 checks; L170–186 acceptance; L187–196 files; L197–203 reports; L204–209 next |
| C-46 | L1–L147 | L1–18 objective/grounding; L19–36 context; L37–42 reads; L43–88 tasks; L89–106 wiring; L107–115 checks; L116–128 acceptance; L129–141 files; L142–147 next |
| C-47 | L1–L133 | L1–L17 objective/grounding; L18–L36 context; L37–42 reads; L43–77 tasks; L78–95 wiring; L96–104 checks; L105–117 acceptance; L118–128 files; L129–133 next |
| C-52 | L1–L169 | L1–L17 objective/grounding; L18–40 context; L41–48 reads; L49–90 tasks; L91–115 wiring; L116–128 checks; L129–147 acceptance; L148–162 files; L163–169 next |
| C-54 | L1–L248 | L1–L30 objective/dependencies; L31–58 context; L59–66 reads; L67–142 tasks; L143–170 wiring; L171–201 checks; L202–218 acceptance; L219–235 files; L236–240 reports; L241–248 next |

### Atomic-ID and manifest audit

| Source | Lines | Atomic items | ID interval | Sequence gaps | Duplicate IDs |
|---|---:|---:|---|---:|---:|
| C-8 | 572 | 56 | `F-C8-001`…`F-C8-056` | 0 | 0 |
| C-17 | 291 | 36 | `F-C17-001`…`F-C17-036` | 0 | 0 |
| C-21 | 263 | 35 | `F-C21-001`…`F-C21-035` | 0 | 0 |
| C-25 | 302 | 52 | `F-C25-001`…`F-C25-052` | 0 | 0 |
| C-26 | 317 | 45 | `F-C26-001`…`F-C26-045` | 0 | 0 |
| C-38 | 209 | 28 | `F-C38-001`…`F-C38-028` | 0 | 0 |
| C-46 | 147 | 24 | `F-C46-001`…`F-C46-024` | 0 | 0 |
| C-47 | 133 | 24 | `F-C47-001`…`F-C47-024` | 0 | 0 |
| C-52 | 169 | 30 | `F-C52-001`…`F-C52-030` | 0 | 0 |
| C-54 | 248 | 39 | `F-C54-001`…`F-C54-039` | 0 | 0 |
| **Total** | **2651** | **369** | **10 complete sequences** | **0** | **0** |

The 16 `X-*` interactions and 59 `D-*` summary deltas are intentionally excluded from the 369 atomic-source count because each synthesizes or compares already-counted source facts. Manifest membership is also duplicate-free: 10 distinct absolute paths, 10 distinct hashes, no generated summary/index source counted as an original. Final result: **10/10 files processed, 2,651/2,651 logical lines read, 140,924/140,924 bytes hash-verified, 369 atomic source items, zero missing ID sequence positions, zero duplicate atomic IDs, and zero out-of-chunk original sources read.**
