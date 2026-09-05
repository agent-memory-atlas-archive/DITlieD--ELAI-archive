# DEEP-PHASE-003 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1357-L1368`, heading `### DEEP-PHASE-003`; the next chunk starts at L1370.
- Declared chunk: family PHASE, **6 files**, **2,751 logical lines**, **182,824 bytes**, all six classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all six listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `phase-1B:Lx-Ly`, `phase-1G:Lx-Ly`, `phase-3H:Lx-Ly`, `phase-4D-SURGICAL:Lx-Ly`, `phase-4E:Lx-Ly`, and `phase-index-3A-4J:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 448 | 28,063 | d5ed0a32e83f7ca32ed95f9bd90fea7bf62fbd10d88c820963d26d3ce5e623a2 | docs/specs/guide/steps/phase-1B.txt | exact |
| numbered/index original | 338 | 31,345 | 45f6e19763ec91538d7b858de0fdada6ec267e50f553dd7908ff6073485f7053 | docs/specs/guide/steps/phase-1G.txt | exact |
| numbered/index original | 521 | 33,410 | 86c9252f116e45e3ba0d0d807f8d96cf3fc1197c228588a1acdeab227b44214f | docs/specs/guide/steps/phase-3H.txt | exact |
| numbered/index original | 143 | 6,038 | 476cee5599a05897eeb1f2cbdf5b3f4338772a7d6b069db86e3461e42027879a | docs/specs/guide/steps/phase-4D-SURGICAL.txt | exact |
| numbered/index original | 911 | 53,599 | 6842d193804e86909b82f59c901063635a44ef8aad23d89c62dd31338ac560e2 | docs/specs/guide/steps/phase-4E.txt | exact |
| numbered/index original | 390 | 30,369 | f6e224a4175ba3d264ce5f7f41158b47c3c97d9238fa63d66beb6516169e2cf9 | docs/specs/guide/steps/phase-index-3A-4J.txt | exact |
| **Aggregate** | **2,751** | **182,824** | — | **6 exact paths** | **exact** |

## Per-file atomic ledgers

### phase-1B — BM25 / Tantivy full-text retrieval

Source: `docs/specs/guide/steps/phase-1B.txt` (448/448 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1B-E001 | E/STATUS | Phase 1B lives in `.elai_cc/crates/knowledge/` after Phase 1A; goal is per-project Tantivy BM25 with sub-10 ms queries under 100K docs and zero orphan text files. | phase-1B:L1-L12 |
| P1B-E002 | E/LIMIT | Binary files with null bytes in the first 8 KiB are the sole zero-orphan exception; non-UTF8 must skip with a warning, never panic. | phase-1B:L10-L12, L32-L32, L101-L102, L232-L234 |
| P1B-E003 | E/DEP | Mandatory pre-reads: knowledge.txt Layer-2/deps/storage/fallback/invariants/versioning and architecture trait boundaries. | phase-1B:L14-L21 |
| P1B-E004 | E/CFG | Pin exact tantivy crate version; record `INDEX_FORMAT_VERSION` (u32). Spec label `tantivy_v21` is project-only, not a tantivy API. | phase-1B:L24-L28, L46-L52, L388-L391 |
| P1B-E005 | E/DATA | Schema: `path` STRING\|STORED untokenized; `content` TEXT\|STORED; `line_start`/`line_end` u64 STORED; `chunk_kind` STRING\|STORED. TEXT path is forbidden (breaks `delete_term`). | phase-1B:L34-L37, L88-L97, L211-L216, L364-L367 |
| P1B-E006 | E/DATA | `BM25Chunk {path, content, line_start, line_end, kind}` and `ChunkKind::{Function,LineBlock,File}` are BM25-owned; distinct from DKG `DkgNode` (no content). Conversion reads file ranges DKG supplies. | phase-1B:L70-L82, L145-L146, L368-L371 |
| P1B-E007 | E/CAP | `Bm25Index::{new,index_file,index_directory,query,delete_file,update_file}` with `project_id` explicit first-class input; index root `{project_root}/.elai_cc/projects/{project_id}/bm25/`. | phase-1B:L83-L118, L372-L374 |
| P1B-E008 | E/DATA | `BM25Hit {path, line_start, line_end, score, chunk_kind, snippet}`; snippet is first 200 chars simple truncation, not SnippetGenerator. | phase-1B:L107-L110, L249-L250, L375-L377 |
| P1B-E009 | E/DATA | `IndexStats {files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms}`. | phase-1B:L103-L106 |
| P1B-E010 | E/FLOW | `index_file`: chunks provided → one doc each (Function); chunks=None → 50-line windows / 10-line overlap (LineBlock); files <50 lines → single File doc. Binary null-byte skip. | phase-1B:L99-L102, L219-L234 |
| P1B-E011 | E/FLOW | `index_directory` walks recursively, skips hidden `.*` and `.elai_cc/`; DKG nodes when present else line fallback; one batch `writer.commit()`; holds IndexWriter for whole batch. | phase-1B:L103-L118, L236-L244 |
| P1B-E012 | E/CONC | Single IndexWriter model: concurrent `update_file` blocks during `index_directory`; second `Bm25Index::new` for same project must Err (lock), not panic/deadlock. | phase-1B:L114-L118, L268-L269, L360-L363 |
| P1B-E013 | E/IDEM | `update_file` is atomic delete_term + add_document in one `writer.commit()`; readers never see intermediate absence; target <5 s per file. | phase-1B:L113-L114, L256-L259, L360-L363 |
| P1B-E014 | E/DEFER | 10-line overlap is UNGROUNDED for BM25 (term independence); kept only for vector-layer alignment and simplicity (~20% extra docs). | phase-1B:L224-L229, L346-L350, L410-L413 |
| P1B-E015 | E/TEST | ≥10 synthetic tempdir tests: index/query/delete/empty-miss/zero-orphan .xyz/binary skip/incremental/project isolation/chunk kinds/path STRING delete regression on `src/deep/nested/module.rs`. | phase-1B:L150-L203 |
| P1B-E016 | E/OBS | Metrics `.elai_cc/metrics/phase_1B.toml`: build/query/update latencies, orphan_files=0, tantivy_version, index_format_version, sizes/counts. | phase-1B:L287-L329 |
| P1B-E017 | E/TEST | Latency gates: query p99 <10 ms dev, <50 ms when `CI=true`; update_file <5 s; warm-up query before measure; index 69 `.elai_cc/rust/` files. | phase-1B:L288-L329, L332-L343 |
| P1B-E018 | E/INT | WIRING: produces `Bm25Index` + types; reachable from `index_cmd::run_index_build`, `IndexingService::run_full_index`, `run_local_repl`, `QueryPipeline`; trigger `cargo run -p elai-cli -- index build`; expect SSB `index_checkpoint`. | phase-1B:L427-L448 |
| P1B-E019 | E/SURFACE | User-facing CLI surface: `elai-cli index build --project-root <path>` builds the BM25 index (WIRING trigger). | phase-1B:L447-L448 |
| P1B-E020 | E/LIMIT | Circuit breaker, SearchMode, query validation, readiness, tracing, versioning scoped out to Phase 1I. | phase-1B:L423-L424 |
| P1B-E021 | E/DATA | Decisions D1–D9: null-byte binary detect; chunk_kind for downstream; explicit project_id; STRING path; BM25Chunk ownership; 200-char snippet. | phase-1B:L345-L377 |
| P1B-E022 | E/EVIDENCE | Grounding corrections: fn complete path not providers; crate path is `.elai_cc/crates/knowledge/` not rust/crates; Chunk from 1A is DkgNode not content Chunk. | phase-1B:L380-L424 |
| P1B-A001 | A/CFG | Exact pinned tantivy crate version and its `INDEX_FORMAT_VERSION` are left to implementer decision doc at execution. | phase-1B:L46-L52 |
| P1B-A002 | A/FAIL | Race when file deleted between walk and read: skip+warn required, but exact error type and IndexStats field not named. | phase-1B:L266-L267 |
| P1B-C001 | C/DATA | knowledge.txt L200 `bm25_format="tantivy_v21"` conflicts with real u32 `INDEX_FORMAT_VERSION`; phase corrects and says knowledge.txt should be updated. | phase-1B:L26-L28, L388-L391 |

Category coverage: capabilities/CLI ✓; workflows/index/query/update ✓; schemas/chunk/hit/stats ✓; flags/paths/defaults ✓; integrations/DKG/pipeline ✓; security n/a (project isolation only); failures/binary-skip ✓; determinism/single-writer ✓; observability/metrics ✓; tests/TDD ✓; constraints/1I deferral ✓. No privacy policy; no formal retry API beyond atomic update.

### phase-1G — cross-encoder reranker

Source: `docs/specs/guide/steps/phase-1G.txt` (338/338 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1G-E001 | E/STATUS | Phase 1G requires 1E pipeline; independent of 1F DROP (Ollama speculative decoding closed). | phase-1G:L1-L9 |
| P1G-E002 | E/CAP | Cross-encoder `ms-marco-MiniLM-L12-v2` after A-RAG, before token-budget fit; CPU-only; target <50 ms for 50 candidates. | phase-1G:L13-L17, L29-L34 |
| P1G-E003 | E/CFG | Raw `ort` + HuggingFace `tokenizers` (not fastembed-rs); pin exact ort RC; re-validate on stable 2.0.0. | phase-1G:L32-L33, L49-L54, L77-L78 |
| P1G-E004 | E/CFG | `RerankerConfig`: model/tokenizer/fallback L-6/quantized paths; `skip_threshold` default 10; max_seq 512; max_queue_depth 4; intra_op_threads default 1; `ORT_ENABLE_ALL`; `enable_deterministic_compute=true`. | phase-1G:L80-L91, L114-L119 |
| P1G-E005 | E/CAP | `CrossEncoderReranker` holds ort Session, Tokenizer, Semaphore, mpsc Sender, CircuitBreaker, `Arc<dyn ClockProvider>`. | phase-1G:L78-L79 |
| P1G-E006 | E/FLOW | Wire `KnowledgeEngine::rerank` between A-RAG output and token-budget fitting; skip entirely in SPARSE_ONLY and BM25_ONLY. | phase-1G:L91-L93, L187-L188 |
| P1G-E007 | E/FLOW | Pair encode batch → single `session.run()` → descending re-sort; skip when candidates ≤ skip_threshold (boundary tests 10 pass-through, 11 rerank). | phase-1G:L146-L159 |
| P1G-E008 | E/CONC | Semaphore(1) guards session; bounded mpsc capacity 4; full channel returns unranked immediately; drain/merge/offset-split/oneshot return per caller. | phase-1G:L63-L65, L151-L166 |
| P1G-E009 | E/FAIL | NaN/Inf scores must be filtered or lowest-ranked; empty candidates return empty; >512 tokens truncate. | phase-1G:L160-L169 |
| P1G-E010 | E/RECOVERY | L-12 breaker: 3 failures / 60 s window / 30 s half-open via ClockProvider; cascade lazy L-6 then skip-rerank A-RAG order. | phase-1G:L181-L202 |
| P1G-E011 | E/DEFER | L-6 intermediate tier requires `workflow/reasoning/phase-1G-spec-amendment.md`; if rejected, keep skip-only final fallback per knowledge.txt L149. | phase-1G:L38-L38, L83-L84, L180-L186, L298-L299 |
| P1G-E012 | E/EVENT | SSB `action_type='knowledge_degradation'` stage=`rerank` on breaker open/close; SSB write failure non-fatal (log, continue). | phase-1G:L188-L197 |
| P1G-E013 | E/OBS | DEBUG `StageTrace` stage `rerank` in QueryTrace; canary logs reranker latency. | phase-1G:L189-L189, L198-L198, L220-L220 |
| P1G-E014 | E/FLOW | Startup canary: embed + BM25 + rerank within 5 s; reranker failure → DEGRADED (not UNAVAILABLE) with reranker skipped. | phase-1G:L210-L228 |
| P1G-E015 | E/CFG | Start fp32; INT8 only if baseline p50 >30 ms and NDCG@10 delta <0.5; script `scripts/quantize_cross_encoder.py` → `.elai_cc/models/cross-encoder-int8.onnx`. | phase-1G:L56-L59, L253-L259 |
| P1G-E016 | E/TEST | Optimization harness: thread sweep 1/2/4/8 with ranking-order stability; skip thresholds 5/10/15/20/30 by NDCG@10 (delta >0.02); fixture `tests/fixtures/reranker_benchmark.json` (1 query, 50 code candidates, ≥10 relevant). | phase-1G:L230-L266 |
| P1G-E017 | E/OBS | Metrics `.elai_cc/metrics/phase_1G.toml`: baseline/thread_sweep/int8/threshold_sweep/final latencies, NDCG, model variant, cold start, queue wait p99 under 4 agents. | phase-1G:L242-L285, L327-L338 |
| P1G-E018 | E/LIMIT | If final p50 still >50 ms after all levers, append data to phase-1G-spec-amendment.md; do not invent a false pass. | phase-1G:L271-L273, L287-L288 |
| P1G-E019 | E/LIMIT | No unwrap/panic; injectable ClockProvider only (no Instant::now); CPU EP only (no CUDA/TensorRT). | phase-1G:L117-L123, L311-L313 |
| P1G-E020 | E/FAIL | Lazy L-6 cold-start 5–15 s acceptable during degradation (no SLA on breaker fallback path). | phase-1G:L122-L122, L208-L208 |
| P1G-E021 | E/EVIDENCE | Model sizes ~120 MB fp32 / ~30 MB int8; L-12 MRR@10 39.02 vs L-6 36.81; ort is RC (conditional pin). | phase-1G:L29-L37 |
| P1G-A001 | A/CFG | Optimal skip_threshold after sweep may replace default 10; exact production value not fixed until measurement. | phase-1G:L61-L62, L261-L266 |
| P1G-A002 | A/TEST | Single-fixture NDCG; production threshold re-evaluation deferred to Phase 1I real query logs. | phase-1G:L240-L240 |
| P1G-A003 | A/CFG | Exact pinned ort RC version number not fixed in phase body beyond rc.12 validation note. | phase-1G:L32-L33, L112-L112 |
| P1G-C001 | C/CAP | Spec L149 defines only skip-rerank fallback; phase adds L-6 tier via amendment — not yet controlling without approval. | phase-1G:L38-L38, L183-L186 |

Category coverage: capabilities/rerank ✓; workflows/batch/skip ✓; config/flags ✓; integrations/pipeline/SSB ✓; failures/breaker/degrade ✓; concurrency/queue ✓; observability/metrics/trace ✓; tests/benchmark ✓; deferred amendment ✓. No CLI command; no privacy surface.

### phase-3H — DAG session handoff and lossless trimming

Source: `docs/specs/guide/steps/phase-3H.txt` (521/521 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3H-E001 | E/STATUS | Phase 3H needs 3G PlanSkeleton, 3B SSB, 3E context budget, providers::token_count; CMV-derived DAG handoff (arxiv 2602.22402). | phase-3H:L1-L18 |
| P3H-E002 | E/FLOW | Checkpoint after every GRC step, tool call, phase transition; handoff at token budget threshold default 85%. | phase-3H:L16-L17, L447-L450 |
| P3H-E003 | E/DATA | Per-session `session_dag.db`: `snapshots` + `snapshot_context` tables; indexes session/tag/stubbed; version table for migration. | phase-3H:L45-L74, L170-L176 |
| P3H-E004 | E/DATA | Snapshot fields: snap_id UUID, parent_id, session_id, fsm_state, phase, step, findings/code_changes/tests_written msgpack blobs, commit_hashes, created_at, tag. | phase-3H:L47-L60, L84-L97 |
| P3H-E005 | E/DATA | Expanded per-message `snapshot_context` rows: seq, role CHECK system/user/assistant/tool, content BLOB, is_stubbed, stub_ssb_seq — diverges from orchestrator.txt blob schema. | phase-3H:L27-L32, L62-L70, L199-L210 |
| P3H-E006 | E/DATA | Types: SessionDag, Snapshot, TrimmedMessage, Stub{original_type,original_tokens,summary_hash,ssb_seq}, BranchRef, MergeResult::{Winner,Conflict}. | phase-3H:L78-L126 |
| P3H-E007 | E/CFG | Not WAL: default DELETE journal for single writer; PRAGMA integrity_check on open; foreign_keys=ON; EntropyProvider for UUIDs; ClockProvider for timestamps. | phase-3H:L78-L79, L151-L154, L175-L176, L188-L197 |
| P3H-E008 | E/CAP | CRUD: save_snapshot transactional; load_snapshot seq-ordered; latest_snapshot(tag?); parent_chain cycle-guarded root-to-tip. | phase-3H:L220-L252 |
| P3H-E009 | E/CAP | Three-pass trim: identify tool results >stub_threshold; stub with Stub+blake3+ssb_seq; validate SSB or un-stub; never stub user/assistant or Slots 1–3. | phase-3H:L271-L310 |
| P3H-E010 | E/CFG | `TrimConfig {stub_threshold, model_context_window, allocated_slots_tokens, output_reserve}`; target = window − allocated − reserve; stub_threshold default 500, enforced [100,1000]. | phase-3H:L272-L274, L306-L308, L316-L323 |
| P3H-E011 | E/FAIL | TrimBudgetExceeded if trimmed total still > target; DatabaseCorrupted/DatabaseNotFound/CycleDetected/BranchLimitExceeded typed errors. | phase-3H:L151-L154, L181-L186, L241-L242, L285-L285, L356-L356 |
| P3H-E012 | E/CAP | Branch API: create_branch (copy parent context, tag `branch:{name}`), list_branches, merge_branch (new trunk snapshot, abandon tags), close_branch; max 3 active; names alphanumeric+hyphen max 64. | phase-3H:L333-L379 |
| P3H-E013 | E/EVENT | Merge records SSB via existing `checkpoint` with payload event=branch_merge (or amended `branch_abandon` if chosen). | phase-3H:L33-L36, L344-L347 |
| P3H-E014 | E/RECOVERY | recover_stubs / recover_single_stub: SSB lookup + blake3 verify; StubRecoveryFailed on missing seq or hash mismatch. | phase-3H:L389-L430 |
| P3H-E015 | E/FLOW | create_handoff trims + tag=handoff; load_handoff prefers branch tip over trunk when branches exist; commit_hashes persist. | phase-3H:L437-L497 |
| P3H-E016 | E/DET | 100-run same-seed identity for snap_ids and trimming; parent chain always acyclic. | phase-3H:L474-L521 |
| P3H-E017 | E/OBS | Measure save/load <50 ms (100 msgs), trim 500 msgs <10 s, token reduction >0.3 tool-heavy, branch ops <100 ms, DAG size <10 MB /1000 snaps. | phase-3H:L500-L508 |
| P3H-E018 | E/LIMIT | Abandoned branch snapshots tagged not deleted (logical SSB exclusion assumption); physical SSB delete forbidden under append-only. | phase-3H:L37-L40, L348-L349, L370-L379 |
| P3H-A001 | A/DATA | SPEC AMENDMENT #1: per-message schema vs orchestrator blob; fallback to blob if spec not updated before implement. | phase-3H:L27-L32, L199-L210 |
| P3H-A002 | A/EVENT | SPEC AMENDMENT #2: new `branch_abandon` vs reuse `checkpoint` unresolved before 3H.4. | phase-3H:L33-L36, L344-L347 |
| P3H-A003 | A/DATA | SPEC AMENDMENT #3: "only winning SSB kept" — phase assumes logical query filter, not physical delete. | phase-3H:L37-L40, L348-L349 |
| P3H-A004 | A/STATE | MergeResult::Conflict for overlapping code_changes is UNGROUNDED optional; orchestrator says always pick winner. | phase-3H:L120-L126, L367-L369, L385-L387 |
| P3H-A005 | A/DATA | blake3 for stub summary_hash is UNGROUNDED (orchestrator silent); sha256 alternative open. | phase-3H:L324-L326 |
| P3H-A006 | A/DEP | TrimConfig needs 3E total_allocated_tokens method; PRE_GATE must verify — exact API name soft. | phase-3H:L155-L158, L268-L269 |
| P3H-C001 | C/DATA | snapshot_context design intentionally diverges from orchestrator.txt L293-294 blob layout until amendment lands. | phase-3H:L27-L32, L207-L210 |

Category coverage: capabilities/session DAG ✓; workflows/trim/branch/handoff ✓; schemas/SQLite ✓; config/thresholds ✓; events/SSB ✓; failures/recovery ✓; determinism ✓; observability ✓; tests/lifecycle ✓. No standalone CLI; operator surfaces are handoff/branch load semantics.

### phase-4D-SURGICAL — surgical decomposition for /task

Source: `docs/specs/guide/steps/phase-4D-SURGICAL.txt` (143/143 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4DS-E001 | E/STATUS | SHIPPED in C-30 (SCHEMA_VERSION 61→62); record lives in surgical_decomposer.rs; decisions C-30 DECISION-1..15. | phase-4D-SURGICAL:L4-L7, L30-L33 |
| P4DS-E002 | E/SURFACE | `/task --surgical "<prompt>"` replaces ConceptDecomposition with DKG blast-radius surgical plan; flag via `parse_task_flags` commands.rs:2414. | phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89 |
| P4DS-E003 | E/INT | live_cli intercept L2632 replaces DefaultWebResearchRunner with SurgicalDecomposer BEFORE runner construction. | phase-4D-SURGICAL:L20-L21, L88-L89, L110-L112 |
| P4DS-E004 | E/DATA | Types: SurgicalPlan, AffectedSet{files,symbols}, ComplexityClass{Simple,Medium,Complex}, RegressionGate, TestBaseline, SurgicalStep. | phase-4D-SURGICAL:L38-L44, L98-L100 |
| P4DS-E005 | E/CFG | Complexity: Simple files≤3 AND loc≤200 → cap 1; Medium (files≤8 OR loc≤500) and not Simple → cap 4; Complex otherwise → cap 8. | phase-4D-SURGICAL:L49-L51, L65-L67 |
| P4DS-E006 | E/FLOW | `SurgicalDecomposer::decompose(task,dkg,mode)`: blast-radius query_callers+dependents+data_flow depth≤MAX_C30_DEPTH=8 cycle-guarded HashSet; classify; inject RegressionGate from DKG test edges; constrain steps. | phase-4D-SURGICAL:L56-L67 |
| P4DS-E007 | E/INT | DkgQuerier gains query_callers, query_dependents, query_data_flow, query_subgraph; only DkgQuerierAdapter has real SQLite traversal (defaults empty). | phase-4D-SURGICAL:L22-L24, L101-L102, L113-L116 |
| P4DS-E008 | E/CFG | SURGICAL_LOC_WARN_THRESHOLD=500; SURGICAL_LOC_ESCALATE_THRESHOLD=1000. | phase-4D-SURGICAL:L72-L74 |
| P4DS-E009 | E/FLOW | loc>500: emit surgical_loc_warning; INTERACTIVE → ConfirmationRequired; AUTONOMOUS confirm_above_loc=500 blocks until confirm OR refuse_above_loc=1000 aborts. | phase-4D-SURGICAL:L76-L81 |
| P4DS-E010 | E/EVENT | SSB SCHEMA 62: SurgicalPlanCreated{complexity_class,blast_radius_files,total_loc_estimate,step_count}; SurgicalLocWarning{loc_estimate,threshold}. | phase-4D-SURGICAL:L27-L28, L90-L93, L105-L105 |
| P4DS-E011 | E/SURFACE | Dashboard SurgicalDrawer shows ComplexityClass + AffectedSet on plan creation. | phase-4D-SURGICAL:L122-L122 |
| P4DS-E012 | E/OBS | observable_effect: `elai /task --surgical "rename Foo to Bar in core"` → surgical_plan_created within 2 s + SurgicalDrawer update. | phase-4D-SURGICAL:L119-L122 |
| P4DS-E013 | E/LIMIT | stub_ban: step cap hard (no Simple plan with >1 step); no .unwrap() in library adapter path. | phase-4D-SURGICAL:L113-L118 |
| P4DS-E014 | E/TEST | DETERMINISTIC_CHECK: rg types/SSB/live_cli/LOC constants; cargo test c30_surgical_oracle_contract. | phase-4D-SURGICAL:L124-L143 |
| P4DS-E015 | E/DEP | Consumes ExecutionMode (context_budget.rs:865), DKG db path, EscalationPolicy autonomous confirm/refuse fields. | phase-4D-SURGICAL:L17-L18, L25-L26, L106-L109 |
| P4DS-A001 | A/FLOW | Steps exceeding cap "merged or dropped with explanatory note" — note surface/format not specified. | phase-4D-SURGICAL:L65-L67 |
| P4DS-A002 | A/SURFACE | SurgicalDrawer field layout beyond ComplexityClass+AffectedSet not specified in this file. | phase-4D-SURGICAL:L122-L122 |

Category coverage: CLI flag ✓; dashboard drawer ✓; SSB events ✓; DKG queries ✓; LOC gates/modes ✓; complexity classes ✓; contract test ✓. Privacy n/a.

### phase-4E — Pipeline Factory L2 + DecompositionTree

Source: `docs/specs/guide/steps/phase-4E.txt` (911/911 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4E-E001 | E/STATUS | Phase 4E after 4D; pure-Rust orchestrator L2, no LLM/agent process; IMPROVED 2026-04-06. | phase-4E:L1-L9 |
| P4E-E002 | E/EVIDENCE | AutoB2G arxiv 2603.26005: adopt DAG + T_DAG + structured delta; reject SOCIA LLM repair; strategies REORDER + SUBSTITUTE only. | phase-4E:L10-L28, L127-L131 |
| P4E-E003 | E/DATA | Newtypes PipelineId/StepId/NodeIndex/StateId/DataRef; ExecutablePipeline with fsm_states, dependency_graph, novel_step_ids, agent_configs, rule_configs, resource_estimate, test_harness, repair_log, output_dir. | phase-4E:L41-L67 |
| P4E-E004 | E/DATA | Supporting: AgentPhaseConfig, RuleBinding, DependencyGraph, DagNode, PipelineFsmState (compiled Predicate gates, is_terminal), ResourceEstimate, TestHarnessSkeleton, RepairAction/Outcome, ValidationError + 8 kinds. | phase-4E:L69-L153 |
| P4E-E005 | E/TEST | Eight structural checks: tool exists, model available, acyclic Kahn, hardware fit, measurable metrics, compile_predicate, novel steps, visual MediaRenderAdapter mapping. | phase-4E:L155-L167, L381-L393 |
| P4E-E006 | E/FAIL | Non-repairable escalate immediately: resource overflow, unmeasurable metric, missing transitive deps with ambiguous resolution; no REORDER/SUBSTITUTE attempt. | phase-4E:L24-L26, L272-L273, L400-L401 |
| P4E-E007 | E/RETRY | DagRepairLoop max T_DAG=10; REORDER for cycles (weakest dep remove), SUBSTITUTE for missing tool same category compatible IO; structural hash fixpoint → escalate; REORDER before SUBSTITUTE same round. | phase-4E:L19-L23, L269-L276, L336-L339, L398-L404 |
| P4E-E008 | E/CAP | Nine stubs/APIs: PipelineBuilder, StructuralValidator, DagRepairLoop, FsmStateGenerator, TestHarnessGenerator, ResourceEstimator, NoveltyDetector, AgentConfigurator, RuleInstaller. | phase-4E:L183-L199 |
| P4E-E009 | E/FLOW | Assembly: PipelineSpec → DependencyGraph → validate → Phase Sizing (3.5) → repair → ExecutablePipeline msgpack under `.elai_cc/pipelines/{id}/`. | phase-4E:L361-L413 |
| P4E-E010 | E/CFG | Phase sizing: complexity = files*avg_file_tokens + tests*avg_test_tokens + contracts*200; split if > available_for_work * 0.6; max depth 2; merge adjacent only; uses PlanSkeleton::split_phase preserve_contracts. | phase-4E:L365-L376 |
| P4E-E011 | E/TEST | ≥30 base tests (validation/repair/novel/FSM/harness/resource/visual/e2e/proptest) + ≥10 adversarial; assembly <1 s for 10-phase; 100-phase <5 s adversarial. | phase-4E:L256-L355, L424-L427 |
| P4E-E012 | E/OBS | Metrics phase_4E.toml: assembly times, repair success ≥50% on ≥10 broken specs, strategy distribution, harness coverage. | phase-4E:L437-L463 |
| P4E-E013 | E/INT | Every routing participant must wire ssb, fsm, clock through single RuntimeClient; no None after construction. | phase-4E:L473-L473 |
| P4E-E014 | E/CAP | DecompositionTree extension is primary execution artifact (typed data, not prompts); TreeNavigator pure Rust flow control. | phase-4E:L476-L490 |
| P4E-E015 | E/DATA | DecompositionTree{tree_id,root,navigator,execution_log,bubble_up_log,integration_checkpoints}; TreeNavigatorState with completed/failed/blocked/traversal_order; FailureRecord; BabyStepResult/Outcome; BubbleUpEvent/Action/Policy; IntegrationCheckpoint. | phase-4E:L494-L577 |
| P4E-E016 | E/CFG | BubbleUpPolicy defaults: baby 3, step 2, phase 1, concept 1 retries; on_concept_fail=Escalate. | phase-4E:L562-L568 |
| P4E-E017 | E/FLOW | next_baby_step: precomputed topo order; skip completed/blocked; require pre_gates + dependencies; DepthFirstNavigator default. | phase-4E:L579-L618 |
| P4E-E018 | E/FLOW | Per baby-step 8-stage cycle: context Slot 3 → PRE_GATE → TESTING_FIRST → EXECUTING → AUDITING → ACCEPTANCE → INTEGRATION → EXIT. | phase-4E:L620-L671 |
| P4E-E019 | E/STATE | Context: BabyStepGrounding in Slot 3 (~500+200 tokens); EXECUTING context only grounding+task+test file; max_lines via TaskSpec; T1/T2/T3 hooks + incremental compile. | phase-4E:L624-L644, L809-L823 |
| P4E-E020 | E/RETRY | Acceptance fail: ResearchAlternative with FlashResearch failure context while attempts < max; else BubbleUpHandler cascade RedecomposeStep/ReplanPhase/ReplanConcept/Escalate. | phase-4E:L652-L707 |
| P4E-E021 | E/EVENT | Bubble-up logged SSB action_type=`replan_request` with tree context; INTERACTIVE escalate to human; AUTONOMOUS ReplanRequest(MAJOR). | phase-4E:L707-L709 |
| P4E-E022 | E/TEST | Integration checkpoints: baby cargo check <5 s; step module tests <30 s; phase crate <60 s; concept workspace <120 s; deferred skips until siblings complete. | phase-4E:L711-L735, L866-L891 |
| P4E-E023 | E/RECOVERY | TreeNavigatorState msgpack after every baby-step completion; corrupt checkpoint recovers via re-traverse from SSB. | phase-4E:L515-L515, L849-L860 |
| P4E-E024 | E/TEST | Extension steps 4E.7–4E.12: ≥18 navigation/bubble-up tests + ≥15 cycle tests → 33+ green; traversal deterministic 100 runs. | phase-4E:L739-L864 |
| P4E-E025 | E/OBS | 4E.12 measures baby steps/idea, FSM cycles, bubble-up rate, integration pass rate, traversal overhead <5%, context p50/p95 target <2K, ResearchCache savings. | phase-4E:L893-L911 |
| P4E-E026 | E/LIMIT | No LLM in L2 path (grep ban); L2 stays in orchestrator crate. | phase-4E:L199-L221, L428-L428 |
| P4E-E027 | E/DEP | Marks novel_step_ids for 4F FEASIBILITY_PROBE; visual_check maps to 4I MediaRenderAdapter. | phase-4E:L65-L67, L163-L167, L470-L470 |
| P4E-I001 | I/FLOW | 5-stage list includes Research and Feasibility Probe as surrounding stages; pure L2 owns assembly/validate/size/repair inside that pipeline. | phase-4E:L361-L364 |
| P4E-A001 | A/DATA | ValidationErrorKind lists CycleDetected while 4E.5 text says CyclicDependency — exact enum variant name open. | phase-4E:L144-L153, L377-L379 |
| P4E-A002 | A/CFG | CONTRACT_OVERHEAD default 200 and SPLIT_RATIO 0.6 named; config key surface for operator override not specified. | phase-4E:L367-L371 |
| P4E-A003 | A/FAIL | "weakest dependency" selection rule for REORDER not formally defined (edge weight? recency?). | phase-4E:L20-L20 |
| P4E-A004 | A/TEST | Integration test_command language-appropriate equivalent for non-Rust not fully specified. | phase-4E:L870-L871 |

Category coverage: builder APIs ✓; DAG repair ✓; phase sizing ✓; tree navigator/UI-less execution ✓; bubble-up ✓; integration checkpoints ✓; metrics ✓; RuntimeClient wiring ✓. No dedicated dashboard panel in this file; execution is orchestrator-driven.

### phase-index-3A-4J — concise phase index

Source: `docs/specs/guide/steps/phase-index-3A-4J.txt` (390/390 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| PIDX-E001 | E/STATUS | Index purpose: concise LLM context for phases 3A–4J (includes 3J); full detail in individual phase files; Phase 3 requires 0–2 complete. | phase-index-3A-4J:L1-L4 |
| PIDX-E002 | E/STATE | 3A full FSM 9 states incl CONFIGURING; 7 DAIRA evidence gates; INTERACTIVE APPROVE/REJECT/MODIFY/REDIRECT; risk LOW/MEDIUM/HIGH; ReplanRequest MINOR/MAJOR; AUTONOMOUS deferred to 4G. | phase-index-3A-4J:L6-L16 |
| PIDX-E003 | E/CAP | 3B SSB: SQLite WAL single-writer; 17 action_types; r2d2 pool; typed msgpack payloads; mpsc writer + broadcast; append-only + update_hook. | phase-index-3A-4J:L18-L27 |
| PIDX-E004 | E/CAP | 3C agents Tier A in-process: 8 roles; 4 death classes; 5 msgpack checkpoint files; supervision max_restarts; anti-spiral 3x+blake2s; heartbeat 5s/15s. | phase-index-3A-4J:L29-L40 |
| PIDX-E005 | E/CAP | 3D BSP: LastValue/Topic/EphemeralValue/UntrackedValue; BinaryOp dropped; JoinSet parallel; Untracked never checkpointed. | phase-index-3A-4J:L42-L53 |
| PIDX-E006 | E/CFG | 3E 8 slots; dynamic 15/40/25/20; floors Ssb≥200 Grc≥500 Out≥2000; 2 cache breakpoints; adaptive reduction to ESCALATE/SPLIT_TASK; assembly <50 ms. | phase-index-3A-4J:L55-L64 |
| PIDX-E007 | E/CAP | 3F rules DATA in SQLite; 5 predicate types; Verdict ALLOW/WARN/BLOCK/ESCALATE; lifecycle PROPOSED→…→RETIRED; ~33 seed rules; ≥254 static tests. | phase-index-3A-4J:L66-L78 |
| PIDX-E008 | E/CAP | 3G skeleton+detail; goal≤120; Kahn deps; stub_reference; coherence auditor checks. | phase-index-3A-4J:L80-L88 |
| PIDX-E009 | E/CAP | 3H handoff summary matches detailed phase: DAG, 3-pass trim, max 3 branches, 85% handoff, blake3 stubs. | phase-index-3A-4J:L90-L98 |
| PIDX-E010 | E/TEST | 3I DST: digital-sibling oracle; proptest-state-machine 10K+; madsim faults; FsmEvent set including SuperstepBoundary. | phase-index-3A-4J:L100-L108 |
| PIDX-E011 | E/CAP | 3K RAG hardening: AST adaptive chunking; multi-query expansion; HyDE via sidecar; tiers FAST/STANDARD/DEEP; 30-query benchmark. | phase-index-3A-4J:L111-L128 |
| PIDX-E012 | E/CAP | 3J vLLM+DoLa+ActAdd+MCP+AppContainer+typed 17 SSB; compose/local/pipeline/wire substeps; backend modes local\|api\|hybrid\|auto; elai.bat E2E. | phase-index-3A-4J:L130-L172 |
| PIDX-E013 | E/SURFACE | 3J.LOCAL/4G model UI: CLI `--local`/`--api`/`--hybrid`; runtime `/model`; config `[agents.{role}.backend]` and `[backend] mode`. | phase-index-3A-4J:L156-L160, L310-L318 |
| PIDX-E014 | E/CAP | 4A Tier C isolation: 15 IPC messages; agent binary no LLM/network/tools; Linux seccomp+namespaces; Windows AppContainer; ToolFilter at IPC. | phase-index-3A-4J:L177-L188 |
| PIDX-E015 | E/CAP | 4B tools: git2, 5-tier validation, DAIRA markers, 3-tier edit match, ShortCoder, 9 failure taxonomy, outline >300 lines, IntegrationTestRunner levels. | phase-index-3A-4J:L190-L206 |
| PIDX-E016 | E/CAP | 4C hooks PreToolUse/PostToolUse/Stop/PreCompact; T1/T2/T3; generic "tool call rejected by policy" sanitization. | phase-index-3A-4J:L208-L215 |
| PIDX-E017 | E/CAP | 4D L1 Research+Designer+recursive Concept→BabySteps; ResearchTree phi/psi; SourceQuality 5-tier; BabyStepSpec 8 blocks; ResearchCache LRU. | phase-index-3A-4J:L217-L238 |
| PIDX-E018 | E/CAP | 4E L2 index summary: 5-stage, T_DAG=10, 8 validation, Phase Sizing 0.6/depth2, DecompositionTree+BubbleUp+integration times. | phase-index-3A-4J:L240-L269 |
| PIDX-E019 | E/CAP | 4F feasibility: skip if zero novel AND ≥20% margin; GRC 3/600; 25s/30s kills; per-concept baby-step probes. | phase-index-3A-4J:L271-L288 |
| PIDX-E020 | E/SURFACE | 4G AutonomousGate; `/mode autonomous\|interactive` next-gate; 6 evidence checks; SPLIT_TASK max 4 depth 2; TreeAutoAdvancePolicy circuit 50 baby steps/concept. | phase-index-3A-4J:L290-L318 |
| PIDX-E021 | E/CAP | 4H self-mod Classes A/B; Layers 1/1A/2/3; MSI bands; AllowedConfigKey; 4 safety layers; PipelineMode SURGICAL/GREENFIELD. | phase-index-3A-4J:L321-L353 |
| PIDX-E022 | E/CAP | 4I media validator conditional; Tier1 tools + Tier2 VLM advisory; CLAP audio; MediaRenderAdapter loop. | phase-index-3A-4J:L355-L371 |
| PIDX-E023 | E/CAP | 4J Empire feature-gated; FactoryLine client; 4 ElaiNeuron capabilities; graceful degrade without Empire. | phase-index-3A-4J:L373-L388 |
| PIDX-E024 | E/SURFACE | Slash/CLI surfaces named in index: INTERACTIVE gate verbs; `/plan` (3J.WIRE.PLAN); `/mode`; `/model`; `elai --local/--hybrid`; `elai.bat`. | phase-index-3A-4J:L12-L12, L166-L170, L300-L317 |
| PIDX-I001 | I/STATUS | Index is compression of many phase files; atoms here are index-level claims, not substitutes for deep chunk ledgers of those phases. | phase-index-3A-4J:L1-L2 |
| PIDX-A001 | A/STATUS | Index lists 3K before 3J narrative order; dependency text says 3K needs 1A–1I and 3J for HyDE only — numbering presentation vs implement order open. | phase-index-3A-4J:L111-L127 |
| PIDX-C001 | C/EVENT | 3B says 17 action_types while 3H source debates adding branch_abandon — index does not resolve enum growth. | phase-index-3A-4J:L20-L21, L90-L98 |

Category coverage: full Phase 3–4 surface map ✓; CLI/mode/model commands ✓; deferred/conditional phases ✓. Index omits surgical `/task --surgical` and SurgicalDrawer (those live in 4D-SURGICAL, not this index).

## Cross-file interaction records

| IX ID | Interaction | Participating atoms |
|---|---|---|
| IX-001 | 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish. | P1B-E007, P1B-E018, P1G-E006 |
| IX-002 | 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators. | P1G-E012, P1G-E014, P1B-E018 |
| IX-003 | 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget. | P3H-E010, P3H-A006, PIDX-E006 |
| IX-004 | 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry. | P3H-E013-E014, PIDX-E003, PIDX-C001 |
| IX-005 | 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots. | P3H-E009, PIDX-E006, PIDX-E008 |
| IX-006 | 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth. | P4E-E010, PIDX-E008, PIDX-E020 |
| IX-007 | 4E novel_step_ids consumed by 4F; visual_check adapters by 4I. | P4E-E027, PIDX-E019, PIDX-E022 |
| IX-008 | 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks. | P4E-E018-E019, PIDX-E002, PIDX-E016 |
| IX-009 | 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1. | P4E-E014, P4E-E020, PIDX-E017 |
| IX-010 | 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds. | P4DS-E009, P4DS-E015, PIDX-E006 |
| IX-011 | 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension. | P4DS-E006-E007, P1B-E001 |
| IX-012 | Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path. | PIDX-E011, P1B-E010, P1B-E014 |
| IX-013 | Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path). | PIDX-E012, P3H-E015 |
| IX-014 | Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder. | P4DS-E002-E003, P4E-E001, PIDX-E017 |
| IX-015 | RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition. | P4E-E013, PIDX-E002-E003 |
| IX-016 | Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families. | P1G-E012, P4DS-E010, P4E-E021 |
| IX-017 | User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G). | P1B-E019, P4DS-E002, P4DS-E011, PIDX-E013, PIDX-E024 |
| IX-018 | 4E integration time ladder matches 4B IntegrationTestRunner levels in index. | P4E-E022, PIDX-E015 |
| IX-019 | 4H MSI tracks SURGICAL/GREENFIELD modes; surgical CLI path is a related but separate C-30 feature. | PIDX-E021, P4DS-E001 |
| IX-020 | Handoff 85% (3H) aligns with 4C PreCompact checkpoint-at-85% theme in index. | P3H-E002, PIDX-E016 |

## Ambiguity and conflict register

| AC ID | Kind | Issue | Atoms |
|---|---|---|---|
| AC-001 | conflict/schema | tantivy_v21 label vs INDEX_FORMAT_VERSION u32. | P1B-C001, P1B-A001 |
| AC-002 | ambiguity/config | Exact tantivy/ort pin versions deferred to execution decision docs. | P1B-A001, P1G-A003 |
| AC-003 | conflict/fallback | L-6 reranker tier needs amendment vs skip-only spec. | P1G-C001, P1G-E011 |
| AC-004 | ambiguity/measure | 1G skip threshold and 50 ms pass/fail after levers. | P1G-A001, P1G-E018 |
| AC-005 | conflict/schema | 3H per-message snapshot_context vs orchestrator blob schema. | P3H-C001, P3H-A001 |
| AC-006 | ambiguity/event | branch_abandon new type vs checkpoint payload. | P3H-A002, PIDX-C001 |
| AC-007 | ambiguity/data | SSB merge cleanup logical vs physical wording. | P3H-A003 |
| AC-008 | ambiguity/merge | File-overlap Conflict variant ungrounded. | P3H-A004 |
| AC-009 | ambiguity/crypto | blake3 stub hash algorithm choice. | P3H-A005 |
| AC-010 | ambiguity/surface | SurgicalDrawer full UI fields; step-cap merge notes. | P4DS-A001, P4DS-A002 |
| AC-011 | ambiguity/enum | CycleDetected vs CyclicDependency naming. | P4E-A001 |
| AC-012 | ambiguity/repair | Weakest-dependency definition for REORDER. | P4E-A003 |
| AC-013 | ambiguity/order | Index 3K listed before 3J vs HyDE dependency on 3J. | PIDX-A001 |
| AC-014 | ambiguity/index | Index omits C-30 surgical CLI/drawer present in 4D-SURGICAL. | PIDX-I001, P4DS-E002 |
| AC-015 | ambiguity/dep | 3E total_allocated_tokens exposure for TrimConfig. | P3H-A006 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`.

