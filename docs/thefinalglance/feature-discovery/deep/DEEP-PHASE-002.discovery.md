# DEEP-PHASE-002 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1344-L1355`, heading `### DEEP-PHASE-002`; the next chunk starts at L1357.
- Declared chunk: family PHASE; **6 files**, **2,750 logical lines**, **178,707 bytes**, all six classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all six listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `1A:Lx-Ly`, `1D:Lx-Ly`, `3D:Lx-Ly`, `3G:Lx-Ly`, `4DOR:Lx-Ly`, and `4G:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 513 | 35,090 | 69a8a3c800c8d7d7dfcf87e8a9c1afa11ed788536229692f9733691f8df77cac | docs/specs/guide/steps/phase-1A.txt | exact |
| numbered/index original | 332 | 27,167 | 94540288abb14374c3dd04b141f1b9c5cd580c72bc14e3edca1f9073c29258c2 | docs/specs/guide/steps/phase-1D.txt | exact |
| numbered/index original | 388 | 28,121 | 005df90dea6fb7731f33352bf7c02d8c4ac3fe97c5a258a602aa35974225fe5a | docs/specs/guide/steps/phase-3D.txt | exact |
| numbered/index original | 463 | 28,390 | 82d8892683f3dbb18ef43736617debc138af5d920551cd70a0dbbefd3469e051 | docs/specs/guide/steps/phase-3G.txt | exact |
| numbered/index original | 137 | 4,940 | 728470c4ff73b333dc416eb572e703ac1b8cca0bd250315086eee3d2355fc04d | docs/specs/guide/steps/phase-4D-ORACLE.txt | exact |
| numbered/index original | 917 | 54,999 | a55360f45161ddb73570cd00fdebe32b1c2a6c6fcb971f28740b76e04e9f0169 | docs/specs/guide/steps/phase-4G.txt | exact |
| **Aggregate** | **2,750** | **178,707** | — | **6 exact paths** | **exact** |

## Per-file atomic ledgers

### 1A — DKG (tree-sitter → SQLite graph)

Source: `docs/specs/guide/steps/phase-1A.txt` (513/513 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1A-E001 | E/STATUS | Phase 1A builds Layer-1 DKG: parse source into a dependency knowledge graph stored in SQLite. Prerequisites: Phase 0 complete (9-crate layout, cargo check passes). | 1A:L1-L12 |
| P1A-E002 | E/DEP | Canonical knowledge crate path is dynamic: `KNOW_PATH=$(find .elai_cc -path "*/knowledge/Cargo.toml" | head -1 | xargs dirname)`. Expected path `.elai_cc/crates/knowledge/`. Phase 0 may still reference pre-restructure `.elai_cc/rust/crates/`. | 1A:L3-L6, L53-L59 |
| P1A-E003 | E/DEP | Downstream consumers: Phase 1D PageRank reads DKG edges; Phase 1E CatRAG traverses DKG; Phase 1B BM25 shares readiness gate but runs independently. | 1A:L13-L15 |
| P1A-E004 | E/STATUS | knowledge.txt “FIVE LAYERS” was corrected to “5 major + 2 sub-layers”; architecture-overview “7 layers” counts sub-layers 3.25 and 3.5. DKG is Layer 1. | 1A:L17-L19, L470-L470 |
| P1A-E005 | E/LIMIT | R7 (LOW): tree-sitter grammar gaps. Start with Rust, Python, TypeScript, Go. | 1A:L26-L27 |
| P1A-E006 | E/DATA | R-1A-1: SQLite schema is not in upstream specs; this phase must define schema explicitly. | 1A:L28-L28, L468-L469 |
| P1A-E007 | E/OBS | R-1A-2: no build-time baseline exists. knowledge.txt L20 “<15s” claim is UNGROUNDED; 1A.5 establishes first measurement. | 1A:L29-L29, L407-L408, L467-L468 |
| P1A-E008 | E/DEP | R-1A-3: Phase 1D PRE_GATE requires edge/adjacency iteration API from DkgEngine; without it Phase 1D is blocked at entry. | 1A:L30-L31, L165-L175, L426-L426 |
| P1A-E009 | E/FAIL | R-1A-4 / COMPLIANCE SCOPE: knowledge.txt Sec 8 has 5 parse-failure items. Phase 1A implements only items 1 (exclude from DKG) and 5 (log + retry on next change). Items 2–4 (BM25 still indexes, vector still embeds, PageRank leaf) are deferred to 1B/1C/1D. | 1A:L32-L47, L338-L342, L427-L427 |
| P1A-E010 | E/LIMIT | Orphan-file invariant “every file appears in at least BM25 index” is NOT achievable until Phase 1B. Phase 1A tracks parse-failed files so 1B can index them. | 1A:L46-L47, L449-L450 |
| P1A-E011 | E/CAP | LanguageGrammar must be object-safe for `dyn` dispatch: all methods take `&self`. | 1A:L35-L35, L84-L91, L460-L461 |
| P1A-E012 | E/DATA | DkgNodeInfo must include `signature` and `SymbolKind` to match Phase 1D contract. | 1A:L36-L37, L92-L109, L453-L454 |
| P1A-E013 | E/CFG | Dependencies (verified crates.io 2026-04-06): `tree-sitter=0.26`, `tree-sitter-rust=0.24`, `rusqlite={version=0.39, features=["bundled"]}`, `blake2=0.10`, dev `proptest=1`. Future: python 0.25, go 0.25, typescript 0.23 (aging; re-verify). | 1A:L60-L69, L456-L459 |
| P1A-E014 | E/CAP | KnowledgeEngine trait methods: `build_graph`, `query`, `rerank`, `update`, `readiness_check`. | 1A:L70-L75 |
| P1A-E015 | E/DATA | DependencyGraph return: `{ node_count, edge_count, files_parsed }`. | 1A:L76-L77, L336-L337 |
| P1A-E016 | E/STATE | ReadinessStatus enum: READY, DEGRADED, UNAVAILABLE (full-engine aggregate). | 1A:L78-L78 |
| P1A-E017 | E/STATE | Phase 1A implements DKG-layer readiness only via `dkg_readiness() -> DkgReadiness` (Ready, Empty, Unavailable). Trait `readiness_check()` aggregation is Phase 1I. | 1A:L79-L83, L172-L177, L343-L348, L429-L429, L477-L477 |
| P1A-E018 | E/CAP | LanguageGrammar methods: `language_id`, `tree_sitter_language`, `extract_nodes`. Start with RustGrammar only. | 1A:L84-L91, L302-L313 |
| P1A-E019 | E/DATA | SymbolKind: Function, Struct, Impl, Trait, Enum, Module, Use, TypeAlias, Other. | 1A:L92-L93 |
| P1A-E020 | E/DATA | DkgNodeInfo fields: node_id, symbol_name, file_path, signature, kind, start_line, end_line. Signature reconstructed from AST (function/struct/trait/impl/enum forms). | 1A:L94-L109, L305-L311 |
| P1A-E021 | E/DATA | DkgEngine::new(db_path) takes explicit project-scoped path (caller provides `.elai_cc/projects/{project_id}/knowledge.db`). | 1A:L110-L113, L351-L353, L433-L433 |
| P1A-E022 | E/DATA | dkg_nodes columns: id PK, file_path, name, kind, signature DEFAULT '', start_line, end_line, parent_id, concept DEFAULT NULL, concept_stale DEFAULT 1, file_hash BLOB. | 1A:L115-L127 |
| P1A-E023 | E/DATA | dkg_edges: source_id, target_id, edge_type, PRIMARY KEY (source_id, target_id, edge_type), ON DELETE CASCADE. Edge types: "imports", "contains", "implements" (directed). | 1A:L128-L139, L318-L321 |
| P1A-E024 | E/DATA | dkg_files: file_path PK, file_hash, parse_status ('ok'/'failed'/'pending'), parse_error, last_indexed. Enables O(1) staleness and Phase 1B failed-file bridge. | 1A:L140-L148 |
| P1A-E025 | E/DATA | dkg_meta key/value with initial rows schema_version=1, created_at. Indexes: name, file, kind, files_status. | 1A:L149-L155 |
| P1A-E026 | E/DATA | index_version.toml five fields: version, dkg_schema_version, vector_dimensions placeholder, bm25_format placeholder, created_at. | 1A:L156-L164, L349-L350, L430-L430 |
| P1A-E027 | E/CAP | Public API: nodes, edges, node_by_id, outbound_edges, inbound_edges, failed_files, dkg_readiness. Directed edge queries replace ambiguous neighbors(). | 1A:L165-L175, L324-L330, L484-L484 |
| P1A-E028 | E/FLOW | Steps: 1A.1 skeleton → 1A.2 verify workspace → 1A.3 TDD tests (≥20, ≥10 fail red) → 1A.4 implement → 1A.5 measure. | 1A:L50-L420 |
| P1A-E029 | E/TEST | 1A.3 functional tests: parse Rust nodes into SQLite; query by function/struct name; signature population; incremental update with file_hash change and concept_stale=1; concept NULL default; file_hash blake2s round-trip. | 1A:L236-L244 |
| P1A-E030 | E/TEST | Edge-type tests: impl→method contains; use→imports; impl Trait→implements; outbound direction. Downstream API tests for edges/outbound/inbound/nodes/failed_files. | 1A:L245-L255 |
| P1A-E031 | E/TEST | Parse-failure tests: empty file zero nodes no panic; syntax errors partial extract + parse_status=failed; binary/non-text graceful skip + failed_files. | 1A:L256-L261 |
| P1A-E032 | E/TEST | Readiness: Unavailable before build; Ready after >0 nodes; Empty after empty/all-failed directory. Isolation: two DkgEngine db_paths do not share state. | 1A:L262-L267, L394-L394 |
| P1A-E033 | E/TEST | proptest: random valid Rust → node_count≥0, line numbers>0, names non-empty, hashes non-empty, signatures non-empty for function/struct. | 1A:L268-L271 |
| P1A-E034 | E/TEST | TDD red gate: ≥20 test functions; cargo test --no-run ok; ≥10 FAILED (stubs pass ≤3 trivial). | 1A:L273-L294 |
| P1A-E035 | E/CFG | SQLite WAL: PRAGMA journal_mode=WAL. blake2s content hash for incremental staleness. | 1A:L315-L317, L331-L335, L360-L361 |
| P1A-E036 | E/FLOW | Incremental update: hash compare; unchanged skip; changed DELETE nodes for file_path (CASCADE edges), re-parse, concept_stale=1. | 1A:L331-L335 |
| P1A-E037 | E/FAIL | On parse failure: log::warn, dkg_files parse_status=failed, no nodes, no panic/abort. | 1A:L338-L342, L393-L393 |
| P1A-E038 | E/OBS | Metrics file `.elai_cc/metrics/phase_1A.toml` with ≥12 keys: build_time_ms, query_p50/p99_ms, db_size_bytes, total_nodes/edges, files_parsed/skipped, edges_imports/contains/implements, signatures_populated. | 1A:L396-L420, L434-L434 |
| P1A-E039 | E/EVENT | Wiring observable: trigger `elai-cli index build --project-root ...` expects SSB `symbol_graph_updated`. | 1A:L511-L513 |
| P1A-E040 | E/INT | Production reachability: `cli/src/index_cmd.rs::run_index_build`, `orchestrator/src/indexing_service.rs::IndexingService::run_full_index`. | 1A:L509-L510 |
| P1A-E041 | E/CAP | produces_symbols: DkgEngine::{new,build_graph,nodes,edges,outbound_edges,inbound_edges,failed_files,dkg_readiness}, DependencyGraph, DkgNodeInfo, SymbolKind, DkgReadiness, LanguageGrammar, ReadinessStatus. | 1A:L490-L508 |
| P1A-E042 | E/LIMIT | Exit criteria require directed edge API, failed_files for 1B, DKG-only readiness (not full-engine), index_version.toml 5 fields, signature+SymbolKind, concept_stale, project-scoped db, metrics, no Phase 0 regressions. | 1A:L423-L435 |
| P1A-E043 | E/DEFER | BM25 still-index, vector embed, PageRank leaf treatment of parse-failed files deferred to 1B/1C/1D. Python/TS/Go grammars deferred beyond Rust start. | 1A:L42-L47, L67-L69, L427-L427 |
| P1A-I001 | I/DATA | Edge types imports/contains/implements are design-derived from knowledge.txt edge-weight/CatRAG and phase-1D, not literal upstream schema. | 1A:L462-L463 |
| P1A-A001 | A/CFG | Exact tree-sitter-typescript pin may need re-verification (aging 2024-11). No concrete acceptance for multi-language beyond Rust start. | 1A:L67-L69 |
| P1A-A002 | A/OBS | No formal query-latency SLO; only baseline recording. | 1A:L468-L468 |
| P1A-A003 | A/FAIL | “Partial extraction” on syntax errors has no defined completeness floor for which nodes may be kept. | 1A:L258-L259 |

Category coverage: capabilities/API ✓; workflows/steps ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events/wiring ✓; security/isolation (project db) ✓; failures/fallback ✓; determinism/tests ✓; observability/metrics ✓; constraints/deferred ✓. No standalone privacy/compliance rule. No formal idempotency key beyond content-hash skip.

### 1D — PageRank repo map

Source: `docs/specs/guide/steps/phase-1D.txt` (332/332 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1D-E001 | E/STATUS | Phase 1D ranks code symbols by PageRank on DKG and produces token-budget-fitted repo maps. Prerequisites: Phase 1C complete (knowledge + ort). | 1D:L1-L8 |
| P1D-E002 | E/EVIDENCE | Derives from aider/repomap.py (design 31.7). Three deliberate divergences: symbol-level nodes, density guard, insertion-order fallback. Five adopted improvements with tests. | 1D:L14-L17, L251-L279 |
| P1D-E003 | E/FAIL | R20 density degradation: density guard at 0.3. NOTE: R20 ID collides with phase-0 entropy quarantine; cross-file renumber deferred. | 1D:L19-L23, L314-L316 |
| P1D-E004 | E/RECOVERY | Dangling nodes (zero out-edges) redistribute via personalization vector (aider dangling=personalization / NetworkX). | 1D:L24-L28, L178-L183, L255-L257 |
| P1D-E005 | E/DATA | Orphan symbols get self-edges weight=0.1 during DkgSnapshot::from_engine so rank > 0. | 1D:L29-L32, L54-L56, L258-L260 |
| P1D-E006 | E/DATA | DkgSnapshot { nodes: Vec<DkgNodeInfo>, edges: Vec<(usize,usize,f64)> }; DkgNodeInfo adds ref_count (incoming references for sqrt scaling). Built via from_engine. | 1D:L48-L53 |
| P1D-E007 | E/CFG | PageRankConfig defaults: damping_factor=0.85, max_iterations=100, tolerance=1e-6, density_threshold=0.3, max_map_tokens=2048. damping/max_iter/tol are standard PageRank (UNGROUNDED in ELAI specs) but overridable via config. | 1D:L58-L60, L303-L304 |
| P1D-E008 | E/DATA | RepoMap { ranked_symbols, token_count, density, used_pagerank, fit_iterations }. RankedSymbol { node_id, symbol_name, file_path, rank, signature }. | 1D:L61-L63 |
| P1D-E009 | E/CAP | compute_repo_map(snapshot, task, active_files, config, token_estimator: &dyn Fn(&str)->usize) -> RepoMap. | 1D:L64-L65 |
| P1D-E010 | E/TEST | ≥21 unit tests T1–T18 + A1–A3; proptest P1–P3; make_snapshot helper. TDD: tests fail on todo!(). | 1D:L96-L154, L319-L320 |
| P1D-E011 | E/TEST | T1 linear graph task mention ranks highest; T2 active_files edges *=50; T3 task mention 10× personalization; T4 density>0.3 → used_pagerank=false insertion-order fallback (NOT BM25); T5 density<0.3 → used_pagerank=true. | 1D:L102-L108, L171-L177 |
| P1D-E012 | E/TEST | T6 token_count ≤ max_map_tokens×1.15; T7 undershoot ≥85% of budget; T8 special files always included; T9 personalization sums to 1.0; T10 fit_iterations ≤10; T11 empty graph; T12 single node; T13 zero budget no panic; T14 all-mentioned uniform personalization finite ranks. | 1D:L108-L117 |
| P1D-E013 | E/TEST | T15 dangling redistribution; T16 orphan self-edges; T17 sqrt ref scaling rank ratio <8 not 25; T18 500-char signature truncated to 100 chars for token estimate. | 1D:L118-L132, L329-L332 |
| P1D-E014 | E/TEST | P1 ranks finite ≥0; P2 sum ranks ≈1 when used_pagerank; P3 token budget bound. A1 near-complete density guard; A2 star hub highest; A3 disconnected components base rank only. | 1D:L134-L140 |
| P1D-E015 | E/FLOW | Implement order: adjacency + sqrt(ref_count); self-edges; active_files boost ×50; personalization 10× then normalize; density = e/(n*(n-1)); if density>0.3 or n≤1 skip PageRank; power iteration with dangling redistribution; NaN/Inf clamp to 0. | 1D:L160-L209 |
| P1D-E016 | E/DATA | Special files hardcoded 13 Rust items: Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile, docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml. Config-driven list deferred. | 1D:L184-L186, L305-L305 |
| P1D-E017 | E/FLOW | Binary search fit: max 10 iterations, ±15%, initial guess min(max_map_tokens/25, num_nodes). max_map_tokens=0 → empty ranked, specials omitted. | 1D:L187-L194, L212-L213 |
| P1D-E018 | E/DATA | Line truncation max_chars=100 before token estimation (aider line[:100]). | 1D:L195-L198, L210-L210, L267-L269 |
| P1D-E019 | E/LIMIT | Not adopted from aider: identifier specificity boost, private underscore penalty, generic-name penalty (>5 defs). | 1D:L281-L288 |
| P1D-E020 | E/OBS | Metrics `.elai_cc/metrics/phase_1D.toml`: test_count, compile_time_seconds, pagerank_loc, ranking_quality, token_budget_accuracy, density_guard_correct, binary_search_max_iterations (7 metrics). | 1D:L230-L248 |
| P1D-E021 | E/TEST | Exit: pagerank tests 0 failures ≥21; density guard; ±15% budget; specials when budget allows; no todo!/unimplemented!; clippy 0 errors; metrics; T13; PROPTEST_CASES=100; T15–T18 pass. | 1D:L318-L332 |
| P1D-E022 | E/DEP | providers token_count_estimate used via mockable estimator closure; PRE_GATE greps for DKG edge API and token_count_estimate. | 1D:L39-L43, L150-L150 |
| P1D-E023 | E/DET | Density formula: edges.len()/(nodes.len()*(nodes.len()-1)). Insertion-order fallback uses DkgSnapshot node order. | 1D:L105-L106, L171-L177 |
| P1D-C001 | C/STATUS | Spec conflict fixed: PageRank is Layer 3.5 not Layer 4 (CatRAG is Layer 4). | 1D:L314-L315 |
| P1D-A001 | A/CFG | Special-files list is ungrounded relative to knowledge.txt “~100 items”; only 13 Rust items ship. | 1D:L305-L305 |
| P1D-A002 | A/CFG | Exact personalization mention-matching algorithm (substring vs symbol equality) not formalized beyond “mentioned in task”. | 1D:L104-L104, L169-L170 |
| P1D-A003 | A/DET | Density guard proof cites max_rank/min_rank <1.1 on dense graphs as justification, but that ratio is not a production invariant outside A1. | 1D:L173-L176, L276-L277 |

Category coverage: capabilities/API ✓; ranking workflow ✓; schemas/config ✓; special-file surface ✓; failures/fallbacks ✓; determinism/properties ✓; metrics ✓; dependencies/aider grounding ✓; deferred extensibility ✓. No UI panel; no privacy rule.

### 3D — BSP supersteps + channels

Source: `docs/specs/guide/steps/phase-3D.txt` (388/388 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3D-E001 | E/STATUS | Phase 3D: deterministic macro orchestration via BSP supersteps (Valiant 1990). Prerequisites: 3B SSB + 3C agents. | 3D:L1-L7 |
| P3D-E002 | E/LIMIT | BinaryOp intentionally dropped (non-deterministic with concurrent writes; audit BLOCK 4). | 3D:L8-L9, L363-L363 |
| P3D-E003 | E/DET | Determinism via total-ordered LastValue (SSB seq) or append-only Topic (SSB seq). Holds in simulation mode (mocked agents, fixed seed). Real LLMs may vary straggler participation; combiners well-defined on partial inputs but byte-identical checkpoints require identical agent sets. | 3D:L10-L18, L357-L361 |
| P3D-E004 | E/CFG | R19 UntrackedValue memory: configurable `elai_cc.toml [orchestrator.bsp.untracked_memory_limit_bytes]` default 10MB; force-clear at >2× limit; DeepSizeOf required (size_of_val banned). | 3D:L27-L32, L232-L238, L375-L376 |
| P3D-E005 | E/FAIL | Straggler timeout reuses Phase 3C heartbeat default 15s; AgentDeath::TIMEOUT removes agent from barrier; remaining proceed. | 3D:L33-L37, L162-L164, L377-L377 |
| P3D-E006 | E/RECOVERY | Crash mid-superstep: supervision tree removes dead agent; orchestrator does not wait for dead agents. | 3D:L38-L41 |
| P3D-E007 | E/DEFER | Spec amendments required before implement: Topic(accumulate=false) definition; memory_monitoring action_type; ChannelRegistry trait; save_checkpoint after evidence_gate. | 3D:L43-L54 |
| P3D-E008 | E/CAP | LastValue<T>: single value, last writer wins by SSB seq; read Option<T>. | 3D:L67-L68 |
| P3D-E009 | E/CAP | Topic<T>(accumulate: bool): true = append-only Vec persists; false = two-phase snapshot then clear before execute writes. Append order by SSB seq. | 3D:L69-L75, L248-L252, L372-L374 |
| P3D-E010 | E/CAP | EphemeralValue<T>: fire-and-forget; clear to MISSING when write_count==0 at barrier; saturating_add write counter; clearing journaled for replay. | 3D:L76-L82, L239-L247 |
| P3D-E011 | E/CAP | UntrackedValue<T:DeepSizeOf>: in-memory only, never checkpointed; multi-writer allowed; lost on crash. | 3D:L83-L85, L369-L369 |
| P3D-E012 | E/DATA | ChannelId typed key; ChannelState enum has exactly 3 variants (LastValue, Topic, Ephemeral) — no UntrackedValue. | 3D:L86-L87, L104-L104 |
| P3D-E013 | E/CAP | ChannelRegistry: read, write, snapshot_for_scheduling, clear_ephemeral, force_clear_untracked, deep_size_untracked. Types ChannelWrite/Value/Error/Snapshot. Implementation-defined until amendment 3. | 3D:L88-L100 |
| P3D-E014 | E/FLOW | Corrected BSP loop: snapshot → prepare_next_tasks → clear_non_accumulating_topics → execute_tasks_parallel → apply_writes (SSB seq) → deferred phase → clear_ephemeral → evidence_gate → save_checkpoint only if gate passes; else attempt++ up to max_superstep_attempts (default 3). max_supersteps total cap 100. | 3D:L125-L160, L385-L386 |
| P3D-E015 | E/DATA | Checkpoint path: `.elai_cc/sessions/{sid}/checkpoints/superstep_{N}.msgpack` (msgpack HashMap ChannelId→ChannelState). Superstep counter u64 monotonic in checkpoint. | 3D:L154-L161 |
| P3D-E016 | E/FLOW | Deferred nodes (NodeSpec.is_deferred): single pass post-barrier same superstep; all_inputs_satisfied required; no cascading deferred. | 3D:L186-L219, L364-L364 |
| P3D-E017 | E/EVENT | memory_monitoring SSB action (NEW, not isolation_metric): payload {channel_id, size_bytes, threshold_bytes, action: measure|force_clear}. isolation_metric remains cognitive-token only. | 3D:L232-L236, L381-L382 |
| P3D-E018 | E/RECOVERY | Crash recovery: load latest checkpoint; replay SSB since superstep: apply writes by seq; reconstruct ephemeral clears; clear Topic(accumulate=false) at boundary before writes. UntrackedValue lost by design. | 3D:L253-L258, L340-L340 |
| P3D-E019 | E/CONC | spawn_parallel upgraded from 3C sequential stub to tokio::JoinSet true parallel; VRAM-aware max_concurrent = floor(available_vram/per_agent_vram) fallback 1; results applied by SSB seq not completion order; wall-time concurrency proof. | 3D:L289-L327, L383-L384 |
| P3D-E020 | E/OBS | Baseline measurements (not exit criteria, UNGROUNDED estimates): superstep overhead <50ms, checkpoint <1MB, recovery <500ms, parallel speedup, UntrackedValue peak memory. | 3D:L346-L354, L387-L388 |
| P3D-E021 | E/TEST | Determinism proof: same seed + mocked agents → byte-identical checkpoints. No checkpoint file on failed gate. Integration 3+ supersteps with 2 agents. | 3D:L165-L184, L330-L344 |
| P3D-E022 | E/LIMIT | size_of_val banned in bsp/; DeepSizeOf required. No isolation_metric in bsp/ code. | 3D:L105-L105, L265-L265, L342-L342 |
| P3D-E023 | E/EVENT | Ephemeral clear recorded as action_type='checkpoint' subtype/payload ephemeral_clear / ephemeral_clears Vec. | 3D:L79-L82, L243-L247 |
| P3D-A001 | A/CFG | max_superstep_attempts default 3 and max_supersteps 100 are stated here; config key names beyond untracked_memory_limit not fully enumerated. | 3D:L158-L160 |
| P3D-A002 | A/INT | ChannelRegistry formalization blocked on orchestrator.txt amendment before 3E. | 3D:L97-L100 |
| P3D-A003 | A/DET | Evidence_gate_passes exact predicate is phase-transition evidence “sufficient” without full boolean algorithm in this file. | 3D:L158-L159 |

Category coverage: channel capabilities ✓; BSP workflow ✓; checkpoint data ✓; config/memory limits ✓; events ✓; concurrency/timeouts ✓; recovery/replay ✓; determinism ✓; deferred amendments ✓. No end-user UI; operator-facing observability is SSB + metrics.

### 3G — hierarchical plan management

Source: `docs/specs/guide/steps/phase-3G.txt` (463/463 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3G-E001 | E/STATUS | Two-artifact plans: skeleton (2–4k tokens, always Slot 3) + per-phase elaboration on demand. Contract extraction + coherence enforcement. Prerequisites: 3D; 3E required for 3G.4 only. | 3G:L1-L16 |
| P3G-E002 | E/DEP | Dependency-graph note: implementation-guide shows 3E∥3G from 3D is wrong for 3G.4; recommended partial parallelism with 3G.4 gated on 3E. | 3G:L6-L9 |
| P3G-E003 | E/DATA | PlanSkeleton: pipeline_id, phases IndexMap ordered, global_constraints, version u32, created_at/updated_at (UNGROUNDED timestamps via ClockProvider). | 3G:L37-L44 |
| P3G-E004 | E/DATA | PhaseEntry: phase_id, goal max 120 chars, inputs/outputs ContractRef, constraints, depends_on/depended_by, version, output_type Option, visual_check bool default false. | 3G:L46-L60 |
| P3G-E005 | E/CFG | visual_check=true requires MediaRenderAdapter for output_type; output_type=None forces visual_check=false at validation. | 3G:L58-L60 |
| P3G-E006 | E/DATA | ContractRef { name, type_signature, invariants }. PhaseDetail { phase_id, skeleton_version, contracts, implemented_contracts, declared_interfaces, content, token_count exact }. ExtractedContracts { input_contracts, output_contracts, shared_constraints }. | 3G:L62-L82 |
| P3G-E007 | E/DATA | Storage dual-path TOML authoritative: `.elai_cc/pipelines/{id}/skeleton.toml` and `phases/{phase_id}.toml`. SSB action_type='plan_update' every write. No plan_phases table; query SSB JSON filter (orchestrator.txt L250 is conceptual shorthand). | 3G:L84-L93, L323-L324 |
| P3G-E008 | E/CFG | Skeleton token strategy: estimate for allocation; exact token_count for final write; hard reject SkeletonBudgetExceeded if exact >4096. Safety margin: estimate >3600 triggers exact check. | 3G:L108-L116, L207-L211 |
| P3G-E009 | E/FLOW | Contract extraction before elaborating X: (1) union dep.outputs as inputs; (2) union depended_by.inputs as outputs; (3) global_constraints + phase.constraints. Model elaborates against contracts not full plan text. | 3G:L118-L128, L244-L248 |
| P3G-E010 | E/SEC | Coherence after elaboration: (1) implemented_contracts superset extracted; (2) declared_interfaces subset skeleton interfaces (structured set compare, not free text); (3) contract change increments phase.version AND skeleton.version. Violation → auditor BLOCK finding. | 3G:L131-L144, L307-L345 |
| P3G-E011 | E/EVENT | stub_reference SSB when skeleton phase lacks PhaseDetail file: payload {phase_id, skeleton_version}. Superseded on later detail write (append-only, no delete). | 3G:L146-L154, L213-L213, L461-L461 |
| P3G-E012 | E/FLOW | Dependency propagation: on contract version change, BFS depended_by; flag needs_review via plan_update payload {phase_id, reason:dependency_contract_changed, source_phase}. Must re-elaborate before execution. | 3G:L157-L164, L357-L361 |
| P3G-E013 | E/FAIL | Cycle detection on skeleton write via Kahn's algorithm O(V+E); reject CycleDetected. | 3G:L165-L167, L362-L362, L376-L376 |
| P3G-E014 | E/CAP | PlanSkeleton::split_phase(phase_id, split_points) with preserve_contracts bool default true: first sub-phase gets inputs, last gets outputs, internal contracts between; max split depth 2 (matches 4G SPLIT_TASK). | 3G:L365-L380 |
| P3G-E015 | E/FLOW | load_phase_context: skeleton always + active detail + extracted contracts + one-hop adjacent; fit Slot 3; overflow → ReducedGranularity (skeleton+contracts only); skeleton alone > budget → SlotBudgetExceeded. skeleton_version mismatch → staleness re-extraction. | 3G:L271-L301 |
| P3G-E016 | E/INT | coherence_check is library invoked by auditor agent via IPC ToolRequest, not standalone component. Findings action_type='finding'. | 3G:L313-L328 |
| P3G-E017 | E/FAIL | Empty declared_interfaces → coherence_check Err (elaboration must self-declare). Version check always reads current file/SSB, never stale cache. | 3G:L337-L341 |
| P3G-E018 | E/DET | Version: phase.version +=1 on contract field change; skeleton.version +=1 on any phase version change; checked_add on u32 overflow → Err. | 3G:L363-L364, L384-L386 |
| P3G-E019 | E/TEST | 3G.7 full lifecycle 5-phase DAG; 100× identical results with ClockProvider injected; unique pipeline_id EntropyProvider; no Instant::now/SystemTime::now in plan_skeleton. | 3G:L398-L434 |
| P3G-E020 | E/OBS | Measure: skeleton tokens, extraction time (<10ms/50-phase target), coherence time, propagation diamond, SSB plan_update count, compile delta, test count, LOC, coverage. | 3G:L441-L450 |
| P3G-E021 | E/TEST | Exit criteria: budget, extraction algorithm match, transitive needs_review, 3 coherence types, BLOCK findings, cycles, monotonic versions, plan_update audit, stub_reference, determinism, contract tests for PlanSkeleton/PhaseDetail/CoherenceChecker. | 3G:L452-L464 |
| P3G-E022 | E/SURFACE | Skeleton TOML format example under [phase.3B] with goal/inputs/outputs/constraints/depends_on/depended_by/version. | 3G:L95-L105 |
| P3G-E023 | E/FAIL | SkeletonBudgetExceeded never silently truncates. PhaseNotFound / ContractConflict on extraction errors. | 3G:L210-L211, L255-L262 |
| P3G-E024 | E/CONC | R26 version monotonicity relies on SSB single-writer; concurrent multi-session skeleton edits are sequentialized by single-writer channel. | 3G:L30-L32 |
| P3G-A001 | A/DATA | SQL plan_phases view/table as amendment requires user confirmation; interim is SSB filter only. | 3G:L88-L93 |
| P3G-A002 | A/CFG | Empty type_signature yields validation warning not error (some contracts textual) — exact warning surface unspecified. | 3G:L192-L193 |
| P3G-A003 | A/TEST | Coherence check 1 prose also says “corresponding implementation reference in elaboration text” while 3G.5 requires structured implemented_contracts match — dual wording. | 3G:L134-L136, L320-L322 |

Category coverage: plan types/surfaces (TOML files, SSB) ✓; workflows ✓; config/token budgets ✓; auditor integration ✓; failures/cycles ✓; determinism ✓; split/depth limits ✓. No GUI panel named; Slot 3 is context surface for agents.

### 4D-ORACLE — Oracle Mode /research cache

Source: `docs/specs/guide/steps/phase-4D-ORACLE.txt` (137/137 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4DOR-E001 | E/STATUS | SHIPPED in C-30 (SCHEMA_VERSION 61→62). Spec record for OracleAnswerCache; cross-ref C-30-decisions DECISION-8/9/13, NOTE-A. | 4DOR:L1-L7 |
| P4DOR-E002 | E/CAP | Cache /research answers in-process per (query × project) so repeated questions within freshness window skip full ConceptDecomposition + WebResearch. | 4DOR:L9-L11 |
| P4DOR-E003 | E/DATA | CacheKey: query_hash blake3(query) [u8;32], project_hash blake3(project_root_canonical_path) [u8;32]. | 4DOR:L32-L34, L65-L66 |
| P4DOR-E004 | E/DATA | CachedAnswer: answer String, cached_at SystemTime, source_tier SourceTier (T1Authoritative|T2Strong|T3Weak|T4Rejected). | 4DOR:L36-L39 |
| P4DOR-E005 | E/CAP | OracleAnswerCache: in-memory LRU; max_entries from elai_cc.toml. | 4DOR:L41-L42 |
| P4DOR-E006 | E/CFG | TTL: T1 2_592_000s (30d); T2 604_800s (7d); T3 86_400s (1d); T4 0s never cached (insert early-returns false). | 4DOR:L44-L50 |
| P4DOR-E007 | E/CFG | Confidence mapping: High→T1, Medium→T2, Low→T3, Unknown→T4Rejected (no cache). Field on ResearchAnswer is `confidence` NOT `answer_confidence` (DECISION-8). | 4DOR:L52-L59 |
| P4DOR-E008 | E/FLOW | Miss path: compute key → run_research_pipeline → tier from confidence → insert if ≠T4 → emit oracle_cache_miss {query_hash_short, source_tier}. | 4DOR:L64-L70 |
| P4DOR-E009 | E/FLOW | Hit path: lookup → verify cached_at+ttl > now else miss/evict → return cached → emit oracle_cache_hit {cached_age_secs, source_tier}. | 4DOR:L72-L77 |
| P4DOR-E010 | E/CFG | LRU: when len > max_entries evict LRU. max_entries from elai_cc.toml [research.oracle] max_cached_answers default 128. | 4DOR:L79-L82 |
| P4DOR-E011 | E/INT | LiveCli field oracle_cache: Option<Arc<OracleAnswerCache>> default None; lazy init on first /research; all three construction sites default None. | 4DOR:L84-L87 |
| P4DOR-E012 | E/EVENT | SSB types oracle_cache_hit {cached_age_secs, source_tier}, oracle_cache_miss {source_tier}; SCHEMA 62 9-tuple atomic bump. | 4DOR:L89-L95 |
| P4DOR-E013 | E/SURFACE | reachable_from: /research and /ask slash commands (commands.rs Research / Ask variants). | 4DOR:L108-L109 |
| P4DOR-E014 | E/LIMIT | stub_ban: T4Rejected MUST early-return on insert; hashes MUST use blake3 (no blake2/sha2). | 4DOR:L110-L113 |
| P4DOR-E015 | E/OBS | Observable: `elai /research "what does B-37 ship?"` twice within 60s → miss then hit with cached_age_secs>0. | 4DOR:L114-L117 |
| P4DOR-E016 | E/TEST | DETERMINISTIC_CHECK: OracleAnswerCache present; TTL constants; OracleCacheHit/Miss; blake3; cargo test c30_surgical_oracle_contract. | 4DOR:L119-L137 |
| P4DOR-E017 | E/INT | consumes: run_research_pipeline (live_cli.rs:2578), AnswerConfidence, SourceTier. Module oracle_cache.rs. | 4DOR:L13-L22, L104-L107 |
| P4DOR-A001 | A/CFG | “normalised query” hashing is stated without specifying normalization algorithm (trim/case/whitespace). | 4DOR:L65-L66 |
| P4DOR-A002 | A/STATE | Expired entry “treat as miss (LRU evicts on next insert)” leaves active expired entry until next insert — not proactive purge. | 4DOR:L74-L75 |

Category coverage: /research and /ask user surfaces ✓; cache capability ✓; config TTLs/max_entries ✓; SSB events ✓; security hash/T4 ban ✓; tests ✓. No separate dashboard panel named.

### 4G — autonomous mode enablement

Source: `docs/specs/guide/steps/phase-4G.txt` (917/917 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4G-E001 | E/STATUS | Enable AUTONOMOUS auto-advance on validated INTERACTIVE pipeline. No new FSM states — only gate evaluation behavior. Prerequisites: Phase 3 interactive validated + Phase 4A–4F. | 4G:L1-L12 |
| P4G-E002 | E/CAP | AutonomousGate wraps Box<dyn InteractiveGate> + AutoAdvanceEvaluator + ExecutionMode + EscalationPolicy + PlanDispatchTable. Implements InteractiveGate. Interactive: delegate inner zero change. Autonomous: evidence first; human only on HIGH risk or missing/invalid evidence. | 4G:L38-L54, L312-L314 |
| P4G-E003 | E/SEC | Within EXECUTING, dangerous tool calls reviewed by auditor agent not human (arch-overview S6:L220) — in-state path distinct from gate transitions. | 4G:L48-L50, L433-L435 |
| P4G-E004 | E/CAP | Plan-dispatch states (TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING) use PlanDispatchTable from PipelineSpec; meta-dispatch (PLANNING, CONFIGURING) use static MetaDispatchTable. | 4G:L51-L54 |
| P4G-E005 | E/FLOW | Evidence map (S13): IDLE→PLANNING knowledge readiness ≠ UNAVAILABLE; PLANNING→TESTING_FIRST zero UNGROUNDED; TESTING_FIRST→EXECUTING cargo test exit≠0 AND no error[E (heuristic); EXECUTING→AUDITING ≥1 FAIL→PASS flip; AUDITING→MEMORIZING zero BLOCK; MEMORIZING→next DKG updated + changelog. | 4G:L56-L71, L310-L311 |
| P4G-E006 | E/DATA | EvidenceCheck { artifact_path, check_fn pure deterministic, description }. EvidenceVerdict Pass|Fail{reason}|Missing. | 4G:L73-L83 |
| P4G-E007 | E/CFG | EscalationPolicy defaults: anti_spiral_threshold=3 (pin L180 not L173>3); high_risk_pauses_in_both_modes=true; replan_major_always_human=true; max_auto_advances_per_phase=10. | 4G:L85-L98, L411-L424 |
| P4G-E008 | E/EVENT | EscalationTrigger: AntiSpiral, HighRisk, ReplanMajor, CircuitBreaker, SubstituteModelFailed, SplitTaskFailed. | 4G:L100-L107 |
| P4G-E009 | E/CAP | SplitTaskPlan: parent_phase, sub_tasks max 4, MergeStrategy::Sequential only, max_depth 2, current_depth. SubTask {task_id, scope, context_budget, depends_on DAG}. | 4G:L109-L133, L318-L322, L418-L422 |
| P4G-E010 | E/LIMIT | S10 branching pick-one is incompatible with SPLIT_TASK (needs ALL results). Parallel merge deferred. Sub-tasks sequential, write parent SSB directly. | 4G:L124-L132, L318-L322, L430-L432 |
| P4G-E011 | E/CAP | ModelFallback iterates fallback_chain from elai_cc.toml [agents.{role}.fallback_models]; requires new ModelPool::can_fit_llm; swap protocol unload/load/health; none fit → SubstituteModelFailed. SLO tier mismatch WARNING allowed. | 4G:L135-L151, L323-L326, L438-L439 |
| P4G-E012 | E/SURFACE | Mode switch CLI: `/mode autonomous` or `/mode interactive`. Takes effect at next gate evaluation, not mid-phase. Mode per-session. | 4G:L327-L333, L245-L248 |
| P4G-E013 | E/SEC | Evidence provenance: expected_producer role; join SSB seq→agent_id→AgentPool::get_role (new API). Prevents forged audit_report. | 4G:L286-L287, L334-L341, L425-L426 |
| P4G-E014 | E/EVENT | SSB action_type 'escalation' must be added to CHECK constraint (or use checkpoint payload — preferred add type). | 4G:L235-L239, L436-L437 |
| P4G-E015 | E/TEST | Base: 28 contract tests (6 auto-advance + 6 missing + 3 risk + 5 escalation + 4 fallback + 3 mode + 1 regression) + 8 adversarial = 36. TDD red then implement. | 4G:L210-L302, L342-L364 |
| P4G-E016 | E/CFG | EscalationPolicy::new enforces min=1 on max_auto_advances; SplitTaskPlan::new clamps max_depth to 2; empty fallback_chain → immediate ESCALATE no panic. | 4G:L260-L265 |
| P4G-E017 | E/FLOW | ReplanRequest MINOR → auditor; MAJOR → human both modes. Max 1 ReplanRequest per agent per phase. | 4G:L231-L232, L290-L291, L315-L317, L414-L415 |
| P4G-E018 | E/OBS | Metrics `.elai_cc/metrics/phase_4G.toml`: auto-advance accuracy 100% on deterministic scenarios; false escalation rate; mode switch latency; SPLIT_TASK overhead; SUBSTITUTE_MODEL swap time; E2E AUTONOMOUS vs INTERACTIVE (with 5s simulated human latency); gate overhead target <1ms (adversarial absolute <0.5ms in 4G.2). | 4G:L366-L396, L200-L205 |
| P4G-E019 | E/CAP | TreeAutoAdvancePolicy: baby_step_auto/step_integration_auto/phase_integration_auto default true; concept_integration_auto default false always human; max_auto_baby_steps_per_concept=50; bubble risk Baby/Step LOW, Phase MEDIUM, Concept HIGH. | 4G:L469-L512, L516-L551 |
| P4G-E020 | E/FLOW | Per-baby-step gates: TESTING→EXECUTING, EXECUTING→AUDITING, AUDITING→ACCEPTANCE, ACCEPTANCE→NEXT. BubbleUp: step auto AUTONOMOUS; phase pause MEDIUM; concept always human. Integration fail baby auto-retry once then bubble. | 4G:L480-L498 |
| P4G-E021 | E/CAP | StagnationPattern: Spinning (blake3 last 3 outputs or errors identical conf 1.0), Oscillation (period-2 over 4 conf 0.9), NoDrift (|Δdrift|<0.01 last 3), DiminishingReturns (improvements <0.01 and monotonic conf 0.8/0.6). Stateless detector. | 4G:L554-L658 |
| P4G-E022 | E/EVENT | SSB: stagnation_spinning_detected, stagnation_oscillation_detected, stagnation_no_drift_detected, stagnation_diminishing_returns_detected; lateral_thinking_activated/succeeded/exhausted; watchdog_timeout; goal_drift_warning/block. | 4G:L654-L656, L682-L687, L717-L719, L745-L752 |
| P4G-E023 | E/CAP | ThinkingPersona affinity: Hacker→Spinning; Researcher→NoDrift/DiminishingReturns; Simplifier→DiminishingReturns/Oscillation; Architect→Oscillation/NoDrift; Contrarian→ALL. suggest_persona first match; fail exclude retry; all exhausted → INTERACTIVE pause / AUTONOMOUS HALT. | 4G:L590-L688 |
| P4G-E024 | E/SURFACE | Persona agent files: `.claude/agents/persona-{hacker,researcher,simplifier,architect,contrarian}.md`. | 4G:L624-L631, L861-L864 |
| P4G-E025 | E/CONC | Parallel fan-out optional: when Contrarian-affinity AND parallel_dispatch=true, spawn all 5 Tier-C with FRESH context (no cross-persona outputs); first success wins cancel rest. | 4G:L690-L697 |
| P4G-E026 | E/CAP | GenerationWatchdog per baby step: idle 60s (no SSB), no_material_progress 180s, safety 600s wall. Material types: phase_transition, evidence_gate_evaluated, integration_checkpoint_success, bubble_up_typed, agent_lifecycle state change. Exclude heartbeat/status_echo/log_entry/grc_observe status. workflow_fingerprint dedup freezes progress gaming. Timeout → cancel + BubbleUpHandler. | 4G:L605-L719 |
| P4G-E027 | E/CAP | GoalDriftCanary: Jaccard 1−|∩|/|∪| on alphanumeric whitespace tokens. drift<0.5 Pass; 0.5–0.7 Warning continue; ≥0.7 Block before audit (save LLM), RetryBabyStep max 1 then escalate. Skip if tokens <20 or empty goal. constraint_drift/ontology_drift NOT ported. | 4G:L721-L873 |
| P4G-E028 | E/SURFACE | INTERACTIVE persona exhaustion renders InterventionCard with all 5 personas and failure reasons. | 4G:L810-L813 |
| P4G-E029 | E/LIMIT | Existing 3× GRC anti-spiral SUBSUMED by Spinning (threshold=3), not removed. | 4G:L451-L452, L839-L841 |
| P4G-E030 | E/INT | Every routing call site wires ssb, fsm, clock through single RuntimeClient; no None after construction. | 4G:L466-L466 |
| P4G-E031 | E/CAP | Addendum shipped: AutoAdvanceEvaluator reasoning-exhaustion 7th check when AutoAdvanceStats.evidence_rounds ≥ evidence_rounds_max from [autonomy.auto_advance] → ContinueInteractive or Halt. | 4G:L886-L898 |
| P4G-E032 | E/CAP | Addendum CloudSpendTracker: 50% budget → cloud_spend_warning; 20% remaining → cloud_spend_critical; below 20% INTERACTIVE EscalationRequired / AUTONOMOUS halt cloud_spend_exhausted. Budget elai_cc.toml [cloud.session_budget_usd] default 5.00 USD. | 4G:L900-L914 |
| P4G-E033 | E/SURFACE | Runtime-selection UX (summary-aligned exit): elai --local sidecar, --hybrid per-role, default cloud; /model live change; [agents.{role}.backend]=local|api|auto (named in MASTER; tree section focuses tree policy). | 4G:L1-L12, exit context via MASTER 4G runtime lines (orientation); source body focuses gates — see P4G-I001 |
| P4G-E034 | E/TEST | 4G.7 ≥9 tree_auto_advance tests; 4G.9 stagnation/persona/watchdog/goal_drift tests with adversarial cases (Oscillation≠Spinning, affinity, heartbeat non-material, fingerprint gaming, InterventionCard, Jaccard boundaries, max 1 block-retry). | 4G:L516-L534, L775-L830 |
| P4G-E035 | E/DET | AutoAdvanceEvaluator pure functions over artifact bytes; no LlmProvider dependency. Gate reads mode under Arc<RwLock<SessionState>>. | 4G:L257-L259, L329-L330 |
| P4G-E036 | E/FAIL | DEGRADED knowledge: IDLE→PLANNING auto-advances with warning (not blocked). | 4G:L280-L281 |
| P4G-C001 | C/CFG | Anti-spiral threshold: orchestrator.txt L180 “3x” vs L173 “same_step_count > 3”; phase pins L180 (escalate on 3rd). | 4G:L86-L89, L411-L413 |
| P4G-C002 | C/SEC | HIGH risk: L227 pause both modes for gates; L220 auditor-only for in-EXECUTING tools — both true, different paths. | 4G:L90-L94, L433-L435 |
| P4G-I001 | I/SURFACE | Runtime flags --local/--hybrid and /model are asserted by MASTER phase-index summary for 4G; this source file’s body does not re-list them after the GOAL/read list — treat MASTER as orientation unless/until codified in addendum or providers wiring. | 4G:L14-L27; MASTER 4G L871-L873 |
| P4G-A001 | A/INT | SessionState struct “not yet defined in any spec”; 4G adds execution_mode to whatever exists. | 4G:L331-L333 |
| P4G-A002 | A/TEST | TESTING_FIRST heuristic false positives (setup panics) accepted; AUDITING catches — no tighter acceptance metric. | 4G:L62-L67, L442-L444 |
| P4G-A003 | A/CFG | parallel_dispatch flag location on EscalationPolicy for fan-out is named in extension but not given default or TOML key. | 4G:L690-L692 |
| P4G-A004 | A/OBS | “Auto-advance accuracy 100%” requires 10 deterministic scenarios with ≥3 adversarial; exact scenario catalog not listed. | 4G:L371-L372, L384-L387 |

Category coverage: modes/CLI/surfaces ✓; gate workflows ✓; escalation/split/model fallback ✓; tree autonomy ✓; stagnation/personas/watchdog/canary ✓; cloud spend ✓; events/SSB ✓; config flags ✓; determinism/security ✓; deferred/new APIs ✓.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE. | P1A-E008, P1A-E012, P1A-E027; P1D-E006, P1D-E022 |
| IX-002 | E | 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4). | P1A-E009, P1A-E010, P1A-E043; P1D-E001 |
| IX-003 | E | 1D density-guard fallback is insertion-order, not BM25, because BM25 rerank is upstream in the query pipeline. | P1D-E011 |
| IX-004 | E | 3D requires 3B SSB + 3C agents; upgrades 3C spawn_parallel stub to true JoinSet parallelism; 3E/3F consume ChannelRegistry. | P3D-E001, P3D-E019, P3D-E013 |
| IX-005 | E | 3D checkpoint+SSB replay composes with 3C agent checkpoints under `.elai_cc/sessions/{sid}/`; UntrackedValue deliberately absent from both. | P3D-E015, P3D-E018, P3D-E011 |
| IX-006 | E | 3G.4 requires 3E Slot 3 ContextBudget; skeleton always loads in PhaseSpec slot; overflow uses 3E ReducedGranularity. | P3G-E001, P3G-E015 |
| IX-007 | E | 3G coherence is invoked by 3C auditor via IPC; findings use action_type finding; plan mutations use plan_update/stub_reference. | P3G-E010, P3G-E011, P3G-E016 |
| IX-008 | E | 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing. | P3G-E014; P4G-E009 |
| IX-009 | E | 4G SPLIT_TASK is the Terminal path consumer of 3E context-budget exhaustion; sequential merge avoids 3H/S10 pick-one branches. | P4G-E009, P4G-E010 |
| IX-010 | E | 4G SUBSTITUTE_MODEL is the autonomous completion of 3C on_max_restarts policy (interactive escalates). | P4G-E011 |
| IX-011 | E | 4G wraps 3A InteractiveGate without new FSM states; evidence gates from orchestrator S13; risk levels from arch-overview S6. | P4G-E001, P4G-E002, P4G-E005 |
| IX-012 | E | 4G tree auto-advance integrates 4E TreeNavigator + BubbleUpHandler; concept-level always human. | P4G-E019, P4G-E020 |
| IX-013 | E | 4G.9 Spinning subsumes prior 3× GRC anti-spiral; personas spawn via 4A Tier-C isolation. | P4G-E021, P4G-E023, P4G-E029 |
| IX-014 | E | 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss. | P4DOR-E001, P4DOR-E002, P4DOR-E013 |
| IX-015 | E | Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys. | P1A-E013, P1A-E035; P4DOR-E014; P4G-E021 |
| IX-016 | E | SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*. | P3D-E017, P3D-E023; P3G-E007, P3G-E011; P4DOR-E012; P4G-E014, P4G-E022, P4G-E032 |
| IX-017 | I | 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks. | P1A-E039; P1D-E009; P4G-E005 |
| IX-018 | E | 4G circuit breakers operate at two grains: phase auto-advances (default 10) and tree baby steps per concept (default 50). | P4G-E007, P4G-E019 |
| IX-019 | E | Goal-drift Block and watchdog timeouts both hand off to 4E BubbleUpHandler rather than inventing a third recovery owner. | P4G-E026, P4G-E027 |
| IX-020 | E | 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step. | P3D-E005; P4G-E026 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict | R20 ID collision: PageRank density vs phase-0 entropy quarantine; renumber out of scope for 1D. | P1D-E003 |
| AC-002 | conflict | Anti-spiral 3× vs same_step_count>3; 4G pins 3rd-failure interpretation. | P4G-C001 |
| AC-003 | qualification | HIGH-risk human pause vs auditor tool review are different layers; both remain binding. | P4G-C002 |
| AC-004 | ambiguity | 1A partial parse on syntax errors lacks completeness floor. | P1A-A003 |
| AC-005 | ambiguity | 1D “mentioned in task” personalization matching algorithm unspecified. | P1D-A002 |
| AC-006 | ambiguity | 3D evidence_gate_passes sufficiency predicate incomplete in phase file. | P3D-A003 |
| AC-007 | dependency | 3D ChannelRegistry + memory_monitoring + Topic(false) + checkpoint placement need orchestrator.txt amendments before/with implement. | P3D-E007, P3D-A002 |
| AC-008 | ambiguity | 3G plan_phases SQL shorthand vs SSB-only interim; view amendment needs user confirmation. | P3G-A001 |
| AC-009 | ambiguity | 3G coherence “text reference” vs structured implemented_contracts dual wording. | P3G-A003 |
| AC-010 | ambiguity | 4DOR query normalization for blake3 key undefined. | P4DOR-A001 |
| AC-011 | ambiguity | 4G SessionState ownership and definition still open; execution_mode attachment site TBD. | P4G-A001 |
| AC-012 | dependency | 4G requires new APIs: can_fit_llm, AgentPool::get_role, SSB escalation type — all require post-approval spec propagation. | P4G-E011, P4G-E013, P4G-E014 |
| AC-013 | ambiguity | parallel_dispatch default/TOML key for persona fan-out unspecified. | P4G-A003 |
| AC-014 | evidence | 4G runtime UX flags appear in MASTER summary more explicitly than in phase-4G body; do not over-claim source atoms for --local/--hybrid without addendum. | P4G-I001 |
| AC-015 | qualification | blake2 (1A file hash) vs blake3 (4DOR/4G) is intentional layer split, not a single-stack rule — document when consolidating hash policy. | IX-015 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1A-001 | **ADD** full COMPLIANCE SCOPE split of knowledge.txt Sec 8 items 1+5 vs deferred 2–4 and orphan-invariant block until 1B. | MASTER:L44-L55 | P1A-E009, P1A-E010, P1A-E043 |
| SD-1A-002 | **ADD** readiness split dkg_readiness vs full KnowledgeEngine.readiness_check (Phase 1I). | MASTER:L48-L51 | P1A-E016, P1A-E017 |
| SD-1A-003 | **ADD** complete schema tables dkg_files/dkg_meta, directed edge semantics, WIRING_CONTRACT symbols, index build SSB symbol_graph_updated, and 12 metric keys. | MASTER:L50-L55 | P1A-E022–P1A-E027, P1A-E038–P1A-E041 |
| SD-1A-004 | **ADD** TDD red gate (≥20 tests, ≥10 fail), object-safety LanguageGrammar, and dynamic KNOW_PATH resolution. | MASTER:L48-L53 | P1A-E002, P1A-E011, P1A-E034 |
| SD-1A-005 | **FLAG** ungrounded <15s build claim and designed-not-spec SQLite schema. | MASTER:L44-L55 | P1A-E006, P1A-E007 |

### 1D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1D-001 | **ADD** full T1–T18/P1–P3/A1–A3 catalog and make_snapshot helper requirements. | MASTER:L93-L93 | P1D-E010–P1D-E014 |
| SD-1D-002 | **ADD** exact 13-item special-file list and config-driven deferral; qualify MASTER “such as” wording. | MASTER:L96-L96 | P1D-E016, P1D-A001 |
| SD-1D-003 | **ADD** R20 ID collision note and Layer 3.5 vs Layer 4 correction. | MASTER:L85-L97 | P1D-E003, P1D-C001 |
| SD-1D-004 | **ADD** aider not-adopted items (specificity/private/generic penalties) and 7-metric phase_1D.toml fields. | MASTER:L88-L97 | P1D-E019, P1D-E020 |
| SD-1D-005 | **ADD** zero-budget T13, NaN/Inf clamp, and initial binary-search guess max_tokens/25. | MASTER:L94-L96 | P1D-E015, P1D-E017 |

### 3D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3D-001 | **ADD** four pre-implement orchestrator.txt amendments and corrected loop rationale (checkpoint after gate). | MASTER:L284-L288 | P3D-E007, P3D-E014 |
| SD-3D-002 | **ADD** ephemeral clear SSB replay algorithm and Topic(false) two-phase snapshot+clear definition (spec gap). | MASTER:L281-L282, L287 | P3D-E009, P3D-E010, P3D-E018 |
| SD-3D-003 | **ADD** DeepSizeOf/size_of_val ban, 10MB/2× force-clear, and isolation_metric separation from memory_monitoring. | MASTER:L283-L287 | P3D-E004, P3D-E017, P3D-E022 |
| SD-3D-004 | **ADD** wall-time concurrency proof, VRAM max_concurrent formula, and simulation-only byte-identical determinism caveat for live LLMs. | MASTER:L277-L288 | P3D-E003, P3D-E019 |
| SD-3D-005 | **ADD** baseline measurement section as non-exit UNGROUNDED estimates. | MASTER:L276-L288 | P3D-E020 |

### 3G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3G-001 | **ADD** 3E dependency correction for 3G.4 and partial-parallel recommendation. | MASTER:L321-L332 | P3G-E001, P3G-E002 |
| SD-3G-002 | **ADD** visual_check/output_type media adapter rules and ClockProvider timestamp ungrounded fields. | MASTER:L325-L325 | P3G-E004, P3G-E005 |
| SD-3G-003 | **ADD** 3600 safety margin, SkeletonBudgetExceeded, ContractConflict, and empty declared_interfaces fail-closed. | MASTER:L327-L330 | P3G-E008, P3G-E017, P3G-E023 |
| SD-3G-004 | **ADD** split_phase preserve_contracts algorithm details and u32 checked_add overflow Err. | MASTER:L331-L331 | P3G-E014, P3G-E018 |
| SD-3G-005 | **ADD** 100-run determinism with EntropyProvider pipeline_id and ban wall clocks in plan_skeleton. | MASTER:L332-L332 | P3G-E019 |

### 4D-ORACLE deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4DOR-001 | **ADD** entire Oracle cache feature set: MASTER lists phase-4D-ORACLE only as unindexed file to inspect, without extracting atoms. | MASTER:L655-L655, L714-L714 | P4DOR-E001–P4DOR-E017 |
| SD-4DOR-002 | **ADD** TTL/confidence mapping, LRU default 128, /research+/ask reachability, blake3 ban, C-30 SCHEMA 62 events. | MASTER:L655 | P4DOR-E006–P4DOR-E015 |
| SD-4DOR-003 | **ADD** LiveCli lazy Option init and T4Rejected early-return stub_ban. | MASTER:L655 | P4DOR-E011, P4DOR-E014 |
| SD-4DOR-004 | **FLAG** query normalization and expired-entry lazy eviction ambiguities absent from any summary. | MASTER:L655 | P4DOR-A001, P4DOR-A002 |

### 4G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4G-001 | **ADD** full EvidenceCheck/EvidenceVerdict types and TESTING_FIRST cargo heuristic caveats. | MASTER:L471-L471 | P4G-E005, P4G-E006, P4G-A002 |
| SD-4G-002 | **ADD** 28+8 test catalog, InterventionCard exhaustion UI, and mode-switch TOCTOU RwLock rules. | MASTER:L476-L476 | P4G-E015, P4G-E028, P4G-E035 |
| SD-4G-003 | **ADD** complete 4G.9 stagnation/persona/watchdog/goal-drift port (thresholds, SSB types, affinity, fingerprint, Jaccard bands). | MASTER:L477-L477 (compressed tree only) | P4G-E021–P4G-E027, P4G-E034 |
| SD-4G-004 | **ADD** shipped addendum: reasoning-exhaustion 7th auto-advance check and CloudSpendTracker budget policy (default $5). | MASTER:L467-L478 | P4G-E031, P4G-E032 |
| SD-4G-005 | **ADD** required new APIs can_fit_llm, get_role, SSB escalation type and SessionState open. | MASTER:L475-L475 | P4G-E011, P4G-E013, P4G-E014, P4G-A001 |
| SD-4G-006 | **QUALIFY** MASTER runtime UX (--local/--hybrid, /model, backend config): more explicit in summary than phase body; keep as orientation/I. | MASTER:L478-L478 | P4G-I001, AC-014 |
| SD-4G-007 | **ADD** anti-spiral pin L180 and HIGH-risk dual-path clarification vs auditor tool review. | MASTER:L472-L472 | P4G-C001, P4G-C002 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **6/6 files**, **2,750/2,750 logical lines**, **178,707/178,707 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **189 unique atoms** = **166 explicit**, **002 inferred**, **018 ambiguous**, **003 conflicting/corrective**.
- Per-file atom counts: 1A **047**; 1D **027**; 3D **026**; 3G **027**; 4D-ORACLE **019**; 4G **043**. Sum **189**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **031** (`SD-*`). Ambiguity/conflict register: **15** (`AC-001`–`AC-015`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1A/P1D/P3D/P3G/P4DOR/P4G-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: 1A index-build CLI path; 1D repo-map API (no panel); 3D SSB/metrics (no GUI); 3G skeleton TOML + Slot 3; 4DOR `/research` `/ask` + LRU cache; 4G `/mode`, InterventionCard, persona files, cloud spend rows, and (orientation) runtime flags — each captured as explicit atoms where the source states them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta/I records and are not elevated to explicit requirements.
- Manifest-boundary audit: the only original sources opened were the six exact DEEP-PHASE-002 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0394`; `output_bytes=058924`.
- Canonical normalized output SHA-256: 14d979ff890fa46e6d529a837fc01aa0f0bf6a442fc472bd0c4c82e4888c8485.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