### phase-1B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1B-001 | **ADD** full API surface IndexStats/BM25Hit snippet policy, single-writer batch lock, and explicit project_id rule. | MASTER:L57-L67 | P1B-E006-E013 |
| SD-1B-002 | **ADD** WIRING_CONTRACT CLI `index build` + SSB `index_checkpoint` + call sites. | MASTER:L57-L67 | P1B-E018-E019 |
| SD-1B-003 | **ADD** environment-aware CI 50 ms vs dev 10 ms gates and warm-up measurement. | MASTER:L67 | P1B-E017 |
| SD-1B-004 | **ADD/CORRECT** tantivy_v21 vs INDEX_FORMAT_VERSION and DkgNode-vs-BM25Chunk ownership. | MASTER:L61-L63 | P1B-E004-E006, P1B-C001 |
| SD-1B-005 | **ADD** 1I scoping-out of readiness/breaker/versioning from 1B. | MASTER:L67 | P1B-E020 |

### phase-1G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1G-001 | **ADD** full RerankerConfig field set, mpsc batching design, and oneshot offset split. | MASTER:L123-L133 | P1G-E004-E008 |
| SD-1G-002 | **ADD** L-6 amendment path, cold-start SLA exception, and canary DEGRADED semantics. | MASTER:L129-L132 | P1G-E010-E014, P1G-C001 |
| SD-1G-003 | **ADD** INT8 trigger p50>30 ms / NDCG delta <0.5 and quantization script path. | MASTER:L127, L133 | P1G-E015 |
| SD-1G-004 | **ADD** complete metrics list including queue wait p99 and amendment-on-fail-50ms rule. | MASTER:L133 | P1G-E017-E018 |
| SD-1G-005 | **ADD** 1F DROP independence note (MASTER has 1F section; 1G body reconfirms). | MASTER:L114-L121 | P1G-E001 |

### phase-3H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3H-001 | **ADD** full SQL column/index set, types, and DELETE-journal/FK/integrity_check safety. | MASTER:L334-L344 | P3H-E003-E008 |
| SD-3H-002 | **EXPAND** three amendments (schema/event/cleanup) and Conflict/blake3 open items beyond MASTER [Q] one-liner. | MASTER:L337 | P3H-A001-A005, P3H-C001 |
| SD-3H-003 | **ADD** recovery APIs, typed StubRecoveryFailed, and 100-run determinism/measure targets. | MASTER:L343-L344 | P3H-E014-E017 |
| SD-3H-004 | **ADD** branch naming rules, close_branch, and immutable trunk-extension merge semantics detail. | MASTER:L342 | P3H-E012 |

### phase-4D-SURGICAL deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4DS-001 | **ADD** entire C-30 surgical path: MASTER only flags file as unindexed; full CLI/SSB/drawer/LOC/complexity atoms are deep-only. | MASTER:L655 | P4DS-E001-E015 |
| SD-4DS-002 | **ADD** intercept-before-research runner semantics and DkgQuerierAdapter stub ban. | MASTER:L655 | P4DS-E003, P4DS-E007, P4DS-E013 |

### phase-4E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4E-001 | **ADD** full type inventory, nine builder stubs, and 8 ValidationErrorKind list. | MASTER:L442-L455 | P4E-E003-E008 |
| SD-4E-002 | **ADD** Phase Sizing formula constants, fixpoint hash escalate, REORDER-before-SUBSTITUTE, RuntimeClient wiring rule. | MASTER:L445-L449 | P4E-E007, P4E-E009-E010, P4E-E013 |
| SD-4E-003 | **ADD** complete baby-step 8-stage cycle, BubbleUpPolicy defaults, SSB replan_request, msgpack recovery. | MASTER:L451-L455 | P4E-E014-E023 |
| SD-4E-004 | **ADD** test catalog counts (≥30+10+33) and 4E.12 context p95/overhead gates. | MASTER:L450-L455 | P4E-E011, P4E-E024-E025 |
| SD-4E-005 | **FLAG** enum name CycleDetected vs CyclicDependency and weakest-dep open. | MASTER:L446-L448 | P4E-A001, P4E-A003 |

### phase-index-3A-4J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-IDX-001 | **ADD** this file is a second index surface parallel to MASTER; captures 3J wire substeps, `/model`, backend modes, 4H MSI, Empire caps not always expanded in MASTER bullets. | MASTER:L334-L478 region | PIDX-E012-E013, PIDX-E021-E024 |
| SD-IDX-002 | **QUALIFY** index is not authoritative over deep phase files when they conflict (e.g. surgical absent here). | MASTER:L1-L8 | PIDX-I001, PIDX-A001, AC-014 |
| SD-IDX-003 | **ADD** 3K FAST/STANDARD/DEEP tier routing and 4B IntegrationTestRunner ladder as index-compressed features. | MASTER:L399, retrieval ladder | PIDX-E011, PIDX-E015 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **6/6 files**, **2,751/2,751 logical lines**, **182,824/182,824 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **151 unique atoms** = **127 explicit**, **002 inferred**, **018 ambiguous**, **004 conflicting/corrective**.
- Per-file atom counts: 1B **025**; 1G **025**; 3H **025**; 4D-SURGICAL **017**; 4E **032**; index-3A-4J **027**. Sum **151**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **024** (`SD-*`). Ambiguity/conflict register: **15** (`AC-001`–`AC-015`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1B/P1G/P3H/P4DS/P4E/PIDX-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: 1B `index build` CLI; 1G no CLI (pipeline-internal); 3H handoff/branch load semantics; 4D-SURGICAL `/task --surgical` + SurgicalDrawer + ConfirmationRequired; 4E orchestrator tree execution (no panel); index `/plan`, `/mode`, `/model`, `--local/--api/--hybrid`, `elai.bat`, INTERACTIVE gate verbs — each captured as explicit SURFACE atoms where the source states them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta/I records and are not elevated to explicit requirements.
- Manifest-boundary audit: the only original sources opened were the six exact DEEP-PHASE-003 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0349`; `output_bytes=046829`.
- Canonical normalized output SHA-256: 43ab4103eb9d6b37f11c813391951d014f376c04f9ad704fa36a31b22a1fcbfd.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
