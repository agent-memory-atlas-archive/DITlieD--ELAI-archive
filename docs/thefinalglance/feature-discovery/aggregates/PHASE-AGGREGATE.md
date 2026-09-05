# PHASE family aggregate — ELAI deep feature discovery

## Meta
- Family: **PHASE** (implementation-guide phase step files + phase indexes).
- Inputs: `DEEP-PHASE-001` … `DEEP-PHASE-008` + `01-phase-index-MASTER.discovery.md`.
- Output: this file only. Inputs read-only.
- Feature atom IDs keep original IDs (`P0-E001`, `IDX-E001`, `P4G-E012`, …).
- Relational IDs namespaced: `PHASE-00N/IX-*`, `PHASE-00N/AC-*`, `PHASE-00N/SD-*`.
- MASTER summary bullets: `MASTER-*` with anchors into `01-phase-index-MASTER.discovery.md`.
- Newer-plan precedence: explicit SUPERSEDED/replacement markers win; else higher plan/phase number wins **target state**; losers kept as superseded notes.
- Classification: E explicit / I inference / A ambiguity / C conflict / D deferred (MASTER) / Q question (MASTER).

## Ledger inventory

| Ledger | Files | Declared feature atoms | Parsed feature rows | IX | SD | AC | Per-file counts |
|---|---:|---:|---:|---:|---:|---:|---|
| PHASE-001 | 6/6 | 278 | 278 | 18 | 36 | 12 | phase-0 **074**; phase-1C **045**; phase-3A **038**; phase-3J-BASELINE **022**; phase-4K **061**; phase-4M **038** |
| PHASE-002 | 6/6 | 189 | 189 | 20 | 31 | 15 | 1A **047**; 1D **027**; 3D **026**; 3G **027**; 4D-ORACLE **019**; 4G **043** |
| PHASE-003 | 6/6 | 151 | 151 | 20 | 24 | 15 | 1B **025**; 1G **025**; 3H **025**; 4D-SURGICAL **017**; 4E **032**; index-3A-4J **027** |
| PHASE-004 | 6/6 | 157 | 157 | 20 | 32 | 12 | 1E **034**; 2B **027**; 2C **024**; 3L **024**; 4C **021**; 4I **027** |
| PHASE-005 | 4/4 | 213 | 213 | 20 | 26 | 15 | 1F **049**; 2A **038**; 3F **050**; 3J **076** |
| PHASE-006 | 5/5 | 238 | 238 | 22 | 38 | 10 | 1H **035**; 3I **036**; 3K **035**; 4A **054**; 4H **078** |
| PHASE-007 | 5/5 | 198 | 198 | 20 | 34 | 13 | pre-phase-0 **028**; phase-1I **044**; phase-2D **028**; phase-3B **035**; phase-4D **063** |
| PHASE-008 | 6/6 | 253 | 253 | 32 | 35 | 11 | 3C **065**; 3E **039**; 4B **041**; 4F **040**; 4J **042**; IDX **026** |
| MASTER summary | 1 | — | 515 | — | — | — | orientation inventory |

---

# Atomic feature records (all deep ledgers)

Every distinct atomic feature from the eight deep ledgers is listed below, grouped by ledger then source unit. Descriptions and anchors preserved from the deep discovery files.

## PHASE-001 — source `DEEP-PHASE-001.discovery.md`

- Declared feature atoms: **278** (E=249 I=6 A=19 C=4).
- Parsed feature rows in this aggregate: **278**.
- Per-file: phase-0 **074**; phase-1C **045**; phase-3A **038**; phase-3J-BASELINE **022**; phase-4K **061**; phase-4M **038**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-001.discovery.md`.

### Feature atoms

### phase-0 — fork, strip, restructure, walking skeleton

Source: `docs/specs/guide/steps/phase-0.txt` (780/780 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P0-E001 | E/STATUS | Phase 0 is FORK AND STRIP: produce a walking skeleton; prerequisite is pre-phase-0 complete (Windows platform fixes). | phase-0:L1-L2 |
| P0-E002 | E/CAP | Goal: claw-code-rust-port compiles as elai_cc with unnecessary code removed; one model (ollama), one FSM loop IDLE->EXECUTING->IDLE, one tool (file read); no new features. | phase-0:L6-L9 |
| P0-E003 | E/DEP | Mandatory pre-read set: architecture-overview.txt §§1-4, core.txt §§3-4, providers.txt §2, base-fork-comparison.txt full. | phase-0:L11-L16 |
| P0-E004 | E/EVIDENCE | R17: claw-code compilation verified 26.24s, 0 errors. | phase-0:L18-L19 |
| P0-E005 | E/LIMIT | R18: module quality unknown; auditor reviews each module during strip. | phase-0:L20-L20 |
| P0-E006 | E/DATA | R19: runtime is 25,035 LOC / 38 files; monoliths main.rs 8,615, commands/lib.rs 4,841, tools/lib.rs 7,345 LOC. | phase-0:L21-L22 |
| P0-E007 | E/DET | R20: entropy quarantine scope is 60 occurrences across 31 files. | phase-0:L23-L23 |
| P0-E008 | E/EVIDENCE | R21: base-fork-comparison LOC numbers are stale (green_contract listed 4,061, actual 152); always verify against actual files. | phase-0:L24-L25 |
| P0-E009 | E/INT | R22: config.rs imports SandboxConfig and FilesystemIsolationMode; both must move to core/ with config.rs to avoid core→orchestrator cycle. | phase-0:L26-L27 |
| P0-E010 | E/DATA | R23: TokenUsage/UsageTracker embedded in Session, conversation.rs, main.rs (61 occurrences / 5 files); stripping usage.rs is structural, not file-delete. | phase-0:L28-L29 |
| P0-E011 | E/INT | R24: tools/lib.rs imports api::*; when api/ absorbs into providers/, tools imports must update simultaneously. | phase-0:L30-L31 |
| P0-E012 | E/SURFACE | R25: commands/lib.rs has 28+ claw-code slash commands that conflict with elai_cc /task, /config, /continue. | phase-0:L32-L33 |
| P0-E013 | E/FLOW | Design: strip before restructure removes ~4K+ LOC of known-bad code before splitting runtime into 5 crates, reducing move/break risk. | phase-0:L37-L41 |
| P0-E014 | E/DATA | Workspace path migration: 0.1-0.3 work in .elai_cc/rust/; 0.4 creates .elai_cc/crates/ with .elai_cc/Cargo.toml workspace root; 0.5+ use that root; old rust/ is archived after 0.5 verifies. | phase-0:L43-L47 |
| P0-E015 | E/CFG | ONNX models live at canonical path .elai_cc/models/onnx/; pre-staged via setup (download + blake2 verify + index_version.toml); never auto-downloaded. | phase-0:L49-L52 |
| P0-E016 | E/FAIL | Missing ONNX models at startup enter DEGRADED mode BM25_ONLY, not crash. | phase-0:L52-L53 |
| P0-E017 | E/SEC | Visibility filter hides .elai_cc/models/** from agents via default hidden_paths. | phase-0:L53-L54 |
| P0-E018 | E/DATA | TokenUsage handling: strip usage.rs but define minimal core TokenUsage {input_tokens, output_tokens, total_tokens, estimated_cost_usd}; strip UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing (billing concepts). | phase-0:L56-L62 |
| P0-E019 | E/CAP | Provider trait migration: keep claw-code Provider trait during Phase 0 marked #[deprecated]; OllamaProvider implements LlmProvider (complete, embed, token_count, token_count_estimate); Anthropic/OpenAI migrate to LlmProvider in Phase 1+. | phase-0:L64-L69 |
| P0-E020 | E/TEST | 0.1 PRE_GATE: pre-phase-0 cargo check passes; cargo installed; .elai_cc/rust/Cargo.toml exists. | phase-0:L74-L79 |
| P0-E021 | E/OBS | 0.1 records baseline metrics into phase_0_baseline.toml fields compile_time_s, test_pass, test_fail, test_total, loc_total, loc_by_crate. | phase-0:L80-L91 |
| P0-E022 | E/DATA | 0.1 creates migration_manifest.toml classifying every .rs file KEEP/STRIP/REPLACE/EXTEND with source crate, target crate, rationale; must cover 69 files. | phase-0:L84-L93 |
| P0-E023 | E/TEST | 0.1 adversarial: baseline only after clean git status (no uncommitted changes in .elai_cc/rust/). | phase-0:L94-L96 |
| P0-E024 | E/FLOW | 0.2 strip order (each followed by cargo check): telemetry crate; compat-harness + call sites; structural usage/compact/summary_compression with TokenUsage shim; lsp_client; oauth DEFERRED; audit prompt.rs; classify slash commands only; evaluate remote.rs. | phase-0:L109-L155 |
| P0-E025 | E/DATA | 0.2c TokenUsage temporary shim is {input_tokens, output_tokens} only; core::TokenUsage replaces it in 0.4a. | phase-0:L127-L130 |
| P0-E026 | E/SURFACE | 0.2g KEEP candidates: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. | phase-0:L146-L148 |
| P0-E027 | E/SURFACE | 0.2g STRIP claw-code commands: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. | phase-0:L149-L150 |
| P0-E028 | E/LIMIT | Patterns sentinel, anti_distill, bias_inject do NOT exist (verified 2026-04-06); do not grep for them. | phase-0:L157-L158 |
| P0-E029 | E/TEST | 0.2 checks: stripped files absent; no dangling compact/summary_compression/usage/lsp_client refs; interactive-agent prompt gone; slash audit in manifest; cargo check 0 errors. | phase-0:L159-L173 |
| P0-E030 | E/TEST | 0.3 classify failing tests EXPECTED or BUG; fix BUGs; record strip delta loc_removed, files_removed, tests_removed, tests_broken_expected, tests_broken_bug. | phase-0:L194-L204 |
| P0-E031 | E/FAIL | 0.3 flag if tests_removed >10% of baseline total as over-stripping signal. | phase-0:L209-L210 |
| P0-E032 | E/CAP | 0.4a creates workspace root .elai_cc/Cargo.toml and crate dirs under .elai_cc/crates/; copies from intact .elai_cc/rust/. | phase-0:L221-L223 |
| P0-E033 | E/CAP | 0.4a core/ extracts config.rs, session_control.rs, json.rs, bootstrap.rs plus sandbox types; defines core::TokenUsage. | phase-0:L225-L236 |
| P0-E034 | E/CAP | 0.4a session_control identity stays in core/; Session state management stays in orchestrator/. | phase-0:L234-L236 |
| P0-E035 | E/CAP | 0.4a rules/ extracts policy_engine.rs, green_contract.rs, trust_resolver.rs. | phase-0:L239-L241 |
| P0-E036 | E/CAP | 0.4a providers/ absorbs all api/src/ plus runtime/sse.rs; mark old Provider #[deprecated]; update all use api:: including tools/lib.rs. | phase-0:L243-L250 |
| P0-E037 | E/LIMIT | core/ must be a true leaf: no deps on orchestrator/plugins/cli. | phase-0:L256-L257 |
| P0-E038 | E/CAP | 0.4b tools/ keeps existing tools crate + file_ops, bash, bash_validation, stale_branch, branch_lock; depends on core/ not runtime/. | phase-0:L276-L282 |
| P0-E039 | E/LIMIT | tools/lib.rs split if >3 tools; no single .rs in tools/ exceeds 2000 LOC after restructure. | phase-0:L280-L298 |
| P0-E040 | E/CAP | 0.4b plugins/ keeps existing + hooks.rs, plugin_lifecycle.rs; depends on core/. | phase-0:L284-L287 |
| P0-E041 | E/CAP | 0.4b knowledge/ and reasoning/ empty skeleton crates for Phase 1/2 fill. | phase-0:L289-L290 |
| P0-E042 | E/CAP | 0.4c orchestrator/ receives remaining runtime modules; depends on core/providers/tools/rules. | phase-0:L309-L321 |
| P0-E043 | E/FLOW | 0.4c evaluate lane_events, task_packet, task_registry, team_cron_registry: KEEP if maps to elai_cc agents/FSM/sessions else STRIP. | phase-0:L316-L319 |
| P0-E044 | E/CAP | 0.4c cli/ absorbs rusty-claude-cli + commands/; main.rs thin REPL target <500 LOC; strip claw-code commands per 0.2g. | phase-0:L323-L334 |
| P0-E045 | E/DATA | 0.4c cleanup: archive .elai_cc/rust/; remove absorbed dirs; workspace members exactly 9 crates. | phase-0:L336-L341 |
| P0-E046 | E/CAP | Canonical nine crates: core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli. | phase-0:L346-L347 |
| P0-E047 | E/TEST | 0.5: cargo test --no-run; cargo test --workspace; classify RESTRUCTURE_BUG or PATH_UPDATE; phase_0_restructure.toml total_crates=9. | phase-0:L372-L383 |
| P0-E048 | E/TEST | 0.6 contracts crate, 5 tests provider_trait/fsm_basic/tool_basic/config_basic/visibility_filter; TDD red (compile, fail runtime). | phase-0:L402-L428 |
| P0-E049 | E/CAP | 0.7a LlmProvider: complete (streaming Token), embed, token_count, token_count_estimate; all take &self. | phase-0:L444-L449 |
| P0-E050 | E/CAP | 0.7a OllamaProvider: POST /api/generate and /api/embed; base_url default http://localhost:11434; NDJSON streaming. | phase-0:L450-L453 |
| P0-E051 | E/CFG | 0.7a elai_cc.toml in core/: provider + project + agent role configs; toml parse; msgpack rmp-serde round-trip. | phase-0:L454-L455 |
| P0-E052 | E/STATE | 0.7b minimal FSM IDLE/EXECUTING; invalid transitions return Error not panic; get_state, transition, get_active_tools. | phase-0:L479-L482 |
| P0-E053 | E/CAP | 0.7b file_read: contents + total_lines + estimated_tokens; hidden/symlink-hidden return NotFound identical to missing. | phase-0:L483-L499 |
| P0-E054 | E/DATA | 0.7b session storage msgpack via rmp-serde; verified by round-trip and valid msgpack on disk. | phase-0:L486-L494 |
| P0-E055 | E/SURFACE | 0.7c slash commands /task, /config, /continue, /explore, /status; bare messages fallback to /task; unknown command error. | phase-0:L517-L519 |
| P0-E056 | E/SEC | 0.7c visibility: allowed_paths + hidden_paths; canonicalize; defaults hide models/**, rules.db, harness, checkpoints, SSB, binary internals. | phase-0:L520-L526 |
| P0-E057 | E/SEC | 0.7c error and tool-schema sanitization allowlist-based (not denylist). | phase-0:L527-L529 |
| P0-E058 | E/OBS | 0.7c tracing + tracing-subscriber replace telemetry; log tool calls, FSM transitions, file ops. | phase-0:L530-L531 |
| P0-E059 | E/TEST | 0.8 smoke_test_ollama.sh WARN not BLOCK if ollama unavailable. | phase-0:L558-L564 |
| P0-E060 | E/CAP | 0.9 ClockProvider: now, elapsed_since, system_now; &self. | phase-0:L581-L586 |
| P0-E061 | E/CAP | 0.9 EntropySource: next_u64, fill_bytes; &mut self. | phase-0:L587-L590 |
| P0-E062 | E/CAP | 0.9 OsClock/OsRng production; SeededClock (explicit tick) and SeededRng (ChaCha8Rng seed_from_u64) for tests. | phase-0:L592-L597 |
| P0-E063 | E/INT | 0.9 RuntimeContext bundles clock Arc and entropy Arc Mutex. | phase-0:L606-L608 |
| P0-E064 | E/LIMIT | Raw time only core/src/clock.rs; raw RNG only core/src/entropy.rs; CI check_entropy_quarantine.sh excludes those and TEST_KEEP. | phase-0:L609-L615 |
| P0-E065 | E/TEST | 0.10 determinism: ELAI_SEED=42, --test-threads=1, two runs stdout diff=0; required for Phase 3I DST. | phase-0:L647-L654 |
| P0-E066 | E/DATA | MODULE MAPPING authoritative runtime→crate destinations with verified 2026-04-06 LOCs; SSE dedup keep api/sse strip runtime/sse. | phase-0:L666-L686 |
| P0-E067 | E/OBS | Measurements: compile <30s target; test counts; LOC; strip delta; ollama latency; raw entropy outside core=0. | phase-0:L748-L754 |
| P0-E068 | E/EVIDENCE | Grounding: 684 tests, 61,317 LOC, 69 .rs files verified; custom clippy UNGROUNDED (CI grep); quarantine ~200 LOC estimate UNGROUNDED vs 60/31 measured. | phase-0:L757-L766 |
| P0-I001 | I/DATA | estimated_cost_usd retained on core TokenUsage despite non-billing design; must not drive claw-code billing paths. | phase-0:L56-L61 |
| P0-A001 | A/CAP | prompt.rs REPLACE vs STRIP open. | phase-0:L142-L144 |
| P0-A002 | A/DEFER | remote.rs DEFERRED vs STRIP open. | phase-0:L155-L155 |
| P0-A003 | A/CAP | sandbox whole-file vs types-only extraction open. | phase-0:L228-L231 |
| P0-A004 | A/CAP | cli/main.rs <500 LOC soft target; else document remaining LOC. | phase-0:L332-L333 |
| P0-C001 | C/DATA | TokenUsage field set conflicts: 0.2c shim 2 fields vs design block 4 fields including total_tokens and estimated_cost_usd. | phase-0:L56-L61, L127-L130 |

Category coverage: capabilities/CLI ✓; workflows/FSM ✓; schemas/config/msgpack ✓; flags/paths ✓; integrations/providers ✓; security/visibility ✓; failures/degraded ✓; determinism/entropy ✓; observability ✓; tests/TDD ✓; constraints/deferred ✓. No formal privacy policy.

### phase-1C — dual-model ONNX vector search

Source: `docs/specs/guide/steps/phase-1C.txt` (531/531 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1C-E001 | E/STATUS | Phase 1C requires Phase 1B; dual GraphCodeBERT/Granite ONNX embeddings, SQLite vector cache, per-model breakers, startup canary. | phase-1C:L1-L9 |
| P1C-E002 | E/DEP | Pre-read knowledge.txt §§2,3,5-7,11,13; providers.txt §§2,9; architecture §10; risks R22,R26,R27,R29; usearch decision finalized in phase-1C-decisions.md. | phase-1C:L11-L17 |
| P1C-E003 | E/CFG | R22: ONNX GPU1 when executor/auditor on GPU0; else CPU-only; ModelPool.can_fit_onnx before allocation. | phase-1C:L20-L23 |
| P1C-E004 | E/RECOVERY | R26 cascade HYBRID -> SPARSE_ONLY -> BM25_ONLY; never total knowledge outage. | phase-1C:L25-L27 |
| P1C-E005 | E/FAIL | R27: canary embed("fn main()") within 5s. | phase-1C:L29-L30 |
| P1C-E006 | E/CONC | R29: cross-encoder Semaphore(1) max queue depth 4. | phase-1C:L32-L33 |
| P1C-E007 | E/DEFER | SPLADE deferred from 1C; dual-vector only; later additive 4th RRF ranker. | phase-1C:L37-L47 |
| P1C-E008 | E/DATA | CODE GraphCodeBERT 768d; TEXT Granite-embedding-small 384d. | phase-1C:L52-L53 |
| P1C-E009 | E/DATA | vector_cache: content_blake2s BLOB PK, code_vec, text_vec, cached_at INTEGER. | phase-1C:L55-L55 |
| P1C-E010 | E/CFG | Breaker: 3 failures / 60s open; half-open after 30s. | phase-1C:L57-L57 |
| P1C-E011 | E/DATA | Cache hit rate >90% incremental re-index; invalidate on SSB file_write. | phase-1C:L60-L61 |
| P1C-E012 | E/RECOVERY | Code fail->TEXT-only; Granite fail->CODE-only; both->SPARSE_ONLY. | phase-1C:L62-L62 |
| P1C-E013 | E/CFG | ONNX+LLM GPU multiplexing only dual-GPU; single GPU = CPU-only ONNX. | phase-1C:L63-L63 |
| P1C-E014 | E/DEP | Pinned ort, usearch v2.24.0, blake2 Blake2s256; do NOT add fastembed with ort. | phase-1C:L65-L77, L114-L121 |
| P1C-E015 | E/DEFER | TurboQuant grounded (~10.7× b=3) but 1C stores raw f32; quant deferred to 1G+. | phase-1C:L79-L92, L365-L367 |
| P1C-E016 | E/LIMIT | Ungrounded until 3.5: tensor formats; embed throughput; dual-vector vs BM25 accuracy. | phase-1C:L94-L100 |
| P1C-E017 | E/CAP | Modules vector/{mod,cache,ort_session,circuit_breaker,hnsw_index}; DualVector; stubs Err(NotReady). | phase-1C:L157-L162 |
| P1C-E018 | E/CAP | OrtSession Arc<Mutex<Session>>; max_seq_len=512; truncate+WARN; path .elai_cc/models/onnx/; missing ModelNotFound->BM25_ONLY. | phase-1C:L163-L178 |
| P1C-E019 | E/CFG | Canonical ONNX path configurable [knowledge.onnx_model_path]. | phase-1C:L172-L172 |
| P1C-E020 | E/CAP | CircuitBreaker CLOSED/OPEN/HALF_OPEN; 3/60s; half-open 30s. | phase-1C:L179-L180 |
| P1C-E021 | E/EVENT | SSB file_write DELETE vector_cache by path; registered at KnowledgeEngine startup. | phase-1C:L181-L183, L397-L398 |
| P1C-E022 | E/CAP | UsearchIndex Cos; 768+384 indexes; add/search/filtered_search/save/load; rebuild from cache if load fails. | phase-1C:L184-L191, L379-L385 |
| P1C-E023 | E/DEP | Chunking assumed as embed input; if undefined add AST tree-sitter sub-step. | phase-1C:L193-L197 |
| P1C-E024 | E/TEST | ≥22 unit tests: dims, cache, breaker, degradation, query validation, canary, path, missing model, semaphore, cache-hit, truncation, HNSW, isolation, concurrent embed, RRF k=60. | phase-1C:L254-L287, L321-L324 |
| P1C-E025 | E/TEST | Contracts: dual-vector HYBRID scores; project isolation via separate knowledge.db files (column filter FAIL). | phase-1C:L288-L294 |
| P1C-E026 | E/DET | Breaker uses ClockProvider not std::time. | phase-1C:L316-L319, L377-L378 |
| P1C-E027 | E/EVIDENCE | Step 3.5 tensor gate; tensor_format_verified:true in phase-1C-decisions.md; tokenizer crate if missing. | phase-1C:L330-L356 |
| P1C-E028 | E/CAP | Step 4 embed; VectorCache via single-writer SSB channel for SQLite writes. | phase-1C:L370-L376 |
| P1C-E029 | E/CAP | readiness_check canary within 5s doubles as ONNX warmup before Ready. | phase-1C:L386-L390 |
| P1C-E030 | E/FLOW | RRF BM25+code+text; score sum 1/(60+rank); k=60 fixed; CodeChunk scores fields. | phase-1C:L391-L396 |
| P1C-E031 | E/CONC | Cross-encoder Semaphore(1) queue 4; overflow returns unranked not error. | phase-1C:L399-L400 |
| P1C-E032 | E/EVENT | SearchMode change emits knowledge_degradation SSB (breaker transitions). | phase-1C:L401-L403 |
| P1C-E033 | E/OBS | QueryTrace DEBUG: query_id, stages, total_latency_ms, mode, result_count. | phase-1C:L404-L405 |
| P1C-E034 | E/LIMIT | vector/ code: no std::time, unwrap, TODO; no TurboQuant in 1C. | phase-1C:L409-L411, L455-L455 |
| P1C-E035 | E/DET | Equal cosine ties post-sort score DESC then chunk_id ASC. | phase-1C:L425-L428 |
| P1C-E036 | E/CONC | SSB DELETE for file_write must complete before query for post-event requests. | phase-1C:L433-L437 |
| P1C-E037 | E/OBS | Step 5: 100-chunk throughput; dual vs BM25; cache hit ≥90% BLOCK if under; phase-1C-measurements.md; dual≤BM25 INFO. | phase-1C:L469-L504 |
| P1C-E038 | E/DATA | Step 5 also names .elai_cc/models/graphcodebert/ and granite-embedding-small/ paths. | phase-1C:L463-L465 |
| P1C-E039 | E/OBS | Also measure warmup first-query latency; canary <5s; cross-encoder p99 <50ms/50 candidates. | phase-1C:L524-L531 |
| P1C-I001 | I/CFG | RRF k=60 const implies no operator-configurable fusion without code change. | phase-1C:L429-L432 |
| P1C-A001 | A/DATA | Exact tensor names/shapes open until Step 3.5. | phase-1C:L94-L97, L337-L344 |
| P1C-A002 | A/DEP | Tokenizer crate choice open until 3.5/4. | phase-1C:L346-L348 |
| P1C-A003 | A/FLOW | Chunking ownership unresolved if earlier phases silent. | phase-1C:L193-L197 |
| P1C-C001 | C/CFG | Model path .elai_cc/models/onnx/ vs Step 5 per-model subdirs. | phase-1C:L169-L172, L463-L465 |
| P1C-C002 | C/DEFER | Source defers SPLADE to 1G; MASTER summary says 1H. | phase-1C:L37-L47 |

Category coverage: modules/query fusion ✓; cache/index ✓; config/paths ✓; SSB events ✓; isolation ✓; breakers ✓; determinism ✓; metrics ✓; tests ✓; deferred TurboQuant/SPLADE ✓. No operator UI.

### phase-3A — full interactive FSM

Source: `docs/specs/guide/steps/phase-3A.txt` (343/343 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3A-E001 | E/STATUS | Prereq Phase 0-2; full 9-state FSM + tool/rule loading + evidence gates + INTERACTIVE HITL; AUTONOMOUS deferred 4G. | phase-3A:L1-L12 |
| P3A-E002 | E/STATE | Chain IDLE->PLANNING->FEASIBILITY_PROBE->TESTING_FIRST->EXECUTING->AUDITING->MEMORIZING->COMPLETE->IDLE. | phase-3A:L9-L11 |
| P3A-E003 | E/CAP | Deliverables 3A.1–3A.5: machine, filters, 7 gates, InteractiveGate, risk levels. | phase-3A:L23-L27 |
| P3A-E004 | E/STATE | FsmState: IDLE PLANNING FEASIBILITY_PROBE TESTING_FIRST EXECUTING AUDITING MEMORIZING COMPLETE CONFIGURING. | phase-3A:L39-L40 |
| P3A-E005 | E/FLOW | Valid matrix: IDLE->PLANNING|CONFIGURING; PLANNING->FEASIBILITY_PROBE|TESTING_FIRST; FEASIBILITY_PROBE->PLANNING|TESTING_FIRST; TESTING_FIRST->EXECUTING; EXECUTING->AUDITING|PLANNING; AUDITING->MEMORIZING|EXECUTING|PLANNING; MEMORIZING->PLANNING|COMPLETE; COMPLETE->IDLE; CONFIGURING->IDLE. | phase-3A:L41-L59 |
| P3A-E006 | E/DEFER | FEASIBILITY_PROBE full impl deferred 4F; transitions remain valid. | phase-3A:L44-L46, L169-L172 |
| P3A-E007 | E/EVIDENCE | AUDITING->EXECUTING/PLANNING PARTIALLY_GROUNDED SKILL.md not orchestrator.txt §2; SPEC_UPDATE_NEEDED. | phase-3A:L51-L56, L332-L335 |
| P3A-E008 | E/SURFACE | CONFIGURING via /config or /workflow; config_paths_only tools; no evidence gate; ->IDLE on completion. | phase-3A:L60-L63 |
| P3A-E009 | E/DATA | Transition carries source, target, phase label, timestamp, Vec<EvidenceArtifact>. | phase-3A:L64-L65 |
| P3A-E010 | E/CAP | token_budget_remaining()->usize for Phase 3E. | phase-3A:L66-L68 |
| P3A-E011 | E/DATA | Persist .elai_cc/sessions/{sid}/.fsm-state.msgpack. | phase-3A:L69-L69 |
| P3A-E012 | E/FLOW | ReplanRequest MINOR (no human) and MAJOR (human); max 1 per agent per phase. | phase-3A:L70-L71 |
| P3A-E013 | E/FAIL | Typed errors InvalidTransition, CorruptState, ReplanLimitExceeded, TransitionConflict. | phase-3A:L79-L88 |
| P3A-E014 | E/CONC | Concurrent transitions: one succeeds, other TransitionConflict. | phase-3A:L86-L88 |
| P3A-E015 | E/CAP | Per-state tool filters: PLANNING no edit/bash; FEASIBILITY stub write <50+bash+web; TESTING_FIRST no edit/bash; EXECUTING no planning; AUDITING no edit/planning; MEMORIZING memory-bank only; CONFIGURING config paths; IDLE all; COMPLETE read-only. | phase-3A:L110-L123 |
| P3A-E016 | E/LIMIT | COMPLETE read-only and IDLE unrestricted UNGROUNDED vs orchestrator.txt §2. | phase-3A:L121-L123, L141-L142 |
| P3A-E017 | E/FAIL | ToolRegistry::execute returns ToolBlockedByState for blocked tools. | phase-3A:L125-L126 |
| P3A-E018 | E/CAP | get_active_rules(state) via RuleEngine. | phase-3A:L124-L124 |
| P3A-E019 | E/FLOW | 7 gates: knowledge READY|DEGRADED; zero ungrounded; tests fail right reasons; ≥1 FAIL->PASS; MINOR replan; zero BLOCKs; DKG+changelog. | phase-3A:L156-L163 |
| P3A-E020 | E/CAP | EvidenceGate trait; EvidenceError {gate_name, missing_artifacts, suggestion}. | phase-3A:L164-L166 |
| P3A-E021 | E/RETRY | DAIRA: if output >budget*0.3 reduce scope retry; max 3. | phase-3A:L167-L168 |
| P3A-E022 | E/DEFER | FEASIBILITY transitions ungated in 3A; full gating 4F. | phase-3A:L169-L172 |
| P3A-E023 | E/FAIL | DEGRADED allows IDLE->PLANNING with warning; UNAVAILABLE blocks. | phase-3A:L183-L185, L196-L196 |
| P3A-E024 | E/SURFACE | INTERACTIVE gate: structured EvidenceSummary present_gate; not prose. | phase-3A:L205-L209 |
| P3A-E025 | E/FLOW | GateDecision APPROVE/REJECT(no change)/MODIFY(recheck)/REDIRECT(valid target). | phase-3A:L210-L214 |
| P3A-E026 | E/CAP | InteractiveGate trait; CLI impl; MockInteractiveGate tests. | phase-3A:L215-L216 |
| P3A-E027 | E/CFG | Timeout configurable; default none INTERACTIVE; AUTONOMOUS timeout 4G. | phase-3A:L217-L218 |
| P3A-E028 | E/STATE | RiskLevel LOW/MEDIUM/HIGH on transitions; examples LOW MEMORIZING->next; MEDIUM PLANNING->TESTING_FIRST; HIGH destructive. | phase-3A:L251-L257 |
| P3A-E029 | E/FLOW | INTERACTIVE: LOW auto-advance; MEDIUM/HIGH pause. | phase-3A:L255-L261 |
| P3A-E030 | E/CFG | [fsm.risk_overrides] in elai_cc.toml. | phase-3A:L258-L259 |
| P3A-E031 | E/TEST | 9-state msgpack; 81 transition pairs; per-state tool tests; 7 gate tests; 4 decision tests; risk tests; replan limit; proptest; adversarial. | phase-3A:L282-L303 |
| P3A-E032 | E/OBS | SLOs: transition <1ms; evidence gate <10ms; tool filter <0.1ms; human latency log-only. | phase-3A:L305-L309 |
| P3A-E033 | E/DEP | SSB phase_transition not a 3A deliverable; emits after 3B. | phase-3A:L342-L343 |
| P3A-I001 | I/CAP | EvidenceGate/InteractiveGate traits ungrounded testability assumptions. | phase-3A:L191-L192, L236-L237 |
| P3A-A001 | A/FLOW | REDIRECT invalid-target handling partially specified. | phase-3A:L230-L232 |
| P3A-A002 | A/CFG | Default RiskLevel for every transition beyond examples not fully tabulated. | phase-3A:L251-L254 |
| P3A-A003 | A/DEP | SKILL.md path for AUDITING recovery unresolved among multiple skill files. | phase-3A:L51-L56 |
| P3A-C001 | C/EVIDENCE | orchestrator.txt §2 omits AUDITING recovery arms SKILL.md requires; SPEC_UPDATE_NEEDED. | phase-3A:L51-L56 |

Category coverage: FSM/gates/HITL ✓; /config /workflow ✓; states ✓; risk config ✓; tool blocks ✓; DAIRA retry ✓; concurrency ✓; latency ✓; tests ✓; deferred 4F/4G/3B ✓.

### phase-3J-BASELINE — token efficiency benchmark harness

Source: `docs/specs/guide/steps/phase-3J-BASELINE.txt` (146/146 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3JB-E001 | E/STATUS | Standalone token-economy harness for every ModelCapabilityProfile tier; writes metrics/capability_profile.toml for runtime slot/compression/budget. | phase-3J-BASELINE:L1-L10 |
| P3JB-E002 | E/LIMIT | Must not boot elai runtime (no SSB/DKG/orchestrator); fixture TOML + HTTP endpoint only. | phase-3J-BASELINE:L8-L10 |
| P3JB-E003 | E/CFG | Tiers: micro_8gb, micro_3b_12gb, micro_7b_12gb, awq_3b_8gb, awq_3b_12gb, awq_7b_12gb, large_32b_48gb, cloud_200k. | phase-3J-BASELINE:L12-L21 |
| P3JB-E004 | E/CAP | New baseline_harness.py; does not overwrite run_benchmark.py. | phase-3J-BASELINE:L24-L27 |
| P3JB-E005 | E/DATA | Fixtures fixtures/*.toml; roster elai_cc.toml [model_capability]; endpoint /v1/messages; output capability_profile.toml. | phase-3J-BASELINE:L28-L31 |
| P3JB-E006 | E/DATA | Fixture ≥10 prompts/tier; schema tier + prompts[{id,text,expect_json}]. | phase-3J-BASELINE:L41-L47 |
| P3JB-E007 | E/OBS | Per prompt: tokens_in/out, slot_utilization_pct, compression_ratio, ttft_ms, p99_latency_ms, cost_per_run_usd. | phase-3J-BASELINE:L49-L56 |
| P3JB-E008 | E/TEST | One output row per tier from [model_capability]. | phase-3J-BASELINE:L61-L63 |
| P3JB-E009 | E/TEST | RTX 3090 targets: micro_7b_12gb util≥0.82 compression≥0.65; cloud_200k util≥0.90. | phase-3J-BASELINE:L65-L67 |
| P3JB-E010 | E/TEST | REPAIR_CONTEXT twice same prompts: tokens_in must not rise >15%; else repair_context_regression row blocks plans. | phase-3J-BASELINE:L69-L74 |
| P3JB-E011 | E/TEST | JSON-schema 20 expect_json prompts: 0 invalid JSON under ConstrainedDecodingConfig. | phase-3J-BASELINE:L76-L79 |
| P3JB-E012 | E/DEFER | EBNF/grammar constrained decoding deferred; sidecar JSON-schema-only via LMFE. | phase-3J-BASELINE:L80-L86 |
| P3JB-E013 | E/LIMIT | stdlib-only harness imports; no requests; no in-tree crates. | phase-3J-BASELINE:L91-L93 |
| P3JB-E014 | E/LIMIT | No ShortCoder import; compression_ratio_estimate via output/input ratio measurement_method=output_token_ratio. | phase-3J-BASELINE:L94-L100 |
| P3JB-E015 | E/FAIL | Exit 0 pass; 1 gate fail; 2 harness error. | phase-3J-BASELINE:L103-L104 |
| P3JB-E016 | E/DATA | Produces harness, capability_profile.toml, 8 tier fixtures, json_schema.toml. | phase-3J-BASELINE:L109-L113 |
| P3JB-E017 | E/INT | Manual python invoke; optional non-blocking CI job bench-capability. | phase-3J-BASELINE:L118-L120 |
| P3JB-E018 | E/SURFACE | CLI: python baseline_harness.py --profile micro_7b_12gb; expect exit 0 and util≥0.82. | phase-3J-BASELINE:L125-L128 |
| P3JB-E019 | E/TEST | Deterministic checks: no forbidden imports; fixture/template present; model_capability sections counted. | phase-3J-BASELINE:L130-L145 |
| P3JB-I001 | I/OBS | Eight tiers imply eight fixtures though only micro_7b named in checks. | phase-3J-BASELINE:L12-L21, L112-L112 |
| P3JB-A001 | A/CFG | Prompt content/length/seed for ≥10 prompts unspecified. | phase-3J-BASELINE:L41-L47 |
| P3JB-A002 | A/OBS | Token-ratio compression vs runtime ShortCoder tolerance unspecified. | phase-3J-BASELINE:L53-L53, L94-L100 |

Category coverage: CLI harness ✓; fixtures/metrics ✓; gates ✓; standalone limits ✓; deferred grammar ✓. No interactive UI.

### phase-4K — native Tauri dashboard MVP

Source: `docs/specs/guide/steps/phase-4K.txt` (427/427 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4K-E001 | E/STATUS | Prereq 3A/3B/3J; 4A-4J recommended; dashboard reads SSB. | phase-4K:L1-L4 |
| P4K-E002 | E/SURFACE | Native structured control surface not chatbot; inputs via FSM classification; MVP interactive mode. | phase-4K:L8-L12 |
| P4K-E003 | E/CAP | Tauri 2.10 / build 2.5; SsbViewer commands; Svelte; no HTTP/REST/WebSocket/CORS/ports. | phase-4K:L14-L17 |
| P4K-E004 | E/FLOW | Frontend polls invoke() every 500ms; cargo tauri build single .exe WebView2. | phase-4K:L18-L19 |
| P4K-E005 | E/EVIDENCE | tauri 2.10.3, tauri-build 2.5.6, tauri-cli 2.10.1 verified 2026-04-16. | phase-4K:L21-L26 |
| P4K-E006 | E/SURFACE | Visual: teal #1B7A6E on charcoal #0f1318; JetBrains Mono; density over whitespace. | phase-4K:L28-L31 |
| P4K-E007 | E/FAIL | SSB schema drift mitigated by typed SsbViewer compile-time deserialization. | phase-4K:L40-L43 |
| P4K-E008 | E/DEP | WebView2 required; bootstrapper in installer; README + clean Win11 test. | phase-4K:L44-L48 |
| P4K-E009 | E/LIMIT | Bundle 3-8MB; frontend <100KB JS. | phase-4K:L49-L52 |
| P4K-E010 | E/LIMIT | 500ms poll accepted; later app.emit push upgrade path. | phase-4K:L55-L59 |
| P4K-E011 | E/CAP | SsbViewer SQLITE_OPEN_READONLY WAL connection separate from writer. | phase-4K:L70-L72 |
| P4K-E012 | E/CAP | Queries: latest_fsm_state, active_session, phase_progress, recent_actions, agent_actions, pending_interventions, file_changes, reasoning_chain. | phase-4K:L73-L81 |
| P4K-E013 | E/EVENT | INTERVENTION_REQUEST: step_id, type ApproveChange|ReplanApproval|ErrorRecovery, description, optional diff, options Accept|Decline|Retry(String)|Skip. | phase-4K:L82-L86 |
| P4K-E014 | E/EVENT | INTERVENTION_RESPONSE: request_id, chosen, optional user_note. | phase-4K:L87-L88 |
| P4K-E015 | E/DEFER | Post-MVP: HEALTH_SNAPSHOT, WINDOW_STATE, health_snapshot/window_state/cost_summary queries. | phase-4K:L89-L91 |
| P4K-E016 | E/DATA | SsbActionSummary first 100 chars; list pagination offset+limit; query <10ms on 100K rows. | phase-4K:L92-L95 |
| P4K-E017 | E/CAP | dashboard crate structure: Cargo.toml, tauri.conf.json, build.rs, main.rs, commands.rs, src-frontend Svelte/Vite. | phase-4K:L118-L133 |
| P4K-E018 | E/CFG | identifier dev.elai.dashboard; window ELAI 1200×800 min 900×600; dist path; devUrl http://localhost:1420; bundle.active. | phase-4K:L134-L139 |
| P4K-E019 | E/SURFACE | Tauri command get_fsm_state. | phase-4K:L140-L141 |
| P4K-E020 | E/SURFACE | Tauri command get_phase_progress. | phase-4K:L142-L142 |
| P4K-E021 | E/SURFACE | Tauri command get_recent_actions(limit,offset). | phase-4K:L143-L143 |
| P4K-E022 | E/SURFACE | Tauri command get_pending_interventions. | phase-4K:L144-L144 |
| P4K-E023 | E/SURFACE | Tauri command get_file_changes(session_id). | phase-4K:L145-L145 |
| P4K-E024 | E/SURFACE | Tauri command submit_intervention_response(request_id,chosen,note). | phase-4K:L146-L146 |
| P4K-E025 | E/INT | SsbViewer as tauri::State; generate_handler registers six MVP commands. | phase-4K:L149-L156 |
| P4K-E026 | E/SURFACE | CLI elai --dashboard; focus existing window if already running. | phase-4K:L157-L158 |
| P4K-E027 | E/SURFACE | CSS tokens bg-void/base/surface/hover/input; teal/purple scales; semantic pass/warn/fail/info; mono font scale xs–xl. | phase-4K:L159-L167 |
| P4K-E028 | E/FAIL | SsbViewer open fail: dialog + graceful exit; commands never unwrap. | phase-4K:L177-L181 |
| P4K-E029 | E/SURFACE | Project selector: logo, subtitle "a deterministic development engine", two creation paths. | phase-4K:L197-L203 |
| P4K-E030 | E/SURFACE | + New Project: name+parent picker; create_dir_all; list add; template+task without leaving app. | phase-4K:L204-L208 |
| P4K-E031 | E/SURFACE | Open Existing: native folder dialog. | phase-4K:L209-L210 |
| P4K-E032 | E/SURFACE | Recent projects: name/path/last opened; max 10; hover style; source elai_cc.toml [projects] or ~/.elai_cc/projects.json. | phase-4K:L211-L214 |
| P4K-E033 | E/SURFACE | Commands list_projects, select_project, create_project; name alnum-_; inline red errors. | phase-4K:L215-L224 |
| P4K-E034 | E/SURFACE | Template picker Svelte/React/Astro/Vanilla from templates/web/manifest.toml; list_templates. | phase-4K:L225-L229 |
| P4K-E035 | E/SURFACE | Task input: header, placeholder, Start purple, Enter submit; submit_task command. | phase-4K:L230-L237 |
| P4K-E036 | E/FAIL | Start disabled if empty; minimum 5 characters. | phase-4K:L262-L263 |
| P4K-E037 | E/SURFACE | Workspace: plan tree left 320px + activity feed right; draggable divider; min 280px; double-click 50/50. | phase-4K:L277-L280 |
| P4K-E038 | E/SURFACE | Status bar 24px: project | FSM state | session; EXECUTING teal. | phase-4K:L281-L282 |
| P4K-E039 | E/SURFACE | Plan tree phases/steps/baby-steps; --> active; done/active/pending styles; expand active; elapsed times. | phase-4K:L283-L291 |
| P4K-E040 | E/SURFACE | Feed [reasoning] teal-ghost collapsed 2-3 lines. | phase-4K:L294-L295 |
| P4K-E041 | E/SURFACE | Feed [edit] path+lines +N -M. | phase-4K:L296-L296 |
| P4K-E042 | E/SURFACE | Feed [validate] checkmark/x one-liner. | phase-4K:L297-L297 |
| P4K-E043 | E/SURFACE | Feed [test] pass/fail counts. | phase-4K:L298-L298 |
| P4K-E044 | E/SURFACE | Feed [preview] teal link opens system browser via shell::open. | phase-4K:L299-L301, L362-L368 |
| P4K-E045 | E/SURFACE | Step headers; auto-scroll; Jump to latest pill when scrolled up. | phase-4K:L302-L305 |
| P4K-E046 | E/FLOW | Poll 500ms phase_progress+recent_actions; skip if in-flight. | phase-4K:L306-L308, L325-L327 |
| P4K-E047 | E/DATA | Map SSB types reasoning_trace, file_write, tool_result, test_outcome, phase_complete, preview_available. | phase-4K:L309-L311 |
| P4K-E048 | E/LIMIT | Render last 500 entries; older lazy-load; virtualization post-MVP. | phase-4K:L322-L324 |
| P4K-E049 | E/SURFACE | Diff viewer on ApproveChange: unified teal add / fail remove / sticky header. | phase-4K:L343-L350 |
| P4K-E050 | E/SURFACE | Diff Accept / Decline / Retry+note; per-hunk Accept/Decline on hover. | phase-4K:L351-L355 |
| P4K-E051 | E/SURFACE | Multi-file vertical list navigation. | phase-4K:L356-L357 |
| P4K-E052 | E/FLOW | Responses write INTERVENTION_RESPONSE; close viewer; feed [accepted]/[declined]. | phase-4K:L358-L361 |
| P4K-E053 | E/SURFACE | Tauri command get_action_detail(action_id) full payload. | phase-4K:L369-L372 |
| P4K-E054 | E/FAIL | Truncate long lines 200 chars; multi-intervention badge Previous/Next. | phase-4K:L382-L390 |
| P4K-E055 | E/RECOVERY | Partial hunk accept may fail compile → new intervention with error context. | phase-4K:L385-L387 |
| P4K-E056 | E/TEST | Completion criteria: native window, project/template/task, plan+feed, per-hunk diffs, preview, <10ms queries, <1s launch. | phase-4K:L400-L409 |
| P4K-E057 | E/DEFER | Post-MVP MONITOR/EXPLORE/notifications/tabs/polish/sidebar/slash palette/shortcuts/autonomous/push/cost_summary. | phase-4K:L412-L427 |
| P4K-I001 | I/SURFACE | PhaseStatus and InterventionRequest fields imply plan-tree/card data contracts. | phase-4K:L76-L76, L83-L86 |
| P4K-A001 | A/INT | submit_task "writes SSB / triggers orchestrator" path underspecified. | phase-4K:L235-L237 |
| P4K-A002 | A/DATA | projects list dual source without precedence. | phase-4K:L214-L214 |
| P4K-A003 | A/SURFACE | Diff viewer replace feed vs modal open. | phase-4K:L345-L345 |

Category coverage: windows/panels/buttons/commands ✓; workflows ✓; SSB schemas ✓; visual config ✓; read-only security ✓; failures ✓; performance ✓; deferred post-MVP ✓.

### phase-4M — multi-runtime backend abstraction

Source: `docs/specs/guide/steps/phase-4M.txt` (465/465 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4M-E001 | E/STATUS | NEW 2026-05-10; prereq 3J+4A; 4G recommended. | phase-4M:L1-L4 |
| P4M-E002 | E/CAP | One RuntimeAdapter + adapters + resolve_backend; Anthropic-shaped messages; adapters translate. | phase-4M:L8-L24 |
| P4M-E003 | E/LIMIT | Non-goals: no new runtimes; no keyword skills; no LiteLLM; extend LlmProvider. | phase-4M:L32-L39 |
| P4M-E004 | E/STATE | RuntimeBackend 11: Sidecar AnthropicApi ClaudeCode CodexCli OpenCode Hermes GeminiCli KiroCli CopilotCli Ollama Vllm. | phase-4M:L60-L72 |
| P4M-E005 | E/CAP | RuntimeAdapter: complete, stream, capabilities, health_check; Send+Sync. | phase-4M:L74-L93 |
| P4M-E006 | E/CAP | capabilities: tools/streaming/vision/max_context/max_output/json_schema_mode/prefill/caching/thinking_budget. | phase-4M:L87-L90 |
| P4M-E007 | E/DATA | RuntimeFactoryConfig: backend, cli_path, base_url, api_key, model, permission_mode, profile, cwd, timeouts. | phase-4M:L99-L110 |
| P4M-E008 | E/CFG | PermissionMode AcceptEdits|Plan|DangerouslySkipPermissions|Default; default AcceptEdits; skip-permissions needs explicit override. | phase-4M:L112-L117, L436-L439 |
| P4M-E009 | E/FAIL | ResolveBackendError {candidate, supported full list}. | phase-4M:L119-L122 |
| P4M-E010 | E/CFG | PerRoleRuntime {role,backend,fallback_chain}; [agents.<role>.runtime]. | phase-4M:L124-L134 |
| P4M-E011 | E/EVIDENCE | 4M.0 ingest ouroboros MIT → ouroboros-runtime-interface.md MEDIUM; ≥7 adapters. | phase-4M:L138-L161 |
| P4M-E012 | E/CAP | 4M.1 types + resolve_backend + create_runtime stubs; [runtime] and per-role schema; existing Sidecar/AnthropicApi stay functional. | phase-4M:L163-L184 |
| P4M-E013 | E/INT | Alias RuntimeAdapter=LlmProvider if vtable compatible else wrapper without reimplement. | phase-4M:L95-L97, L169-L172 |
| P4M-E014 | E/CAP | 4M.2 SidecarRuntime + AnthropicApiRuntime /v1/messages; health canary Ready|Degraded|Unavailable. | phase-4M:L204-L217 |
| P4M-E015 | E/CFG | Migrate [backend] local|api|hybrid|auto → [runtime] sidecar|anthropic_api|...; deprecated aliases one release with warning. | phase-4M:L218-L221 |
| P4M-E016 | E/CAP | 4M.3 seven CLI adapters: spawn+pipe; translate stdin; parse stdout. | phase-4M:L239-L253 |
| P4M-E017 | E/CFG | permission_mode flags per backend; honor startup and stdout_idle timeouts. | phase-4M:L254-L257 |
| P4M-E018 | E/SEC | CLI adapters MUST Tier C isolation; stdout sanitized via 4A IPC before prompt context. | phase-4M:L264-L268, L286-L289 |
| P4M-E019 | E/FAIL | Missing CLI Unavailable not panic; hang kill on idle timeout; nonzero exit always Err; Plan mode no Edit/Write. | phase-4M:L277-L296 |
| P4M-E020 | E/CAP | 4M.4 Ollama /api/chat; Vllm /v1/chat/completions; health /api/tags or /health. | phase-4M:L304-L322 |
| P4M-E021 | E/CFG | Defaults: designer/auditor/researcher/media_validator AnthropicApi; executor/memory_writer/context_compiler/prompt_engineer Sidecar. | phase-4M:L331-L339 |
| P4M-E022 | E/RECOVERY | Runtime fallback on Unavailable; SSB runtime_fallback; exhaust ESCALATE INTERACTIVE or HALT AUTONOMOUS; before model fallback. | phase-4M:L340-L365 |
| P4M-E023 | E/SURFACE | CLI /runtime show. | phase-4M:L344-L345 |
| P4M-E024 | E/SURFACE | CLI /runtime set <role> <backend>. | phase-4M:L346-L346 |
| P4M-E025 | E/SURFACE | CLI /runtime fallback <role> <b1>,<b2>. | phase-4M:L347-L347 |
| P4M-E026 | E/STATE | PerRoleRuntime per-session SessionState not global. | phase-4M:L353-L356 |
| P4M-E027 | E/EVENT | runtime_fallback SSB reason+chain; dashboard yellow; CloudSpendTracker integration. | phase-4M:L357-L360, L443-L447 |
| P4M-E028 | E/SURFACE | RuntimePanel/Settings: per-role backend, chain, live health g/y/r, Test backend button. | phase-4M:L376-L380 |
| P4M-E029 | E/OBS | phase_4M.toml: p50/p99, tokens/sec, cost, fallback-fire rate, health uptime %. | phase-4M:L381-L385 |
| P4M-E030 | E/DATA | Migration guide phase-4M-migration.md old [backend]→new [runtime]. | phase-4M:L386-L388 |
| P4M-E031 | E/INT | RuntimeClient must wire ssb, fsm, clock; no None after construction. | phase-4M:L463-L465 |
| P4M-E032 | E/EVIDENCE | Grounding: ouroboros MIT pattern; 7 CLIs; PermissionMode; Tier C ELAI-specific; Ollama/vLLM ELAI-specific; per-role PARTIAL; fallback OUR DESIGN. | phase-4M:L399-L429 |
| P4M-E033 | E/LIMIT | R1 CLI drift probe-cache capabilities; R4 cost risk via SSB+dashboard+cost tracker. | phase-4M:L432-L447 |
| P4M-I001 | I/CFG | TOML examples imply snake_case resolve_backend names. | phase-4M:L131-L134, L175-L178 |
| P4M-A001 | A/CAP | RuntimeAdapter alias vs new trait open at execution. | phase-4M:L169-L172 |
| P4M-A002 | A/SURFACE | RuntimePanel vs SettingsWindow placement open. | phase-4M:L376-L376 |
| P4M-A003 | A/CFG | Full 11 name strings and hybrid/auto migration beyond local/api incomplete. | phase-4M:L218-L221 |
| P4M-A004 | A/INT | SlashCommand::Runtime full parser wiring thin. | phase-4M:L351-L351 |

Category coverage: adapters ✓; /runtime CLI ✓; RuntimePanel ✓; fallback ✓; config ✓; Tier C security ✓; health ✓; metrics ✓; non-goals ✓.

### Cross-source interactions (PHASE-001)

| Interaction ID | Class | Explicit discovery or bounded interpretation | Source atoms |
|---|---|---|---|
| PHASE-001/IX-001 | E | Phase 0 nine-crate split, LlmProvider, minimal FSM, visibility, msgpack, contracts, entropy quarantine are substrate for later phases here. | P0-E002, P0-E046, P0-E049–P0-E065 |
| PHASE-001/IX-002 | E | Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience. | P0-E015–P0-E017; P1C-E018, P1C-E029 |
| PHASE-001/IX-003 | E | Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine. | P0-E060–P0-E064; P1C-E026 |
| PHASE-001/IX-004 | E | Phase 1C SSB file_write invalidation and knowledge_degradation require Phase 3B SSB (outside chunk) but specify production wiring. | P1C-E021, P1C-E032 |
| PHASE-001/IX-005 | E | Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G. | P0-E052; P3A-E001–P3A-E029 |
| PHASE-001/IX-006 | E | Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG. | P3A-E019, P3A-E023; P1C-E029 |
| PHASE-001/IX-007 | E | Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal. | P3A-E033; P4K-E012, P4K-E047 |
| PHASE-001/IX-008 | E | Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI. | P3A-E024–P3A-E025; P4K-E013–P4K-E014, P4K-E049–P4K-E052 |
| PHASE-001/IX-009 | E | Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface. | P4K-E011–P4K-E026 |
| PHASE-001/IX-010 | E | Phase 3J-BASELINE fills capability_profile.toml for ModelCapabilityProfile without booting 3A/4K. | P3JB-E001–P3JB-E002, P3JB-E007–P3JB-E009 |
| PHASE-001/IX-011 | E | 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability. | P3JB-E011; P4M-E006 |
| PHASE-001/IX-012 | E | Phase 4M migrates 3J backends and Ollama/vLLM into RuntimeAdapter; requires 4A Tier C for CLI adapters. | P4M-E001, P4M-E014–P4M-E020 |
| PHASE-001/IX-013 | E | Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace. | P0-E055; P4K-E026; P4M-E023–P4M-E028 |
| PHASE-001/IX-014 | E | Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback. | P3A-E001; P4M-E022 |
| PHASE-001/IX-015 | E | Slash command evolution: Phase 0 /task|/config|/continue|/explore|/status; 3A /config|/workflow CONFIGURING; 4M /runtime family. | P0-E055; P3A-E008; P4M-E023–P4M-E025 |
| PHASE-001/IX-016 | I | RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here. | P1C-E029; P4M-E014, P4M-E028 |
| PHASE-001/IX-017 | E | 1C three-ranker RRF stays additive for later SPLADE/cross-encoder; semaphore overflow unranked. | P1C-E007, P1C-E030–P1C-E031 |
| PHASE-001/IX-018 | E | Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis. | P0-E053, P0-E056; P3A-E017 |

### Ambiguity and conflict register (PHASE-001)

| Register ID | Class | Issue | Source atoms |
|---|---|---|---|
| PHASE-001/AC-001 | C | TokenUsage field set differs design (4 fields) vs 0.2c shim (2 fields). | P0-C001 |
| PHASE-001/AC-002 | A | prompt.rs REPLACE vs STRIP open. | P0-A001 |
| PHASE-001/AC-003 | A | remote.rs DEFERRED vs STRIP open. | P0-A002 |
| PHASE-001/AC-004 | A | sandbox whole-file vs types-only open. | P0-A003 |
| PHASE-001/AC-005 | C | 1C model path onnx/ vs Step 5 per-model subdirs. | P1C-C001 |
| PHASE-001/AC-006 | C | SPLADE deferral 1G (source) vs 1H (MASTER summary). | P1C-C002 |
| PHASE-001/AC-007 | C | AUDITING recovery transitions need orchestrator.txt SPEC_UPDATE. | P3A-C001 |
| PHASE-001/AC-008 | A | UNGROUNDED COMPLETE/IDLE tool policies and gate traits. | P3A-E016, P3A-I001 |
| PHASE-001/AC-009 | A | REDIRECT invalid-target and full risk matrix incomplete. | P3A-A001–P3A-A002 |
| PHASE-001/AC-010 | A | 3J-BASELINE fixture prompts and compression tolerance unspecified. | P3JB-A001–P3JB-A002 |
| PHASE-001/AC-011 | A | 4K submit_task path, projects precedence, diff modal vs replace. | P4K-A001–P4K-A003 |
| PHASE-001/AC-012 | A | 4M adapter alias, panel placement, name list, hybrid/auto detail. | P4M-A001–P4M-A003 |

### Summary deltas vs MASTER (PHASE-001)

Summary artifact: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. Anchors use `01-phase:` line numbers.

### phase-0 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P0-001 | **ADD** full risk register R17–R25 with LOC/import facts and stale comparison-doc warning. | 01-phase:L24-L42 | P0-E004–P0-E012, P0-E068 |
| PHASE-001/SD-P0-002 | **ADD** workspace path migration (rust/→crates/), archive semantics, strip-before-restructure rationale. | 01-phase:L24-L33 | P0-E013–P0-E014 |
| PHASE-001/SD-P0-003 | **ADD** ONNX path, blake2 setup, DEGRADED BM25_ONLY, models/** hidden_paths. | 01-phase:L24-L42 | P0-E015–P0-E017 |
| PHASE-001/SD-P0-004 | **ADD** ordered strip sequence, KEEP/STRIP slash lists, TokenUsage shim vs core fields. | 01-phase:L29-L32 | P0-E024–P0-E029, P0-C001 |
| PHASE-001/SD-P0-005 | **ADD** module mapping LOCs, tools 2000-LOC cap, cli <500 soft, nine crates, contracts TDD red phase. | 01-phase:L31-L39 | P0-E032–P0-E048, P0-E066 |
| PHASE-001/SD-P0-006 | **ADD** LlmProvider streaming complete, Ollama endpoints, allowlist sanitization, hidden_paths inventory, smoke skip. | 01-phase:L36-L44 | P0-E049–P0-E059 |
| PHASE-001/SD-P0-007 | **ADD** ClockProvider/EntropySource shapes, RuntimeContext, CI exclusions, ELAI_SEED=42 dual-run proof. | 01-phase:L40-L41 | P0-E060–P0-E065 |
| PHASE-001/SD-P0-008 | **FLAG** open prompt/remote/sandbox extraction choices omitted from summary. | 01-phase:L24-L42 | P0-A001–P0-A004 |

### phase-1C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P1C-001 | **ADD** R22/R26/R27/R29 mitigations and HYBRID→SPARSE_ONLY→BM25_ONLY cascade. | 01-phase:L69-L83 | P1C-E003–P1C-E006, P1C-E012–P1C-E013 |
| PHASE-001/SD-P1C-002 | **CORRECT** SPLADE deferral: original 1G vs summary 1H. | 01-phase:L74-L74 | P1C-E007, P1C-C002 |
| PHASE-001/SD-P1C-003 | **ADD** TurboQuant grounded params and ban on 1C quantization. | 01-phase:L83-L83 | P1C-E015, P1C-E034 |
| PHASE-001/SD-P1C-004 | **ADD** DualVector/OrtSession/UsearchIndex APIs, 512 truncation, filtered_search rebuild, RRF k=60 fixed. | 01-phase:L75-L81 | P1C-E017–P1C-E022, P1C-E030, P1C-E035 |
| PHASE-001/SD-P1C-005 | **ADD** ≥22 tests, physical DB isolation, QueryTrace, knowledge_degradation triggers, write-before-query race rule. | 01-phase:L79-L81 | P1C-E024–P1C-E025, P1C-E032–P1C-E036 |
| PHASE-001/SD-P1C-006 | **ADD** Step 3.5 tensor gate, Step 5 fields, cache_hit BLOCK <90%, path naming tension. | 01-phase:L80-L82 | P1C-E027, P1C-E037–P1C-E039, P1C-C001 |
| PHASE-001/SD-P1C-007 | **QUALIFY** summary fastembed pin: original forbids adding fastembed with ort. | 01-phase:L74-L74 | P1C-E014 |

### phase-3A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P3A-001 | **ADD** full transition matrix including CONFIGURING, FEASIBILITY revise, ReplanRequest, SKILL.md AUDITING arms + SPEC_UPDATE. | 01-phase:L233-L244 | P3A-E005–P3A-E008, P3A-C001 |
| PHASE-001/SD-P3A-002 | **ADD** transition payload, token_budget_remaining, msgpack path, TransitionConflict, typed errors. | 01-phase:L237-L237 | P3A-E009–P3A-E014 |
| PHASE-001/SD-P3A-003 | **ADD** full tool-filter table including FEASIBILITY stub-write and UNGROUNDED COMPLETE/IDLE. | 01-phase:L238-L239 | P3A-E015–P3A-E017 |
| PHASE-001/SD-P3A-004 | **ADD** DAIRA max-3, EvidenceError shape, DEGRADED-allows-with-warning, FEASIBILITY ungated. | 01-phase:L240-L242 | P3A-E019–P3A-E023 |
| PHASE-001/SD-P3A-005 | **ADD** structured EvidenceSummary, trait mockability, no-timeout default, risk examples + overrides. | 01-phase:L243-L244 | P3A-E024–P3A-E030 |
| PHASE-001/SD-P3A-006 | **ADD** 81-pair tests, latency SLOs, SSB phase_transition non-deliverable note. | 01-phase:L233-L244 | P3A-E031–P3A-E033 |

### phase-3J-BASELINE deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P3JB-001 | **ADD entire file**: MASTER does not index phase-3J-BASELINE.txt. | 01-phase:L360-L375, L655 | P3JB-E001–P3JB-E019 |
| PHASE-001/SD-P3JB-002 | **ADD** standalone stdlib harness, 8 tiers, 7 metrics, REPAIR_CONTEXT ±15%, JSON-schema 20-prompt, exit 0/1/2. | 01-phase:L655 | P3JB-E002–P3JB-E015 |
| PHASE-001/SD-P3JB-003 | **ADD** non-overwrite run_benchmark.py, optional CI bench-capability, ShortCoder ban + output_token_ratio. | 01-phase:L655 | P3JB-E004, P3JB-E014, P3JB-E017 |

### phase-4K deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P4K-001 | **CORRECT/EXPAND** summary phase ribbon/commit history: MVP is project selector + task + plan tree + activity feed + diff viewer. | 01-phase:L528-L528 | P4K-E029–P4K-E053 |
| PHASE-001/SD-P4K-002 | **ADD** full visual tokens, window geometry, identifier, Vite :1420, elai --dashboard single-instance focus. | 01-phase:L522-L528 | P4K-E006, P4K-E018, P4K-E026–P4K-E027 |
| PHASE-001/SD-P4K-003 | **ADD** distinct Tauri commands including projects/templates/submit_task/get_action_detail. | 01-phase:L527-L527 | P4K-E019–P4K-E024, P4K-E033–P4K-E035, P4K-E053 |
| PHASE-001/SD-P4K-004 | **ADD** intervention options, per-hunk controls, multi-file nav, preview open, 500-entry cap, in-flight poll skip, 200-char truncate. | 01-phase:L525-L526 | P4K-E013–P4K-E014, P4K-E040–P4K-E055 |
| PHASE-001/SD-P4K-005 | **ADD** project name validation, min 5-char task, projects.json corrupt empty-list, post-MVP deferred list. | 01-phase:L528-L529 | P4K-E033, P4K-E036, P4K-E057 |
| PHASE-001/SD-P4K-006 | **ADD** WebView2 bootstrapper, <100KB JS, typed schema drift compile-fail, graceful open-fail dialog. | 01-phase:L529-L529 | P4K-E007–P4K-E010, P4K-E028 |

### phase-4M deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P4M-001 | **ADD** summary stops at 4M.1; original continues 4M.2–4M.6 adapters/hybrid/CLI/dashboard/metrics/migration. | 01-phase:L531-L541, L653 | P4M-E014–P4M-E031 |
| PHASE-001/SD-P4M-002 | **ADD** PermissionMode defaults, ResolveBackendError, capabilities fields, factory timeouts. | 01-phase:L537-L539 | P4M-E006–P4M-E009 |
| PHASE-001/SD-P4M-003 | **ADD** default per-role map, runtime vs model fallback order, ESCALATE/HALT, per-session state. | 01-phase:L539-L539 | P4M-E021–P4M-E027 |
| PHASE-001/SD-P4M-004 | **ADD** /runtime show|set|fallback and RuntimePanel Test-backend canary. | 01-phase:L531-L541 | P4M-E023–P4M-E028 |
| PHASE-001/SD-P4M-005 | **ADD** Tier C mandate, stdout sanitization, missing-binary Unavailable, exit_code fail-closed, RuntimeClient wiring. | 01-phase:L531-L541 | P4M-E018–P4M-E019, P4M-E031 |
| PHASE-001/SD-P4M-006 | **ADD** [backend]→[runtime] deprecation, phase_4M.toml metrics, ouroboros grounding distinctions, R1–R4. | 01-phase:L533-L541 | P4M-E015, P4M-E029–P4M-E033 |

## PHASE-002 — source `DEEP-PHASE-002.discovery.md`

- Declared feature atoms: **189** (E=166 I=2 A=18 C=3).
- Parsed feature rows in this aggregate: **189**.
- Per-file: 1A **047**; 1D **027**; 3D **026**; 3G **027**; 4D-ORACLE **019**; 4G **043**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-002.discovery.md`.

### Feature atoms

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

### Cross-source interactions (PHASE-002)

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| PHASE-002/IX-001 | E | Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE. | P1A-E008, P1A-E012, P1A-E027; P1D-E006, P1D-E022 |
| PHASE-002/IX-002 | E | 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4). | P1A-E009, P1A-E010, P1A-E043; P1D-E001 |
| PHASE-002/IX-003 | E | 1D density-guard fallback is insertion-order, not BM25, because BM25 rerank is upstream in the query pipeline. | P1D-E011 |
| PHASE-002/IX-004 | E | 3D requires 3B SSB + 3C agents; upgrades 3C spawn_parallel stub to true JoinSet parallelism; 3E/3F consume ChannelRegistry. | P3D-E001, P3D-E019, P3D-E013 |
| PHASE-002/IX-005 | E | 3D checkpoint+SSB replay composes with 3C agent checkpoints under `.elai_cc/sessions/{sid}/`; UntrackedValue deliberately absent from both. | P3D-E015, P3D-E018, P3D-E011 |
| PHASE-002/IX-006 | E | 3G.4 requires 3E Slot 3 ContextBudget; skeleton always loads in PhaseSpec slot; overflow uses 3E ReducedGranularity. | P3G-E001, P3G-E015 |
| PHASE-002/IX-007 | E | 3G coherence is invoked by 3C auditor via IPC; findings use action_type finding; plan mutations use plan_update/stub_reference. | P3G-E010, P3G-E011, P3G-E016 |
| PHASE-002/IX-008 | E | 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing. | P3G-E014; P4G-E009 |
| PHASE-002/IX-009 | E | 4G SPLIT_TASK is the Terminal path consumer of 3E context-budget exhaustion; sequential merge avoids 3H/S10 pick-one branches. | P4G-E009, P4G-E010 |
| PHASE-002/IX-010 | E | 4G SUBSTITUTE_MODEL is the autonomous completion of 3C on_max_restarts policy (interactive escalates). | P4G-E011 |
| PHASE-002/IX-011 | E | 4G wraps 3A InteractiveGate without new FSM states; evidence gates from orchestrator S13; risk levels from arch-overview S6. | P4G-E001, P4G-E002, P4G-E005 |
| PHASE-002/IX-012 | E | 4G tree auto-advance integrates 4E TreeNavigator + BubbleUpHandler; concept-level always human. | P4G-E019, P4G-E020 |
| PHASE-002/IX-013 | E | 4G.9 Spinning subsumes prior 3× GRC anti-spiral; personas spawn via 4A Tier-C isolation. | P4G-E021, P4G-E023, P4G-E029 |
| PHASE-002/IX-014 | E | 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss. | P4DOR-E001, P4DOR-E002, P4DOR-E013 |
| PHASE-002/IX-015 | E | Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys. | P1A-E013, P1A-E035; P4DOR-E014; P4G-E021 |
| PHASE-002/IX-016 | E | SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*. | P3D-E017, P3D-E023; P3G-E007, P3G-E011; P4DOR-E012; P4G-E014, P4G-E022, P4G-E032 |
| PHASE-002/IX-017 | I | 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks. | P1A-E039; P1D-E009; P4G-E005 |
| PHASE-002/IX-018 | E | 4G circuit breakers operate at two grains: phase auto-advances (default 10) and tree baby steps per concept (default 50). | P4G-E007, P4G-E019 |
| PHASE-002/IX-019 | E | Goal-drift Block and watchdog timeouts both hand off to 4E BubbleUpHandler rather than inventing a third recovery owner. | P4G-E026, P4G-E027 |
| PHASE-002/IX-020 | E | 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step. | P3D-E005; P4G-E026 |

### Ambiguity and conflict register (PHASE-002)

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| PHASE-002/AC-001 | conflict | R20 ID collision: PageRank density vs phase-0 entropy quarantine; renumber out of scope for 1D. | P1D-E003 |
| PHASE-002/AC-002 | conflict | Anti-spiral 3× vs same_step_count>3; 4G pins 3rd-failure interpretation. | P4G-C001 |
| PHASE-002/AC-003 | qualification | HIGH-risk human pause vs auditor tool review are different layers; both remain binding. | P4G-C002 |
| PHASE-002/AC-004 | ambiguity | 1A partial parse on syntax errors lacks completeness floor. | P1A-A003 |
| PHASE-002/AC-005 | ambiguity | 1D “mentioned in task” personalization matching algorithm unspecified. | P1D-A002 |
| PHASE-002/AC-006 | ambiguity | 3D evidence_gate_passes sufficiency predicate incomplete in phase file. | P3D-A003 |
| PHASE-002/AC-007 | dependency | 3D ChannelRegistry + memory_monitoring + Topic(false) + checkpoint placement need orchestrator.txt amendments before/with implement. | P3D-E007, P3D-A002 |
| PHASE-002/AC-008 | ambiguity | 3G plan_phases SQL shorthand vs SSB-only interim; view amendment needs user confirmation. | P3G-A001 |
| PHASE-002/AC-009 | ambiguity | 3G coherence “text reference” vs structured implemented_contracts dual wording. | P3G-A003 |
| PHASE-002/AC-010 | ambiguity | 4DOR query normalization for blake3 key undefined. | P4DOR-A001 |
| PHASE-002/AC-011 | ambiguity | 4G SessionState ownership and definition still open; execution_mode attachment site TBD. | P4G-A001 |
| PHASE-002/AC-012 | dependency | 4G requires new APIs: can_fit_llm, AgentPool::get_role, SSB escalation type — all require post-approval spec propagation. | P4G-E011, P4G-E013, P4G-E014 |
| PHASE-002/AC-013 | ambiguity | parallel_dispatch default/TOML key for persona fan-out unspecified. | P4G-A003 |
| PHASE-002/AC-014 | evidence | 4G runtime UX flags appear in MASTER summary more explicitly than in phase-4G body; do not over-claim source atoms for --local/--hybrid without addendum. | P4G-I001 |
| PHASE-002/AC-015 | qualification | blake2 (1A file hash) vs blake3 (4DOR/4G) is intentional layer split, not a single-stack rule — document when consolidating hash policy. | IX-015 |

### Summary deltas vs MASTER (PHASE-002)

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-1A-001 | **ADD** full COMPLIANCE SCOPE split of knowledge.txt Sec 8 items 1+5 vs deferred 2–4 and orphan-invariant block until 1B. | MASTER:L44-L55 | P1A-E009, P1A-E010, P1A-E043 |
| PHASE-002/SD-1A-002 | **ADD** readiness split dkg_readiness vs full KnowledgeEngine.readiness_check (Phase 1I). | MASTER:L48-L51 | P1A-E016, P1A-E017 |
| PHASE-002/SD-1A-003 | **ADD** complete schema tables dkg_files/dkg_meta, directed edge semantics, WIRING_CONTRACT symbols, index build SSB symbol_graph_updated, and 12 metric keys. | MASTER:L50-L55 | P1A-E022–P1A-E027, P1A-E038–P1A-E041 |
| PHASE-002/SD-1A-004 | **ADD** TDD red gate (≥20 tests, ≥10 fail), object-safety LanguageGrammar, and dynamic KNOW_PATH resolution. | MASTER:L48-L53 | P1A-E002, P1A-E011, P1A-E034 |
| PHASE-002/SD-1A-005 | **FLAG** ungrounded <15s build claim and designed-not-spec SQLite schema. | MASTER:L44-L55 | P1A-E006, P1A-E007 |

### 1D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-1D-001 | **ADD** full T1–T18/P1–P3/A1–A3 catalog and make_snapshot helper requirements. | MASTER:L93-L93 | P1D-E010–P1D-E014 |
| PHASE-002/SD-1D-002 | **ADD** exact 13-item special-file list and config-driven deferral; qualify MASTER “such as” wording. | MASTER:L96-L96 | P1D-E016, P1D-A001 |
| PHASE-002/SD-1D-003 | **ADD** R20 ID collision note and Layer 3.5 vs Layer 4 correction. | MASTER:L85-L97 | P1D-E003, P1D-C001 |
| PHASE-002/SD-1D-004 | **ADD** aider not-adopted items (specificity/private/generic penalties) and 7-metric phase_1D.toml fields. | MASTER:L88-L97 | P1D-E019, P1D-E020 |
| PHASE-002/SD-1D-005 | **ADD** zero-budget T13, NaN/Inf clamp, and initial binary-search guess max_tokens/25. | MASTER:L94-L96 | P1D-E015, P1D-E017 |

### 3D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-3D-001 | **ADD** four pre-implement orchestrator.txt amendments and corrected loop rationale (checkpoint after gate). | MASTER:L284-L288 | P3D-E007, P3D-E014 |
| PHASE-002/SD-3D-002 | **ADD** ephemeral clear SSB replay algorithm and Topic(false) two-phase snapshot+clear definition (spec gap). | MASTER:L281-L282, L287 | P3D-E009, P3D-E010, P3D-E018 |
| PHASE-002/SD-3D-003 | **ADD** DeepSizeOf/size_of_val ban, 10MB/2× force-clear, and isolation_metric separation from memory_monitoring. | MASTER:L283-L287 | P3D-E004, P3D-E017, P3D-E022 |
| PHASE-002/SD-3D-004 | **ADD** wall-time concurrency proof, VRAM max_concurrent formula, and simulation-only byte-identical determinism caveat for live LLMs. | MASTER:L277-L288 | P3D-E003, P3D-E019 |
| PHASE-002/SD-3D-005 | **ADD** baseline measurement section as non-exit UNGROUNDED estimates. | MASTER:L276-L288 | P3D-E020 |

### 3G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-3G-001 | **ADD** 3E dependency correction for 3G.4 and partial-parallel recommendation. | MASTER:L321-L332 | P3G-E001, P3G-E002 |
| PHASE-002/SD-3G-002 | **ADD** visual_check/output_type media adapter rules and ClockProvider timestamp ungrounded fields. | MASTER:L325-L325 | P3G-E004, P3G-E005 |
| PHASE-002/SD-3G-003 | **ADD** 3600 safety margin, SkeletonBudgetExceeded, ContractConflict, and empty declared_interfaces fail-closed. | MASTER:L327-L330 | P3G-E008, P3G-E017, P3G-E023 |
| PHASE-002/SD-3G-004 | **ADD** split_phase preserve_contracts algorithm details and u32 checked_add overflow Err. | MASTER:L331-L331 | P3G-E014, P3G-E018 |
| PHASE-002/SD-3G-005 | **ADD** 100-run determinism with EntropyProvider pipeline_id and ban wall clocks in plan_skeleton. | MASTER:L332-L332 | P3G-E019 |

### 4D-ORACLE deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-4DOR-001 | **ADD** entire Oracle cache feature set: MASTER lists phase-4D-ORACLE only as unindexed file to inspect, without extracting atoms. | MASTER:L655-L655, L714-L714 | P4DOR-E001–P4DOR-E017 |
| PHASE-002/SD-4DOR-002 | **ADD** TTL/confidence mapping, LRU default 128, /research+/ask reachability, blake3 ban, C-30 SCHEMA 62 events. | MASTER:L655 | P4DOR-E006–P4DOR-E015 |
| PHASE-002/SD-4DOR-003 | **ADD** LiveCli lazy Option init and T4Rejected early-return stub_ban. | MASTER:L655 | P4DOR-E011, P4DOR-E014 |
| PHASE-002/SD-4DOR-004 | **FLAG** query normalization and expired-entry lazy eviction ambiguities absent from any summary. | MASTER:L655 | P4DOR-A001, P4DOR-A002 |

### 4G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-002/SD-4G-001 | **ADD** full EvidenceCheck/EvidenceVerdict types and TESTING_FIRST cargo heuristic caveats. | MASTER:L471-L471 | P4G-E005, P4G-E006, P4G-A002 |
| PHASE-002/SD-4G-002 | **ADD** 28+8 test catalog, InterventionCard exhaustion UI, and mode-switch TOCTOU RwLock rules. | MASTER:L476-L476 | P4G-E015, P4G-E028, P4G-E035 |
| PHASE-002/SD-4G-003 | **ADD** complete 4G.9 stagnation/persona/watchdog/goal-drift port (thresholds, SSB types, affinity, fingerprint, Jaccard bands). | MASTER:L477-L477 (compressed tree only) | P4G-E021–P4G-E027, P4G-E034 |
| PHASE-002/SD-4G-004 | **ADD** shipped addendum: reasoning-exhaustion 7th auto-advance check and CloudSpendTracker budget policy (default $5). | MASTER:L467-L478 | P4G-E031, P4G-E032 |
| PHASE-002/SD-4G-005 | **ADD** required new APIs can_fit_llm, get_role, SSB escalation type and SessionState open. | MASTER:L475-L475 | P4G-E011, P4G-E013, P4G-E014, P4G-A001 |
| PHASE-002/SD-4G-006 | **QUALIFY** MASTER runtime UX (--local/--hybrid, /model, backend config): more explicit in summary than phase body; keep as orientation/I. | MASTER:L478-L478 | P4G-I001, AC-014 |
| PHASE-002/SD-4G-007 | **ADD** anti-spiral pin L180 and HIGH-risk dual-path clarification vs auditor tool review. | MASTER:L472-L472 | P4G-C001, P4G-C002 |

## PHASE-003 — source `DEEP-PHASE-003.discovery.md`

- Declared feature atoms: **151** (E=127 I=2 A=18 C=4).
- Parsed feature rows in this aggregate: **151**.
- Per-file: 1B **025**; 1G **025**; 3H **025**; 4D-SURGICAL **017**; 4E **032**; index-3A-4J **027**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-003.discovery.md`.

### Feature atoms

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

### Cross-source interactions (PHASE-003)

| IX ID | Interaction | Participating atoms |
|---|---|---|
| PHASE-003/IX-001 | 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish. | P1B-E007, P1B-E018, P1G-E006 |
| PHASE-003/IX-002 | 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators. | P1G-E012, P1G-E014, P1B-E018 |
| PHASE-003/IX-003 | 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget. | P3H-E010, P3H-A006, PIDX-E006 |
| PHASE-003/IX-004 | 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry. | P3H-E013-E014, PIDX-E003, PIDX-C001 |
| PHASE-003/IX-005 | 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots. | P3H-E009, PIDX-E006, PIDX-E008 |
| PHASE-003/IX-006 | 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth. | P4E-E010, PIDX-E008, PIDX-E020 |
| PHASE-003/IX-007 | 4E novel_step_ids consumed by 4F; visual_check adapters by 4I. | P4E-E027, PIDX-E019, PIDX-E022 |
| PHASE-003/IX-008 | 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks. | P4E-E018-E019, PIDX-E002, PIDX-E016 |
| PHASE-003/IX-009 | 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1. | P4E-E014, P4E-E020, PIDX-E017 |
| PHASE-003/IX-010 | 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds. | P4DS-E009, P4DS-E015, PIDX-E006 |
| PHASE-003/IX-011 | 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension. | P4DS-E006-E007, P1B-E001 |
| PHASE-003/IX-012 | Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path. | PIDX-E011, P1B-E010, P1B-E014 |
| PHASE-003/IX-013 | Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path). | PIDX-E012, P3H-E015 |
| PHASE-003/IX-014 | Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder. | P4DS-E002-E003, P4E-E001, PIDX-E017 |
| PHASE-003/IX-015 | RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition. | P4E-E013, PIDX-E002-E003 |
| PHASE-003/IX-016 | Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families. | P1G-E012, P4DS-E010, P4E-E021 |
| PHASE-003/IX-017 | User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G). | P1B-E019, P4DS-E002, P4DS-E011, PIDX-E013, PIDX-E024 |
| PHASE-003/IX-018 | 4E integration time ladder matches 4B IntegrationTestRunner levels in index. | P4E-E022, PIDX-E015 |
| PHASE-003/IX-019 | 4H MSI tracks SURGICAL/GREENFIELD modes; surgical CLI path is a related but separate C-30 feature. | PIDX-E021, P4DS-E001 |
| PHASE-003/IX-020 | Handoff 85% (3H) aligns with 4C PreCompact checkpoint-at-85% theme in index. | P3H-E002, PIDX-E016 |

### Ambiguity and conflict register (PHASE-003)

| AC ID | Kind | Issue | Atoms |
|---|---|---|---|
| PHASE-003/AC-001 | conflict/schema | tantivy_v21 label vs INDEX_FORMAT_VERSION u32. | P1B-C001, P1B-A001 |
| PHASE-003/AC-002 | ambiguity/config | Exact tantivy/ort pin versions deferred to execution decision docs. | P1B-A001, P1G-A003 |
| PHASE-003/AC-003 | conflict/fallback | L-6 reranker tier needs amendment vs skip-only spec. | P1G-C001, P1G-E011 |
| PHASE-003/AC-004 | ambiguity/measure | 1G skip threshold and 50 ms pass/fail after levers. | P1G-A001, P1G-E018 |
| PHASE-003/AC-005 | conflict/schema | 3H per-message snapshot_context vs orchestrator blob schema. | P3H-C001, P3H-A001 |
| PHASE-003/AC-006 | ambiguity/event | branch_abandon new type vs checkpoint payload. | P3H-A002, PIDX-C001 |
| PHASE-003/AC-007 | ambiguity/data | SSB merge cleanup logical vs physical wording. | P3H-A003 |
| PHASE-003/AC-008 | ambiguity/merge | File-overlap Conflict variant ungrounded. | P3H-A004 |
| PHASE-003/AC-009 | ambiguity/crypto | blake3 stub hash algorithm choice. | P3H-A005 |
| PHASE-003/AC-010 | ambiguity/surface | SurgicalDrawer full UI fields; step-cap merge notes. | P4DS-A001, P4DS-A002 |
| PHASE-003/AC-011 | ambiguity/enum | CycleDetected vs CyclicDependency naming. | P4E-A001 |
| PHASE-003/AC-012 | ambiguity/repair | Weakest-dependency definition for REORDER. | P4E-A003 |
| PHASE-003/AC-013 | ambiguity/order | Index 3K listed before 3J vs HyDE dependency on 3J. | PIDX-A001 |
| PHASE-003/AC-014 | ambiguity/index | Index omits C-30 surgical CLI/drawer present in 4D-SURGICAL. | PIDX-I001, P4DS-E002 |
| PHASE-003/AC-015 | ambiguity/dep | 3E total_allocated_tokens exposure for TrimConfig. | P3H-A006 |

### Summary deltas vs MASTER (PHASE-003)

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`.

### phase-1B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-1B-001 | **ADD** full API surface IndexStats/BM25Hit snippet policy, single-writer batch lock, and explicit project_id rule. | MASTER:L57-L67 | P1B-E006-E013 |
| PHASE-003/SD-1B-002 | **ADD** WIRING_CONTRACT CLI `index build` + SSB `index_checkpoint` + call sites. | MASTER:L57-L67 | P1B-E018-E019 |
| PHASE-003/SD-1B-003 | **ADD** environment-aware CI 50 ms vs dev 10 ms gates and warm-up measurement. | MASTER:L67 | P1B-E017 |
| PHASE-003/SD-1B-004 | **ADD/CORRECT** tantivy_v21 vs INDEX_FORMAT_VERSION and DkgNode-vs-BM25Chunk ownership. | MASTER:L61-L63 | P1B-E004-E006, P1B-C001 |
| PHASE-003/SD-1B-005 | **ADD** 1I scoping-out of readiness/breaker/versioning from 1B. | MASTER:L67 | P1B-E020 |

### phase-1G deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-1G-001 | **ADD** full RerankerConfig field set, mpsc batching design, and oneshot offset split. | MASTER:L123-L133 | P1G-E004-E008 |
| PHASE-003/SD-1G-002 | **ADD** L-6 amendment path, cold-start SLA exception, and canary DEGRADED semantics. | MASTER:L129-L132 | P1G-E010-E014, P1G-C001 |
| PHASE-003/SD-1G-003 | **ADD** INT8 trigger p50>30 ms / NDCG delta <0.5 and quantization script path. | MASTER:L127, L133 | P1G-E015 |
| PHASE-003/SD-1G-004 | **ADD** complete metrics list including queue wait p99 and amendment-on-fail-50ms rule. | MASTER:L133 | P1G-E017-E018 |
| PHASE-003/SD-1G-005 | **ADD** 1F DROP independence note (MASTER has 1F section; 1G body reconfirms). | MASTER:L114-L121 | P1G-E001 |

### phase-3H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-3H-001 | **ADD** full SQL column/index set, types, and DELETE-journal/FK/integrity_check safety. | MASTER:L334-L344 | P3H-E003-E008 |
| PHASE-003/SD-3H-002 | **EXPAND** three amendments (schema/event/cleanup) and Conflict/blake3 open items beyond MASTER [Q] one-liner. | MASTER:L337 | P3H-A001-A005, P3H-C001 |
| PHASE-003/SD-3H-003 | **ADD** recovery APIs, typed StubRecoveryFailed, and 100-run determinism/measure targets. | MASTER:L343-L344 | P3H-E014-E017 |
| PHASE-003/SD-3H-004 | **ADD** branch naming rules, close_branch, and immutable trunk-extension merge semantics detail. | MASTER:L342 | P3H-E012 |

### phase-4D-SURGICAL deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-4DS-001 | **ADD** entire C-30 surgical path: MASTER only flags file as unindexed; full CLI/SSB/drawer/LOC/complexity atoms are deep-only. | MASTER:L655 | P4DS-E001-E015 |
| PHASE-003/SD-4DS-002 | **ADD** intercept-before-research runner semantics and DkgQuerierAdapter stub ban. | MASTER:L655 | P4DS-E003, P4DS-E007, P4DS-E013 |

### phase-4E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-4E-001 | **ADD** full type inventory, nine builder stubs, and 8 ValidationErrorKind list. | MASTER:L442-L455 | P4E-E003-E008 |
| PHASE-003/SD-4E-002 | **ADD** Phase Sizing formula constants, fixpoint hash escalate, REORDER-before-SUBSTITUTE, RuntimeClient wiring rule. | MASTER:L445-L449 | P4E-E007, P4E-E009-E010, P4E-E013 |
| PHASE-003/SD-4E-003 | **ADD** complete baby-step 8-stage cycle, BubbleUpPolicy defaults, SSB replan_request, msgpack recovery. | MASTER:L451-L455 | P4E-E014-E023 |
| PHASE-003/SD-4E-004 | **ADD** test catalog counts (≥30+10+33) and 4E.12 context p95/overhead gates. | MASTER:L450-L455 | P4E-E011, P4E-E024-E025 |
| PHASE-003/SD-4E-005 | **FLAG** enum name CycleDetected vs CyclicDependency and weakest-dep open. | MASTER:L446-L448 | P4E-A001, P4E-A003 |

### phase-index-3A-4J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-003/SD-IDX-001 | **ADD** this file is a second index surface parallel to MASTER; captures 3J wire substeps, `/model`, backend modes, 4H MSI, Empire caps not always expanded in MASTER bullets. | MASTER:L334-L478 region | PIDX-E012-E013, PIDX-E021-E024 |
| PHASE-003/SD-IDX-002 | **QUALIFY** index is not authoritative over deep phase files when they conflict (e.g. surgical absent here). | MASTER:L1-L8 | PIDX-I001, PIDX-A001, AC-014 |
| PHASE-003/SD-IDX-003 | **ADD** 3K FAST/STANDARD/DEEP tier routing and 4B IntegrationTestRunner ladder as index-compressed features. | MASTER:L399, retrieval ladder | PIDX-E011, PIDX-E015 |

## PHASE-004 — source `DEEP-PHASE-004.discovery.md`

- Declared feature atoms: **157** (E=135 I=6 A=11 C=5).
- Parsed feature rows in this aggregate: **157**.
- Per-file: 1E **034**; 2B **027**; 2C **024**; 3L **024**; 4C **021**; 4I **027**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-004.discovery.md`.

### Feature atoms

### 1E — CatRAG + A-RAG (query pipeline assembly)

Source: `docs/specs/guide/steps/phase-1E.txt` (443/443 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1E-E001 | E/STATUS | Phase 1E assembles full retrieval: query → BM25 → vector → PageRank → CatRAG → A-RAG. Prerequisites: 1D complete (PageRank + DKG); depends on 1B BM25 and 1C vectors. | 1E:L1-L9 |
| P1E-E002 | E/CAP | CatRAG is query-aware graph traversal over DKG (evidence-chain expansion). A-RAG is hierarchical retrieval with snippet progressive disclosure. Target 55–87% token savings vs full-chunk; sub-second latency goal; interactive budget ≤2s total. | 1E:L7-L11, L61-L62, L438-L442 |
| P1E-E003 | E/EVIDENCE | Grounding papers: CatRAG arXiv 2602.01965 (interface `workflow/papers/catrag-interface.md`); A-RAG arXiv 2602.03442 (interface `workflow/papers/arag-interface.md`); knowledge.txt Layers 4–5 + §3 query pipeline. | 1E:L13-L24 |
| P1E-E004 | E/DATA | CatRAG paper→ELAI map: entity→DkgNode; passage→file chunks; relation edges→calls/references/imports/implements; synonym edges→cosine>0.8; context edges→node-to-file; OpenIE/NER not used (tree-sitter + regex/fuzzy symbol match). | 1E:L26-L35 |
| P1E-E005 | E/CAP | A-RAG adaptation: keyword_search→BM25+snippets; semantic_search→dual-vector+snippets; chunk_read→disk file/line range; agent loop→ReAct with ollama/external LLM; one tool per step (no parallel tool calls). | 1E:L37-L43 |
| P1E-E006 | E/FLOW | Canonical stage order (knowledge.txt L104-105): BM25+SPLADE(RRF) → dual-vector rerank → PageRank filter → CatRAG expand → A-RAG filter → cross-encoder rerank → token-budget fit → context. Cross-encoder is Phase 1G; token-budget is Phase 3E; this phase stubs both. | 1E:L45-L49, L375-L377 |
| P1E-E007 | E/FAIL | R30: limit CatRAG LLM to N_seed=5, K_edge=15; cache scores. R31: if density <0.01 skip CatRAG and pass PageRank→A-RAG; add synonym edges >0.8. R32: A-RAG max_loops=15 + token budget + context tracker. R33: snippets ±2 code lines. R34: measure per-layer latency. | 1E:L51-L62 |
| P1E-E008 | E/CFG | CatRagConfig defaults: damping=0.5, epsilon=0.2, beta=2.5, n_seed=5, k_edge=15, min_graph_density=0.01. | 1E:L76-L83 |
| P1E-E009 | E/DATA | CatRagOverlay is sparse only: `reset_distribution: Vec<(usize,f64)>` and `edge_weight_deltas: Vec<(usize,usize,f64)>`; never full adjacency matrix copies. | 1E:L84-L87, L157-L159, L252 |
| P1E-E010 | E/CAP | CatRag API: `new(config)`, `compute_overlay(query, snapshot, pagerank_scores) -> Result<CatRagOverlay>`; modifies PPR inputs without mutating base graph. | 1E:L88-L97, L265-L269 |
| P1E-E011 | E/CFG | AragConfig defaults: max_loops=15, max_token_budget=128000, top_k=5. | 1E:L100-L104 |
| P1E-E012 | E/CAP | RetrievalTool enum: KeywordSearch{keywords,top_k}, SemanticSearch{query,top_k}, ChunkRead{chunk_ids}. RetrievalResult: chunk_id, snippet, score, tool_used. | 1E:L105-L115 |
| P1E-E013 | E/STATE | ContextTracker holds per-query HashSet of read_chunks; created fresh each query_pipeline call; not shared/concurrent. | 1E:L119-L121, L160-L161, L310-L313, L333-L334 |
| P1E-E014 | E/CAP | AragOrchestrator: `new`, `execute_tool`, `query_pipeline` (full BM25→vectors→PageRank→CatRAG→A-RAG wiring). | 1E:L116-L134 |
| P1E-E015 | E/CAP | QueryPipeline holds bm25 + catrag + arag (+ vector/pagerank from prior phases); `query` returns `Vec<PipelineStage{stage,results,latency_ms}>`. Modules: catrag, arag, pipeline wired in knowledge lib.rs. | 1E:L135-L150 |
| P1E-E016 | E/TEST | 1E.1 PRE_GATE: 1D tests ok; DkgSnapshot, Bm25Index, pagerank symbols present. DETERMINISTIC: cargo check + three pub mod greps. | 1E:L68-L72, L152-L156 |
| P1E-E017 | E/TEST | 1E.3 CatRAG tests: symbolic anchoring weight formula epsilon/|files|; dynamic edge weights only from seeds; key-fact boost *(1+beta); density guard empty overlay; phi tiers 0-3→0, 4→0.2, 5→0.25, 6→0.3, 7→2.0, 8→2.5, 9→3.0, 10→5.0. | 1E:L184-L194 |
| P1E-E018 | E/TEST | 1E.3 A-RAG tests: keyword score sum(count*\|keyword\|) snippets only; semantic oversample 3× aggregate max score; chunk_read second hit returns "(previously read)"; tracker monotonic; token budget stops with partial. | 1E:L196-L205 |
| P1E-E019 | E/TEST | 1E.3 pipeline tests: stages order bm25/vector/pagerank/catrag/arag; latency_ms>0; empty query no panic. Contracts: overlay apply/revert; RetrievalResult fields non-empty. All new tests fail on stubs. | 1E:L207-L223 |
| P1E-E020 | E/FLOW | Symbolic anchoring: split whitespace/camelCase/snake_case; case-insensitive substring fuzzy match; reset_weight=epsilon*\|files_containing\|^(-1); combine with PageRank seeds; normalize sum=1; no LLM. Cap matches at 2*N_seed; warn if >50% DKG matched. | 1E:L232-L238, L279-L280 |
| P1E-E021 | E/FLOW | Dynamic edge weighting: top N_seed by reset; prune neighbors by embedding similarity if >K_edge; heuristic scores by edge type calls=8 implements=7 references=5 imports=3 + target relevance + file proximity; map via phi; LLM scoring optional via circuit breaker only. | 1E:L240-L252 |
| P1E-E022 | E/FLOW | Key-fact enhancement: T_seed = query symbols + direct callees; boost symbol→file context edges by *(1+beta). Pure algorithmic. | 1E:L254-L258 |
| P1E-E023 | E/FAIL | Density = \|edges\|/(\|nodes\|*(\|nodes\|-1)); if < min_graph_density return empty overlay; log "CatRAG skipped: graph density {d} below threshold {t}". | 1E:L260-L263, L367 |
| P1E-E024 | E/CAP | Keyword tool: merge BM25 per keyword; snippet sentences with keyword ±2 lines; top_k snippets not full chunks. Semantic: vector query, 3× oversample, max-score per file. Chunk read: full content or previously-read placeholder. | 1E:L290-L308 |
| P1E-E025 | E/FLOW | Phase 1E A-RAG orchestration is deterministic fixed pipeline keyword→semantic→chunk_read; full ReAct LLM tool choice deferred to Phase 2+ reasoning crate. | 1E:L320-L324 |
| P1E-E026 | E/FLOW | Pipeline stages: (1) BM25 top_k=20 after stopword filter; (2) vector + RRF k=60 with BM25, skip if breaker; (3) PageRank importance filter; (4) CatRAG overlay re-PPR expand; (5) A-RAG snippet filter; (6–7) stubs stage "rerank" and "token_fit". | 1E:L343-L379 |
| P1E-E027 | E/FAIL | Stage failures wrap Result, log stage name, fall through. RRF ranks start at 1 (k=60) to avoid NaN. Optional stages degrade gracefully. | 1E:L385-L395 |
| P1E-E028 | E/OBS | measure_1e.rs indexes .elai_cc crates; 5 queries Q1–Q5; metrics phase_1E.toml: pipeline_p50_ms, pipeline_p99_ms, token_savings_mean_pct, catrag_overlay_size, arag_tools_per_query, stages_skipped. Targets p50<2000ms, savings>50%. | 1E:L397-L435 |
| P1E-E029 | E/TEST | Exit: full pipeline operational; savings>50%; latency<2s; graceful optional-layer skip; cargo test -p knowledge --lib 21+ green. | 1E:L438-L443 |
| P1E-E030 | E/DEP | Modules under `.elai_cc/crates/knowledge/src/{catrag,arag,pipeline}.rs`; PRE_GATE greps DkgSnapshot, Bm25Index, pagerank. | 1E:L69-L72, L74-L75, L98, L135 |
| P1E-I001 | I/DEFER | "Phase 2+" for ReAct A-RAG means reasoning-crate agent loop ownership; tools themselves still ship in 1E. | 1E:L320-L324 |
| P1E-A001 | A/CFG | SPLADE appears in knowledge.txt order but 1E pipeline stage list does not implement a dedicated splade stage (1H owns SPLADE). | 1E:L45-L49, L207-L209 |
| P1E-A002 | A/DET | "Heuristic scoring" formula for combining edge type + relevance + proximity into 0–10 before phi is not fully specified. | 1E:L245-L248 |
| P1E-C001 | C/OBS | Goal line says "sub-second" while R34 and exit criteria bind interactive budget to 2 seconds; treat 2s p50 as the hard acceptance, sub-second as aspirational. | 1E:L11, L61-L62, L411, L433, L441 |

Category coverage: capabilities/pipeline ✓; workflows/stages ✓; schemas/config ✓; flags/defaults ✓; integrations/BM25-vector-PageRank ✓; security/privacy none named; failures/degradation ✓; determinism (no hot-path LLM anchoring) ✓; observability/metrics ✓; tests/TDD ✓; constraints/deferred ReAct+1G+3E ✓. No formal CLI surface beyond measure binary.

### 2B — grounding verification

Source: `docs/specs/guide/steps/phase-2B.txt` (580/580 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2B-E001 | E/STATUS | Phase 2B implements verification methods that resolve claims against real sources. Prerequisite: Phase 2A (9+ GRC tests pass). | 2B:L1-L2 |
| P2B-E002 | E/CAP | Six methods: FILE_CHECK, SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK, API_REGISTRY_CHECK. Each returns binary pass/fail + evidence string (no confidence scores). | 2B:L6-L14, L44-L54 |
| P2B-E003 | E/FLOW | Plan grounding sweep before EXECUTING: classify claims GROUNDED / VERIFY_NEEDED / UNGROUNDED; verify VERIFY_NEEDED; block if any UNGROUNDED remain. Blocks PLANNING→EXECUTING. | 2B:L9-L18, L537-L543 |
| P2B-E004 | E/RECOVERY | Uncertain (2A) claims auto-promoted via FILE_CHECK/SOURCE_CHECK to Verified/High when resolvable; unresolved stay Uncertain and are flagged (rejection policy is 2C per-agent). | 2B:L20-L23, L463-L470 |
| P2B-E005 | E/DEFER | Phase 4A wraps verification behind IPC GroundingRequest; 2B is direct calls structured for later wrap without trait signature change (`&self`, stateless). | 2B:L25-L27, L141-L143 |
| P2B-E006 | E/DEP | Mandatory reads: reasoning.txt §§4–5,8–11; architecture-overview §4 stubs; phase-2A.1 stubs; tools.txt for file ops. | 2B:L30-L39 |
| P2B-E007 | E/DATA | VerificationResult {claim, passes, evidence, method_used: VerificationMethod}. GroundingResult {claim, method: VerificationMethod, level, source, verified_at:u64 TimeSource}. | 2B:L44-L45, L75-L80, L167-L175 |
| P2B-E008 | E/DATA | PlanGroundingResult {claims, all_grounded, ungrounded, verify_needed}. verify_needed = training: claims that could not auto-resolve; subset of ungrounded; all_grounded = ungrounded.is_empty(). | 2B:L96-L104, L176-L186 |
| P2B-E009 | E/CAP | VerificationMethod enum + requires_network() (Web/Api/ApiRegistry=true) + requires_tools() (File/Source/Test=true). | 2B:L67-L73, L157-L161, L339-L341 |
| P2B-E010 | E/CAP | Verifier trait object-safe Send+Sync: verify(method,claim,context)->Result; supported_methods(). TestRunner: run_test(name,crate)->TestRunResult{passed,output,duration_ms}. | 2B:L119-L129, L189-L202 |
| P2B-E011 | E/CAP | CargoTestRunner {sandbox BashSandbox, timeout_ms} production; MockTestRunner {results HashMap} for tests. LocalVerifier::new(Box<dyn TestRunner>). NetworkVerifier stubs. | 2B:L124-L128, L200-L212 |
| P2B-E012 | E/INT | Breaking trait changes from 2A: verify_claim gains context param; ground_plan returns PlanGroundingResult not Vec. Stubs were unimplemented!() so safe. | 2B:L96-L116, L203-L208 |
| P2B-E013 | E/DEP | reasoning/ gains tools/ dependency for FILE_CHECK/SOURCE_CHECK. Must not create tools→reasoning cycle; transitive knowledge via tools must not appear in cargo tree. Allowed deps: core, tools, thiserror, serde. | 2B:L82-L94, L132-L135, L209, L257-L276 |
| P2B-E014 | E/FAIL | GrcError::VerificationError {method, claim, reason}. Network methods return typed VerificationError not panic. | 2B:L187-L188, L423-L430, L483-L485 |
| P2B-E015 | E/TEST | ≥16 verification_tests: existing/missing file; source exist/missing; mock test; web/api/registry stubs; plan block/pass/empty; uncertain auto-resolve; requires_network; dyn Verifier; adversarial regex comment vs real claim; strip file: prefix. | 2B:L286-L388 |
| P2B-E016 | E/LIMIT | Unit tests ban network (no reqwest/hyper/TcpStream) and ban sleep/Duration/Instant timing. Network gated #[cfg(feature="network")] integration only. | 2B:L136-L138, L372-L375 |
| P2B-E017 | E/CAP | FILE_CHECK: tools::read_file after stripping "file:"/"grep:" prefix; evidence with path+lines or not found. SOURCE_CHECK: read_file + regex `(fn\|struct\|enum\|trait\|impl\|const\|static\|type\|mod)\s+{name}\b`; claim=symbol, context=path. | 2B:L398-L412, L436-L442 |
| P2B-E018 | E/CAP | TEST_CHECK: CargoTestRunner BashSandbox::execute `cargo test --exact {name} -p {crate}` timeout 30s; pass on exit 0; fail on nonzero/timeout. | 2B:L414-L421 |
| P2B-E019 | E/FLOW | verify_claim prefix map: file:/grep:→FileCheck; test:→TestCheck; search:→WebCheck; tool:→SourceCheck; training:→no verify passes=false Uncertain. Prefix always stripped before Verifier. | 2B:L432-L446 |
| P2B-E020 | E/FLOW | ground_plan: regex extract type:reference {file,grep,test,search,tool,training}; verify each; training tries FILE/SOURCE auto-resolve; empty plan trivially all_grounded=true. verified_at from TimeSource. | 2B:L448-L461, L139-L140 |
| P2B-E021 | E/OBS | Metrics `.elai_cc/metrics/phase_2B.toml`: file_check_latency_us <2000; source_check_latency_us <10000 (1000-line file); plan_sweep_latency_us <100000 (20 claims); methods_implemented=3/6. | 2B:L503-L534 |
| P2B-E022 | E/LIMIT | Zero todo!/unimplemented!/unwrap()/panic! in verification.rs; network stubs typed errors only. | 2B:L477-L500, L547 |
| P2B-E023 | E/EVIDENCE | Binary checks grounded MiniCheck EMNLP 2024 + Vertex AI; structural decomposition vs RARR/SAFE LLM; 6 methods reasoning.txt §4; tools dep crate map. | 2B:L50-L52, L61-L64, L551-L580 |
| P2B-I001 | I/SEC | Send+Sync on Verifier is inferred from 4A process isolation IPC, not explicit reasoning.txt bound. | 2B:L245-L249, L571-L574 |
| P2B-A001 | A/DATA | Exact regex for plan claim extraction (context vs free-text "file:") is required by T15 but pattern body not fully normative beyond type:reference set. | 2B:L349-L355, L452-L454 |
| P2B-A002 | A/FLOW | SOURCE_CHECK "looks like a path" vs "looks like a symbol" auto-resolution heuristic for Uncertain claims is not fully formalized. | 2B:L463-L469 |
| P2B-C001 | C/DEFER | Network stub message wording varies ("not implemented in phase 2B" vs summary "not impl until Phase 4A IPC"); behavioral requirement is typed error without network, not exact string. | 2B:L423-L430 |

Category coverage: capabilities/verification ✓; workflows/plan sweep ✓; schemas/types ✓; flags/feature network ✓; integrations/tools BashSandbox ✓; security integrity ✓ (no privacy policy); failures/typed errors ✓; determinism/TimeSource ✓; observability/metrics ✓; tests 16+ ✓; deferred network+IPC ✓.

### 2C — per-agent GRC configuration

Source: `docs/specs/guide/steps/phase-2C.txt` (496/496 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2C-E001 | E/STATUS | Phase 2C: per-role GRC budgets, step sequences, grounding constraints, allowed methods. Prerequisite: Phase 2B. | 2C:L1-L2 |
| P2C-E002 | E/CAP | Engine routes AgentGrcConfig at spawn. Layered config: [grc.defaults] + [agents.{role}.grc] field-by-field override (not deep merge). Underscore role keys (context_compiler) not hyphens. | 2C:L7-L12, L54-L66, L209-L210 |
| P2C-E003 | E/DATA | AgentGrcConfig 8 fields: role, budget, required_steps, grounding_level, csq_required, allowed_methods, transition_overrides, require_scope_enumeration. | 2C:L111-L125, L162-L164, L468-L469 |
| P2C-E004 | E/FLOW | D1 transition_overrides additive HashMap only (union with allowed_next, never remove). Auditor needs Verify→[Verify,Observe] for double-verify. | 2C:L28-L40, L321-L326, L448 |
| P2C-E005 | E/CFG | D2 validate at load: (a) required_steps non-empty; (b) valid chain via merged transitions; (c) max_calls ≥ len(required_steps); (d) csq_required ⇒ ≥1 Verify; (e) allowed_methods ⊆ known VerificationMethod. Err ConfigValidation. | 2C:L42-L52, L312-L319, L452 |
| P2C-E006 | E/SEC | allowed_methods is POLICY layer (what role may request) ≤ CAPABILITY layer (IPC proxy in 4A). Context-compiler policy FILE_CHECK+SOURCE_CHECK only; executor/auditor all methods. 2C enforces in-process; 4A adds second IPC layer. | 2C:L68-L84 |
| P2C-E007 | E/FAIL | Anti-rubber-stamp D5: auditor Converge requires ≥1 Verify step with FINDING: prefix (case-sensitive after trim); else InsufficientFindings {role, expected_min:1}. Const FINDING_PREFIX. | 2C:L86-L97, L350-L358, L450 |
| P2C-E008 | E/DATA | require_scope_enumeration: Observe ThinkStep.scope must list components; orchestrator checks plan_action coverage; unaddressed → WARN. Default true executor/designer, false memory_writer. | 2C:L120-L125, L279-L282, L455 |
| P2C-E009 | E/CAP | GrcError::InsufficientFindings, ConfigValidation. AgentGrcConfigLoader load_config/validate_config. GrcDefaults {budget, grounding_level, csq_required, allowed_methods}. Engine with_agent_config / get_agent_config. | 2C:L126-L136 |
| P2C-E010 | E/LIMIT | Role stored as String not enum; validate known roles at load. No toml crate in reasoning; receives typed structs from core/orchestrator. | 2C:L152-L156, L182-L188 |
| P2C-E011 | E/TEST | ≥15 agent_config tests T1–T15: auditor double-verify; context_compiler observe-only; executor 8/1200; memory_writer 2/400; empty steps; budget too small; rubber-stamp; Uncertain reject; allowed_methods filter; defaults+override; additive transitions; auditor 10/1200; prompt_engineer 5/800; designer distinct; scope enum per role. | 2C:L198-L302 |
| P2C-E012 | E/FLOW | effective_allowed_next = (base ∪ overrides) ∩ (required_steps ∪ {Status, Converge}). Whitelist makes observe-only actually block Hypothesize. | 2C:L321-L335 |
| P2C-E013 | E/STATE | GroundingLevel PartialOrd must be Uncertain < High < Verified (discriminant order or explicit impl). Reject think if level < config.grounding_level. Phase 2A order {Verified,High,Uncertain} is inverted and must be fixed. | 2C:L337-L348, L405-L406, L488-L491 |
| P2C-E014 | E/FLOW | with_agent_config stores config and resets topic state (no budget leak across roles). Missing role table → defaults only; malformed table → error not silent fallback. | 2C:L360-L372, L399-L404 |
| P2C-E015 | E/STATE | Budget authority: 2C in-process GrcEngine is AUTHORITATIVE through Phases 0–3; Phase 4A demotes agent-side to advisory and orchestrator enforces (orchestrator.txt L440-444). | 2C:L373-L380, L471 |
| P2C-E016 | E/OBS | Metrics phase_2C.toml: config_load_us <1000; routing_overhead_pct <10; memory per config <2KB including overrides. | 2C:L414-L442 |
| P2C-E017 | E/SURFACE | TOML surfaces: `[grc.defaults]`, `[agents.executor.grc]`, `[agents.auditor.grc]`, `[agents.context_compiler.grc]`, `[agents.memory_writer.grc]`, `[agents.prompt_engineer.grc]`, designer role table; pattern matches existing supervision/slo_tier/sandbox/role_prompt. | 2C:L54-L66, L23, L445-L447 |
| P2C-E018 | E/CAP | Six roles including Designer: same model as executor possible, distinct role_prompt/required_steps/output PipelineSpec. | 2C:L270-L277, L446-L447, L470 |
| P2C-E019 | E/LIMIT | Zero todo!/unimplemented!/unwrap() in engine.rs and config.rs after implement. | 2C:L387-L411, L457 |
| P2C-E020 | E/EVIDENCE | Spec gaps flagged for reconciliation: AgentGrcConfig needs 3 extra fields in reasoning.txt L186; GrcError missing variants; GroundingLevel order; invariant for rubber-stamp; allowed_methods vs grounding_level complementary docs. | 2C:L480-L496 |
| P2C-I001 | I/DEP | "Status" always-allowed alongside Converge implies Status is a GRC step type reachable even when not listed in required_steps (from 2A enum). | 2C:L327-L335 |
| P2C-A001 | A/DATA | Exact FINDING marker structured type deferred; prefix contract is interim internal protocol. | 2C:L294-L296, L394-L398 |
| P2C-A002 | A/CFG | Full authoritative TOML tables for all six roles are test-described but not listed as complete sample config block in this phase file. | 2C:L198-L277 |
| P2C-C001 | C/DATA | reasoning.txt still documents 4-field AgentGrcConfig while 2C requires 8; phase flags update, does not edit owned spec. | 2C:L82-L84, L468-L469, L481-L484 |

Category coverage: capabilities/config routing ✓; workflows/whitelist transitions ✓; schemas/TOML ✓; surfaces/role tables ✓; security policy methods ✓; failures/InsufficientFindings ✓; determinism PartialOrd ✓; observability/metrics ✓; tests 15+ ✓; deferred 4A dual-enforce ✓.

### 3L — Project Intelligence (conventions + Pi tier hooks + telescope)

Source: `docs/specs/guide/steps/phase-3L.txt` (251/251 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3L-E001 | E/STATUS | Phase 3L extracts project coding conventions; surfaces ConventionStats/NamingConventions; wires PiT1/PiT2/PiT3 hooks into prompt construction; integrates context telescope; latency p50≤50ms / p99≤100ms. | 3L:L1-L10 |
| P3L-E002 | E/DEP | Predecessor phase-3K is RAG production hardening (adaptive chunking, query expansion, HyDE, quality tiers), not a "Tier0 prompt scaffold". | 3L:L33-L36, L70-L71 |
| P3L-E003 | E/CAP | D1: hooks live in elai_knowledge::project_intelligence as PiT1Hook/PiT2Hook/PiT3Hook to avoid collision with elai_plugins::tiered_analysis T1/T2/T3. Reuse HookVerdict + TieredAnalysisResult (no fork). | 3L:L12-L17, L207-L208 |
| P3L-E004 | E/CFG | D2: regex via std::sync::LazyLock (Rust 1.80+), not once_cell::Lazy. Static-init .expect() allowed for literal patterns (CLAUDE.md S1). | 3L:L18-L20, L86-L89, L218-L219 |
| P3L-E005 | E/DATA | D3: conventions migration at knowledge/src/conventions_migration_v1.sql (flat sibling to metadata_migration_v2.sql, not migrations/ subdir). Separate conventions.db. | 3L:L21-L23, L94-L101 |
| P3L-E006 | E/DATA | D4: NamingConventions single surface from elai_core::project_profile; extract_conventions returns ConventionStats with naming field; no PiNamingConventions fork. | 3L:L24-L27, L79-L80 |
| P3L-E007 | E/OBS | D5: ProjectProfile snapshot at `.elai_cc/metrics/project_profile.toml` via persist_profile; reloaded at session start by `/profile` slash command (cli Profile variant). | 3L:L28-L30, L148-L151, L212 |
| P3L-E008 | E/CAP | D6: ProjectRuleSeeder API is seed_rules (not seed); PiT2Hook::run calls it. | 3L:L31-L32, L129-L131 |
| P3L-E009 | E/CAP | extract_conventions(&CorpusView)->ConventionStats; CorpusView holds Option<&[DkgNodeInfo]> + project_root + primary_language; DKG path if nodes non-empty else regex fallback. CONVENTIONS_SCHEMA_VERSION:u32=1. | 3L:L81-L91 |
| P3L-E010 | E/DET | extract_conventions byte-identical on repeat (sorted walker + determinism_matrix for Rust/Python/TypeScript/Svelte). Expected majority styles: snake_case, snake_case, camelCase, PascalCase. | 3L:L102-L113 |
| P3L-E011 | E/RECOVERY | PiT2 VERSION_GATE: if schema stale, execute_batch embedded migration (IF NOT EXISTS) and return SchemaTooOld for re-run. | 3L:L98-L101, L132-L135 |
| P3L-E012 | E/SURFACE | PiT1Hook::load + ProjectIntelligenceHook::render_block → CONVENTIONS block for SYSTEM via build_with_conventions. Format: per-category summaries + error_crate + top-3 avoid anti-patterns. Cap PI_T1_TOKEN_BUDGET=200 tokens (800 chars); truncate at line boundary. | 3L:L115-L126, L200-L201 |
| P3L-E013 | E/CAP | PiT2Hook::run seeds rules.db; evaluate Ok(None) on unrecognised language OR synthesised line count > PI_T2_CIRCUIT_BREAKER_LINES (mirrors plugins T2). | 3L:L128-L140, L221 |
| P3L-E014 | E/STATE | ProjectProfile aggregation after full reindex via populate_tier2_conventions + populate_tier3. PiT3Hook::evaluate ADVISORY ONLY — never HookVerdict::Block. | 3L:L142-L153, L220 |
| P3L-E015 | E/FLOW | assemble_telescope_with_hooks(profile, hooks)->(TelescopeContext, Vec<String>); priorities PI_T1=100 > PI_T2=80 > PI_T3=60; ties by hook id; TELESCOPE_MAX_CHARS=1200 drop whole blocks. Legacy assemble_telescope preserved. | 3L:L155-L167 |
| P3L-E016 | E/TEST | pi_telescope_contract: (a) top-K stable across 10 repeats; (b) T1>T2>T3; (c) per-hook budget; (d) PiT3 never Block; (e) PiT2 unrecognised language Ok(None). | 3L:L168-L174 |
| P3L-E017 | E/OBS | calibrate_phase_3L.py stdlib-only; 1000-file multi-lang corpus; emits phase_3L.toml; gates p50≤50ms p99≤100ms; floor 80% of gates for auditor. Fallback: reduce regex, shard by language, or in-process bench. | 3L:L176-L186 |
| P3L-E018 | E/INT | Reachable from prompt_builder build_with_conventions, /profile, telescope ordering; consumes rules.db seeder, conventions.db migration, HookVerdict from plugins. | 3L:L189-L214 |
| P3L-E019 | E/LIMIT | stub_ban: no .unwrap() outside tests or LazyLock init; regex only LazyLock; PiT3 never Block; PiT2 Ok(None) language/breaker. | 3L:L215-L221 |
| P3L-E020 | E/TEST | Observable: cargo test -p elai-knowledge conventions:: and elai_cc_contracts pi_telescope_contract; 4 language determinism + 5 telescope invariants; phase_3L.toml written. | 3L:L222-L227 |
| P3L-E021 | E/DET | DETERMINISTIC_CHECK rg list: extract_conventions, CorpusView, CONVENTIONS_SCHEMA_VERSION, PiT*Hook, ProjectIntelligenceHook, migration, persist_profile, assemble_telescope_with_hooks, pi_telescope_contract, build_with_conventions, calibration files. | 3L:L229-L251 |
| P3L-I001 | I/SURFACE | `/profile` is the primary operator CLI surface for reloading ProjectProfile; CONVENTIONS block is agent-visible prompt surface not a dashboard panel. | 3L:L148-L151, L119-L121 |
| P3L-A001 | A/CFG | Exact numeric value of PI_T2_CIRCUIT_BREAKER_LINES is not stated (only that it mirrors plugins T2). | 3L:L137-L140 |
| P3L-A002 | A/DATA | Full conventions.db SQL column schema beyond "migration_v1 + meta schema_version" is not expanded in this phase text. | 3L:L94-L97, L45-L46 |

Category coverage: capabilities/hooks ✓; surfaces /profile + CONVENTIONS prompt ✓; workflows/telescope ✓; schemas/migration ✓; failures SchemaTooOld ✓; determinism multi-lang ✓; observability calibration ✓; tests contract ✓; constraints Pi naming ✓. No privacy/compliance section.

### 4C — plugins hooks + tiered static analysis

Source: `docs/specs/guide/steps/phase-4C.txt` (380/380 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4C-E001 | E/STATUS | Phase 4C: invisible PreToolUse/PostToolUse/Stop/PreCompact hooks in orchestrator; tiered static analysis on SSB writes. Prerequisite: 4B tools via IPC. Agents never see hook existence. | 4C:L1-L7 |
| P4C-E002 | E/DEFER | Skills system and MCP bridge out of scope (skills=prompt templates no code path; MCP=1751 LOC claw-code later sub-phase). | 4C:L8-L10, L376-L380 |
| P4C-E003 | E/FLOW | Under Tier C: PreToolUse on ToolRequest/BashRequest before executor; PostToolUse after tool before ToolResult to agent; PostToolUse also ShortCoder code_simplify on AgentResponse before auditor view FS; Stop on session end/AgentKill; PreCompact when token_used/token_limit >0.85. | 4C:L22-L29, L258-L261 |
| P4C-E004 | E/SEC | Error sanitization: Block → generic IPC "Tool call rejected by policy" / "tool call rejected"; strip rule_id, pattern, severity, trading.* — agent must not see PreToolUse BLOCK from rule trading.14. Sanitize at HookVerdict enum mapping not string scrub only. | 4C:L34-L36, L66, L123-L129, L262-L263, L294-L296 |
| P4C-E005 | E/CAP | T1: predicate DSL from rules.db — AST_PATTERN, GREP_PATTERN, METRIC_GATE, COMPOSITE, SCHEMA_REQUIRED + phase_mask. T1 does not parse AST (T2 does). SCHEMA_REQUIRED → Escalate{target=ProviderTier} not ALLOW/BLOCK. Indexed SELECT by phase_mask & domain. Cache per phase; invalidate SHADOW→ACTIVE at next BSP superstep boundary. | 4C:L37-L41, L265-L272 |
| P4C-E006 | E/FLOW | T3 fires only when WARN exists AND no BLOCK from any tier. If T1=BLOCK and T2=WARN, T3 does not fire. Pattern-match only, no LLM. | 4C:L42-L43, L136-L137, L276-L278 |
| P4C-E007 | E/CFG | ruff PostToolUse is subprocess (~50–200ms); no public Rust API (astral-sh/ruff #12351). Circuit breaker skip ruff files >10000 lines. PostToolUse total target <500ms. | 4C:L44-L46, L251-L252, L337 |
| P4C-E008 | E/CAP | HookTrait methods pre_tool_use/post_tool_use/stop/pre_compact. HookVerdict Allow \| Block{reason} \| Warn{reason}. HookContext: fsm_state, agent_id, agent_role, session_id, active_rules. | 4C:L58-L61, L81 |
| P4C-E009 | E/CAP | TieredAnalysis T1/T2/T3 entry points. T2 tree-sitter syntax, import deny-list, dead/lazy code; circuit breaker >5000 lines → T1 only. T3 examples: fn>200 lines WARN:complexity; catch Exception:pass BLOCK:swallow. | 4C:L62-L69, L273-L280 |
| P4C-E010 | E/EVENT | BLOCK from any tier → SSB finding entry via single-writer channel. Orchestrator normalizes findings to blake2s pattern_hash (not auditor). PROPOSED rules source=AUDITOR never auto-promote; finding-source auditor cannot approve own findings. | 4C:L30-L33, L140-L148, L281-L285, L309-L310 |
| P4C-E011 | E/TEST | 30 contract tests plugins_hooks: generic block error; allow forward; ruff syntax block; stop IDLE gate; PreCompact checkpoint; sanitization; T1<1ms every SSB write; GREP pass-in-except BLOCK; T2 only file_write; T2<10ms <5000; T2 breaker >5000; T3 only WARN; T3<5ms; complexity/swallow patterns; SSB finding; IPC Tool/Bash PreToolUse; BLOCK prevents execution; normalization; AUDITOR no auto-promote; independent validation; cross-session lock write block/allow; SCHEMA_REQUIRED Escalate; T3 compound no-fire; ShortCoder placement + AST-equivalent. | 4C:L120-L175 |
| P4C-E012 | E/TEST | 10 adversarial: path traversal Unix+Windows; FSM-gated tool; independent Bash after Tool block; 1000-rule T1 still <1ms indexed; malformed predicate WARN; multi-WARN process all; T3 not on BLOCK; reason injection strip; same-auditor validate reject; AUDITOR flood stuck PROPOSED. | 4C:L197-L234 |
| P4C-E013 | E/SEC | PreToolUse loads get_active_rules for FSM state+domain; first BLOCK wins; cross-session file lock via SSB. Stop: non-IDLE Block, IDLE Allow. PreCompact: checkpoint before truncation. | 4C:L247-L257 |
| P4C-E014 | E/LIMIT | plugins depends on core/ + rules/ only; no plugins→orchestrator imports (orchestrator→plugins OK). Zero todo!/unimplemented!/pass bodies after implement. | 4C:L103-L110, L290, L304 |
| P4C-E015 | E/OBS | Metrics phase_4C.toml: T1/T2/T3 p50/p95/p99; hook overhead per IPC tool call; PostToolUse incl ruff; compile delta; test count; LOC; coverage. Targets T1 p99<1ms; T2 p99<10ms <5000 lines; T3 p99<5ms; hook overhead <2ms non-write; PostToolUse p99<500ms; coverage >80%; compile +<10s vs 4B. | 4C:L321-L347 |
| P4C-E016 | E/INT | RuntimeClient wiring: every routing call site/struct must wire ssb, fsm, clock; no None after construction. | 4C:L360 |
| P4C-E017 | E/DEP | 4A WARN mitigations: PostToolUse analysis is T1/T2/T3 not GRC WEB_CHECK; hooks synchronous before IPC response so GRC convergence cannot mid-cut; ShortCoder placement resolved tests 29-30. | 4C:L362-L374 |
| P4C-E018 | E/SURFACE | No agent-visible UI for hooks; operator-facing artifacts are SSB findings, metrics TOML, generic rejection errors on agent channel. | 4C:L4-L5, L355-L359 |
| P4C-I001 | I/CONC | T1 latency test note prefers Criterion benches over inline Instant in contracts; adversarial wall-clock Instant still listed as monotonic (not SystemTime). | 4C:L166-L167, L176 |
| P4C-A001 | A/CFG | Exact deny-list contents for T2 imports and lazy-code patterns beyond examples not fully enumerated. | 4C:L273-L275 |
| P4C-C001 | C/OBS | Adversarial note "ruff should use library binding" conflicts with decided subprocess + circuit breaker when binding unavailable; subprocess path is authoritative. | 4C:L44-L46, L297-L300 |

Category coverage: capabilities/hooks/tiers ✓; workflows/IPC intercept ✓; schemas/verdicts ✓; security sanitization+circular trust ✓; failures BLOCK/WARN ✓; concurrency BSP cache boundary ✓; observability metrics ✓; tests 40 ✓; deferred skills/MCP ✓. No formal privacy policy beyond agent isolation opacity.

### 4I — conditional media validation + Render-Capture-Evaluate

Source: `docs/specs/guide/steps/phase-4I.txt` (623/623 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4I-E001 | E/STATUS | Phase 4I conditional: implement only if pipeline tasks produce media. Prerequisites: 4A isolation. Validates images, 3D, audio, UI screenshots. Tier1 deterministic veto; Tier2 VLM advisory. | 4I:L1-L11 |
| P4I-E002 | E/FAIL | R15: VLM never auto-overrides Tier1; disagree → human review. R-NEW-1 tool availability degrade. R-NEW-2 GPU swap batch at phase boundary. R-NEW-3 Mode B requires SGLang. R-NEW-4 MoE VRAM stores all params. R-NEW-5 SSIM/LPIPS CPU EP + epsilon 1e-6. | 4I:L19-L39 |
| P4I-E003 | E/DATA | MediaValidationRequest: media_type Image\|Model3D\|Audio\|UiScreenshot; file_path; quality_spec; reference_path; mode Quick\|Full. Result: tier1, tier2 Option, final_verdict Pass\|Fail\|HumanReview\|Degraded, reason. | 4I:L72-L82 |
| P4I-E004 | E/CAP | Tier1Validator trait + Image/Model3D/Audio/UiScreenshot validators. ExternalToolWrapper check_available+execute timeout; ImageMagick/FFprobe/Tesseract/Blender wrappers. ToolCapabilityFlags built at startup; missing tool → DEGRADED check not panic. | 4I:L83-L91 |
| P4I-E005 | E/CFG | Mode A: Qwen3.5-VL-30B-A3B ollama Q4_K_M ~17GB; needs ollama≥0.12.7; Phase0 12GB cannot run (MockVlm). Mode B: InternVL3.5-38B SGLang only RadixAttention; ~38GB 8-bit dual 3090 or ~21GB Q4. Hardware guard nvml-wrapper. | 4I:L92-L103, L48-L49 |
| P4I-E006 | E/CFG | CLAP audio: Xenova/clap-htsat-unfused ONNX; ~200M; FP16 ~0.4GB always-loaded when audio active; raw PCM input. | 4I:L104-L107, L271-L276 |
| P4I-E007 | E/DATA | VlmValidationOutput constrained: overall_quality f32[0,1], issues[], confidence, reasoning; Issue severity Critical\|Major\|Minor + category/description/location. Mode B SGLang schema; Mode A format=json + post-validation; retry once temp=0 then Degraded. | 4I:L108-L118 |
| P4I-E008 | E/TEST | Contracts: T1 before T2; T1 fail→Fail ignore T2; T2 only if T1 pass + qualitative checks; schema conform; only media PipelineSpec; SSIM epsilon; T2 disagree→HumanReview. ≥20 tests base; ≥8 adversarial; ≥15 RCE; total ≥43. | 4I:L172-L200, L231-L240, L495-L496, L551 |
| P4I-E009 | E/SEC | Adversarial: EXIF/overlay prompt injection still schema-bound + Tier1-based verdict; 500MB size reject pre-VLM; malformed headers no panic; path must canonicalize inside output dir (symlink reject); VLM no /proc or parent enum; queue 64 overflow QueueFull; max 4 Tier1 / 1 Tier2 concurrent; ComfyUI timeout 600s cleanup; CLAP noise low score deterministic. | 4I:L211-L240 |
| P4I-E010 | E/CAP | Image: identify format/resolution/colorspace; size; SSIM image crate CPU EP; LPIPS ONNX CPU EP. 3D: Blender headless gated BLENDER flag. Audio: FFprobe + RMS silence + peak clip integer math. UI: resolution + SSIM mockup + Tesseract OCR. | 4I:L253-L263 |
| P4I-E011 | E/FLOW | Veto matrix: T1 fail→Fail; T1 pass no qualitative→Pass; T1+T2 pass→Pass; T1 pass T2 fail→HumanReview; T1 pass T2 disagree→HumanReview; schema-invalid after retry→Degraded; VLM unavailable→Degraded. | 4I:L277-L287 |
| P4I-E012 | E/CAP | Lazy load: ToolRegistry::register_lazy; factory builds flags + optional VLM on first MediaValidationRequest; non-media PipelineSpec never loads. VLM uses 15-message postcard IPC like agents; startup_timeout default 30s. | 4I:L288-L294 |
| P4I-E013 | E/CONC | Tier1 max 4 concurrent; Tier2 max 1; queue depth 64 → Error::QueueFull. | 4I:L295-L298, L223-L225 |
| P4I-E014 | E/OBS | Metrics phase_4I.toml: tier1_p95_ms (<100 image/audio/ui; 3D <500); tier2 mode A/B; gpu_swap mode A <15s / B <60s; vram; test counts; later RCE fields rce_* render/loop/convergence. | 4I:L331-L362, L578-L602 |
| P4I-E015 | E/CAP | MediaRenderAdapter trait: can_handle, async render→CapturedFrame[], cleanup. Types: Viewport, FrameMetadata, RenderConfig, CaptureMode Screenshot\|KeyFrames\|MultiAngle, Interaction Click/Scroll/Wait/Hover/Type, CameraAngle. | 4I:L380-L395 |
| P4I-E016 | E/CAP | Adapters: WebRenderAdapter Playwright; BlenderRenderAdapter; AnimationRenderAdapter FFmpeg; UiScreenshotAdapter pass-through; ComfyUIRenderAdapter REST. AdapterRegistry by OutputType from ToolCapabilityFlags. | 4I:L396-L406, L513-L547 |
| P4I-E017 | E/FLOW | VisualFeedbackLoop: render→T1→T2→issues; stop on T1 fail, max_iterations default 3, or issues unchanged (compare severity+category+location not raw string). Feedback structured VisualIssue with actionable_fix to executor prompt. | 4I:L407-L416, L451-L456, L535-L542 |
| P4I-E018 | E/EVENT | SSB action_type='visual_feedback' per iteration {iteration, issue_count, resolved_count, verdict} so orchestrator knows agent alive. | 4I:L492-L494, L543-L544, L566 |
| P4I-E019 | E/SURFACE | Functional viewports web 375/768/1440; Blender 4 angles; FFmpeg keyframes; ComfyUI mock never real in tests. visual_check=false → no render overhead. | 4I:L449-L469, L458-L459 |
| P4I-E020 | E/SEC | VLM process isolation: no read outside allowed_paths; no write; no network. Path traversal reject. Tier C seccomp/RLIMIT from 4A. | 4I:L185-L188, L221-L222 |
| P4I-E021 | E/LIMIT | architecture-overview L283 stale LLaVA models; media-validator.txt L60 "Active params ~3B" misleading for VRAM. RuntimeClient ssb/fsm/clock wiring required. | 4I:L41-L46, L613 |
| P4I-E022 | E/STATUS | Phase 4 overall exit: arbitrary task design pipeline INTERACTIVE; AUTONOMOUS on top (4G); mode switch preserves state; isolated agents; no enforcement visibility; isolation escape test; overhead <5%; self-mod cannot touch evaluation criteria. | 4I:L615-L623 |
| P4I-E023 | E/DEP | PRE_GATE tools: ImageMagick+FFmpeg required; Tesseract/Blender optional degrade; Playwright for RCE; PhaseEntry output_type/visual_check from 3G; L2 MissingAdapter/InvalidVisualCheck from 4E. | 4I:L61-L70, L376-L378 |
| P4I-I001 | I/SURFACE | HumanReview/Degraded verdicts are operator-facing outcomes (review queue semantics) even though no dashboard panel is specified in this phase file. | 4I:L81, L277-L287 |
| P4I-A001 | A/CFG | Exact QualitySpec threshold field names for SSIM/LPIPS/CLAP not restated here (owned by pipeline-factory QualitySpec). | 4I:L16, L75 |
| P4I-A002 | A/CONC | Whether max_iterations is globally fixed or QualitySpec-overridable beyond default 3 is only partially specified (field exists on VisualFeedbackLoop). | 4I:L408-L409, L451 |
| P4I-C001 | C/EVIDENCE | architecture-overview media models conflict with media-validator Mode A/B; phase binds media-validator as authority and flags overview stale. | 4I:L41-L45 |

Category coverage: capabilities/validators/adapters ✓; surfaces verdicts+viewports+SSB visual_feedback ✓; workflows veto+feedback loop ✓; schemas VLM JSON ✓; flags ToolCapabilityFlags+lazy ✓; security isolation/path ✓; failures degrade/queue ✓; determinism CPU EP ✓; concurrency bounds ✓; observability metrics ✓; tests ≥43 ✓; deferred non-media skip ✓.

### Cross-source interactions (PHASE-004)

| Interaction ID | Class | Interaction | Atom references |
|---|---|---|---|
| PHASE-004/IX-001 | E | 1E composes 1B BM25 + 1C vectors + 1D PageRank/DKG; leaves explicit stubs for 1G rerank and 3E token_fit. | P1E-E001, P1E-E006, P1E-E026 |
| PHASE-004/IX-002 | E | 1E density guard and circuit-breaker skips feed 1I resilience/mode cascade and QueryTrace stage list. | P1E-E007, P1E-E023, P1E-E027 |
| PHASE-004/IX-003 | E | 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy. | P2B-E012, P2B-E019, P2C-E006 |
| PHASE-004/IX-004 | E | 2B plan sweep supplies the ungrounded-zero evidence gate for 3A PLANNING→TESTING_FIRST / EXECUTING path. | P2B-E003, P2B-E020 |
| PHASE-004/IX-005 | E | 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC. | P2C-E006, P2C-E015, P2B-E005 |
| PHASE-004/IX-006 | E | 2C auditor double-verify + FINDING anti-rubber-stamp is last defense before acceptance; composes with 3A AUDITING evidence. | P2C-E004, P2C-E007 |
| PHASE-004/IX-007 | E | 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family. | P3L-E003, P4C-E009 |
| PHASE-004/IX-008 | E | 3L CONVENTIONS injection via reasoning prompt_builder and /profile CLI; telescope priorities sit upstream of agent SYSTEM context. | P3L-E012, P3L-E015, P3L-E007 |
| PHASE-004/IX-009 | E | 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline. | P3L-E002, P1E-E001 |
| PHASE-004/IX-010 | E | 4C PostToolUse ShortCoder placement resolves 4B tools.txt code_simplify + 4A WARN-3 under Tier C before auditor FS. | P4C-E003, P4C-E017 |
| PHASE-004/IX-011 | E | 4C T1 SCHEMA_REQUIRED Escalate to ProviderTier couples rules DSL to provider constrained decoding (outside 4C body). | P4C-E005, P4C-E011 |
| PHASE-004/IX-012 | E | 4C circular-trust normalization + AUDITOR PROPOSED freeze composes with rules.txt self-discovery and 3F-style promotion gates. | P4C-E010 |
| PHASE-004/IX-013 | E | 4I lazy media path depends on 4D MediaSpec / 3G visual_check+output_type and 4E adapter structural check #8. | P4I-E001, P4I-E012, P4I-E023 |
| PHASE-004/IX-014 | E | 4I VLM process uses 4A Tier C isolation + same postcard IPC as agents; ToolCapabilityFlags degrade without panicking. | P4I-E012, P4I-E020, P4I-E004 |
| PHASE-004/IX-015 | E | 4I VisualFeedbackLoop writes SSB visual_feedback so 3B journal and 3C/4G liveness observe long RCE iterations. | P4I-E018, P4I-E017 |
| PHASE-004/IX-016 | E | Hash split in chunk: 4C violation_patterns blake2s vs knowledge blake2 elsewhere; 4I does not introduce new content hash for media. | P4C-E010 |
| PHASE-004/IX-017 | E | Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary). | P2C-E015, P4C-E001 |
| PHASE-004/IX-018 | I | 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards. | P1E-E028, P2B-E021, P2C-E016, P3L-E017, P4C-E015, P4I-E014 |
| PHASE-004/IX-019 | E | 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth). | P4C-E013, P2B-E017 |
| PHASE-004/IX-020 | E | Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory. | P2C-E018, P4I-E001 |

### Ambiguity and conflict register (PHASE-004)

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| PHASE-004/AC-001 | conflict | 1E "sub-second" goal vs 2s hard interactive budget. | P1E-C001 |
| PHASE-004/AC-002 | ambiguity | 1E heuristic edge-score aggregation into 0–10 before phi not fully formal. | P1E-A002 |
| PHASE-004/AC-003 | ambiguity | 1E SPLADE in knowledge order vs no 1E splade stage. | P1E-A001 |
| PHASE-004/AC-004 | ambiguity | 2B claim regex / Uncertain path-vs-symbol heuristics incomplete. | P2B-A001, P2B-A002 |
| PHASE-004/AC-005 | conflict | reasoning.txt 4-field AgentGrcConfig vs 2C 8-field requirement + GroundingLevel order fix in 2A. | P2C-C001, P2C-E013, P2C-E020 |
| PHASE-004/AC-006 | ambiguity | FINDING: prefix vs future structured FindingReport. | P2C-A001 |
| PHASE-004/AC-007 | ambiguity | PI_T2_CIRCUIT_BREAKER_LINES exact value; full conventions SQL columns. | P3L-A001, P3L-A002 |
| PHASE-004/AC-008 | conflict | 4C adversarial "prefer ruff library" vs decided subprocess+breaker. | P4C-C001 |
| PHASE-004/AC-009 | ambiguity | T2 deny-list / lazy-code pattern catalog incomplete in phase text. | P4C-A001 |
| PHASE-004/AC-010 | conflict | architecture-overview media models stale vs 4I Mode A/B authority. | P4I-C001 |
| PHASE-004/AC-011 | ambiguity | QualitySpec threshold field names and max_iterations override policy. | P4I-A001, P4I-A002 |
| PHASE-004/AC-012 | qualification | MASTER marks phase-3L as unindexed inventory item; deep extract is authoritative for 3L atoms. | P3L-E001 |

### Summary deltas vs MASTER (PHASE-004)

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-1E-001 | **ADD** full risk set R30–R34 with mitigation numbers (N_seed, K_edge, density 0.01, max_loops, ±2 lines, 2s budget). | MASTER:L99-L112 | P1E-E007 |
| PHASE-004/SD-1E-002 | **ADD** sparse CatRagOverlay type fields, QueryPipeline/PipelineStage schema, and measure_1e five-query catalog Q1–Q5 + phase_1E.toml keys. | MASTER:L103-L112 | P1E-E009, P1E-E015, P1E-E028 |
| PHASE-004/SD-1E-003 | **ADD** deterministic 1E A-RAG fixed tool order vs deferred ReAct Phase 2+; stage failure fall-through; RRF rank-from-1. | MASTER:L111 | P1E-E025, P1E-E027 |
| PHASE-004/SD-1E-004 | **ADD** match-cap 2*N_seed and >50% DKG match warning; optional LLM edge scoring via breaker only. | MASTER:L107 | P1E-E020, P1E-E021 |
| PHASE-004/SD-1E-005 | **FLAG** sub-second vs 2s acceptance tension absent from summary compression. | MASTER:L100, L112 | P1E-C001 |

### 2B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-2B-001 | **ADD** PlanGroundingResult verify_needed semantics (training-only subset of ungrounded) and empty-plan trivially grounded. | MASTER:L194 | P2B-E008, P2B-E020 |
| PHASE-004/SD-2B-002 | **ADD** complete prefix dispatch table including grep→FileCheck and tool→SourceCheck; 30s cargo test timeout; BashSandbox API. | MASTER:L198-L201 | P2B-E017-P2B-E019 |
| PHASE-004/SD-2B-003 | **ADD** full T1–T16 catalog, network/timing bans, tempfile cleanup, and dependency gate (no knowledge in cargo tree). | MASTER:L197, L196 | P2B-E015, P2B-E016, P2B-E013 |
| PHASE-004/SD-2B-004 | **ADD** Uncertain auto-resolution during think() enhancement of 2A; 4A wrap-without-signature-change constraint. | MASTER:L191, L200 | P2B-E004, P2B-E005 |
| PHASE-004/SD-2B-005 | **QUALIFY** summary network stub string vs source "not implemented in phase 2B" — behavioral typed error is the requirement. | MASTER:L200 | P2B-C001 |

### 2C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-2C-001 | **ADD** complete T1–T15 test catalog and role budgets (executor 8/1200, auditor 10/1200, memory_writer 2/400, prompt_engineer 5/800, designer distinct). | MASTER:L209, L215 | P2C-E011, P2C-E018 |
| PHASE-004/SD-2C-002 | **ADD** whitelist formula `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})` and with_agent_config topic reset. | MASTER:L216 | P2C-E012, P2C-E014 |
| PHASE-004/SD-2C-003 | **ADD** FINDING_PREFIX contract, scope enumeration WARN path, and five validation rules D2 including csq_required. | MASTER:L211-L212 | P2C-E005, P2C-E007, P2C-E008 |
| PHASE-004/SD-2C-004 | **ADD** full SPEC GAPS list (8-field type, GrcError variants, PartialOrd order, rubber-stamp invariant, methods vs quality complementarity). | MASTER:L207-L218 | P2C-E020, P2C-C001 |
| PHASE-004/SD-2C-005 | **ADD** metrics memory <2KB and malformed-table fail vs missing-table default distinction. | MASTER:L218 | P2C-E016, P2C-E014 |

### 3L deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-3L-001 | **ADD** entire Phase 3L feature set: MASTER only notes phase-3L.txt as unindexed file to inspect (Q-line), without atom extraction. | MASTER:L655, L713 | P3L-E001–P3L-E021 |
| PHASE-004/SD-3L-002 | **ADD** Pi-prefixed hook naming, LazyLock D2, conventions_migration_v1 flat path, seed_rules API, profile.toml + /profile. | MASTER:L655 | P3L-E003–P3L-E008 |
| PHASE-004/SD-3L-003 | **ADD** telescope priorities 100/80/60, TELESCOPE_MAX_CHARS 1200, pi_telescope_contract five invariants, calibration gates 50/100 ms. | MASTER:L655 | P3L-E015–P3L-E017 |
| PHASE-004/SD-3L-004 | **ADD** four-language determinism matrix and CONVENTIONS prompt block budget 200 tokens. | MASTER:L655 | P3L-E010, P3L-E012 |
| PHASE-004/SD-3L-005 | **ADD** 3K predecessor correction (not Tier0 scaffold) and reuse of plugins HookVerdict without forking 4C tiers. | MASTER:L655 | P3L-E002, P3L-E003 |

### 4C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-4C-001 | **ADD** complete 30+10 test catalogs, cross-session locks, SCHEMA_REQUIRED Escalate, ShortCoder AST-equivalent tests. | MASTER:L421 | P4C-E011, P4C-E012 |
| PHASE-004/SD-4C-002 | **ADD** PreCompact 0.85 handoff threshold, ruff >10000 breaker, T2 >5000 breaker, BSP cache invalidation timing. | MASTER:L413-L418 | P4C-E003, P4C-E007, P4C-E005 |
| PHASE-004/SD-4C-003 | **ADD** exact sanitization string "Tool call rejected by policy", plugins dependency direction, RuntimeClient ssb/fsm/clock rule. | MASTER:L419 | P4C-E004, P4C-E014, P4C-E016 |
| PHASE-004/SD-4C-004 | **ADD** 4A WARN-1/WARN-6 mitigations and metrics nine categories with coverage >80%. | MASTER:L422 | P4C-E017, P4C-E015 |
| PHASE-004/SD-4C-005 | **ADD** deferred skills/MCP rationale (no 4D–4G dependency on 4C for those). | MASTER:L423 | P4C-E002 |

### 4I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-004/SD-4I-001 | **ADD** full veto matrix including schema-invalid Degraded and VLM-unavailable Degraded; Mode A post-validation field checks + retry. | MASTER:L506-L507 | P4I-E007, P4I-E011 |
| PHASE-004/SD-4I-002 | **ADD** complete adversarial set (injection, 500MB, headers, path/symlink, sandbox, queue 65, ComfyUI 600s, CLAP noise). | MASTER:L496-L507 | P4I-E009 |
| PHASE-004/SD-4I-003 | **ADD** full RCE type system (MediaRenderAdapter, five adapters, VisualFeedbackLoop, VisualIssue, convergence by severity/category/location). | MASTER:L505 | P4I-E015–P4I-E017 |
| PHASE-004/SD-4I-004 | **ADD** SSB visual_feedback liveness, lazy register_lazy factory, ToolCapabilityFlags, MockVlm Phase0, ollama ≥0.12.7. | MASTER:L498-L502 | P4I-E012, P4I-E004, P4I-E005, P4I-E018 |
| PHASE-004/SD-4I-005 | **ADD** RCE latency targets (web <3s, Blender <10s, loop web <10s/iter), convergence ≥70% measure, total ≥43 tests, Phase 4 overall exit block. | MASTER:L506-L507 | P4I-E014, P4I-E008, P4I-E022 |
| PHASE-004/SD-4I-006 | **ADD** stale architecture-overview LLaVA note and MoE VRAM clarification conflict with media-validator L60. | MASTER:L501-L502 | P4I-E021, P4I-C001 |
| PHASE-004/SD-4I-007 | **QUALIFY** summary "bounded N" iterations: source default max_iterations=3 with early stop on unchanged issues. | MASTER:L505 | P4I-E017 |

## PHASE-005 — source `DEEP-PHASE-005.discovery.md`

- Declared feature atoms: **213** (E=190 I=5 A=12 C=6).
- Parsed feature rows in this aggregate: **213**.
- Per-file: 1F **049**; 2A **038**; 3F **050**; 3J **076**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-005.discovery.md`.

### Feature atoms

### 1F — speculative decoding evaluation

Source: `docs/specs/guide/steps/phase-1F.txt` (332/332 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1F-E001 | E/STATUS | Phase 1F is SPECULATIVE DECODING EVALUATION: an evaluation phase whose output is a measured ADOPT/DROP decision, not a feature ship. Status stamp 2026-04-07. | 1F:L1-L14, L24-L27 |
| P1F-E002 | E/STATUS | Steps 1F.1+1F.2 shipped: SpeculativeConfig, OllamaConfig nesting, ProbeResult, probe_speculative_support in ollama.rs with unit+live tests. | 1F:L3-L6 |
| P1F-E003 | E/EVIDENCE | Live probe on ollama v0.20.2 returned NOT_AVAILABLE: PR #8134 closed never-merged; draft_model silently ignored; accepted_tokens always null. | 1F:L6-L8, L56-L59 |
| P1F-E004 | E/DEFER | Steps 1F.3 / 1F.3b / 1F.4 / 1F.5 are DEFERRED-BY-DESIGN to Phase 2+ (vLLM provider); not missing work. | 1F:L8-L10, L58-L60 |
| P1F-E005 | E/OBS | Per-step TOMLs (correctness/throughput/vram/prompt_lookup) intentionally not produced; consolidated `.elai_cc/metrics/phase_1F.toml` records decision=UNAVAILABLE with reason. | 1F:L10-L12 |
| P1F-E006 | E/STATUS | EXIT CRITERIA "Decision made and documented: ADOPT or DROP" is satisfied via DROP path. | 1F:L13-L14, L309-L310 |
| P1F-E007 | E/DEP | Prerequisite corrected to Phase 1C complete (vector search + VRAM accounting); dependency graph L141 shows 1F branches from 1C; original "Phase 0 independent of 1A-1E" was wrong. | 1F:L16-L20, L100-L101 |
| P1F-E008 | E/CAP | Goal: evaluate whether speculative decoding improves throughput enough to justify VRAM cost; adopt if >1.3x throughput on target hardware, drop if less. | 1F:L24-L26, L321-L322 |
| P1F-E009 | E/DEP | Mandatory pre-reads: providers.txt L87-97, external-analysis EAGLE-3/TIDE, architecture-overview §7 HARDWARE TARGET L233-252. | 1F:L29-L32 |
| P1F-E010 | E/LIMIT | R4 VRAM: Gemma 4 E2B ~7.2GB Q4_K_M leaves ~4.8GB headroom on RTX 3060 12GB (not ~10GB as providers.txt L47 stated). Scenario A ~2GB draft → ~2.8GB free; B ~3GB delta → ~1.8GB free. | 1F:L34-L41, L76-L78 |
| P1F-E011 | E/OBS | Monitor VRAM via `nvidia-smi --query-gpu=memory.used` with polling during benchmark, not static snapshots. | 1F:L40-L41, L287-L289, L293-L295 |
| P1F-E012 | E/FAIL | R-NEW-1: Ollama speculative support unconfirmed; fallback (a) llama.cpp speculative only in Phase 1 (vLLM Phase 2+), (b) defer until ollama merges; else DROP/defer Phase 2 re-eval. | 1F:L42-L47 |
| P1F-E013 | E/LIMIT | R-NEW-2: EAGLE-3 published 2x-6.5x is for large (70B+) models; gains on 2.3B consumer GPU will be substantially lower; 1.3x threshold accounts for this. | 1F:L48-L50, L328-L329 |
| P1F-E014 | E/LIMIT | R-NEW-3: Gemma 4 EAGLE-3 draft may not exist (vLLM issue #38893); if missing, use same-family smaller draft, not EAGLE-3 head. Step 1F.2 verifies availability. | 1F:L51-L55 |
| P1F-E015 | E/STATUS | RESOLVED 2026-04-07: DROP speculative decoding for Phase 1; Prompt Lookup Decoding remains viable zero-VRAM alternative; revisit Phase 2+ with vLLM. | 1F:L56-L64 |
| P1F-E016 | E/CAP | OllamaProvider + LlmProvider trait abstraction from 1F.1 means vLLM integration is a new provider impl, not a rewrite. | 1F:L63-L64 |
| P1F-E017 | E/EVIDENCE | Grounding: EAGLE-3 corrected to 2x-6.5x (arxiv 2503.01840); TIDE 1.15x over static verified (arxiv 2602.05145); "Ollama supports natively" UNVERIFIED until probe; >1.3x threshold UNGROUNDED design decision. | 1F:L71-L85 |
| P1F-E018 | E/CFG | Draft model size guidance from llama.cpp #10466: smaller drafts often better net speedup; prefer 0.5B-1.5B over 2B-3B for Gemma 4 E2B target. | 1F:L86-L90, L145-L149 |
| P1F-E019 | E/CFG | num_draft_tokens default 5 is suboptimal; step 1F.4 uses parameter sweep instead of fixed k=5. | 1F:L91-L93, L114-L116, L229-L232 |
| P1F-E020 | E/OBS | Acceptance rate is a required metric; below 0.5 speculative decoding hurts throughput (early kill signal). | 1F:L94-L96, L234-L237, L324-L324 |
| P1F-E021 | E/CAP | Prompt lookup decoding added as step 1F.3b: zero-VRAM, up to 2.8x on summarization (vLLM blog); baseline before committing VRAM to draft. | 1F:L97-L99, L194-L219 |
| P1F-E022 | E/CFG | 1F.1 fields: speculative_decoding bool default false; draft_model Option<String>; num_draft_tokens u8 default 5; nested SpeculativeConfig or on OllamaConfig; serde round-trip. | 1F:L106-L117, L128-L130 |
| P1F-E023 | E/TEST | 1F.1 adversarial: serde name mismatch silent drop (round-trip test); num_draft_tokens=0 invalid (validate >=1). | 1F:L122-L126 |
| P1F-E024 | E/TEST | 1F.1 PRE_GATE: Phase 1C cargo test knowledge lib; providers cargo check; OllamaConfig exists via grep. | 1F:L107-L110 |
| P1F-E025 | E/CAP | 1F.2 probe checks ollama draft model param on /api/generate or /api/chat; outputs SUPPORTED/NOT_SUPPORTED; if neither ollama nor llama.cpp viable → DROP document. | 1F:L132-L162 |
| P1F-E026 | E/SURFACE | Decision artifact path: workflow/reasoning/phase-1F-decisions.md documents fallback and ADOPT/DROP with measurements. | 1F:L152-L153, L161-L161, L285-L285, L315-L315 |
| P1F-E027 | E/TEST | 1F.2 adversarial: silent ignore of unknown draft_model (false positive) must be disproved by timing/headers; record ollama --version. | 1F:L153-L158 |
| P1F-E028 | E/TEST | 1F.3 correctness: 10 prompts min; baseline vs speculative same prompt+seed; soft text match 10/10; token ID divergence WARN not fail; temp=0 + fixed seed; unique prompts/clear cache. | 1F:L164-L192 |
| P1F-E029 | E/DATA | Correctness metrics file `.elai_cc/metrics/phase_1F_correctness.toml`. | 1F:L179-L179, L192-L192 |
| P1F-E030 | E/CAP | 1F.3b prompt lookup: same 5 prompts as 1F.4, 3 runs each (15 measurements); ratio vs baseline; if ollama lacks support check llama.cpp --lookup-ngram-min; else UNAVAILABLE. | 1F:L194-L219 |
| P1F-E031 | E/DATA | Prompt lookup metrics: `.elai_cc/metrics/phase_1F_prompt_lookup.toml`. | 1F:L209-L209 |
| P1F-E032 | E/FLOW | Decision preference: if prompt_lookup >1.3x AND spec_decode also >1.3x, prefer prompt_lookup for zero VRAM; if both >1.3x and spec better, document both. | 1F:L217-L219, L277-L281 |
| P1F-E033 | E/TEST | 1F.4 throughput: 5 diverse prompts; k sweep {1,2,3,4,5,7}; 3 runs per prompt per k per mode; record acceptance rate; early kill if acceptance <0.5 across all k for 3+ prompts → DROP. | 1F:L221-L263 |
| P1F-E034 | E/OBS | Per run: prompt/output length, wall time, tokens/sec, GPU util, acceptance rate; mean+stddev per mode at best-k; ratio single number; stddev <20% of mean per mode independently. | 1F:L233-L245, L256-L261 |
| P1F-E035 | E/DATA | Throughput metrics: `.elai_cc/metrics/phase_1F_throughput.toml`. | 1F:L241-L241 |
| P1F-E036 | E/TEST | 1F.4 adversarial: GPU temp >85C discard + 60s cooldown; ≥2 prompts with >200 output tokens; warm-up exclude first run; timing proxy if acceptance not exposed. | 1F:L246-L255 |
| P1F-E037 | E/FLOW | 1F.5 decision gate top-to-bottom: NOT_AVAILABLE→DROP; acceptance<0.5→DROP; prompt_lookup>1.3x and spec≤lookup→DROP spec ADOPT lookup; spec>1.3x and delta_vram_mb<3000→ADOPT with best-k; else DROP; boundary ratio==1.3x or delta==3000 → DROP (conservative). | 1F:L265-L284 |
| P1F-E038 | E/CFG | If ADOPT: providers config default enables speculative decoding, best-k and draft model set; if DROP: default remains disabled. | 1F:L290-L291, L305-L306, L316-L318 |
| P1F-E039 | E/DATA | VRAM metrics: `.elai_cc/metrics/phase_1F_vram.toml` with PEAK baseline and PEAK speculative via nvidia-smi polling. | 1F:L287-L289 |
| P1F-E040 | E/LIMIT | ADOPT decision applies to Phase 0 hardware only; production re-evaluation needed in Phase 4 when hardware changes. | 1F:L296-L298 |
| P1F-E041 | E/TEST | Marginal ratio within 0.05 of 1.3x: run 5 additional prompts and recompute. | 1F:L299-L300 |
| P1F-E042 | E/TEST | Exit: Phase 0 + 1A-1C tests still pass (no regression); decision binary ADOPT or DROP (no maybe). | 1F:L301-L304, L319-L319 |
| P1F-E043 | E/OBS | Measurement targets: throughput >1.3x to adopt; acceptance >0.5; VRAM delta <3000MB; draft size 0.5B-1.5B expected optimal; TIDE only if static adopted and further opt desired. | 1F:L321-L332 |
| P1F-E044 | E/SURFACE | No dashboard UI in 1F; operator-visible surfaces are decision markdown, metrics TOMLs, and probe SUPPORTED/NOT_SUPPORTED console output. | 1F:L10-L14, L151-L152, L285-L289 |
| P1F-I001 | I/DEP | "Phase 2+" revisit wording implies any post-Phase-1 vLLM path; concrete provider step is Phase 3J, not a Phase-2 numbered phase. | 1F:L8-L10, L59-L60 |
| P1F-A001 | A/CFG | Exact draft model name/artifact for Gemma 4 family is unresolved until 1F.2 availability check. | 1F:L51-L55, L145-L149 |
| P1F-A002 | A/TEST | Soft text match algorithm (normalization, whitespace) is not fully specified beyond "identical text content". | 1F:L175-L176 |
| P1F-C001 | C/DEP | Original prerequisite "Phase 0 independent of 1A-1E" conflicts with dependency graph 1C branch; corrected to Phase 1C. | 1F:L16-L20, L100-L101 |
| P1F-C002 | C/STATUS | Opening evaluation plan describes full 1F.3-1F.5 suite, but RESOLVED block permanently skips them for Phase 1 after NOT_AVAILABLE probe. | 1F:L3-L14, L56-L60, L164-L306 |

Category coverage: capabilities/actions ✓; surfaces (CLI/metrics/decision docs) ✓; workflows/decision gates ✓; schemas/metrics data ✓; defaults/config ✓; integrations (ollama/llama.cpp/nvidia-smi) ✓; security/privacy N/A named; failures/fallback ✓; determinism (seed/temp) ✓; observability/evidence ✓; tests/done gates ✓; constraints/deferred ✓. No formal retry/idempotency API; concurrency not in scope.

### 2A — GRC state machine (basic)

Source: `docs/specs/guide/steps/phase-2A.txt` (382/382 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2A-E001 | E/STATUS | Phase 2A: GRC STATE MACHINE (basic). Prerequisites: Phase 0 complete (contract tests pass, cargo check clean). | 2A:L1-L2 |
| P2A-E002 | E/CAP | Goal: GRC engine core — all reasoning cites grounding; budget enforces per-topic call/char limits; state machine validates transition order; rejected steps produce typed errors. | 2A:L5-L9 |
| P2A-E003 | E/LIMIT | Scope excludes grounding verification methods (2B), per-agent config (2C), CSQ (2D). | 2A:L11-L13 |
| P2A-E004 | E/DEP | Pre-reads: reasoning.txt §§1-3,10,11 items 1-3; architecture-overview §4 GrcEngine; prompt-architecture §1. | 2A:L16-L21 |
| P2A-E005 | E/STATE | D1: GrcState with allowed_next(); observe→hypothesize→plan_action→act→verify; reject invalid (e.g. hypothesize→verify). | 2A:L26-L32, L72-L76 |
| P2A-E006 | E/FAIL | D2: GrcError { UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding } with context; thiserror. | 2A:L34-L37, L96-L100 |
| P2A-E007 | E/CFG | D3: GrcBudget tracks remaining_calls/chars; when remaining_calls<=1 engine forces convergence (must produce conclusion). | 2A:L39-L44, L89-L93 |
| P2A-E008 | E/DATA | D4: grounding string parsed into source citations format "source_type:reference" (e.g. file:src/foo.rs:L42); 2A format-only, 2B verifies sources. | 2A:L46-L52, L273-L285 |
| P2A-E009 | E/STATE | D5: GroundingLevel VERIFIED > HIGH > UNCERTAIN (PartialOrd); UNCERTAIN flagged warn log, not rejected in 2A. | 2A:L54-L57, L94-L95, L279-L283 |
| P2A-E010 | E/CAP | 2A.1 greenfield: create crates/reasoning with Cargo.toml dep core; add to workspace. | 2A:L62-L70 |
| P2A-E011 | E/STATE | GrcState enum: Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status; Converge and Status allowed from any state; first non-Status/non-Converge step must be Observe. | 2A:L72-L76, L217-L220, L269-L271 |
| P2A-E012 | E/DATA | ThinkStep { step_type, content, topic, grounding: Option<String>, conclusion: Option<String>, scope: Option<Vec<String>> }. | 2A:L77-L85 |
| P2A-E013 | E/CFG | scope: when AgentGrcConfig.require_scope_enumeration=true, Observe must populate scope; orchestrator validates every scope item appears in subsequent plan_action; unaddressed → WARN into agent context. | 2A:L81-L85, L222-L237, L364-L364 |
| P2A-E014 | E/DATA | ThinkResult { step, budget_remaining: GrcBudget, grounding_level: Option<GroundingLevel>, warnings: Vec<String> }. | 2A:L87-L88, L305-L307 |
| P2A-E015 | E/CFG | GrcBudget default max_calls=5, max_chars=800 ("moderate"); methods remaining_calls, remaining_chars, is_exhausted, must_converge (remaining_calls<=1). | 2A:L89-L93, L211-L215 |
| P2A-E016 | E/CAP | GrcEngine trait 2A methods: think, set_budget, get_budget, reset_topic; 2B stubs unimplemented: verify_claim, ground_plan; full 6-method trait for stability. | 2A:L101-L112, L142-L144 |
| P2A-E017 | E/DATA | 2B forward stubs: VerificationResult, GroundingResult with #[allow(dead_code)]. | 2A:L113-L116 |
| P2A-E018 | E/STATE | BasicGrcEngine: HashMap<String, TopicState> with budget, last_state, steps, grounding_log; default budget applied on first think if no set_budget. | 2A:L117-L120, L300-L303 |
| P2A-E019 | E/DEP | 2A.2: reasoning depends only on core/, thiserror, serde — no providers/, tools/, knowledge/; pure logic, no LLM calls. | 2A:L146-L164 |
| P2A-E020 | E/TEST | 2A.3 TDD: 12 failing tests T1-T12 covering ungrounded observe, budget overflow/forced converge (max_calls=3), per-topic isolation, invalid transition, must_converge, malformed grounding, free Status, default budget, first-step Observe, scope present/missing/optional. | 2A:L166-L255 |
| P2A-E021 | E/TEST | T2 uses max_calls=3 not 2 so must_converge at remaining<=1 does not mask overflow test. | 2A:L179-L189 |
| P2A-E022 | E/TEST | T7 Status is free: does not consume budget (calls_used stays 0). | 2A:L207-L209, L362-L362 |
| P2A-E023 | E/TEST | Stubs must use todo!/unimplemented!, not Ok placeholders; zero timing-dependent tests. | 2A:L247-L254 |
| P2A-E024 | E/FLOW | I1 transitions: last_state.allowed_next(); first step Observe; Status does not update last_state; Converge allowed any state. | 2A:L265-L271 |
| P2A-E025 | E/DATA | I2 grounding required on Observe, Hypothesize, Verify; optional on PlanAction/Act; none on Status. Parse comma-separated type:reference. Valid types: file, grep, test, search, tool, training. | 2A:L273-L285 |
| P2A-E026 | E/DATA | GroundingLevel map: test/search→Verified; file/grep/tool→High; training→Uncertain; warn on Uncertain, do not reject. | 2A:L279-L283 |
| P2A-E027 | E/FLOW | I3 budget check-then-increment: exhaust calls, chars overflow, or must_converge without Converge → BudgetExhausted before increment; then saturating_add. | 2A:L287-L298 |
| P2A-E028 | E/STATE | I4 reset_topic clears all state for topic; entry fully removed from map (not zeroed). | 2A:L300-L303, L322-L323 |
| P2A-E029 | E/TEST | 2A.4 acceptance: cargo test -p reasoning 0 failures ≥12; no todo!/unimplemented!/unwrap() in engine.rs. | 2A:L309-L327 |
| P2A-E030 | E/OBS | 2A.5 metrics: validation_overhead_us <5000; memory per topic <1KB base + history; write `.elai_cc/metrics/phase_2A.toml`; median of 1000 iterations; no criterion without approval. | 2A:L329-L354 |
| P2A-E031 | E/TEST | Exit criteria: grounding enforced; budget overflow → converge or BudgetExhausted; InvalidTransition; MalformedGrounding; free Status; independent topics; scope WARN when required; 12+ tests. | 2A:L357-L365 |
| P2A-E032 | E/EVIDENCE | Budget range grounding: max_calls 1-12, max_chars 200-1500 from reasoning.txt §3; defaults are moderate 5/800. | 2A:L369-L370, L93-L93 |
| P2A-E033 | E/SURFACE | No dashboard panel in 2A; operator/engine surface is GrcEngine API and typed errors; Status free-echo aligns with mcp_think(step_type="status"). | 2A:L207-L209, L373-L373 |
| P2A-I001 | I/INT | AgentGrcConfig and orchestrator scope validation are named but config crate/module location is outside 2A skeleton (types only reference require_scope_enumeration). | 2A:L81-L85, L222-L237 |
| P2A-A001 | A/STATE | Exact allowed_next() table for every state (beyond Observe→not Verify and Verify terminal note) is not fully enumerated in the phase file. | 2A:L72-L76, L136-L138, L317-L319 |
| P2A-A002 | A/TEST | T11 says "Err or ThinkResult with warning" for missing scope — hard fail vs soft warn unresolved. | 2A:L229-L232 |
| P2A-A003 | A/FAIL | InvalidTransition "from=None indication" for first non-Observe step lacks a dedicated error variant shape. | 2A:L217-L220 |
| P2A-C001 | C/DATA | Header says "Grounding required on every step" in grounding summary item while body requires grounding only on Observe/Hypothesize/Verify (PlanAction/Act optional, Status free). | 2A:L273-L285, L358-L362, L371-L371 |

Category coverage: capabilities ✓; surfaces (API/errors, no UI) ✓; workflows/states ✓; schemas/types ✓; defaults/config ✓; integrations (trait stubs for 2B) ✓; failures/typed errors ✓; determinism (no timing tests) ✓; observability/metrics ✓; tests/TDD ✓; constraints/deferred ✓. Concurrency/retry/idempotency not specified. Privacy/SEC N/A.

### 3F — rules engine (self-improving, three-tier enforcement)

Source: `docs/specs/guide/steps/phase-3F.txt` (448/448 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3F-E001 | E/STATUS | Phase 3F: RULES ENGINE. Prerequisites: Phase 3D complete (parallel with 3E); implicit Phase 1A tree-sitter for AST_PATTERN. | 3F:L1-L3 |
| P3F-E002 | E/CAP | Goal: deterministic rule enforcement replacing probabilistic prompts; Predicate DSL with 5 types; self-discovery from auditor violations; rules DATA in SQLite evaluated by Rust; three-tier enforcement (Tier1 code, Tier2 constrained decoding, Tier3 prompt+auditor). | 3F:L7-L12 |
| P3F-E003 | E/DEP | Pre-reads: rules.txt full; prompt-architecture §§1-2,4.3,5,6; risk-register R25. | 3F:L14-L22 |
| P3F-E004 | E/SEC | R25 LLM ignores domain rules — this phase is the mitigation. | 3F:L24-L25 |
| P3F-E005 | E/FAIL | Malformed self-discovered predicates mitigated by compile-time validation before DB insert (3F.7). | 3F:L26-L27, L271-L272 |
| P3F-E006 | E/INT | tree-sitter: rules/ must NOT add separate dep; knowledge exposes AstEvaluator facade; recommended trait Box<dyn AstEvaluator> injected at construction (option b preserves dep graph). | 3F:L28-L42 |
| P3F-E007 | E/FLOW | Evaluation order cheapest-first short-circuit on BLOCK: METRIC_GATE → GREP_PATTERN → AST_PATTERN → COMPOSITE; within type severity DESC (BLOCK before WARN). | 3F:L44-L55, L133-L134 |
| P3F-E008 | E/FLOW | WARN collection: WARNs before a BLOCK preserved with BLOCK as context. | 3F:L52-L53 |
| P3F-E009 | E/DEP | Dev deps: proptest, criterion; runtime jsonschema; tree-sitter via knowledge AstEvaluator. | 3F:L57-L61 |
| P3F-E010 | E/DATA | 3F.1 tables: rules, rule_metrics, contract_tests, violation_patterns; schema_version migration; columns per rules.txt §2 including phase_mask, severity, mutable, source; scoring score_clarity/frequency/precision; counterfactual causal_catches/spurious_catches. | 3F:L66-L93 |
| P3F-E011 | E/TEST | CHECK constraints reject invalid tier/status/source; NULL predicate INSERT fails; migration idempotent preserves v1 data. | 3F:L81-L93 |
| P3F-E012 | E/CAP | 3F.2 five predicate types: AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE. | 3F:L95-L100, L119-L119 |
| P3F-E013 | E/STATE | Verdict: ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget); EscalationTarget: Auditor, Human, NextTierRule(RuleId). | 3F:L101-L102, L120-L121 |
| P3F-E014 | E/DATA | Predicate serde to/from TEXT (JSON or postcard); round-trip for all types. | 3F:L103-L103, L109-L109 |
| P3F-E015 | E/CAP | AstEvaluator: evaluate_query(language, source, query)→Vec<AstMatch>; validate_query→Result. | 3F:L104-L106 |
| P3F-E016 | E/FAIL | COMPOSITE circular graph returns error within 1ms; serde version drift explicit error or defaults. | 3F:L112-L117 |
| P3F-E017 | E/CAP | 3F.3 RuleEngine trait: evaluate(action, context)→ALLOW\|BLOCK\|WARN; get_active_rules(fsm_state, file_context); compile_predicate at insertion (regex, tree-sitter, jsonschema). Object-safe dyn. | 3F:L123-L148 |
| P3F-E018 | E/STATE | 3F.4 lifecycle: PROPOSED → CANDIDATE → SHADOW → ACTIVE → RETIRED. | 3F:L150-L155, L189-L189 |
| P3F-E019 | E/FLOW | Guards: PROPOSED→CANDIDATE ≥3 contract tests all pass; CANDIDATE→SHADOW human OR independent second-auditor (finding-source auditor cannot approve); SHADOW→ACTIVE FPR<5% AND TPR>80% AND counterfactual_causal_rate>50% AND no safety regressions; ACTIVE→RETIRED FPR>30% over 10 pipelines OR superseded OR human. | 3F:L155-L162 |
| P3F-E020 | E/SEC | mutable=0 rejects auto-retirement; mutable=0 exempt from causal rate gate (human-promotable only). | 3F:L163-L164, L177-L177, L191-L191 |
| P3F-E021 | E/OBS | Every transition logged with timestamp, metrics snapshot, reason. | 3F:L165-L165, L178-L178 |
| P3F-E022 | E/FAIL | Division-by-zero safe rates: 0 shadow data → "insufficient shadow data" error not panic; RETIRED kept in DB not deleted. | 3F:L179-L184, L192-L192 |
| P3F-E023 | E/SEC | PROPOSED source=AUDITOR cannot auto-promote without human/second-auditor. | 3F:L175-L187, L276-L277 |
| P3F-E024 | E/CAP | 3F.5 interpreter: AST via AstEvaluator; GREP compile-once regex; SCHEMA_REQUIRED jsonschema; METRIC_GATE lt/gt/eq/between; COMPOSITE AND/OR/NOT max depth 8. | 3F:L194-L229 |
| P3F-E025 | E/DATA | Evaluation returns Verdict { rule_id, result, evidence: String }. | 3F:L205-L205 |
| P3F-E026 | E/FLOW | ESCALATE routing: Auditor for SHADOW rules; Human for mutable=0 promotion; NextTierRule for tier chain. | 3F:L206-L207, L215-L215 |
| P3F-E027 | E/DET | Same code + same predicate = same verdict over 100 repetitions. | 3F:L216-L216, L229-L229, L408-L408 |
| P3F-E028 | E/FAIL | COMPOSITE depth 100 → Err(DepthExceeded); ReDoS patterns complete within timeout (regex linear-time). | 3F:L217-L222 |
| P3F-E029 | E/CFG | 3F.6 phase_mask bitmask + domain detection; get_active_rules status=ACTIVE AND phase_mask AND domain; SQL includes OR domain IN ('engineering','security'); elai_cc.toml [rules.domain_mapping]. | 3F:L231-L242 |
| P3F-E030 | E/CFG | Engineering and security domains always active (cross-cutting, security mutable=0). | 3F:L239-L242, L247-L248, L257-L258 |
| P3F-E031 | E/TEST | Domain detection covers 5 domains + engineering + security; case-insensitive path matching; phase_mask=0 never active. | 3F:L243-L258 |
| P3F-E032 | E/CAP | 3F.7 self-discovery on orchestrator side: normalize, blake2s hash, count, auto-propose at occurrences≥3; generate AST then regex then Tier3-only; compile_predicate before insert; invalid logged discarded. | 3F:L260-L298 |
| P3F-E033 | E/TEST | Auto-contract tests: each example_code → BLOCK test; orchestrator generates ALLOW counterexamples (not auditor). | 3F:L273-L275, L285-L285 |
| P3F-E034 | E/OBS | Violation scoring 3-axis composite clarity/frequency/precision in [0,15]. | 3F:L278-L278, L298-L298 |
| P3F-E035 | E/SEC | Normalization runs in orchestrator process, not auditor (circular trust). | 3F:L265-L266, L292-L293 |
| P3F-E036 | E/DATA | 3F.8 seed: rules_seed.db ~33 highest-value rules; status=ACTIVE source=SEED skip lifecycle; merge global+project rule_id override; mutable=0 seeds cannot be overridden by mutable=1 project rules. | 3F:L300-L337 |
| P3F-E037 | E/SURFACE | Post-tool completeness GREP rules (Tier1, engineering, PostToolUse): web_fetch <img> → WARN "N images not extracted"; <table> → WARN "N tables not extracted"; file_read TODO/FIXME → INFO "N TODO/FIXME items"; <1ms, no pre-exec I/O. | 3F:L309-L314 |
| P3F-E038 | E/TEST | Every seed Tier1 rule ≥3 contract tests; Tier2 ≥2; absent seed DB explicit error with path. | 3F:L322-L328 |
| P3F-E039 | E/TEST | 3F.9 ≥254 static contract tests; proptest 256 cases/rule; generators violation/valid/mutation_invariant/fix_mutation; COMPOSITE prop_recursive depth 8 size 64; De Morgan + associativity; shrunk counterexamples → contract_tests source=GENERATED; ~1400 proptest lines / 7 domains. | 3F:L339-L371 |
| P3F-E040 | E/SEC | contract_tests READ-LOCK during self-modification (INSERT rejected when flag true). | 3F:L364-L371 |
| P3F-E041 | E/OBS | 3F.10 criterion benchmarks: GREP median <1ms; METRIC <0.1ms; AST <5ms target; 10 mixed <10ms; store `.elai_cc/metrics/phase_3F.toml`. | 3F:L373-L397 |
| P3F-E042 | E/DEFER | Counterfactual shadow RUNTIME integration deferred to orchestrator+auditor (3C+3A) and 3I DST model; 3F only models rates/guards. | 3F:L399-L405 |
| P3F-E043 | E/TEST | Exit: determinism 100x; lifecycle all guards; self-discovery ≥3; ≥254+proptest; GREP/METRIC latency; auditor trust boundary. | 3F:L407-L414 |
| P3F-E044 | E/EVIDENCE | 254 arithmetic verified from rules.txt §9A; COMPOSITE depth=8 UNGROUNDED defensive (not in rules.txt); policy_engine.rs/permission_enforcer.rs LOC verified present. | 3F:L422-L448 |
| P3F-E045 | E/SURFACE | No dedicated rules dashboard panel in 3F phase file; enforcement surfaces are evaluate verdicts, seed DB, PostToolUse WARN/INFO strings, and transition logs. | 3F:L309-L314, L165-L165 |
| P3F-I001 | I/INT | SCHEMA_REQUIRED may interact with provider-tier escalation only if rules.txt says so; phase-3F body defines schema validation, not provider-tier escalation. | 3F:L99-L100, L202-L202 |
| P3F-A001 | A/DATA | Predicate TEXT encoding choice JSON vs postcard left open. | 3F:L103-L103 |
| P3F-A002 | A/CFG | Exact five business domains for detection listed only as "all 5 domains + engineering + security" without full enum in this file. | 3F:L256-L258 |
| P3F-A003 | A/FLOW | Verdict trait evaluate signature shows ALLOW\|BLOCK\|WARN while enum also has ESCALATE — escalation path vs evaluate return shape underspecified. | 3F:L128-L129, L101-L102, L206-L207 |
| P3F-C001 | C/DEP | Crate map says rules depends on core only, while architecture decision requires knowledge AstEvaluator or knowledge dep — resolved by recommended trait injection but dual wording remains. | 3F:L31-L41 |

Category coverage: capabilities/predicates ✓; surfaces (PostToolUse messages, errors) ✓; lifecycle states ✓; schema/data ✓; config/domain mapping ✓; integrations (knowledge AstEvaluator, orchestrator discovery) ✓; security/trust ✓; failures/depth/ReDoS ✓; determinism ✓; observability/metrics ✓; tests/proptest ✓; deferred counterfactual runtime ✓. Privacy formal policy N/A; retry only via compile discard.

### 3J — vLLM + advanced providers + plugins + orchestrator hardening

Source: `docs/specs/guide/steps/phase-3J.txt` (1573/1573 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3J-E001 | E/STATUS | Phase 3J: vLLM integration + advanced providers + plugins + orchestrator hardening. Prerequisites: Phase 3I complete; Phase 1F speculative eval. | 3J:L1-L2 |
| P3J-E002 | E/CAP | Goal: vLLM production backend; layer opts DoLa/TurboQuant/steering; ModelPool GPU multiplex/preload/profiles; MCP bridge; tiered static hooks; AppContainer; advanced SSB types; MILP eval. | 3J:L6-L11 |
| P3J-E003 | E/CFG | Primary test model Gemma4 E2B on RTX 3060 12GB; final 3J.14/15 apply to Gemma4 31B via CPU offload on 64GB RAM (~0.5-2 tok/s); EAGLE-3 draft training not feasible on CPU. | 3J:L13-L30, L99-L101 |
| P3J-E004 | E/EVIDENCE | Installed 2026-04-08: gemma4:31b 19GB Q4 262144 ctx vision+tools+thinking; gemma4:e2b 7.2GB; gemma3:4b fallback; no separate VLM needed. | 3J:L32-L41 |
| P3J-E005 | E/CFG | vLLM 0.19.0 in WSL2 ~/vllm-env/ PyTorch 2.10+cu128; serve example port 8000; Rust on Windows connects via HTTP localhost auto-forward. | 3J:L42-L51 |
| P3J-E006 | E/DEP | Pre-reads: providers.txt vLLM/LayerConfig/ModelPool; orchestrator SSB/AppContainer/self-mod; plugins MCP+hooks; paper summaries DoLa/TurboQuant/EAGLE-3/TIDE/ActAdd/Murakkab. | 3J:L54-L64 |
| P3J-E007 | E/DEP | Phase is prerequisite for 4A isolation and 4G autonomous mode (placement/optimization must stabilize first). | 3J:L67-L69 |
| P3J-E008 | E/FAIL | Risks R1-R11: HTTP latency, DoLa layer access, TurboQuant deep integration, ActAdd coupled, MILP Windows build, MCP 1751 LOC, AppContainer weaker-than-Linux, VRAM overcommit, small-model transfer, 31B CPU speed, CPU hook availability. | 3J:L72-L105 |
| P3J-E009 | E/CAP | 3J.1 VllmProvider: /v1/completions, /v1/models, SSE (reuse sse.rs), health GET /health, LlmProvider complete/embed/token_count; cfg feature "vllm"; latency 100 req p50/p95/p99 in vllm_spike.toml; timeout 5s on dead server. | 3J:L110-L141 |
| P3J-E010 | E/CAP | 3J.2 InferenceBackend::{Ollama(OllamaConfig), Vllm(VllmConfig)}; VllmConfig server_url, tensor_parallelism, served_model_names; ModelPool does not load/unload under Vllm (vLLM scheduler); POST/DELETE load/unload model APIs. | 3J:L143-L174 |
| P3J-E011 | E/CFG | elai_cc.toml [providers.backend] = "ollama"\|"vllm"; fallback to Ollama emits WARNING including "layer-level optimizations unavailable". | 3J:L158-L169 |
| P3J-E012 | E/CAP | 3J.3 DolaDecoder: JSD premature layer, alpha=0.1, theta=1.2 repetition; model-specific candidate buckets; DolaConfig enabled/alpha/buckets/repetition_penalty. | 3J:L176-L199 |
| P3J-E013 | E/CFG | Per-role DoLa: Executor on, Auditor amplified, Prompt-engineer off; elai_cc.toml [agents.{role}.layer_config.dola]. | 3J:L200-L204, L234-L234 |
| P3J-E014 | E/LIMIT | Guard: model_params <1B auto-disable DoLa with WARNING; 1-7B EVALUATE empirically; degraded top-N logprobs path if full logits unavailable. | 3J:L211-L219, L226-L227 |
| P3J-E015 | E/TEST | E2B: 20 factuality prompts baseline vs DoLa; latency budget ~5-8% (accept within 10%). | 3J:L204-L209, L236-L237 |
| P3J-E016 | E/DEFER | 3J.4 TurboQuant FEASIBLE/DEFER decision; QR rotation + Lloyd-Max; b-bit codebooks; d>=64; unit-norm + orthogonal Pi invariants; decision file phase-3J-turboquant.md. | 3J:L239-L291 |
| P3J-E017 | E/CAP | 3J.5 SteeringVectorEngine ActAdd: h_A^l = h+^l − h-^l; SteeringVector {layer, vector, coefficient}; config vectors/enabled; elai_cc.toml [agents.{role}.layer_config.steering]; mid-layer default model_layers/3; c in [3,20] start 5. | 3J:L293-L350 |
| P3J-E018 | E/DEP | Steering coupled to DoLa layer hook: if DoLa deferred, steering deferred. Local models only (no API models). Composable multi-vector. | 3J:L323-L329, L350-L350 |
| P3J-E019 | E/DATA | Store vectors .safetensors or .npz; E2B ≥2 contrast pairs skepticism+completeness; 5 prompts/vector eval. | 3J:L330-L334 |
| P3J-E020 | E/CAP | 3J.6 MultiplexingPolicy: safety_margin_bytes default 1GB; max_colocated 3; min_model_size_for_dedicated_gpu 4GB; ModelPool.can_colocate; Phase 0-1 single GPU no multiplex. | 3J:L352-L369, L401-L404 |
| P3J-E021 | E/FLOW | Predictive preload on FSM entry: EXECUTING→auditor; AUDITING→executor; PLANNING→prompt-engineer; TESTING_FIRST→executor; tokio::spawn background; blocked if VRAM insufficient. | 3J:L370-L378 |
| P3J-E022 | E/CFG | PreloadPolicy map FsmState→Vec<ModelId> in [providers.preload]; SloTier BEST/GOOD/FAIR/BASIC per [agents.{role}.slo_tier]; tier mismatch WARNING. | 3J:L377-L386 |
| P3J-E023 | E/EVENT | 3J.7 WorkloadProfile SSB action_type='workload_profile' at phase_complete: role, phase_id, tokens_in/out, latency_ms, vram_peak_mb, model_id, gpu_id, timestamp ClockProvider; budget <1ms. | 3J:L406-L442 |
| P3J-E024 | E/CAP | 3J.8 MILP eval vs greedy: good_lp+HiGHS preferred; ADOPT if >10% swap latency improvement; DROP if heuristic <5% worse; EVALUATE if 5-10%; always implement GreedyModelPlacer; decision phase-3J-milp.md; solve budget <50ms; infeasible→greedy WARNING. | 3J:L444-L487 |
| P3J-E025 | E/CAP | 3J.9 McpBridge: connect/list_tools/call_tool/disconnect; transport stdio/SSE/streamable HTTP; feature mcp; port claw-code 912+839 LOC. | 3J:L489-L533 |
| P3J-E026 | E/RECOVERY | MCP retry 3 attempts 1s/2s/4s; heartbeat timeout 30s; tool call timeout 30s configurable; graceful degradation explicit errors to agent. | 3J:L504-L509, L525-L526 |
| P3J-E027 | E/SEC | MCP untrusted: sanitize results; treat as data not instructions; version handshake reject incompatible; agent never connects to MCP directly (orchestrator proxy). | 3J:L509-L528 |
| P3J-E028 | E/CAP | 3J.10 T1 RULES <1ms every SSB write; T2 AST <10ms on file_write only, circuit break files>5000 lines; T3 PATTERN <5ms only when WARN and no BLOCK; no LLM hot path. | 3J:L535-L567 |
| P3J-E029 | E/SEC | BLOCK/WARN → SSB finding; orchestrator blake2s normalizes for violation_patterns; agent sees sanitized "Tool call rejected by policy" only (no rule ID/tier/pattern). | 3J:L556-L563, L582-L584 |
| P3J-E030 | E/SEC | 3J.11 AppContainerConfig profile elai_cc_agent_{id}; granted_paths; network_allowed false except researcher; Job limits ACTIVE_PROCESS=1, PROCESS_MEMORY=512MB, JOB_TIME; env only LANG/TERM/PATH/TMPDIR. | 3J:L586-L614 |
| P3J-E031 | E/SEC | Self-mod locks: IPC realpath Write/Edit reject; FS not granted; GitCommit filter on locked paths; fallback restricted token documented weaker (junctions). | 3J:L605-L611, L624-L625 |
| P3J-E032 | E/CAP | AgentSandbox trait create/launch_agent/cleanup; Windows AppContainer cfg; Linux clone+seccomp Phase 4A. | 3J:L615-L617 |
| P3J-E033 | E/EVENT | 3J.12 adds typed SSB actions: model_swap, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, memory_monitoring, phase_complete, workload_profile; total variants ≥17; write-time type validation. | 3J:L637-L681 |
| P3J-E034 | E/OBS | isolation_metric: context_savings_from_isolation = sum(agent_grc_tokens)/model.max_context at phase transition. | 3J:L663-L665 |
| P3J-E035 | E/CAP | 3J.PIPELINE InferenceDispatcher trait; ApiDispatcher + LocalDispatcher; [backend] mode local\|api\|hybrid\|auto; role_override map. | 3J:L683-L712 |
| P3J-E036 | E/FLOW | FSM tool filtering: PLANNING read-only; EXECUTING full; AUDITING read-only+test; local max 15 tools truncate descriptions; API all tools. | 3J:L714-L721 |
| P3J-E037 | E/CFG | ContextBudget: local 80% of 131K; API 200K; slots system/tools/messages. | 3J:L723-L730 |
| P3J-E038 | E/SURFACE | Slash /model and /mode switch backend at runtime; elai --local uses orchestrator pipeline; default elai uses API same pipeline. | 3J:L737-L737, L765-L767 |
| P3J-E039 | E/FLOW | Auto-start sidecar when mode local/auto; health wait max 120s; kill sidecar on CLI exit; auto→API if start fails. | 3J:L739-L744 |
| P3J-E040 | E/CAP | 3J.UNIFIED single RuntimeClient: auth/backend/tool_policy/budget_policy; one stream(); max 3 backend branch points; BackendConfig max 10 fields. | 3J:L769-L848 |
| P3J-E041 | E/CFG | ToolPolicy Full \| FsmFiltered {max_tools, fsm_state_path} \| None(Idle); BudgetPolicy ApiDefault{ceiling:200K} \| Local{ceiling:131K, fraction:80%}. | 3J:L800-L807 |
| P3J-E042 | E/SURFACE | --local flag sets BackendConfig mode Local; no flag → Api from env auth; elai.bat simplified to --local + HOME fallback. | 3J:L814-L815, L862-L862 |
| P3J-E043 | E/STATUS | 3J.PIPELINE session summary 2026-04-09: dispatcher.rs, Fsm::read_persisted_state, LocalRuntimeClient, start_sidecar windows_to_wsl_path, /shutdown, real tokenizer counting, 7/7 tests, patches 0.3/0.4. | 3J:L850-L875 |
| P3J-E044 | E/CAP | 3J.WIRE.FOUNDATION: LiveCli owns Ssb, Arc<Fsm>, ClockProvider; paths sessions/{sid}/ssb.db and .fsm-state.msgpack. | 3J:L877-L907 |
| P3J-E045 | E/SURFACE | CliInteractiveGate: present_gate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances. | 3J:L895-L898 |
| P3J-E046 | E/EVENT | SessionTracer backed by Ssb: turn_started, tool_execution, turn_completed → SsbPayload; Ssb::shutdown on exit. | 3J:L902-L907 |
| P3J-E047 | E/CFG | 3J.WIRE.BUDGET: real fixed_slot_sizes for slots 0/1/2; max_tokens = min(model max, remaining); ApiDefault skips ContextBudget; [pipeline] telemetry logs slots. | 3J:L925-L963 |
| P3J-E048 | E/CAP | 3J.WIRE.RULES: load seed rules (~30-40 eng+security); DefaultRuleEngine on PreToolUse; BLOCK reject; WARN→Ssb; ESCALATE→InteractiveGate; AstEvaluator/DkgQuerier stubs initially; strip CLAUDE.md rules from local system prompt only. | 3J:L966-L1015 |
| P3J-E049 | E/OBS | Rule eval <5ms per tool call Tier1 target; seed rules versioned under .elai_cc/data/rules/ TOML. | 3J:L1004-L1007 |
| P3J-E050 | E/CAP | 3J.WIRE.SESSION: SessionDag sessions/{sid}/dag.db; auto-snapshot each turn; trim at 85% budget 3-pass; /resume via load_snapshot+parent_chain; /session fork create_branch max 3; recover_stubs from Ssb. | 3J:L1017-L1060 |
| P3J-E051 | E/LIMIT | SessionDag sync SQLite not WAL; separate from Ssb WAL connection; keep /compact as manual override. | 3J:L1045-L1052 |
| P3J-E052 | E/SURFACE | 3J.WIRE.PLAN: optional PlanSkeleton from .elai_cc/plans/{pipeline_id}.toml; /plan displays skeleton; /plan phase <id> shows PhaseDetail; Slot 2 phase context; contract satisfied vs pending display. | 3J:L1063-L1103 |
| P3J-E053 | E/LIMIT | Plan load optional (None ok); do not auto-create plans. | 3J:L1089-L1096 |
| P3J-E054 | E/CAP | 3J.WIRE.STUBS: TreeSitterAstEvaluator + DkgQuerierAdapter replace PassAstEvaluator/EmptyDkgQuerier; max_depth<=20; visited set on BFS; lazy DKG init; fallback EmptyDkgQuerier on init fail with warning. | 3J:L1106-L1201 |
| P3J-E055 | E/DEFER | Stub inventory defers OllamaProvider/Embedder todos, NetworkVerifier, NoopTestRunner, OrtSession.embed, VectorIndex readiness to other phases. | 3J:L1120-L1127 |
| P3J-E056 | E/OBS | 3J.13 metrics phase_3J.toml fields: latencies, dola_*, turboquant_*, milp_*, steering_*, multiplexing_enabled, ssb_action_type_count, t1/t2/t3 hook medians, compile_time_delta_sec; non-vllm build still passes. | 3J:L1204-L1244 |
| P3J-E057 | E/SURFACE | 3J.14 WIRE.E2E: elai.bat single command; primary sidecar DoLa+TurboQuant+NF4; ollama fallback; QueryPipeline retrieval; release elai.exe. | 3J:L1246-L1253 |
| P3J-E058 | E/FLOW | Ollama fallback on sidecar health fail or mid-session HTTP error; re-probe each turn; both down → clear error no crash; user sees "[degraded] using ollama fallback" prompt prefix. | 3J:L1263-L1277 |
| P3J-E059 | E/CFG | elai_cc.toml [backend] fallback_url default http://localhost:11434. | 3J:L1277-L1277 |
| P3J-E060 | E/CAP | Knowledge retrieval: BM25 index cwd; QueryPipeline BM25+CatRag+Arag; optional CrossEncoder; inject top-K into Slot 4 CodeContext; update index on file_write/edit; DKG PageRank if dkg.db exists. | 3J:L1279-L1297 |
| P3J-E061 | E/FLOW | Startup sequence: probe/start sidecar or ollama; BM25; QueryPipeline; Ssb/Fsm/SessionDag/RuleEngine/ContextBudget/PlanSkeleton; banner backend+index+session; REPL. | 3J:L1307-L1314 |
| P3J-E062 | E/TEST | E2E accept: REPL ≤60s excluding model load; both backends complete; BM25 context in prompt; non-zero CodeContext slot; release binary path. | 3J:L1341-L1348 |
| P3J-E063 | E/CAP | 3J.15 31B CPU offload: ≥2 steering vectors stored; DoLa 20 prompts buckets [0,20),[20,40),[40,60); TurboQuant codebook validate; EAGLE-3 architecture only, training deferred 48GB; metrics phase_3J_31b.toml. | 3J:L1350-L1428 |
| P3J-E064 | E/DEFER | 100B Q3 validation gated on 48GB GPU; SUBSTITUTE_MODEL escalation path test; metrics phase_3J_100b.toml if executed. | 3J:L1430-L1441 |
| P3J-E065 | E/TEST | Exit criteria list: VllmProvider+ModelPool; DoLa/steering E2B+31B; TurboQuant decision; multiplexing+preload; workload profiles; MILP decision; MCP; hooks SLOs; AppContainer; 17 SSB types; zero 0-3I regressions; EAGLE-3 arch deferred. | 3J:L1446-L1462 |
| P3J-E066 | E/STATUS | 3J.ADDENDUM shipped post-baseline: A-9 ETCLogitsProcessor; A-17/A-25 GG mean_logprob+entropy_mean; A-10 ConstrainedDecodingConfig grammar/JSON; A-26 routing_rules; A-27 bash safety_filter; A-28 git_commit; A-31 action_classifier; A-8 steering batch mode. | 3J:L1517-L1569 |
| P3J-E067 | E/SURFACE | Addendum surfaces: /research JSON-mode uses constrained decoding; COMMITTING FSM phase invokes git_commit tool; calibrate_etc_threshold.py; data/steering_contrast_pairs.toml. | 3J:L1525-L1569 |
| P3J-E068 | E/SURFACE | User-facing CLI verbs/flags in 3J body: elai / elai --local; elai.bat; /mode local\|api\|hybrid; /model; /plan; /plan phase <id>; /resume; /session fork; /compact; /shutdown (sidecar); REPL banner; degraded prefix; InteractiveGate Approve/Reject/Modify/Redirect. | 3J:L737-L767, L814-L831, L895-L898, L1027-L1052, L1078-L1079, L1268-L1273, L1307-L1314 |
| P3J-I001 | I/DEP | 3J.COMPOSE and 3J.LOCAL are PRE_GATE prerequisites for PIPELINE/E2E but are not defined as step headings in this file (likely baseline/other chunk). | 3J:L685-L686, L1257-L1257 |
| P3J-I002 | I/CFG | Summary "can_fit_onnx()" for multiplexing does not appear in 3J.6; source API is can_colocate (and can_fit may live elsewhere). | 3J:L368-L369 |
| P3J-A001 | A/CAP | Exact vLLM plugin/hook API for full DoLa logits remains VERIFY; degraded path always required until proven. | 3J:L76-L79, L210-L214, L1491-L1493 |
| P3J-A002 | A/CFG | Contrast prompt pairs for Auditor skepticism and Context-compiler completeness marked TBD at implementation. | 3J:L308-L313 |
| P3J-A003 | A/FLOW | hybrid/auto role_override vs later UNIFIED removal of Hybrid/Auto dead variants (session summary) leaves production mode enum history ambiguous. | 3J:L705-L712, L870-L870 |
| P3J-A004 | A/TEST | 3J.14 "NF4" in sidecar primary stack is named without algorithm detail in this file. | 3J:L1249-L1249 |
| P3J-C001 | C/STATUS | Step numbers in body (3J.6 multiplex+preload, 3J.7 profiles, 3J.8 MILP, 3J.9 MCP…) differ from MASTER summary bundling (3J.6 can_fit_onnx; 3J.7-8 preload+profiles; 3J.9 MILP; 3J.10-13 MCP/hooks/AppContainer/SSB). | 3J:L352-L681 vs MASTER:L370-L373 |
| P3J-C002 | C/STATUS | Opening says final step 3J.14 applies 31B; body renumbers 31B validation to 3J.15 and 3J.14 becomes WIRE.E2E. | 3J:L15-L16, L1246-L1350 |

Category coverage: capabilities ✓; user surfaces (CLI/flags/slash/gate/banner/degraded) ✓; workflows/FSM/preload ✓; schemas/SSB/types ✓; config flags ✓; integrations vLLM/MCP/sidecar/WSL ✓; security AppContainer/MCP/self-mod ✓; failures/fallback/retry ✓; determinism feature-gates ✓; observability metrics ✓; tests/exit ✓; deferred TurboQuant/MILP/EAGLE-3/100B ✓. Formal privacy policy N/A beyond env sanitization.

### Cross-source interactions (PHASE-005)

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| PHASE-005/IX-001 | E | 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite. | P1F-E004, P1F-E015, P1F-E016; P3J-E001, P3J-E009 |
| PHASE-005/IX-002 | E | 1F requires 1C ModelPool VRAM accounting for draft overhead measurement. | P1F-E007, P1F-E010 |
| PHASE-005/IX-003 | E | 2A freezes GrcEngine six-method trait so 2B can implement verify_claim/ground_plan without breaking BasicGrcEngine. | P2A-E016, P2A-E017 |
| PHASE-005/IX-004 | E | 2A scope enumeration WARNs feed agent context; orchestrator (outside 2A) enforces plan_action coverage. | P2A-E013 |
| PHASE-005/IX-005 | E | 3F depends on 3D complete and 1A tree-sitter via AstEvaluator; self-discovery needs 3C agent/auditor IPC. | P3F-E001, P3F-E006, P3F-E032 |
| PHASE-005/IX-006 | E | 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM). | P3F-E018, P3F-E019 |
| PHASE-005/IX-007 | E | 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7). | P3F-E032, P3F-E035; P3J-E028, P3J-E029 |
| PHASE-005/IX-008 | E | 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters. | P3J-E048, P3J-E054; P3F-E036 |
| PHASE-005/IX-009 | E | 3J workload_profile + advanced SSB types extend 3B core actions to ≥17 including phase_complete atomic boundary used later by 4A. | P3J-E023, P3J-E033 |
| PHASE-005/IX-010 | E | 3J.WIRE.SESSION composes 3H SessionDag trim 85% + stub recovery from SSB. | P3J-E050, P3J-E051 |
| PHASE-005/IX-011 | E | 3J.WIRE.PLAN composes 3G PlanSkeleton into CLI Slot 2 and /plan surfaces. | P3J-E052 |
| PHASE-005/IX-012 | E | 3J.WIRE.FOUNDATION InteractiveGate is the 3A/4G-shared human gate surface (Approve/Reject/Modify/Redirect). | P3J-E045 |
| PHASE-005/IX-013 | E | 3J.14 retrieval injects 1B/1E/1G pipeline pieces (BM25, CatRAG, ARAG, optional cross-encoder) into prompt Slot 4. | P3J-E060 |
| PHASE-005/IX-014 | E | 3J.15 SUBSTITUTE_MODEL 100B path anticipates 4G escalation; anti-spiral 3× same error swap. | P3J-E064 |
| PHASE-005/IX-015 | E | Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception. | P3F-E032; P3J-E029 |
| PHASE-005/IX-016 | E | 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers. | P2A-E007, P2A-E027; P3J-E037, P3J-E047 |
| PHASE-005/IX-017 | I | 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals. | P1F-E021, P1F-E032; P3J-E002 |
| PHASE-005/IX-018 | E | 3J MCP tools join native tools under uniform ToolRequest/ToolResult after 3C agents exist. | P3J-E025, P3J-E027 |
| PHASE-005/IX-019 | E | AppContainer self-mod locks prefigure 4A BLOCK-8 realpath+FS+git filters. | P3J-E031 |
| PHASE-005/IX-020 | E | DoLa disabled for prompt-engineer aligns with constrained-decoding-only role (addendum A-10). | P3J-E013, P3J-E066 |

### Ambiguity and conflict register (PHASE-005)

| Register ID | Kind | Note | Evidence atoms |
|---|---|---|---|
| PHASE-005/AC-001 | conflict | 1F prerequisite Phase 0-independent vs 1C dependency graph. | P1F-C001 |
| PHASE-005/AC-002 | conflict | 1F full eval suite vs RESOLVED skip 1F.3-1F.5. | P1F-C002 |
| PHASE-005/AC-003 | conflict | 2A "grounding every step" summary line vs required subset. | P2A-C001 |
| PHASE-005/AC-004 | ambiguity | 2A T11 hard Err vs warning for missing scope. | P2A-A002 |
| PHASE-005/AC-005 | ambiguity | 2A full allowed_next table incomplete. | P2A-A001 |
| PHASE-005/AC-006 | conflict | 3F rules→core-only crate map vs AstEvaluator knowledge coupling. | P3F-C001 |
| PHASE-005/AC-007 | ambiguity | 3F evaluate return ALLOW\|BLOCK\|WARN vs ESCALATE enum arm. | P3F-A003 |
| PHASE-005/AC-008 | conflict | 3J step numbering vs MASTER summary bundles. | P3J-C001 |
| PHASE-005/AC-009 | conflict | 3J.14 31B in intro vs 3J.15 body renumber + E2E as 3J.14. | P3J-C002 |
| PHASE-005/AC-010 | ambiguity | DoLa full-logit access method still VERIFY. | P3J-A001 |
| PHASE-005/AC-011 | ambiguity | hybrid/auto BackendMode vs session removal of dead variants. | P3J-A003 |
| PHASE-005/AC-012 | dependency | 3J.COMPOSE/3J.LOCAL referenced but not defined in this file. | P3J-I001 |
| PHASE-005/AC-013 | qualification | MASTER can_fit_onnx for 3J.6 vs source can_colocate. | P3J-I002 |
| PHASE-005/AC-014 | ambiguity | SCHEMA_REQUIRED provider-tier escalation is MASTER-only, not 3F body. | P3F-I001 |
| PHASE-005/AC-015 | ambiguity | 3F.5 NaN/Inf metric guard appears in MASTER not phase-3F body. | MASTER:L314 vs P3F-E024 |

### Summary deltas vs MASTER (PHASE-005)

### 1F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-005/SD-1F-001 | **ADD** full RESOLVED timeline: ollama 0.20.2 NOT_AVAILABLE, PR #8134 never merged, consolidated phase_1F.toml decision=UNAVAILABLE, EXIT satisfied via DROP. | MASTER:L114-L117 | P1F-E002–P1F-E006, P1F-E015 |
| PHASE-005/SD-1F-002 | **ADD** complete decision gate order, boundary DROP at ==1.3x/3000MB, prompt-lookup preference, acceptance early kill, k-sweep, GPU temp rules, marginal retest. | MASTER:L120-L120 | P1F-E020, P1F-E032–P1F-E037, P1F-E041 |
| PHASE-005/SD-1F-003 | **ADD** hardware scenarios A/B remaining VRAM, nvidia-smi polling, Phase-0-hardware-only ADOPT caveat, TIDE conditional later opt. | MASTER:L121-L121 | P1F-E010-P1F-E011, P1F-E040, P1F-E043 |
| PHASE-005/SD-1F-004 | **ADD** adversarial serde/num_draft_tokens=0, silent-ignore false positive, cache uniqueness, and all metrics file paths. | MASTER:L118-L120 | P1F-E022-P1F-E031, P1F-E039 |
| PHASE-005/SD-1F-005 | **QUALIFY** "Phase 2+" revisit as post-Phase-1/vLLM (3J), not a Phase-2 numbered dependency. | MASTER:L117, L659 | P1F-I001, P1F-E004 |

### 2A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-005/SD-2A-001 | **ADD** design D1-D5 paper grounding (BudgetThinker, RARR/SAFE, DSPy Assert/Suggest) and check-then-increment rationale. | MASTER:L172-L186 | P2A-E005-P2A-E009, P2A-E027 |
| PHASE-005/SD-2A-002 | **ADD** full T1-T12 catalog notes including max_calls=3 trick, free Status, default 5/800, scope three-way tests. | MASTER:L182-L182 | P2A-E020-P2A-E023 |
| PHASE-005/SD-2A-003 | **ADD** valid grounding type list and level mapping including tool→High and search→Verified. | MASTER:L184-L184 | P2A-E025, P2A-E026 |
| PHASE-005/SD-2A-004 | **ADD** metrics methodology (1000-iter median, no criterion without approval) and reset_topic full removal. | MASTER:L186-L186 | P2A-E028, P2A-E030 |
| PHASE-005/SD-2A-005 | **FLAG** T11 Err-vs-warn, incomplete allowed_next table, and "every step grounding" wording conflict. | MASTER:L172-L186 | P2A-A001-P2A-A002, P2A-C001 |

### 3F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-005/SD-3F-001 | **ADD** WARN preservation with BLOCK, severity DESC within type, and evaluation-order rationale. | MASTER:L305-L306 | P3F-E007, P3F-E008 |
| PHASE-005/SD-3F-002 | **ADD** full transition thresholds, mutable=0 dual invariants, transition log fields, insufficient-shadow-data error. | MASTER:L311-L313 | P3F-E018-P3F-E023 |
| PHASE-005/SD-3F-003 | **ADD** PostToolUse seed GREP surfaces (img/table/TODO strings), merge override rules, seed contract-test minimums. | MASTER:L317-L317 | P3F-E036-P3F-E038 |
| PHASE-005/SD-3F-004 | **ADD** proptest De Morgan/associativity, READ-LOCK on contract_tests, COMPOSITE depth 8 UNGROUNDED note, counterfactual runtime deferral. | MASTER:L318-L319 | P3F-E039-P3F-E042, P3F-E044 |
| PHASE-005/SD-3F-005 | **CORRECT/QUALIFY** MASTER SCHEMA_REQUIRED "escalates provider tier" and NaN/Inf guard: not stated in phase-3F body (schema validate + metric ops only). | MASTER:L306, L314 | P3F-I001, AC-014, AC-015 |
| PHASE-005/SD-3F-006 | **ADD** AstEvaluator injection option (b) recommended, domain_mapping config, scoring [0,15], blake2s discovery. | MASTER:L303-L316 | P3F-E006, P3F-E029, P3F-E032, P3F-E034 |

### 3J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-005/SD-3J-001 | **CORRECT** step numbering: multiplex+preload are 3J.6; profiles 3J.7; MILP 3J.8; MCP 3J.9; hooks 3J.10; AppContainer 3J.11; SSB 3J.12; verify 3J.13; E2E 3J.14; 31B 3J.15. | MASTER:L370-L374 | P3J-C001, P3J-C002, P3J-E020-P3J-E063 |
| PHASE-005/SD-3J-002 | **CORRECT** 3J.6 API is can_colocate + MultiplexingPolicy fields, not can_fit_onnx in this file. | MASTER:L370 | P3J-E020, P3J-I002 |
| PHASE-005/SD-3J-003 | **ADD** complete user-facing surface set: --local, elai.bat, /mode,/model,/plan,/resume,/session fork,/compact, InteractiveGate four choices, degraded prefix, startup banner. | MASTER:L675-L681 | P3J-E038, P3J-E042, P3J-E045, P3J-E050, P3J-E052, P3J-E058, P3J-E061, P3J-E068 |
| PHASE-005/SD-3J-004 | **ADD** full WIRE.* chain FOUNDATION→BUDGET→RULES→SESSION→PLAN→STUBS and UNIFIED RuntimeClient constraints. | MASTER:L675-L681 | P3J-E040-P3J-E055 |
| PHASE-005/SD-3J-005 | **ADD** MCP security/retry, hook sanitization string, AppContainer job limits/env allowlist, isolation_metric formula, SSB ≥17 write-time validation. | MASTER:L373-L373 | P3J-E025-P3J-E034 |
| PHASE-005/SD-3J-006 | **ADD** 3J.14 Ollama fallback re-probe, QueryPipeline Slot 4, BM25 update-on-edit, 60s REPL SLO, release binary checks. | MASTER:L675-L681 | P3J-E057-P3J-E062 |
| PHASE-005/SD-3J-007 | **ADD** 3J.15 31B buckets, EAGLE-3 skip reason, optional 100B SUBSTITUTE_MODEL gate, metrics file names. | MASTER:L374 | P3J-E063, P3J-E064 |
| PHASE-005/SD-3J-008 | **ADD** entire 3J.ADDENDUM shipped extensions A-8/9/10/17/25/26/27/28/31 with file paths. | MASTER:L360-L375 (absent) | P3J-E066, P3J-E067 |
| PHASE-005/SD-3J-009 | **ADD** hardware/model install snapshot (vLLM 0.19.0 WSL2, gemma4 sizes, native vision) and R1-R11 risk register. | MASTER:L363-L364 | P3J-E003-P3J-E008 |
| PHASE-005/SD-3J-010 | **FLAG** undefined 3J.COMPOSE/LOCAL steps, TBD contrast pairs, NF4 detail, hybrid mode history. | MASTER:L675-L681 | P3J-I001, P3J-A002-P3J-A004, AC-011-AC-012 |

## PHASE-006 — source `DEEP-PHASE-006.discovery.md`

- Declared feature atoms: **238** (E=221 I=5 A=11 C=1).
- Parsed feature rows in this aggregate: **238**.
- Per-file: 1H **035**; 3I **036**; 3K **035**; 4A **054**; 4H **078**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-006.discovery.md`.

### Feature atoms

### 1H — conditional SPLADE keyword expansion

Source: `docs/specs/guide/steps/phase-1H.txt` (290/290 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1H-E001 | E/STATUS | Phase 1H is conditional SPLADE keyword expansion evaluation; prerequisites Phase 1G complete. | 1H:L1-L2 |
| P1H-E002 | E/CAP | Goal: evaluate SPLADE sparse keyword expansion behind cfg(feature="splade"); compare prithivida/Splade_PP_en_v1 vs naver/splade-v3; pick better on code corpus then keep/drop. | 1H:L6-L9 |
| P1H-E003 | E/FAIL | DROP if <5% recall improvement over BM25-only on test corpus. | 1H:L10-L10 |
| P1H-E004 | E/LIMIT | CRITICAL RISK: BERT WordPiece vocab (30522 tokens) mismatches code tokens (fn, impl, async, Vec<T>, unwrap()); phase MUST measure vocabulary coverage before concluding SPLADE helps on code. | 1H:L12-L17 |
| P1H-E005 | E/DEP | Pre-read knowledge.txt L3.25 SPLADE (L57-62), query pipeline (L88); architecture-overview §10 ort/tokenizers/model candidates; knowledge.txt §6 circuit breaker; §7 optional SPLADE load. | 1H:L19-L24 |
| P1H-E006 | E/LIMIT | knowledge.txt L58 typo "prithvida" and missing SPLADE-v3/quantized/vocab-risk; owned by s-702732; update when unlocked. Correct spelling is prithivida. | 1H:L24-L24, L34-L34 |
| P1H-E007 | E/FAIL | R24: SPLADE ONNX weight ~532MB vs marginal code-search gain. R25: NL benchmarks (29% nDCG@10 ESCI) may not transfer to code. R26: ort downloads prebuilt from Microsoft (air-gap risk). | 1H:L27-L29 |
| P1H-E008 | E/DEP | R27: ort does NOT include tokenization; needs separate tokenizers crate (may exist from 1G cross-encoder). R28: ort is unconditional for GraphCodeBERT/Granite; feature-gating SPLADE only gates SPLADE Session init, not ort. | 1H:L30-L31 |
| P1H-E009 | E/LIMIT | R29: programming tokens not in BERT vocab and split to char n-grams, reducing expansion quality. R30: 10-query eval underpowered; TREC standard ≥50 topics; effects <~15% undetectable with 10 queries. | 1H:L32-L33 |
| P1H-E010 | E/EVIDENCE | Grounding: PP_en_v1 ONNX 532MB main / 436MB quantized; ort pyke; RRF with BM25 GROUNDED knowledge.txt; 5% drop criterion GROUNDED; code-search recall UNGROUNDED; splade-v3 VERIFIED arxiv 2403.06789; 0.3 vocab threshold UNGROUNDED heuristic. | 1H:L37-L50 |
| P1H-E011 | E/CAP | Step 1: optional ort dep; create knowledge/splade.rs fully behind cfg(feature="splade"); stub SpladeExpander and expand_query returning empty Vec<(String,f32)>; re-export from mod.rs gated. | 1H:L55-L68 |
| P1H-E012 | E/TEST | Step 1 checks: cargo check no features compiles without SpladeExpander; cargo check --features splade includes module; non-gated code must not contain SpladeExpander. | 1H:L70-L87 |
| P1H-E013 | E/LIMIT | Do NOT test for ort absence; ort always present for other models. | 1H:L78-L80 |
| P1H-E014 | E/TEST | Step 2 TDD: feature-gated tests/knowledge/splade_tests.rs — expand_query synonym terms, empty query empty vec, long >10000 chars no panic, RRF differs from BM25-only, property finite weights >=0, integration recall BM25 vs BM25+SPLADE. | 1H:L90-L103 |
| P1H-E015 | E/TEST | Step 2 acceptance: ≥5 unit + 1 property + 1 integration; tests FAIL (red); at least one asserts non-empty expand; property corpus includes Unicode. | 1H:L105-L120 |
| P1H-E016 | E/CFG | Model path default .elai_cc/models/splade/model.onnx; override SPLADE_MODEL_PATH env. | 1H:L130-L131 |
| P1H-E017 | E/CAP | SpladeExpander::new loads ONNX via ort::Session and BERT tokenizer (vocab.txt + tokenizer.json). expand_query tokenizes to input_ids/attention_mask/token_type_ids, runs inference, returns top-k (token, weight). | 1H:L133-L136 |
| P1H-E018 | E/CAP | model_variant enum PrePooled vs RawMLM: PP_en_v1 may output pre-pooled; splade-v3 optimum export is RawMLM [1,seq,30522] requiring max-pool+ReLU+log1p in Rust. | 1H:L137-L139 |
| P1H-E019 | E/OBS | vocab_coverage(corpus_tokens)->f64 fraction of unique tokens mapping to single WordPiece; if coverage <0.3 SPLADE unlikely to help. | 1H:L140-L140 |
| P1H-E020 | E/CAP | rrf_fuse(bm25, splade, k=60, weight_bm25=0.5, weight_splade=0.5); ranks 1-based; k=max(k,1) prevents div-by-zero. | 1H:L141-L141 |
| P1H-E021 | E/RECOVERY | Circuit breaker on ONNX calls (knowledge.txt §6); SPLADE failure returns empty expansion so BM25-only continues. | 1H:L142-L143 |
| P1H-E022 | E/CFG | Both model candidates share same interface; only model_path differs: A PP_en_v1 532/436MB; B splade-v3 ONNX export. | 1H:L144-L147 |
| P1H-E023 | E/TEST | Step 3 adversarial: code-specific vocab_coverage on tree-sitter identifiers not all tokens; corrupted 1-byte model returns Err not panic; non-English no panic; expand_query p50 <2s/100-char excluding warmup; k=0 clamps; missing tokenizer files Err. | 1H:L159-L181 |
| P1H-E024 | E/FLOW | 4a early-exit: extract code identifiers via tree-sitter; if code-specific coverage <0.3 EARLY DROP without recall; log rationale; skip 4b-4d. Coverage 0.2-0.4 may still proceed to calibrate threshold. | 1H:L193-L201 |
| P1H-E025 | E/CFG | 4b models pre-staged outside session; quant Jaccard top-50 terms A-full vs A-quant; if overlap>=0.95 prefer quant (saves 96MB + ~3x). Jaccard only for same-model quant, NOT cross-model. | 1H:L203-L211 |
| P1H-E026 | E/FLOW | 4b cross-model: quick recall@10 on 10 of 50 queries; pick higher mean; if within 1% pick faster; log to workflow/reasoning/phase-1H-measurements.md. | 1H:L212-L216 |
| P1H-E027 | E/TEST | 4c corpus: 100 code files, 50 queries; ≥15 vocab-mismatch, ≥15 distractors, ≥10 straightforward keyword; Wilcoxon signed-rank on paired recall@10; exclude bm25_recall==0 from delta %. | 1H:L218-L230 |
| P1H-E028 | E/FLOW | 4d KEEP if mean delta>=5% AND p<0.05; DROP if delta<5% OR p>=0.05; edge delta>=5% but p>=0.05: DROP default, preserve module, log inconclusive. | 1H:L232-L236 |
| P1H-E029 | E/TEST | Adversarial: ≥15 distractor queries; ≥15 code-syntax queries; all 50 queries run (no silent drop); clear caches between runs; ≥20 non-zero paired deltas else DROP; quant Jaccard >=0.80 vs full. | 1H:L248-L261 |
| P1H-E030 | E/STATE | Three exit outcomes: EARLY DROP vocab; DROP stats; KEEP. All preserve module behind cfg; DROP disables feature by default. KEEP: circuit breaker + RRF + winning model documented. | 1H:L273-L280 |
| P1H-E031 | E/FLOW | With splade disabled, SPARSE_ONLY cascade must degrade to BM25_ONLY (no dead path). | 1H:L281-L281 |
| P1H-E032 | E/OBS | Measurements: vocab_coverage_code, model_comparison table, recall_delta_percent mean+std, wilcoxon_p_value, splade_query_latency_ms p50/p95, model_load_time_s, binary_size_delta_mb. | 1H:L283-L290 |
| P1H-A001 | A/CFG | 0.3 vocab-coverage threshold is ungrounded heuristic, not published study; 0.2-0.4 band may still measure recall. | 1H:L201-L201 |
| P1H-A002 | A/TEST | If Candidate B unavailable, proceed with A only; full A/B protocol partially skipped. | 1H:L207-L207 |
| P1H-I001 | I/DEP | tokenizers may already exist from Phase 1G; step adds only if missing. | 1H:L128-L134 |

Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data ✓; flags/config ✓; integrations/events (none new) ✓; security (model path/circuit) ✓; failures/recovery ✓; determinism/stats ✓; observability/metrics ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No standalone UI panel; feature-flag and measurements.md are the operator surfaces.

### 3I — deterministic simulation testing

Source: `docs/specs/guide/steps/phase-3I.txt` (465/465 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3I-E001 | E/STATUS | Phase 3I deterministic simulation testing (DST); prerequisites 0.8 entropy quarantine + 3A-3D FSM/SSB/agents/BSP. NOT optional. | 3I:L1-L17 |
| P3I-E002 | E/CAP | Goal: prove FSM+SSB+BSP via deterministic simulation; reference model digital sibling is infallible oracle; same seed = same output. | 3I:L6-L9 |
| P3I-E003 | E/DEP | Pre-read orchestrator BSP/channels/invariants; core KEY TYPES + entropy traits L67-92; external-analysis DST L268-305. | 3I:L11-L14 |
| P3I-E004 | E/CAP | VCR pattern: intercept HTTP at transport (not LlmProvider trait); blake2s hash of full request body; store (hash,response) msgpack cassettes; DST lookup returns recorded response with simulated latency; missing hash fails (prompt drift). | 3I:L22-L40 |
| P3I-E005 | E/CAP | Trait-level LlmProvider mock is complementary unit pattern; VCR catches ollama cache anomalies trait mocks miss. | 3I:L42-L45 |
| P3I-E006 | E/CAP | TraceOps records entire agent trajectory (GRC, tools, branches) as structured trace; DST feeds trajectory instead of live LLM. | 3I:L47-L51 |
| P3I-E007 | E/CAP | Custom SQLite VFS traps write/fsync into in-memory buffer; sim PRNG dictates I/O latency; enables torn WAL, deterministic latency, crash recovery tests; ~200-400 LOC unsafe C FFI. | 3I:L59-L77 |
| P3I-E008 | E/DEP | Custom VFS must use madsim timer queue as single deterministic time source. SPIKE: evaluate if madsim libc I/O hooks make custom VFS unnecessary. | 3I:L72-L81 |
| P3I-E009 | E/FAIL | R1 madsim Windows unverified — spike first. R2 not drop-in tokio — needs cfg(madsim) and madsim-tokio swap. R3 must use proptest-state-machine ReferenceStateMachine not hand-rolled. R4 RUSTFLAGS madsim full rebuild. R5 10k cases need 10min timeout or reduced local count. | 3I:L84-L94 |
| P3I-E010 | E/TEST | 3I.1 madsim 0.2.34 dev-dep spike: two nodes send/receive on Windows with RUSTFLAGS; fail -> document manual SeededClock+SeededRng scheduler alternative. | 3I:L103-L119 |
| P3I-E011 | E/CAP | 3I.2 SimulatorRuntime wraps SeededClock+SeededRng+madsim; advance_time, inject_fault, snapshot_state. | 3I:L126-L127 |
| P3I-E012 | E/CAP | FsmReferenceModel implements ReferenceStateMachine; State=ReferenceState (FSM+SSB rows+agents+channels); Transition=FsmEvent; deliberately simpler than real system. | 3I:L128-L135 |
| P3I-E013 | E/DATA | FsmEvent 10 variants: PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary. | 3I:L136-L144 |
| P3I-E014 | E/FLOW | SuperstepBoundary composite atomic: clear_non_accumulating_topics + execute + clear_ephemeral + evidence_gate_check. AgentCleanup models crashed->cleaned after crash. | 3I:L139-L144 |
| P3I-E015 | E/CAP | StateMachineTest: SUT=actual FSM+SSB; apply returns SUT by value (ownership model); check_invariants compares FSM state, SSB row counts, active agents, LastValue/Topic/Ephemeral channel state. | 3I:L145-L152 |
| P3I-E016 | E/CFG | dev-deps proptest v1.11+, proptest-state-machine v0.8+, madsim 0.2.34 cfg-gated; #[cfg(not(madsim))] stubs skip IPC sim gracefully. | 3I:L154-L157 |
| P3I-E017 | E/LIMIT | Reference model LOC <30% of real FSM LOC (oracle simplicity). Every FsmState reachable via BFS on reference state space. | 3I:L165-L169 |
| P3I-E018 | E/TEST | 3I.3 verify workspace: cargo test --workspace 0; warnings <= baseline. | 3I:L178-L188 |
| P3I-E019 | E/TEST | 3I.4 prop_state_machine sequential 1..200; PROPTEST_CASES=10000 CI 10min hard, 256 local; check_determinism takes bare static fn not closure; synthetic regression generates proptest-regressions/ checked into git; TDD red until 3I.8. | 3I:L190-L230 |
| P3I-E020 | E/TEST | 3I.5 PRE: entropy quarantine — zero Instant::now\|thread_rng\|SystemTime::now in orchestrator src (excl comments) else BLOCK. | 3I:L236-L240 |
| P3I-E021 | E/TEST | 3I.5 BSP: 100 identical checkpoints same seed; UntrackedValue never in checkpoint; EphemeralValue zero-write clears MISSING + write_count reset; Topic appends follow SSB seq; SuperstepBoundary two-phase Topic reset. | 3I:L241-L265 |
| P3I-E022 | E/TEST | 3I.6 madsim IPC (prep 4A): packet_loss_rate 0.3; send_latency 0..500ms Duration range; clog_link asymmetric; SIGKILL mid-message cleanup no orphaned SSB; burst 100% loss 5 msgs then 0%; seed replay; skip+document if madsim unavailable Windows. | 3I:L267-L300 |
| P3I-E023 | E/TEST | 3I.7 adversarial: anti-spiral 3x same GRC / >3 same-file edits STOP; evidence_gate_passes false without artifacts; context budget adaptive truncate; SSB concurrent same-path single-writer; demonic proptest interleavings. Whitespace-only triples still escalate; zero-byte evidence rejected. | 3I:L302-L328 |
| P3I-E024 | E/CAP | 3I.8 implement SimulatorRuntime with with_seed; FsmReferenceModel pure apply; SuperstepBoundary atomic clears; madsim::check_determinism bare fn; NetSim partition/delay/drop + buggify_with_prob; LOC ratio python assert. | 3I:L330-L376 |
| P3I-E025 | E/OBS | 3I.9 metrics phase_3I.toml: state_coverage_pct (>80% valid transition pairs), bugs_found, determinism_verified (100%), regression_count, ops_per_sec (ungrounded 100K target), compile_time_delta_sec, madsim_compile_time_sec. | 3I:L378-L401 |
| P3I-E026 | E/TEST | Exit: 10k+ sequences match; same seed identical; 100 BSP runs; IPC faults pass or documented skip; VCR cassette identical SSB; prompt hash mismatch reported; TraceOps all transitions; SQLite VFS spike done or hooks proven. AUTONOMOUS not enabled (4G). | 3I:L404-L417 |
| P3I-E027 | E/EVIDENCE | madsim drop-in claim FALSE; Windows UNGROUNDED; proptest-state-machine v0.8 VERIFIED; orchestrator.txt has NO DST section (SPEC_GAP — design in external-analysis); channel types GROUNDED orchestrator §§4-5. | 3I:L420-L468 |
| P3I-A001 | A/DEP | Whether custom SQLite VFS is required vs madsim hooks is unresolved until spike. | 3I:L79-L81 |
| P3I-A002 | A/CFG | Windows madsim availability and manual-scheduler equivalence details unresolved until 3I.1. | 3I:L85-L87, L110-L111 |
| P3I-I001 | I/DET | blake2s for VCR request hashing is the named hash in this phase (not blake3). | 3I:L30-L37 |

| P3I-E028 | E/DATA | ReferenceState includes FSM state, SSB row counts, agent lifecycle set, and channel state (LastValue current, Topic vec contents, Ephemeral presence); UntrackedValue excluded from reference channel state. | 3I:L129-L130, L339-L340 |
| P3I-E029 | E/TEST | 3I.4 adversarial: always-true preconditions must still produce at least one invariant failure; mutating one SSB row count in reference must be caught by check_invariants. | 3I:L222-L228 |
| P3I-E030 | E/TEST | 3I.5 adversarial: serialize checkpoint must not contain UntrackedValue marker; crash after ephemeral clear and SSB replay must restore EphemeralValue=MISSING. | 3I:L257-L261 |
| P3I-E031 | E/TEST | 3I.6 adversarial: crash agent mid-SSB-write yields no orphaned rows after cleanup; RUSTFLAGS madsim full rebuild expected and measured in compile_time metrics. | 3I:L286-L297 |
| P3I-E032 | E/CFG | madsim net::Config fields: packet_loss_rate f64, send_latency Range<Duration> with explicit Duration construction, NetSim::clog_link, buggify_with_prob. | 3I:L274-L277, L359-L360, L424-L426 |
| P3I-E033 | E/TEST | 3I.8 acceptance requires PROPTEST_CASES=10000 fsm_reference_model_matches within 10 min, all ipc_ tests under cfg(madsim), and cross-platform LOC ratio script. | 3I:L361-L376 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data/lifecycle ✓; flags/config ✓; integrations/events ✓; security (entropy quarantine) ✓; failures/recovery ✓; determinism/concurrency ✓; observability/metrics ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No operator UI; metrics TOML and proptest-regressions are artifacts.

### 3K — RAG production hardening

Source: `docs/specs/guide/steps/phase-3K.txt` (317/317 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3K-E001 | E/STATUS | Phase 3K RAG production hardening; prerequisites 1E CatRAG/A-RAG, 1G cross-encoder, 3J sidecar for HyDE (3K.3 only). | 3K:L1-L3 |
| P3K-E002 | E/CAP | Upgrade crates/knowledge/ with adaptive chunking, query expansion, HyDE, quality-tier routing; better retrieval → smaller prompts for local LLMs. | 3K:L7-L14 |
| P3K-E003 | E/STATUS | Current stack: DKG 1A, BM25 1B, dual vectors 1C, PageRank 1D, CatRAG/A-RAG 1E, reranker 1G, SPLADE 1H KEEP/DROP done, resilience 1I. Missing: AST chunks, expansion, HyDE, tier routing, recall/MRR metrics. | 3K:L17-L35 |
| P3K-E004 | E/FAIL | R1 HyDE latency 1-3s mitigated by tier routing. R2 AST chunking reuses DKG LanguageGrammar. R3 expansion noise mitigated by template (not LLM) and 2-3 variants. | 3K:L38-L44 |
| P3K-E005 | E/CAP | 3K.1 adaptive chunking: replace 50-line windows; split Rust at fn/impl/struct/enum/mod; Python def/class; TS/Go function/class; non-code markdown headings or blank paragraphs; fallback 50-line on AST fail. | 3K:L49-L60 |
| P3K-E006 | E/CFG | Chunk targets 200-800 tokens; 50-token overlap; metadata file_path, start_line, end_line, chunk_type; Tantivy schema adds chunk_type + line range; re-index if chunk format version changes. | 3K:L61-L66 |
| P3K-E007 | E/TEST | 3K.1: boundaries align AST; no chunk >1000 tokens; overlap exists; parse fail falls back not crash; BM25 results include chunk_type; reindex without data loss. | 3K:L67-L76 |
| P3K-E008 | E/CAP | 3K.2 multi-query template expansion (no LLM): keyword extraction; CamelCase/snake_case symbol guess; synonym error→failure/exception; 2-3 variants for >5 words; skip <5 words. | 3K:L78-L93 |
| P3K-E009 | E/FLOW | 3K.2 run BM25+vector per expansion; merge dedupe by chunk ID; then cross-encoder rerank. Accept: Recall@5 improves; no >100ms latency regression on simple queries. | 3K:L91-L101 |
| P3K-E010 | E/CAP | 3K.3 HyDE: sidecar POST /v1/completions max_tokens=128 temperature=0.7 prompt Write short technical paragraph; embed hypothesis with 1C models; search with hypothesis embedding; RERANK against ORIGINAL query. | 3K:L103-L118 |
| P3K-E011 | E/CFG | HyDE cache hypothetical embeddings 5 minutes same query. Sidecar timeout 3s; unavailable/timeout degrade to direct search never block. Integrates 1I circuit breaker. | 3K:L119-L131 |
| P3K-E012 | E/TEST | 3K.3: HyDE for complex; simple skip zero latency regression; MRR improved on complex; graceful sidecar outage. | 3K:L123-L136 |
| P3K-E013 | E/CAP | 3K.4 QueryTier: FAST BM25-only <20ms; STANDARD hybrid+rerank no HyDE <200ms; DEEP expansion+HyDE+hybrid+rerank <5s. Distinct from 1I SearchMode degradation. | 3K:L138-L159 |
| P3K-E014 | E/CFG | Auto-detect: <5 words FAST; 5-15 code keywords STANDARD; >15 question/debug/arch DEEP. QueryPipeline::search optional tier; default AUTO backwards compatible. | 3K:L152-L157 |
| P3K-E015 | E/TEST | 3K.4: FAST skips reranker+vector; DEEP includes HyDE; three tiers measurably different latency; quality DEEP>=STANDARD>=FAST; existing callers unchanged. | 3K:L160-L168 |
| P3K-E016 | E/TEST | 3K.5 bench: 30 queries (10 simple/10 code/10 complex) known-correct chunks; Recall@5/@10, MRR, latency p50/p95 per tier; write .elai_cc/metrics/phase_3K_rag.toml; regress >5% blocks ship. | 3K:L170-L190 |
| P3K-E017 | E/EVIDENCE | Papers: A-RAG 2602.03442, CatRAG 2602.01965, Deep Research 2508.12752, Deep Search Agents 2508.05668; interface maps workflow/papers/*; independent of 4D web research. | 3K:L193-L199 |
| P3K-E018 | E/CAP | 3K.6 fix A-RAG keyword_search stub: Score_kw=SUM(count(k,text)*len(k)) case-insensitive; longer keywords weighted higher (arag-interface L72-77). | 3K:L202-L211 |
| P3K-E019 | E/CAP | 3K.6 fix semantic_search stub: dual-vector 1C; 3x oversample top_k*3; group by parent; score=MAX similarity among matches. | 3K:L212-L216 |
| P3K-E020 | E/CAP | 3K.6 progressive disclosure: search returns SNIPPETS not full chunks; ChunkRead full text only on explicit request; ContextTracker prevents re-read; ≥50% token savings goal. | 3K:L217-L224 |
| P3K-E021 | E/TEST | 3K.6 adversarial: short 10-line with 3 hits outscores 500-line with 3 (length domination); 60 candidates retrieval <100ms; arag contract tests pass. | 3K:L230-L246 |
| P3K-E022 | E/CAP | 3K.7 Symbolic Anchoring: rule-based CamelCase/UPPER_SNAKE entity extract; match DKG; weak PPR reset_weight(e)=epsilon*\|passages(e)\|^(-1); epsilon=0.2 from CatRagConfig. | 3K:L249-L260 |
| P3K-E023 | E/CAP | 3K.7 Key-Fact enhancement: w_hat=w*(1+beta*I(u,p in verified_triples)); beta=2.5; verified triples high-confidence DKG edges; pure algorithmic no LLM. | 3K:L261-L267 |
| P3K-E024 | E/DEFER | CatRAG Innovation 2 Dynamic Edge Weighting LLM-scored DEFERRED; too expensive for local code retrieval; re-evaluate after 4D web research LLM-in-loop. | 3K:L268-L271 |
| P3K-E025 | E/TEST | 3K.7: symbolic_anchor and key_fact_enhance symbols present; query 'how does the system work' extracts 0 entities; with beta=2.5 non-verified still in top-5; Recall@5 >= 3K.6. | 3K:L272-L294 |
| P3K-E026 | E/OBS | 3K.8 re-run same 30 queries; deltas Recall@5/@10/MRR/tokens-per-query/latency; phase_3K_rag_improved.toml; tokens-per-query >=30% reduction; no metric regress >5%. | 3K:L297-L317 |
| P3K-A001 | A/CFG | Exact synonym lexicon beyond error/failure/exception not fully specified. | 3K:L88-L90 |
| P3K-A002 | A/CFG | AUTO routing keyword lists for code/debug/architecture are descriptive not enumerated. | 3K:L152-L155 |
| P3K-I001 | I/SURFACE | 3K has no new operator UI; surfaces are QueryTier API, metrics TOML artifacts, and retrieval result snippet fields. | 3K:L156-L157, L181-L182, L308-L308 |

| P3K-E027 | E/INT | 3K.1 touches knowledge/src/bm25.rs chunking and knowledge/src/dkg.rs grammar reuse. | 3K:L76 |
| P3K-E028 | E/INT | 3K.2–3K.4 touch knowledge/src/pipeline.rs; QueryTier exported from knowledge/src/lib.rs. | 3K:L101, L136, L168 |
| P3K-E029 | E/INT | 3K.3 reuses providers/src/sidecar_client.rs; never blocks retrieval on sidecar outage. | 3K:L116-L122, L136 |
| P3K-E030 | E/INT | 3K.6–3K.7 touch knowledge/src/arag.rs and catrag.rs; contracts tests/contracts/arag_contract.rs must still pass. | 3K:L241-L246, L288-L294 |
| P3K-E031 | E/EVIDENCE | 3K.6–3K.7 grounding cites arag-interface.md and catrag-interface.md line ranges; progressive disclosure 90% paper figure is grounded interface claim, local acceptance uses ≥50%/≥30% token reductions. | 3K:L242-L245, L289-L293, L237-L239, L314 |
| P3K-E032 | E/TEST | Regression gate both 3K.5 and 3K.8: any metric regresses >5% vs prior baseline requires fix before complete. | 3K:L182-L185, L309-L312 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data ✓; flags/config (QueryTier) ✓; integrations (sidecar HyDE) ✓; security (none new privacy) ✓; failures/fallback ✓; determinism ✓; observability/benchmarks ✓; tests/done gates ✓; constraints/deferred ✓. Surfaces: QueryTier API default AUTO, snippet fields, phase_3K_rag*.toml metrics.

### 4A — Tier-C agent process isolation

Source: `docs/specs/guide/steps/phase-4A.txt` (427/427 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4A-E001 | E/STATUS | Phase 4A Tier-C agent process isolation; prerequisite Phase 3 INTERACTIVE pipeline gates all pass. Foundational for all later Phase 4. | 4A:L1-L9 |
| P4A-E002 | E/CAP | Agents run as isolated OS processes; no agent sees enforcement, other agents, or harness; orchestrator communicates via unix socket / named pipe. | 4A:L6-L8 |
| P4A-E003 | E/DEP | Pre-read orchestrator §12, tools §8-9, reasoning §4 GroundingRequest IPC, audit-results-phase4 all 9 BLOCK+WARN, architecture-overview §9 roles. | 4A:L11-L16 |
| P4A-E004 | E/SEC | BLOCK-1: two seccomp profiles IPC_BASELINE (all) + NETWORK_EXTRA (researcher only); BPF checks socket domain arg not blanket socket. AF_UNIX baseline; AF_INET researcher only. | 4A:L19-L21 |
| P4A-E005 | E/SEC | BLOCK-2: researcher uses veth+iptables NOT CAP_NET_RAW; zero capabilities any role; prctl(PR_SET_NO_NEW_PRIVS) before exec. | 4A:L22-L23 |
| P4A-E006 | E/SEC | BLOCK-3: agent binary has NO LLM client; InferRequest→orchestrator→providers→InferResponse. | 4A:L24-L25 |
| P4A-E007 | E/SEC | BLOCK-4: GRC budget orchestrator-side; counts ThinkStep externally; agent budget advisory only. | 4A:L26-L27 |
| P4A-E008 | E/SEC | BLOCK-5: sockets in 0700 /tmp/elai_cc_{session}/; SO_PEERCRED on accept; unlink on agent death. | 4A:L28-L29 |
| P4A-E009 | E/SEC | BLOCK-6: auditor self-discovery → PROPOSED never auto-CANDIDATE; human approval; independent second auditor validates predicates. | 4A:L30-L32 |
| P4A-E010 | E/SEC | BLOCK-7: Windows weaker; AppContainer preferred; restricted token+ACL deny documented-weaker; Linux is production security target. | 4A:L33-L35 |
| P4A-E011 | E/SEC | BLOCK-8: self-mod locks BOTH IPC canonical realpath AND FS RO mount; BashRequest+GitCommit also filtered. | 4A:L36-L37 |
| P4A-E012 | E/SEC | BLOCK-9: adversarial IPC test battery required. | 4A:L38-L38 |
| P4A-E013 | E/CAP | 4A.1 AgentProcess wraps Command; IPC unix socket Linux / named pipe Windows; frame: 1-byte version + 4-byte length + postcard; MAX_FRAME_SIZE 10MB. | 4A:L43-L54 |
| P4A-E014 | E/DATA | Per-type size caps: AgentHeartbeat <1KB, ThinkStep <64KB, ToolResult <10MB, AgentResponse <context_limit. | 4A:L53-L54 |
| P4A-E015 | E/DATA | Exactly 15 message types: AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest. | 4A:L55-L59 |
| P4A-E016 | E/CAP | elai_cc_agent binary: GRC loop + IPC client + structured output parser; MUST NOT contain LLM client, reqwest, network, tools, FSM, SSB, rules, knowledge. | 4A:L60-L63 |
| P4A-E017 | E/CFG | ProcessConfig per role: allowed_paths, seccomp_profile, resource_limits. ToolFilter whitelist {allowed_tools}; not in list → generic "tool call rejected by policy". Defense-in-depth on PreToolUse. | 4A:L64-L70 |
| P4A-E018 | E/CFG | AgentDispatchConfig authoritative: role, model_preference, tool_filter, rule_domains, grc_config, sandbox_config. Role alone insufficient. | 4A:L71-L74 |
| P4A-E019 | E/EVENT | phase_complete SSB action_type at model-swap phase boundaries: {phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes}; atomic durability marker; model unload only after fsync WAL. | 4A:L75-L79 |
| P4A-E020 | E/DATA | agent_id UUID v4 assigned at spawn by orchestrator. | 4A:L80-L80 |
| P4A-E021 | E/FLOW | Connection sequence listen-before-spawn: create 0700 dir → create socket listen → spawn with path → SO_PEERCRED/SecurityIdentification → version handshake mismatch kills. | 4A:L81-L87 |
| P4A-E022 | E/TEST | 4A.1: two binaries elai_cc + elai_cc_agent; cargo tree agent no reqwest/providers; 15 variants const_assert; postcard round-trip each type; 11MB frame rejected pre-alloc. | 4A:L88-L103 |
| P4A-E023 | E/TEST | 4A.2: cargo build/test/clippy -D warnings clean; cargo test 3x all pass (temp dir isolation). | 4A:L105-L120 |
| P4A-E024 | E/TEST | 4A.3 functional isolation TDD: cannot read .elai_cc ENOENT; NSpid==1 host-verified; AF_INET EPERM; AF_UNIX allowed; researcher HTTPS via veth; 15s heartbeat → SIGKILL; crash exit1 not crash orchestrator; cannot write outside scratch; concurrent agents isolated scratch and no peer AF_UNIX; auditor can read executor output via orchestrator copy; executor cannot read auditor; env only LANG/TERM/PATH/TMPDIR no HOME/USER/API/ELAI_CC_*; wrong PID reject; /proc masked /sys unmounted; RLIMIT_FSIZE 100MB fail write; RLIMIT_AS 512MB OOM agent only. | 4A:L122-L151 |
| P4A-E025 | E/TEST | 4A.3 LLM proxy: InferRequest path; no localhost:11434; oversize prompt rejected. | 4A:L153-L157 |
| P4A-E026 | E/TEST | 4A.3 GRC budget: orchestrator counts ThinkStep per topic; max_calls injects convergence; per-topic independent; orchestrator authoritative when agent local count disagrees. | 4A:L159-L166 |
| P4A-E027 | E/TEST | 4A.3 GroundingRequest proxy: FILE/WEB/SOURCE_CHECK; role-method matrix executor/auditor get FILE/SOURCE/WEB/TEST; context-compiler/prompt-engineer/memory-writer FILE+SOURCE only; TEST_CHECK from context-compiler rejected; counts to budget; no active ThinkStep topic rejected. | 4A:L168-L180 |
| P4A-E028 | E/TEST | 4A.3 test names test_4a_isolation_*/llm_proxy_*/grc_budget_*/grounding_*; one assert each; FAIL until 4A.5. | 4A:L182-L192 |
| P4A-E029 | E/TEST | 4A.4 adversarial IPC: >10MB reject; length=u32::MAX pre-alloc reject; malformed postcard no crash; unsolicited AgentResponse reject; ToolResult unrequested ToolId reject; 10k ThinkStep burst budget drop; second process SO_PEERCRED reject; SIGKILL socket cleanup respawn; mismatched agent_id reject; version mismatch kill; partial frame timeout cleanup. | 4A:L194-L211 |
| P4A-E030 | E/TEST | 4A.4 ReplanRequest: MINOR+evidence auditor fast-track; MAJOR human INTERACTIVE; max 1 replan per agent per phase; no evidence reject; evidence >500 tokens truncate/reject. | 4A:L213-L218 |
| P4A-E031 | E/TEST | 4A.4 self-mod locks: WriteFile locked path generic error; path traversal realpath reject; BashRequest locked RO mount block; GitCommit locked diff reject; symlink realpath reject. | 4A:L220-L225 |
| P4A-E032 | E/TEST | 4A.4 auditor discovery: PROPOSED not CANDIDATE; human before CANDIDATE; second auditor validates; inflated violation_patterns same-hash capped. | 4A:L227-L231 |
| P4A-E033 | E/TEST | 4A.4 crash: orchestrator crash restart SSB consistent; agent crash heartbeat respawn; SSB write BEFORE ToolResult send; partial frame discarded on fd close. | 4A:L233-L237 |
| P4A-E034 | E/TEST | 4A.4 Windows cfg: AppContainer no .elai_cc; Job limits memory/process/timeout; named pipe SecurityIdentification; fallback restricted token still tested. Every BLOCK 1-9 and WARN 1-9 has ≥1 test comment. | 4A:L239-L257 |
| P4A-E035 | E/CAP | 4A.5 agent: heartbeat every 5s; AgentKill 1s flush. Orchestrator gateway: routes Infer→providers, Tool→tools+rules+visibility, Bash→sandbox, Grounding→knowledge+web; GRC budget external; SSB sync before ToolResult. | 4A:L259-L280 |
| P4A-E036 | E/SEC | Linux: clone NEWNS\|NEWPID\|NEWNET; NO_NEW_PRIVS; seccomp IPC_BASELINE + NETWORK_EXTRA SOCK_STREAM AF_INET/6; bind-mount project RO + scratch RW; .elai_cc unmounted; /proc hidepid=2; socket dir NOT mounted into ns; self-mod locked RO; PID 1; RLIMIT_NPROC=1 FSIZE=100MB AS=512MB; env sanitized. | 4A:L282-L295 |
| P4A-E037 | E/SEC | Windows: AppContainer CreateAppContainerProfile preferred; fallback restricted token+Job+ACL; Job ACTIVE_PROCESS=1 PROCESS_MEMORY=512MB JOB_TIME configurable; named pipe SecurityIdentification. | 4A:L297-L302 |
| P4A-E038 | E/SEC | mutable=0 rules protected from all automated modification. Self-mod: IPC realpath + RO mount + GitCommit path filter. | 4A:L304-L312 |
| P4A-E039 | E/DEFER | ShortCoder post-process MUST run in orchestrator after AgentResponse BEFORE auditor view FS — not in agent; verify in 4B PRE_GATE (WARN-3). | 4A:L314-L317 |
| P4A-E040 | E/OBS | 4A.6 metrics phase_4A.toml: IPC p99 <1ms for <64KB; spawn <100ms; memory/agent <30MB; overhead vs in-process <5%; isolation escape fails. Each metric value/unit/timestamp/method. Separation cost estimates 0.1ms/50ms/20MB. | 4A:L337-L373 |
| P4A-E041 | E/INT | Every request-routing call site/struct must wire ssb, fsm, clock through single RuntimeClient; no None/uninitialized after construction. | 4A:L427-L427 |
| P4A-E042 | E/SURFACE | Operator-facing isolation surfaces: generic policy rejection string; human approval for MAJOR replan (INTERACTIVE gate); human approval for auditor PROPOSED rules; Windows vs Linux isolation strength documentation. | 4A:L67-L68, L215-L216, L228-L230, L33-L35 |
| P4A-A001 | A/OBS | IPC/spawn/memory/overhead SLO numbers are UNGROUNDED targets pending 4A.6 empirical measurement. | 4A:L346-L353, L376-L384 |
| P4A-A002 | A/CFG | WARN list 1-9 referenced for test mapping but individual WARN bodies not expanded in this phase file. | 4A:L257-L257 |
| P4A-I001 | I/SURFACE | No new dashboard panel is specified in 4A; isolation is infrastructure with generic IPC error strings as the agent-visible surface. | 4A:L67-L68 |

| P4A-E043 | E/CFG | Heartbeat interval 5s with 15s timeout → orchestrator SIGKILL. AgentKill grants 1s flush window for graceful shutdown. | 4A:L137, L269, L275 |
| P4A-E044 | E/SEC | Researcher NETWORK_EXTRA allows AF_INET/AF_INET6 SOCK_STREAM only via BPF argument inspection; other roles EPERM on AF_INET. | 4A:L134-L136, L285-L287 |
| P4A-E045 | E/SEC | Socket directory /tmp/elai_cc_{session}/ is NOT mounted into agent namespace; agent receives socket fd or path argument; per-agent scratch isolated. | 4A:L141-L142, L290-L291 |
| P4A-E046 | E/FLOW | Orchestrator routes: InferRequest→providers; ToolRequest→tools+rules+visibility_filter; BashRequest→sandbox; GroundingRequest→knowledge+web. | 4A:L276-L277 |
| P4A-E047 | E/TEST | Isolation escape acceptance: agent instructed to find .elai_cc/ must fail (measured in 4A.5 and 4A.6). | 4A:L335, L354-L365 |
| P4A-E048 | E/FAIL | Generic IPC policy rejection message is "tool call rejected by policy" with no rule ID leakage to agent. | 4A:L67-L68 |
| P4A-E049 | E/DATA | ReplanRequest max 1 per agent per phase; MINOR with evidence auditor fast-track; MAJOR requires human INTERACTIVE gate. | 4A:L213-L216 |
| P4A-E050 | E/EVENT | SSB write is synchronous with ToolResult dispatch: write BEFORE sending result (WARN-6 fix). | 4A:L236, L279, L404-L405 |
| P4A-E051 | E/DEP | Decision doc workflow/reasoning/phase-4A-decisions.md is PRE_GATE for 4A.5 IMPLEMENT. | 4A:L262 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/IPC data ✓; flags/config ✓; integrations/events ✓; security/permissions ✓; failures/recovery ✓; determinism ✓; observability/metrics ✓; tests/done gates ✓; constraints/deferred ✓. User-facing: generic policy rejection string, INTERACTIVE human approval for MAJOR replan and auditor PROPOSED rules, isolation strength docs.

### 4H — measured bounded self-modification

Source: `docs/specs/guide/steps/phase-4H.txt` (1292/1292 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4H-E001 | E/STATUS | Phase 4H measured bounded self-modification (DGM/CEL/DarwinNet-derived), revised 2026-04-15 after adversarial audit (4 BLOCK+8 WARN resolved). Prerequisites 4A Tier C isolation. | 4H:L1-L24 |
| P4H-E002 | E/CAP | System improves configs/rules/data from pipeline outcomes via Layer1 metrics (no LLM), Layer1A data-table growth (no LLM), Layer2 staged config proposals, Layer3 structural human-only stubs. Modifying agent NEVER evaluates own modifications. | 4H:L7-L12 |
| P4H-E003 | E/LIMIT | Architectural divergence from DGM: 4H mutates CONFIG KEYS and DATA TABLES only, never agent/Rust source recompilation. | 4H:L36-L41 |
| P4H-E004 | E/CAP | Class A data-table additive low-risk Layer1A: deprecated_patterns, FailureTaxonomy, lazy-code regexes, source-tier entries; validate FPR over last N pipelines; no staged bench. | 4H:L43-L51 |
| P4H-E005 | E/CAP | Class B parameter systemic Layer2: ETC threshold, baby-step LOC, AB-MCTS temps, context weights, freshness half-life, retries, steering coeffs; staged 10/50/200. | 4H:L52-L56 |
| P4H-E006 | E/FAIL | R14 reward hacking — SWE-bench rotation + trigger task (DGM Appendix H). R16 resource contention idle-only lowest priority. R53 learned reward bias EVALUATE conditional. | 4H:L67-L72 |
| P4H-E007 | E/DATA | PipelineOutcome fields include pipeline_mode SURGICAL\|GREENFIELD\|ORACLE, first-try phases, audit findings, coverage, time, tokens, interventions, rules_triggered, test_outcomes, etc_trigger_events, stale_api_catches, context_budget_truncations, failure_taxonomy_misses. | 4H:L87-L103 |
| P4H-E008 | E/DATA | StaleApiCatch: file_path, deprecated_name, replacement_name, ecosystem, caught_by CatchLayer T1_HOOK\|T2_HOOK\|AUDITOR\|ETC. | 4H:L110-L116 |
| P4H-E009 | E/DATA | UnclassifiedError auto-proposes FailureTaxonomy when occurrence_count>=3 across pipelines. | 4H:L118-L123 |
| P4H-E010 | E/DATA | TestOutcomeReport 4-category: fail_to_pass, pass_to_pass, fail_to_fail, pass_to_fail (REGRESSION hard reject). | 4H:L125-L129 |
| P4H-E011 | E/CFG | Score always [0,1] weighted: first_try 0.3, coverage 0.2, efficiency 0.2, intervention 0.15, findings 0.15 via elai_cc.toml [self_mod.score_weights]. | 4H:L131-L140 |
| P4H-E012 | E/OBS | MSI = exponential-decay weighted moving avg of delta_score over N=10; lambda=0.3; weights oldest~0.07 newest 1.0; MSI in [-1,1]; inspired by DarwinNet PSI different formula. | 4H:L142-L152 |
| P4H-E013 | E/DATA | ConfigVariant: variant_id, parent_id, score [0,1], children_count, config_blob, created_at, pipeline_outcomes, pipeline_mode, status ACTIVE\|ARCHIVED\|REVERTED; SQLite table matches fields. | 4H:L154-L165 |
| P4H-E014 | E/FLOW | Parent selection DGM: s_i=sigmoid(10*(alpha_i-0.5)); h_i=1/(1+n_i); p_i proportional s*h; all non-zero; MODE FILTER — SURGICAL parent never for GREENFIELD eval. | 4H:L167-L175 |
| P4H-E015 | E/CFG | AllowedConfigKey closed enum Parameter keys: EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc{vram_tier}, ContextSlotWeight{slot}, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, SelfModFrequency. | 4H:L177-L193 |
| P4H-E016 | E/CFG | AllowedConfigKey DataTable: DeprecatedPattern, FailureTaxonomyCategory, LazyCodePattern, SourceTierEntry. | 4H:L195-L199 |
| P4H-E017 | E/SEC | LOCKED never modifiable even human_approved: score_weights, gate_definitions, auditor_config, benchmark_harness, test_suite, config_variants_schema, safety_locks, lock_list, cloud_monthly_budget, cloud_opus_allowed, cloud_escalation_threshold. | 4H:L201-L207 |
| P4H-E018 | E/DATA | SelfModProposal: variant/parent ids, target_config_key AllowedConfigKey, ProposedChange InsertRow\|UpdateKey, ssb_evidence, reasoning_trace_evidence, rationale, optional ProjectScope. | 4H:L209-L228 |
| P4H-E019 | E/DATA | ProjectScope ecosystem/language/pipeline_mode filters prevent cross-ecosystem over-generalization. | 4H:L219-L224 |
| P4H-E020 | E/DATA | ReasoningTraceRef pinpoints GRC failure location (agent, phase, step index/type, failure_mode). StagedEvaluation Stage1(10)\|Stage2(50)\|Stage3(200) Class B only. | 4H:L230-L239 |
| P4H-E021 | E/DATA | DataTableValidation: FPR=false_positive_count/total; threshold default 0.05; window default 10; verdict PROMOTE\|REJECT\|INSUFFICIENT_DATA. | 4H:L241-L249 |
| P4H-E022 | E/DATA | ConfigVariantSimilarity ouroboros: 0.5*key_overlap+0.3*type_match+0.2*exact_match; STAGNANT if similarity>=0.95 over stagnation_window=3. | 4H:L256-L263 |
| P4H-E023 | E/STATE | OscVerdict: Stagnant{window,similarity}, Oscillating{period}, Evolving, Inconclusive{reason}. | 4H:L265-L269 |
| P4H-E024 | E/DATA | AcRegression + RegressionReport cross-pipeline AC history; never-passed AC is not regression; distinct from single-pipeline pass_to_fail. | 4H:L271-L290 |
| P4H-E025 | E/STATE | EvolutionGateResult Allow{evolved_count}\|Withhold{reason,identical_streak}; evolved_count pairs with similarity<0.95. | 4H:L292-L301 |
| P4H-E026 | E/CFG | SandboxConfig copy parent to .elai_cc/sandbox/{mod_id}/. Layer3 StructuralChange types FSM_STATE\|AGENT_ROLE\|GRC_PROTOCOL\|EVIDENCE_GATE\|PIPELINE_CONTROL\|ALLOWED_CONFIG_KEY_ADDITION require human_approved=true. | 4H:L303-L310 |
| P4H-E027 | E/SEC | Self-mod agent Tier C EXTRA restrictions: cannot write metrics/gates/auditor/benchmark/tests/schema/AllowedConfigKey def/locked keys; cannot READ lock list; four layers dispatch ToolFilter + IPC + FS + git. | 4H:L312-L319 |
| P4H-E028 | E/TEST | 4H.1 acceptance: AllowedConfigKey ≥12 Parameter + ≥4 DataTable; score [0,1]; parent_selection signature; config_variants schema match. | 4H:L321-L349 |
| P4H-E029 | E/TEST | 4H.3 config archive: max 50 variants; one ACTIVE per PipelineMode; mode-filtered parent; revert restores same-mode previous; overflow archives lowest non-ACTIVE; deceptive-dip ancestors survive prune even low score; ancestor-vs-cap collision keeps ancestor under cap. | 4H:L380-L433 |
| P4H-E030 | E/FLOW | 4H.3B deprecated_patterns: 3 catches → auto-propose PROPOSED scoped; FPR over 10 pipelines; <0.05 PROMOTE else REJECT. FailureTaxonomy: 3 misses → category+regex+fix template from successful resolution. Lazy-code: 3 auditor findings → GREP_PATTERN. Source-tier: domain 5+ as T3 with verified_facts/total>0.8 → propose T2. Scope: cargo rule not eval on TypeScript. | 4H:L442-L522 |
| P4H-E031 | E/LIMIT | Layer1A path does NOT trigger staged 10/50/200. Proposals use AllowedConfigKey DataTable variants only. | 4H:L517-L522 |
| P4H-E032 | E/FLOW | 4H.4 log-driven proposals read SSB full trace + reasoning_traces; must cite ssb_evidence>=1 and reasoning WHERE failed; target AllowedConfigKey; heuristics fallback only; DataTable→1A Parameter→staged. | 4H:L537-L581 |
| P4H-E033 | E/TEST | 4H.5 staged Class B: Stage1 10 discard on fail; Stage2 50 includes trigger_task; Stage3 200 if >40% on 60-subset AND top2 archive + 3 SWE rotation; PASS_TO_FAIL immediate reject; mode-aware tasks; Stage1 fail not archived; Stage2/3 fail stay for branching; Class A never enters. | 4H:L584-L652 |
| P4H-E034 | E/SURFACE | Trigger task surface: .elai_cc/benchmarks/trigger_task.toml with input, expected_output_hash sha256, metric_key — project-specific canary against objective hacking. | 4H:L589-L598 |
| P4H-E035 | E/SURFACE | Stage3 promotion gate requires better trigger task AND no SWE regression AND no auditor safety regressions AND no evidence-gate definition mods → ACTIVE demote previous ARCHIVED. | 4H:L612-L615 |
| P4H-E036 | E/SEC | 4H.6 four independent enforcement layers; generic error no path leak; cannot read lock list; locked keys rejected at dispatch; auto-revert >10% degrade over 3 pipelines; Layer3 human_approved enforced; cannot extend AllowedConfigKey; FPR threshold locked; objective-hacking trigger task. | 4H:L655-L723 |
| P4H-E037 | E/OBS | 4H.7 MSI pure arithmetic no LLM; IMPROVING MSI>0.3 frequency 1/3 pipelines; STABILIZING -0.1<MSI<=0.3 frequency 1/5; DIVERGING MSI<=-0.1 pause that mode + user alert + restore highest-score; per-mode independent; stored self_mod_history.db. | 4H:L734-L801 |
| P4H-E038 | E/SURFACE | DIVERGING MSI alerts user and pauses self-mod for that PipelineMode only (never silently continues). | 4H:L746-L746, L775-L779 |
| P4H-E039 | E/OBS | 4H.7B OSC: stagnation 3 consecutive sim>=0.95 → ConvergenceStagnant SSB + 2x exploration_bonus next 3 selections (does not block promotion). Oscillating period-2 full sim(N,N-2) AND sim(N-1,N-3) both >=0.95 needs ≥4 variants → ConfigOscillation SSB + freeze differing keys 5 pipelines. | 4H:L804-L848 |
| P4H-E040 | E/OBS | 4H.7C REG: cross-pipeline AC regression blocks 4H.5 promotion even if score up; 3 consecutive regressed pipelines on active → 4H.6 auto-revert; persistent never-passed not regression; per-mode. | 4H:L897-L970 |
| P4H-E041 | E/OBS | 4H.7D EVO: if evolved_count==0 AND latest_sim>=0.95 Withhold → EvolutionWithheld SSB and FORCE MSI STABILIZING (not IMPROVING) regardless of score noise; per-mode. | 4H:L973-L1050 |
| P4H-E042 | E/FLOW | 4H.8 Layer1 security/safety/architecture rules NEVER auto-adjusted. Layer1A four growth paths + ProjectScope + locked FPR. Layer2 log-driven + archive + staged + sandbox + rotation + 4-layer read-lock + AllowedConfigKey routing. Layer3 stubs only. | 4H:L1053-L1090 |
| P4H-E043 | E/FLOW | Self-referential alignment loop: pipeline→SSB→self-mod reads→propose→1A/2 validate→archive→next pipeline. | 4H:L1092-L1097 |
| P4H-E044 | E/SURFACE | Anti-staleness wiring: 4H triggers nightly elai dep-audit CLI; outdated deps → stale_api_catches → Layer1A deprecated_patterns → steering vector refresh metrics/compute_steering_vector.py. | 4H:L1099-L1106 |
| P4H-E045 | E/FLOW | Reasoning wiring: 3+ unclassified errors → FailureTaxonomy category + fix template from Layers2-6 resolution → future Layer1 deterministic sub-100ms. Token efficiency: truncations → ContextSlotWeight / BabyStepMaxLoc proposals. | 4H:L1108-L1121 |
| P4H-E046 | E/EVENT | SSB action_types: convergence_stagnant, config_oscillation, regression_detected, evolution_withheld. | 4H:L1133-L1134 |
| P4H-E047 | E/DEFER | Optional cross-transfer validation with different FM; FM-specific variants marked not universal. | 4H:L1136-L1139 |
| P4H-E048 | E/OBS | 4H.9 metrics phase_4H.toml: e2e self-mod time; Layer1A proposal/FPR/growth rates; Stage1 elimination >60% target / Stage3 <20% pattern; MSI trajectory; anti-fragility recovery; closed-loop tightness; stale catch trends; Layer1 classification rate. | 4H:L1186-L1238 |
| P4H-E049 | E/DEFER | EVALUATE SWE-World-style learned reward pre-screen ONLY if MSI shows >50% Stage1 fail (poor proposals); else skip. SWR accuracy numbers do NOT transfer; real Stage1 still runs after screen. Risk R53. | 4H:L1241-L1264 |
| P4H-E050 | E/INT | RuntimeClient ssb/fsm/clock wiring rule same as 4A exit. | 4H:L1290-L1292 |
| P4H-E051 | E/LIMIT | 4H specifies no new full dashboard panel or live cockpit UI; operator surfaces are alerts, CLI dep-audit, TOML metrics, and SQLite history only. | 4H:L1186-L1230 |
| P4H-A001 | A/CFG | max 50 archive cap is OUR DESIGN not DGM; resource-driven. | 4H:L437-L437 |
| P4H-A002 | A/CFG | MSI thresholds 0.3/-0.1 and 3 phases are OUR DESIGN vs DarwinNet 2 phases. | 4H:L796-L797 |
| P4H-A003 | A/CFG | Source-tier auto-promote verified_facts/total >0.8 and domain hit count 5+ are specified but broader tier-mapping rules remain design-local. | 4H:L483-L488 |
| P4H-I001 | I/SURFACE | self_mod_history.db and phase_4H.toml are operator-visible measurement/history surfaces for later cockpit consumption. | 4H:L751-L751, L1220-L1221 |
| P4H-C001 | C/LIMIT | DGM paper modifies agent code; 4H deliberately refuses code mutation — safety divergence must not be collapsed to "full DGM". | 4H:L36-L41 |

| P4H-E052 | E/CFG | Default score weights in elai_cc.toml [self_mod.score_weights]: first_try_rate_weight=0.3, coverage_weight=0.2, efficiency_weight=0.2, intervention_weight=0.15, findings_weight=0.15. | 4H:L134-L139 |
| P4H-E053 | E/CFG | Parent selection parameters lambda=10 and alpha_0=0.5 per DGM section 2.3; exploration h_i=1/(1+children_count). | 4H:L167-L172 |
| P4H-E054 | E/CFG | MSI exponential weights with N=10 lambda=0.3 listed as [0.07,0.08,0.10,0.13,0.17,0.22,0.29,0.37,0.48,0.61] then normalized. | 4H:L740-L742 |
| P4H-E055 | E/STATE | Only one variant status=ACTIVE per PipelineMode at a time; revert promotes previous highest-score ARCHIVED same-mode variant. | 4H:L391-L400 |
| P4H-E056 | E/FLOW | Archive overflow when count>50 moves lowest-score non-ACTIVE to cold storage; ancestor-of-active and children_count>0 protected from deceptive-dip prune. | 4H:L401-L425 |
| P4H-E057 | E/TEST | 4H.3B scope enforcement: cargo-scoped deprecated_patterns not evaluated on TypeScript pipelines; unscoped rules evaluate on all. | 4H:L490-L495 |
| P4H-E058 | E/TEST | 4H.3B FailureTaxonomy proposed pattern must match ≥3 distinct historical error instances, not 3 occurrences of one pipeline error. | 4H:L510-L513 |
| P4H-E059 | E/FLOW | Stage1 fail → variant discarded entirely (not archived). Stage2 fail → archived with Stage2 score. Stage3 fail → archived near-miss higher parent priority than Stage2 fails. | 4H:L616-L621 |
| P4H-E060 | E/TEST | Stage3 entry requires >40% success on 60-task subset AND top two performers in archive (DGM section 2.5). | 4H:L602-L604, L643 |
| P4H-E061 | E/SEC | Self-mod cannot write metrics code (objective hacking defense); trigger task known-answer catches metric gaming even if score improves. | 4H:L678-L684, L705-L708 |
| P4H-E062 | E/SEC | Layer1A cannot target Parameter keys; FPR threshold is locked and proposing change to it is rejected. | 4H:L685-L690, L714-L715 |
| P4H-E063 | E/OBS | Stagnant OSC does NOT block promotion (valid stop state) but multiplies exploration_bonus by 2.0 for next 3 parent_selection rounds in that mode. | 4H:L842-L846 |
| P4H-E064 | E/OBS | Oscillating freezes keys that differ between the two states into AllowedConfigKey runtime exclusion list for next 5 pipelines. | 4H:L847-L849 |
| P4H-E065 | E/EVENT | SSB rows ConvergenceStagnant / ConfigOscillation emitted on OSC verdicts; EvolutionWithheld on EVO Withhold. | 4H:L843-L848, L1007-L1008, L1133-L1134 |
| P4H-E066 | E/FLOW | REG used as additional gate at 4H.5 Stage3→ACTIVE promotion and every 3 pipelines on active variant for auto-revert. | 4H:L923-L927, L1127-L1130 |
| P4H-E067 | E/OBS | 4H.9 targets: Stage1 elimination rate >60% pattern, Stage3 reach <20% pattern (DGM cost pattern, measured as baseline not hard fail). | 4H:L1199-L1201, L1227 |
| P4H-E068 | E/FAIL | StructuralChange.apply without human_approved=true returns Err or panics; Layer3 is enforced not advisory. | 4H:L674-L675, L709-L711 |
| P4H-E069 | E/SURFACE | CLI surface: `elai dep-audit` nightly anti-staleness trigger invoked by 4H loop. | 4H:L1100-L1100, L1144 |
| P4H-E070 | E/DATA | self_mod_history.db stores MSI history and RegressionReport persistence. | 4H:L751, L1129-L1130 |
| P4H-E071 | E/TEST | 4H.8 adversarial: every proposal.ssb_evidence must reference logs from last COMPLETE pipeline with completed_at < proposal.created_at. | 4H:L1148-L1152 |
| P4H-E072 | E/LIMIT | Security/safety/architecture rules NEVER auto-adjusted by Layer1. | 4H:L1058-L1059 |
| P4H-E073 | E/DEFER | Layer3 structural execution deferred until system maturity proves need; only stubs+gate in this phase. | 4H:L1088-L1090 |
Category coverage: capabilities/actions ✓; workflows/states/lifecycle ✓; schemas/data ✓; flags/config/enums ✓; integrations/events ✓; security/permissions ✓; failures/revert/retry ✓; determinism ✓; observability/MSI/OSC/REG/EVO ✓; tests/done gates ✓; constraints/deferred ✓. Surfaces: user alert on MSI DIVERGING, Layer3 human_approved gate, `elai dep-audit` CLI, trigger_task.toml, phase_4H.toml, self_mod_history.db.

### Cross-source interactions (PHASE-006)

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| PHASE-006/IX-001 | E | Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this). | P1H-E027, P1H-E032; P3K-E003 |
| PHASE-006/IX-002 | E | 3K hardening composes 1E A-RAG/CatRAG stubs, 1G reranker, 1C embeddings, 1A DKG grammars, and 3J sidecar only for HyDE. | P3K-E001–P3K-E003, P3K-E010, P3K-E015–P3K-E018 |
| PHASE-006/IX-003 | E | 3K.4 QueryTier intentional quality selection is explicitly distinct from 1I SearchMode fault-degradation cascade. | P3K-E012 |
| PHASE-006/IX-004 | E | 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence. | P3I-E001–P3I-E002; P4A-E036; P4H-E001 |
| PHASE-006/IX-005 | E | 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary. | P3I-E018; P4A-E013–P4A-E014 |
| PHASE-006/IX-006 | E | 3I entropy quarantine (zero raw time/rng in orchestrator) is a hard PRE_GATE for BSP determinism and for any later seed-replay of isolation tests. | P3I-E016 |
| PHASE-006/IX-007 | E | 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch. | P4A-E001; P4H-E001, P4H-E027, P4H-E038 |
| PHASE-006/IX-008 | E | 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging. | P4A-E016; P4H-E027, P4H-E038 |
| PHASE-006/IX-009 | E | 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern. | P4A-E008; P4H-E026, P4H-E032 |
| PHASE-006/IX-010 | E | 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list. | P4A-E010, P4A-E034; P4H-E027, P4H-E038 |
| PHASE-006/IX-011 | E | 4A ShortCoder deferral to 4B orchestrator placement is a cross-phase WARN-3 handoff outside this chunk's implementation. | P4A-E035 |
| PHASE-006/IX-012 | E | 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria. | P4A-E018, P4A-E038; P4H-E048 |
| PHASE-006/IX-013 | E | 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance. | P4H-E009, P4H-E031, P4H-E043 |
| PHASE-006/IX-014 | E | 4H FailureTaxonomy expansion closes reasoning-architecture Layer1 with fix templates from higher layers' successful resolutions. | P4H-E010, P4H-E032, P4H-E044 |
| PHASE-006/IX-015 | E | 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop. | P4H-E034–P4H-E036; P3K-E014; P1H-E024–P1H-E026 |
| PHASE-006/IX-016 | E | 4H OSC/REG/EVO ouroboros ports operate per PipelineMode in parallel with MSI and feed SSB action types. | P4H-E040–P4H-E042, P4H-E045 |
| PHASE-006/IX-017 | E | 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc. | P3K-E017; P4A-E021 |
| PHASE-006/IX-018 | E | Budget authority handoff: 2C in-process budgets become advisory under 4A orchestrator ThinkStep counting. | P4A-E006, P4A-E023 |
| PHASE-006/IX-019 | I | 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation. | P3K-E003; P1H-E027 |
| PHASE-006/IX-020 | E | Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT. | P4A-E032; P4H-E041 |
| PHASE-006/IX-021 | E | 3I VCR uses blake2s for prompt hashing while much of ELAI elsewhere bans blake2; this phase-local choice is intentional for cassette keys. | P3I-I001 |
| PHASE-006/IX-022 | E | Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network. | P4A-E003–P4A-E004; P3K-E010 |

### Ambiguity and conflict register (PHASE-006)

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| PHASE-006/AC-001 | ambiguity | 1H 0.3 vocab threshold ungrounded; 0.2–0.4 band may still measure recall. | P1H-A001 |
| PHASE-006/AC-002 | ambiguity | 1H Candidate B missing path skips full A/B comparison. | P1H-A002 |
| PHASE-006/AC-003 | ambiguity/dependency | 3I custom SQLite VFS necessity vs madsim hooks unresolved until spike. | P3I-A001 |
| PHASE-006/AC-004 | ambiguity | 3I Windows madsim availability and manual scheduler equivalence open until 3I.1. | P3I-A002 |
| PHASE-006/AC-005 | ambiguity/config | 3K synonym lexicon and AUTO keyword lists incomplete. | P3K-A001–P3K-A002 |
| PHASE-006/AC-006 | ambiguity/measurement | 4A IPC/spawn/memory/overhead SLOs are ungrounded until 4A.6. | P4A-A001 |
| PHASE-006/AC-007 | ambiguity | 4A WARN 1–9 bodies not expanded in phase file despite required test mapping. | P4A-A002 |
| PHASE-006/AC-008 | ambiguity/design | 4H archive cap 50 and MSI thresholds are local design choices, not paper transfers. | P4H-A001–P4H-A002 |
| PHASE-006/AC-009 | ambiguity | 4H source-tier auto-promote thresholds (5 hits, 0.8 verified ratio) lack broader tier policy. | P4H-A003 |
| PHASE-006/AC-010 | conflict | DGM-style code self-mod vs 4H config/data-only must not be flattened. | P4H-C001 |

### Summary deltas vs MASTER (PHASE-006)

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-006/SD-1H-001 | **ADD** full risk register R24–R30 including air-gap ort download, tokenizers separate crate, and statistical power of 10 vs 50 queries. | MASTER:L135-L143 | P1H-E007–P1H-E009 |
| PHASE-006/SD-1H-002 | **ADD** PrePooled vs RawMLM ONNX output shapes, RRF k=60 0.5/0.5 weights, circuit-breaker empty-expansion fallback, and SPLADE_MODEL_PATH default. | MASTER:L141 | P1H-E016–P1H-E021 |
| PHASE-006/SD-1H-003 | **ADD** early-drop 0.2–0.4 calibration note, quant Jaccard≥0.95 prefer int8, cross-model within-1% pick-faster, Wilcoxon paired protocol, and ≥20 non-zero pairs power gate. | MASTER:L142-L143 | P1H-E022–P1H-E026, P1H-A001 |
| PHASE-006/SD-1H-004 | **ADD** SPARSE_ONLY→BM25_ONLY cascade verification when splade disabled and full measurement field list including binary_size_delta_mb. | MASTER:L143 | P1H-E028–P1H-E029 |
| PHASE-006/SD-1H-005 | **ADD** knowledge.txt typo/lock note and UNGROUNDED code-search recall labeling. | MASTER:L135-L143 | P1H-E006, P1H-E010 |

### 3I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-006/SD-3I-001 | **ADD** full VCR cassette design (blake2s HTTP body hash, msgpack, prompt-drift fail) and TraceOps trajectory layer. | MASTER:L349 | P3I-E004–P3I-E006 |
| PHASE-006/SD-3I-002 | **ADD** custom SQLite VFS details, madsim timer coordination, and spike-to-possibly-skip VFS. | MASTER:L349 | P3I-E007–P3I-E008, P3I-A001 |
| PHASE-006/SD-3I-003 | **ADD** complete FsmEvent ten-name list, SuperstepBoundary atomic substeps, AgentCleanup lifecycle, and by-value SUT apply semantics. | MASTER:L351 | P3I-E012–P3I-E014 |
| PHASE-006/SD-3I-004 | **ADD** R1–R5 risk block, PROPTEST_CASES 10000/256 split, bare-fn check_determinism constraint, and proptest-regressions synthetic seed. | MASTER:L350-L354 | P3I-E009, P3I-E017 |
| PHASE-006/SD-3I-005 | **ADD** entropy-quarantine BLOCK PRE_GATE, burst-loss 5×100%, whitespace anti-spiral, zero-byte evidence rejection, and ops/sec ungrounded 100K target. | MASTER:L355-L358 | P3I-E016, P3I-E018–P3I-E021 |
| PHASE-006/SD-3I-006 | **ADD** SPEC_GAP that orchestrator.txt has no DST section; design lives in external-analysis. | MASTER:L346-L358 | P3I-E023 |
| PHASE-006/SD-3I-007 | **ADD** exit note AUTONOMOUS not enabled (Phase 4G) and HITL walking-skeleton is multi-phase. | MASTER:L358 | P3I-E022 |

### 3K deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-006/SD-3K-001 | **ADD** current-stack inventory (1A–1I) and explicit missing list before steps. | MASTER:L159-L161 | P3K-E003 |
| PHASE-006/SD-3K-002 | **ADD** chunk metadata fields, Tantivy schema extension, format-version reindex migration, and 1000-token hard cap check. | MASTER:L162 | P3K-E005–P3K-E007 |
| PHASE-006/SD-3K-003 | **ADD** exact HyDE sidecar prompt/max_tokens/temperature, 3s timeout, 5-minute cache, and rerank-against-original adversarial mitigation. | MASTER:L164 | P3K-E010–P3K-E011 |
| PHASE-006/SD-3K-004 | **ADD** per-tier latency budgets FAST<20ms STANDARD<200ms DEEP<5s and AUTO word-count rules. | MASTER:L165 | P3K-E012–P3K-E013 |
| PHASE-006/SD-3K-005 | **ADD** A-RAG Score_kw formula, 3× oversample MAX grouping, progressive disclosure ≥50% tokens, and length-domination adversarial. | MASTER:L167 | P3K-E015–P3K-E018 |
| PHASE-006/SD-3K-006 | **ADD** epsilon/beta formulas, entity-extraction 0-entity adversarial, beta dominance adversarial, and post-improvement tokens ≥30% reduction criterion. | MASTER:L168-L170 | P3K-E019–P3K-E022 |
| PHASE-006/SD-3K-007 | **QUALIFY** paper list and independence from 4D web research. | MASTER:L159-L170 | P3K-E015 |

### 4A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-006/SD-4A-001 | **ADD** per-type frame size caps, postcard version framing details, and exactly-15 const_assert. | MASTER:L380-L381 | P4A-E013–P4A-E015 |
| PHASE-006/SD-4A-002 | **ADD** full listen-before-spawn sequence, UUID v4 agent_id, and version-handshake kill. | MASTER:L385 | P4A-E019–P4A-E020 |
| PHASE-006/SD-4A-003 | **ADD** complete functional isolation test battery including env sanitization, /proc/sys masks, RLIMIT behaviors, concurrent agent isolation, and role-method grounding matrix. | MASTER:L392 | P4A-E021–P4A-E025 |
| PHASE-006/SD-4A-004 | **ADD** full adversarial IPC/replan/self-mod/crash/Windows test inventory and WARN/BLOCK coverage requirement. | MASTER:L389 | P4A-E026–P4A-E031 |
| PHASE-006/SD-4A-005 | **ADD** Linux clone flags, rlimits defaults, heartbeat 5s/15s, and Windows Job object numbers. | MASTER:L382-L388 | P4A-E032–P4A-E034 |
| PHASE-006/SD-4A-006 | **ADD** phase_complete SSB fields and RuntimeClient wiring exit rule. | MASTER:L391 | P4A-E018, P4A-E038 |
| PHASE-006/SD-4A-007 | **ADD** separation-cost planning numbers (0.1ms/50ms/20MB) vs SLO targets. | MASTER:L392 | P4A-E036–P4A-E037, P4A-A001 |
| PHASE-006/SD-4A-008 | **QUALIFY** feasibility GRC 3/600 stub-write/bash timeouts appear in MASTER dispatch bullet but are not restated as 4A step numbers in phase-4A.txt (they live in later 4F); do not invent them as 4A.x steps. | MASTER:L390 | P4A-E017 |

### 4H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-006/SD-4H-001 | **ADD** revision history 2026-04-15 audit resolution (4 BLOCK/8 WARN), paper refs DGM/DarwinNet/SWE-World, and code-vs-config safety divergence. | MASTER:L480-L482 | P4H-E001–P4H-E003, P4H-C001 |
| PHASE-006/SD-4H-002 | **ADD** complete PipelineOutcome field set including ETC/staleness/truncation/miss vectors and CatchLayer enum. | MASTER:L486-L488 | P4H-E007–P4H-E010 |
| PHASE-006/SD-4H-003 | **ADD** full AllowedConfigKey Parameter/DataTable/LOCKED lists including cloud spend locks and Freshness* keys. | MASTER:L493 | P4H-E015–P4H-E017 |
| PHASE-006/SD-4H-004 | **ADD** DataTableValidation FPR defaults, ProjectScope, ReasoningTraceRef, ProposedChange variants, and StructuralChange type set. | MASTER:L479-L493 | P4H-E018–P4H-E026 |
| PHASE-006/SD-4H-005 | **ADD** ouroboros OSC/REG/EVO algorithms, SSB action types, exploration boost, key freeze, and MSI force-STABILIZING on Withhold. | MASTER:L480-L494 | P4H-E022–P4H-E025, P4H-E040–P4H-E042, P4H-E045 |
| PHASE-006/SD-4H-006 | **ADD** staged-eval archive handling differences Stage1 discard vs Stage2/3 retain, trigger_task.toml schema, and mode-aware evaluation. | MASTER:L484 | P4H-E034–P4H-E036 |
| PHASE-006/SD-4H-007 | **ADD** Layer1A four growth paths with numeric thresholds (3 catches, FPR 0.05, domain 5+, verified 0.8) and no-staged-eval path. | MASTER:L483 | P4H-E031–P4H-E033 |
| PHASE-006/SD-4H-008 | **ADD** MSI frequency schedules (1/3 and 1/5 pipelines), per-mode pause, anti-fragility N=10 recovery bound, and exponential weight series. | MASTER:L491-L492 | P4H-E039 |
| PHASE-006/SD-4H-009 | **ADD** anti-staleness `elai dep-audit`, reasoning fix-template loop, token-efficiency slot proposals, and optional FM cross-transfer. | MASTER:L494 | P4H-E043–P4H-E046 |
| PHASE-006/SD-4H-010 | **ADD** conditional learned-reward EVALUATE section with domain-mismatch warnings and R53. | MASTER:L480-L494 | P4H-E047 |
| PHASE-006/SD-4H-011 | **ADD** four-layer enforcement detail, lock-list unreadability, objective-hacking trigger, and FPR threshold lock. | MASTER:L494 | P4H-E027, P4H-E038 |

## PHASE-007 — source `DEEP-PHASE-007.discovery.md`

- Declared feature atoms: **198** (E=175 I=5 A=14 C=4).
- Parsed feature rows in this aggregate: **198**.
- Per-file: pre-phase-0 **028**; phase-1I **044**; phase-2D **028**; phase-3B **035**; phase-4D **063**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-007.discovery.md`.

### Feature atoms

### pre-phase-0 — Windows platform fixes

Source: `docs/specs/guide/steps/pre-phase-0.txt` (200/200 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| PP0-E001 | E/STATUS | PRE-PHASE 0 makes claw-code-rust-port compile and pass all tests on Windows 11 with zero compile errors, zero test failures, zero warnings, and no feature changes. | pre-phase-0:L1-L10 |
| PP0-E002 | E/DEP | Prerequisite: claw-code-rust-port cloned into `.elai_cc/`. No pre-spec files; pure platform work on the upstream fork. | pre-phase-0:L2-L13 |
| PP0-E003 | E/LIMIT | Scope forbids behavioral code-path changes; Unix-only tests must be `cfg`-gated, not deleted, and still run on Linux CI. | pre-phase-0:L10-L17, L167-L171 |
| PP0-E004 | E/FAIL | R-pre1: inherently Unix-only shell tests; mitigation is cfg-gate. R-pre2: path-separator fixes must use `MAIN_SEPARATOR`/`Path` methods, never hardcoded slash direction. | pre-phase-0:L15-L19 |
| PP0-E005 | E/EVIDENCE | Baseline 2026-04-06: cargo check 40.15s / 0 errors / 2 warnings; cargo test 10 compile errors (2 root causes) + 8 runtime failures (4 root causes); 69 `.rs` files; 61,317 LOC; 684 test annotations. | pre-phase-0:L22-L25, L180-L189 |
| PP0-E006 | E/CAP | P0.1 fixes 10 compile errors in three files by cfg-gating `std::os::unix::fs::PermissionsExt` and all `set_mode(0o755)` sites. | pre-phase-0:L30-L50 |
| PP0-E007 | E/DATA | P0.1 files: `mcp_stdio.rs` (use L1413; set_mode L1456/1502/1636/1761/2680), `mcp_tool_bridge.rs` (use L317; set_mode L434), `mock_parity_harness.rs` (use L4; set_mode L425). | pre-phase-0:L36-L46 |
| PP0-E008 | E/CFG | Windows pattern: `#[cfg(unix)]` chmod block; `#[cfg(windows)]` no-op because Windows has no file mode bits and `.cmd`/`.ps1` are executable by default. | pre-phase-0:L39-L50 |
| PP0-E009 | E/TEST | P0.1 gates: cargo test --no-run 0 errors; all `use std::os::unix` imports cfg-gated; acceptance is 0 compile errors across crates. | pre-phase-0:L51-L61 |
| PP0-E010 | E/CAP | P0.2 fixes 4 plugin-hook test failures in `plugins/src/hooks.rs` by cfg-gating Unix `.sh` tests and adding Windows `.cmd`/PowerShell equivalents. | pre-phase-0:L63-L87 |
| PP0-E011 | E/SURFACE | Failing plugin tests: `collects_and_runs_hooks_from_enabled_plugins` (L395, pre.sh), `pre_tool_use_denies_when_plugin_hook_exits_two` (L456, printf/exit 2), `propagates_plugin_hook_failures` (L473, printf/exit 1). | pre-phase-0:L72-L81 |
| PP0-E012 | E/FLOW | `write_hook_plugin` (L362): Unix keeps `.sh`; Windows writes `.cmd` with `@echo off` + echo. `HookRunner::run_hook_command` (~L326) must use cmd.exe on Windows if it hardcodes `/bin/sh`. | pre-phase-0:L82-L87 |
| PP0-E013 | E/TEST | P0.2 acceptance: `cargo test -p plugins --lib` 0 failures / "33 passed; 0 failed". | pre-phase-0:L88-L97 |
| PP0-E014 | E/CAP | P0.3 fixes 4 tools-crate failures in `tools/src/lib.rs` test module. | pre-phase-0:L99-L118 |
| PP0-E015 | E/TEST | `glob_and_grep_tools_cover_success_and_errors` (L6672): path ends_with must tolerate Windows backslash via Path::ends_with or separator normalize. | pre-phase-0:L106-L108 |
| PP0-E016 | E/TEST | `powershell_runs_via_stub_shell` (L7069): cfg-gate Unix chmod; Windows scripts executable by default. | pre-phase-0:L109-L111 |
| PP0-E017 | E/TEST | `repl_executes_python_code` (L7007): exit 9009 means `python3` missing; use `python` on Windows, `python3` on Unix. | pre-phase-0:L112-L114 |
| PP0-E018 | E/TEST | `skill_loads_local_skill_prompt` (L5745): PoisonError on env lock; fix with dedicated mutex, catch_unwind, or `#[serial]` isolation. | pre-phase-0:L115-L118 |
| PP0-E019 | E/TEST | P0.3 acceptance: `cargo test -p tools --lib` 0 failures / "61 passed; 0 failed"; adversarial asks 10 consecutive clean runs. | pre-phase-0:L119-L128 |
| PP0-E020 | E/CAP | P0.4 removes 2 warnings: unused `Path` import in `plugins/hooks.rs` L2; unused `mut` on `command_builder` in `runtime/hooks.rs` L636. | pre-phase-0:L130-L144 |
| PP0-E021 | E/OBS | P0.5 records clean baseline to `.elai_cc/metrics/pre_phase_0.toml` fields: compile_time, test_total, test_pass, test_fail, loc_total, warnings. | pre-phase-0:L146-L157, L173-L177 |
| PP0-E022 | E/TEST | Exit criteria: cargo check 0 errors/warnings on Windows; cargo test 0 failures; all cfg-gated tests documented with Linux-only flag. | pre-phase-0:L161-L171 |
| PP0-E023 | E/OBS | Measurements: compile time, test counts (total/passed/failed/cfg-gated-out), warning count 0, files modified list. | pre-phase-0:L173-L177 |
| PP0-E024 | E/EVIDENCE | Grounding table verifies 10 errors, 7 set_mode sites, 3 unix imports, 4+4 failures, 2 warnings, exit 9009, PoisonError, 61,317 LOC/69 files/684 tests. | pre-phase-0:L180-L189 |
| PP0-E025 | E/INT | WIRING_CONTRACT: produces `hooks.rs::shell_command`; reachable from `hooks.rs::run_command`; observable via `elai.exe archive verify --json` expecting SSB `archive_verify_result`. | pre-phase-0:L192-L200 |
| PP0-I001 | I/OBS | Archive-verify observable effect implies pre-phase-0 shell fix is production-reachable through CLI archive path, not only unit tests. | pre-phase-0:L198-L200 |
| PP0-A001 | A/TEST | Exact list of which tests become cfg-gated-out on Windows is required at exit but not pre-enumerated beyond the three named plugin and four tools cases. | pre-phase-0:L159-L164, L171-L171 |
| PP0-A002 | A/CFG | Whether `HookRunner` already hardcodes `/bin/sh` is left as an audit question at implement time. | pre-phase-0:L85-L87, L92-L93 |

Category coverage: compile/runtime fixes ✓; path/python/env platform rules ✓; metrics artifact ✓; wiring CLI surface ✓; constraints ✓. No privacy/compliance policy; no retry/idempotency beyond serial tests.

### phase-1I — resilience layer

Source: `docs/specs/guide/steps/phase-1I.txt` (343/343 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1I-E001 | E/STATUS | PHASE 1I is Resilience Layer: circuit breaker, degradation, startup, tracing. Prerequisites: 1E complete and 1G complete. | phase-1I:L1-L2 |
| P1I-E002 | E/DEP | Blocking note: implementation-guide dep graph L141 shows only 1E→1I; must add 1G→1I before implementation or 1I.5 fails. ACTION: update guide L141. | phase-1I:L3-L9 |
| P1I-E003 | E/CAP | Goal: survive individual component failures with circuit breakers, degradation cascade, startup readiness, query validation, cross-encoder concurrency, pipeline tracing, index versioning. | phase-1I:L13-L16 |
| P1I-E004 | E/DEP | Pre-read: knowledge.txt §§5-8, 10-12; architecture-overview.txt §4 readiness_check/ReadinessStatus. | phase-1I:L18-L23 |
| P1I-E005 | E/CAP | 1I.1 `EmbeddingCircuitBreaker` states CLOSED, OPEN, HALF_OPEN; independent instance per ONNX model (GraphCodeBERT, Granite, cross-encoder, SPLADE). | phase-1I:L28-L36 |
| P1I-E006 | E/CFG | Breaker config: failure_threshold=3 (transient), failure_window=60s, half_open_timeout_base=30s; all time via ClockProvider. | phase-1I:L37-L38 |
| P1I-E007 | E/FAIL | Failure classification via error-type enum (not string match): TRANSIENT (timeout/temp OOM) counts to 3; PERMANENT (segfault/missing model/invalid ONNX) opens after 1. | phase-1I:L39-L42, L66-L67 |
| P1I-E008 | E/RETRY | Exponential backoff OPEN timeouts: 30s, 60s, 120s cap; deterministic jitter from hash(model_id) prevents thundering herd. | phase-1I:L43-L46, L71-L73 |
| P1I-E009 | E/STATE | Breaker state is NOT checkpointed; resets CLOSED on restart. | phase-1I:L47-L47, L79-L79 |
| P1I-E010 | E/FLOW | OPEN skips model and returns fallback immediately. HALF_OPEN allows limited probes; needs 2 consecutive successes for CLOSED; any failure → OPEN with reset timer and increased backoff. | phase-1I:L48-L51, L69-L70 |
| P1I-E011 | E/RECOVERY | HALF_OPEN probes create a fresh ONNX InferenceSession (never reuse potentially corrupted session; cites ORT issues #20043/#20354). | phase-1I:L52-L53, L74-L74 |
| P1I-E012 | E/TEST | 1I.1 acceptance includes independent per-model breakers, 3-in-60s open, permanent-1 open, spread failures stay CLOSED, 2-success HALF_OPEN, jitter distinctness, fresh session. | phase-1I:L65-L75 |
| P1I-E013 | E/STATE | 1I.2 `SearchMode` enum: HYBRID, SPARSE_ONLY, BM25_ONLY; any layer failure drops to next mode. | phase-1I:L87-L94 |
| P1I-E014 | E/FLOW | Fallback chain: GraphCodeBERT fail→TEXT-vector-only (Granite); Granite fail→CODE-vector-only; both→SPARSE_ONLY (BM25+SPLADE); SPLADE fail→BM25_ONLY; cross-encoder fail→skip rerank use vector scores. | phase-1I:L95-L100 |
| P1I-E015 | E/RECOVERY | Mode auto-restores when circuit-breaker HALF_OPEN probe succeeds and cascade re-evaluates; polling driven by half_open_timeout (30s), not a separate timer. | phase-1I:L101-L104, L117-L117 |
| P1I-E016 | E/EVENT | Every mode change logs SSB action_type=`knowledge_degradation`. | phase-1I:L105-L105, L118-L118 |
| P1I-E017 | E/FAIL | Never total outage while BM25 is up: BM25_ONLY always reachable (tantivy has no ONNX dependency). Kill embedding → BM25 results within 100ms of detection. | phase-1I:L116-L119 |
| P1I-E018 | E/CFG | If SPLADE disabled (conditional 1H), SPARSE_ONLY collapses to BM25_ONLY and cascade skips straight. | phase-1I:L114-L114 |
| P1I-E019 | E/CAP | 1I.3 `QueryValidator` runs before any embedding call. | phase-1I:L127-L131 |
| P1I-E020 | E/FAIL | Reject empty (zero tokens after whitespace split), >10,000 chars, null bytes, control characters; accept newlines; latency <1ms. | phase-1I:L132-L135, L144-L151 |
| P1I-E021 | E/FLOW | 1I.4 startup sequence: load ONNX models → open SQLite (DKG, vector cache, BM25) → canary embed("fn main()")+BM25 search("main")+rerank within 5s → drift check → report ReadinessStatus. | phase-1I:L159-L176 |
| P1I-E022 | E/STATE | `ReadinessStatus`: READY (all canary pass); DEGRADED (ONNX failed, BM25 ok → BM25_ONLY invariant); UNAVAILABLE (BM25 also failed → total outage). | phase-1I:L177-L181, L204-L205 |
| P1I-E023 | E/RETRY | Startup: after 3 retries enter DEGRADED BM25_ONLY + WARNING; if BM25 fails enter UNAVAILABLE + ERROR. | phase-1I:L182-L183, L198-L199 |
| P1I-E024 | E/INT | Orchestrator FSM: IDLE→PLANNING blocked until READY or DEGRADED; UNAVAILABLE blocks all FSM transitions. | phase-1I:L184-L185, L200-L200 |
| P1I-E025 | E/OBS | Canary drift: cosine_sim(embed("fn main()"), reference) < 0.85 → DEGRADED. Reference stored in `index_version.toml` as `canary_reference_vec`; first startup generates and persists reference after success. | phase-1I:L169-L173, L192-L203 |
| P1I-E026 | E/CONC | 1I.5: `tokio::Semaphore(1)` around cross-encoder ONNX session; bounded channel capacity 4; max 5 in-flight; 6th returns unranked results immediately. | phase-1I:L215-L223, L233-L236 |
| P1I-E027 | E/CAP | Batching: if multiple requests queued, merge candidate sets and rerank once; each caller receives only its original candidates (reranked). | phase-1I:L224-L224, L237-L238 |
| P1I-E028 | E/DATA | 1I.6 `QueryTrace` fields: query_id UUID, stages Vec StageTrace, total_latency_ms, mode SearchMode, result_count. | phase-1I:L247-L253 |
| P1I-E029 | E/DATA | `StageTrace` fields: stage, latency_ms, input_count, output_count, error Option, circuit_breaker_state Option. | phase-1I:L254-L254 |
| P1I-E030 | E/OBS | Nine stages: bm25, splade, code_vector, text_vector, pagerank, catrag, arag, rerank, token_fit. Emit via tracing at DEBUG; NOT per-query in SSB. | phase-1I:L255-L258, L268-L271 |
| P1I-E031 | E/OBS | Aggregated per-session p50/p99 per stage stored in SSB at session end only. | phase-1I:L257-L258 |
| P1I-E032 | E/DATA | 1I.7 `index_version.toml` at `.elai_cc/projects/{id}/index_version.toml` schema: version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format, created_at. | phase-1I:L281-L287 |
| P1I-E033 | E/FLOW | On upgrade compare versions → full rebuild or BM25-only rebuild or reuse; archive old index to `index_backup_{version}/` before rebuild. | phase-1I:L288-L289, L303-L303 |
| P1I-E034 | E/SURFACE | User notified before rebuild (not silent). | phase-1I:L290-L290, L302-L302 |
| P1I-E035 | E/FAIL | Missing version file creates default version=1; corrupted TOML → full rebuild + warning; archive failure aborts rebuild. | phase-1I:L296-L301 |
| P1I-E036 | E/TEST | VERIFY: knowledge crate compiles; no 1G regressions; project isolation holds under BM25_ONLY; vector cache invalidation holds under SPARSE_ONLY/BM25_ONLY. Isolation owned by 1A/1B/1C. | phase-1I:L311-L320 |
| P1I-E037 | E/OBS | MEASURE targets: degradation latency <100ms; canary <5s; breaker recovery <35s typical; semaphore p99 queue wait; QueryTrace overhead <1ms; index version check <10ms. | phase-1I:L322-L328 |
| P1I-E038 | E/TEST | EXIT: full query pipeline on elai_cc codebase; survives ONNX failure; breaker 3 states; cascade to BM25_ONLY auto-recovers; readiness gates FSM; semaphore prevents concurrent ONNX corruption; QueryTrace every query; index migration preserves backup. | phase-1I:L330-L338 |
| P1I-E039 | E/OBS | Phase 1 overall measurements: index build, query latency per layer, token savings, dual-vector accuracy, reranker precision, SPLADE recall delta, degradation correctness, startup readiness latency. | phase-1I:L340-L343 |
| P1I-I001 | I/EVENT | knowledge_degradation event is the operator-visible degradation signal; dashboard consumers are not specified in this phase file. | phase-1I:L105-L105, L118-L118 |
| P1I-A001 | A/CFG | Exact permanent-vs-transient error enum variants beyond the listed examples are not exhaustively named. | phase-1I:L39-L42 |
| P1I-A002 | A/CONC | "Limited probe calls" in HALF_OPEN does not specify concurrency/permit count beyond "allow limited probes". | phase-1I:L49-L51 |
| P1I-A003 | A/OBS | Session-end SSB aggregation schema for p50/p99 stage stats is not named as a payload type. | phase-1I:L257-L258 |
| P1I-C001 | C/DEP | Source blocks on missing 1G→1I edge in implementation-guide while prerequisites claim both 1E and 1G; guide is incomplete until ACTION is done. | phase-1I:L1-L9 |

Category coverage: breakers ✓; cascade ✓; validation ✓; readiness/FSM gate ✓; semaphore ✓; traces ✓; index version UX notification ✓; measures/exit ✓. No privacy/compliance; no formal idempotency key.

### phase-2D — CSQ and inline feasibility

Source: `docs/specs/guide/steps/phase-2D.txt` (428/428 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2D-E001 | E/STATUS | PHASE 2D: CSQ and inline feasibility. Prerequisite: Phase 2C complete. Completes reasoning engine 2A–2D. | phase-2D:L1-L9, L409-L410 |
| P2D-E002 | E/CAP | Goal: CSQ on functions touching files/data/network/auth; inline feasibility during EXECUTING (max 2 GRC calls). | phase-2D:L6-L8 |
| P2D-E003 | E/DEP | Pre-read: reasoning.txt §§6,7,8,10[2D],11[2D]; orchestrator.txt §8; architecture-overview §4. | phase-2D:L12-L19 |
| P2D-E004 | E/FLOW | D1: CSQ trigger via function signature/pattern match on Path/File/TcpStream/Money/AuthToken or std::fs/std::net; must work WITHOUT DKG dependency (reasoning depends on core+providers+tools). Orchestrator injects sensitive signatures at call time. | phase-2D:L24-L38 |
| P2D-E005 | E/STATE | D2 severity: CRITICAL (data_loss, security_breach, financial_error, auth_bypass) guard MANDATORY; WARNING (incorrect_output, performance_degradation, stale_cache) RECOMMENDED. Critical without guard = CSQ failure blocks AUDITING; warning without guard is logged only. | phase-2D:L40-L50, L401-L403 |
| P2D-E006 | E/CFG | D3: Inline feasibility uses dedicated `InlineFeasibilityState` counter separate from per-topic GRC budget (cross-topic scope). Methods: web_search, grep, api_registry_check (spec); receiver is &mut self. | phase-2D:L52-L65 |
| P2D-E007 | E/FLOW | D4 outcomes: PASS proceed; FAIL use original plan approach; BLOCKED emit ReplanRequest(MAJOR) → EXECUTING→PLANNING requires human approval in AUTONOMOUS and INTERACTIVE. | phase-2D:L67-L75, L404-L405 |
| P2D-E008 | E/DATA | Types: CsqCheck {assumption, consequence, severity, guard_added}; CsqSeverity {Critical, Warning}; CsqResult {function_id, checks} with computed `passed()` (no stored field). | phase-2D:L77-L84, L112-L120 |
| P2D-E009 | E/DATA | FeasibilityMethod enum {SourceMatch} now; future WebSearch/Grep/ApiRegistryCheck in phase 4F. InlineFeasibilityResult {claim, method, outcome}; FeasibilityOutcome {Pass, Fail, Blocked}. | phase-2D:L85-L91, L127-L135 |
| P2D-E010 | E/STATE | InlineFeasibilityState {checks_used, max_checks=2 hardcoded}; is_exhausted, record_check (saturating_add), reset (orchestrator between phases). Exhausted treats new claims as Fail / BudgetExhausted. | phase-2D:L92-L95, L121-L126, L163-L165 |
| P2D-E011 | E/CAP | GrcEngine methods: `generate_csq(function_id, sensitive_ops)`; `check_inline_feasibility(claim, sources)` mut self. Stubs todo!() until 2D.4. BasicGrcEngine holds InlineFeasibilityState. | phase-2D:L136-L144 |
| P2D-E012 | E/LIMIT | 2D.2: no new reasoning deps beyond core, providers, tools, thiserror, serde. | phase-2D:L177-L190 |
| P2D-E013 | E/TEST | 2D.3 TDD file `reasoning/tests/csq_tests.rs` with ≥10 tests T1–T10 all FAIL against stubs; zero timing-dependent tests. | phase-2D:L192-L274 |
| P2D-E014 | E/TEST | T1 file-function CSQ ≥1 Critical; T2 Critical no guard fails; T3 Critical with guard passes; T4 Warning no guard still passes; T5 empty sensitive_ops empty checks. | phase-2D:L201-L223 |
| P2D-E015 | E/TEST | T6 known claim Pass; T7 unknown Fail; T8 max two calls then BudgetExhausted; T9 CONTRADICTS_PLAN: prefix → Blocked (ReplanRequest emission is orchestrator 3A+); T10 reset after exhaust. | phase-2D:L225-L257 |
| P2D-E016 | E/FLOW | I1 generate_csq: empty ops → empty checks; else per op Critical if case-insensitive match patterns (fs::, file, write, delete, remove, net::, tcpstream, money, payment, auth, token, credential, password) else Warning; guard_added=false. | phase-2D:L284-L304 |
| P2D-E017 | E/FLOW | I2 check_inline_feasibility: exhaust → Err BudgetExhausted topic=inline_feasibility; source substring/exact match → Pass; not found → Fail; CONTRADICTS_PLAN: prefix → Blocked (full semantic deferred 4F). Then record_check. | phase-2D:L306-L323 |
| P2D-E018 | E/DET | CsqResult.passed computed: all Critical have guard_added. Adversarial: case-insensitive patterns; meaningful multi-word claim match not single common words; different ops not Blocked. | phase-2D:L325-L353 |
| P2D-E019 | E/OBS | 2D.5 metrics in `.elai_cc/metrics/phase_2D.toml`: csq_overhead_us <1000, inline_check_overhead_us <2000, false_positive_rate == 0.0; update architecture-overview GrcEngine listing. | phase-2D:L360-L397 |
| P2D-E020 | E/EVIDENCE | Grounding: CSQ/guard/inline max-2/PASS-FAIL-BLOCKED/per-agent CSQ config from reasoning.txt; severity classification VERIFIED from dangerous-consequence distinction; detection via patterns VERIFIED vs ast-grep; overhead targets UNGROUNDED; CONTRADICTS_PLAN prefix UNGROUNDED placeholder for 4F. | phase-2D:L413-L428 |
| P2D-E021 | E/DEP | Per-agent CSQ config: executor=mandatory, auditor=mandatory, compiler=none (reasoning.txt §7). | phase-2D:L421-L421 |
| P2D-E022 | E/DEFER | Actual ReplanRequest emission is orchestrator responsibility phase 3A+. Full semantic contradiction detection deferred to 4F FEASIBILITY_PROBE. | phase-2D:L250-L252, L318-L321, L428-L428 |
| P2D-E023 | E/LIMIT | Zero todo!/unimplemented!/unwrap() in CSQ and inline feasibility production code at exit. | phase-2D:L334-L338, L407-L407 |
| P2D-I001 | I/SURFACE | Human approval for MAJOR replan is an interactive gate surface owned by orchestrator, not implemented in reasoning crate in this phase. | phase-2D:L70-L73, L250-L252 |
| P2D-A001 | A/FLOW | Design D3 lists web_search/grep/api_registry_check methods while Tier-1 type only implements SourceMatch; call-path for those methods is unspecified until 4F. | phase-2D:L58-L58, L85-L87, L132-L132 |
| P2D-A002 | A/TEST | T4 writes `result.passed == true` field access while type design forbids a stored field (computed method). | phase-2D:L83-L84, L216-L218 |
| P2D-A003 | A/DET | Substring match "meaningful content (>3 words)" threshold is adversarial guidance, not a named constant/API. | phase-2D:L345-L347 |
| P2D-C001 | C/DATA | D3 method list (web_search/grep/api_registry) conflicts with FeasibilityMethod::{SourceMatch} only + 4F future variants. | phase-2D:L58-L58, L85-L87, L132-L132 |

Category coverage: CSQ types/flow ✓; inline budget ✓; FSM replan signal ✓; metrics ✓; deferrals ✓. No dedicated UI panel in 2D; interactive approval is orchestrator-owned.

### phase-3B — Shared State Board

Source: `docs/specs/guide/steps/phase-3B.txt` (492/492 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3B-E001 | E/STATUS | PHASE 3B: SSB. Prerequisite: Phase 3A complete (9 states, evidence gates, INTERACTIVE gating). | phase-3B:L1-L2 |
| P3B-E002 | E/CAP | Goal: inter-agent communication via SQLite WAL single-writer; append-only within session; backbone for tool results, findings, claims, checkpoints, phase transitions. | phase-3B:L6-L12 |
| P3B-E003 | E/STATE | One Ssb instance permanently binds one (project_id, session_id); multi-session needs new instance (anti-forgery). | phase-3B:L14-L16, L491-L491 |
| P3B-E004 | E/DEP | Downstream consumers: 1C vector cache invalidation, 3C agents, 3D BSP, 3E context budget. | phase-3B:L10-L12 |
| P3B-E005 | E/DEP | Pre-read: orchestrator.txt §§3,5,6,7A; core.txt ClockProvider L68-82; audit-results BLOCK 1/5/8; phase4 WARN-6; risk R10; architecture-overview SSB refs. | phase-3B:L18-L29 |
| P3B-E006 | E/LIMIT | R10: ~50K inserts/sec claimed; 80K achievable with optimized pragmas; agent rate 1–10/s gives 50x headroom; fallback batch transaction. | phase-3B:L32-L35, L451-L453 |
| P3B-E007 | E/FAIL | R-3B-1: WAL autocheckpoint 1000; manual checkpoint only between supersteps (3D). R-3B-2: disable r2d2 idle_timeout/max_lifetime on read pool. R-3B-3: CHECK changes need user_version migration (drop+recreate session-scoped OK). | phase-3B:L36-L49 |
| P3B-E008 | E/CAP | Deliverables D1 schema/WAL/pool/migration; D2 conflict+append-only+events; D3 typed actions+traces. | phase-3B:L51-L54 |
| P3B-E009 | E/DATA | Schema PRAGMA user_version=1; table ssb(seq PK AUTOINCREMENT, project_id, session_id, agent_id, superstep DEFAULT 0, action_type CHECK 17 values, payload BLOB, created_at INTEGER). | phase-3B:L56-L74 |
| P3B-E010 | E/EVENT | 17 action_types: file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, phase_complete, memory_monitoring. | phase-3B:L65-L71, L96-L100 |
| P3B-E011 | E/DATA | Indexes: project, session, superstep, conflict(project_id,superstep). Junction ssb_files(seq,file_path) with path and seq indexes. | phase-3B:L75-L85 |
| P3B-E012 | E/EVIDENCE | orchestrator.txt §3 synopsis lists only 9 action_types; full 17 from cross-refs is CANONICAL in this schema. | phase-3B:L87-L94, L492-L492 |
| P3B-E013 | E/DATA | Rust SsbActionType enum serde rename_all=snake_case; typed SsbPayload per type validated at write. | phase-3B:L96-L100 |
| P3B-E014 | E/CAP | Conflict SQL: JOIN ssb_files→ssb; filter file_path IN rarray, project_id, session_id != self, superstep; return distinct agent_ids. | phase-3B:L102-L115, L270-L274 |
| P3B-E015 | E/CFG | WAL pragmas order: journal_mode=WAL; synchronous=NORMAL; wal_autocheckpoint=1000; busy_timeout=5000; cache_size=-64000 (64MB); foreign_keys=ON; temp_store=MEMORY. | phase-3B:L117-L127 |
| P3B-E016 | E/CFG | foreign_keys is per-connection; applied to writer and every read-pool checkout via r2d2 CustomizeConnection. | phase-3B:L129-L132, L157-L158 |
| P3B-E017 | E/CAP | 3B.1 module ssb.rs: SsbActionType, SsbEntry (payload Arc<[u8]>), SsbFileEntry, SsbConfig, SsbPragmaApplier, Ssb::new with migration/user_version, read pool size max(num_cpus*2,4) idle/lifetime None, mpsc unbounded writer, broadcast capacity 1024. | phase-3B:L137-L178 |
| P3B-E018 | E/INT | SsbReader queries: by_session, by_file, by_type, by_superstep, query_conflict, last_seq; subscribe(); shutdown(). | phase-3B:L170-L178 |
| P3B-E019 | E/FLOW | 3B.2 write path: BEGIN IMMEDIATE → INSERT ssb → INSERT ssb_files → COMMIT → broadcast; oneshot returns seq; project_id/session_id injected at construction; created_at from ClockProvider never SystemTime. | phase-3B:L208-L228 |
| P3B-E020 | E/INT | WARN-6 IPC sync: SSB write MUST complete (.await seq) before ToolResult dispatch to agent. | phase-3B:L226-L228, L459-L459 |
| P3B-E021 | E/RECOVERY | shutdown drops sender, drains loop, write-after-shutdown Err(SsbShutdown); idempotent; no hung oneshots. | phase-3B:L229-L234, L258-L259 |
| P3B-E022 | E/LIMIT | Append-only: no UPDATE/DELETE methods; Connection not exposed; debug update_hook panics on UPDATE/DELETE. | phase-3B:L275-L280, L309-L309 |
| P3B-E023 | E/DATA | 3B.4 typed payloads: FileWrite/Read, ToolResult, TestOutcome, Finding, Claim, Checkpoint, ModelSwap, PhaseTransition, KnowledgeDegradation, IsolationMetric, PlanUpdate, ReplanRequest, StubReference, ReasoningTrace, PhaseComplete, MemoryMonitoring (17 structs). | phase-3B:L317-L338 |
| P3B-E024 | E/EVENT | PhaseCompletePayload written BEFORE model unload with WAL fsync; crash before → re-execute phase; after → skip. | phase-3B:L333-L337 |
| P3B-E025 | E/INT | SsbTraceable trait in orchestrator/; reasoning/tools never call SSB; orchestrator writes after ThinkStep IPC or in-process GRC callbacks. Serialization msgpack rmp-serde for disk. | phase-3B:L342-L352, L379-L380 |
| P3B-E026 | E/FAIL | Payload type mismatch returns Err not panic; payload >1MB rejected; disk-full propagates error. | phase-3B:L339-L341, L364-L366, L416-L417 |
| P3B-E027 | E/EVENT | Broadcast after commit only; slow subscribers lag/logged; Arc payload O(1) clone; capacity 1024. | phase-3B:L224-L224, L398-L400, L445-L445 |
| P3B-E028 | E/TEST | Unit/property/adversarial/determinism suites: 10K monotonic seq, self-not-conflict, cross-project empty, symmetric conflict, seed+clock byte-identical DB, SQLITE_BUSY storm proves channel necessity. | phase-3B:L383-L429 |
| P3B-E029 | E/OBS | MEASURE: write >10K inserts/s; read p99 <1ms at 10K; conflict <5ms for 100 paths vs 100K; pool 0 timeouts 60s; RSS <50MB at 100K; broadcast overhead <1% with 3 subscribers. | phase-3B:L431-L438 |
| P3B-E030 | E/TEST | EXIT: typed 17 payloads, conflict JOIN+project isolation, append-only, event notification, ClockProvider determinism, user_version migration, graceful drain, all tests pass. | phase-3B:L440-L449 |
| P3B-E031 | E/EVIDENCE | Audit fixes list BLOCK1–5 and WARN1–6 including reasoning_trace/phase_complete/memory_monitoring additions, JOIN conflict, ClockProvider rename, Arc payload, CustomizeConnection, shutdown, user_version, rarray, SsbTraceable placement. | phase-3B:L477-L492 |
| P3B-I001 | I/DEP | rusqlite bundled+array and r2d2_sqlite are required Windows/deps choices for this phase. | phase-3B:L141-L147, L462-L463 |
| P3B-A001 | A/DATA | Step text says "SsbPayload enum (15 variants)" while schema and audit claim 17 action_types/payloads. | phase-3B:L339-L341, L97-L100, L375-L375 |
| P3B-A002 | A/CFG | CURRENT_VERSION constant numeric value beyond initial user_version=1 is not specified for future migrations. | phase-3B:L57-L57, L161-L163 |
| P3B-C001 | C/DATA | Internal 15-vs-17 payload count wording conflicts with the canonical 17-type CHECK list. | phase-3B:L65-L71, L339-L341 |

Category coverage: schema/pragmas ✓; write/read/conflict ✓; typed events ✓; shutdown ✓; measures ✓. No direct user window; event bus is backend surface for later dashboards.

### phase-4D — Pipeline Factory L1 / FlashResearch

Source: `docs/specs/guide/steps/phase-4D.txt` (1260/1260 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4D-E001 | E/STATUS | PHASE 4D Pipeline Factory L1 (designer + FlashResearch). Prerequisite: Phase 4A Tier C isolation. Status IMPROVED 2026-04-06. | phase-4D:L1-L3 |
| P4D-E002 | E/CAP | Goal: Research Agent designs pipelines from tasks via tree-structured GRC-grounded research; produces PipelineSpec + QualitySpec via ResearchReport. Paper arxiv 2510.05145 FlashResearch. | phase-4D:L7-L10 |
| P4D-E003 | E/DEP | Pre-read: pipeline-factory.txt §§1-9; reasoning GRC + IPC verification; external-analysis FlashResearch; flashresearch-interface.md. | phase-4D:L12-L16 |
| P4D-E004 | E/CFG | Research Agent role: cloud preferred or Gemma4; GRC budget 10/1200. | phase-4D:L27-L27, L119-L119 |
| P4D-E005 | E/CFG | Designer role distinct AgentRole::Designer (not Executor); model reuses executor Gemma 4 31B; GRC 8/1200; required_steps Observe/Hypothesize/PlanAction/Verify/Converge; plan-only role_prompt for TOML PipelineSpec. | phase-4D:L28-L36 |
| P4D-E006 | E/DATA | SystemConstraints input: model_max_context, available_for_work, avg_file_tokens, avg_test_tokens. Defaults when index <10 files from elai_cc.toml [planning.sizing] default_avg_file_tokens=150, default_avg_test_tokens=80; else knowledge_engine.index_stats(). | phase-4D:L37-L45 |
| P4D-E007 | E/DATA | ResearchTree {plan, state, root, breadth_factor, depth_limit, max_iterations, phi_min, psi_min, speculative_depth, time_budget}. | phase-4D:L46-L49 |
| P4D-E008 | E/DATA | ResearchNode {question, parent, children, phi, psi, status, info_gain, iterations}; NodeId newtype u64; NodeStatus {Pending, Active, Converged, Pruned}. | phase-4D:L50-L53 |
| P4D-E009 | E/DATA | ResearchPlan {query, complexity, sub_questions}; SharedResearchState {nodes HashMap, next_id}; QueryComplexity {Simple, Moderate, Complex}. | phase-4D:L54-L56 |
| P4D-E010 | E/DATA | Supporting: ResearchTreeSnapshot, ConvergenceStatus {Converged, Partial, Failed}, Alternative, Contradiction, EvidenceLevel {Benchmark(1.0), OfficialDocs(0.8), BlogPost(0.6), Training(0.4)}. | phase-4D:L57-L63 |
| P4D-E011 | E/DATA | QualityMetric {name, target, unit, method, phase, hard_fail}; MediaSpec {output_name, format, resolution, ssim_threshold, vlm_check, output_type}; OutputType {Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom}; PerfSpec. | phase-4D:L64-L69 |
| P4D-E012 | E/DATA | ResearchReport + SubQuestionResult with sources/confidence/phi/psi/depth/alternatives/contradictions; confidence = source_count * avg_quality * agreement_ratio. | phase-4D:L70-L78 |
| P4D-E013 | E/DATA | SourceCitation {url, extracted_quote, quality_tier}; SourceQuality 5-tier Tier1(1.0)…Tier5(0.2); domain lookup no LLM; configurable [research.trusted_domains]. | phase-4D:L79-L83 |
| P4D-E014 | E/SEC | Research Agent Tier C isolated process: network web-search only via IPC proxy; GRC via IPC GroundingRequest; no FS to .elai_cc/ or project source; model via IPC LlmProvider. | phase-4D:L86-L90, L181-L182 |
| P4D-E015 | E/CFG | Adaptive complexity: Simple breadth2/depth3/φ0.8/ψ0.7; Moderate 3/4/0.7/0.6; Complex 4/5/0.6/0.5 (no hardcoded single thresholds). | phase-4D:L91-L95 |
| P4D-E016 | E/CFG | information_gain_threshold range [0.1, 0.9]; below for 3 consecutive iterations → prune node. | phase-4D:L96-L97, L273-L273 |
| P4D-E017 | E/CONC | Parallel root children for independent sub-questions; local models sequential per level; cloud parallel with rate-limit Dynamic Fan-Out; pruned branches free resources immediately. | phase-4D:L98-L102, L267-L270 |
| P4D-E018 | E/EVENT | SSB progress stubs action_type=`research_step` per iteration. | phase-4D:L85-L85, L183-L183, L271-L271 |
| P4D-E019 | E/TEST | 4D.3 ≥14 contract invariants (depth/breadth bounds, acyclic, dual-threshold convergence, deterministic quality, contradictions surfaced, speculative bounds, source tracking); ≥3 integration; ≥3 proptest; all FAIL red phase. | phase-4D:L158-L204 |
| P4D-E020 | E/TEST | 4D.4 ≥8 adversarial: FS escape, direct network, GRC exhaustion, depth/breadth bombs, speculative prune, quality gaming, empty task, ALL-TRAINING HIGH_RISK, contradictions. | phase-4D:L206-L232 |
| P4D-E021 | E/FLOW | 4D.5 per-node loop: FORMULATE → SEARCH (top N default 10) → EXTRACT (500-token sample filter) → REFLECT (phi/psi/contradictions/info_gain) → CONVERGE/EXPAND/ITERATE. | phase-4D:L239-L247 |
| P4D-E022 | E/FLOW | Speculative execution cloud-only: trigger phi>0.4; top speculative_depth children before parent converges; parent direction change cancels children; local speculative_depth forced 0; cost bound speculative_depth*breadth_factor. | phase-4D:L252-L257 |
| P4D-E023 | E/STATE | Dual-threshold convergence: phi≥phi_min AND psi≥psi_min. Node lifecycle Pending→Active→(Converged\|Pruned). Acyclic parent-chain check. | phase-4D:L248-L260 |
| P4D-E024 | E/LIMIT | L1 PipelineSpec may ONLY reference alternatives from ResearchReport; QualityMetric must have automated check method (unmeasurable → REJECT). | phase-4D:L264-L266, L326-L327 |
| P4D-E025 | E/OBS | 4D.6 metrics phase_4D.toml: grounding_distribution, spec_token_count, depth/breadth utilization, convergence p50/p95, speculative_hit_rate, source_tier_distribution, time_budget_ratio, ssb_event_count, info_gain_trend. Accept: zero UNCERTAIN; time_budget_ratio≤1.1. | phase-4D:L293-L313 |
| P4D-E026 | E/DATA | Required docs: workflow/reasoning/phase-4D-decisions.md and phase-4D-grounding.md before EXECUTING. | phase-4D:L316-L318 |
| P4D-E027 | E/INT | EXIT: RuntimeClient must wire ssb, fsm, clock; no None after construction. INTERACTIVE mode visibility via SSB progress. | phase-4D:L320-L329 |
| P4D-E028 | E/FLOW | Recursive extension: FlashResearch at Idea→Concept→Phase→Step→BabyStep; orchestrator TreeNavigator drives; LLM never controls flow. | phase-4D:L332-L344 |
| P4D-E029 | E/STATE | Level 0 IDEA → ConceptDecomposition with feasibility scores; gate drop concepts under threshold. Level 1 CONCEPT → phases/frameworks/packages. Level 2 PHASE → steps with grounded paths. Level 3 STEP → BabyStepSpec 8-block each <2K tokens. | phase-4D:L346-L368 |
| P4D-E030 | E/DATA | Decomposition types: ConceptDecomposition, ConceptNode, PhaseNode, StepNode, BabyStepSpec (+ grounding/pre_gate/task/tdd/deterministic_check/acceptance/integration/exit). Newtypes ConceptId, BabyStepId. | phase-4D:L370-L419, L510-L513 |
| P4D-E031 | E/DATA | BabyStepGrounding max 500 tokens overall_context_summary; TddSpec write_before_code always true; AcceptanceFailAction {BubbleUp, ResearchAlternative, Replan}; max_retry_alternatives default 3. | phase-4D:L421-L464 |
| P4D-E032 | E/DATA | TechStackDecision + DecisionSource {Research, UserChoice, AutoSelected}; PackageRef pinned verified via API_REGISTRY_CHECK; ResearchCache BTreeMap LRU max_entries + ttl_seconds + ClockProvider cached_at. | phase-4D:L477-L508 |
| P4D-E033 | E/FLOW | research_decompose: Level0 Complex research; Level1 Moderate with cache; Interactive present_options vs Autonomous auto_select by evidence_level; Level2 Simple + verify_packages; Level3 baby steps with compressed grounding. | phase-4D:L515-L562 |
| P4D-E034 | E/SURFACE | INTERACTIVE mode presents alternatives with evidence for user pick; AUTONOMOUS selects highest evidence_level * source_quality. | phase-4D:L531-L535, L645-L647 |
| P4D-E035 | E/SEC | Package verification API_REGISTRY_CHECK is anti-hallucination gate; nonexistent/wrong version retries; LLM cannot invent packages. | phase-4D:L568-L570 |
| P4D-E036 | E/TEST | 4D.7–4D.10: recursive types skeleton, ≥13 FAIL tests (levels, cache, TTL, package verify, grounding <500, Interactive/Autonomous, cross-deps, proptest), implement, measure cache_hit_rate/baby_step metrics into phase_4D.toml. | phase-4D:L574-L678 |
| P4D-E037 | E/EVENT | SSB emit research_decompose_level per level transition. | phase-4D:L650-L650 |
| P4D-E038 | E/LIMIT | Design principles: API retrieval not browser (2–3x cost); ManuSearch deterministic orchestration; A-RAG progressive disclosure ~90% token savings; evidence grows monotonically; hybrid tree preferred; source quality never LLM-scored. | phase-4D:L703-L714 |
| P4D-E039 | E/CAP | 4D.11 new crate web_search; SearXNG Docker start-searxng.sh port 8888; custom settings JSON format result_count=20 no telemetry. | phase-4D:L717-L730 |
| P4D-E040 | E/CAP | SearxClient {base_url default localhost:8888, timeout 10s, max_results 20}; search GET /search format=json; SearchResult {title,url,snippet,engine,score,parsed_domain}; health_check GET / 2s. | phase-4D:L731-L746 |
| P4D-E041 | E/CFG | SearXNG rate limit max 1 req/s configurable; 1 retry exponential backoff on 5xx; lives in orchestrator, IPC-proxied. | phase-4D:L747-L752 |
| P4D-E042 | E/CAP | 4D.12 WebFetcher: timeout 15s, max_redirects 5, UA ELAI-Research/0.1, max_body 2MB, per-domain rate max 2 req/s. | phase-4D:L782-L789 |
| P4D-E043 | E/CAP | ContentExtractor trait + ReadabilityExtractor (readability-rust; fallback strip tags); robots.txt respect RFC9309 with 1h cache; 404 robots = allow; Empty page_type on JS-only. | phase-4D:L791-L826 |
| P4D-E044 | E/DEFER | rs-trafilatura higher quality alternative evaluated in 4D.18 if extraction quality insufficient. | phase-4D:L828-L831, L1236-L1239 |
| P4D-E045 | E/DATA | 4D.13 SourceQualityTable + SourceTier; ≥200 domains; exact domain then subdomain rules (*.gov Tier1, *.edu Tier3); default Tier5; TOML [research.trusted_domains] + [research.trusted_subdomain_rules]. | phase-4D:L836-L891 |
| P4D-E046 | E/CAP | 4D.14 WebContentIndex/WebChunk for prose A-RAG; sentence split [.!?\n]+ min 10 chars; ~500 token chunks; KeywordSearch score count*len; SemanticSearch 3x oversample; ChunkRead with ContextTracker; progressive snippets-first. | phase-4D:L899-L959 |
| P4D-E047 | E/CFG | Embedding strategy: CPU sentence-transformers if no GPU model; sidecar /v1/embed if running; else keyword-only. | phase-4D:L930-L933 |
| P4D-E048 | E/CFG | 4D.15 [research.vram_tiers] six tiers 8gb/12gb/12gb_7b/16gb/24gb/48gb with model, quant, context_ceiling, research_context_budget, chunk_size, breadth/depth, phi/psi, embedding_device, max_sources, max_concurrent_fetches; only 48gb has speculative_depth=2. | phase-4D:L968-L1057 |
| P4D-E049 | E/SURFACE | ResearchTierConfig auto-detect GPU VRAM (nvidia-smi/CUDA) or manual `--vram-tier` flag; detect_vram_tier rounds DOWN; no-GPU falls back 8gb. | phase-4D:L1059-L1079 |
| P4D-E050 | E/CAP | 4D.16 ResearchBudgetManager can_admit; prune_to_budget drops lowest tier then lowest phi, never Tier1; compress_evidence first3+last sentence; token approx 1tok~4bytes overcount 10%. | phase-4D:L1087-L1140 |
| P4D-E051 | E/CAP | 4D.17 WebResearchOrchestrator wires SearxClient, WebFetcher, SourceQualityTable, WebContentIndex, ResearchBudgetManager, ResearchTierConfig. | phase-4D:L1143-L1155 |
| P4D-E052 | E/FLOW | Wired loop: FORMULATE IPC → SEARCH → QUALITY GATE Tier1-3 first → FETCH/EXTRACT parallel ≤max_concurrent → INDEX → A-RAG retrieve → REFLECT IPC → CONVERGE. | phase-4D:L1156-L1175 |
| P4D-E053 | E/INT | IPC messages: SearchRequest, FetchRequest, EmbedRequest, InferenceRequest; agent never direct network. | phase-4D:L1176-L1182 |
| P4D-E054 | E/EVENT | SSB emit web_research_step per node iteration. | phase-4D:L1183-L1183 |
| P4D-E055 | E/FAIL | SearXNG down → partial result + error note not hang; >80% Tier5 → confidence LOW; sidecar missing → clear error; rate limiter max 2 concurrent same domain. | phase-4D:L1189-L1198 |
| P4D-E056 | E/OBS | 4D.18 E2E 10 queries (3 simple/4 moderate/3 complex) metrics phase_4D_web_research.toml; 12GB RTX 3060 3B+7B; accept token_budget_util <1.0; ≥30% Tier1-3 on complex; time <5min simple / <15min complex; extraction ≥80% clean. | phase-4D:L1212-L1256 |
| P4D-E057 | E/SURFACE | SSB research_step / research_decompose_level / web_research_step intended for INTERACTIVE mode operator visibility of research progress. | phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183 |
| P4D-I001 | I/SURFACE | present_options Interactive gate is a user-facing selection surface; concrete panel/command wiring is not named in this phase file. | phase-4D:L531-L533, L622-L622 |
| P4D-A001 | A/EVENT | research_step, research_decompose_level, and web_research_step are not in phase-3B's 17-type CHECK list; registration path is unspecified here. | phase-4D:L85-L85, L650-L650, L1183-L1183; phase-3B:L65-L71 |
| P4D-A002 | A/CFG | Several context_ceiling values marked projected vs MEASURED; measurement obligation before treating as hard SLOs is incomplete. | phase-4D:L977-L1047 |
| P4D-A003 | A/SURFACE | Exact Interactive UI for present_options (CLI, dashboard modal, gate API) is not specified. | phase-4D:L531-L533, L645-L647 |
| P4D-A004 | A/DATA | PipelineSpec/QualitySpec full field schemas deferred to pipeline-factory.txt sec 10 beyond the metrics/media/perf structs listed. | phase-4D:L64-L69, L178-L180 |
| P4D-C001 | C/EVENT | Phase-4D introduces SSB action types (research_step, research_decompose_level, web_research_step) absent from phase-3B canonical 17-type schema without a schema-migration step in either file. | phase-4D:L85-L85, L650-L650, L1183-L1183; phase-3B:L65-L71 |

Category coverage: research/designer roles ✓; tree types ✓; recursive decomposition ✓; web stack ✓; VRAM tiers/CLI flag ✓; budget manager ✓; Interactive selection surface ✓; metrics ✓. Privacy/compliance not standalone; robots.txt is the crawl ethics rule.

### Cross-source interactions (PHASE-007)

| Interaction ID | Class | Explicit discovery or bounded interpretation | Source atoms |
|---|---|---|---|
| PHASE-007/IX-001 | E | Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D. | PP0-E001–PP0-E022; P1I-E012; P2D-E013; P3B-E028; P4D-E019 |
| PHASE-007/IX-002 | E | Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path. | P1I-E016; P3B-E010, P3B-E019–P3B-E020 |
| PHASE-007/IX-003 | E | Phase 1I ReadinessStatus READY\|DEGRADED\|UNAVAILABLE gates orchestrator FSM IDLE→PLANNING; UNAVAILABLE blocks all transitions (3A evidence gate consumer). | P1I-E022–P1I-E024 |
| PHASE-007/IX-004 | E | Phase 1I SearchMode cascade and BM25_ONLY degraded mode compose with 1B/1C isolation verified under degradation. | P1I-E013–P1I-E018, P1I-E036 |
| PHASE-007/IX-005 | E | Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate. | P1I-E006; P3B-E019 |
| PHASE-007/IX-006 | E | Phase 2D Blocked outcome signals ReplanRequest(MAJOR) for orchestrator 3A+ human-approval EXECUTING→PLANNING. | P2D-E007, P2D-E022; P2D-E015 |
| PHASE-007/IX-007 | E | Phase 2D CSQ depends on 2C AgentGrcConfig.csq_required and per-role CSQ mandates. | P2D-E001, P2D-E021 |
| PHASE-007/IX-008 | E | Phase 2D inline feasibility SourceMatch is temporary; 4F adds WebSearch/Grep/ApiRegistryCheck variants and deeper contradiction analysis. | P2D-E009, P2D-E022; P2D-C001 |
| PHASE-007/IX-009 | E | Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events. | P3B-E002–P3B-E004; P4D-E018, P4D-E037, P4D-E054 |
| PHASE-007/IX-010 | E | Phase 3B WARN-6 write-before-ToolResult contract binds agent tool IPC to durable journal order. | P3B-E020 |
| PHASE-007/IX-011 | E | Phase 3B PhaseCompletePayload fsync-before-unload durability rule composes with agent lifecycle unload paths. | P3B-E024 |
| PHASE-007/IX-012 | E | Phase 4D Research Agent Tier C isolation reuses Phase 4A process walls and IPC proxy patterns. | P4D-E001, P4D-E014, P4D-E053 |
| PHASE-007/IX-013 | E | Phase 4D Designer GRC config reuses 2C role specialization pattern (distinct from Executor). | P4D-E005 |
| PHASE-007/IX-014 | E | Phase 4D SystemConstraints sizing defaults feed later L2 sizing/feasibility consumers. | P4D-E006 |
| PHASE-007/IX-015 | E | Phase 4D package API_REGISTRY_CHECK reuses 2B verification method family. | P4D-E032, P4D-E035 |
| PHASE-007/IX-016 | E | Phase 4D Interactive present_options is a human-gate surface sibling to 3A InteractiveGate / later dashboard interventions. | P4D-E034, P4D-I001 |
| PHASE-007/IX-017 | C | Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension. | P4D-C001; P3B-E010; P4D-A001 |
| PHASE-007/IX-018 | E | Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer. | P1I-E034; P4D-E057 |
| PHASE-007/IX-019 | I | Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness. | PP0-E025; P3B-E010, P3B-E012 |
| PHASE-007/IX-020 | E | Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml. | PP0-E021; P2D-E019; P4D-E025, P4D-E049, P4D-E056 |

### Ambiguity and conflict register (PHASE-007)

| Register ID | Class | Issue | Source atoms |
|---|---|---|---|
| PHASE-007/AC-001 | C | 1I requires adding missing 1G→1I edge to implementation-guide dep graph before implement. | P1I-C001, P1I-E002 |
| PHASE-007/AC-002 | A | 1I permanent/transient error enum variants incomplete; HALF_OPEN probe concurrency unspecified. | P1I-A001–P1I-A002 |
| PHASE-007/AC-003 | A | 1I session-end p50/p99 SSB aggregation payload shape unnamed. | P1I-A003 |
| PHASE-007/AC-004 | C | 2D design methods web_search/grep/api_registry vs SourceMatch-only type until 4F. | P2D-C001, P2D-A001 |
| PHASE-007/AC-005 | A | 2D T4 uses stored `.passed` field vs computed method design. | P2D-A002 |
| PHASE-007/AC-006 | A | 2D multi-word match threshold not a named constant. | P2D-A003 |
| PHASE-007/AC-007 | C | 3B SsbPayload "15 variants" wording vs 17 action_types. | P3B-C001, P3B-A001 |
| PHASE-007/AC-008 | A | 3B CURRENT_VERSION evolution policy beyond v1 not specified. | P3B-A002 |
| PHASE-007/AC-009 | C | 4D research_step / research_decompose_level / web_research_step absent from 3B 17-type CHECK. | P4D-C001, P4D-A001 |
| PHASE-007/AC-010 | A | 4D Interactive present_options concrete UI surface open. | P4D-A003, P4D-I001 |
| PHASE-007/AC-011 | A | 4D several VRAM context_ceiling values projected not measured. | P4D-A002 |
| PHASE-007/AC-012 | A | pre-phase-0 full cfg-gated test inventory not pre-enumerated. | PP0-A001 |
| PHASE-007/AC-013 | A | pre-phase-0 HookRunner /bin/sh hardcode status left as audit. | PP0-A002 |

### Summary deltas vs MASTER (PHASE-007)

Summary artifact: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. Anchors use `01-phase:` line numbers.

### pre-phase-0 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-007/SD-PP0-001 | **ADD** exact file/line sites for PermissionsExt and set_mode (7 sites / 3 files) and Windows no-op chmod pattern. | 01-phase:L13-L14 | PP0-E006–PP0-E008 |
| PHASE-007/SD-PP0-002 | **ADD** named failing plugin tests and Windows .cmd / cmd.exe shell dispatch requirements. | 01-phase:L15-L16 | PP0-E010–PP0-E013 |
| PHASE-007/SD-PP0-003 | **ADD** named tools failures (path ends_with, chmod, python3→python, env PoisonError) and 10-run adversarial stability. | 01-phase:L17-L18 | PP0-E014–PP0-E019 |
| PHASE-007/SD-PP0-004 | **ADD** exact warning sites, pre_phase_0.toml fields, measurements, grounding table, and WIRING_CONTRACT archive-verify surface. | 01-phase:L19-L22 | PP0-E020–PP0-E025 |
| PHASE-007/SD-PP0-005 | **ADD** R-pre1/R-pre2 mitigations and cfg-gate-not-delete rule. | 01-phase:L10-L11 | PP0-E003–PP0-E004 |
| PHASE-007/SD-PP0-006 | **FLAG** incomplete pre-enumeration of all cfg-gated tests and HookRunner shell audit. | 01-phase:L21-L22 | PP0-A001–PP0-A002 |

### phase-1I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-007/SD-P1I-001 | **ADD** ACTION-REQUIRED 1G→1I dep-graph fix and compile-failure scenario if ignored. | 01-phase:L147-L147 | P1I-E002, P1I-C001 |
| PHASE-007/SD-P1I-002 | **ADD** permanent-vs-transient classification, 2-success HALF_OPEN, fresh InferenceSession, deterministic model_id jitter details. | 01-phase:L148-L149 | P1I-E007–P1I-E011 |
| PHASE-007/SD-P1I-003 | **ADD** full cascade chain including TEXT-only/CODE-only intermediate modes and SPLADE-disabled collapse. | 01-phase:L150-L151 | P1I-E013–P1I-E018 |
| PHASE-007/SD-P1I-004 | **ADD** newline-accepting validation tests, canary_reference_vec first-startup generation, DEGRADED⇒BM25_ONLY invariant tests. | 01-phase:L152-L154 | P1I-E020–P1I-E025 |
| PHASE-007/SD-P1I-005 | **ADD** batching merge/segregate rules for cross-encoder queue and overflow returns unranked non-empty candidates. | 01-phase:L155-L155 | P1I-E026–P1I-E027 |
| PHASE-007/SD-P1I-006 | **ADD** full QueryTrace/StageTrace field lists, nine stage names, DEBUG-not-SSB rule, session-end aggregates. | 01-phase:L156-L156 | P1I-E028–P1I-E031 |
| PHASE-007/SD-P1I-007 | **ADD** index_version schema fields, archive-on-failure abort, user notification, isolation-under-degradation verification ownership. | 01-phase:L157-L158 | P1I-E032–P1I-E036 |
| PHASE-007/SD-P1I-008 | **ADD** MEASURE targets and Phase-1 overall measurement list. | 01-phase:L145-L158 | P1I-E037–P1I-E039 |

### phase-2D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-007/SD-P2D-001 | **ADD** design decisions D1–D5 (no-DKG detection, severity classes, separate inline counter, ReplanRequest MAJOR, type inventory). | 01-phase:L220-L226 | P2D-E004–P2D-E011 |
| PHASE-007/SD-P2D-002 | **ADD** full T1–T10 TDD inventory including CONTRADICTS_PLAN: marker and BudgetExhausted topic. | 01-phase:L228-L229 | P2D-E013–P2D-E015 |
| PHASE-007/SD-P2D-003 | **ADD** exact Critical pattern list, case-insensitive compare, assumption/consequence templates, guard_added=false default. | 01-phase:L227-L227 | P2D-E016–P2D-E018 |
| PHASE-007/SD-P2D-004 | **ADD** phase_2D.toml fields and overhead/false-positive SLOs; architecture-overview update requirement. | 01-phase:L231-L231 | P2D-E019 |
| PHASE-007/SD-P2D-005 | **ADD** grounding summary distinguishing GROUNDED/VERIFIED/UNGROUNDED claims and 4F deferrals. | 01-phase:L230-L230 | P2D-E020–P2D-E022 |
| PHASE-007/SD-P2D-006 | **FLAG** D3 method list vs SourceMatch-only type conflict and T4 `.passed` field wording. | 01-phase:L224-L229 | P2D-C001, P2D-A001–P2D-A002 |

### phase-3B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-007/SD-P3B-001 | **ADD** R10/R-3B-1/2/3 risks, r2d2 WAL cleanup, CHECK migration drop+recreate session semantics. | 01-phase:L246-L249 | P3B-E006–P3B-E007 |
| PHASE-007/SD-P3B-002 | **ADD** full SQL schema, indexes, ssb_files junction, and note that orchestrator.txt §3 synopsis is incomplete. | 01-phase:L249-L252 | P3B-E009–P3B-E012 |
| PHASE-007/SD-P3B-003 | **ADD** complete typed payload struct inventory including PhaseComplete fsync-before-unload semantics. | 01-phase:L256-L256 | P3B-E023–P3B-E024 |
| PHASE-007/SD-P3B-004 | **ADD** write-loop BEGIN IMMEDIATE, oneshot seq, shutdown drain, broadcast-after-commit, Arc payload, CustomizeConnection. | 01-phase:L253-L254 | P3B-E017–P3B-E021, P3B-E027 |
| PHASE-007/SD-P3B-005 | **ADD** SsbTraceable boundary, msgpack disk serialization, 1MB payload reject, full unit/property/adversarial/determinism suite and measure SLOs. | 01-phase:L255-L257 | P3B-E025–P3B-E029 |
| PHASE-007/SD-P3B-006 | **ADD** audit-fix history BLOCK1–5 WARN1–6 from s-1427264. | 01-phase:L246-L257 | P3B-E031 |
| PHASE-007/SD-P3B-007 | **FLAG** internal 15-vs-17 payload wording conflict. | 01-phase:L250-L250 | P3B-C001, P3B-A001 |

### phase-4D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-007/SD-P4D-001 | **ADD** Designer role config, SystemConstraints sizing defaults, confidence/ranking formulas, adaptive complexity table, info_gain prune rule. | 01-phase:L426-L434 | P4D-E004–P4D-E016 |
| PHASE-007/SD-P4D-002 | **ADD** full 4D.1–4D.6 TDD/adversarial gates, FORMULATE-SEARCH-EXTRACT-REFLECT loop, cloud-only speculation, measure keys, RuntimeClient wire rule. | 01-phase:L435-L435 | P4D-E019–P4D-E027 |
| PHASE-007/SD-P4D-003 | **ADD** recursive 4-level protocol, all decomposition/baby-step block types, ResearchCache, DecisionSource, package anti-hallucination gate, Interactive/Autonomous selection. | 01-phase:L436-L440 | P4D-E028–P4D-E037 |
| PHASE-007/SD-P4D-004 | **ADD entire web-research extension 4D.11–4D.18**: SearXNG client, WebFetcher/robots/readability, SourceQualityTable ≥200 domains, Web A-RAG, six VRAM tiers, budget manager, WebResearchOrchestrator, E2E metrics file. | 01-phase:L425-L440 | P4D-E038–P4D-E056 |
| PHASE-007/SD-P4D-005 | **ADD** CLI `--vram-tier`, INTERACTIVE progress via research_step events, design principles from survey papers. | 01-phase:L440-L440 | P4D-E038, P4D-E049, P4D-E057 |
| PHASE-007/SD-P4D-006 | **FLAG CONFLICT** research_* SSB action types not in 3B 17-type schema; Interactive present_options UI unspecified; projected VRAM ceilings. | 01-phase:L425-L440 | P4D-C001, P4D-A001–P4D-A003 |
| PHASE-007/SD-P4D-007 | **CORRECT/EXPAND** summary DecisionSource labels: source uses Research/UserChoice/AutoSelected (not RESEARCH\|EXISTING_PROJECT\|USER_SPECIFIED\|FALLBACK). | 01-phase:L439-L439 | P4D-E032 |

## PHASE-008 — source `DEEP-PHASE-008.discovery.md`

- Declared feature atoms: **253** (E=232 I=6 A=10 C=5).
- Parsed feature rows in this aggregate: **253**.
- Per-file: 3C **065**; 3E **039**; 4B **041**; 4F **040**; 4J **042**; IDX **026**.
- Deep ledger path: `docs/thefinalglance/feature-discovery/deep/DEEP-PHASE-008.discovery.md`.

### Feature atoms

### 3C — Agent system (pool, lifecycle, checkpoint, recovery, supervision, anti-spiral)

Source: `docs/specs/guide/steps/phase-3C.txt` (471/471 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3C-E001 | E/STATUS | Phase 3C goal is agent spawning, lifecycle, checkpointing, crash recovery, supervision tree, and anti-spiral escalation. Agents run in-process (Tier A) in Phase 3. | 3C:L7-L10 |
| P3C-E002 | E/DEFER | Process separation (Tier C) is deferred to Phase 4A. | 3C:L10-L10 |
| P3C-E003 | E/DEP | Prerequisites: Phase 3B SSB operational with single-writer guarantee. Status label IMPROVED, grounded against orchestrator/reasoning/architecture-overview/worker_boot. | 3C:L1-L3 |
| P3C-E004 | E/CAP | Transforms claw-code `WorkerRegistry` (~1069 LOC worker_boot.rs) and `recovery_recipes.rs` (~630 LOC) into architecture-overview AgentPool trait with checkpoint/recovery/supervision from orchestrator §7. | 3C:L12-L14 |
| P3C-E005 | E/DEP | Input from 3B: `ssb::write` for checkpoint/reasoning_trace; single-writer, append-only, seq monotonic. | 3C:L31-L32 |
| P3C-E006 | E/DEP | Input from 3A: `Fsm::get_state`/`transition`; phase-aware tool loading restricts tools per FSM state. | 3C:L33-L34 |
| P3C-E007 | E/DEP | Input from 2C/2A: `AgentGrcConfig` loaded at spawn; `GrcEngine::think` produces `ReasoningTraceEntry` to SSB. | 3C:L35-L38 |
| P3C-E008 | E/CAP | Output to 3D: `AgentPool::spawn` returns `AgentHandle`; `spawn_parallel` STUBBED sequential here, true parallel in 3D. | 3C:L41-L43 |
| P3C-E009 | E/CAP | Output to 3E: handle exposes `agent_id`, `role`, `active_files` for file-overlap scoring. | 3C:L44-L45 |
| P3C-E010 | E/FLOW | Output to 3F: agent tool calls route through orchestrator dispatch and `RuleEngine::evaluate` (no direct tool calls). | 3C:L46-L47 |
| P3C-E011 | E/CAP | Output to 4A: `AgentHandle` is IPC-ready; 3C wraps in-process, 4A wraps IPC socket with same interface. | 3C:L48-L49 |
| P3C-E012 | E/FLOW | Agents emit `ReplanRequest(evidence, severity)`: max 1 per agent per phase; SSB `action_type='replan_request'`; auditor validates evidence; MINOR fast-track to PLANNING; MAJOR human gate. | 3C:L50-L54 |
| P3C-E013 | E/DATA | Mapping: `WorkerStatus`→`AgentStatus` (add Checkpointing+BudgetExceeded; drop TrustRequired); `WorkerRegistry`→`AgentPool` with `HashMap<Uuid, AgentHandle>` via UUID v4 from EntropySource (not raw uuid crate). | 3C:L56-L62 |
| P3C-E014 | E/DET | Replace `now_secs()`/SystemTime with `ClockProvider::now_monotonic()` (entropy quarantine). | 3C:L65-L67 |
| P3C-E015 | E/DATA | `FailureScenario`/`RecoveryRecipe` map to death classification and `SupervisionPolicy.on_failure()`. | 3C:L68-L69 |
| P3C-E016 | E/CONC | R6: on single GPU (12GB), if model+embedding exceed VRAM, serialize agents (never parallel). VRAM check built into spawn. | 3C:L72-L74 |
| P3C-E017 | E/SEC | R31: Tier A invisible walls must return "not found" not "denied" for hidden paths. | 3C:L75-L77 |
| P3C-E018 | E/CAP | Six independently testable deliverables 3C.1–3C.6: pool/handle/status; lifecycle+death; checkpointing; recovery; supervision; anti-spiral. | 3C:L80-L86 |
| P3C-E019 | E/STATE | `AgentStatus`: Spawning, Ready, Running, Checkpointing, Finished, Failed. Drop TrustRequired; rename ReadyForPrompt→Ready; add Checkpointing. | 3C:L100-L104 |
| P3C-E020 | E/DATA | `AgentHandle` fields: `agent_id: Uuid`, `role: AgentRole`, `status`, `active_files: Vec<PathBuf>`, `grc_config: AgentGrcConfig`, `scratch_dir: PathBuf`. | 3C:L105-L110 |
| P3C-E021 | E/CFG | `grc_config` loaded from `elai_cc.toml [agents.{role}]` at spawn. | 3C:L110-L110 |
| P3C-E022 | E/DATA | `AgentRole` (8): Executor, Auditor, PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer. | 3C:L111-L113 |
| P3C-E023 | E/CAP | Designer produces PipelineSpec from Research Report; not Executor; distinct role_prompt/required_steps/output; reuses executor model (Gemma 4 31B) with separate AgentGrcConfig. | 3C:L114-L116 |
| P3C-E024 | E/CAP | `AgentPool` trait: `spawn(role,task)->AgentHandle`; `spawn_parallel` sequential stub (not unimplemented!); `communicate` direct in-process stub (IPC in 4A). | 3C:L117-L120 |
| P3C-E025 | E/FAIL | spawn VRAM check via `ModelPool::available_vram()` (never nvml direct); insufficient → `ResourceError` (not panic). | 3C:L121-L124 |
| P3C-E026 | E/TEST | Adversarial: Executor budget 8/1200, MemoryWriter 2/400 at spawn; spawn_parallel sequential; ResourceError on VRAM=0. | 3C:L131-L137 |
| P3C-E027 | E/STATE | Lifecycle SM: Spawning→Ready→Running→Finished|Failed; Phase 3 agent is tokio task. | 3C:L162-L164 |
| P3C-E028 | E/DATA | `AgentDeathClassification`: CLEAN_EXIT, CRASH, TIMEOUT (15s heartbeat silence), BUDGET_EXCEEDED. | 3C:L165-L169 |
| P3C-E029 | E/FLOW | Heartbeat every 5s via `tokio::sync::watch`; 15s silence = TIMEOUT; Tier C replaces with IPC AgentHeartbeat. | 3C:L170-L172 |
| P3C-E030 | E/SEC | Tier A visibility filter: per-agent `allowed_paths: Vec<GlobPattern>`; hidden default: `.elai_cc/`, rules.db, harness source, other agent checkpoints, SSB db, ONNX models, binary internals; returns NotFound. | 3C:L173-L177 |
| P3C-E031 | E/FLOW | ReplanRequest stub: counter on AgentHandle; second call `ReplanDenied`; MINOR→auditor then EXECUTING→PLANNING; MAJOR→INTERACTIVE human approval. | 3C:L178-L182 |
| P3C-E032 | E/EVENT | Every GRC think writes `ReasoningTraceEntry` to SSB: agent_id, step_index, step_type, content_summary max 200 chars, grounding_source/level, confidence, elapsed_ms, topic, budget_remaining. | 3C:L183-L186 |
| P3C-E033 | E/DATA | SSB schema migration may be required for action_types: reasoning_trace, isolation_metric, spiral_detected, supervision_exhausted (section 3 list omits them). | 3C:L154-L160 |
| P3C-E034 | E/DATA | Checkpoint path `.elai_cc/sessions/{sid}/agents/{agent_id}/` with five files: state/grc_history/tool_results/output_draft `.msgpack` + plain `checkpoint_ts` via ClockProvider. | 3C:L227-L235 |
| P3C-E035 | E/FLOW | Auto checkpoint triggers: after every GRC think, tool result, file edit confirmed; before slow ops (latency >1s). | 3C:L236-L240 |
| P3C-E036 | E/RECOVERY | Atomic checkpoint write: temp+fsync+rename; Windows delete-old-then-rename same-volume. | 3C:L241-L243 |
| P3C-E037 | E/EVENT | SSB checkpoint event payload is metadata only (agent_id, step_index, file sizes), not checkpoint body. | 3C:L244-L245 |
| P3C-E038 | E/SEC | Checkpoint serializer rejects UntrackedValue channel IDs with `CheckpointError::UntrackedValueReference`; LastValue/Topic allowed. | 3C:L246-L253 |
| P3C-E039 | E/CFG | If tool_results grows unbounded, rolling window default keep last 50 (configurable); target write <100ms after 100 tool calls. | 3C:L269-L271 |
| P3C-E040 | E/RECOVERY | Recovery: read latest checkpoint, spawn same role+agent_id new task, inject structured context, continue from last GRC step; first recovered step grounding_source=`CHECKPOINT:{path}`. | 3C:L291-L310 |
| P3C-E041 | E/DATA | Recovery recipe map: PromptMisdelivery/StaleBranch/CompileRedCrossCrate/ProviderFailure→CRASH; TrustPromptUnresolved/McpHandshakeFailure/PartialPluginStartup→N/A. | 3C:L300-L307 |
| P3C-E042 | E/RECOVERY | Corrupt checkpoint: degrade to fresh start with empty history + WARNING SSB (not permanent stuck). | 3C:L318-L320 |
| P3C-E043 | E/DET | Recovered agent_id unchanged; SSB step_index continues monotonically (no reset). | 3C:L308-L308, L321-L323 |
| P3C-E044 | E/IDEM | Recovered first action is NEXT GRC step (no replay of completed tool side effects). | 3C:L324-L326 |
| P3C-E045 | E/TEST | CRASH recovery resume <2s; TIMEOUT within heartbeat_timeout+2s. | 3C:L328-L329 |
| P3C-E046 | E/CFG | SupervisionPolicy: max_restarts default 3 (per role elai_cc.toml [agents.{role}.supervision]); restart_window = current phase duration; on_max_restarts Escalate | SubstituteModel(ModelId). | 3C:L343-L347 |
| P3C-E047 | E/CFG | Phase 3 INTERACTIVE: Escalate; Phase 4G AUTONOMOUS: SubstituteModel; Phase 3 enum has SubstituteModel but gated (error if used INTERACTIVE). | 3C:L348-L351 |
| P3C-E048 | E/STATE | restart_count per agent per phase; reset 0 at phase boundary. | 3C:L352-L352 |
| P3C-E049 | E/FLOW | CLEAN_EXIT no restart; CRASH/TIMEOUT restart+increment; BUDGET_EXCEEDED inject convergence, no restart, no count increment. | 3C:L353-L357 |
| P3C-E050 | E/EVENT | On max restarts: stop, checkpoint final, execute on_max_restarts, log supervision_exhausted with full history. | 3C:L358-L360 |
| P3C-E051 | E/CFG | max_restarts <1 in config clamps to 1 with WARNING. | 3C:L375-L376 |
| P3C-E052 | E/FLOW | Anti-spiral: track last N GRC steps; same_step_count >3 (same step_type+topic+similar content) = SPIRAL. | 3C:L394-L397 |
| P3C-E053 | E/DET | Similar content = blake3 of normalized content (lowercase, whitespace-collapsed); B-16A supersedes prior blake2s. | 3C:L398-L401 |
| P3C-E054 | E/FLOW | On SPIRAL: (1) checkpoint (2) inject convergence status step (3) another identical step → permanent stop (not restart) (4) SSB spiral_detected with 3 steps evidence. | 3C:L402-L408 |
| P3C-E055 | E/FLOW | Spiral stop is supervised stop not crash; on_spiral same as on_max_restarts (Escalate INTERACTIVE). | 3C:L409-L410 |
| P3C-E056 | E/SURFACE | INTERACTIVE spiral escalation UI text: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]". | 3C:L411-L413 |
| P3C-E057 | E/RECOVERY | Spiral tags pre-spiral checkpoint (N-3) as recovery_target; recovery loads pre-spiral not latest. | 3C:L428-L433 |
| P3C-E058 | E/DEFER | Tier B sandbox (Firecracker/gVisor) not in 3C: bash has only Tier A path filter; known security gap until 4A. | 3C:L446-L451 |
| P3C-E059 | E/DEFER | isolation_metric aggregation at phase boundary is 3A/3D scope; 3C only supplies per-agent budget_remaining data. | 3C:L452-L456 |
| P3C-E060 | E/OBS | Measures: checkpoint write <50ms; recovery <2s; checkpoint size <1MB/agent; heartbeat overhead <0.1% CPU; anti-spiral <1ms/step; memory overhead per agent logged. | 3C:L458-L465 |
| P3C-E061 | E/LIMIT | No lazy stubs: unimplemented!() is build failure; spawn_parallel sequential is valid stub. | 3C:L129-L129, L471 |
| P3C-I001 | I/SURFACE | HUMAN-facing surfaces in 3C are INTERACTIVE escalation panels for max-restarts and spiral (options listed); no dedicated dashboard panel is named. | 3C:L348-L349, L411-L413 |
| P3C-A001 | A/CFG | Rolling-window N for tool_results default 50 is given only as adversarial mitigation when write exceeds 100ms; whether always-on is required is unclear. | 3C:L269-L271 |
| P3C-A002 | A/DATA | Exact SSB CHECK constraint migration mechanism (ALTER vs recreate) left as implementer choice. | 3C:L157-L160 |
| P3C-A003 | A/FLOW | "last N GRC steps" window size for spiral tracking is not numerically fixed beyond the >3 consecutive trigger. | 3C:L395-L397 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events ✓; security ✓; failures/rollback ✓; determinism/concurrency ✓; observability ✓; tests/done gates ✓; constraints/deferred ✓. No formal privacy or compliance rule.

### 3E — Context budget allocator

Source: `docs/specs/guide/steps/phase-3E.txt` (366/366 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3E-E001 | E/STATUS | Goal: fixed-slot context budget so model context is never exceeded; adaptive reduction with floors; cache breakpoints for provider prefix caching. | 3E:L7-L10 |
| P3E-E002 | E/DEP | Prerequisites: Phase 3D BSP+channels; depends on providers token_count/token_count_estimate, SSB, BSP. | 3E:L2-L3 |
| P3E-E003 | E/DATA | `ContextSlot` (8): SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext, SsbEntries, GrcHistory, OutputReserve. | 3E:L21-L30 |
| P3E-E004 | E/DATA | `SlotAllocation`: slot, allocated_tokens, floor_tokens (0 for slots 1-3), actual_tokens, source_truncated. | 3E:L32-L38 |
| P3E-E005 | E/DATA | `ContextBudget`: model_max_context from ModelCapabilityRegistry; slots[8]; cache_breakpoints[2]; total_allocated/used; adaptive_level. | 3E:L40-L48 |
| P3E-E006 | E/DATA | `CacheBreakpoint`: after_slot (BP1 after PhaseSpec, BP2 after CodeContext), prefix_hash u64, valid bool. | 3E:L49-L53 |
| P3E-E007 | E/STATE | `AdaptiveLevel`: Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal. | 3E:L55-L61 |
| P3E-E008 | E/FLOW | Allocation uses token_count_estimate; final validation uses exact token_count; exact exceed triggers adaptive; hard truncate never silent. | 3E:L66-L72 |
| P3E-E009 | E/OBS | Full budget computation latency target <50ms. | 3E:L73-L73 |
| P3E-E010 | E/FLOW | Fixed slot order never reordered: 1 SystemPrompt, 2 Rules, 3 PhaseSpec, BP1, 4 RepoMap, 5 CodeContext, BP2, 6 SsbEntries, 7 GrcHistory, 8 OutputReserve. | 3E:L78-L88 |
| P3E-E011 | E/FLOW | BP1 invalidated by phase transition, role change, rule set change; BP2 by file set change; slots 6-8 recompute every invocation. | 3E:L90-L93 |
| P3E-E012 | E/INT | When BP1/BP2 valid, provider reuses cached KV for slots 1-3 / 1-5 via CacheBreakpoint.valid. | 3E:L95-L98 |
| P3E-E013 | E/CFG | Dynamic percentages: RepoMap 15%, CodeContext 40%, SsbEntries 25%, GrcHistory 20% after fixed 1-3 exact and OutputReserve subtracted. | 3E:L103-L113 |
| P3E-E014 | E/CFG | Floors: SsbEntries≥200, GrcHistory≥500, OutputReserve≥2000; dynamic_budget < sum(floors) terminal. | 3E:L115-L119 |
| P3E-E015 | E/FLOW | File overlap score = |intersection(entry.files, agent.files)| / |agent.files|; sort score DESC then seq DESC; score 0.0 excluded. | 3E:L124-L128 |
| P3E-E016 | E/FLOW | Budget recomputed per agent invocation in BSP superstep; called after prepare_next_tasks, before execute_tasks. | 3E:L133-L139 |
| P3E-E017 | E/DATA | Channel feed to slot 6: LastValue if overlap>0; Topic filtered by overlap; EphemeralValue only current superstep; UntrackedValue excluded. | 3E:L134-L138 |
| P3E-E018 | E/EVENT | Adaptive reduction max 3 attempts; each logs SSB action_type='context_reduction' with before/after tokens. | 3E:L144-L145 |
| P3E-E019 | E/FLOW | Attempt1 FilteredByRelevance: raise overlap threshold (e.g. 0.3), GRC current-topic only. | 3E:L147-L150 |
| P3E-E020 | E/FLOW | Attempt2 ReducedGranularity: denser PageRank, code→signatures+doc comments. | 3E:L152-L155 |
| P3E-E021 | E/FLOW | Attempt3 ReducedScope: current file+direct deps map; current file code; agent's own SSB only. | 3E:L157-L161 |
| P3E-E022 | E/FAIL | Terminal: INTERACTIVE ESCALATE with budget breakdown; AUTONOMOUS SPLIT_TASK (Phase 4G). Never silent truncate. | 3E:L163-L166 |
| P3E-E023 | E/DATA | Prompt assembly map: slots 1→role_prompt, 2→tier_3_rules, 4+5 merge→code_context, 3→task_spec+output_format+constraints; 6-7 structured after task_spec; 8 not in prompt. | 3E:L171-L186 |
| P3E-E024 | E/FLOW | Session handoff CMV trimming preserves BP1 slots 1-3; BP2 recomputed on new session (do not assume BP2 survives). | 3E:L191-L197 |
| P3E-E025 | E/CFG | OutputReserve: max(configured, 2000) subtracted before percentage split. | 3E:L229-L229 |
| P3E-E026 | E/DET | Remainder after floor division assigned to largest slot (CodeContext); sum all slots == model_max_context. | 3E:L231-L238 |
| P3E-E027 | E/FLOW | Floor steal order: CodeContext first, then RepoMap; both insufficient → Err(BudgetInfeasible); dynamic <700 → BudgetInfeasible. | 3E:L250-L254 |
| P3E-E028 | E/OBS | If CodeContext <200 after steal, log warning to SSB (not error). | 3E:L261-L261 |
| P3E-E029 | E/DET | prefix_hash uses deterministic hasher (xxhash/FxHash), never DefaultHasher. | 3E:L51-L51, L281-L281 |
| P3E-E030 | E/DET | Empty agent_files → overlap score 0.0 (no division by zero). | 3E:L302-L304 |
| P3E-E031 | E/OBS | 10k SSB entries filtering target <10ms. | 3E:L305-L305 |
| P3E-E032 | E/EVENT | Reduction logging writes SSB but does NOT enter current invocation's context (visible next turn only). | 3E:L327-L327 |
| P3E-E033 | E/TEST | Integration: Gemma 4 E2B 128K context; test 4096 and 131072; assembly <50ms via ClockProvider; 100x byte-identical. | 3E:L337-L348 |
| P3E-E034 | E/LIMIT | No Instant::now in context budget module (entropy quarantine). | 3E:L351-L351 |
| P3E-E035 | E/SURFACE | ContextBudget implements Display for debug-friendly budget breakdown; INTERACTIVE terminal surfaces Escalate(breakdown) to user. | 3E:L210-L210, L163-L164, L324 |
| P3E-E036 | E/TEST | 3E.1–3E.7 steps: types, compute, floors, cache breakpoints, SSB filter, adaptive, e2e integration. | 3E:L200-L355 |
| P3E-I001 | I/CFG | Configurable overlap raise "e.g. 0.3" is example not hard default for attempt 1. | 3E:L148-L148 |
| P3E-A001 | A/CFG | Exact default values for model_max_context sources and fixed-slot measurement sites left to ModelCapabilityRegistry/runtime. | 3E:L41-L41, L226-L227 |
| P3E-A002 | A/SURFACE | No dedicated cockpit/panel for budget is named; only Display + escalate breakdown. | 3E:L210, L163-L164 |

Category coverage: slots/allocation/floors/cache/adaptive/SSB filter/BSP timing/handoff/tests ✓. No standalone privacy rule.

### 4B — Tools (git, validation, DAIRA, edit match, ShortCoder, taxonomy, file read, incr. compile)

Source: `docs/specs/guide/steps/phase-4B.txt` (429/429 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4B-E001 | E/STATUS | Prerequisites: Phase 4A agent isolation + IPC. Goal: full tool suite in orchestrator; agents access via IPC ToolRequest/ToolResult. | 4B:L1-L2, L17-L21 |
| P4B-E002 | E/STATUS | Recorded 2026-04-16 commit 55ece36: 4B.1–4B.4 DONE (10 tool modules, IpcToolRegistry, 15 typed arg structs, 19/19 tests; 3 BLOCKs fixed). | 4B:L3-L7 |
| P4B-E003 | E/DEFER | 4B.5 MEASURE deferred (metrics remaining). 4B.6–4B.7 IntegrationTestRunner deferred blocked on 4E.7+ types. | 4B:L8-L10 |
| P4B-E004 | E/LIMIT | Scope exclusions: VisibilityFilter + schema sanitization = 4A; BashSandbox inherited not implemented; lsp_client evaluation deferred Phase 5 if not in MEASURE. | 4B:L39-L46 |
| P4B-E005 | E/FAIL | R1: IPC schema drift mitigated by postcard round-trip contract tests per ToolRequest variant. | 4B:L51-L54 |
| P4B-E006 | E/FAIL | R2: compilation on written file only; unresolved import of unwritten module → DEFERRED not COMPILATION_ERROR. | 4B:L55-L59 |
| P4B-E007 | E/CFG | R3: tools/ and knowledge/ pin identical tree-sitter grammar versions workspace-level. | 4B:L60-L63 |
| P4B-E008 | E/CONC | R4: ToolRegistry checks SSB file locks before write tools (no two agents write same file). | 4B:L64-L67 |
| P4B-E009 | E/DATA | ToolRequest variants: GitCommit/GitUndo/GitBranch, ValidateRequest, TraceRequest, EditRequest(+tier hint), SimplifyRequest, ClassifyRequest, FileReadRequest(+force_full), CompileCheckRequest. | 4B:L81-L86 |
| P4B-E010 | E/CFG | Tools registered with phase-aware filtering; PLANNING must not expose EditRequest. | 4B:L87-L88, L96-L97 |
| P4B-E011 | E/CAP | git2: auto-commit deterministic message from phase outcome; branch plan-{pipeline_id}; WIP commits for >5 edits; session-owned undo. | 4B:L177-L180 |
| P4B-E012 | E/CAP | 5-tier validation: T1 determinism → T2 quality/AST → T3 unit → T4 integration → T5 stress; auditor only, executor never runs own tests. | 4B:L181-L184 |
| P4B-E013 | E/CAP | DAIRA markers: CALL, RETURN, MUTATION, BRANCH, ERROR, PANIC, TIMEOUT, FINDING, PATTERN, ANOMALY, STAT:name, DURATION:name, MEMORY:name, CHECKPOINT, STEP:n; Rust tracing + Python sys.settrace. | 4B:L185-L190 |
| P4B-E014 | E/CAP | AdaptiveTool<T> budget-based scope reduction max 3 then error. | 4B:L189-L190, L237-L239 |
| P4B-E015 | E/OBS | MarkerReport includes raw_line_count and marker_line_count (ratio drives AdaptiveTool). | 4B:L132-L133 |
| P4B-E016 | E/CAP | 3-tier edit match: exact → whitespace-normalized (preserve indent) → ellipsis-elision; on fail SequenceMatcher similarity hint; never silent wrong match. | 4B:L191-L195 |
| P4B-E017 | E/CAP | ShortCoder: 10 tree-sitter rules R1–R10; R4/R6 configurable; post-process executor output before auditor. | 4B:L196-L198 |
| P4B-E018 | E/CAP | Failure taxonomy 9 categories: SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED; deterministic suggested_action; auditor reads before raw; 3+ same category/phase → anti-spiral. | 4B:L199-L204 |
| P4B-E019 | E/CAP | File read: outline mode >300 lines (tree-sitter sections); force_full bypass; budget guard warn if file >50% remaining context; INTERACTIVE user decides; AUTONOMOUS auto-downgrade outline. | 4B:L205-L208 |
| P4B-E020 | E/CAP | Incremental compile ASYNC after each EXECUTING write: cargo check/ruff/tsc/go vet on written file; results via IPC; max 3 rounds (2 retries); then NEEDS_AUDIT; skip TESTING_FIRST. | 4B:L209-L218 |
| P4B-E021 | E/SEC | IPC error messages sanitized: strip rule_id, predicate types, hook names. | 4B:L220-L222 |
| P4B-E022 | E/OBS | Metrics path `.elai_cc/metrics/phase_4B.toml` keys: ipc_overhead_ms, compile_check_latency_{rust,python,typescript,go}_ms, shortcoder_savings_pct, test_count, loc_tools. | 4B:L274-L277 |
| P4B-E023 | E/OBS | IPC dispatch overhead target <2ms p95; measure 100 calls p50/p95/p99. | 4B:L256-L260, L280-L281 |
| P4B-E024 | E/OBS | Compile latency expectations: Rust 2-5s, Python ruff <500ms, tsc <2s, go vet <1s (non-Rust marked UNGROUNDED until measured). | 4B:L261-L265 |
| P4B-E025 | E/OBS | Auditor BLOCK reduction ~30% is UNGROUNDED design estimate. | 4B:L266-L270 |
| P4B-E026 | E/OBS | ShortCoder paper claims 18-38% savings; measure on 10 representative samples. | 4B:L271-L272 |
| P4B-E027 | E/DEP | Papers daira-interface.md and shortcoder-interface.md required ingested before TESTING_FIRST. | 4B:L34-L36, L78-L79 |
| P4B-E028 | E/CAP | IntegrationTestRunner levels: BabyStep (<5s cargo check), Step (<30s module tests), Phase (<60s crate tests), Concept (<120s workspace concept_tag). | 4B:L306-L321 |
| P4B-E029 | E/DATA | IntegrationResult: level, passed, test_count, failures, stdout_hash [u8;32], elapsed_ms. | 4B:L329-L336 |
| P4B-E030 | E/CFG | tool_id="integration_test" available in AUDITING only; NOT during EXECUTING; orchestrator-triggered not agent-triggered. | 4B:L344-L346 |
| P4B-E031 | E/INT | Integration uses BashSandbox, FailureTaxonomy, IPC ToolResult, BABY_STEP uses incr. compile infrastructure. | 4B:L338-L342 |
| P4B-E032 | E/LIMIT | Every request-routing site must wire ssb, fsm, clock through single RuntimeClient (no None fields post-construction). | 4B:L298-L298 |
| P4B-E033 | E/TEST | ≥15 test functions; step ladder skeleton→verify→failing tests→implement→measure→integration. | 4B:L71-L170, L252-L287 |
| P4B-E034 | E/DET | git2 timestamps from ClockProvider not system time. | 4B:L232-L233 |
| P4B-E035 | E/FLOW | Validation tiers must short-circuit: tier-1 failure stops tiers 2-5. | 4B:L234-L236 |
| P4B-E036 | E/TEST | Incremental compile event-ordering test: next ToolRequest before mock compile completes. | 4B:L240-L243 |
| P4B-E037 | E/TEST | Supported languages for incr. compile tests: Rust, Python, TS, Go (one each). | 4B:L165-L166 |
| P4B-I001 | I/SURFACE | Tools are agent-facing IPC surfaces, not human UI; human sees outcomes via auditor/SSB/dashboard elsewhere. | 4B:L17-L18 |
| P4B-A001 | A/STATUS | Whether papers remain MISSING at time of this discovery is runtime state; plan still requires them as PRE_GATE. | 4B:L35-L36 |
| P4B-A002 | A/CFG | Exact deterministic git commit message format template not specified beyond "from phase outcome". | 4B:L178-L178 |
| P4B-C001 | C/STATUS | Header completion status says 4B.1-4.4 DONE while body still lists full TDD ladder; status is historical record, steps remain normative for residual/measure work. | 4B:L3-L10, L71-L287 |

Category coverage: tools/IPC/async compile/integration runner/metrics/security sanitization ✓.

### 4F — Feasibility probe

Source: `docs/specs/guide/steps/phase-4F.txt` (560/560 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4F-E001 | E/STATUS | Prerequisites: Phase 4E ExecutablePipeline. Goal: system probe + novel-step stubs (<50 lines) with branch-and-explore; skip when zero novel; escalate after max 3 alternatives; output stub_results.msgpack. | 4F:L1-L11 |
| P4F-E002 | E/DATA | SystemProbeResult: gpu_count, gpu_info, ram_total/free, disk_free, os, is_supported. | 4F:L28-L36 |
| P4F-E003 | E/DATA | GpuInfo: index, name, total/free bytes, driver_version, uuid (stable NVML). | 4F:L38-L45 |
| P4F-E004 | E/DATA | SystemConstraints: min_vram/ram/disk bytes, required_gpu_count. | 4F:L47-L52 |
| P4F-E005 | E/DATA | StubResult: step_id, alternative_index 0-2, FeasibilityOutcome, elapsed_ms, stdout_hash blake3, stderr_summary first 512 bytes, ResourceDelta. | 4F:L54-L62 |
| P4F-E006 | E/DATA | FeasibilityReport: probe, constraints, constraint_results, stubs, skipped_steps, escalations. | 4F:L71-L78 |
| P4F-E007 | E/DATA | ConstraintResult: resource string, required, available, margin, passed (margin≥0 met; margin≥0.20 skip threshold). | 4F:L80-L86 |
| P4F-E008 | E/DATA | FeasibilityEscalation: step_id, reason, stubs_tried (3 fails). | 4F:L88-L92 |
| P4F-E009 | E/STATE | FeasibilityDecision: Proceed, ProceedWithConstraints(Vec), Escalate(Vec) INTERACTIVE, Replan(Vec) AUTONOMOUS. | 4F:L94-L99 |
| P4F-E010 | E/DATA | Serialization: msgpack disk, postcard IPC; FeasibilityOutcome reused from reasoning (2D). | 4F:L101-L103 |
| P4F-E011 | E/FLOW | Skip when novel_steps empty AND all VRAM/RAM/disk free ≥ required*1.20; else run probe. | 4F:L107-L115 |
| P4F-E012 | E/EVENT | Skip logs SSB action_type='phase_transition' with skip_reason; FSM PLANNING→TESTING_FIRST. | 4F:L116-L118 |
| P4F-E013 | E/FAIL | No NVIDIA: Nvml::init Err → gpu_count=0, empty gpu_info. | 4F:L122-L124 |
| P4F-E014 | E/OBS | Keep single sysinfo System; refresh_specifics only needed categories. | 4F:L126-L128 |
| P4F-E015 | E/FLOW | Re-check VRAM via ModelPool.available_vram immediately before stub launch. | 4F:L130-L133 |
| P4F-E016 | E/SEC | Stubs Tier C with FEASIBILITY tools only: stub write + bash + web search; NO edit/git. | 4F:L139-L141 |
| P4F-E017 | E/CAP | FeasibilityProbe trait: probe_system, check_constraints, run_stub, decide; object-safe &self. | 4F:L157-L161 |
| P4F-E018 | E/CFG | Feasibility AgentDispatchConfig overrides: GRC 3/600; allowed_tools [stub_write,bash]; sandbox 25s graceful/30s hard; line limit strict <50. DispatchStage.agent AUTHORITATIVE. | 4F:L164-L172 |
| P4F-E019 | E/INT | IPC reuse 4A: 1-byte version + 4-byte length + postcard. | 4F:L173-L173 |
| P4F-E020 | E/EVENT | SSB branch per stub alternative tag="branch:stub_{step_id}_{n}"; max 3; merge winner discard losers. | 4F:L175-L175, L377-L380 |
| P4F-E021 | E/TEST | 16 contract tests T1–T16 covering probe, constraints, stub pass/reject/timeout, max3 escalate, skip matrix, msgpack, sandbox deny edit, VRAM recheck, dual timeout, ProceedWithConstraints, malformed IPC. | 4F:L228-L281 |
| P4F-E022 | E/TEST | 8 adversarial A1–A8: fs escape, network, malformed frame, concurrent GPU, no-GPU CI, oversized frame, VRAM race, 4th branch escalate. | 4F:L305-L323 |
| P4F-E023 | E/LIMIT | Stub ≥50 lines rejected before execution (strict <50). | 4F:L243-L245, L368 |
| P4F-E024 | E/FLOW | Dual timeout: graceful shutdown IPC at 25s; hard kill at 30s with explicit child.kill (kill_on_drop false). | 4F:L269-L273, L369-L374 |
| P4F-E025 | E/FAIL | Malformed IPC → StubResult Fail with stderr_summary containing "malformed_ipc". | 4F:L278-L281 |
| P4F-E026 | E/FAIL | VRAM drop pre-launch → Fail reason="vram_dropped" / "insufficient_vram". | 4F:L266-L268, L319-L321 |
| P4F-E027 | E/CFG | MAX_FRAME_SIZE 10MB for FeasibilityReport postcard. | 4F:L189-L190 |
| P4F-E028 | E/OBS | Metrics phase_4F.toml: system_probe_latency_ms (<500), stub p50/p99, constraint_check_latency_us (<100), report_serialization_bytes, skip_criteria_evaluation_us, test_count/pass, compile_time_delta_s. | 4F:L416-L425 |
| P4F-E029 | E/CAP | Baby-step extension: novel if PackageRef.verified=false OR unused framework OR untested capability (GPU/FFI). | 4F:L488-L491 |
| P4F-E030 | E/FLOW | Feasibility per concept before concept N execution (not all upfront). | 4F:L493-L496 |
| P4F-E031 | E/DATA | BabyStepFeasibilityReport / BabyStepStubResult / BabyStepEscalation types. | 4F:L503-L520 |
| P4F-E032 | E/FLOW | Per-concept skip when zero novel baby steps. | 4F:L498-L499 |
| P4F-E033 | E/DEP | Crate deps: nvml-wrapper, sysinfo, blake3 (if missing); rmp-serde already present. | 4F:L461-L465 |
| P4F-E034 | E/CFG | E-22 note: sysinfo dep may be rejected; RAM/disk via wmic//proc/meminfo/df + ELAI_FREE_RAM_GB/ELAI_FREE_DISK_GB fallbacks; nvml-wrapper activated non-optional. | 4F:L339-L346 |
| P4F-E035 | E/LIMIT | RuntimeClient ssb/fsm/clock wiring required on all request-routing structs. | 4F:L475-L475 |
| P4F-E036 | E/DET | Constraint check pure (no I/O); stdout blake3 deterministic for identical bytes. | 4F:L445-L456 |
| P4F-I001 | I/SURFACE | INTERACTIVE Escalate decision is human-facing; no dedicated panel name beyond decision enum. | 4F:L97-L98 |
| P4F-A001 | A/CFG | Whether web_search is always in FEASIBILITY tool set conflicts slightly: skeleton says stub_write+bash only; I3 text adds web search. | 4F:L167-L167, L366-L367 |
| P4F-C001 | C/DATA | Trait named FeasibilityProbe in TASK but greps look for FeasibilityProber; SD2 notes flat feasibility_probe.rs layout vs subdir. | 4F:L157-L184 |
| P4F-C002 | C/DEP | Body still specifies sysinfo API, while E-22 pre-gate rejects sysinfo as new dep and uses shell-out fallbacks. | 4F:L353-L360, L339-L344 |

Category coverage: probe/skip/stubs/timeouts/branches/decisions/baby-step/metrics/security ✓.

### 4J — Empire integration

Source: `docs/specs/guide/steps/phase-4J.txt` (710/710 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4J-E001 | E/STATUS | Conditional phase: only when Empire codebase accessible/deployed. Prereqs 4A, 3B, 4D, 4G. | 4J:L1-L8 |
| P4J-E002 | E/STATUS | EMPIRE-PENDING gate (2026-05-22): Empire not accessible; ship host scaffolding now with EMPIRE-PENDING labels; live activation one-line flip. | 4J:L12-L19 |
| P4J-E003 | E/CFG | Implementation at crates/orchestrator/src/empire/* under cfg(feature="empire"); not crates/empire; cargo check -p elai-orchestrator --features empire. | 4J:L21-L28 |
| P4J-E004 | E/EVENT | SSB action_type granularity = 5 variants: empire_connected, empire_disconnected, empire_event_received, empire_capability_invoked, empire_repair_proposed (not single empire_event). | 4J:L30-L37 |
| P4J-E005 | E/DATA | Empire IPC namespace empire::ipc::EmpireIpcMessageType (do not shadow elai_ipc_types::IpcMessageType 16 locked variants). | 4J:L39-L43 |
| P4J-E006 | E/CAP | Goal: speak FactoryLine, register as brain, subscribe Empire events as live GRC/SSB source, expose neuron capabilities, replace brainctl. | 4J:L45-L50 |
| P4J-E007 | E/INT | Empire topology: EngineMKI→MKII→MKIII; IPC sock /tmp/brain-engine.sock; FactoryLine /tmp/brain-factory.sock; newline JSON AF_UNIX; max 65536 IPC; FSB 12-byte header, 256KB max frame. | 4J:L52-L59 |
| P4J-E008 | E/DATA | FactoryLine cmds: register|heartbeat|list|request_update protocol v:1. | 4J:L64-L65 |
| P4J-E009 | E/DATA | Engine handlers MKI: REGISTER_HOOK, EMIT, QUERY_HOOKS, SUBSCRIBE, UNSUBSCRIBE, BUS_CREATE, BUS_JOIN, BUS_LEAVE, BUS_LIST. | 4J:L71-L72 |
| P4J-E010 | E/FAIL | Exactly 3 verified broken issues: brainctl↔brain_lifecycle circular; EngineMKII hard-import slc; unique-factory/types stdlib shadow. | 4J:L73-L78 |
| P4J-E011 | E/LIMIT | Dead code supersede: workforce lambda stubs; unvalidated brain.json superseded by PipelineSpec 6 layers. MetricsStore/SeedState alive. | 4J:L79-L83 |
| P4J-E012 | E/CAP | FactoryLineTransport trait: connect, health_check; TransportConfig Unix|Tcp; UnixTransport cfg(unix), TcpTransport all. | 4J:L132-L138 |
| P4J-E013 | E/CFG | FactoryLineConfig: transport, heartbeat_interval, reconnect_backoff ExponentialBackoff, event_rate_limit. | 4J:L146-L147 |
| P4J-E014 | E/DATA | FactoryLineCommand/FactoryCmd/FactoryLineResponse; IpcMessage/EmpireIpcMessageType (9 kinds)/IpcResponse; shared Event/Command/Response. | 4J:L152-L173 |
| P4J-E015 | E/DATA | BrainManifest with schema validation: brain_id, version, engine_tier MKI|MKII|MKIII, neurons, capabilities, heartbeat_interval_ms. | 4J:L175-L184 |
| P4J-E016 | E/CAP | EmpireEventBridge: ssb_writer, rate_limiter (always present not Option), EventFilter patterns+priority_map High|Normal|Low. | 4J:L186-L194 |
| P4J-E017 | E/CAP | ElaiNeuron exposes only task_submit, task_status, pipeline_result, health_check; never FSM/evidence/rules/SSB-direct/config/agent_spawn. | 4J:L196-L202, L409-L411 |
| P4J-E018 | E/DATA | EmpireIssue CircularDep|HardImport|StdlibShadow; FixType Refactor|Replace|Remove; repair via 4D pipeline factory. | 4J:L204-L212 |
| P4J-E019 | E/SEC | Empire-originated requests enter IPC proxy ToolFilter; cannot trigger FSM transitions or bypass gates. | 4J:L113-L117 |
| P4J-E020 | E/FLOW | Heartbeat at configured interval; 3 consecutive misses → reconnection backoff; reconnect re-sends Register. | 4J:L304-L308, L538-L539 |
| P4J-E021 | E/LIMIT | Zero brainctl imports; pure Rust FactoryLine client; zero pyo3/cpython. | 4J:L105-L108, L344-L351 |
| P4J-E022 | E/FLOW | Rate limit global (not per-pattern); high-priority bypass elevated but bounded (e.g. 100/sec not unlimited); dropped_count metrics. | 4J:L360-L389 |
| P4J-E023 | E/SEC | Rejection responses generic "capability not available" with no internal state leak. | 4J:L417-L449 |
| P4J-E024 | E/FLOW | task_submit returns task_id async; task_status polls FSM+progress; pipeline_result final output; same path as CLI /task. | 4J:L412-L432 |
| P4J-E025 | E/CAP | Repair: detect 3 issues programmatically; one PipelineSpec per issue; blast radius enforcement; TestOutcomeReport zero PASS_TO_FAIL. | 4J:L468-L517 |
| P4J-E026 | E/RECOVERY | Graceful degradation: Empire optional; features disable cleanly; elai starts without connection. | 4J:L540-L544, L612-L615 |
| P4J-E027 | E/FAIL | Startup unreachable: max 5 reconnect attempts then Error::EmpireUnreachable. | 4J:L536-L537 |
| P4J-E028 | E/DEFER | FSB binary transport evaluate-only; upgrade if event_latency_p95 >5ms consistently; frame payload_len > MAX_FSB_PAYLOAD default 16MB reject. | 4J:L545-L549, L678-L685, L563-L565 |
| P4J-E029 | E/DEFER | Multi-brain orchestration evaluate when multiple elai instances need coordination beyond multi-session. | 4J:L687-L694 |
| P4J-E030 | E/OBS | Metrics phase_4J.toml: connection_latency_ms (<100 Unix / <500 TCP), heartbeat_jitter_ms (<500), event_throughput_per_sec (≥90% of limit), event_latency_p95_ms (<10), capability_roundtrip_ms, repair_scan_time_ms (<5s), binary_size_delta_bytes, test_count (≥40). | 4J:L644-L672 |
| P4J-E031 | E/DET | Wire newline is exactly "\n" not "\r\n". Protocol v:1 preserved. | 4J:L288-L292, L320-L322 |
| P4J-E032 | E/INT | Empire events grounding source type EMPIRE_EVENT / source='empire' for GRC. | 4J:L370-L373, L596-L597 |
| P4J-E033 | E/SURFACE | Host scaffolding includes Tauri command surface + Svelte panel + feature gate (EMPIRE-PENDING). | 4J:L16-L18 |
| P4J-E034 | E/SURFACE | brainctl CLI replacement: orchestrator drives FactoryLine (start/stop/status brain lifecycle). | 4J:L343-L349, L704 |
| P4J-E035 | E/SURFACE | Capability discovery action "capabilities" returns exactly 4 entries. | 4J:L423-L427 |
| P4J-E036 | E/CFG | Feature gate cfg(feature="empire"); non-empire builds zero cost. | 4J:L614-L615, L708 |
| P4J-E037 | E/LIMIT | RuntimeClient ssb/fsm/clock wiring invariant. | 4J:L710-L710 |
| P4J-E038 | E/TEST | Steps 4J.1–4J.9 skeleton→verify→protocol tests→event bridge→capability→repair→resilience→implement→measure. | 4J:L122-L676 |
| P4J-I001 | I/SURFACE | Svelte panel and Tauri commands are required scaffolding surfaces but exact command/panel names are not listed in this phase file. | 4J:L16-L18 |
| P4J-A001 | A/INT | Exact default socket paths for Windows TCP and remote host config schema beyond TransportConfig not fully specified. | 4J:L137-L138 |
| P4J-C001 | C/EVENT | PRE-RESOLVED DECISION 3 requires 5 SSB variants, but 4J.4/4J.8 TASK still write action_type='empire_event' singular. | 4J:L30-L37, L356-L358, L596-L596 |
| P4J-C002 | C/TEST | DETERMINISTIC_CHECK still says cargo test -p elai_empire while DECISION 2 redirects to elai-orchestrator --features empire. | 4J:L21-L28, L313-L314, L618 |

Category coverage: transport/protocol/brain/neuron/bridge/repair/degradation/metrics/security/UI scaffolding ✓.

### IDX — Phase index pre0–2D (concise reference)

Source: `docs/specs/guide/steps/phase-index-pre0-2D.txt` (161/161 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| IDX-E001 | E/STATUS | Document is concise LLM-context index of pre-phase-0 through phase-2D: goals, deliverables, dependencies. | IDX:L1-L2 |
| IDX-E002 | E/CAP | PRE-PHASE 0: Windows platform fixes only (cfg-gate unix PermissionsExt, .cmd hooks, path separators, python not python3, PoisonError, warnings). No feature changes. | IDX:L5-L13 |
| IDX-E003 | E/OBS | PRE0 baseline: 69 .rs files, 61K LOC, 684 test annotations; 0 errors/failures/warnings on Windows. | IDX:L11-L13 |
| IDX-E004 | E/CAP | PHASE 0: strip claw-code to elai_cc; 9-crate workspace; one model ollama; FSM IDLE→EXECUTING→IDLE; one tool file-read; archive old code. | IDX:L16-L24 |
| IDX-E005 | E/LIMIT | Phase 0 risks: 25K LOC runtime/38 files, entropy quarantine 60 occurrences, config circular deps, TokenUsage refactor. | IDX:L21-L22 |
| IDX-E006 | E/CAP | 1A DKG: tree-sitter→SQLite graph; langs Rust/Python/TS/Go; dkg_nodes+dkg_edges; parse failure log+skip; LanguageGrammar trait. | IDX:L27-L34 |
| IDX-E007 | E/CAP | 1B BM25 tantivy: per-project; zero orphan non-binary; null-byte binary detect first 8KB; path STRING untokenized; update_file <5s; p99 query <10ms. | IDX:L37-L43 |
| IDX-E008 | E/CAP | 1C dual-model ONNX vectors: GraphCodeBERT 768d + Granite 384d; cache by content hash; circuit breakers; cascade HYBRID→SPARSE_ONLY→BM25_ONLY; canary embed("fn main()") within 5s. | IDX:L46-L53 |
| IDX-E009 | E/DEFER | SPLADE deferred from 1C to 1H measurement. | IDX:L51-L51 |
| IDX-E010 | E/CAP | 1D PageRank repo map: density guard 0.3; personalization 10x; edge boost *50; sqrt(ref_count); +-15% token fit; line trunc 100 chars; 21+ tests. | IDX:L56-L65 |
| IDX-E011 | E/CAP | 1E CatRAG+A-RAG full pipeline assembly BM25→vector→PageRank→CatRAG→A-RAG. | IDX:L68-L72 |
| IDX-E012 | E/CAP | 1F speculative decoding evaluation: ADOPT if >1.3x throughput on RTX 3060 12GB else DROP; k=1-7 sweep; decision cascade availability→acceptance→prompt lookup→throughput→VRAM. | IDX:L75-L84 |
| IDX-E013 | E/CAP | 1G cross-encoder ms-marco-MiniLM-L12-v2 ~120MB; CPU; <50ms/50 candidates; Semaphore(1)+mpsc depth 4; circuit breaker L-6 fallback. | IDX:L87-L94 |
| IDX-E014 | E/CAP | 1H SPLADE conditional feature=splade; vocab coverage <30% early DROP; KEEP if mean recall Δ≥5% and p<0.05. | IDX:L97-L105 |
| IDX-E015 | E/CAP | 1I resilience: circuit breaker; SearchMode cascade; query validator empty/>10K/null reject; readiness READY/DEGRADED/UNAVAILABLE; QueryTrace/StageTrace; index version migration. Deps 1E AND 1G. | IDX:L108-L116 |
| IDX-E016 | E/CAP | 2A GRC SM: observe→hypothesize→plan_action→act→verify (+status, converge); budget per-topic; forced converge remaining_calls≤1; format-only grounding; GroundingLevel Verified>High>Uncertain; ThinkStep.scope; typed errors. | IDX:L119-L127 |
| IDX-E017 | E/CAP | 2A GrcEngine trait 6 methods (4 implemented + 2 stubs); 12+ tests. | IDX:L126-L126 |
| IDX-E018 | E/CAP | 2B verify methods FILE/SOURCE/TEST/WEB/API/API_REGISTRY_CHECK; binary pass/fail; plan sweep blocks UNGROUNDED before EXECUTING. | IDX:L130-L137 |
| IDX-E019 | E/CFG | 2C six roles with budgets: executor 8/1200; auditor 10/1200 double-verify anti-rubber-stamp; context-compiler observe-only; memory-writer 2/400; prompt-engineer 5/800; designer 8/1200 PipelineSpec. | IDX:L140-L149 |
| IDX-E020 | E/CFG | AgentGrcConfig 8 fields including require_scope_enumeration true for executor/designer; layer [grc.defaults]+[agents.{role}.grc]; additive transition overrides. | IDX:L145-L147 |
| IDX-E021 | E/FLOW | Auditor anti-rubber-stamp: ≥1 FINDING: before Converge or rejected. | IDX:L148-L148 |
| IDX-E022 | E/CAP | 2D CSQ: CRITICAL (data_loss/security/financial/auth guard mandatory blocks) vs WARNING (logged). | IDX:L153-L156 |
| IDX-E023 | E/CAP | Inline feasibility max 2 checks per EXECUTING; Pass/Fail/Blocked; Blocked emits MAJOR ReplanRequest. Completes 2A-2D reasoning. | IDX:L157-L160 |
| IDX-E024 | E/DEP | Dependency chain PRE0→0→1A→1B→1C→1D→1E→1G→1I; 1F after 1C; 1H after 1G; 2A after 0; 2B after 2A; 2C after 2B; 2D after 2C. | IDX:L13-L161 |
| IDX-I001 | I/LIMIT | Index compresses full phase files; step-level PRE_GATE/TASK/ACCEPTANCE not present here (MASTER also delegates). | IDX:L1-L2 |
| IDX-A001 | A/STATUS | Index does not record completion status of any pre0–2D phase. | IDX:L1-L161 |

Category coverage: full pre0–2D phase catalog with key numbers ✓. Surfaces are mostly backend; no dashboard in this index range.

### Cross-source interactions (PHASE-008)

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| PHASE-008/IX-001 | E | 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection. | P3C-E009; P3E-E015 |
| PHASE-008/IX-002 | E | 3C spawn_parallel sequential stub is upgraded to true parallel only by 3D. | P3C-E008, P3C-E024 |
| PHASE-008/IX-003 | E | 3C communicate in-process stub becomes IPC in 4A; same AgentHandle abstraction. | P3C-E011, P3C-E024 |
| PHASE-008/IX-004 | E | 3C ReplanRequest MAJOR human gate and MINOR PLANNING fast-track compose with 3A FSM and auditor. | P3C-E012, P3C-E031 |
| PHASE-008/IX-005 | E | 3C ReasoningTraceEntry and checkpoint events depend on 3B SSB single-writer append path. | P3C-E005, P3C-E032, P3C-E037 |
| PHASE-008/IX-006 | E | 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400. | P3C-E007, P3C-E026; IDX-E019 |
| PHASE-008/IX-007 | E | 3C Designer role produces PipelineSpec later consumed by 4D/4E/4F chain. | P3C-E023 |
| PHASE-008/IX-008 | E | 3E budget is recomputed inside 3D BSP superstep between prepare_next_tasks and execute_tasks; consumes channel kinds from 3D. | P3E-E016, P3E-E017 |
| PHASE-008/IX-009 | E | 3E Terminal AUTONOMOUS SPLIT_TASK is owned by 4G; INTERACTIVE escalate is immediate. | P3E-E022 |
| PHASE-008/IX-010 | E | 3E OutputReserve and floors protect model generation headroom used by all later agent invocations. | P3E-E013, P3E-E014, P3E-E025 |
| PHASE-008/IX-011 | E | 4B tools run only through 4A IPC ToolRequest/ToolResult; isolation/sanitization owned by 4A, execution by 4B. | P4B-E001, P4B-E004, P4B-E021 |
| PHASE-008/IX-012 | E | 4B IntegrationTestRunner blocked on 4E DecompositionTree types and matches 4E integration time budgets. | P4B-E003, P4B-E028 |
| PHASE-008/IX-013 | E | 4B ShortCoder post-processes executor output before auditor; placement also referenced from 4C hooks. | P4B-E017 |
| PHASE-008/IX-014 | E | 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral. | P4B-E018; P3C-E052 |
| PHASE-008/IX-015 | E | 4B write tools honor SSB file locks (orchestrator concurrency invariant). | P4B-E008 |
| PHASE-008/IX-016 | E | 4F reuses 2D FeasibilityOutcome and 4A Tier-C agent process with restricted AgentDispatchConfig. | P4F-E010, P4F-E016, P4F-E018 |
| PHASE-008/IX-017 | E | 4F skip path PLANNING→TESTING_FIRST and novel_steps marking come from 4E structural validation. | P4F-E011, P4F-E012 |
| PHASE-008/IX-018 | E | 4F decision Escalate vs Replan depends on execution mode introduced by 3A/4G. | P4F-E009 |
| PHASE-008/IX-019 | E | 4F SSB branch tags compose with 3B SSB and 3D branch semantics (tag=branch:…). | P4F-E020 |
| PHASE-008/IX-020 | E | 4F baby-step feasibility requires 4E DecompositionTree/BabyStepSpec and runs before concept execution in 4G. | P4F-E029–P4F-E032 |
| PHASE-008/IX-021 | E | 4J event bridge writes Empire telemetry into 3B SSB for GRC grounding. | P4J-E016, P4J-E032 |
| PHASE-008/IX-022 | E | 4J ElaiNeuron capability path reuses 4A IPC proxy + ToolFilter trust boundary. | P4J-E017, P4J-E019 |
| PHASE-008/IX-023 | E | 4J repair pipelines are generated by 4D pipeline factory and validated with 4H TestOutcomeReport. | P4J-E018, P4J-E025 |
| PHASE-008/IX-024 | E | 4J autonomous unattended Empire operation depends on 4G autonomous mode. | P4J-E001 |
| PHASE-008/IX-025 | E | Index 2C role budgets are the same numbers 3C adversarial expects at spawn. | IDX-E019; P3C-E026 |
| PHASE-008/IX-026 | E | Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase. | IDX-E023; P3C-E012 |
| PHASE-008/IX-027 | E | Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs). | IDX-E011; P3E-E003 |
| PHASE-008/IX-028 | E | Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts. | P3C-E014; P3E-E034; P4B-E034; P4F-E024 |
| PHASE-008/IX-029 | E | RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J. | P4B-E032; P4F-E035; P4J-E037 |
| PHASE-008/IX-030 | E | blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A). | P3C-E053; P4F-E005 |
| PHASE-008/IX-031 | I | 4J Tauri/Svelte Empire panel will eventually sit beside 4K dashboard surfaces but 4K is out of this chunk. | P4J-E033 |
| PHASE-008/IX-032 | E | 3C SubstituteModel is type-ready but gated until 4G; 4F/4B do not enable it. | P3C-E047 |

### Ambiguity and conflict register (PHASE-008)

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| PHASE-008/AC-001 | ambiguity | 3C tool_results rolling window always-on vs only-if-slow. | P3C-A001 |
| PHASE-008/AC-002 | ambiguity | 3C SSB CHECK migration mechanism and spiral "last N" size. | P3C-A002, P3C-A003 |
| PHASE-008/AC-003 | ambiguity | 3E attempt-1 overlap threshold "e.g. 0.3" not fixed default. | P3E-I001, P3E-A001 |
| PHASE-008/AC-004 | status/conflict | 4B DONE status vs remaining measure/integration body. | P4B-C001, P4B-E002-E003 |
| PHASE-008/AC-005 | ambiguity | 4B deterministic commit message template unspecified. | P4B-A002 |
| PHASE-008/AC-006 | conflict | 4F FeasibilityProbe vs FeasibilityProber naming and sysinfo vs shell-out RAM strategy. | P4F-C001, P4F-C002 |
| PHASE-008/AC-007 | conflict | 4F FEASIBILITY tool set: stub_write+bash only vs +web search. | P4F-A001 |
| PHASE-008/AC-008 | conflict | 4J 5 SSB empire_* variants vs body 'empire_event' singular. | P4J-C001 |
| PHASE-008/AC-009 | conflict | 4J cargo package name elai_empire vs orchestrator --features empire. | P4J-C002 |
| PHASE-008/AC-010 | ambiguity | 4J Tauri/Svelte surface names not enumerated. | P4J-I001 |
| PHASE-008/AC-011 | ambiguity | Index pre0–2D omits completion and step-level gates. | IDX-I001, IDX-A001 |

### Summary deltas vs MASTER (PHASE-008)

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. "ADD" means an explicit source behavior omitted or materially compressed by the summary; "CORRECT/QUALIFY/REMOVE" means the summary's wording changes scope or certainty. These are relational records, not additional feature atoms.

### 3C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-3C-001 | **ADD** full input/output contracts (3B/3A/2C→3D/3E/3F/4A/ReplanRequest max-1). | 01-MASTER:L259-L274 | P3C-E005–P3C-E012 |
| PHASE-008/SD-3C-002 | **ADD** claw-code mapping details (UUID EntropySource, ClockProvider, TrustRequired drop). | 01-MASTER:L262 | P3C-E013–P3C-E015 |
| PHASE-008/SD-3C-003 | **ADD** R6 VRAM serialize rule and R31 NotFound indistinguishability tests. | 01-MASTER:L268 | P3C-E016–P3C-E017, P3C-E030 |
| PHASE-008/SD-3C-004 | **ADD** five checkpoint files, UntrackedValue rejection, Windows atomic rename, metadata-only SSB checkpoint. | 01-MASTER:L269-L271 | P3C-E034–P3C-E038 |
| PHASE-008/SD-3C-005 | **ADD** recovery recipe map, corrupt degrade, monotonic step_index, no tool replay. | 01-MASTER:L272 | P3C-E040–P3C-E044 |
| PHASE-008/SD-3C-006 | **ADD** spiral UI options string, N-3 recovery_target, blake3 B-16A supersession. | 01-MASTER:L274 | P3C-E053–P3C-E057 |
| PHASE-008/SD-3C-007 | **ADD** Tier B bash gap deferral, isolation_metric aggregation out-of-scope, measure targets. | 01-MASTER:L259-L274 | P3C-E058–P3C-E060 |

### 3E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-3E-001 | **ADD** full AdaptiveLevel enum including Terminal; exact compute remainder-to-CodeContext rule. | 01-MASTER:L290-L300 | P3E-E007, P3E-E026 |
| PHASE-008/SD-3E-002 | **ADD** BSP call timing, channel→slot6 rules, handoff BP2 recompute, Display surface. | 01-MASTER:L290-L300 | P3E-E016–P3E-E017, P3E-E024, P3E-E035 |
| PHASE-008/SD-3E-003 | **ADD** floor steal order, BudgetInfeasible at dynamic<700, CodeContext<200 warning. | 01-MASTER:L294 | P3E-E027–P3E-E028 |
| PHASE-008/SD-3E-004 | **ADD** numbered steps 3E.1–3E.7 acceptance detail MASTER only points to phase file for. | 01-MASTER:L300 | P3E-E036 |
| PHASE-008/SD-3E-005 | **ADD** reduction logs excluded from current context; 100x determinism; no Instant::now. | 01-MASTER:L297-L299 | P3E-E032–P3E-E034 |

### 4B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-4B-001 | **ADD** full DAIRA marker set (15 forms) and MarkerReport counts. | 01-MASTER:L402-L403 | P4B-E013, P4B-E015 |
| PHASE-008/SD-4B-002 | **ADD** force_full INTERACTIVE user-decide vs AUTONOMOUS auto-outline. | 01-MASTER:L407 | P4B-E019 |
| PHASE-008/SD-4B-003 | **ADD** R1–R4 risks, paper ingest PRE_GATE, RuntimeClient wiring exit rule. | 01-MASTER:L395-L409 | P4B-E005–P4B-E008, P4B-E027, P4B-E032 |
| PHASE-008/SD-4B-004 | **ADD** IntegrationTestRunner tool_id AUDITING-only, IntegrationResult stdout_hash. | 01-MASTER:L409 | P4B-E028–P4B-E030 |
| PHASE-008/SD-4B-005 | **ADD** metrics key list and UNGROUNDED labels on latency/BLOCK reduction projections. | 01-MASTER:L409 | P4B-E022–P4B-E026 |
| PHASE-008/SD-4B-006 | **QUALIFY** recorded DONE status applies to 4B.1–4.4 only; measure+runner remain deferred. | 01-MASTER:L398-L399 | P4B-E002–P4B-E003, P4B-C001 |

### 4F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-4F-001 | **ADD** complete type field lists (GpuInfo uuid, stderr 512, ResourceDelta, ConstraintResult margin). | 01-MASTER:L461 | P4F-E002–P4F-E008 |
| PHASE-008/SD-4F-002 | **ADD** 16+8 named tests T1–T16/A1–A8 and dual-timeout kill_on_drop detail. | 01-MASTER:L463 | P4F-E021–P4F-E024 |
| PHASE-008/SD-4F-003 | **ADD** E-22 sysinfo rejection / env fallback path and SD2 flat file layout. | 01-MASTER:L457-L465 | P4F-E034, P4F-C001–P4F-C002 |
| PHASE-008/SD-4F-004 | **ADD** metrics targets and 12 testable invariants summary. | 01-MASTER:L463 | P4F-E028, P4F-E036 |
| PHASE-008/SD-4F-005 | **ADD** DispatchStage.agent AUTHORITATIVE override rule and SSB branch merge/discard. | 01-MASTER:L462 | P4F-E018, P4F-E020 |
| PHASE-008/SD-4F-006 | **FLAG** web_search inclusion conflict in FEASIBILITY tool set. | 01-MASTER:L462 | P4F-A001 |

### 4J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-4J-001 | **ADD** EMPIRE-PENDING pre-resolved decisions 1–4 (scaffold, submodule path, 5 SSB variants, EmpireIpc namespace). | 01-MASTER:L508-L517 | P4J-E002–P4J-E005 |
| PHASE-008/SD-4J-002 | **ADD** Tauri command surface + Svelte panel scaffolding (user-facing). | 01-MASTER:L508-L517 | P4J-E033 |
| PHASE-008/SD-4J-003 | **ADD** rate-limit global vs high-priority elevated cap, generic rejection, reconnect Register. | 01-MASTER:L514-L515 | P4J-E020–P4J-E023 |
| PHASE-008/SD-4J-004 | **ADD** full measure keys/targets, FSB evaluate trigger, multi-brain evaluate trigger. | 01-MASTER:L517 | P4J-E028–P4J-E030 |
| PHASE-008/SD-4J-005 | **ADD** repair blast-radius, TYPE_CHECKING false-positive guard, one-issue-per-pipeline. | 01-MASTER:L512 | P4J-E025 |
| PHASE-008/SD-4J-006 | **FLAG CONFLICT** summary/body empire_event singular vs decision 5 variants; elai_empire crate name drift. | 01-MASTER:L514-L516 | P4J-C001–P4J-C002 |

### IDX (pre0–2D) deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-008/SD-IDX-001 | **ADD** PRE0 Windows-only scope and baseline LOC/test counts not expanded in MASTER's PRE0 section compression. | 01-MASTER:L13 | IDX-E002–IDX-E003 |
| PHASE-008/SD-IDX-002 | **ADD** Phase 0 entropy-quarantine occurrence count (60) and 9-crate/one-tool skeleton. | 01-MASTER:L24 | IDX-E004–IDX-E005 |
| PHASE-008/SD-IDX-003 | **ADD** 1F ADOPT threshold 1.3x and 1H vocab/recall KEEP gates as compact numbers. | 01-MASTER:L168-L181 (1F), related | IDX-E012, IDX-E014 |
| PHASE-008/SD-IDX-004 | **ADD** 2C six role budget table and anti-rubber-stamp FINDING rule. | 01-MASTER:L340-L361 area | IDX-E019–IDX-E021 |
| PHASE-008/SD-IDX-005 | **QUALIFY** index is not a substitute for numbered phase files' PRE_GATE/TASK/ACCEPTANCE. | 01-MASTER:L5 | IDX-I001 |

---

# MASTER summary record atoms

Source: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md` (769 lines).
Orientation/summary bullets from the family index. Where a deep ledger SD-* CORRECT/QUALIFY/ADD applies, the deep atom is **target state**; the MASTER bullet is kept here as compressed/superseded context.

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor | Section |
|---|---|---|---|---|
| MASTER-E001 | E/SUMMARY | P0 goal: make `claw-code-rust-port` compile and pass all 684 tests on Windows without feature changes (`L10–11`). | 01-phase-index-MASTER.discovery.md:L15 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E002 | E/SUMMARY | P0.1: fix ten compile errors by `cfg`-gating `std::os::unix::PermissionsExt` and `set_mode` in `runtime/mcp_stdio.rs`, `runtime/mcp_tool_bridge.rs`, and `rusty-claude-cli/tests/mock_parity_harness.rs` (`L13–14`). | 01-phase-index-MASTER.discovery.md:L16 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E003 | E/SUMMARY | P0.2: make four plugin-hook failures Windows-compatible by translating `.sh` to `.cmd` for `write_hook_plugin` and making `HookRunner::run_hook_command` dispatch through the platform shell (`L15–16`). | 01-phase-index-MASTER.discovery.md:L17 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E004 | E/SUMMARY | P0.3: fix four tool tests through path-separator normalization, `python3` → `python` on Windows, and poisoned environment-mutex recovery (`L17–18`). | 01-phase-index-MASTER.discovery.md:L18 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E005 | E/SUMMARY | P0.4: remove two unused-import/unused-`mut` warnings (`L19`). | 01-phase-index-MASTER.discovery.md:L19 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E006 | E/SUMMARY | P0.5: persist a clean baseline to `.elai_cc/metrics/pre_phase_0.toml` (`L20`). | 01-phase-index-MASTER.discovery.md:L20 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E007 | E/SUMMARY | Exit gate: `cargo check` must have zero errors and zero warnings and `cargo test` must have zero failures on Windows (`L21`). | 01-phase-index-MASTER.discovery.md:L21 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E008 | E/SUMMARY | Recorded baseline: 69 Rust files, 61,317 LOC, 684 test annotations, and a 26.24-second `cargo check` (`L22`). | 01-phase-index-MASTER.discovery.md:L22 | PRE-PHASE 0 — Windows platform fixes (`L10–22`) |
| MASTER-E009 | E/SUMMARY | Product reduction: strip approximately 4K+ LOC of known-bad code and restructure the 25K-LOC/38-file `runtime/` into a nine-crate workspace under `.elai_cc/crates/` (`L24–27`). | 01-phase-index-MASTER.discovery.md:L26 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E010 | E/SUMMARY | Walking skeleton: one Ollama model, one `IDLE → EXECUTING → IDLE` FSM loop, one `file_read` tool, workspace root `.elai_cc/Cargo.toml` (`L25–27`). | 01-phase-index-MASTER.discovery.md:L27 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E011 | E/SUMMARY | 0.1 migration ledger: `migration_manifest.toml` must classify every `.rs` file as `KEEP`, `STRIP`, `REPLACE`, or `EXTEND` (`L29`). | 01-phase-index-MASTER.discovery.md:L28 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E012 | E/SUMMARY | 0.2 strip set: remove `telemetry/`, `compat-harness/`, and runtime modules `usage`, `compact`, `summary_compression`, `lsp_client`, `oauth`, and `prompt`; replace stripped token accounting with a structural `TokenUsage` shim while refactoring 61 `Session` occurrences; audit 28+ slash commands into keep/strip sets (`L30–32`). | 01-phase-index-MASTER.discovery.md:L29 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E013 | E/SUMMARY | 0.3 strip verification: classify every failing test as `EXPECTED` or `BUG`, fix all `BUG`s, and record the strip delta (`L33`). | 01-phase-index-MASTER.discovery.md:L30 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E014 | E/SUMMARY | 0.4a leaf crates: create `core/`, `rules/`, and `providers/`; providers absorbs `api/` plus `runtime/sse.rs` (`L34`). | 01-phase-index-MASTER.discovery.md:L31 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E015 | E/SUMMARY | 0.4b middle layer: create `tools/` and `plugins/` plus empty `knowledge/` and `reasoning/` skeletons (`L35`). | 01-phase-index-MASTER.discovery.md:L32 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E016 | E/SUMMARY | 0.4c top layer: create `orchestrator/` from the remaining runtime and `cli/` from `rusty-claude-cli` plus `commands/`; archive `.elai_cc/rust/`; keep `main.rs` below 500 LOC as a thin REPL (`L36–37`). | 01-phase-index-MASTER.discovery.md:L33 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E017 | E/SUMMARY | 0.5 restructure verification: run `cargo test --workspace` and classify/fix `RESTRUCTURE_BUG` and `PATH_UPDATE` failures (`L38`). | 01-phase-index-MASTER.discovery.md:L34 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E018 | E/SUMMARY | 0.6 contract TDD: create `crates/contracts/` with five deliberately failing contract tests as the red phase (`L39`). | 01-phase-index-MASTER.discovery.md:L35 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E019 | E/SUMMARY | 0.7a provider/config: define a provider trait and `OllamaProvider`; configure through `elai_cc.toml`; support TOML and MessagePack (`rmp-serde`) serialization (`L40`). | 01-phase-index-MASTER.discovery.md:L36 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E020 | E/SUMMARY | 0.7b runtime base: implement `IDLE`/`EXECUTING` transition validation, a visibility-filtered `file_read`, and MessagePack session storage (`L41–42`). | 01-phase-index-MASTER.discovery.md:L37 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E021 | E/SUMMARY | 0.7c CLI/runtime UX: commands `/task`, `/config`, `/continue`, `/explore`, `/status`; visibility policy with `allowed_paths`, `hidden_paths`, and allowlist sanitization; `tracing`-based logging (`L43–44`). | 01-phase-index-MASTER.discovery.md:L38 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E022 | E/SUMMARY | 0.8 implementation gate: all five contract tests pass (`L45`). | 01-phase-index-MASTER.discovery.md:L39 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E023 | E/SUMMARY | 0.9 entropy quarantine: `ClockProvider` and `EntropySource` traits live in `core/`; production uses `OsClock`/`OsRng`; tests use `SeededClock`/`SeededRng`; replace 60 direct time/entropy uses across 31 files; `RuntimeContext` bundles both providers; a CI grep enforces the boundary (`L46–48`). | 01-phase-index-MASTER.discovery.md:L40 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E024 | E/SUMMARY | 0.10 determinism proof: with `ELAI_SEED=42` and `--test-threads=1`, two runs must have a zero diff (`L49`). | 01-phase-index-MASTER.discovery.md:L41 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E025 | E/SUMMARY | Phase output: exactly the named nine crates—`core`, `providers`, `knowledge`, `reasoning`, `orchestrator`, `tools`, `plugins`, `rules`, `cli`—plus the contract-test crate and no raw time/entropy outside core implementations (`L50–51`). | 01-phase-index-MASTER.discovery.md:L42 | PHASE 0 — fork, strip, restructure, walking skeleton (`L24–51`) |
| MASTER-E026 | E/SUMMARY | Goal/layer: parse source into a dependency knowledge graph in SQLite as knowledge-pipeline Layer 1, initially Rust with Python/TypeScript/Go later, in `.elai_cc/crates/knowledge/` (`L57–60`). | 01-phase-index-MASTER.discovery.md:L46 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E027 | E/SUMMARY | Dependencies: `tree-sitter 0.26`, `tree-sitter-rust 0.24`, `rusqlite 0.39` with bundled SQLite, `blake2`, and development-only `proptest` (`L59–60`). | 01-phase-index-MASTER.discovery.md:L47 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E028 | E/SUMMARY | 1A.1 interfaces: object-safe `KnowledgeEngine` (`build_graph`, `query`, `rerank`, `update`, `readiness_check`) and `LanguageGrammar` with `&self` methods (`L62–63`). | 01-phase-index-MASTER.discovery.md:L48 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E029 | E/SUMMARY | 1A.1 graph types: `SymbolKind`; `DkgNodeInfo {node_id, symbol_name, file_path, signature, kind, start_line, end_line}`; `DkgEngine::new(db_path)` (`L63–65`). | 01-phase-index-MASTER.discovery.md:L49 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E030 | E/SUMMARY | 1A.1 schema: `dkg_nodes` including `concept`, `concept_stale`, `file_hash`; `dkg_edges` typed `imports`/`contains`/`implements` with `ON DELETE CASCADE`; `dkg_files` including `parse_status`; `dkg_meta`; indexes on name/file/kind; WAL mode; five-field `index_version.toml` (`L65–67`). | 01-phase-index-MASTER.discovery.md:L50 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E031 | E/SUMMARY | 1A.1 read API: nodes, edges, outbound edges, inbound edges, failed files, and `dkg_readiness` (`L68`). | 01-phase-index-MASTER.discovery.md:L51 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E032 | E/SUMMARY | 1A.2 build gate: workspace compiles with no regressions (`L69`). | 01-phase-index-MASTER.discovery.md:L52 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E033 | E/SUMMARY | 1A.3 tests: at least 20 tests for Rust parsing, populated signatures, directions/types of edges (`impl→method=contains`, `use_decl→target=imports`, `impl Trait=implements`), parse-failure tracking, readiness states, project isolation, and property tests (`L70–72`). | 01-phase-index-MASTER.discovery.md:L53 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E034 | E/SUMMARY | 1A.4 implementation: reconstruct Rust signatures from AST; dispatch grammars through `HashMap<&str, Box<dyn LanguageGrammar>>`; create schema and WAL pragma; use Blake2s content hashes for incremental update with cascade cleanup; log and skip parse failures; readiness moves `Unavailable → Ready\|Empty` (`L73–75`). | 01-phase-index-MASTER.discovery.md:L54 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E035 | E/SUMMARY | 1A.5 measurements: record build time, query p50/p99, DB size, node count, and edge count in `phase_1A.toml` (`L76`). | 01-phase-index-MASTER.discovery.md:L55 | PHASE 1A — DKG / tree-sitter-to-SQLite graph (`L57–76`) |
| MASTER-E036 | E/SUMMARY | Goal/SLO: a Tantivy index per project; query p99 below 10 ms, `update_file` below 5 seconds, zero orphan text files, and binary-file exclusion when a null byte occurs in the first 8 KiB (`L78–80`). | 01-phase-index-MASTER.discovery.md:L59 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E037 | E/SUMMARY | Storage: `.elai_cc/projects/{id}/bm25/` per project (`L80`). | 01-phase-index-MASTER.discovery.md:L60 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E038 | E/SUMMARY | Schema invariant: `path` is `STRING\|STORED` and untokenized; `content` is `TEXT\|STORED`; `line_start`/`line_end` are `u64`; `chunk_kind` is `STRING`; using `TEXT` for path is forbidden because it breaks `delete_term` (`L81–82`). | 01-phase-index-MASTER.discovery.md:L61 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E039 | E/SUMMARY | 1B.1 compatibility pin: pin exact Tantivy version and record `INDEX_FORMAT_VERSION` (`L84`). | 01-phase-index-MASTER.discovery.md:L62 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E040 | E/SUMMARY | 1B.2 API: distinct `BM25Chunk` (not `DkgNode`) and `Bm25Index::{index_file,index_directory,query,delete_file,update_file}`, with atomic update (`L85–86`). | 01-phase-index-MASTER.discovery.md:L63 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E041 | E/SUMMARY | 1B.3 tests: at least ten tests for index/query/delete/incremental behavior, project isolation, and a path-field precision regression using `src/deep/nested/module.rs` (`L87–88`). | 01-phase-index-MASTER.discovery.md:L64 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E042 | E/SUMMARY | 1B.4 fallback chunking: when DKG has no nodes, use 50-line sliding windows with ten-line overlap; overlap is explicitly ungrounded for BM25 and retained only for vector-layer alignment (`L89–90`). | 01-phase-index-MASTER.discovery.md:L65 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E043 | E/SUMMARY | 1B.4 chunk/update semantics: `ChunkKind::{Function,LineBlock,File}` and atomic delete+insert in a single `writer.commit()` (`L91`). | 01-phase-index-MASTER.discovery.md:L66 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E044 | E/SUMMARY | 1B.5 benchmark: run on 69 files from `.elai_cc/rust/`; record p50/p99, `orphan_files=0`, Tantivy version, and index format version in `phase_1B.toml`; latency gate is <10 ms dev and <50 ms CI (`L92–93`). | 01-phase-index-MASTER.discovery.md:L67 | PHASE 1B — BM25 / Tantivy full-text retrieval (`L78–93`) |
| MASTER-E045 | E/SUMMARY | Semantic models: GraphCodeBERT for code (768 dimensions) and Granite for text (384 dimensions), both ONNX (`L95–97`). | 01-phase-index-MASTER.discovery.md:L71 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E046 | E/SUMMARY | Vector cache: SQLite cache keyed by Blake2s content hash (`L96–97`). | 01-phase-index-MASTER.discovery.md:L72 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E047 | E/SUMMARY | Resilience: per-model circuit breakers; startup must embed `"fn main()"` within five seconds; canonical model path `.elai_cc/models/onnx/`; a missing model degrades to `BM25_ONLY` rather than crashing (`L97–99`). | 01-phase-index-MASTER.discovery.md:L73 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E048 | E/SUMMARY | Dependency rule: use pinned `ort`, `usearch 2.24`, `fastembed 5.12`, and `blake2`/`Blake2s256`, but do not mix `fastembed` with `ort`; SPLADE waits for 1H (`L100–103`). | 01-phase-index-MASTER.discovery.md:L74 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E049 | E/SUMMARY | 1C.2 modules: `vector/{mod,cache,ort_session,circuit_breaker,hnsw_index}.rs` (`L104`). | 01-phase-index-MASTER.discovery.md:L75 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E050 | E/SUMMARY | 1C.2 ONNX session: `Arc<Mutex<ort::Session>>`, sequence length 512, truncate and warn on overflow (`L105`). | 01-phase-index-MASTER.discovery.md:L76 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E051 | E/SUMMARY | 1C.2 ANN index: `UsearchIndex` with cosine metric, save/load, and project-isolated filtered search (`L106`). | 01-phase-index-MASTER.discovery.md:L77 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E052 | E/SUMMARY | 1C.2 breaker: open after three failures in 60 seconds, half-open after 30 seconds, and use injected `ClockProvider` rather than `std::time` (`L107`). | 01-phase-index-MASTER.discovery.md:L78 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E053 | E/SUMMARY | 1C.3 tests: at least 22 covering cache lookup/invalidation, breaker open/half-open, degradation cascade, validation of empty/oversized/control-character queries, canonical path, missing-model degradation, semaphore overflow, cache-hit embed avoidance, truncation warnings, HNSW persistence, three-ranker RRF with `k=60`, and concurrent embedding mutex behavior (`L108–111`). | 01-phase-index-MASTER.discovery.md:L79 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E054 | E/SUMMARY | 1C.3.5 tensor gate: verify GraphCodeBERT/Granite tensor shapes from model cards or `ort` metadata and record the decision in `workflow/reasoning/phase-1C-decisions.md` (`L112–113`). | 01-phase-index-MASTER.discovery.md:L80 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E055 | E/SUMMARY | 1C.4 implementation: embedding; SSB-single-writer vector-cache writes; clocked breaker; HNSW persistence with cache rebuild when load fails; three-ranker RRF `k=60`; SSB `file_write` causes cache deletion; mode changes emit `action_type='knowledge_degradation'`; readiness canary doubles as warmup (`L114–117`). | 01-phase-index-MASTER.discovery.md:L81 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E056 | E/SUMMARY | 1C.5 measurement: embedding throughput, cache-hit rate ≥90%, and dual-vector recall versus BM25 (`L118`). | 01-phase-index-MASTER.discovery.md:L82 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-D057 | D/SUMMARY | TurboQuant: grounded as approximately 10.7× compression at `b=3` for 768d/384d and ~0.001 s indexing, but deferred to 1G or later; Phase 1C stores raw `f32` (`L119–120`). | 01-phase-index-MASTER.discovery.md:L83 | PHASE 1C — dual-model ONNX vector search (`L95–120`) |
| MASTER-E058 | E/SUMMARY | Goal: rank symbols by importance and fit repo maps to a context token budget (`L122–123`). | 01-phase-index-MASTER.discovery.md:L87 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E059 | E/SUMMARY | Aider derivation/divergence: derived from `aider/repomap.py`; use symbol-level rather than file-level nodes, a 0.3 density guard, and insertion-order fallback (`L124–125`). | 01-phase-index-MASTER.discovery.md:L88 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E060 | E/SUMMARY | Adopted algorithms: redistribute dangling mass through personalization; orphan self-edges weight 0.1; edge weights scale with square root of reference count; binary-search initial guess `max_map_tokens/25`; truncate lines to 100 characters before token estimation (`L126–128`). | 01-phase-index-MASTER.discovery.md:L89 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E061 | E/SUMMARY | 1D.1 snapshot/performance design: `DkgSnapshot::from_engine()` provides an in-memory DKG mirror to avoid SQLite queries per iteration (`L130`). | 01-phase-index-MASTER.discovery.md:L90 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E062 | E/SUMMARY | 1D.1 configuration: `PageRankConfig {damping=0.85,max_iter=100,tol=1e-6,density_threshold=0.3,max_map_tokens=2048}` (`L131–132`). | 01-phase-index-MASTER.discovery.md:L91 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E063 | E/SUMMARY | 1D.1 output/API: `RepoMap {ranked_symbols,token_count,density,used_pagerank,fit_iterations}` and `compute_repo_map(snapshot,task,active_files,config,token_estimator)` (`L132–133`). | 01-phase-index-MASTER.discovery.md:L92 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E064 | E/SUMMARY | 1D.2–1D.3 gates: workspace compiles; tests T1–T18, properties P1–P3, adversarial A1–A3 (at least 21), with `make_snapshot` helper (`L134–135`). | 01-phase-index-MASTER.discovery.md:L93 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E065 | E/SUMMARY | 1D.4 scoring: adjacency matrix with square-root scaling/self-edges; 10× personalization for task-mentioned nodes normalized to sum 1; 50× edge boost for active files (`L136–138`). | 01-phase-index-MASTER.discovery.md:L94 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E066 | E/SUMMARY | 1D.4 safety/fallback: density >0.3 disables PageRank and uses insertion order; power iteration clamps NaN/Inf and redistributes dangling mass (`L138–140`). | 01-phase-index-MASTER.discovery.md:L95 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E067 | E/SUMMARY | 1D.4 special files/fit: include a 13-item Rust special-file list such as `Cargo.toml`, `README.md`, `build.rs`; binary-search fitting is capped at ten iterations (`L139–140`). | 01-phase-index-MASTER.discovery.md:L96 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E068 | E/SUMMARY | 1D.5: persist metrics in `phase_1D.toml` (`L141`). | 01-phase-index-MASTER.discovery.md:L97 | PHASE 1D — PageRank repository map (`L122–141`) |
| MASTER-E069 | E/SUMMARY | End-to-end route: `query → BM25 → vector → PageRank → CatRAG → A-RAG → snippets`; target 55–87% token savings against naive full-chunk retrieval and latency under two seconds (`L143–146`). | 01-phase-index-MASTER.discovery.md:L101 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E070 | E/SUMMARY | Adaptation: CatRAG/A-RAG paper entity nodes map to `DkgNode`, passage nodes to file chunks; tree-sitter replaces OpenIE triples; regex/fuzzy DKG-name matching replaces NER (`L146–148`). | 01-phase-index-MASTER.discovery.md:L102 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E071 | E/SUMMARY | 1E.1 CatRAG config: `damping=0.5`, `epsilon=0.2`, `beta=2.5`, `n_seed=5`, `k_edge=15`, `min_graph_density=0.01`; overlays are sparse deltas and do not copy full matrices (`L150–151`). | 01-phase-index-MASTER.discovery.md:L103 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E072 | E/SUMMARY | 1E.1 A-RAG config: `max_loops=15`, `max_token_budget=128000`, `top_k=5`; `RetrievalTool::{KeywordSearch,SemanticSearch,ChunkRead}`; `ContextTracker` deduplicates (`L152–153`). | 01-phase-index-MASTER.discovery.md:L104 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E073 | E/SUMMARY | 1E.1 tracing: `QueryPipeline` is stage-based and every `PipelineStage` carries `latency_ms` (`L154`). | 01-phase-index-MASTER.discovery.md:L105 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E074 | E/SUMMARY | 1E.2–1E.3 gates: compile verification plus five CatRAG tests (anchoring, weighting, enhancement, density guard, `phi()` tiers), five A-RAG tests, three pipeline tests, and two contract tests (`L155–157`). | 01-phase-index-MASTER.discovery.md:L106 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E075 | E/SUMMARY | 1E.4 CatRAG behavior: extract CamelCase/snake_case symbols, fuzzy-match DKG names, score edges deterministically (`call=8`, `implements=7`, `references=5`, `imports=3`) through `phi()` tiers with no hot-path LLM, and boost verified context edges by `1+beta` (`L158–160`). | 01-phase-index-MASTER.discovery.md:L107 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E076 | E/SUMMARY | 1E.5 keyword search: wrap BM25 and return sentence-level snippets with ±2 lines (`L161`). | 01-phase-index-MASTER.discovery.md:L108 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E077 | E/SUMMARY | 1E.5 semantic search: oversample by 3× and aggregate candidates per file using maximum score (`L162`). | 01-phase-index-MASTER.discovery.md:L109 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E078 | E/SUMMARY | 1E.5 chunk reads: repeated content becomes a `"(previously read)"` placeholder through `ContextTracker` (`L162–163`). | 01-phase-index-MASTER.discovery.md:L110 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E079 | E/SUMMARY | 1E.6 composition: stages are `bm25→vector→pagerank→catrag→arag`, with rerank (1G) and token-fit (3E) stubs; BM25/vector fuse by `1/(60+rank)`; density guard passes through (`L164–165`). | 01-phase-index-MASTER.discovery.md:L111 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E080 | E/SUMMARY | 1E.7 acceptance: five representative queries, mean token savings >50%, p50 pipeline latency <2000 ms (`L166`). | 01-phase-index-MASTER.discovery.md:L112 | PHASE 1E — CatRAG + A-RAG assembled query pipeline (`L143–166`) |
| MASTER-E081 | E/SUMMARY | Decision feature: an `ADOPT`/`DROP` gate for speculative decoding, dependent on 1C `ModelPool` VRAM accounting (`L168–169`). | 01-phase-index-MASTER.discovery.md:L116 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-C082 | C/SUMMARY | Resolution: Ollama 0.20.2 lacked speculative decoding; 1F.3–1F.5 were skipped; Phase 1 outcome is `DROP`; revisit under the 3J vLLM provider in Phase 2+; config skeleton and provider probe are done; prompt-lookup decoding was also dropped (`L170–172`). | 01-phase-index-MASTER.discovery.md:L117 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-E083 | E/SUMMARY | 1F.1 dormant config: `OllamaConfig.speculative_decoding`, `draft_model: Option<String>`, and `num_draft_tokens: u8` (`L173–175`). | 01-phase-index-MASTER.discovery.md:L118 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-E084 | E/SUMMARY | 1F.2 capability probe: resolve `SUPPORTED` versus `NOT_SUPPORTED` for `draft_model` (`L175`). | 01-phase-index-MASTER.discovery.md:L119 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-D085 | D/SUMMARY | Original evaluation: ten temperature-zero prompts with soft-text equivalence; prompt-lookup zero-VRAM baseline; throughput sweep `k={1,2,3,4,5,7}` and early kill below 0.5 acceptance; VRAM measurement; adopt only above 1.3× throughput and below 3 GB extra VRAM (`L176–179`). | 01-phase-index-MASTER.discovery.md:L120 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-E086 | E/SUMMARY | Hardware correction: RTX 3060 12 GB has 4.8 GB headroom after Gemma 4 E2B (~7.2 GB Q4_K_M), not the prior ~10 GB claim in `providers.txt` (`L180–181`). | 01-phase-index-MASTER.discovery.md:L121 | PHASE 1F — speculative decoding evaluation (`L168–181`) |
| MASTER-E087 | E/SUMMARY | Model/SLO: post-A-RAG `ms-marco-MiniLM-L12-v2` (33.4M parameters, ~120 MB ONNX fp32/~30 MB int8), CPU-only, target <50 ms for 50 candidates (`L183–186`). | 01-phase-index-MASTER.discovery.md:L125 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E088 | E/SUMMARY | Skip/concurrency: skip at candidate count ≤10 by default; `tokio::Semaphore(1)`, bounded MPSC queue depth 4, request batching (`L185–186`). | 01-phase-index-MASTER.discovery.md:L126 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E089 | E/SUMMARY | Runtime decisions: raw `ort` + `tokenizers`, not `fastembed-rs`; begin fp32; consider int8 only if baseline p50 >30 ms and NDCG@10 delta <0.5; MPSC drain batching with per-caller oneshot result splitting (`L187–189`). | 01-phase-index-MASTER.discovery.md:L127 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E090 | E/SUMMARY | 1G.1 config: max sequence 512, queue 4, one intra-op thread, `ORT_ENABLE_ALL`, deterministic compute enabled (`L190–193`). | 01-phase-index-MASTER.discovery.md:L128 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E091 | E/SUMMARY | 1G.2 load/tokenize/fallback: deterministic optimized ORT session, BERT `tokenizer.json` truncation at 512, lazy L-6 fallback loaded only after L-12 breaker opens (`L194–196`). | 01-phase-index-MASTER.discovery.md:L129 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E092 | E/SUMMARY | 1G.3: pair scoring, reranking, semaphore-protected MPSC batch draining, offset-tracked split, oneshot return per caller (`L197–198`). | 01-phase-index-MASTER.discovery.md:L130 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E093 | E/SUMMARY | 1G.4 resilience: three failures/60 s, 30 s half-open, injected clock; lazy L-6 fallback; SSB `knowledge_degradation`; `StageTrace 'rerank'`; final fallback skips reranking (`L199–201`). | 01-phase-index-MASTER.discovery.md:L131 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E094 | E/SUMMARY | 1G.5 readiness: startup canary performs embedding, BM25, and rerank inside five seconds, using BM25 candidates (`L202`). | 01-phase-index-MASTER.discovery.md:L132 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E095 | E/SUMMARY | 1G.6 optimization harness: thread sweep 1/2/4/8 with determinism/rank-order stability; conditional int8; skip thresholds 5/10/15/20/30 selected by NDCG@10; committed 50-candidate fixture `tests/fixtures/reranker_benchmark.json`; quantization script `scripts/quantize_cross_encoder.py` (`L203–206`). | 01-phase-index-MASTER.discovery.md:L133 | PHASE 1G — cross-encoder reranker (`L183–206`) |
| MASTER-E096 | E/SUMMARY | Experiment: feature-gated `splade` evaluation comparing `prithivida/Splade_PP_en_v1` with `naver/splade-v3` (`L208–210`). | 01-phase-index-MASTER.discovery.md:L137 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E097 | E/SUMMARY | Decision gates: drop if recall@10 improvement over BM25 is <5% or Wilcoxon `p ≥ 0.05`; early-drop before recall if programming-identifier vocabulary coverage <0.3 because BERT WordPiece has 30,522 non-code-specific tokens (`L210–212`). | 01-phase-index-MASTER.discovery.md:L138 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E098 | E/SUMMARY | 1H.1 build isolation: `splade.rs` behind `cfg(feature="splade")`; `ort` remains unconditional for GraphCodeBERT/Granite; only SPLADE session construction is gated (`L214–215`). | 01-phase-index-MASTER.discovery.md:L139 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E099 | E/SUMMARY | 1H.2 tests: finite/nonnegative sparse expansion, empty/long query behavior, RRF changes versus BM25 alone, and recall integration (`L216–217`). | 01-phase-index-MASTER.discovery.md:L140 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E100 | E/SUMMARY | 1H.3 variants: `PrePooled` for PP_en_v1; `RawMLM` for splade-v3 with max-pool + ReLU + `log1p` in Rust; tree-sitter identifier extraction for `vocab_coverage()`; weighted RRF fusion (`L218–220`). | 01-phase-index-MASTER.discovery.md:L141 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E101 | E/SUMMARY | 1H.4 measurement: vocabulary fitness, model selection (Jaccard only for quantization regression and recall@10 for A/B model comparison), 50-query Wilcoxon test, and keep only when mean delta ≥5% and `p<0.05` (`L221–223`). | 01-phase-index-MASTER.discovery.md:L142 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-D102 | D/SUMMARY | Early-drop completion: phase can complete without recall measurement; feature remains disabled by default while module is preserved for later production-data reevaluation (`L224–225`). | 01-phase-index-MASTER.discovery.md:L143 | PHASE 1H — conditional SPLADE keyword expansion (`L208–225`) |
| MASTER-E103 | E/SUMMARY | Dependency: requires 1E and 1G; add the missing `1G→1I` dependency edge before implementation (`L227–228`). | 01-phase-index-MASTER.discovery.md:L147 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E104 | E/SUMMARY | 1I.1 breaker classification: transient errors trip after three failures; permanent errors trip after one; exponential backoff 30/60/120 seconds capped, with deterministic jitter from `model_id`; half-open needs two consecutive successes (`L230–233`). | 01-phase-index-MASTER.discovery.md:L148 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E105 | E/SUMMARY | 1I.1 safe recovery: a half-open probe creates a fresh ONNX session and never reuses a possibly corrupt one; clock injected; breaker state is not checkpointed and restarts closed (`L232–234`). | 01-phase-index-MASTER.discovery.md:L149 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E106 | E/SUMMARY | 1I.2 modes: `SearchMode::{HYBRID,SPARSE_ONLY,BM25_ONLY}` (`L235`). | 01-phase-index-MASTER.discovery.md:L150 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E107 | E/SUMMARY | 1I.2 cascade: GraphCodeBERT failure → text-only; Granite failure → code-only; both → `SPARSE_ONLY`; SPLADE failure → `BM25_ONLY`; reranker failure → no reranking; mode restores after a successful half-open probe; every change journals `knowledge_degradation` (`L235–238`). | 01-phase-index-MASTER.discovery.md:L151 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E108 | E/SUMMARY | 1I.3 validation: under 1 ms; reject empty queries, >10K characters, null bytes, and control characters except newline (`L239`). | 01-phase-index-MASTER.discovery.md:L152 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E109 | E/SUMMARY | 1I.4 readiness: load ONNX, open SQLite, run embed+BM25+rerank canary within five seconds; `READY`, `DEGRADED` (which invariantly means `BM25_ONLY`), or `UNAVAILABLE` (blocks all FSM transitions) (`L240–242`). | 01-phase-index-MASTER.discovery.md:L153 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E110 | E/SUMMARY | 1I.4 drift guard: cosine similarity between current and reference embedding for `"fn main()"` below 0.85 forces `DEGRADED`; persist reference vector in `index_version.toml` (`L242–243`). | 01-phase-index-MASTER.discovery.md:L154 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E111 | E/SUMMARY | 1I.5 overload behavior: one concurrent cross-encoder, queue four; sixth concurrent request is returned unranked; compatible requests batch (`L244`). | 01-phase-index-MASTER.discovery.md:L155 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E112 | E/SUMMARY | 1I.6 observability: debug `QueryTrace`/`StageTrace` for nine stages—`bm25`, `splade`, `code_vector`, `text_vector`, `pagerank`, `catrag`, `arag`, `rerank`, `token_fit`; aggregate p50/p99 to SSB only at session end, not per query (`L245–246`). | 01-phase-index-MASTER.discovery.md:L156 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E113 | E/SUMMARY | 1I.7 migration: index-version mismatch rebuilds and archives old index to `index_backup_{version}/`; missing version file defaults to v1 (`L247–248`). | 01-phase-index-MASTER.discovery.md:L157 | PHASE 1I — retrieval resilience/readiness/tracing (`L227–248`) |
| MASTER-E114 | E/SUMMARY | Dependency: requires 1E, 1G, and 3J (`L250–251`). | 01-phase-index-MASTER.discovery.md:L161 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E115 | E/SUMMARY | 3K.1 adaptive chunking: replace fixed 50-line BM25 windows with DKG/tree-sitter AST splitting; Rust boundaries `fn`/`impl`/`struct`/`enum`/`mod`, Python/TS/Go `def`/`class`; target 200–800 tokens with 50-token overlap; split Markdown by headings (`L253–255`). | 01-phase-index-MASTER.discovery.md:L162 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E116 | E/SUMMARY | 3K.2 deterministic multi-query: no LLM; keyword extraction, CamelCase/snake_case variants, synonyms such as error→failure/exception; merge and deduplicate by `chunk_id` (`L256–257`). | 01-phase-index-MASTER.discovery.md:L163 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E117 | E/SUMMARY | 3K.3 HyDE: sidecar creates a 128-token hypothetical answer at temperature 0.7, embeds it, searches vectors, then reranks against the original query; cache five minutes; sidecar outage degrades gracefully (`L258–260`). | 01-phase-index-MASTER.discovery.md:L164 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E118 | E/SUMMARY | 3K.4 intent tiering: `FAST`/`STANDARD`/`DEEP`, auto-detected from query length and keywords; distinct from 1I's fault-degradation `SearchMode`; `AUTO` remains backward compatible (`L261–262`). | 01-phase-index-MASTER.discovery.md:L165 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E119 | E/SUMMARY | 3K.5 quality bench: 30 queries (10 simple/10 code/10 complex), measuring Recall@5, Recall@10, MRR, and per-tier latency in `phase_3K_rag.toml` (`L263–264`). | 01-phase-index-MASTER.discovery.md:L166 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E120 | E/SUMMARY | 3K.6: A-RAG keyword score `Σ count(k,text)·len(k)`, 3× semantic oversampling, progressive disclosure through snippets, goal ≥50% token savings (`L265–267`). | 01-phase-index-MASTER.discovery.md:L167 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E121 | E/SUMMARY | 3K.7: CatRAG symbolic anchoring with `epsilon=0.2` weak DKG-entity seeds and `beta=2.5` verified-edge boost (`L268–269`). | 01-phase-index-MASTER.discovery.md:L168 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-D122 | D/SUMMARY | 3K.7 Innovation 2: LLM edge scoring deferred because it is too expensive for code search (`L269–270`). | 01-phase-index-MASTER.discovery.md:L169 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E123 | E/SUMMARY | 3K.8 acceptance: compare against 3K.5 baseline and allow no metric regression above 5% (`L271`). | 01-phase-index-MASTER.discovery.md:L170 | PHASE 3K — RAG production hardening (logically in knowledge pipeline) (`L250–271`) |
| MASTER-E124 | E/SUMMARY | Goal: core GRC state machine, reasoning budget, and grounding-format validation (`L277–278`). | 01-phase-index-MASTER.discovery.md:L174 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E125 | E/SUMMARY | State type: `GrcState::{Observe,Hypothesize,PlanAction,Act,Verify,Converge,Status}` with `allowed_next()` (`L279–280`). | 01-phase-index-MASTER.discovery.md:L175 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E126 | E/SUMMARY | Step/result types: `ThinkStep {step_type,content,topic,grounding: Option<String>,conclusion,scope: Option<Vec<String>>}`; `ThinkResult {step,budget_remaining,grounding_level,warnings}` (`L280–282`). | 01-phase-index-MASTER.discovery.md:L176 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E127 | E/SUMMARY | Budget: `GrcBudget {max_calls,max_chars,calls_used,chars_used}` and forced convergence when remaining calls ≤1 (`L282`). | 01-phase-index-MASTER.discovery.md:L177 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E128 | E/SUMMARY | Grounding/error taxonomy: ordered `GroundingLevel::{Uncertain,High,Verified}` and `GrcError::{UngroundedStep,BudgetExhausted,InvalidTransition,MalformedGrounding}` (`L283–284`). | 01-phase-index-MASTER.discovery.md:L178 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E129 | E/SUMMARY | Stable trait: `GrcEngine::{think,set_budget,get_budget,reset_topic}` implemented now; `verify_claim` and `ground_plan` declared as `unimplemented!()` stubs for 2B so the trait shape remains stable (`L285–286`). | 01-phase-index-MASTER.discovery.md:L179 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E130 | E/SUMMARY | 2A.1 minimal crate: dependencies only on core, `thiserror`, and `serde`; types in `types.rs`; `BasicGrcEngine` in `engine.rs`; per-topic state in `HashMap<String,TopicState>` (`L288–289`). | 01-phase-index-MASTER.discovery.md:L180 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E131 | E/SUMMARY | 2A.2 dependency gate: no providers, tools, or knowledge dependencies (`L290`). | 01-phase-index-MASTER.discovery.md:L181 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E132 | E/SUMMARY | 2A.3 tests: 12 covering grounding-required observe, forced convergence, per-topic isolation, invalid transitions, convergence threshold, malformed grounding, free `Status`, default budget, first-step observe, and scope present/missing/optional (`L291–294`). | 01-phase-index-MASTER.discovery.md:L182 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E133 | E/SUMMARY | 2A.4 transition rules: default `Observe→Hypothesize→PlanAction→Act→Verify`; `Status` and `Converge` always permitted; first step must be `Observe` (`L295–296`). | 01-phase-index-MASTER.discovery.md:L183 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E134 | E/SUMMARY | 2A.4 grounding parser: parse `type:reference`; `test`/`search` → `Verified`, `file`/`grep`/`tool` → `High`, `training` → `Uncertain` (`L296–298`). | 01-phase-index-MASTER.discovery.md:L184 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E135 | E/SUMMARY | 2A.4 budget/state behavior: check before increment; use `saturating_add`; topics isolated; missing scope enumeration warns (`L298`). | 01-phase-index-MASTER.discovery.md:L185 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E136 | E/SUMMARY | 2A.5 SLO: validation below 5000 µs and memory below 1 KiB per topic (`L299`). | 01-phase-index-MASTER.discovery.md:L186 | PHASE 2A — Grounded Reasoning Chain state machine (`L277–299`) |
| MASTER-E137 | E/SUMMARY | Verification methods: six named binary MiniCheck-style methods—`FILE_CHECK`, `SOURCE_CHECK`, `TEST_CHECK`, `WEB_CHECK`, `API_CHECK`, `API_REGISTRY_CHECK` (`L301–304`). | 01-phase-index-MASTER.discovery.md:L190 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E138 | E/SUMMARY | Plan gate: a grounding sweep blocks `PLANNING→EXECUTING` if any claim is `UNGROUNDED` and resolves 2A uncertainty where possible (`L303–304`). | 01-phase-index-MASTER.discovery.md:L191 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E139 | E/SUMMARY | Design decisions: binary verdicts; structural-regex claim decomposition rather than NLI; enum-based method dispatch; `verified_at` from a `TimeSource`; add `tools/` to reasoning; breaking `ground_plan` result becomes `PlanGroundingResult`; `verify_claim` gains context; injectable `TestRunner` with production `CargoTestRunner` and test `MockTestRunner` (`L305–309`). | 01-phase-index-MASTER.discovery.md:L192 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E140 | E/SUMMARY | 2B.1 method metadata: `VerificationMethod` has six variants plus `requires_network`/`requires_tools`; typed method stored in `GroundingResult` and `VerificationResult`; `GroundingResult.verified_at: u64` (`L311–313`). | 01-phase-index-MASTER.discovery.md:L193 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E141 | E/SUMMARY | 2B.1 plan result: `PlanGroundingResult {claims,all_grounded,ungrounded,verify_needed}` (`L313–315`). | 01-phase-index-MASTER.discovery.md:L194 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E142 | E/SUMMARY | 2B.1 verifier abstractions: object-safe `Verifier`, `TestRunner`, `LocalVerifier` and `NetworkVerifier` stubs (`L314–315`). | 01-phase-index-MASTER.discovery.md:L195 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E143 | E/SUMMARY | 2B.2 dependency gate: prevent tools→reasoning circularity and transitive knowledge inclusion (`L316`). | 01-phase-index-MASTER.discovery.md:L196 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E144 | E/SUMMARY | 2B.3 tests: 16 for existing/missing files and source symbols, mock test execution, typed network-method errors without test networking, plan block/pass/empty-pass, uncertain auto-resolution, object safety, comment-vs-real grounding adversary, and stripping the `file:` prefix (`L317–321`). | 01-phase-index-MASTER.discovery.md:L197 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E145 | E/SUMMARY | 2B.4 local verifier: file check delegates to `tools::read_file`; source check reads then matches `(fn\|struct\|enum\|trait\|impl\|const\|static\|type\|mod)\s+{name}\b` (`L322–324`). | 01-phase-index-MASTER.discovery.md:L198 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E146 | E/SUMMARY | 2B.4 test verifier: `CargoTestRunner` runs through `BashSandbox::execute` with 30-second timeout (`L324–325`). | 01-phase-index-MASTER.discovery.md:L199 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-D147 | D/SUMMARY | 2B.4 network verifier: returns typed `VerificationError("not impl until Phase 4A IPC proxy")` (`L325`). | 01-phase-index-MASTER.discovery.md:L200 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E148 | E/SUMMARY | 2B.4 claim/plan processing: strip grounding prefix before verifier dispatch; extract `type:reference` claims by regex; classify; attempt FILE/SOURCE auto-resolution for training-tagged claims; return aggregate plan result (`L326–328`). | 01-phase-index-MASTER.discovery.md:L201 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E149 | E/SUMMARY | 2B.5 SLO/status: file check <2000 µs, source check <10000 µs for 1000 lines, 20-claim sweep <100000 µs; only 3/6 methods implemented and three stubbed at this phase (`L329–330`). | 01-phase-index-MASTER.discovery.md:L202 | PHASE 2B — grounding verification (`L301–330`) |
| MASTER-E150 | E/SUMMARY | Role specialization: six roles get independent budgets, step sequences, grounding constraints, and allowed verification methods (`L332–334`). | 01-phase-index-MASTER.discovery.md:L206 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E151 | E/SUMMARY | Config type: `AgentGrcConfig` fields `role`, `budget`, `required_steps`, `grounding_level`, `csq_required`, `allowed_methods`, `transition_overrides`, `require_scope_enumeration` (`L334–336`). | 01-phase-index-MASTER.discovery.md:L207 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E152 | E/SUMMARY | Transition invariant: per-role overrides are additive and can never remove a default transition (`L335–336`). | 01-phase-index-MASTER.discovery.md:L208 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E153 | E/SUMMARY | Role defaults: executor 8 calls/1200 chars and full chain; auditor 10/1200 with double-verify and anti-rubber-stamp; context compiler observe-only and only FILE/SOURCE checks; memory writer 2/400; prompt engineer 5/800; designer 8/1200 and distinct from executor because it emits `PipelineSpec` (`L337–340`). | 01-phase-index-MASTER.discovery.md:L209 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E154 | E/SUMMARY | TOML layering: `[grc.defaults]` plus `[agents.{role}.grc]`; role keys use underscores (for example `context_compiler`), not hyphens (`L341–342`). | 01-phase-index-MASTER.discovery.md:L210 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E155 | E/SUMMARY | Auditor anti-rubber-stamp: on auditor `Converge`, history must contain at least one `Verify` whose content begins `FINDING:`; otherwise `InsufficientFindings` (`L343–344`). | 01-phase-index-MASTER.discovery.md:L211 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E156 | E/SUMMARY | Grounding order invariant: enum order must be `Uncertain, High, Verified` or explicit comparison must enforce `Verified > Uncertain` (`L345–346`). | 01-phase-index-MASTER.discovery.md:L212 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E157 | E/SUMMARY | 2C.1 interfaces/errors: `GrcError::{InsufficientFindings,ConfigValidation}`; `AgentGrcConfigLoader::{load_config,validate_config}`; `GrcDefaults`; `TomlConfigLoader` stub; engine `with_agent_config`/`get_agent_config` (`L348–350`). | 01-phase-index-MASTER.discovery.md:L213 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E158 | E/SUMMARY | 2C.2 dependency gate: no new external dependency; core parses TOML and reasoning accepts typed data (`L351`). | 01-phase-index-MASTER.discovery.md:L214 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E159 | E/SUMMARY | 2C.3 tests: 15 for role defaults, double verification, observe-only mode, config rejection, rubber-stamp rejection, uncertainty rejection, method allowlist, layered merge, additive transitions, distinct designer, and per-role scope requirement (`L352–356`). | 01-phase-index-MASTER.discovery.md:L215 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E160 | E/SUMMARY | 2C.4 implementation: five config-validation rules; effective next states are `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})`; enforce minimum grounding; anti-rubber-stamp; merge defaults/overrides; assigning agent config resets topic state (`L357–359`). | 01-phase-index-MASTER.discovery.md:L216 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E161 | E/SUMMARY | Authority transition: 2C budget enforcement is authoritative in-process through Phases 0–3; at 4A agent-side enforcement becomes advisory and orchestrator enforcement authoritative (`L360–361`). | 01-phase-index-MASTER.discovery.md:L217 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E162 | E/SUMMARY | 2C.5 SLO: config load <1000 µs and routing overhead <10% (`L362`). | 01-phase-index-MASTER.discovery.md:L218 | PHASE 2C — per-agent GRC configuration (`L332–362`) |
| MASTER-E163 | E/SUMMARY | CSQ trigger: generate counterfactual safety questions for functions touching files, network, money, or authentication; allow at most two inline feasibility calls during `EXECUTING` (`L364–366`). | 01-phase-index-MASTER.discovery.md:L222 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E164 | E/SUMMARY | CSQ types: `CsqSeverity::{Critical,Warning}`; `CsqCheck {assumption,consequence,severity,guard_added}`; `CsqResult {function_id,checks}` where pass means every critical check gained a guard (`L367–368`). | 01-phase-index-MASTER.discovery.md:L223 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E165 | E/SUMMARY | Feasibility types: `FeasibilityMethod::SourceMatch` now, with `WebSearch`/`Grep`/`ApiRegistryCheck` added in 4F; `InlineFeasibilityResult {claim,method,outcome: Pass\|Fail\|Blocked}` (`L368–370`). | 01-phase-index-MASTER.discovery.md:L224 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E166 | E/SUMMARY | Feasibility state: `InlineFeasibilityState {checks_used,max_checks=2}` with `is_exhausted`, `record_check`, `reset` (`L370–371`). | 01-phase-index-MASTER.discovery.md:L225 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E167 | E/SUMMARY | Trait surface: `generate_csq(function_id,sensitive_ops)` and `check_inline_feasibility(claim,sources)` (`L372`). | 01-phase-index-MASTER.discovery.md:L226 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E168 | E/SUMMARY | Critical patterns: case-insensitive `fs::`, `file`, `write`, `delete`, `remove`, `net::`, `tcpstream`, `money`, `payment`, `auth`, `token`, `credential`, `password`; everything else warns (`L373–375`). | 01-phase-index-MASTER.discovery.md:L227 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E169 | E/SUMMARY | 2D.1–2D.3: add seven types, two trait methods, feasibility state; no dependency changes; ten tests for all outcomes, two-call budget, reset, and `CONTRADICTS_PLAN:` blocking marker (`L376–380`). | 01-phase-index-MASTER.discovery.md:L228 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E170 | E/SUMMARY | 2D.4 implementation: lowercase-pattern classification; source substring matching; `CONTRADICTS_PLAN:` → `Blocked`, signaling a `MAJOR` `ReplanRequest`; record count with saturating addition (`L381–384`). | 01-phase-index-MASTER.discovery.md:L229 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-D171 | D/SUMMARY | Orchestrator emission: actual `ReplanRequest` emission waits for 3A+ (`L383`). | 01-phase-index-MASTER.discovery.md:L230 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E172 | E/SUMMARY | 2D.5 SLO: CSQ <1000 µs, inline check <2000 µs, false-positive rate exactly zero (`L385`). | 01-phase-index-MASTER.discovery.md:L231 | PHASE 2D — counterfactual safety questions and inline feasibility (`L364–385`) |
| MASTER-E173 | E/SUMMARY | Nine states: `IDLE`, `PLANNING`, `FEASIBILITY_PROBE`, `TESTING_FIRST`, `EXECUTING`, `AUDITING`, `MEMORIZING`, `COMPLETE`, `CONFIGURING` (`L391–394`). | 01-phase-index-MASTER.discovery.md:L235 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E174 | E/SUMMARY | Recovery transitions: `AUDITING→EXECUTING` for blocking fixes and `AUDITING→PLANNING` for dead ends (`L394–395`). | 01-phase-index-MASTER.discovery.md:L236 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E175 | E/SUMMARY | 3A.1 state/persistence: transition matrix; `ReplanRequest::{MINOR,MAJOR}` capped at one per agent per phase; `token_budget_remaining()`; MessagePack state at `.elai_cc/sessions/{sid}/.fsm-state.msgpack` (`L397–399`). | 01-phase-index-MASTER.discovery.md:L237 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E176 | E/SUMMARY | 3A.2 phase-aware capability filters: PLANNING blocks edit/bash; EXECUTING blocks planning tools; AUDITING blocks edit/planning; MEMORIZING allows memory-bank tools only; CONFIGURING only config paths; COMPLETE read-only; IDLE unrestricted (`L400–402`). | 01-phase-index-MASTER.discovery.md:L238 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E177 | E/SUMMARY | Enforcement point: `ToolRegistry::execute()` must return `ToolBlockedByState`; filters are not advisory (`L402–403`). | 01-phase-index-MASTER.discovery.md:L239 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E178 | E/SUMMARY | 3A.3 seven evidence gates: IDLE→PLANNING requires knowledge `READY\|DEGRADED`; PLANNING→TESTING_FIRST requires zero ungrounded claims; TESTING_FIRST→EXECUTING requires tests fail for correct reasons; EXECUTING→AUDITING requires at least one FAIL→PASS; EXECUTING→PLANNING fast-track requires minor replan plus auditor validation; AUDITING→MEMORIZING requires zero blocks; MEMORIZING→next requires DKG + changelog (`L404–408`). | 01-phase-index-MASTER.discovery.md:L240 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E179 | E/SUMMARY | Gate error UX: `EvidenceError` includes actionable suggestions (`L408`). | 01-phase-index-MASTER.discovery.md:L241 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-D180 | D/SUMMARY | Feasibility transition gates: wait for 4F (`L409`). | 01-phase-index-MASTER.discovery.md:L242 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E181 | E/SUMMARY | 3A.4 human gate: mockable `InteractiveGate::present_gate(evidence)` yields `APPROVE`, `REJECT` (no transition), `MODIFY` (recompute evidence), or `REDIRECT` (valid target only); interactive mode has no timeout (`L410–412`). | 01-phase-index-MASTER.discovery.md:L243 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E182 | E/SUMMARY | 3A.5 risk UX: every transition has `LOW\|MEDIUM\|HIGH`; interactive pauses on medium/high and automatically advances low; overrides in `[fsm.risk_overrides]` (`L413–414`). | 01-phase-index-MASTER.discovery.md:L244 | PHASE 3A — full interactive FSM (`L391–414`) |
| MASTER-E183 | E/SUMMARY | Journal architecture: single-writer append-only SQLite/WAL inter-agent state; each `Ssb` instance permanently binds one `(project_id,session_id)` (`L416–419`). | 01-phase-index-MASTER.discovery.md:L248 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E184 | E/SUMMARY | Schema: `ssb`, `ssb_files` junction, `PRAGMA user_version` migrations, and r2d2 read pool whose `CustomizeConnection` enables foreign keys on every connection (`L418–420`). | 01-phase-index-MASTER.discovery.md:L249 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E185 | E/SUMMARY | 17 action types: `file_write`, `file_read`, `tool_result`, `test_outcome`, `finding`, `claim`, `checkpoint`, `model_swap`, `phase_transition`, `knowledge_degradation`, `isolation_metric`, `plan_update`, `replan_request`, `stub_reference`, `reasoning_trace`, `phase_complete`, `memory_monitoring` (`L421–423`). | 01-phase-index-MASTER.discovery.md:L250 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E186 | E/SUMMARY | SQLite pragmas/order: WAL, `synchronous=NORMAL`, auto-checkpoint 1000, busy timeout 5000, 64 MB negative cache size, foreign keys on, temp store memory; throughput expectation ~80K inserts/s (`L424–425`). | 01-phase-index-MASTER.discovery.md:L251 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E187 | E/SUMMARY | Conflict query: join `ssb_files.seq=ssb.seq`, filter project, and use `rarray()` for `IN` to preserve statement caching (`L426–427`). | 01-phase-index-MASTER.discovery.md:L252 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E188 | E/SUMMARY | 3B.1 payload/read-pool behavior: `Arc<[u8]>` for O(1) broadcast clones; disable idle timeout/max lifetime so default pool recycling does not close WAL (`L429–430`). | 01-phase-index-MASTER.discovery.md:L253 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E189 | E/SUMMARY | 3B.2 commit protocol: `BEGIN IMMEDIATE → INSERT ssb → INSERT ssb_files → COMMIT → broadcast`; broadcast only after commit; `created_at` from clock provider; SSB write completes before tool result dispatch; shutdown drains writes then exits (`L431–433`). | 01-phase-index-MASTER.discovery.md:L254 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E190 | E/SUMMARY | 3B.3 reads/immutability: WAL snapshot isolation; no update/delete APIs; debug update hook panics on mutation (`L434–435`). | 01-phase-index-MASTER.discovery.md:L255 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E191 | E/SUMMARY | 3B.4 typing/dependency boundary: typed MessagePack payloads for all 17 types, no opaque blobs; `SsbTraceable` lives in orchestrator to avoid dependency cycles; reasoning/tools emit IPC while orchestrator persists (`L436–438`). | 01-phase-index-MASTER.discovery.md:L256 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E192 | E/SUMMARY | Determinism: seed + injected clock yields byte-identical database dumps (`L439`). | 01-phase-index-MASTER.discovery.md:L257 | PHASE 3B — Shared State Board (`L416–439`) |
| MASTER-E193 | E/SUMMARY | Scope: spawn/lifecycle/checkpoint/crash recovery/supervision/anti-spiral; Phase 3 agents remain in-process Tier A until 4A Tier C (`L441–443`). | 01-phase-index-MASTER.discovery.md:L261 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E194 | E/SUMMARY | Port mapping: `WorkerRegistry→AgentPool`; UUID v4 from `EntropySource`; `WorkerStatus→AgentStatus` plus `Checkpointing`/`BudgetExceeded` and removal of `TrustRequired`; failure scenario maps to `AgentDeathClassification` (`L444–446`). | 01-phase-index-MASTER.discovery.md:L262 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E195 | E/SUMMARY | 3C.1 pool: `spawn`; sequential stub `spawn_parallel`; direct stub `communicate`; VRAM availability checked through `ModelPool` before spawn (`L447–452`). | 01-phase-index-MASTER.discovery.md:L263 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E196 | E/SUMMARY | Agent handle: `{agent_id,role,status,active_files,grc_config,scratch_dir}` (`L449`). | 01-phase-index-MASTER.discovery.md:L264 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E197 | E/SUMMARY | Eight roles: `Executor`, `Auditor`, `PromptEngineer`, `ContextCompiler`, `MemoryWriter`, `Researcher`, `MediaValidator`, `Designer`; designer differs by prompt/required steps despite shared model (`L450–452`). | 01-phase-index-MASTER.discovery.md:L265 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E198 | E/SUMMARY | 3C.2 lifecycle/deaths: `Spawning→Ready→Running→Finished\|Failed`; death classes `CLEAN_EXIT`, `CRASH`, `TIMEOUT` (heartbeat 5 s/15 s), `BUDGET_EXCEEDED` (`L453–455`). | 01-phase-index-MASTER.discovery.md:L266 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E199 | E/SUMMARY | Tier-A invisible walls: hidden paths return exactly the same `NotFound` as absent paths (`L454–455`). | 01-phase-index-MASTER.discovery.md:L267 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E200 | E/SUMMARY | Reasoning telemetry: at most one replan per phase; every GRC step writes `ReasoningTraceEntry` to SSB (`L455–456`). | 01-phase-index-MASTER.discovery.md:L268 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E201 | E/SUMMARY | 3C.3 checkpoint contents/location: `.elai_cc/sessions/{sid}/agents/{agent_id}/{state,grc_history,tool_results,output_draft}.msgpack` plus `checkpoint_ts` (`L457–459`). | 01-phase-index-MASTER.discovery.md:L269 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E202 | E/SUMMARY | Checkpoint triggers/durability: every GRC step, tool call, file edit, and before slow operations; atomic temp-write + fsync + rename (`L458–459`). | 01-phase-index-MASTER.discovery.md:L270 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E203 | E/SUMMARY | Channel checkpointability: single `Topic`/`LastValue` references allowed; `UntrackedValue` references fail with `CheckpointError::UntrackedValueReference` (`L460–461`). | 01-phase-index-MASTER.discovery.md:L271 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E204 | E/SUMMARY | 3C.4 recovery: classify death, journal it, respawn with the same UUID, inject structured checkpoint; first recovered GRC step grounds to `CHECKPOINT:{path}`; corrupt checkpoint warns and starts fresh rather than permanently failing (`L462–464`). | 01-phase-index-MASTER.discovery.md:L272 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E205 | E/SUMMARY | 3C.5 supervision: default max three restarts within a phase; actions `Escalate` or `SubstituteModel(ModelId)`; Phase 3 escalates; model substitution waits for 4G; budget exhaustion does not increment restart count; reset count at phase boundary (`L465–468`). | 01-phase-index-MASTER.discovery.md:L273 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E206 | E/SUMMARY | 3C.6 loop detection: Blake3 content hash of step type + topic + normalized content; three consecutive identical hashes trigger spiral handling—checkpoint, convergence signal, then permanent stop if another identical step; recover from N−3 checkpoint, not latest (`L469–472`). | 01-phase-index-MASTER.discovery.md:L274 | PHASE 3C — agent lifecycle, recovery, supervision, anti-spiral (`L441–472`) |
| MASTER-E207 | E/SUMMARY | Goal/guarantee: deterministic BSP macro-orchestration; strict determinism only with mocked agents and fixed seed because live-LLM timeout participation can vary (`L474–477`). | 01-phase-index-MASTER.discovery.md:L278 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E208 | E/SUMMARY | Removed primitive: `BinaryOp` dropped because concurrent writes are nondeterministic (`L478`). | 01-phase-index-MASTER.discovery.md:L279 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E209 | E/SUMMARY | `LastValue<T>`: one value; winner is highest/last SSB sequence (`L479`). | 01-phase-index-MASTER.discovery.md:L280 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E210 | E/SUMMARY | `Topic<T>`: accumulating mode persists `Vec<T>`; nonaccumulating mode snapshots then clears after snapshot and before task writes (`L480–481`). | 01-phase-index-MASTER.discovery.md:L281 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E211 | E/SUMMARY | `EphemeralValue<T>`: saturating per-superstep write count; zero at barrier means `MISSING`; clearing is journaled so SSB-only replay works (`L482–483`). | 01-phase-index-MASTER.discovery.md:L282 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E212 | E/SUMMARY | `UntrackedValue<T:DeepSizeOf>`: memory-only, never serialized; force-clear beyond 2× default 10 MB limit; do not use shallow `size_of_val` (`L484–485`). | 01-phase-index-MASTER.discovery.md:L283 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E213 | E/SUMMARY | 3D.1 registry: read, write, scheduling snapshot, ephemeral clear, untracked force-clear, deep-size measurement (`L486–488`). | 01-phase-index-MASTER.discovery.md:L284 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E214 | E/SUMMARY | 3D.2 loop order: snapshot → prepare tasks → clear nonaccumulating topics → parallel execution → sequence-sorted apply → one-pass deferred phase → clear ephemerals → evidence gate → checkpoint only after gate passes; cap superstep attempts (`L489–492`). | 01-phase-index-MASTER.discovery.md:L285 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E215 | E/SUMMARY | 3D.3 deferred nodes: `NodeSpec.is_deferred`, same-superstep post-barrier, one pass with no cascade, `all_inputs_satisfied` gate (`L493–494`). | 01-phase-index-MASTER.discovery.md:L286 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E216 | E/SUMMARY | 3D.4 checkpoint/replay: serialize `HashMap<ChannelId,ChannelState>` except untracked values; separate `memory_monitoring` event; recovery loads checkpoint then replays SSB in sequence, reconstructing ephemeral counts and topic-boundary clears (`L495–498`). | 01-phase-index-MASTER.discovery.md:L287 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E217 | E/SUMMARY | 3D.5 real parallelism: upgrade pool to `tokio::JoinSet`, per-agent heartbeat timeout, `max_concurrent=floor(available_vram/per_agent_vram)`, and apply results in SSB order rather than completion order (`L499–501`). | 01-phase-index-MASTER.discovery.md:L288 | PHASE 3D — BSP supersteps and typed channels (`L474–501`) |
| MASTER-E218 | E/SUMMARY | Eight slots: `SystemPrompt`, `Rules`, `PhaseSpec`, `RepoMap`, `CodeContext`, `SsbEntries`, `GrcHistory`, `OutputReserve` (`L503–506`). | 01-phase-index-MASTER.discovery.md:L292 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E219 | E/SUMMARY | Allocation: exact-measure slots 1–3; reserve `max(configured,2000)` output tokens; split dynamic remainder repo map 15%, code context 40%, SSB 25%, GRC 20% (`L506–507`). | 01-phase-index-MASTER.discovery.md:L293 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E220 | E/SUMMARY | Floors: SSB ≥200, GRC ≥500, output ≥2000; steal from code context then repo map; dynamic remainder <700 is `BudgetInfeasible` (`L508–509`). | 01-phase-index-MASTER.discovery.md:L294 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E221 | E/SUMMARY | Cache stability: breakpoints after PhaseSpec and CodeContext; use deterministic xxhash/FxHash, never `DefaultHasher` (`L510–511`). | 01-phase-index-MASTER.discovery.md:L295 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E222 | E/SUMMARY | SSB relevance: file overlap `\|entry.files ∩ agent.files\| / \|agent.files\|`; sort score descending then sequence descending; omit zero score; empty agent file set safely scores zero (`L512–513`). | 01-phase-index-MASTER.discovery.md:L296 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E223 | E/SUMMARY | Three reductions: `FilteredByRelevance` (stricter overlap/same-topic GRC), `ReducedGranularity` (denser map, signatures/doc comments), `ReducedScope` (current file/direct deps only); journal `context_reduction` (`L514–517`). | 01-phase-index-MASTER.discovery.md:L297 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E224 | E/SUMMARY | Terminal handling: interactive escalates with breakdown; autonomous splits task in 4G (`L518`). | 01-phase-index-MASTER.discovery.md:L298 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E225 | E/SUMMARY | Counting/SLO: sampling estimate for allocation, exact count for validation; under 50 ms (`L519–520`). | 01-phase-index-MASTER.discovery.md:L299 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-Q226 | Q/SUMMARY | Step detail: only “Steps 3E.1–3E.7 as numbered” appears; individual tasks require `phase-3E.txt` (`L521`). | 01-phase-index-MASTER.discovery.md:L300 | PHASE 3E — fixed-slot context budget allocator (`L503–521`) |
| MASTER-E227 | E/SUMMARY | Rules-as-data: deterministic enforcement from SQLite rather than prompt rules (`L523–524`). | 01-phase-index-MASTER.discovery.md:L304 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E228 | E/SUMMARY | Dependency architecture: rules does not depend directly on tree-sitter; it consumes knowledge through `AstEvaluator` (`L524–526`). | 01-phase-index-MASTER.discovery.md:L305 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E229 | E/SUMMARY | Evaluation order: cheapest first, short-circuit on block: `METRIC_GATE → GREP_PATTERN → AST_PATTERN → COMPOSITE` (`L526–527`). | 01-phase-index-MASTER.discovery.md:L306 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E230 | E/SUMMARY | Predicates: `AST_PATTERN`, `GREP_PATTERN`, `SCHEMA_REQUIRED` (escalates provider tier), `METRIC_GATE`, and depth≤8 `COMPOSITE` `AND\|OR\|NOT`; verdict `ALLOW\|WARN\|BLOCK\|ESCALATE(target)` (`L528–530`). | 01-phase-index-MASTER.discovery.md:L307 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E231 | E/SUMMARY | 3F.1 schema: `rules`, `rule_metrics`, `contract_tests`, `violation_patterns`, `schema_version` (`L531–532`). | 01-phase-index-MASTER.discovery.md:L308 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E232 | E/SUMMARY | 3F.2 admission compile: validate regex, AST query, and JSON schema at insertion; `AstEvaluator` defined for knowledge implementation (`L532–534`). | 01-phase-index-MASTER.discovery.md:L309 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E233 | E/SUMMARY | 3F.3 object-safe API: `RuleEngine::{evaluate,get_active_rules,compile_predicate}` (`L535`). | 01-phase-index-MASTER.discovery.md:L310 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E234 | E/SUMMARY | 3F.4 lifecycle: `PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED` (`L536`). | 01-phase-index-MASTER.discovery.md:L311 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E235 | E/SUMMARY | Promotion gates: at least three contract tests; human/independent second-auditor approval, with finding-source auditor forbidden to self-approve; shadow→active requires FPR<5%, TPR>80%, counterfactual causal rate>50%, and no safety regressions (`L536–539`). | 01-phase-index-MASTER.discovery.md:L312 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E236 | E/SUMMARY | Immutable exception: `mutable=0` rules do not auto-retire and need not pass the causal-rate gate (`L539`). | 01-phase-index-MASTER.discovery.md:L313 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E237 | E/SUMMARY | 3F.5 interpreter: compile regex once; depth guard 8; NaN/Inf guard; escalation route is shadow→auditor, immutable→human, tier chain→next-tier rule (`L540–541`). | 01-phase-index-MASTER.discovery.md:L314 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E238 | E/SUMMARY | 3F.6 activation: phase bitmask plus domain detection; engineering/security always active; indexed case-insensitive domain query (`L542–543`). | 01-phase-index-MASTER.discovery.md:L315 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E239 | E/SUMMARY | 3F.7 discovery: orchestrator normalizes and Blake2s-hashes recurring violations; three occurrences auto-propose after compile validation; auditor-origin rules remain proposed; orchestrator, not auditor, creates ALLOW counterexamples (`L544–547`). | 01-phase-index-MASTER.discovery.md:L316 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E240 | E/SUMMARY | 3F.8 seeds: `rules_seed.db` with at least 33 rules, including three post-tool completeness grep rules; web fetch image/table warnings and file-read TODO/FIXME info findings; immutable seeds protected (`L548–549`). | 01-phase-index-MASTER.discovery.md:L317 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E241 | E/SUMMARY | 3F.9 test scale: at least 254 static tests plus 256 property cases/rule for violation/valid generators, mutation invariance, fix mutation, De Morgan, associativity; shrunk counterexamples reinserted with `source=GENERATED` (`L550–552`). | 01-phase-index-MASTER.discovery.md:L318 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E242 | E/SUMMARY | 3F.10 SLO: grep median <1 ms, metric <0.1 ms, mixed ten-rule evaluation <10 ms (`L553`). | 01-phase-index-MASTER.discovery.md:L319 | PHASE 3F — deterministic, self-improving rules engine (`L523–553`) |
| MASTER-E243 | E/SUMMARY | Two-level artifact: skeleton ≤2–4K tokens always in Slot 3 plus per-phase elaborations, at `.elai_cc/pipelines/{id}/skeleton.toml` and `phases/{phase_id}.toml`; every update journals `plan_update` (`L555–558`). | 01-phase-index-MASTER.discovery.md:L323 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E244 | E/SUMMARY | Storage clarification: no `plan_phases` table; queries use SSB action filter plus JSON payload (`L558–559`). | 01-phase-index-MASTER.discovery.md:L324 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E245 | E/SUMMARY | Types: ordered `PlanSkeleton`; 120-character-goal `PhaseEntry` with typed inputs/outputs, constraints, dependency links, versions, output type/visual check; `ContractRef`; `PhaseDetail` with skeleton version, contracts, implemented contracts, structured declared interfaces, content/token count; `ExtractedContracts` (`L560–564`). | 01-phase-index-MASTER.discovery.md:L325 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E246 | E/SUMMARY | 3G.1: TOML serde and goal-length validation (`L565–566`). | 01-phase-index-MASTER.discovery.md:L326 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E247 | E/SUMMARY | 3G.2 size/write gate: estimated precheck, exact count at 3600 safety margin, hard reject above 4096; every write emits plan update; missing detail emits `stub_reference`; contract changes require versioning (`L567–569`). | 01-phase-index-MASTER.discovery.md:L327 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E248 | E/SUMMARY | 3G.3 extraction: inputs are union of dependency outputs; outputs union of dependents' inputs; shared constraints are global+phase; conflicting contract names across dependencies detected (`L570–571`). | 01-phase-index-MASTER.discovery.md:L328 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E249 | E/SUMMARY | 3G.4 context: skeleton always, active detail, extracted contracts, and one-hop adjacent contracts in Slot 3; reduce granularity on overflow; skeleton-version mismatch marks stale (`L572–574`). | 01-phase-index-MASTER.discovery.md:L329 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E250 | E/SUMMARY | 3G.5 coherence: implemented contracts must superset extracted contracts; declared interfaces must be subset of skeleton; contract change must increment both phase and skeleton version; failures block and journal finding (`L575–579`). | 01-phase-index-MASTER.discovery.md:L330 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E251 | E/SUMMARY | 3G.6 propagation: BFS `depended_by` marks `needs_review`; Kahn cycle detection O(V+E); checked version increment; split preserves contracts and depth≤2 to match 4G (`L580–583`). | 01-phase-index-MASTER.discovery.md:L331 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E252 | E/SUMMARY | 3G.7 acceptance: five-phase E2E verifying budget/extraction/coherence/propagation/version/journal, deterministic 100 times (`L584–585`). | 01-phase-index-MASTER.discovery.md:L332 | PHASE 3G — hierarchical plan management (`L555–585`) |
| MASTER-E253 | E/SUMMARY | Goal: DAG-versioned sessions with structurally lossless three-pass trimming; stubs are recoverable by SSB sequence (`L587–589`). | 01-phase-index-MASTER.discovery.md:L336 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-Q254 | Q/SUMMARY | Required amendments: per-message `snapshot_context` rather than snapshot blobs; branch-op action type must reuse checkpoint or add `branch_abandon`; merged-branch cleanup must be logical filtering, never physical deletion (`L590–592`). | 01-phase-index-MASTER.discovery.md:L337 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E255 | E/SUMMARY | Schema: snapshot UUID/parent/FSM/phase/step/findings/code changes/tests/commits/tag; context row role constrained to system/user/assistant/tool plus content/stub fields; partial index on stubbed rows (`L593–596`). | 01-phase-index-MASTER.discovery.md:L338 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E256 | E/SUMMARY | 3H.1 DB safety: default journal mode, not WAL, because session has one writer; integrity check on open; foreign keys (`L597–599`). | 01-phase-index-MASTER.discovery.md:L339 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E257 | E/SUMMARY | 3H.2 CRUD/traversal: transactional save, sequence-ordered joined load, latest snapshot, cycle-guarded parent chain; ~50K batch inserts/s (`L600–601`). | 01-phase-index-MASTER.discovery.md:L340 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E258 | E/SUMMARY | 3H.3 trimming: stub tool results over configurable threshold constrained to [100,1000]; stub records original type/token count, Blake3 summary hash, SSB sequence; validate sequence exists or un-stub; never stub user/assistant or cache-prefix slots 1–3; target tokens = model context − allocated slots − output reserve (`L602–606`). | 01-phase-index-MASTER.discovery.md:L341 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E259 | E/SUMMARY | 3H.4 branching: maximum three active branches; merge creates a new trunk snapshot from winner tip; immutable DAG parents; abandoned branches tagged, not deleted; overlapping modified files produce conflict (`L607–609`). | 01-phase-index-MASTER.discovery.md:L342 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E260 | E/SUMMARY | 3H.5 restore: lookup by SSB sequence, verify Blake3, restore content/flags; pruned SSB is a typed nonpermanent error (`L610–611`). | 01-phase-index-MASTER.discovery.md:L343 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E261 | E/SUMMARY | 3H.6 lifecycle: trigger handoff at 85% token usage; when branches exist load branch tip, not trunk handoff; commits persist; 100-run deterministic test with seeded providers (`L612–614`). | 01-phase-index-MASTER.discovery.md:L344 | PHASE 3H — DAG session handoff and lossless trimming (`L587–614`) |
| MASTER-E262 | E/SUMMARY | Goal/oracle constraint: prove FSM+SSB+BSP using an infallible digital-sibling reference model whose LOC is <30% of real FSM (`L616–618`). | 01-phase-index-MASTER.discovery.md:L348 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E263 | E/SUMMARY | Test infrastructure: VCR at HTTP transport (not provider trait), trajectory `TraceOps` record/replay, and SQLite VFS write/fsync interception subject to spike (`L619–621`). | 01-phase-index-MASTER.discovery.md:L349 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E264 | E/SUMMARY | 3I.1 platform spike: evaluate `madsim 0.2.34` on Windows with feature-gated dependency swap; document manual deterministic scheduler if it fails (`L622–624`). | 01-phase-index-MASTER.discovery.md:L350 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E265 | E/SUMMARY | 3I.2 model: `SimulatorRuntime`; proptest reference-state-machine implementation; SUT returned by value; ten-event `FsmEvent`, including composite `SuperstepBoundary` and post-recovery `AgentCleanup` (`L625–628`). | 01-phase-index-MASTER.discovery.md:L351 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E266 | E/SUMMARY | 3I.3 build gate: zero regressions and no warnings above baseline (`L629`). | 01-phase-index-MASTER.discovery.md:L352 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E267 | E/SUMMARY | 3I.4 state testing: sequences 1..200; 10,000 CI cases with 10-minute limit, 256 local; seed regression corpus through synthetic failure; `check_determinism` takes bare static fn, never closure (`L630–633`). | 01-phase-index-MASTER.discovery.md:L353 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E268 | E/SUMMARY | 3I.5 BSP proof: 100 identical checkpoints; untracked excluded; zero-write ephemeral cleared; topics ordered by SSB sequence; two-phase reset represented; entropy grep over orchestrator must return zero raw time/random calls (`L634–637`). | 01-phase-index-MASTER.discovery.md:L354 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E269 | E/SUMMARY | 3I.6 faults: 30% packet loss, uniform 0..500 ms latency, asymmetric link clogging, SIGKILL, correlated burst loss 100% for five then zero (`L638–639`). | 01-phase-index-MASTER.discovery.md:L355 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E270 | E/SUMMARY | 3I.7 adversarial space: anti-spiral, gate bypass, context overflow, SSB conflict, demonic scheduling (`L640–641`). | 01-phase-index-MASTER.discovery.md:L356 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E271 | E/SUMMARY | 3I.8: implement reference/bridge/invariants and cross-platform LOC ratio check (`L642–643`). | 01-phase-index-MASTER.discovery.md:L357 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E272 | E/SUMMARY | 3I.9 acceptance: `phase_3I.toml` state coverage >80%, determinism 100%, regression count (`L644`). | 01-phase-index-MASTER.discovery.md:L358 | PHASE 3I — deterministic simulation testing (`L616–644`) |
| MASTER-E273 | E/SUMMARY | Backend: vLLM over OpenAI-compatible HTTP enables layer-level DoLa, TurboQuant KV, and ActAdd that Ollama cannot expose (`L646–649`). | 01-phase-index-MASTER.discovery.md:L362 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E274 | E/SUMMARY | Hardware envelope: dev Ryzen 7800X3D/64 GB/RTX 3060 12 GB, primary Gemma4 E2B ~2B, target Gemma4 31B ~18 GB Q4 on 48 GB GPU; CPU offload 0.5–2 tok/s is enough for artifacts but not EAGLE-3 training (`L650–652`). | 01-phase-index-MASTER.discovery.md:L363 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E275 | E/SUMMARY | Installed/model assumptions: `gemma4:31b` 19 GB Q4/262144 context; `gemma4:e2b` 7.2 GB Q4_K_M; `gemma3:4b` fallback; vLLM 0.19.0 in WSL2 `~/vllm-env/` (`L653–654`). | 01-phase-index-MASTER.discovery.md:L364 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E276 | E/SUMMARY | 3J.1: `VllmProvider` for `/v1/completions`, `/v1/models`, SSE streaming (`L655–656`). | 01-phase-index-MASTER.discovery.md:L365 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E277 | E/SUMMARY | 3J.2: `InferenceBackend::{Ollama,Vllm}`, tensor parallelism, fallback with warning (`L657`). | 01-phase-index-MASTER.discovery.md:L366 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E278 | E/SUMMARY | 3J.3 DoLa: JSD-selected premature layer, plausibility alpha 0.1, model-specific buckets, per-role toggle, guard below 1B parameters, degraded top-N-logprobs path (`L658–660`). | 01-phase-index-MASTER.discovery.md:L367 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E279 | E/SUMMARY | 3J.5 ActAdd: mid-layer contrast `h_A^l=h+^l−h−^l`, sharing DoLa hook path (`L663–664`). | 01-phase-index-MASTER.discovery.md:L369 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E280 | E/SUMMARY | 3J.6: GPU multiplexing with 1 GB margin and `ModelPool.can_fit_onnx()` (`L665`). | 01-phase-index-MASTER.discovery.md:L370 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E281 | E/SUMMARY | 3J.7–3J.8: FSM-driven predictive preload and per-agent/per-phase workload profiles in SSB (`L666–667`). | 01-phase-index-MASTER.discovery.md:L371 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E282 | E/SUMMARY | 3J.10–3J.13: port 1751-LOC MCP bridge; static hooks T1<1 ms/T2<10 ms/T3<5 ms and no hot-path LLM; Windows AppContainer/job limits/self-mod locks; complete typed SSB actions (`L670–673`). | 01-phase-index-MASTER.discovery.md:L373 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E283 | E/SUMMARY | 3J.14/15: validate Gemma4 31B CPU offload for steering vectors, DoLa, and codebook generation; numbering moved from 3J.14 to 3J.15 (`L674, L682`). | 01-phase-index-MASTER.discovery.md:L374 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-C284 | C/SUMMARY | Added composition: Python Anthropic-compatible composed inference sidecar auto-started by CLI; `--local` full interactive mode; `InferenceDispatcher` with API/local implementations, FSM tool filtering, backend mode `local\|api\|hybrid\|auto`; wiring substeps foundation/budget/rules/session/plan/stubs/E2E including Ollama fallback, retrieval prompt injection, release build, one `elai.bat` command (`L675–681`). | 01-phase-index-MASTER.discovery.md:L375 | PHASE 3J — vLLM, advanced providers, plugins, orchestration hardening (`L646–682`) |
| MASTER-E285 | E/SUMMARY | Isolation boundary: every agent is an isolated OS process; orchestrator-agent IPC uses Unix sockets or named pipes; all Phase 4 subphases must cross this boundary (`L688–690`). | 01-phase-index-MASTER.discovery.md:L379 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E286 | E/SUMMARY | Framing: one-byte version + four-byte length + Postcard payload; absolute frame cap 10 MB; per-type caps heartbeat <1 KiB, think step <64 KiB, tool result <10 MB, response <context limit (`L691–692`). | 01-phase-index-MASTER.discovery.md:L380 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E287 | E/SUMMARY | 15 messages: `AgentRequest`, `AgentResponse`, `InferRequest`, `InferResponse`, `ToolRequest`, `ToolResult`, `BashRequest`, `BashResult`, `GroundingRequest`, `GroundingResult`, `ThinkStep`, `CheckpointSync`, `AgentHeartbeat`, `AgentKill`, `ReplanRequest` (`L693–695`). | 01-phase-index-MASTER.discovery.md:L381 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E288 | E/SUMMARY | BLOCK-1: Linux seccomp profiles `IPC_BASELINE` for all roles and `NETWORK_EXTRA` only for researcher; BPF checks socket-domain arguments, not blanket socket permission (`L696–698`). | 01-phase-index-MASTER.discovery.md:L382 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E289 | E/SUMMARY | BLOCK-2: researcher networking uses veth+iptables with zero capabilities, explicitly no `CAP_NET_RAW` (`L699`). | 01-phase-index-MASTER.discovery.md:L383 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E290 | E/SUMMARY | BLOCK-3/4: agent binary has no LLM client, `reqwest`, or providers dependency; all inference is proxied; orchestrator externally counts think steps and owns GRC budget while agent budget is advisory (`L700–702`). | 01-phase-index-MASTER.discovery.md:L384 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E291 | E/SUMMARY | BLOCK-5: IPC sockets under 0700 `/tmp/elai_cc_{session}/`, peer credentials verified on accept, socket removed on agent death (`L703–704`). | 01-phase-index-MASTER.discovery.md:L385 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E292 | E/SUMMARY | BLOCK-6: auditor rules stay proposed until approved by a separate auditor (`L705`). | 01-phase-index-MASTER.discovery.md:L386 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E293 | E/SUMMARY | BLOCK-7: Windows prefers AppContainer; restricted token + ACL deny is documented as weaker fallback (`L706–707`). | 01-phase-index-MASTER.discovery.md:L387 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E294 | E/SUMMARY | BLOCK-8: self-mod locks enforced by canonical-realpath IPC check, read-only filesystem mount, and commit diff filter (`L708–709`). | 01-phase-index-MASTER.discovery.md:L388 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E295 | E/SUMMARY | BLOCK-9: adversarial IPC battery (`L710`). | 01-phase-index-MASTER.discovery.md:L389 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E296 | E/SUMMARY | Dispatch authority: `AgentDispatchConfig {role,model_preference,tool_filter,rule_domains,grc_config,sandbox_config}` is authoritative, including IPC whitelist; role alone is insufficient; feasibility executor override is GRC 3/600, only stub-write/bash, 25/30-second timeouts (`L711–714`). | 01-phase-index-MASTER.discovery.md:L390 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E297 | E/SUMMARY | Swap durability: `phase_complete` SSB row is atomic boundary marker and fsyncs before model unload (`L715`). | 01-phase-index-MASTER.discovery.md:L391 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E298 | E/SUMMARY | 4A gates/SLO: skeleton, compile, functional isolation, adversarial IPC, implementation, measurements; IPC p99 <1 ms, spawn <100 ms, memory <30 MB, overhead <5% (`L716–717`). | 01-phase-index-MASTER.discovery.md:L392 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-D299 | D/SUMMARY | ShortCoder placement: wait for 4B; orchestrator must run it after `AgentResponse` and before auditor views filesystem (`L718–719`). | 01-phase-index-MASTER.discovery.md:L393 | PHASE 4A — Tier-C agent process isolation (`L688–719`) |
| MASTER-E300 | E/SUMMARY | Architecture: tools execute in orchestrator and agents reach them only through typed IPC requests/results (`L721–722`). | 01-phase-index-MASTER.discovery.md:L397 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-C301 | C/SUMMARY | Recorded status 2026-04-16/commit `55ece36`: 4B.1–4B.4 done—ten tool modules, `IpcToolRegistry`, 15 typed argument structs, 19/19 tests, three blocks fixed (`L723–725`). | 01-phase-index-MASTER.discovery.md:L398 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-D302 | D/SUMMARY | Recorded status: 4B.5 measurement deferred; 4B.6–4B.7 `IntegrationTestRunner` deferred pending 4E `DecompositionTree` types (`L724–725`). | 01-phase-index-MASTER.discovery.md:L399 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E303 | E/SUMMARY | Git tools: native `git2`, deterministic auto-commit message, `plan-{id}` branch, WIP commits, safe undo (`L727`). | 01-phase-index-MASTER.discovery.md:L400 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E304 | E/SUMMARY | Five-tier validation: determinism → quality/AST → unit → integration → stress; auditor-only (`L728`). | 01-phase-index-MASTER.discovery.md:L401 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E305 | E/SUMMARY | DAIRA markers: `[CALL]`, `[RETURN]`, `[MUTATION]`, `[BRANCH]`, `[ERROR]`, `[PANIC]`, `[TIMEOUT]`, `[FINDING]`, `[PATTERN]`, `[ANOMALY]`, `[STAT:name]`, `[DURATION:name]`, `[MEMORY:name]`, `[CHECKPOINT]`, `[STEP:n]` (`L729–730`). | 01-phase-index-MASTER.discovery.md:L402 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E306 | E/SUMMARY | Adaptive tool: `AdaptiveTool<T>` reduces scope based on budget at most three times (`L730–731`). | 01-phase-index-MASTER.discovery.md:L403 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E307 | E/SUMMARY | Edit matching: exact → whitespace-normalized → ellipsis-elision, returning a `SequenceMatcher` similarity hint on failure (`L732–733`). | 01-phase-index-MASTER.discovery.md:L404 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E308 | E/SUMMARY | ShortCoder: AST-equivalent compression with ten tree-sitter rules R1–R10; R4/R6 configurable (`L734`). | 01-phase-index-MASTER.discovery.md:L405 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E309 | E/SUMMARY | Failure taxonomy: deterministic regex categories `SYNTAX_ERROR`, `TYPE_ERROR`, `IMPORT_ERROR`, `ASSERTION_ERROR`, `TIMEOUT`, `EMPTY_OUTPUT`, `RUNTIME_PANIC`, `PERMISSION_DENIED`, `RESOURCE_EXHAUSTED` (`L735–736`). | 01-phase-index-MASTER.discovery.md:L406 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E310 | E/SUMMARY | Large-file read: files >300 lines default to outline; `force_full` is budget guarded (`L737`). | 01-phase-index-MASTER.discovery.md:L407 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E311 | E/SUMMARY | Async incremental compilation: after write run `cargo check`, `ruff`, `tsc`, or `go vet` for the file while executor proceeds; result returns through IPC; three retries then `NEEDS_AUDIT`; skip during TESTING_FIRST; defer errors referencing modules not yet written (`L738–740`). | 01-phase-index-MASTER.discovery.md:L408 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E312 | E/SUMMARY | Step ladder: 15 typed args, verify, ≥15 tests, implement, measure, then integration runner skeleton/implementation with `BABY_STEP<5s`, `STEP<30s`, `PHASE<60s`, `CONCEPT<120s` (`L741–743`). | 01-phase-index-MASTER.discovery.md:L409 | PHASE 4B — orchestrator-owned tools (`L721–743`) |
| MASTER-E313 | E/SUMMARY | Interception model: hooks run in orchestrator around IPC and agents cannot see that hooks exist; T3 is deterministic pattern matching with no LLM; `ruff` post-tool subprocess gets a circuit breaker above 10,000 lines (`L745–748`). | 01-phase-index-MASTER.discovery.md:L413 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E314 | E/SUMMARY | Hook API: `Hook` with `PreToolUse`, `PostToolUse`, `Stop`, `PreCompact`; verdict `Allow\|Block{reason}\|Warn{reason}` (`L749–750`). | 01-phase-index-MASTER.discovery.md:L414 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E315 | E/SUMMARY | Hook timing: pre fires before forwarding tool/bash; post runs ruff + T1/T2 + ShortCoder before result/response; stop requires FSM idle; precompact checkpoints at 85% (`L750–752`). | 01-phase-index-MASTER.discovery.md:L415 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E316 | E/SUMMARY | T1: indexed phase/domain rules DSL, <1 ms; schema predicate escalates provider tier rather than being handled at T1 (`L753–755`). | 01-phase-index-MASTER.discovery.md:L416 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E317 | E/SUMMARY | T2: tree-sitter syntax/dead-code/lazy-code checks, <10 ms under 5000 lines; breaker above 5000 (`L756–757`). | 01-phase-index-MASTER.discovery.md:L417 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E318 | E/SUMMARY | T3: pattern library, runs only when T1+T2 warn without blocking, <5 ms; never runs once already blocked (`L758–759`). | 01-phase-index-MASTER.discovery.md:L418 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E319 | E/SUMMARY | Error sanitization: external block message is generic `"Tool call rejected by policy"`; internal rule ID/pattern/severity removed at enum level, not by string scrubbing (`L760–761`). | 01-phase-index-MASTER.discovery.md:L419 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E320 | E/SUMMARY | Circular trust: orchestrator normalizes and Blake2s-hashes; independent approval for auditor rules (`L762–763`). | 01-phase-index-MASTER.discovery.md:L420 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E321 | E/SUMMARY | 4C tests: 30 contract tests covering hooks, tiers, IPC, trust, session locks, schema escalation, T3 compound behavior, ShortCoder placement; ten adversarial tests (`L764–766`). | 01-phase-index-MASTER.discovery.md:L421 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E322 | E/SUMMARY | 4C SLO: T1<1 ms, T2<10 ms, T3<5 ms, generic hook overhead <2 ms, ruff-inclusive post-hook <500 ms (`L766–767`). | 01-phase-index-MASTER.discovery.md:L422 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-D323 | D/SUMMARY | Skills/MCP: prompt-template skills have no code path here; MCP bridge is a separate 1751-LOC subphase (`L768`). | 01-phase-index-MASTER.discovery.md:L423 | PHASE 4C — invisible hooks and tiered static analysis (`L745–768`) |
| MASTER-E324 | E/SUMMARY | Goal: isolated Research Agent + distinct Designer turn task text into `PipelineSpec` and `QualitySpec` through grounded tree research (`L770–775`). | 01-phase-index-MASTER.discovery.md:L427 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E325 | E/SUMMARY | Research isolation: Tier-C process, web only via IPC proxy, no `.elai_cc/`, no project source, no direct model access (`L772–775`). | 01-phase-index-MASTER.discovery.md:L428 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E326 | E/SUMMARY | Role distinction: Designer and Executor may use same model but differ in role prompt, required steps, and output (`L774–775`). | 01-phase-index-MASTER.discovery.md:L429 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E327 | E/SUMMARY | Research types: `ResearchTree` config (`breadth_factor`, `depth_limit`, `max_iterations`, `phi_min`, `psi_min`, `speculative_depth`, `time_budget`); `ResearchNode` with parent/children/scores/status/info gain/iterations; `ResearchPlan`; `ResearchReport`; snapshot; convergence status (`L776–780`). | 01-phase-index-MASTER.discovery.md:L430 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E328 | E/SUMMARY | Evidence/source types: `EvidenceLevel::{Benchmark(1.0),OfficialDocs(0.8),BlogPost(0.6),Training(0.4)}`; `Alternative`; `Contradiction`; deterministic five-tier `SourceQuality` (`L780–782`). | 01-phase-index-MASTER.discovery.md:L431 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E329 | E/SUMMARY | Product-spec types: `PipelineSpec`, `QualityMetric`, `PerfSpec`, `SourceCitation`; `MediaSpec` output `Web\|Model3D\|Audio\|UiScreenshot\|Animation\|Image\|Custom` (`L782–784`). | 01-phase-index-MASTER.discovery.md:L432 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E330 | E/SUMMARY | Complexity presets: Simple 2×depth3/φ.8/ψ.7; Moderate 3×depth4/.7/.6; Complex 4×depth5/.6/.5 (`L785–788`). | 01-phase-index-MASTER.discovery.md:L433 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E331 | E/SUMMARY | Pruning: info gain under threshold three consecutive iterations marks `Pruned` and frees resources (`L789`). | 01-phase-index-MASTER.discovery.md:L434 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E332 | E/SUMMARY | Base gates: skeleton, compile, ≥14 contract/integration/property tests, adversarial tests, implement, measure (`L790–791`). | 01-phase-index-MASTER.discovery.md:L435 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E333 | E/SUMMARY | 4D.7–4D.10 recursive decomposition: `Idea→Concepts→Phases→Steps→BabySteps`; every level invokes grounded FlashResearch (`L792–794`). | 01-phase-index-MASTER.discovery.md:L436 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E334 | E/SUMMARY | Decomposition types: `ConceptDecomposition`, `ConceptNode`, `PhaseNode`, `StepNode`, `BabyStepSpec` (`L793–795`). | 01-phase-index-MASTER.discovery.md:L437 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E335 | E/SUMMARY | Eight baby-step blocks: `GROUNDING`, `PRE_GATE`, `TASK`, `TDD`, `DETERMINISTIC_CHECK`, `ACCEPTANCE`, `INTEGRATION`, `EXIT_CRITERIA` (`L794–795`). | 01-phase-index-MASTER.discovery.md:L438 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E336 | E/SUMMARY | Technology/package grounding: `TechStackDecision` source is `RESEARCH\|EXISTING_PROJECT\|USER_SPECIFIED\|FALLBACK`; `PackageRef` must pass `API_REGISTRY_CHECK`; `ResearchCache` is LRU (`L796–797`). | 01-phase-index-MASTER.discovery.md:L439 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E337 | E/SUMMARY | Selection UX: interactive exposes alternatives; autonomous selects highest evidence (`L798`). | 01-phase-index-MASTER.discovery.md:L440 | PHASE 4D — Pipeline Factory L1, research + design + recursive decomposition (`L770–798`) |
| MASTER-E338 | E/SUMMARY | Pure builder: no LLM; pure-Rust orchestrator transforms `PipelineSpec` into executable pipeline (`L800–802`). | 01-phase-index-MASTER.discovery.md:L444 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E339 | E/SUMMARY | Five-stage flow: Research → Plan Assembly → Structural Validation → Phase Sizing (stage 3.5) → Feasibility Probe (`L802–803`). | 01-phase-index-MASTER.discovery.md:L445 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E340 | E/SUMMARY | Bounded DAG repair: at most ten rounds with structured delta feedback; deterministic `REORDER` repairs cycles and `SUBSTITUTE` replaces missing tools only when category and I/O match; reject SOCIA LLM repair (`L804–807`). | 01-phase-index-MASTER.discovery.md:L446 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E341 | E/SUMMARY | Types: `ExecutablePipeline` with ID/spec/FSM/dependency graph/novel steps/agent configs/rules/resources/tests/repair log/output; `DependencyGraph`; `DagNode`; compiled `PipelineFsmState`; `RepairAction::{Reorder,Substitute}`; `RepairOutcome::{Repaired,Escalated}`; eight-kind validation error (`L808–811`). | 01-phase-index-MASTER.discovery.md:L447 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E342 | E/SUMMARY | Eight structural checks: registered tool, available model, acyclic graph via Kahn, hardware fit, measurable quality metrics, compiled predicates, novel-step marking for 4F, visual-check adapter mapping (`L812–814`). | 01-phase-index-MASTER.discovery.md:L448 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E343 | E/SUMMARY | Sizing formula: `files_touched·avg_file_tokens + test_count·avg_test_tokens + contract_count·200`; split above 60% of available work budget; depth≤2 (`L815–817`). | 01-phase-index-MASTER.discovery.md:L449 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E344 | E/SUMMARY | 4E.1–4E.6: base builder with at least 30 tests (`L818`). | 01-phase-index-MASTER.discovery.md:L450 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E345 | E/SUMMARY | 4E.7–4E.12 primary execution artifact: typed `DecompositionTree` + `TreeNavigator`, not prompts; depth-first/topological concept→phase→step→baby step (`L819–822`). | 01-phase-index-MASTER.discovery.md:L451 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E346 | E/SUMMARY | Baby-step cycle: context Slot 3 → pre-gate → TESTING_FIRST → EXECUTING → AUDITING → acceptance → integration → exit criteria (`L822–823`). | 01-phase-index-MASTER.discovery.md:L452 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E347 | E/SUMMARY | Bubble-up retries/actions: retry thresholds baby 3, step 2, phase 1, concept 1; actions `ResearchAlternative`, `RedecomposeStep`, `ReplanPhase`, `ReplanConcept`, `Escalate` (`L824–825`). | 01-phase-index-MASTER.discovery.md:L453 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E348 | E/SUMMARY | Integration checkpoints: baby `cargo check` <5 s, step module tests <30 s, phase crate tests <60 s, concept workspace tests <120 s (`L826–827`). | 01-phase-index-MASTER.discovery.md:L454 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E349 | E/SUMMARY | Crash recovery: serialize `TreeNavigatorState` to MessagePack after every baby-step completion (`L828`). | 01-phase-index-MASTER.discovery.md:L455 | PHASE 4E — deterministic Pipeline Factory L2 (`L800–828`) |
| MASTER-E350 | E/SUMMARY | Scope: resource probe plus novel-step stubs in Tier-C sandbox with branch-and-explore (`L830–832`). | 01-phase-index-MASTER.discovery.md:L459 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E351 | E/SUMMARY | Skip rule: skip only if there are zero novel steps and each of VRAM, RAM, disk has ≥20% margin (`L831–832`). | 01-phase-index-MASTER.discovery.md:L460 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E352 | E/SUMMARY | Types: `SystemProbeResult`, stable-NVML-UUID `GpuInfo`, `SystemConstraints`, `StubResult`, `ResourceDelta`, `FeasibilityReport`, `ConstraintResult`, `FeasibilityEscalation`, and decision `Proceed\|ProceedWithConstraints\|Escalate\|Replan` (`L833–835`). | 01-phase-index-MASTER.discovery.md:L461 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E353 | E/SUMMARY | Restricted executor: GRC 3/600, only `stub_write`/`bash`, 25 s graceful and 30 s hard kill, each stub <50 lines, via 4A IPC (`L836–837`). | 01-phase-index-MASTER.discovery.md:L462 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E354 | E/SUMMARY | Tests/workflow: object-safe trait and eight types; compile; ≥20 tests for success/no GPU/low RAM/skip/isolation/dual timeout/max-three alternatives/escalation/oversized report; implement, measure, adversarial (`L838–841`). | 01-phase-index-MASTER.discovery.md:L463 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E355 | E/SUMMARY | 4F.7–4F.8 concept sequencing: baby-step feasibility per concept, sequential because earlier outcomes may change later approach; types `BabyStepFeasibilityReport`, `BabyStepStubResult`, `BabyStepEscalation` (`L842–844`). | 01-phase-index-MASTER.discovery.md:L464 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E356 | E/SUMMARY | Novel baby-step predicates: unverified package, unused framework, or untested capability (`L844–845`). | 01-phase-index-MASTER.discovery.md:L465 | PHASE 4F — feasibility probing (`L830–845`) |
| MASTER-E357 | E/SUMMARY | Mode architecture: autonomy adds no FSM state; `AutonomousGate` wraps an `InteractiveGate` with evaluator/mode/escalation policy/dispatch table (`L847–850`). | 01-phase-index-MASTER.discovery.md:L469 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E358 | E/SUMMARY | Compatibility: interactive delegates exactly to inner gate; autonomous checks deterministic evidence first and calls human only on high risk, missing evidence, or escalation; in-state tool review remains auditor-driven (`L849–852`). | 01-phase-index-MASTER.discovery.md:L470 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E359 | E/SUMMARY | Evidence map: knowledge not unavailable; zero ungrounded; test exits nonzero without Rust compile error; fail→pass; zero audit block; DKG+changelog (`L853–856`). | 01-phase-index-MASTER.discovery.md:L471 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E360 | E/SUMMARY | Escalation defaults: spiral threshold 3, high risk pauses in both modes, major replan always human, at most ten auto advances/phase (`L857–858`). | 01-phase-index-MASTER.discovery.md:L472 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E361 | E/SUMMARY | Triggers: `AntiSpiral`, `HighRisk`, `ReplanMajor`, `CircuitBreaker`, `SubstituteModelFailed`, `SplitTaskFailed` (`L859`). | 01-phase-index-MASTER.discovery.md:L473 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E362 | E/SUMMARY | Split task: at most four children, depth two, sequential merge of all results; parent writes directly to SSB; pick-one branch semantics are explicitly incompatible (`L860–862`). | 01-phase-index-MASTER.discovery.md:L474 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E363 | E/SUMMARY | Model fallback: iterate fallback chain through swap protocol; add `ModelPool.can_fit_llm()`; no fit yields `SubstituteModelFailed` (`L863–864`). | 01-phase-index-MASTER.discovery.md:L475 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E364 | E/SUMMARY | Base gates: skeleton, interactive regression check, contract tests, adversarial, implementation, measurement (`L865–866`). | 01-phase-index-MASTER.discovery.md:L476 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E365 | E/SUMMARY | Tree autonomy: operate at baby-step grain; risk baby/step low, phase medium, concept high; concept bubble-up always human; human check forced after 50 auto baby steps per concept (`L867–870`). | 01-phase-index-MASTER.discovery.md:L477 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E366 | E/SUMMARY | Runtime-selection UX: `elai --local` sidecar, `--hybrid` per-role, default cloud API; `/model` live change; `[agents.{role}.backend] = local\|api\|auto` (`L871–873`). | 01-phase-index-MASTER.discovery.md:L478 | PHASE 4G — autonomous-mode wrapper (`L847–873`) |
| MASTER-E367 | E/SUMMARY | Core separation: learn configs/rules/data from outcomes; modifying agent never evaluates its own change; DGM archive accumulates variants for parent selection rather than direct pairwise comparison (`L875–878`). | 01-phase-index-MASTER.discovery.md:L482 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E368 | E/SUMMARY | Class A: additive data rows for deprecated patterns, failure categories, lazy-code regexes, source tiers; validate FPR across N pipelines; no staged benchmark (`L879–881`). | 01-phase-index-MASTER.discovery.md:L483 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E369 | E/SUMMARY | Class B: tune ETC threshold, baby-step LOC, AB-MCTS temperatures, context weights, freshness half-life, retries, steering coefficients; staged 10/50/200 evaluation (`L882–884`). | 01-phase-index-MASTER.discovery.md:L484 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E370 | E/SUMMARY | Layer 3: FSM states, roles, and GRC protocol are human-only (`L885`). | 01-phase-index-MASTER.discovery.md:L485 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E371 | E/SUMMARY | Outcome telemetry: mode `SURGICAL\|GREENFIELD\|ORACLE`, ETC triggers, stale API catches, context truncations, taxonomy misses (`L886–888`). | 01-phase-index-MASTER.discovery.md:L486 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E372 | E/SUMMARY | Catch taxonomy: `T1_HOOK\|T2_HOOK\|AUDITOR\|ETC`; three unclassified occurrences auto-propose a failure category (`L887–889`). | 01-phase-index-MASTER.discovery.md:L487 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E373 | E/SUMMARY | SWE-bench outcome matrix: `FAIL_TO_PASS`, `PASS_TO_PASS`, `FAIL_TO_FAIL`, `PASS_TO_FAIL`; regression hard-rejects (`L889–891`). | 01-phase-index-MASTER.discovery.md:L488 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E374 | E/SUMMARY | Variant: ID, parent, normalized score [0,1], mode, status `ACTIVE\|ARCHIVED\|REVERTED` (`L891–892`). | 01-phase-index-MASTER.discovery.md:L489 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E375 | E/SUMMARY | Score: normalized weighted components only, never raw count; default weights first-try .3, coverage .2, efficiency .2, intervention .15, findings .15 (`L893–895`). | 01-phase-index-MASTER.discovery.md:L490 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E376 | E/SUMMARY | MSI: λ=.3 exponentially decayed delta score over N=10 per mode; improving >.3, stabilizing −.1.. .3, diverging ≤−.1 pauses+alerts (`L896–898`). | 01-phase-index-MASTER.discovery.md:L491 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E377 | E/SUMMARY | Parent selection: `sigmoid(10·(alpha_i−.5))`, exploration `1/(1+children_count)`, product normalized to probability; surgical parent never feeds greenfield evaluation (`L899–901`). | 01-phase-index-MASTER.discovery.md:L492 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E378 | E/SUMMARY | Closed mutation surface: new allowed keys require Layer-3 approval; parameter keys `EtcThreshold`, `FreshnessHalfLife`, `BabyStepMaxLoc{vram_tier}`, `ContextSlotWeight{slot}`, `RetryBudget`, `AbMctsMaxBranches`, `AbMctsTemperatures`, `MicroReflectionMaxTokens`, `SteeringVectorCoefficient`, `SelfModFrequency`; data keys `DeprecatedPattern`, `FailureTaxonomyCategory`, `LazyCodePattern`, `SourceTierEntry` (`L902–907`). | 01-phase-index-MASTER.discovery.md:L493 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E379 | E/SUMMARY | Four safety layers: dispatch, IPC, filesystem, git; modifying agent cannot read lock list; auto-revert after >10% degradation across three pipelines; trigger task detects objective hacking (`L908–910`). | 01-phase-index-MASTER.discovery.md:L494 | PHASE 4H — measured, bounded self-modification (`L875–910`) |
| MASTER-E380 | E/SUMMARY | Condition/scope: instantiate only for media pipelines; validate image, 3D, audio, UI screenshot (`L912–914, L923`). | 01-phase-index-MASTER.discovery.md:L498 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E381 | E/SUMMARY | Tier 1: deterministic ImageMagick/FFmpeg/Tesseract/Blender CLI, target <100 ms, veto authority (`L914–915`). | 01-phase-index-MASTER.discovery.md:L499 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E382 | E/SUMMARY | Tier 2: VLM constrained JSON, advisory only (`L915`). | 01-phase-index-MASTER.discovery.md:L500 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E383 | E/SUMMARY | Mode A: Qwen3.5-VL-30B-A3B via Ollama Q4_K_M ~17 GB, all-30B storage despite ~3B active, Ollama ≥0.12.7 (`L916–917`). | 01-phase-index-MASTER.discovery.md:L501 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E384 | E/SUMMARY | Mode B: InternVL3.5-38B via SGLang only with RadixAttention; ~38 GB 8-bit needs dual 3090 tensor parallel, or ~21 GB Q4 (`L918–919`). | 01-phase-index-MASTER.discovery.md:L502 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E385 | E/SUMMARY | Audio: CLAP audio-text similarity, ~200M/~0.4 GB FP16, Xenova export (`L920`). | 01-phase-index-MASTER.discovery.md:L503 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E386 | E/SUMMARY | Image metrics determinism: SSIM/LPIPS CPU execution only; epsilon 1e−6 rather than bit-exact (`L921–922`). | 01-phase-index-MASTER.discovery.md:L504 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E387 | E/SUMMARY | Render/capture/evaluate: adapter trait for Playwright web, Blender 3D, FFmpeg animation; produce→render→VLM→structured feedback→iterate, bounded N (`L924–926`). | 01-phase-index-MASTER.discovery.md:L505 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E388 | E/SUMMARY | Concurrency/verdicts: Tier1 max4, Tier2 max1, queue64; T1 fail→Fail; T1 pass+T2 disagree→HumanReview; no VLM→Degraded (`L927–928`). | 01-phase-index-MASTER.discovery.md:L506 | PHASE 4I — conditional media validation (`L912–928`) |
| MASTER-E389 | E/SUMMARY | Condition/value exchange: only when Empire source available; ELAI becomes a brain and supplies self-improvement/reasoning/pipelines, receiving a live deployment environment (`L930–933`). | 01-phase-index-MASTER.discovery.md:L510 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E390 | E/SUMMARY | Verified topology: Engine MKI→MKII→MKIII; engine IPC at `/tmp/brain-engine.sock` and factory line `/tmp/brain-factory.sock`, newline JSON/Unix stream; max 65,536-byte engine messages and 256 KiB FSB frames; hook route/event, fnmatch bus, multi-brain isolation (`L934–937`). | 01-phase-index-MASTER.discovery.md:L511 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E391 | E/SUMMARY | Repair targets: `brainctl↔brain_lifecycle` circular dependency, EngineMKII hard import of `slc`, and `unique-factory/types` standard-library shadow (`L938–939`). | 01-phase-index-MASTER.discovery.md:L512 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E392 | E/SUMMARY | Supersession: replace seven-line lambda action stubs and unvalidated `brain.json` with PipelineSpec's six enforcement layers (`L940–941`). | 01-phase-index-MASTER.discovery.md:L513 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E393 | E/SUMMARY | 4J.1 abstractions: cross-platform `FactoryLineTransport` with Unix/TCP; generic client; schema-validated `BrainManifest`; rate-limited/filtered event bridge; `ElaiNeuron`; repair target with `CircularDep\|HardImport\|StdlibShadow` (`L942–947`). | 01-phase-index-MASTER.discovery.md:L514 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E394 | E/SUMMARY | Exposed capabilities: only `task_submit`, `task_status`, `pipeline_result`, `health_check`; tool filter; leak zero internal state (`L944–946`). | 01-phase-index-MASTER.discovery.md:L515 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E395 | E/SUMMARY | Wire types: `FactoryLineCommand` (`Register\|Heartbeat\|List\|RequestUpdate`) with protocol `v:u8`; nine-kind `IpcMessage`; shared Event/Command/Response (`L948–949`). | 01-phase-index-MASTER.discovery.md:L516 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E396 | E/SUMMARY | Degradation/build: generic transport, graceful operation without Empire, feature gate `empire` (`L950`). | 01-phase-index-MASTER.discovery.md:L517 | PHASE 4J — conditional Empire integration (`L930–950`) |
| MASTER-E397 | E/SUMMARY | UX: native structured control surface, not a chatbot; every input undergoes FSM classification; interactive user drives and approves while ELAI executes (`L952–955`). | 01-phase-index-MASTER.discovery.md:L521 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E398 | E/SUMMARY | Stack: Tauri 2.10/build 2.5/CLI 2.10, Svelte/Vite webview, Rust commands; no HTTP, REST, WebSocket, or CORS; invoke-poll read model every 500 ms (`L956–958`). | 01-phase-index-MASTER.discovery.md:L522 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E399 | E/SUMMARY | Visual identity: teal `#1B7A6E`, charcoal `#0f1318`, JetBrains Mono, information density over whitespace (`L959–960`). | 01-phase-index-MASTER.discovery.md:L523 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E400 | E/SUMMARY | 4K.1 read model: read-only WAL `SsbViewer`; typed queries `latest_fsm_state`, `active_session`, `phase_progress`, `recent_actions`, `agent_actions`, `pending_interventions`, `file_changes`, `reasoning_chain` (`L961–964`). | 01-phase-index-MASTER.discovery.md:L524 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E401 | E/SUMMARY | Intervention events: add `INTERVENTION_REQUEST` with type `ApproveChange\|ReplanApproval\|ErrorRecovery` and options `Accept\|Decline\|Retry(String)\|Skip`; add `INTERVENTION_RESPONSE` (`L964–966`). | 01-phase-index-MASTER.discovery.md:L525 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E402 | E/SUMMARY | Viewer performance: lightweight projection, pagination, <10 ms on 100K rows (`L967`). | 01-phase-index-MASTER.discovery.md:L526 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E403 | E/SUMMARY | 4K.2 scaffold: dashboard crate, six Tauri commands, Svelte/Vite/CSS, `elai --dashboard` (`L968–969`). | 01-phase-index-MASTER.discovery.md:L527 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E404 | E/SUMMARY | UI surfaces: project selector, task input, phase ribbon, intervention panel, activity feed, reasoning chain, commit history (`L970–971`). | 01-phase-index-MASTER.discovery.md:L528 | PHASE 4K — native Tauri dashboard MVP (`L952–974`) |
| MASTER-E405 | E/SUMMARY | Goal: replace bespoke local-sidecar/cloud dispatch with one workflow spec and interchangeable runtime adapters; port the Ouroboros adapter/factory/name-resolution pattern (`L976–981`). | 01-phase-index-MASTER.discovery.md:L533 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E406 | E/SUMMARY | Non-goals: do not build runtimes, use magic-keyword routing, or add LiteLLM; adapt existing CLIs/APIs, retain FSM+Designer routing, extend `LlmProvider` (`L982–983`). | 01-phase-index-MASTER.discovery.md:L534 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E407 | E/SUMMARY | Backends: `Sidecar`, `AnthropicApi`, `ClaudeCode`, `CodexCli`, `OpenCode`, `Hermes`, `GeminiCli`, `KiroCli`, `CopilotCli`, `Ollama`, `Vllm` (`L984–985`). | 01-phase-index-MASTER.discovery.md:L535 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E408 | E/SUMMARY | Adapter: `complete`, `stream`, `capabilities`, `health_check`; verify vtable compatibility and alias to `LlmProvider` if compatible (`L986–987`). | 01-phase-index-MASTER.discovery.md:L536 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E409 | E/SUMMARY | Factory config: CLI path, base URL, secret API key, model, permission mode (`AcceptEdits\|Plan\|DangerouslySkipPermissions\|Default`), profile, CWD, timeouts (`L987–989`). | 01-phase-index-MASTER.discovery.md:L537 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E410 | E/SUMMARY | Per-role hybrid: `PerRoleRuntime {role,backend,fallback_chain}` assigns arbitrary runtime per role (`L990–991`). | 01-phase-index-MASTER.discovery.md:L538 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E411 | E/SUMMARY | Resolution error: unknown candidate returns candidate plus full supported-name list (`L992–993`). | 01-phase-index-MASTER.discovery.md:L539 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E412 | E/SUMMARY | 4M.0 provenance: ingest Ouroboros source and write `workflow/papers/ouroboros-runtime-interface.md` (`L994–996`). | 01-phase-index-MASTER.discovery.md:L540 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E413 | E/SUMMARY | 4M.1 skeleton/config: backend enum, adapter or alias, factory config, permission/error/role/capability/health types, stub resolve/create, `[runtime] backend` and `[agents.<role>.runtime]` schema (`L997–1000`). | 01-phase-index-MASTER.discovery.md:L541 | PHASE 4M — multi-runtime backend abstraction (`L976–1000`) |
| MASTER-E414 | E/SUMMARY | Phase-0 substrate: the nine-crate split creates the dependency boundaries used by every later phase; `ClockProvider`/`EntropySource`, `RuntimeContext`, the provider trait, minimal FSM, visibility filter, session format, and contract crate are foundational (`L24–51`). | 01-phase-index-MASTER.discovery.md:L545 | Interactions and dependencies |
| MASTER-E415 | E/SUMMARY | Retrieval ladder: 1A DKG supplies symbols/edges to 1B chunking fallback alignment, 1D snapshots/PageRank, 1E graph anchoring, 3F AST evaluation, 3G contract context, and 3K AST chunking (`L57–76, L89–90, L122–165, L524–534, L555–585, L250–271`). | 01-phase-index-MASTER.discovery.md:L546 | Interactions and dependencies |
| MASTER-E416 | E/SUMMARY | Retrieval assembly: 1E composes 1B BM25 + 1C vectors + 1D PageRank, leaves explicit 1G rerank and 3E token-fit stubs, and 3K later hardens the assembled pipeline (`L143–166, L250–271`). | 01-phase-index-MASTER.discovery.md:L547 | Interactions and dependencies |
| MASTER-E417 | E/SUMMARY | Resilience dependency fix: 1I cannot start until 1E and 1G; the missing graph edge `1G→1I` must be added first (`L227–228`). | 01-phase-index-MASTER.discovery.md:L548 | Interactions and dependencies |
| MASTER-E418 | E/SUMMARY | RAG hardening prerequisites: 3K depends on 1E+1G+3J because it uses the assembled retrieval/reranking path and sidecar generation for HyDE (`L250–260`). | 01-phase-index-MASTER.discovery.md:L549 | Interactions and dependencies |
| MASTER-E419 | E/SUMMARY | Speculative decoding revisit: 1F is dropped on Ollama but explicitly reopened when the 3J vLLM backend makes layer-level control possible (`L168–181, L646–664`). | 01-phase-index-MASTER.discovery.md:L550 | Interactions and dependencies |
| MASTER-E420 | E/SUMMARY | Shared model-runtime degradation: 1C/1G/1I circuit breakers emit the same SSB `knowledge_degradation` event; readiness gates the full FSM (`L114–117, L199–202, L230–246, L404–405`). | 01-phase-index-MASTER.discovery.md:L551 | Interactions and dependencies |
| MASTER-E421 | E/SUMMARY | Reasoning evolution: 2A freezes trait methods for 2B; 2B breaks/implements their signatures; 2C layers role config; 2D adds safety/feasibility methods (`L277–385`). | 01-phase-index-MASTER.discovery.md:L552 | Interactions and dependencies |
| MASTER-E422 | E/SUMMARY | Reasoning dependency boundary: 2A is core-only; 2B deliberately adds tools without allowing a tools→reasoning cycle or knowledge transitively; 4A later replaces direct network verification with IPC (`L288–290, L305–330, L700–702`). | 01-phase-index-MASTER.discovery.md:L553 | Interactions and dependencies |
| MASTER-E423 | E/SUMMARY | Budget authority handoff: 2C owns in-process budgets through Phase 3; 4A demotes it to advisory and makes the external orchestrator authoritative (`L360–361, L700–702`). | 01-phase-index-MASTER.discovery.md:L554 | Interactions and dependencies |
| MASTER-E424 | E/SUMMARY | Feasibility escalation: 2D's blocked inline check signals a major replan whose actual SSB/FSM emission arrives in 3A+ (`L381–384, L397–408`). | 01-phase-index-MASTER.discovery.md:L555 | Interactions and dependencies |
| MASTER-E425 | E/SUMMARY | FSM evidence composition: transitions consume knowledge readiness (1I), plan grounding (2B), expected failing tests and fail→pass results, auditor blocks, and DKG/changelog completion (`L404–409`). | 01-phase-index-MASTER.discovery.md:L556 | Interactions and dependencies |
| MASTER-E426 | E/SUMMARY | SSB backbone: retrieval mode changes, GRC traces, phase transitions, plans, conflicts, checkpoints, channel clears, context reductions, findings, model swaps, phase completion, interventions, and metrics all converge on 3B's ordered journal (`L114–117, L199–200, L235–246, L416–439 and downstream references`). | 01-phase-index-MASTER.discovery.md:L557 | Interactions and dependencies |
| MASTER-E427 | E/SUMMARY | Deterministic parallelism: 3D schedules concurrently but applies writes in SSB sequence order; 3C spawn-parallel is a sequential stub until 3D upgrades it (`L447–449, L474–501`). | 01-phase-index-MASTER.discovery.md:L558 | Interactions and dependencies |
| MASTER-E428 | E/SUMMARY | Checkpoint composition: 3C per-agent checkpoints + 3D channel checkpoint/replay + 3H session snapshots/handoffs + 4E navigator state form layered recovery; all avoid nonserializable untracked state (`L457–464, L495–498, L587–614, L828`). | 01-phase-index-MASTER.discovery.md:L559 | Interactions and dependencies |
| MASTER-E429 | E/SUMMARY | Context composition: 3E's eight slots consume PageRank/retrieval output, SSB entries, GRC history, and 3G skeleton/detail; 3H preserves cache-prefix slots 1–3 and trims tool results only (`L503–520, L555–574, L602–606`). | 01-phase-index-MASTER.discovery.md:L560 | Interactions and dependencies |
| MASTER-E430 | E/SUMMARY | Rules dependency direction: rules consumes a knowledge-implemented `AstEvaluator`; orchestrator owns discovery/normalization and SSB writes; the originating auditor cannot approve its own rule (`L523–552`). | 01-phase-index-MASTER.discovery.md:L561 | Interactions and dependencies |
| MASTER-E431 | E/SUMMARY | Plan splitting contract: 3G and 4E both cap split depth at two to match 4G autonomous splitting; 4G needs all child results and therefore uses sequential merge rather than 3H's pick-one winner branches (`L580–583, L815–817, L860–862`). | 01-phase-index-MASTER.discovery.md:L562 | Interactions and dependencies |
| MASTER-E432 | E/SUMMARY | Plan-to-execution chain: 4D Research+Designer creates grounded `PipelineSpec` and recursive decomposition; 4E deterministically builds/repairs/sizes it; 4F probes novel work/resources; 4G advances the resulting validated tree (`L770–870`). | 01-phase-index-MASTER.discovery.md:L563 | Interactions and dependencies |
| MASTER-E433 | E/SUMMARY | Feasibility executor: 4F reuses 4A `AgentDispatchConfig` and IPC restrictions rather than trusting the generic `AgentRole` (`L711–714, L830–841`). | 01-phase-index-MASTER.discovery.md:L564 | Interactions and dependencies |
| MASTER-E434 | E/SUMMARY | Integration-test dependency: 4B's tree-grained runner waits for 4E `DecompositionTree` types; its time limits match 4E's four integration levels (`L723–725, L741–743, L819–828`). | 01-phase-index-MASTER.discovery.md:L565 | Interactions and dependencies |
| MASTER-E435 | E/SUMMARY | ShortCoder placement: tool compression is defined in 4B and invoked from 4C post-hooks in orchestrator after an agent response, before auditor visibility (`L718–719, L734, L750–752`). | 01-phase-index-MASTER.discovery.md:L566 | Interactions and dependencies |
| MASTER-E436 | E/SUMMARY | Isolation/control: 4A makes inference, tools, grounding, budgets, and whitelists orchestrator-owned; 4B and 4C execute tools/hooks on the trusted side of that boundary (`L688–768`). | 01-phase-index-MASTER.discovery.md:L567 | Interactions and dependencies |
| MASTER-E437 | E/SUMMARY | Autonomous compatibility: 4G wraps 3A's gate rather than altering its FSM; interactive behavior is invariant; high-risk and major-replan human control remains (`L847–870, L1032–1034`). | 01-phase-index-MASTER.discovery.md:L568 | Interactions and dependencies |
| MASTER-E438 | E/SUMMARY | Model substitution: 3C declares but disables `SubstituteModel`; 3J builds providers/pool; 4G supplies fit checking and fallback iteration; 4M generalizes the backend interface and per-role chains (`L465–468, L646–681, L863–873, L976–1000`). | 01-phase-index-MASTER.discovery.md:L569 | Interactions and dependencies |
| MASTER-E439 | E/SUMMARY | Self-mod inputs and protection: 4H consumes normalized `PipelineOutcome`/test/security/retrieval telemetry but is fenced by 3F immutable rules and 4A's dispatch+IPC+filesystem+git protections (`L536–547, L696–710, L875–910`). | 01-phase-index-MASTER.discovery.md:L570 | Interactions and dependencies |
| MASTER-E440 | E/SUMMARY | Conditional media path: 4D `MediaSpec` and 4E visual-adapter mapping decide whether 4I is lazily instantiated; deterministic Tier 1 veto remains authoritative over VLM advice (`L782–784, L812–814, L912–928`). | 01-phase-index-MASTER.discovery.md:L571 | Interactions and dependencies |
| MASTER-E441 | E/SUMMARY | Conditional Empire path: 4J is behind a feature gate and degrades gracefully; it uses 4D/4E PipelineSpec enforcement to repair verified Empire issues (`L930–950`). | 01-phase-index-MASTER.discovery.md:L572 | Interactions and dependencies |
| MASTER-E442 | E/SUMMARY | Dashboard read/write split: 4K queries SSB through read-only WAL connections and writes user decisions as typed intervention events; no alternate HTTP/control plane bypasses FSM classification (`L952–974`). | 01-phase-index-MASTER.discovery.md:L573 | Interactions and dependencies |
| MASTER-E443 | E/SUMMARY | 3J→4M refactor: 3J's bespoke `InferenceDispatcher` and backend mode are the legacy surface that 4M converts into `RuntimeAdapter`/factory/per-role runtime configuration (`L675–681, L976–1000`). | 01-phase-index-MASTER.discovery.md:L574 | Interactions and dependencies |
| MASTER-I444 | I/SUMMARY | Full completion path: a task is classified by FSM, grounded/researched, converted to a typed/sized/probed tree, executed by isolated role-configured agents through orchestrator-owned tools, journaled in SSB, verified at baby/step/phase/concept and FSM gates, optionally media-validated, memorized/handoff-checkpointed, and only then complete. This is the composition implied by the explicit phase interfaces; no single MASTER passage states the whole workflow. | 01-phase-index-MASTER.discovery.md:L575 | Interactions and dependencies |
| MASTER-E445 | E/SUMMARY | Determinism/entropy quarantine: all phases must use injected time/randomness; CI grep, DST reference model, crash replay, and 100-run identity checks enforce it (`L1006–1008`). | 01-phase-index-MASTER.discovery.md:L581 | System-wide (`L1002–1034`) |
| MASTER-E446 | E/SUMMARY | TDD ordering: every implementation phase creates failing tests before implementation and confirms the failures are correct (`L1010–1011`). | 01-phase-index-MASTER.discovery.md:L582 | System-wide (`L1002–1034`) |
| MASTER-E447 | E/SUMMARY | Anti-stale choices: prefer Blake3 over Blake2s where the anti-stale rule applies, `tracing` over `log`, `thiserror`/`anyhow` over boxed dynamic errors, `deadpool` over r2d2 for new code, and `globset` over glob; deeper policy is in `CLAUDE.md` (`L1013–1014`). | 01-phase-index-MASTER.discovery.md:L583 | System-wide (`L1002–1034`) |
| MASTER-Q448 | Q/SUMMARY | Hash coexistence: the MASTER still explicitly requires Blake2s in 1A/1C/3F/4C while its cross-cutting anti-stale line says Blake3 over Blake2s. Treat phase-local named hashes as explicit exceptions until original plans clarify (`L74, L96–103, L544, L762, L1013–1014`). | 01-phase-index-MASTER.discovery.md:L584 | System-wide (`L1002–1034`) |
| MASTER-E449 | E/SUMMARY | Grounding hygiene: every phase classifies claims as `GROUNDED\|VERIFIED\|UNGROUNDED` with cited source; fixtures/bench corpora are committed for reproducibility (`L1016–1018`). | 01-phase-index-MASTER.discovery.md:L585 | System-wide (`L1002–1034`) |
| MASTER-E450 | E/SUMMARY | Tool/rule filtering: enforce at `ToolRegistry::execute`, not in prompts; hidden and missing paths are observationally identical (`L1020–1022`). | 01-phase-index-MASTER.discovery.md:L586 | System-wide (`L1002–1034`) |
| MASTER-E451 | E/SUMMARY | Isolation authority: after 4A agents have no network, model client, or tools; orchestrator owns GRC budget, inference, tools, and grounding over capped IPC (`L1024–1026`). | 01-phase-index-MASTER.discovery.md:L587 | System-wide (`L1002–1034`) |
| MASTER-E452 | E/SUMMARY | Self-mod immutability: security/finance `mutable=0` rules cannot be automatically modified; auditor proposals need independent approval; 4H may change only closed enum keys, not source code (`L1028–1030`). | 01-phase-index-MASTER.discovery.md:L588 | System-wide (`L1002–1034`) |
| MASTER-E453 | E/SUMMARY | Graduation: interactive first, autonomous opt-in; no added FSM state; high risk and major replan pause in both; within-execution tool review belongs to auditor (`L1032–1034`). | 01-phase-index-MASTER.discovery.md:L589 | System-wide (`L1002–1034`) |
| MASTER-E454 | E/SUMMARY | Build baseline/exit: Windows 684-test baseline; zero compile warnings/errors and zero failures before Phase 0 (`L10–22`). | 01-phase-index-MASTER.discovery.md:L593 | Numeric and structural invariants |
| MASTER-E455 | E/SUMMARY | Thin entry point/crate map: nine named crates and `main.rs<500` (`L24–51`). | 01-phase-index-MASTER.discovery.md:L594 | Numeric and structural invariants |
| MASTER-E456 | E/SUMMARY | Repro run: seed 42, single test thread, diff zero (`L46–49`). | 01-phase-index-MASTER.discovery.md:L595 | Numeric and structural invariants |
| MASTER-E457 | E/SUMMARY | DKG: WAL, cascading typed edges, logged parse skip, indexed name/file/kind (`L62–76`). | 01-phase-index-MASTER.discovery.md:L596 | Numeric and structural invariants |
| MASTER-E458 | E/SUMMARY | BM25: untokenized path, atomic delete+insert, no binary ingestion, no orphans (`L78–93`). | 01-phase-index-MASTER.discovery.md:L597 | Numeric and structural invariants |
| MASTER-E459 | E/SUMMARY | Vector: max sequence 512, cache hit avoids embed, missing model never crashes, cache hit target ≥90% (`L95–120`). | 01-phase-index-MASTER.discovery.md:L598 | Numeric and structural invariants |
| MASTER-E460 | E/SUMMARY | PageRank: density >.3 forces insertion fallback; personalization sums to one; clamp nonfinite values; ≤10 fit iterations (`L122–141`). | 01-phase-index-MASTER.discovery.md:L599 | Numeric and structural invariants |
| MASTER-E461 | E/SUMMARY | Retrieval: no hot-path LLM for CatRAG; A-RAG loops≤15/tokens≤128K; target savings/latency gates (`L143–166`). | 01-phase-index-MASTER.discovery.md:L600 | Numeric and structural invariants |
| MASTER-E462 | E/SUMMARY | Statistical feature gates: SPLADE requires ≥5% mean recall improvement and `p<.05`; speculative decoding originally required >1.3× and <3 GB (`L168–179, L208–225`). | 01-phase-index-MASTER.discovery.md:L601 | Numeric and structural invariants |
| MASTER-E463 | E/SUMMARY | Circuit behavior: clocked breakers and deterministic jitter; fresh session on probe; half-open recovery is evidence-driven; degraded readiness means BM25-only (`L199–202, L227–248`). | 01-phase-index-MASTER.discovery.md:L602 | Numeric and structural invariants |
| MASTER-E464 | E/SUMMARY | GRC: first step Observe; default chain; Status/Converge always available; budget checked before saturating increment; enum order carries meaning (`L277–299`). | 01-phase-index-MASTER.discovery.md:L603 | Numeric and structural invariants |
| MASTER-E465 | E/SUMMARY | Verification: six binary methods, empty plan passes, any ungrounded plan blocks, local test timeout 30 s (`L301–330`). | 01-phase-index-MASTER.discovery.md:L604 | Numeric and structural invariants |
| MASTER-E466 | E/SUMMARY | Role config: overrides only add transitions; auditor cannot converge finding-free; assigning config resets topic (`L332–362`). | 01-phase-index-MASTER.discovery.md:L605 | Numeric and structural invariants |
| MASTER-E467 | E/SUMMARY | CSQ: all critical assumptions require guards; exactly two inline checks; false positive target zero (`L364–385`). | 01-phase-index-MASTER.discovery.md:L606 | Numeric and structural invariants |
| MASTER-E468 | E/SUMMARY | FSM: tool permissions depend on state; redirect target must be valid; reject leaves state unchanged; interactive has no timeout (`L391–414`). | 01-phase-index-MASTER.discovery.md:L607 | Numeric and structural invariants |
| MASTER-E469 | E/SUMMARY | SSB: one writer/binding; append-only typed rows; commit before broadcast and result dispatch; drain on shutdown; deterministic dump (`L416–439`). | 01-phase-index-MASTER.discovery.md:L608 | Numeric and structural invariants |
| MASTER-E470 | E/SUMMARY | Agent: UUID from entropy provider; hidden paths indistinguishable; same UUID on recovery; corrupt checkpoint is recoverable; budget death does not count as restart; spiral recovery uses pre-spiral checkpoint (`L441–472`). | 01-phase-index-MASTER.discovery.md:L609 | Numeric and structural invariants |
| MASTER-E471 | E/SUMMARY | BSP: no concurrent `BinaryOp`; sequence orders writes; untracked never serializes; checkpoint only after gate; deferred nodes cannot cascade (`L474–501`). | 01-phase-index-MASTER.discovery.md:L610 | Numeric and structural invariants |
| MASTER-E472 | E/SUMMARY | Context: sum never exceeds model window; SSB/GRC/output floors; deterministic prefix hashes; terminal overflow escalates/splits (`L503–520`). | 01-phase-index-MASTER.discovery.md:L611 | Numeric and structural invariants |
| MASTER-E473 | E/SUMMARY | Rules: predicates compile at admission; composites depth≤8; no LLM in deterministic tiers; immutable rules are protected; self-approval impossible (`L523–553`). | 01-phase-index-MASTER.discovery.md:L612 | Numeric and structural invariants |
| MASTER-E474 | E/SUMMARY | Plans: skeleton ≤4096 exact tokens; structured—not prose—coherence checks; contract/version changes propagate; cycles forbidden; split depth≤2 (`L555–585`). | 01-phase-index-MASTER.discovery.md:L613 | Numeric and structural invariants |
| MASTER-E475 | E/SUMMARY | Handoff: session DAG immutable; no physical branch deletion; user/assistant/cache-prefix never stubbed; stub must resolve+hash or become explicit error (`L587–614`). | 01-phase-index-MASTER.discovery.md:L614 | Numeric and structural invariants |
| MASTER-E476 | E/SUMMARY | DST: oracle <30% SUT LOC; transport-level replay; CI state-machine bounds; deterministic rate 100% (`L616–644`). | 01-phase-index-MASTER.discovery.md:L615 | Numeric and structural invariants |
| MASTER-E477 | E/SUMMARY | Provider/GPU: 1 GB multiplexing margin; model-specific guards; provider failure warns/falls back (`L646–681`). | 01-phase-index-MASTER.discovery.md:L616 | Numeric and structural invariants |
| MASTER-E478 | E/SUMMARY | IPC: 10 MB frame hard cap plus lower per-type caps; 0700 socket dir and peer credentials; no agent provider dependency; phase marker fsync before unload (`L688–717`). | 01-phase-index-MASTER.discovery.md:L617 | Numeric and structural invariants |
| MASTER-E479 | E/SUMMARY | Tools/hooks: auditor-only validation; bounded reduction/retries; ShortCoder AST-equivalent; generic sanitized block errors; T3 never runs after a block (`L721–767`). | 01-phase-index-MASTER.discovery.md:L618 | Numeric and structural invariants |
| MASTER-E480 | E/SUMMARY | Research: deterministic evidence weights and five-tier source quality; prune only after three low-gain iterations; package registry verification (`L770–798`). | 01-phase-index-MASTER.discovery.md:L619 | Numeric and structural invariants |
| MASTER-E481 | E/SUMMARY | Builder: pure deterministic assembly; repair≤10; substitute only compatible I/O/category; eight checks; phase work≤60%; depth≤2 (`L800–828`). | 01-phase-index-MASTER.discovery.md:L620 | Numeric and structural invariants |
| MASTER-E482 | E/SUMMARY | Probe: skip only on conjunctive zero-novel + all-resource margin; stubs <50 LOC; max three alternatives; strict dual timeout (`L830–845`). | 01-phase-index-MASTER.discovery.md:L621 | Numeric and structural invariants |
| MASTER-E483 | E/SUMMARY | Autonomy: evidence precedes advance; high risk/major replan stays human; ten advances/phase; four subtasks/depth two; 50 baby steps/concept then human (`L847–873`). | 01-phase-index-MASTER.discovery.md:L622 | Numeric and structural invariants |
| MASTER-E484 | E/SUMMARY | Self-mod: generator never evaluator; scores normalized; mode-separated parent selection; hard reject regression; layer-3 human; auto-revert threshold; lock list hidden (`L875–910`). | 01-phase-index-MASTER.discovery.md:L623 | Numeric and structural invariants |
| MASTER-E485 | E/SUMMARY | Media: deterministic tier vetoes advisory model; CPU metrics with epsilon; bounded concurrency; unavailable VLM degrades rather than fabricates verdict (`L912–928`). | 01-phase-index-MASTER.discovery.md:L624 | Numeric and structural invariants |
| MASTER-E486 | E/SUMMARY | Empire: explicit size/versioned newline-JSON transport and narrow four-capability surface, no internal-state leakage (`L930–950`). | 01-phase-index-MASTER.discovery.md:L625 | Numeric and structural invariants |
| MASTER-E487 | E/SUMMARY | Dashboard: SSB is read-only from viewer; no network server; every input classified by FSM; typed/paginated projections (`L952–974`). | 01-phase-index-MASTER.discovery.md:L626 | Numeric and structural invariants |
| MASTER-E488 | E/SUMMARY | Runtime adapters: known enum rather than freeform magic routing; API keys typed secret; resolver returns supported set; role-specific fallbacks (`L976–1000`). | 01-phase-index-MASTER.discovery.md:L627 | Numeric and structural invariants |
| MASTER-D489 | D/SUMMARY | Master granularity: individual phase files still own the exhaustive pre-gates/tasks/checks/adversarial/acceptance details (`L1–4`). | 01-phase-index-MASTER.discovery.md:L631 | Open and deferred items |
| MASTER-D490 | D/SUMMARY | 1A language expansion: Python/TypeScript/Go are later than initial Rust (`L57–60`). | 01-phase-index-MASTER.discovery.md:L632 | Open and deferred items |
| MASTER-D491 | D/SUMMARY | 1C TurboQuant: raw f32 now; compression later (`L119–120`). | 01-phase-index-MASTER.discovery.md:L633 | Open and deferred items |
| MASTER-D492 | D/SUMMARY | 1H: entirely conditional; early drop is valid completion and stays disabled by default (`L208–225`). | 01-phase-index-MASTER.discovery.md:L635 | Open and deferred items |
| MASTER-D493 | D/SUMMARY | 3K: LLM edge scoring rejected/deferred on cost (`L268–270`). | 01-phase-index-MASTER.discovery.md:L636 | Open and deferred items |
| MASTER-D494 | D/SUMMARY | 2A: `verify_claim`/`ground_plan` initially unimplemented pending 2B (`L285–286`). | 01-phase-index-MASTER.discovery.md:L637 | Open and deferred items |
| MASTER-D495 | D/SUMMARY | 2B: WEB/API/API_REGISTRY remain stubbed until 4A IPC proxy; phase ends at 3/6 methods (`L301–330`). | 01-phase-index-MASTER.discovery.md:L638 | Open and deferred items |
| MASTER-D496 | D/SUMMARY | 2D: web/grep/registry feasibility methods wait for 4F; orchestrator replan emission waits for 3A+ (`L368–384`). | 01-phase-index-MASTER.discovery.md:L639 | Open and deferred items |
| MASTER-D497 | D/SUMMARY | 3A: feasibility-state evidence gates wait for 4F (`L404–409`). | 01-phase-index-MASTER.discovery.md:L640 | Open and deferred items |
| MASTER-D498 | D/SUMMARY | 3C: parallel spawn/direct communication are stubs until 3D; model substitution waits for 4G; Phase 3 remains Tier A (`L441–468`). | 01-phase-index-MASTER.discovery.md:L641 | Open and deferred items |
| MASTER-Q499 | Q/SUMMARY | 3E: numbered substeps are omitted from MASTER (`L521`). | 01-phase-index-MASTER.discovery.md:L642 | Open and deferred items |
| MASTER-Q500 | Q/SUMMARY | 3H amendments: branch event name, row granularity, and logical cleanup need reconciliation in original plan (`L590–592`). | 01-phase-index-MASTER.discovery.md:L643 | Open and deferred items |
| MASTER-D501 | D/SUMMARY | 3J: TurboQuant is FEASIBLE/DEFER; MILP is ADOPT/DROP and has Windows build risk; CPU offload is not fit for EAGLE-3 training (`L650–669`). | 01-phase-index-MASTER.discovery.md:L645 | Open and deferred items |
| MASTER-D502 | D/SUMMARY | 4A: Windows fallback is explicitly weaker; ShortCoder moves to 4B (`L706–719`). | 01-phase-index-MASTER.discovery.md:L646 | Open and deferred items |
| MASTER-D503 | D/SUMMARY | 4C: skills code path and MCP bridge are outside the phase (`L768`). | 01-phase-index-MASTER.discovery.md:L648 | Open and deferred items |
| MASTER-D504 | D/SUMMARY | 4H: structural FSM/role/GRC mutation remains human-only (`L885, L902`). | 01-phase-index-MASTER.discovery.md:L649 | Open and deferred items |
| MASTER-D505 | D/SUMMARY | 4I: exists only when pipeline emits media; VLM absence yields degraded state (`L912–928`). | 01-phase-index-MASTER.discovery.md:L650 | Open and deferred items |
| MASTER-D506 | D/SUMMARY | 4J: only with Empire source and `empire` feature; otherwise graceful degradation (`L930–950`). | 01-phase-index-MASTER.discovery.md:L651 | Open and deferred items |
| MASTER-D507 | D/SUMMARY | 4K: 500 ms polling is MVP; true push is future (`L972–974`). | 01-phase-index-MASTER.discovery.md:L652 | Open and deferred items |
| MASTER-Q508 | Q/SUMMARY | 4M completeness: MASTER lists only 4M.0–4M.1 even though the phase goal implies adapter implementations, resolution, health, streaming, fallbacks, migrations, and tests; original `phase-4M.txt` must supply them (`L976–1000`). | 01-phase-index-MASTER.discovery.md:L653 | Open and deferred items |
| MASTER-Q509 | Q/SUMMARY | Inventory drift: L1037 says “38 files” but L1039–1056 enumerates 38 phase-detail files plus two split indexes, the MASTER, and `desktop.ini`; the current directory also has four extra phase-detail files. Treat MASTER's completeness claim as dated, not current (`L2, L1037–1056`). | 01-phase-index-MASTER.discovery.md:L654 | Open and deferred items |
| MASTER-Q510 | Q/SUMMARY | Current unindexed phase details: `phase-3J-BASELINE.txt`, `phase-3L.txt`, `phase-4D-ORACLE.txt`, `phase-4D-SURGICAL.txt` exist now but are not named in MASTER inventory; deeper agents must inspect them rather than infer content. | 01-phase-index-MASTER.discovery.md:L655 | Open and deferred items |
| MASTER-Q511 | Q/SUMMARY | Event enumeration drift: 3B says exactly 17 action types, 3E later uses `context_reduction`, 3H may add `branch_abandon`, and 4K adds two intervention types. The eventual canonical enum/migration needs reconciliation (`L416–423, L514–518, L590–592, L961–967`). | 01-phase-index-MASTER.discovery.md:L656 | Open and deferred items |
| MASTER-Q512 | Q/SUMMARY | Pool guidance drift: 3B explicitly chooses r2d2 while cross-cutting anti-stale guidance says deadpool for new code; original/implementation scope must clarify grandfathering (`L418–420, L1013–1014`). | 01-phase-index-MASTER.discovery.md:L657 | Open and deferred items |
| MASTER-Q513 | Q/SUMMARY | Dashboard command count: 4K.1 names eight viewer queries while 4K.2 says six Tauri wrappers; original plan must map which queries are combined or not exposed (`L961–969`). | 01-phase-index-MASTER.discovery.md:L658 | Open and deferred items |
| MASTER-Q514 | Q/SUMMARY | “Phase 2+” wording: 1F says revisit in Phase 2+ vLLM provider but vLLM is Phase 3J; interpret as any post-Phase-1 implementation, not a Phase-2 dependency (`L168–172, L646–656`). | 01-phase-index-MASTER.discovery.md:L659 | Open and deferred items |
| MASTER-Q515 | Q/SUMMARY | Unresolved exact file: `SKILL.md` is referenced generically for FSM workflow patterns (`L394–395`), but no directory is supplied and the repository has multiple skill files; `phase-3A.txt` should identify the intended one. | 01-phase-index-MASTER.discovery.md:L739 | C. Cross-spec and decision documents explicitly referenced by the MASTER |

---

# User-facing surfaces

Every window, panel, command, CLI verb, flag, status display, and notification atom of the PHASE family. IDs reference full atoms above. Feeds dashboard DESIGN.md.

## Windows / shell hosts

- `P4K-E026` — CLI elai --dashboard; focus existing window if already running. (phase-4K:L157-L158; PHASE-001)
- `P4M-A002` — RuntimePanel vs SettingsWindow placement open. (phase-4M:L376-L376; PHASE-001)
- `P4A-E042` — Operator-facing isolation surfaces: generic policy rejection string; human approval for MAJOR replan (INTERACTIVE gate); human approval for auditor PROPOSED rul… (4A:L67-L68, L215-L216, L228-L230, L33-L35; PHASE-006)

## Panels, drawers, workspace controls

- `P4K-E029` — Project selector: logo, subtitle "a deterministic development engine", two creation paths. (phase-4K:L197-L203; PHASE-001)
- `P4K-E037` — Workspace: plan tree left 320px + activity feed right; draggable divider; min 280px; double-click 50/50. (phase-4K:L277-L280; PHASE-001)
- `P4K-E038` — Status bar 24px: project (FSM state | session; EXECUTING teal. | phase-4K:L281-L282; PHASE-001)
- `P4K-E039` — Plan tree phases/steps/baby-steps; --> active; done/active/pending styles; expand active; elapsed times. (phase-4K:L283-L291; PHASE-001)
- `P4K-E040` — Feed [reasoning] teal-ghost collapsed 2-3 lines. (phase-4K:L294-L295; PHASE-001)
- `P4K-E041` — Feed [edit] path+lines +N -M. (phase-4K:L296-L296; PHASE-001)
- `P4K-E042` — Feed [validate] checkmark/x one-liner. (phase-4K:L297-L297; PHASE-001)
- `P4K-E043` — Feed [test] pass/fail counts. (phase-4K:L298-L298; PHASE-001)
- `P4K-E044` — Feed [preview] teal link opens system browser via shell::open. (phase-4K:L299-L301, L362-L368; PHASE-001)
- `P4K-E049` — Diff viewer on ApproveChange: unified teal add / fail remove / sticky header. (phase-4K:L343-L350; PHASE-001)
- `P4K-I001` — PhaseStatus and InterventionRequest fields imply plan-tree/card data contracts. (phase-4K:L76-L76, L83-L86; PHASE-001)
- `P4K-A003` — Diff viewer replace feed vs modal open. (phase-4K:L345-L345; PHASE-001)
- `P4M-E028` — RuntimePanel/Settings: per-role backend, chain, live health g/y/r, Test backend button. (phase-4M:L376-L380; PHASE-001)
- `P4G-E028` — INTERACTIVE persona exhaustion renders InterventionCard with all 5 personas and failure reasons. (4G:L810-L813; PHASE-002)
- `P4DS-E011` — Dashboard SurgicalDrawer shows ComplexityClass + AffectedSet on plan creation. (phase-4D-SURGICAL:L122-L122; PHASE-003)
- `P4DS-A002` — SurgicalDrawer field layout beyond ComplexityClass+AffectedSet not specified in this file. (phase-4D-SURGICAL:L122-L122; PHASE-003)
- `P3L-I001` — `/profile` is the primary operator CLI surface for reloading ProjectProfile; CONVENTIONS block is agent-visible prompt surface not a dashboard panel. (3L:L148-L151, L119-L121; PHASE-004)
- `P4I-I001` — HumanReview/Degraded verdicts are operator-facing outcomes (review queue semantics) even though no dashboard panel is specified in this phase file. (4I:L81, L277-L287; PHASE-004)
- `P2A-E033` — No dashboard panel in 2A; operator/engine surface is GrcEngine API and typed errors; Status free-echo aligns with mcp_think(step_type="status"). (2A:L207-L209, L373-L373; PHASE-005)
- `P3F-E045` — No dedicated rules dashboard panel in 3F phase file; enforcement surfaces are evaluate verdicts, seed DB, PostToolUse WARN/INFO strings, and transition logs. (3F:L309-L314, L165-L165; PHASE-005)
- `P4A-I001` — No new dashboard panel is specified in 4A; isolation is infrastructure with generic IPC error strings as the agent-visible surface. (4A:L67-L68; PHASE-006)
- `P4D-I001` — present_options Interactive gate is a user-facing selection surface; concrete panel/command wiring is not named in this phase file. (phase-4D:L531-L533, L622-L622; PHASE-007)
- `P3C-I001` — HUMAN-facing surfaces in 3C are INTERACTIVE escalation panels for max-restarts and spiral (options listed); no dedicated dashboard panel is named. (3C:L348-L349, L411-L413; PHASE-008)
- `P3E-A002` — No dedicated cockpit/panel for budget is named; only Display + escalate breakdown. (3E:L210, L163-L164; PHASE-008)
- `P4F-I001` — INTERACTIVE Escalate decision is human-facing; no dedicated panel name beyond decision enum. (4F:L97-L98; PHASE-008)

## Tauri / dashboard commands

- `P4K-E019` — Tauri command get_fsm_state. (phase-4K:L140-L141; PHASE-001)
- `P4K-E020` — Tauri command get_phase_progress. (phase-4K:L142-L142; PHASE-001)
- `P4K-E021` — Tauri command get_recent_actions(limit,offset). (phase-4K:L143-L143; PHASE-001)
- `P4K-E022` — Tauri command get_pending_interventions. (phase-4K:L144-L144; PHASE-001)
- `P4K-E023` — Tauri command get_file_changes(session_id). (phase-4K:L145-L145; PHASE-001)
- `P4K-E024` — Tauri command submit_intervention_response(request_id,chosen,note). (phase-4K:L146-L146; PHASE-001)
- `P4K-E033` — Commands list_projects, select_project, create_project; name alnum-_; inline red errors. (phase-4K:L215-L224; PHASE-001)
- `P4K-E034` — Template picker Svelte/React/Astro/Vanilla from templates/web/manifest.toml; list_templates. (phase-4K:L225-L229; PHASE-001)
- `P4K-E035` — Task input: header, placeholder, Start purple, Enter submit; submit_task command. (phase-4K:L230-L237; PHASE-001)
- `P4K-E053` — Tauri command get_action_detail(action_id) full payload. (phase-4K:L369-L372; PHASE-001)
- `P4J-E033` — Host scaffolding includes Tauri command surface + Svelte panel + feature gate (EMPIRE-PENDING). (4J:L16-L18; PHASE-008)
- `P4J-I001` — Svelte panel and Tauri commands are required scaffolding surfaces but exact command/panel names are not listed in this phase file. (4J:L16-L18; PHASE-008)

## CLI verbs and flags

- `P3JB-E018` — CLI: python baseline_harness.py --profile micro_7b_12gb; expect exit 0 and util≥0.82. (phase-3J-BASELINE:L125-L128; PHASE-001)
- `P1B-E019` — User-facing CLI surface: `elai-cli index build --project-root <path>` builds the BM25 index (WIRING trigger). (phase-1B:L447-L448; PHASE-003)
- `P3J-E042` — --local flag sets BackendConfig mode Local; no flag → Api from env auth; elai.bat simplified to --local + HOME fallback. (3J:L814-L815, L862-L862; PHASE-005)
- `P3J-E057` — 3J.14 WIRE.E2E: elai.bat single command; primary sidecar DoLa+TurboQuant+NF4; ollama fallback; QueryPipeline retrieval; release elai.exe. (3J:L1246-L1253; PHASE-005)
- `P4H-E044` — Anti-staleness wiring: 4H triggers nightly elai dep-audit CLI; outdated deps → stale_api_catches → Layer1A deprecated_patterns → steering vector refresh metrics… (4H:L1099-L1106; PHASE-006)
- `P4H-E069` — CLI surface: `elai dep-audit` nightly anti-staleness trigger invoked by 4H loop. (4H:L1100-L1100, L1144; PHASE-006)
- `P4D-E049` — ResearchTierConfig auto-detect GPU VRAM (nvidia-smi/CUDA) or manual `--vram-tier` flag; detect_vram_tier rounds DOWN; no-GPU falls back 8gb. (phase-4D:L1059-L1079; PHASE-007)

## Slash commands

- `P0-E012` — R25: commands/lib.rs has 28+ claw-code slash commands that conflict with elai_cc /task, /config, /continue. (phase-0:L32-L33; PHASE-001)
- `P0-E055` — 0.7c slash commands /task, /config, /continue, /explore, /status; bare messages fallback to /task; unknown command error. (phase-0:L517-L519; PHASE-001)
- `P3A-E008` — CONFIGURING via /config or /workflow; config_paths_only tools; no evidence gate; ->IDLE on completion. (phase-3A:L60-L63; PHASE-001)
- `P4M-E023` — CLI /runtime show. (phase-4M:L344-L345; PHASE-001)
- `P4M-E024` — CLI /runtime set <role> <backend>. (phase-4M:L346-L346; PHASE-001)
- `P4M-E025` — CLI /runtime fallback <role> <b1>,<b2>. (phase-4M:L347-L347; PHASE-001)
- `P4DOR-E013` — reachable_from: /research and /ask slash commands (commands.rs Research / Ask variants). (4DOR:L108-L109; PHASE-002)
- `P4G-E012` — Mode switch CLI: `/mode autonomous` or `/mode interactive`. Takes effect at next gate evaluation, not mid-phase. Mode per-session. (4G:L327-L333, L245-L248; PHASE-002)
- `P4G-E033` — Runtime-selection UX (summary-aligned exit): elai --local sidecar, --hybrid per-role, default cloud; /model live change; [agents.{role}.backend]=local|api|auto … (4G:L1-L12, exit context via MASTER 4G runtime lines (orientation); source body focuses gates — see P4G-I001; PHASE-002)
- `P4G-I001` — Runtime flags --local/--hybrid and /model are asserted by MASTER phase-index summary for 4G; this source file’s body does not re-list them after the GOAL/read l… (4G:L14-L27; MASTER 4G L871-L873; PHASE-002)
- `P4DS-E002` — `/task --surgical "<prompt>"` replaces ConceptDecomposition with DKG blast-radius surgical plan; flag via `parse_task_flags` commands.rs:2414. (phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89; PHASE-003)
- `PIDX-E013` — 3J.LOCAL/4G model UI: CLI `--local`/`--api`/`--hybrid`; runtime `/model`; config `[agents.{role}.backend]` and `[backend] mode`. (phase-index-3A-4J:L156-L160, L310-L318; PHASE-003)
- `PIDX-E020` — 4G AutonomousGate; `/mode autonomous\|interactive` next-gate; 6 evidence checks; SPLIT_TASK max 4 depth 2; TreeAutoAdvancePolicy circuit 50 baby steps/concept. (phase-index-3A-4J:L290-L318; PHASE-003)
- `PIDX-E024` — Slash/CLI surfaces named in index: INTERACTIVE gate verbs; `/plan` (3J.WIRE.PLAN); `/mode`; `/model`; `elai --local/--hybrid`; `elai.bat`. (phase-index-3A-4J:L12-L12, L166-L170, L300-L317; PHASE-003)
- `P3J-E038` — Slash /model and /mode switch backend at runtime; elai --local uses orchestrator pipeline; default elai uses API same pipeline. (3J:L737-L737, L765-L767; PHASE-005)
- `P3J-E052` — 3J.WIRE.PLAN: optional PlanSkeleton from .elai_cc/plans/{pipeline_id}.toml; /plan displays skeleton; /plan phase <id> shows PhaseDetail; Slot 2 phase context; c… (3J:L1063-L1103; PHASE-005)
- `P3J-E067` — Addendum surfaces: /research JSON-mode uses constrained decoding; COMMITTING FSM phase invokes git_commit tool; calibrate_etc_threshold.py; data/steering_contra… (3J:L1525-L1569; PHASE-005)
- `P3J-E068` — User-facing CLI verbs/flags in 3J body: elai / elai --local; elai.bat; /mode local\|api\|hybrid; /model; /plan; /plan phase <id>; /resume; /session fork; /compa… (3J:L737-L767, L814-L831, L895-L898, L1027-L1052, L1078-L1079, L1268-L1273, L1307-L1314; PHASE-005)
- `P4J-E034` — brainctl CLI replacement: orchestrator drives FactoryLine (start/stop/status brain lifecycle). (4J:L343-L349, L704; PHASE-008)

## Status displays, gates, notifications, operator feedback

- `P3A-E024` — INTERACTIVE gate: structured EvidenceSummary present_gate; not prose. (phase-3A:L205-L209; PHASE-001)
- `P4C-E018` — No agent-visible UI for hooks; operator-facing artifacts are SSB findings, metrics TOML, generic rejection errors on agent channel. (4C:L4-L5, L355-L359; PHASE-004)
- `P1F-E044` — No dashboard UI in 1F; operator-visible surfaces are decision markdown, metrics TOMLs, and probe SUPPORTED/NOT_SUPPORTED console output. (1F:L10-L14, L151-L152, L285-L289; PHASE-005)
- `P3J-E045` — CliInteractiveGate: present_gate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances. (3J:L895-L898; PHASE-005)
- `P3K-I001` — 3K has no new operator UI; surfaces are QueryTier API, metrics TOML artifacts, and retrieval result snippet fields. (3K:L156-L157, L181-L182, L308-L308; PHASE-006)
- `P4H-E035` — Stage3 promotion gate requires better trigger task AND no SWE regression AND no auditor safety regressions AND no evidence-gate definition mods → ACTIVE demote … (4H:L612-L615; PHASE-006)
- `P4D-E034` — INTERACTIVE mode presents alternatives with evidence for user pick; AUTONOMOUS selects highest evidence_level * source_quality. (phase-4D:L531-L535, L645-L647; PHASE-007)
- `P3C-E056` — INTERACTIVE spiral escalation UI text: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute … (3C:L411-L413; PHASE-008)
- `P3E-E035` — ContextBudget implements Display for debug-friendly budget breakdown; INTERACTIVE terminal surfaces Escalate(breakdown) to user. (3E:L210-L210, L163-L164, L324; PHASE-008)

## Other operator / agent-visible surfaces

- `P0-E026` — 0.2g KEEP candidates: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. (phase-0:L146-L148; PHASE-001)
- `P0-E027` — 0.2g STRIP claw-code commands: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. (phase-0:L149-L150; PHASE-001)
- `P4K-E002` — Native structured control surface not chatbot; inputs via FSM classification; MVP interactive mode. (phase-4K:L8-L12; PHASE-001)
- `P4K-E006` — Visual: teal #1B7A6E on charcoal #0f1318; JetBrains Mono; density over whitespace. (phase-4K:L28-L31; PHASE-001)
- `P4K-E027` — CSS tokens bg-void/base/surface/hover/input; teal/purple scales; semantic pass/warn/fail/info; mono font scale xs–xl. (phase-4K:L159-L167; PHASE-001)
- `P4K-E030` — + New Project: name+parent picker; create_dir_all; list add; template+task without leaving app. (phase-4K:L204-L208; PHASE-001)
- `P4K-E031` — Open Existing: native folder dialog. (phase-4K:L209-L210; PHASE-001)
- `P4K-E032` — Recent projects: name/path/last opened; max 10; hover style; source elai_cc.toml [projects] or ~/.elai_cc/projects.json. (phase-4K:L211-L214; PHASE-001)
- `P4K-E045` — Step headers; auto-scroll; Jump to latest pill when scrolled up. (phase-4K:L302-L305; PHASE-001)
- `P4K-E050` — Diff Accept / Decline / Retry+note; per-hunk Accept/Decline on hover. (phase-4K:L351-L355; PHASE-001)
- `P4K-E051` — Multi-file vertical list navigation. (phase-4K:L356-L357; PHASE-001)
- `P3G-E022` — Skeleton TOML format example under [phase.3B] with goal/inputs/outputs/constraints/depends_on/depended_by/version. (3G:L95-L105; PHASE-002)
- `P4G-E024` — Persona agent files: `.claude/agents/persona-{hacker,researcher,simplifier,architect,contrarian}.md`. (4G:L624-L631, L861-L864; PHASE-002)
- `P2C-E017` — TOML surfaces: `[grc.defaults]`, `[agents.executor.grc]`, `[agents.auditor.grc]`, `[agents.context_compiler.grc]`, `[agents.memory_writer.grc]`, `[agents.prompt… (2C:L54-L66, L23, L445-L447; PHASE-004)
- `P3L-E012` — PiT1Hook::load + ProjectIntelligenceHook::render_block → CONVENTIONS block for SYSTEM via build_with_conventions. Format: per-category summaries + error_crate +… (3L:L115-L126, L200-L201; PHASE-004)
- `P4I-E019` — Functional viewports web 375/768/1440; Blender 4 angles; FFmpeg keyframes; ComfyUI mock never real in tests. visual_check=false → no render overhead. (4I:L449-L469, L458-L459; PHASE-004)
- `P1F-E026` — Decision artifact path: workflow/reasoning/phase-1F-decisions.md documents fallback and ADOPT/DROP with measurements. (1F:L152-L153, L161-L161, L285-L285, L315-L315; PHASE-005)
- `P3F-E037` — Post-tool completeness GREP rules (Tier1, engineering, PostToolUse): web_fetch <img> → WARN "N images not extracted"; <table> → WARN "N tables not extracted"; f… (3F:L309-L314; PHASE-005)
- `P4H-E034` — Trigger task surface: .elai_cc/benchmarks/trigger_task.toml with input, expected_output_hash sha256, metric_key — project-specific canary against objective hack… (4H:L589-L598; PHASE-006)
- `P4H-E038` — DIVERGING MSI alerts user and pauses self-mod for that PipelineMode only (never silently continues). (4H:L746-L746, L775-L779; PHASE-006)
- `P4H-I001` — self_mod_history.db and phase_4H.toml are operator-visible measurement/history surfaces for later cockpit consumption. (4H:L751-L751, L1220-L1221; PHASE-006)
- `PP0-E011` — Failing plugin tests: `collects_and_runs_hooks_from_enabled_plugins` (L395, pre.sh), `pre_tool_use_denies_when_plugin_hook_exits_two` (L456, printf/exit 2), `pr… (pre-phase-0:L72-L81; PHASE-007)
- `P1I-E034` — User notified before rebuild (not silent). (phase-1I:L290-L290, L302-L302; PHASE-007)
- `P2D-I001` — Human approval for MAJOR replan is an interactive gate surface owned by orchestrator, not implemented in reasoning crate in this phase. (phase-2D:L70-L73, L250-L252; PHASE-007)
- `P4D-E057` — SSB research_step / research_decompose_level / web_research_step intended for INTERACTIVE mode operator visibility of research progress. (phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183; PHASE-007)
- `P4D-A003` — Exact Interactive UI for present_options (CLI, dashboard modal, gate API) is not specified. (phase-4D:L531-L533, L645-L647; PHASE-007)
- `P4B-I001` — Tools are agent-facing IPC surfaces, not human UI; human sees outcomes via auditor/SSB/dashboard elsewhere. (4B:L17-L18; PHASE-008)
- `P4J-E035` — Capability discovery action "capabilities" returns exactly 4 entries. (4J:L423-L427; PHASE-008)

## MASTER summary surface-related bullets (orientation)

- `MASTER-E001` — P0 goal: make `claw-code-rust-port` compile and pass all 684 tests on Windows without feature changes (`L10–11`). (01-phase-index-MASTER.discovery.md:L15)
- `MASTER-E002` — P0.1: fix ten compile errors by `cfg`-gating `std::os::unix::PermissionsExt` and `set_mode` in `runtime/mcp_stdio.rs`, `runtime/mcp_tool_bridge.rs`, and `rusty-… (01-phase-index-MASTER.discovery.md:L16)
- `MASTER-E003` — P0.2: make four plugin-hook failures Windows-compatible by translating `.sh` to `.cmd` for `write_hook_plugin` and making `HookRunner::run_hook_command` dispatc… (01-phase-index-MASTER.discovery.md:L17)
- `MASTER-E004` — P0.3: fix four tool tests through path-separator normalization, `python3` → `python` on Windows, and poisoned environment-mutex recovery (`L17–18`). (01-phase-index-MASTER.discovery.md:L18)
- `MASTER-E007` — Exit gate: `cargo check` must have zero errors and zero warnings and `cargo test` must have zero failures on Windows (`L21`). (01-phase-index-MASTER.discovery.md:L21)
- `MASTER-E012` — 0.2 strip set: remove `telemetry/`, `compat-harness/`, and runtime modules `usage`, `compact`, `summary_compression`, `lsp_client`, `oauth`, and `prompt`; repla… (01-phase-index-MASTER.discovery.md:L29)
- `MASTER-E016` — 0.4c top layer: create `orchestrator/` from the remaining runtime and `cli/` from `rusty-claude-cli` plus `commands/`; archive `.elai_cc/rust/`; keep `main.rs` … (01-phase-index-MASTER.discovery.md:L33)
- `MASTER-E021` — 0.7c CLI/runtime UX: commands `/task`, `/config`, `/continue`, `/explore`, `/status`; visibility policy with `allowed_paths`, `hidden_paths`, and allowlist sani… (01-phase-index-MASTER.discovery.md:L38)
- `MASTER-E025` — Phase output: exactly the named nine crates—`core`, `providers`, `knowledge`, `reasoning`, `orchestrator`, `tools`, `plugins`, `rules`, `cli`—plus the contract-… (01-phase-index-MASTER.discovery.md:L42)
- `MASTER-E035` — 1A.5 measurements: record build time, query p50/p99, DB size, node count, and edge count in `phase_1A.toml` (`L76`). (01-phase-index-MASTER.discovery.md:L55)
- `MASTER-E042` — 1B.4 fallback chunking: when DKG has no nodes, use 50-line sliding windows with ten-line overlap; overlap is explicitly ungrounded for BM25 and retained only fo… (01-phase-index-MASTER.discovery.md:L65)
- `MASTER-E047` — Resilience: per-model circuit breakers; startup must embed `"fn main()"` within five seconds; canonical model path `.elai_cc/models/onnx/`; a missing model degr… (01-phase-index-MASTER.discovery.md:L73)
- `MASTER-E049` — 1C.2 modules: `vector/{mod,cache,ort_session,circuit_breaker,hnsw_index}.rs` (`L104`). (01-phase-index-MASTER.discovery.md:L75)
- `MASTER-E067` — 1D.4 special files/fit: include a 13-item Rust special-file list such as `Cargo.toml`, `README.md`, `build.rs`; binary-search fitting is capped at ten iteration… (01-phase-index-MASTER.discovery.md:L96)
- `MASTER-E098` — 1H.1 build isolation: `splade.rs` behind `cfg(feature="splade")`; `ort` remains unconditional for GraphCodeBERT/Granite; only SPLADE session construction is gat… (01-phase-index-MASTER.discovery.md:L139)
- `MASTER-E113` — 1I.7 migration: index-version mismatch rebuilds and archives old index to `index_backup_{version}/`; missing version file defaults to v1 (`L247–248`). (01-phase-index-MASTER.discovery.md:L157)
- `MASTER-E115` — 3K.1 adaptive chunking: replace fixed 50-line BM25 windows with DKG/tree-sitter AST splitting; Rust boundaries `fn`/`impl`/`struct`/`enum`/`mod`, Python/TS/Go `… (01-phase-index-MASTER.discovery.md:L162)
- `MASTER-E132` — 2A.3 tests: 12 covering grounding-required observe, forced convergence, per-topic isolation, invalid transitions, convergence threshold, malformed grounding, fr… (01-phase-index-MASTER.discovery.md:L182)
- `MASTER-E140` — 2B.1 method metadata: `VerificationMethod` has six variants plus `requires_network`/`requires_tools`; typed method stored in `GroundingResult` and `Verification… (01-phase-index-MASTER.discovery.md:L193)
- `MASTER-E148` — 2B.4 claim/plan processing: strip grounding prefix before verifier dispatch; extract `type:reference` claims by regex; classify; attempt FILE/SOURCE auto-resolu… (01-phase-index-MASTER.discovery.md:L201)
- `MASTER-E160` — 2C.4 implementation: five config-validation rules; effective next states are `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})`; enforce minimum ground… (01-phase-index-MASTER.discovery.md:L216)
- `MASTER-E176` — 3A.2 phase-aware capability filters: PLANNING blocks edit/bash; EXECUTING blocks planning tools; AUDITING blocks edit/planning; MEMORIZING allows memory-bank to… (01-phase-index-MASTER.discovery.md:L238)
- `MASTER-E188` — 3B.1 payload/read-pool behavior: `Arc<[u8]>` for O(1) broadcast clones; disable idle timeout/max lifetime so default pool recycling does not close WAL (`L429–43… (01-phase-index-MASTER.discovery.md:L253)
- `MASTER-E194` — Port mapping: `WorkerRegistry→AgentPool`; UUID v4 from `EntropySource`; `WorkerStatus→AgentStatus` plus `Checkpointing`/`BudgetExceeded` and removal of `TrustRe… (01-phase-index-MASTER.discovery.md:L262)
- `MASTER-E197` — Eight roles: `Executor`, `Auditor`, `PromptEngineer`, `ContextCompiler`, `MemoryWriter`, `Researcher`, `MediaValidator`, `Designer`; designer differs by prompt/… (01-phase-index-MASTER.discovery.md:L265)
- `MASTER-E235` — Promotion gates: at least three contract tests; human/independent second-auditor approval, with finding-source auditor forbidden to self-approve; shadow→active … (01-phase-index-MASTER.discovery.md:L312)
- `MASTER-E247` — 3G.2 size/write gate: estimated precheck, exact count at 3600 safety margin, hard reject above 4096; every write emits plan update; missing detail emits `stub_r… (01-phase-index-MASTER.discovery.md:L327)
- `MASTER-E255` — Schema: snapshot UUID/parent/FSM/phase/step/findings/code changes/tests/commits/tag; context row role constrained to system/user/assistant/tool plus content/stu… (01-phase-index-MASTER.discovery.md:L338)
- `MASTER-E264` — 3I.1 platform spike: evaluate `madsim 0.2.34` on Windows with feature-gated dependency swap; document manual deterministic scheduler if it fails (`L622–624`). (01-phase-index-MASTER.discovery.md:L350)
- `MASTER-E275` — Installed/model assumptions: `gemma4:31b` 19 GB Q4/262144 context; `gemma4:e2b` 7.2 GB Q4_K_M; `gemma3:4b` fallback; vLLM 0.19.0 in WSL2 `~/vllm-env/` (`L653–65… (01-phase-index-MASTER.discovery.md:L364)
- `MASTER-E276` — 3J.1: `VllmProvider` for `/v1/completions`, `/v1/models`, SSE streaming (`L655–656`). (01-phase-index-MASTER.discovery.md:L365)
- `MASTER-E282` — 3J.10–3J.13: port 1751-LOC MCP bridge; static hooks T1<1 ms/T2<10 ms/T3<5 ms and no hot-path LLM; Windows AppContainer/job limits/self-mod locks; complete typed… (01-phase-index-MASTER.discovery.md:L373)
- `MASTER-C284` — Added composition: Python Anthropic-compatible composed inference sidecar auto-started by CLI; `--local` full interactive mode; `InferenceDispatcher` with API/l… (01-phase-index-MASTER.discovery.md:L375)
- `MASTER-E290` — BLOCK-3/4: agent binary has no LLM client, `reqwest`, or providers dependency; all inference is proxied; orchestrator externally counts think steps and owns GRC… (01-phase-index-MASTER.discovery.md:L384)
- `MASTER-E293` — BLOCK-7: Windows prefers AppContainer; restricted token + ACL deny is documented as weaker fallback (`L706–707`). (01-phase-index-MASTER.discovery.md:L387)
- `MASTER-E308` — ShortCoder: AST-equivalent compression with ten tree-sitter rules R1–R10; R4/R6 configurable (`L734`). (01-phase-index-MASTER.discovery.md:L405)
- `MASTER-E315` — Hook timing: pre fires before forwarding tool/bash; post runs ruff + T1/T2 + ShortCoder before result/response; stop requires FSM idle; precompact checkpoints a… (01-phase-index-MASTER.discovery.md:L415)
- `MASTER-E342` — Eight structural checks: registered tool, available model, acyclic graph via Kahn, hardware fit, measurable quality metrics, compiled predicates, novel-step mar… (01-phase-index-MASTER.discovery.md:L448)
- `MASTER-E357` — Mode architecture: autonomy adds no FSM state; `AutonomousGate` wraps an `InteractiveGate` with evaluator/mode/escalation policy/dispatch table (`L847–850`). (01-phase-index-MASTER.discovery.md:L469)
- `MASTER-E366` — Runtime-selection UX: `elai --local` sidecar, `--hybrid` per-role, default cloud API; `/model` live change; `[agents.{role}.backend] = local|api|auto` (`L871–87… (01-phase-index-MASTER.discovery.md:L478)
- `MASTER-E380` — Condition/scope: instantiate only for media pipelines; validate image, 3D, audio, UI screenshot (`L912–914, L923`). (01-phase-index-MASTER.discovery.md:L498)
- `MASTER-E381` — Tier 1: deterministic ImageMagick/FFmpeg/Tesseract/Blender CLI, target <100 ms, veto authority (`L914–915`). (01-phase-index-MASTER.discovery.md:L499)
- `MASTER-E389` — Condition/value exchange: only when Empire source available; ELAI becomes a brain and supplies self-improvement/reasoning/pipelines, receiving a live deployment… (01-phase-index-MASTER.discovery.md:L510)
- `MASTER-E393` — 4J.1 abstractions: cross-platform `FactoryLineTransport` with Unix/TCP; generic client; schema-validated `BrainManifest`; rate-limited/filtered event bridge; `E… (01-phase-index-MASTER.discovery.md:L514)
- `MASTER-E395` — Wire types: `FactoryLineCommand` (`Register|Heartbeat|List|RequestUpdate`) with protocol `v:u8`; nine-kind `IpcMessage`; shared Event/Command/Response (`L948–94… (01-phase-index-MASTER.discovery.md:L516)
- `MASTER-E396` — Degradation/build: generic transport, graceful operation without Empire, feature gate `empire` (`L950`). (01-phase-index-MASTER.discovery.md:L517)
- `MASTER-E398` — Stack: Tauri 2.10/build 2.5/CLI 2.10, Svelte/Vite webview, Rust commands; no HTTP, REST, WebSocket, or CORS; invoke-poll read model every 500 ms (`L956–958`). (01-phase-index-MASTER.discovery.md:L522)
- `MASTER-E401` — Intervention events: add `INTERVENTION_REQUEST` with type `ApproveChange|ReplanApproval|ErrorRecovery` and options `Accept|Decline|Retry(String)|Skip`; add `INT… (01-phase-index-MASTER.discovery.md:L525)
- `MASTER-E403` — 4K.2 scaffold: dashboard crate, six Tauri commands, Svelte/Vite/CSS, `elai --dashboard` (`L968–969`). (01-phase-index-MASTER.discovery.md:L527)
- `MASTER-E404` — UI surfaces: project selector, task input, phase ribbon, intervention panel, activity feed, reasoning chain, commit history (`L970–971`). (01-phase-index-MASTER.discovery.md:L528)
- `MASTER-E406` — Non-goals: do not build runtimes, use magic-keyword routing, or add LiteLLM; adapt existing CLIs/APIs, retain FSM+Designer routing, extend `LlmProvider` (`L982–… (01-phase-index-MASTER.discovery.md:L534)
- `MASTER-E407` — Backends: `Sidecar`, `AnthropicApi`, `ClaudeCode`, `CodexCli`, `OpenCode`, `Hermes`, `GeminiCli`, `KiroCli`, `CopilotCli`, `Ollama`, `Vllm` (`L984–985`). (01-phase-index-MASTER.discovery.md:L535)
- `MASTER-E409` — Factory config: CLI path, base URL, secret API key, model, permission mode (`AcceptEdits|Plan|DangerouslySkipPermissions|Default`), profile, CWD, timeouts (`L98… (01-phase-index-MASTER.discovery.md:L537)
- `MASTER-E418` — RAG hardening prerequisites: 3K depends on 1E+1G+3J because it uses the assembled retrieval/reranking path and sidecar generation for HyDE (`L250–260`). (01-phase-index-MASTER.discovery.md:L549)
- `MASTER-E420` — Shared model-runtime degradation: 1C/1G/1I circuit breakers emit the same SSB `knowledge_degradation` event; readiness gates the full FSM (`L114–117, L199–202, … (01-phase-index-MASTER.discovery.md:L551)
- `MASTER-E432` — Plan-to-execution chain: 4D Research+Designer creates grounded `PipelineSpec` and recursive decomposition; 4E deterministically builds/repairs/sizes it; 4F prob… (01-phase-index-MASTER.discovery.md:L563)
- `MASTER-E438` — Model substitution: 3C declares but disables `SubstituteModel`; 3J builds providers/pool; 4G supplies fit checking and fallback iteration; 4M generalizes the ba… (01-phase-index-MASTER.discovery.md:L569)
- `MASTER-E442` — Dashboard read/write split: 4K queries SSB through read-only WAL connections and writes user decisions as typed intervention events; no alternate HTTP/control p… (01-phase-index-MASTER.discovery.md:L573)
- `MASTER-E443` — 3J→4M refactor: 3J's bespoke `InferenceDispatcher` and backend mode are the legacy surface that 4M converts into `RuntimeAdapter`/factory/per-role runtime confi… (01-phase-index-MASTER.discovery.md:L574)
- `MASTER-Q448` — Hash coexistence: the MASTER still explicitly requires Blake2s in 1A/1C/3F/4C while its cross-cutting anti-stale line says Blake3 over Blake2s. Treat phase-loca… (01-phase-index-MASTER.discovery.md:L584)
- `MASTER-E451` — Isolation authority: after 4A agents have no network, model client, or tools; orchestrator owns GRC budget, inference, tools, and grounding over capped IPC (`L1… (01-phase-index-MASTER.discovery.md:L587)
- `MASTER-E454` — Build baseline/exit: Windows 684-test baseline; zero compile warnings/errors and zero failures before Phase 0 (`L10–22`). (01-phase-index-MASTER.discovery.md:L593)
- `MASTER-E472` — Context: sum never exceeds model window; SSB/GRC/output floors; deterministic prefix hashes; terminal overflow escalates/splits (`L503–520`). (01-phase-index-MASTER.discovery.md:L611)
- `MASTER-E479` — Tools/hooks: auditor-only validation; bounded reduction/retries; ShortCoder AST-equivalent; generic sanitized block errors; T3 never runs after a block (`L721–7… (01-phase-index-MASTER.discovery.md:L618)
- `MASTER-E481` — Builder: pure deterministic assembly; repair≤10; substitute only compatible I/O/category; eight checks; phase work≤60%; depth≤2 (`L800–828`). (01-phase-index-MASTER.discovery.md:L620)
- `MASTER-E486` — Empire: explicit size/versioned newline-JSON transport and narrow four-capability surface, no internal-state leakage (`L930–950`). (01-phase-index-MASTER.discovery.md:L625)
- `MASTER-E487` — Dashboard: SSB is read-only from viewer; no network server; every input classified by FSM; typed/paginated projections (`L952–974`). (01-phase-index-MASTER.discovery.md:L626)
- `MASTER-D489` — Master granularity: individual phase files still own the exhaustive pre-gates/tasks/checks/adversarial/acceptance details (`L1–4`). (01-phase-index-MASTER.discovery.md:L631)
- `MASTER-D501` — 3J: TurboQuant is FEASIBLE/DEFER; MILP is ADOPT/DROP and has Windows build risk; CPU offload is not fit for EAGLE-3 training (`L650–669`). (01-phase-index-MASTER.discovery.md:L645)
- `MASTER-D502` — 4A: Windows fallback is explicitly weaker; ShortCoder moves to 4B (`L706–719`). (01-phase-index-MASTER.discovery.md:L646)
- `MASTER-Q512` — Pool guidance drift: 3B explicitly chooses r2d2 while cross-cutting anti-stale guidance says deadpool for new code; original/implementation scope must clarify g… (01-phase-index-MASTER.discovery.md:L657)
- `MASTER-Q513` — Dashboard command count: 4K.1 names eight viewer queries while 4K.2 says six Tauri wrappers; original plan must map which queries are combined or not exposed (`… (01-phase-index-MASTER.discovery.md:L658)

## Full SURFACE-domain atom ID index (deep ledgers)

| ID | Class | Anchor | Ledger |
|---|---|---|---|
| P0-E012 | E/SURFACE | phase-0:L32-L33 | PHASE-001 |
| P0-E026 | E/SURFACE | phase-0:L146-L148 | PHASE-001 |
| P0-E027 | E/SURFACE | phase-0:L149-L150 | PHASE-001 |
| P0-E055 | E/SURFACE | phase-0:L517-L519 | PHASE-001 |
| P3A-E008 | E/SURFACE | phase-3A:L60-L63 | PHASE-001 |
| P3A-E024 | E/SURFACE | phase-3A:L205-L209 | PHASE-001 |
| P3JB-E018 | E/SURFACE | phase-3J-BASELINE:L125-L128 | PHASE-001 |
| P4K-E002 | E/SURFACE | phase-4K:L8-L12 | PHASE-001 |
| P4K-E006 | E/SURFACE | phase-4K:L28-L31 | PHASE-001 |
| P4K-E019 | E/SURFACE | phase-4K:L140-L141 | PHASE-001 |
| P4K-E020 | E/SURFACE | phase-4K:L142-L142 | PHASE-001 |
| P4K-E021 | E/SURFACE | phase-4K:L143-L143 | PHASE-001 |
| P4K-E022 | E/SURFACE | phase-4K:L144-L144 | PHASE-001 |
| P4K-E023 | E/SURFACE | phase-4K:L145-L145 | PHASE-001 |
| P4K-E024 | E/SURFACE | phase-4K:L146-L146 | PHASE-001 |
| P4K-E026 | E/SURFACE | phase-4K:L157-L158 | PHASE-001 |
| P4K-E027 | E/SURFACE | phase-4K:L159-L167 | PHASE-001 |
| P4K-E029 | E/SURFACE | phase-4K:L197-L203 | PHASE-001 |
| P4K-E030 | E/SURFACE | phase-4K:L204-L208 | PHASE-001 |
| P4K-E031 | E/SURFACE | phase-4K:L209-L210 | PHASE-001 |
| P4K-E032 | E/SURFACE | phase-4K:L211-L214 | PHASE-001 |
| P4K-E033 | E/SURFACE | phase-4K:L215-L224 | PHASE-001 |
| P4K-E034 | E/SURFACE | phase-4K:L225-L229 | PHASE-001 |
| P4K-E035 | E/SURFACE | phase-4K:L230-L237 | PHASE-001 |
| P4K-E037 | E/SURFACE | phase-4K:L277-L280 | PHASE-001 |
| P4K-E038 | E/SURFACE | FSM state | session; EXECUTING teal. | phase-4K:L281-L282 | PHASE-001 |
| P4K-E039 | E/SURFACE | phase-4K:L283-L291 | PHASE-001 |
| P4K-E040 | E/SURFACE | phase-4K:L294-L295 | PHASE-001 |
| P4K-E041 | E/SURFACE | phase-4K:L296-L296 | PHASE-001 |
| P4K-E042 | E/SURFACE | phase-4K:L297-L297 | PHASE-001 |
| P4K-E043 | E/SURFACE | phase-4K:L298-L298 | PHASE-001 |
| P4K-E044 | E/SURFACE | phase-4K:L299-L301, L362-L368 | PHASE-001 |
| P4K-E045 | E/SURFACE | phase-4K:L302-L305 | PHASE-001 |
| P4K-E049 | E/SURFACE | phase-4K:L343-L350 | PHASE-001 |
| P4K-E050 | E/SURFACE | phase-4K:L351-L355 | PHASE-001 |
| P4K-E051 | E/SURFACE | phase-4K:L356-L357 | PHASE-001 |
| P4K-E053 | E/SURFACE | phase-4K:L369-L372 | PHASE-001 |
| P4K-I001 | I/SURFACE | phase-4K:L76-L76, L83-L86 | PHASE-001 |
| P4K-A003 | A/SURFACE | phase-4K:L345-L345 | PHASE-001 |
| P4M-E023 | E/SURFACE | phase-4M:L344-L345 | PHASE-001 |
| P4M-E024 | E/SURFACE | phase-4M:L346-L346 | PHASE-001 |
| P4M-E025 | E/SURFACE | phase-4M:L347-L347 | PHASE-001 |
| P4M-E028 | E/SURFACE | phase-4M:L376-L380 | PHASE-001 |
| P4M-A002 | A/SURFACE | phase-4M:L376-L376 | PHASE-001 |
| P3G-E022 | E/SURFACE | 3G:L95-L105 | PHASE-002 |
| P4DOR-E013 | E/SURFACE | 4DOR:L108-L109 | PHASE-002 |
| P4G-E012 | E/SURFACE | 4G:L327-L333, L245-L248 | PHASE-002 |
| P4G-E024 | E/SURFACE | 4G:L624-L631, L861-L864 | PHASE-002 |
| P4G-E028 | E/SURFACE | 4G:L810-L813 | PHASE-002 |
| P4G-E033 | E/SURFACE | 4G:L1-L12, exit context via MASTER 4G runtime lines (orientation); source body focuses gates — see P4G-I001 | PHASE-002 |
| P4G-I001 | I/SURFACE | 4G:L14-L27; MASTER 4G L871-L873 | PHASE-002 |
| P1B-E019 | E/SURFACE | phase-1B:L447-L448 | PHASE-003 |
| P4DS-E002 | E/SURFACE | phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89 | PHASE-003 |
| P4DS-E011 | E/SURFACE | phase-4D-SURGICAL:L122-L122 | PHASE-003 |
| P4DS-A002 | A/SURFACE | phase-4D-SURGICAL:L122-L122 | PHASE-003 |
| PIDX-E013 | E/SURFACE | phase-index-3A-4J:L156-L160, L310-L318 | PHASE-003 |
| PIDX-E020 | E/SURFACE | phase-index-3A-4J:L290-L318 | PHASE-003 |
| PIDX-E024 | E/SURFACE | phase-index-3A-4J:L12-L12, L166-L170, L300-L317 | PHASE-003 |
| P2C-E017 | E/SURFACE | 2C:L54-L66, L23, L445-L447 | PHASE-004 |
| P3L-E012 | E/SURFACE | 3L:L115-L126, L200-L201 | PHASE-004 |
| P3L-I001 | I/SURFACE | 3L:L148-L151, L119-L121 | PHASE-004 |
| P4C-E018 | E/SURFACE | 4C:L4-L5, L355-L359 | PHASE-004 |
| P4I-E019 | E/SURFACE | 4I:L449-L469, L458-L459 | PHASE-004 |
| P4I-I001 | I/SURFACE | 4I:L81, L277-L287 | PHASE-004 |
| P1F-E026 | E/SURFACE | 1F:L152-L153, L161-L161, L285-L285, L315-L315 | PHASE-005 |
| P1F-E044 | E/SURFACE | 1F:L10-L14, L151-L152, L285-L289 | PHASE-005 |
| P2A-E033 | E/SURFACE | 2A:L207-L209, L373-L373 | PHASE-005 |
| P3F-E037 | E/SURFACE | 3F:L309-L314 | PHASE-005 |
| P3F-E045 | E/SURFACE | 3F:L309-L314, L165-L165 | PHASE-005 |
| P3J-E038 | E/SURFACE | 3J:L737-L737, L765-L767 | PHASE-005 |
| P3J-E042 | E/SURFACE | 3J:L814-L815, L862-L862 | PHASE-005 |
| P3J-E045 | E/SURFACE | 3J:L895-L898 | PHASE-005 |
| P3J-E052 | E/SURFACE | 3J:L1063-L1103 | PHASE-005 |
| P3J-E057 | E/SURFACE | 3J:L1246-L1253 | PHASE-005 |
| P3J-E067 | E/SURFACE | 3J:L1525-L1569 | PHASE-005 |
| P3J-E068 | E/SURFACE | 3J:L737-L767, L814-L831, L895-L898, L1027-L1052, L1078-L1079, L1268-L1273, L1307-L1314 | PHASE-005 |
| P3K-I001 | I/SURFACE | 3K:L156-L157, L181-L182, L308-L308 | PHASE-006 |
| P4A-E042 | E/SURFACE | 4A:L67-L68, L215-L216, L228-L230, L33-L35 | PHASE-006 |
| P4A-I001 | I/SURFACE | 4A:L67-L68 | PHASE-006 |
| P4H-E034 | E/SURFACE | 4H:L589-L598 | PHASE-006 |
| P4H-E035 | E/SURFACE | 4H:L612-L615 | PHASE-006 |
| P4H-E038 | E/SURFACE | 4H:L746-L746, L775-L779 | PHASE-006 |
| P4H-E044 | E/SURFACE | 4H:L1099-L1106 | PHASE-006 |
| P4H-I001 | I/SURFACE | 4H:L751-L751, L1220-L1221 | PHASE-006 |
| P4H-E069 | E/SURFACE | 4H:L1100-L1100, L1144 | PHASE-006 |
| PP0-E011 | E/SURFACE | pre-phase-0:L72-L81 | PHASE-007 |
| P1I-E034 | E/SURFACE | phase-1I:L290-L290, L302-L302 | PHASE-007 |
| P2D-I001 | I/SURFACE | phase-2D:L70-L73, L250-L252 | PHASE-007 |
| P4D-E034 | E/SURFACE | phase-4D:L531-L535, L645-L647 | PHASE-007 |
| P4D-E049 | E/SURFACE | phase-4D:L1059-L1079 | PHASE-007 |
| P4D-E057 | E/SURFACE | phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183 | PHASE-007 |
| P4D-I001 | I/SURFACE | phase-4D:L531-L533, L622-L622 | PHASE-007 |
| P4D-A003 | A/SURFACE | phase-4D:L531-L533, L645-L647 | PHASE-007 |
| P3C-E056 | E/SURFACE | 3C:L411-L413 | PHASE-008 |
| P3C-I001 | I/SURFACE | 3C:L348-L349, L411-L413 | PHASE-008 |
| P3E-E035 | E/SURFACE | 3E:L210-L210, L163-L164, L324 | PHASE-008 |
| P3E-A002 | A/SURFACE | 3E:L210, L163-L164 | PHASE-008 |
| P4B-I001 | I/SURFACE | 4B:L17-L18 | PHASE-008 |
| P4F-I001 | I/SURFACE | 4F:L97-L98 | PHASE-008 |
| P4J-E033 | E/SURFACE | 4J:L16-L18 | PHASE-008 |
| P4J-E034 | E/SURFACE | 4J:L343-L349, L704 | PHASE-008 |
| P4J-E035 | E/SURFACE | 4J:L423-L427 | PHASE-008 |
| P4J-I001 | I/SURFACE | 4J:L16-L18 | PHASE-008 |

---

# Superseded notes (newer-plan precedence)

Losing descriptions are **kept** in atomic tables. This section records **target state** winners.

| Supersession ID | Target state (winner) | Superseded description (loser kept) | Reason | Anchor |
|---|---|---|---|---|
| SUP-001 | P3C-E053 (blake3 content hash) | prior blake2s spiral hash | B-16A supersedes prior blake2s (explicit) | DEEP-PHASE-008 P3C-E053 3C:L398-L401 |
| SUP-002 | P4J-E011 PipelineSpec 6 layers | unvalidated brain.json workforce lambda stubs | explicit dead-code supersede in phase-4J | DEEP-PHASE-008 P4J-E011 4J:L79-L83 |
| SUP-003 | P4J-E034 FactoryLine lifecycle | brainctl CLI as operator driver | brainctl CLI replacement: orchestrator drives FactoryLine | DEEP-PHASE-008 P4J-E034 4J:L343-L349 |
| SUP-004 | P0-E025 core::TokenUsage (4 fields) | 0.2c TokenUsage shim (2 fields) | core TokenUsage replaces temporary shim at 0.4a; keep both as C conflict P0-C001 | DEEP-PHASE-001 P0-E025, P0-C001 |
| SUP-005 | P4DS-E002 /task --surgical SurgicalDecomposer | DefaultWebResearchRunner / ConceptDecomposition path for surgical flag | surgical flag replaces ConceptDecomposition when --surgical set | DEEP-PHASE-003 P4DS-E002, P4DS-E003 |
| SUP-006 | P4M-E015 [runtime] config keys | [backend] local|api|hybrid|auto | migrate [backend]→[runtime]; deprecated aliases one release with warning | DEEP-PHASE-001 P4M-E015 phase-4M:L218-L221 |
| SUP-007 | P4G-E021 Spinning stagnation (threshold=3) | standalone 3× GRC anti-spiral as sole detector | Existing 3× GRC anti-spiral SUBSUMED by Spinning, not removed | DEEP-PHASE-002 P4G-E029 4G:L451-L452 |
| SUP-008 | P1C-E007 SPLADE deferred to 1G/1H implement path (source body) | MASTER summary claim SPLADE deferred to 1H only | deep phase-1C body authoritative over MASTER compression (P1C-C002); 1H is implement phase | DEEP-PHASE-001 P1C-C002; MASTER:L74 |
| SUP-009 | P4I Mode A/B media models (media-validator authority) | architecture-overview LLaVA model list | phase binds media-validator as authority; overview stale (P4I-C001) | DEEP-PHASE-004 P4I-C001 4I:L41-L45 |
| SUP-010 | P3G-E011 stub_reference later detail write | prior stub_reference rows for same phase (logical) | Superseded on later detail write (append-only, no delete) | DEEP-PHASE-002 P3G-E011 3G:L146-L154 |
| SUP-011 | Deep phase files as authority over phase-index compression | index-only claims when deep phase files conflict (PIDX-I001 / IDX-I001) | indexes are not substitutes for deep phase files | DEEP-PHASE-003 PIDX-I001; DEEP-PHASE-008 IDX-I001 |
| SUP-012 | Deep ledger atoms as target state over MASTER compression | MASTER one-line bullets where deep ledger expands/corrects (all SD-*) | deep discovery exhaustive; MASTER orientation; SD-* record deltas | all SD-* records; 01-phase-index-MASTER as orientation |

### Plan-number precedence (no explicit SUPERSEDED marker; higher plan/phase wins target state)

| Note ID | Topic | Older (kept) | Newer target | Anchors |
|---|---|---|---|---|
| PREC-001 | FSM autonomy | P3A-E001 AUTONOMOUS deferred 4G | P4G-E001–E002 AutonomousGate | PHASE-001 3A; PHASE-002 4G |
| PREC-002 | Provider backend config | PIDX/P3J [backend] local|api|hybrid|auto | P4M-E015 [runtime] migration | PHASE-003/005; PHASE-001 4M |
| PREC-003 | Runtime selection UX | P4G-I001 MASTER orientation for --local/--hybrid | P4M-E023–E028 /runtime + RuntimePanel | PHASE-002 4G; PHASE-001 4M |
| PREC-004 | Agent isolation | P3C Tier A in-process | P4A Tier C process isolation | PHASE-008 3C; PHASE-006 4A |
| PREC-005 | BM25 chunking | P1B-E010 50-line windows | 3K AST adaptive chunking | PHASE-003 1B; PHASE-006 3K |
| PREC-006 | Tools ownership | Phase 0 tools crate strip | P4B orchestrator-owned tools via IPC | PHASE-001 0; PHASE-008 4B |
| PREC-007 | Budget authority | P2C-E015 in-process GrcEngine Phases 0–3 | Phase 4A demotes agent-side to advisory | PHASE-004 2C |
| PREC-008 | Hash policy spiral | blake2s (older 3C notes) | P3C-E053 blake3 B-16A | PHASE-008 |
| PREC-009 | SPLADE deferral target | MASTER says 1H only | P1C-E007 1G path; P1H implement when conditional | PHASE-001 P1C-C002; PHASE-006 1H |
| PREC-010 | Media model list | architecture-overview LLaVA | P4I Mode A/B media-validator | PHASE-004 P4I-C001 |

---

# Namespaced interaction index (all IX)

| Interaction ID | Class | Interaction | Evidence |
|---|---|---|---|
| PHASE-001/IX-001 | E | Phase 0 nine-crate split, LlmProvider, minimal FSM, visibility, msgpack, contracts, entropy quarantine are substrate for later phases here. | P0-E002, P0-E046, P0-E049–P0-E065 |
| PHASE-001/IX-002 | E | Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience. | P0-E015–P0-E017; P1C-E018, P1C-E029 |
| PHASE-001/IX-003 | E | Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine. | P0-E060–P0-E064; P1C-E026 |
| PHASE-001/IX-004 | E | Phase 1C SSB file_write invalidation and knowledge_degradation require Phase 3B SSB (outside chunk) but specify production wiring. | P1C-E021, P1C-E032 |
| PHASE-001/IX-005 | E | Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G. | P0-E052; P3A-E001–P3A-E029 |
| PHASE-001/IX-006 | E | Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG. | P3A-E019, P3A-E023; P1C-E029 |
| PHASE-001/IX-007 | E | Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal. | P3A-E033; P4K-E012, P4K-E047 |
| PHASE-001/IX-008 | E | Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI. | P3A-E024–P3A-E025; P4K-E013–P4K-E014, P4K-E049–P4K-E052 |
| PHASE-001/IX-009 | E | Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface. | P4K-E011–P4K-E026 |
| PHASE-001/IX-010 | E | Phase 3J-BASELINE fills capability_profile.toml for ModelCapabilityProfile without booting 3A/4K. | P3JB-E001–P3JB-E002, P3JB-E007–P3JB-E009 |
| PHASE-001/IX-011 | E | 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability. | P3JB-E011; P4M-E006 |
| PHASE-001/IX-012 | E | Phase 4M migrates 3J backends and Ollama/vLLM into RuntimeAdapter; requires 4A Tier C for CLI adapters. | P4M-E001, P4M-E014–P4M-E020 |
| PHASE-001/IX-013 | E | Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace. | P0-E055; P4K-E026; P4M-E023–P4M-E028 |
| PHASE-001/IX-014 | E | Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback. | P3A-E001; P4M-E022 |
| PHASE-001/IX-015 | E | Slash command evolution: Phase 0 /task|/config|/continue|/explore|/status; 3A /config|/workflow CONFIGURING; 4M /runtime family. | P0-E055; P3A-E008; P4M-E023–P4M-E025 |
| PHASE-001/IX-016 | I | RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here. | P1C-E029; P4M-E014, P4M-E028 |
| PHASE-001/IX-017 | E | 1C three-ranker RRF stays additive for later SPLADE/cross-encoder; semaphore overflow unranked. | P1C-E007, P1C-E030–P1C-E031 |
| PHASE-001/IX-018 | E | Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis. | P0-E053, P0-E056; P3A-E017 |
| PHASE-002/IX-001 | E | Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE. | P1A-E008, P1A-E012, P1A-E027; P1D-E006, P1D-E022 |
| PHASE-002/IX-002 | E | 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4). | P1A-E009, P1A-E010, P1A-E043; P1D-E001 |
| PHASE-002/IX-003 | E | 1D density-guard fallback is insertion-order, not BM25, because BM25 rerank is upstream in the query pipeline. | P1D-E011 |
| PHASE-002/IX-004 | E | 3D requires 3B SSB + 3C agents; upgrades 3C spawn_parallel stub to true JoinSet parallelism; 3E/3F consume ChannelRegistry. | P3D-E001, P3D-E019, P3D-E013 |
| PHASE-002/IX-005 | E | 3D checkpoint+SSB replay composes with 3C agent checkpoints under `.elai_cc/sessions/{sid}/`; UntrackedValue deliberately absent from both. | P3D-E015, P3D-E018, P3D-E011 |
| PHASE-002/IX-006 | E | 3G.4 requires 3E Slot 3 ContextBudget; skeleton always loads in PhaseSpec slot; overflow uses 3E ReducedGranularity. | P3G-E001, P3G-E015 |
| PHASE-002/IX-007 | E | 3G coherence is invoked by 3C auditor via IPC; findings use action_type finding; plan mutations use plan_update/stub_reference. | P3G-E010, P3G-E011, P3G-E016 |
| PHASE-002/IX-008 | E | 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing. | P3G-E014; P4G-E009 |
| PHASE-002/IX-009 | E | 4G SPLIT_TASK is the Terminal path consumer of 3E context-budget exhaustion; sequential merge avoids 3H/S10 pick-one branches. | P4G-E009, P4G-E010 |
| PHASE-002/IX-010 | E | 4G SUBSTITUTE_MODEL is the autonomous completion of 3C on_max_restarts policy (interactive escalates). | P4G-E011 |
| PHASE-002/IX-011 | E | 4G wraps 3A InteractiveGate without new FSM states; evidence gates from orchestrator S13; risk levels from arch-overview S6. | P4G-E001, P4G-E002, P4G-E005 |
| PHASE-002/IX-012 | E | 4G tree auto-advance integrates 4E TreeNavigator + BubbleUpHandler; concept-level always human. | P4G-E019, P4G-E020 |
| PHASE-002/IX-013 | E | 4G.9 Spinning subsumes prior 3× GRC anti-spiral; personas spawn via 4A Tier-C isolation. | P4G-E021, P4G-E023, P4G-E029 |
| PHASE-002/IX-014 | E | 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss. | P4DOR-E001, P4DOR-E002, P4DOR-E013 |
| PHASE-002/IX-015 | E | Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys. | P1A-E013, P1A-E035; P4DOR-E014; P4G-E021 |
| PHASE-002/IX-016 | E | SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*. | P3D-E017, P3D-E023; P3G-E007, P3G-E011; P4DOR-E012; P4G-E014, P4G-E022, P4G-E032 |
| PHASE-002/IX-017 | I | 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks. | P1A-E039; P1D-E009; P4G-E005 |
| PHASE-002/IX-018 | E | 4G circuit breakers operate at two grains: phase auto-advances (default 10) and tree baby steps per concept (default 50). | P4G-E007, P4G-E019 |
| PHASE-002/IX-019 | E | Goal-drift Block and watchdog timeouts both hand off to 4E BubbleUpHandler rather than inventing a third recovery owner. | P4G-E026, P4G-E027 |
| PHASE-002/IX-020 | E | 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step. | P3D-E005; P4G-E026 |
| PHASE-003/IX-001 | E | 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish. | P1B-E007, P1B-E018, P1G-E006 |
| PHASE-003/IX-002 | E | 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators. | P1G-E012, P1G-E014, P1B-E018 |
| PHASE-003/IX-003 | E | 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget. | P3H-E010, P3H-A006, PIDX-E006 |
| PHASE-003/IX-004 | E | 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry. | P3H-E013-E014, PIDX-E003, PIDX-C001 |
| PHASE-003/IX-005 | E | 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots. | P3H-E009, PIDX-E006, PIDX-E008 |
| PHASE-003/IX-006 | E | 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth. | P4E-E010, PIDX-E008, PIDX-E020 |
| PHASE-003/IX-007 | E | 4E novel_step_ids consumed by 4F; visual_check adapters by 4I. | P4E-E027, PIDX-E019, PIDX-E022 |
| PHASE-003/IX-008 | E | 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks. | P4E-E018-E019, PIDX-E002, PIDX-E016 |
| PHASE-003/IX-009 | E | 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1. | P4E-E014, P4E-E020, PIDX-E017 |
| PHASE-003/IX-010 | E | 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds. | P4DS-E009, P4DS-E015, PIDX-E006 |
| PHASE-003/IX-011 | E | 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension. | P4DS-E006-E007, P1B-E001 |
| PHASE-003/IX-012 | E | Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path. | PIDX-E011, P1B-E010, P1B-E014 |
| PHASE-003/IX-013 | E | Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path). | PIDX-E012, P3H-E015 |
| PHASE-003/IX-014 | E | Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder. | P4DS-E002-E003, P4E-E001, PIDX-E017 |
| PHASE-003/IX-015 | E | RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition. | P4E-E013, PIDX-E002-E003 |
| PHASE-003/IX-016 | E | Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families. | P1G-E012, P4DS-E010, P4E-E021 |
| PHASE-003/IX-017 | E | User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G). | P1B-E019, P4DS-E002, P4DS-E011, PIDX-E013, PIDX-E024 |
| PHASE-003/IX-018 | E | 4E integration time ladder matches 4B IntegrationTestRunner levels in index. | P4E-E022, PIDX-E015 |
| PHASE-003/IX-019 | E | 4H MSI tracks SURGICAL/GREENFIELD modes; surgical CLI path is a related but separate C-30 feature. | PIDX-E021, P4DS-E001 |
| PHASE-003/IX-020 | E | Handoff 85% (3H) aligns with 4C PreCompact checkpoint-at-85% theme in index. | P3H-E002, PIDX-E016 |
| PHASE-004/IX-001 | E | 1E composes 1B BM25 + 1C vectors + 1D PageRank/DKG; leaves explicit stubs for 1G rerank and 3E token_fit. | P1E-E001, P1E-E006, P1E-E026 |
| PHASE-004/IX-002 | E | 1E density guard and circuit-breaker skips feed 1I resilience/mode cascade and QueryTrace stage list. | P1E-E007, P1E-E023, P1E-E027 |
| PHASE-004/IX-003 | E | 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy. | P2B-E012, P2B-E019, P2C-E006 |
| PHASE-004/IX-004 | E | 2B plan sweep supplies the ungrounded-zero evidence gate for 3A PLANNING→TESTING_FIRST / EXECUTING path. | P2B-E003, P2B-E020 |
| PHASE-004/IX-005 | E | 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC. | P2C-E006, P2C-E015, P2B-E005 |
| PHASE-004/IX-006 | E | 2C auditor double-verify + FINDING anti-rubber-stamp is last defense before acceptance; composes with 3A AUDITING evidence. | P2C-E004, P2C-E007 |
| PHASE-004/IX-007 | E | 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family. | P3L-E003, P4C-E009 |
| PHASE-004/IX-008 | E | 3L CONVENTIONS injection via reasoning prompt_builder and /profile CLI; telescope priorities sit upstream of agent SYSTEM context. | P3L-E012, P3L-E015, P3L-E007 |
| PHASE-004/IX-009 | E | 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline. | P3L-E002, P1E-E001 |
| PHASE-004/IX-010 | E | 4C PostToolUse ShortCoder placement resolves 4B tools.txt code_simplify + 4A WARN-3 under Tier C before auditor FS. | P4C-E003, P4C-E017 |
| PHASE-004/IX-011 | E | 4C T1 SCHEMA_REQUIRED Escalate to ProviderTier couples rules DSL to provider constrained decoding (outside 4C body). | P4C-E005, P4C-E011 |
| PHASE-004/IX-012 | E | 4C circular-trust normalization + AUDITOR PROPOSED freeze composes with rules.txt self-discovery and 3F-style promotion gates. | P4C-E010 |
| PHASE-004/IX-013 | E | 4I lazy media path depends on 4D MediaSpec / 3G visual_check+output_type and 4E adapter structural check #8. | P4I-E001, P4I-E012, P4I-E023 |
| PHASE-004/IX-014 | E | 4I VLM process uses 4A Tier C isolation + same postcard IPC as agents; ToolCapabilityFlags degrade without panicking. | P4I-E012, P4I-E020, P4I-E004 |
| PHASE-004/IX-015 | E | 4I VisualFeedbackLoop writes SSB visual_feedback so 3B journal and 3C/4G liveness observe long RCE iterations. | P4I-E018, P4I-E017 |
| PHASE-004/IX-016 | E | Hash split in chunk: 4C violation_patterns blake2s vs knowledge blake2 elsewhere; 4I does not introduce new content hash for media. | P4C-E010 |
| PHASE-004/IX-017 | E | Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary). | P2C-E015, P4C-E001 |
| PHASE-004/IX-018 | I | 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards. | P1E-E028, P2B-E021, P2C-E016, P3L-E017, P4C-E015, P4I-E014 |
| PHASE-004/IX-019 | E | 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth). | P4C-E013, P2B-E017 |
| PHASE-004/IX-020 | E | Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory. | P2C-E018, P4I-E001 |
| PHASE-005/IX-001 | E | 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite. | P1F-E004, P1F-E015, P1F-E016; P3J-E001, P3J-E009 |
| PHASE-005/IX-002 | E | 1F requires 1C ModelPool VRAM accounting for draft overhead measurement. | P1F-E007, P1F-E010 |
| PHASE-005/IX-003 | E | 2A freezes GrcEngine six-method trait so 2B can implement verify_claim/ground_plan without breaking BasicGrcEngine. | P2A-E016, P2A-E017 |
| PHASE-005/IX-004 | E | 2A scope enumeration WARNs feed agent context; orchestrator (outside 2A) enforces plan_action coverage. | P2A-E013 |
| PHASE-005/IX-005 | E | 3F depends on 3D complete and 1A tree-sitter via AstEvaluator; self-discovery needs 3C agent/auditor IPC. | P3F-E001, P3F-E006, P3F-E032 |
| PHASE-005/IX-006 | E | 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM). | P3F-E018, P3F-E019 |
| PHASE-005/IX-007 | E | 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7). | P3F-E032, P3F-E035; P3J-E028, P3J-E029 |
| PHASE-005/IX-008 | E | 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters. | P3J-E048, P3J-E054; P3F-E036 |
| PHASE-005/IX-009 | E | 3J workload_profile + advanced SSB types extend 3B core actions to ≥17 including phase_complete atomic boundary used later by 4A. | P3J-E023, P3J-E033 |
| PHASE-005/IX-010 | E | 3J.WIRE.SESSION composes 3H SessionDag trim 85% + stub recovery from SSB. | P3J-E050, P3J-E051 |
| PHASE-005/IX-011 | E | 3J.WIRE.PLAN composes 3G PlanSkeleton into CLI Slot 2 and /plan surfaces. | P3J-E052 |
| PHASE-005/IX-012 | E | 3J.WIRE.FOUNDATION InteractiveGate is the 3A/4G-shared human gate surface (Approve/Reject/Modify/Redirect). | P3J-E045 |
| PHASE-005/IX-013 | E | 3J.14 retrieval injects 1B/1E/1G pipeline pieces (BM25, CatRAG, ARAG, optional cross-encoder) into prompt Slot 4. | P3J-E060 |
| PHASE-005/IX-014 | E | 3J.15 SUBSTITUTE_MODEL 100B path anticipates 4G escalation; anti-spiral 3× same error swap. | P3J-E064 |
| PHASE-005/IX-015 | E | Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception. | P3F-E032; P3J-E029 |
| PHASE-005/IX-016 | E | 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers. | P2A-E007, P2A-E027; P3J-E037, P3J-E047 |
| PHASE-005/IX-017 | I | 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals. | P1F-E021, P1F-E032; P3J-E002 |
| PHASE-005/IX-018 | E | 3J MCP tools join native tools under uniform ToolRequest/ToolResult after 3C agents exist. | P3J-E025, P3J-E027 |
| PHASE-005/IX-019 | E | AppContainer self-mod locks prefigure 4A BLOCK-8 realpath+FS+git filters. | P3J-E031 |
| PHASE-005/IX-020 | E | DoLa disabled for prompt-engineer aligns with constrained-decoding-only role (addendum A-10). | P3J-E013, P3J-E066 |
| PHASE-006/IX-001 | E | Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this). | P1H-E027, P1H-E032; P3K-E003 |
| PHASE-006/IX-002 | E | 3K hardening composes 1E A-RAG/CatRAG stubs, 1G reranker, 1C embeddings, 1A DKG grammars, and 3J sidecar only for HyDE. | P3K-E001–P3K-E003, P3K-E010, P3K-E015–P3K-E018 |
| PHASE-006/IX-003 | E | 3K.4 QueryTier intentional quality selection is explicitly distinct from 1I SearchMode fault-degradation cascade. | P3K-E012 |
| PHASE-006/IX-004 | E | 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence. | P3I-E001–P3I-E002; P4A-E036; P4H-E001 |
| PHASE-006/IX-005 | E | 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary. | P3I-E018; P4A-E013–P4A-E014 |
| PHASE-006/IX-006 | E | 3I entropy quarantine (zero raw time/rng in orchestrator) is a hard PRE_GATE for BSP determinism and for any later seed-replay of isolation tests. | P3I-E016 |
| PHASE-006/IX-007 | E | 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch. | P4A-E001; P4H-E001, P4H-E027, P4H-E038 |
| PHASE-006/IX-008 | E | 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging. | P4A-E016; P4H-E027, P4H-E038 |
| PHASE-006/IX-009 | E | 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern. | P4A-E008; P4H-E026, P4H-E032 |
| PHASE-006/IX-010 | E | 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list. | P4A-E010, P4A-E034; P4H-E027, P4H-E038 |
| PHASE-006/IX-011 | E | 4A ShortCoder deferral to 4B orchestrator placement is a cross-phase WARN-3 handoff outside this chunk's implementation. | P4A-E035 |
| PHASE-006/IX-012 | E | 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria. | P4A-E018, P4A-E038; P4H-E048 |
| PHASE-006/IX-013 | E | 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance. | P4H-E009, P4H-E031, P4H-E043 |
| PHASE-006/IX-014 | E | 4H FailureTaxonomy expansion closes reasoning-architecture Layer1 with fix templates from higher layers' successful resolutions. | P4H-E010, P4H-E032, P4H-E044 |
| PHASE-006/IX-015 | E | 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop. | P4H-E034–P4H-E036; P3K-E014; P1H-E024–P1H-E026 |
| PHASE-006/IX-016 | E | 4H OSC/REG/EVO ouroboros ports operate per PipelineMode in parallel with MSI and feed SSB action types. | P4H-E040–P4H-E042, P4H-E045 |
| PHASE-006/IX-017 | E | 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc. | P3K-E017; P4A-E021 |
| PHASE-006/IX-018 | E | Budget authority handoff: 2C in-process budgets become advisory under 4A orchestrator ThinkStep counting. | P4A-E006, P4A-E023 |
| PHASE-006/IX-019 | I | 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation. | P3K-E003; P1H-E027 |
| PHASE-006/IX-020 | E | Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT. | P4A-E032; P4H-E041 |
| PHASE-006/IX-021 | E | 3I VCR uses blake2s for prompt hashing while much of ELAI elsewhere bans blake2; this phase-local choice is intentional for cassette keys. | P3I-I001 |
| PHASE-006/IX-022 | E | Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network. | P4A-E003–P4A-E004; P3K-E010 |
| PHASE-007/IX-001 | E | Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D. | PP0-E001–PP0-E022; P1I-E012; P2D-E013; P3B-E028; P4D-E019 |
| PHASE-007/IX-002 | E | Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path. | P1I-E016; P3B-E010, P3B-E019–P3B-E020 |
| PHASE-007/IX-003 | E | Phase 1I ReadinessStatus READY\|DEGRADED\|UNAVAILABLE gates orchestrator FSM IDLE→PLANNING; UNAVAILABLE blocks all transitions (3A evidence gate consumer). | P1I-E022–P1I-E024 |
| PHASE-007/IX-004 | E | Phase 1I SearchMode cascade and BM25_ONLY degraded mode compose with 1B/1C isolation verified under degradation. | P1I-E013–P1I-E018, P1I-E036 |
| PHASE-007/IX-005 | E | Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate. | P1I-E006; P3B-E019 |
| PHASE-007/IX-006 | E | Phase 2D Blocked outcome signals ReplanRequest(MAJOR) for orchestrator 3A+ human-approval EXECUTING→PLANNING. | P2D-E007, P2D-E022; P2D-E015 |
| PHASE-007/IX-007 | E | Phase 2D CSQ depends on 2C AgentGrcConfig.csq_required and per-role CSQ mandates. | P2D-E001, P2D-E021 |
| PHASE-007/IX-008 | E | Phase 2D inline feasibility SourceMatch is temporary; 4F adds WebSearch/Grep/ApiRegistryCheck variants and deeper contradiction analysis. | P2D-E009, P2D-E022; P2D-C001 |
| PHASE-007/IX-009 | E | Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events. | P3B-E002–P3B-E004; P4D-E018, P4D-E037, P4D-E054 |
| PHASE-007/IX-010 | E | Phase 3B WARN-6 write-before-ToolResult contract binds agent tool IPC to durable journal order. | P3B-E020 |
| PHASE-007/IX-011 | E | Phase 3B PhaseCompletePayload fsync-before-unload durability rule composes with agent lifecycle unload paths. | P3B-E024 |
| PHASE-007/IX-012 | E | Phase 4D Research Agent Tier C isolation reuses Phase 4A process walls and IPC proxy patterns. | P4D-E001, P4D-E014, P4D-E053 |
| PHASE-007/IX-013 | E | Phase 4D Designer GRC config reuses 2C role specialization pattern (distinct from Executor). | P4D-E005 |
| PHASE-007/IX-014 | E | Phase 4D SystemConstraints sizing defaults feed later L2 sizing/feasibility consumers. | P4D-E006 |
| PHASE-007/IX-015 | E | Phase 4D package API_REGISTRY_CHECK reuses 2B verification method family. | P4D-E032, P4D-E035 |
| PHASE-007/IX-016 | E | Phase 4D Interactive present_options is a human-gate surface sibling to 3A InteractiveGate / later dashboard interventions. | P4D-E034, P4D-I001 |
| PHASE-007/IX-017 | C | Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension. | P4D-C001; P3B-E010; P4D-A001 |
| PHASE-007/IX-018 | E | Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer. | P1I-E034; P4D-E057 |
| PHASE-007/IX-019 | I | Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness. | PP0-E025; P3B-E010, P3B-E012 |
| PHASE-007/IX-020 | E | Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml. | PP0-E021; P2D-E019; P4D-E025, P4D-E049, P4D-E056 |
| PHASE-008/IX-001 | E | 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection. | P3C-E009; P3E-E015 |
| PHASE-008/IX-002 | E | 3C spawn_parallel sequential stub is upgraded to true parallel only by 3D. | P3C-E008, P3C-E024 |
| PHASE-008/IX-003 | E | 3C communicate in-process stub becomes IPC in 4A; same AgentHandle abstraction. | P3C-E011, P3C-E024 |
| PHASE-008/IX-004 | E | 3C ReplanRequest MAJOR human gate and MINOR PLANNING fast-track compose with 3A FSM and auditor. | P3C-E012, P3C-E031 |
| PHASE-008/IX-005 | E | 3C ReasoningTraceEntry and checkpoint events depend on 3B SSB single-writer append path. | P3C-E005, P3C-E032, P3C-E037 |
| PHASE-008/IX-006 | E | 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400. | P3C-E007, P3C-E026; IDX-E019 |
| PHASE-008/IX-007 | E | 3C Designer role produces PipelineSpec later consumed by 4D/4E/4F chain. | P3C-E023 |
| PHASE-008/IX-008 | E | 3E budget is recomputed inside 3D BSP superstep between prepare_next_tasks and execute_tasks; consumes channel kinds from 3D. | P3E-E016, P3E-E017 |
| PHASE-008/IX-009 | E | 3E Terminal AUTONOMOUS SPLIT_TASK is owned by 4G; INTERACTIVE escalate is immediate. | P3E-E022 |
| PHASE-008/IX-010 | E | 3E OutputReserve and floors protect model generation headroom used by all later agent invocations. | P3E-E013, P3E-E014, P3E-E025 |
| PHASE-008/IX-011 | E | 4B tools run only through 4A IPC ToolRequest/ToolResult; isolation/sanitization owned by 4A, execution by 4B. | P4B-E001, P4B-E004, P4B-E021 |
| PHASE-008/IX-012 | E | 4B IntegrationTestRunner blocked on 4E DecompositionTree types and matches 4E integration time budgets. | P4B-E003, P4B-E028 |
| PHASE-008/IX-013 | E | 4B ShortCoder post-processes executor output before auditor; placement also referenced from 4C hooks. | P4B-E017 |
| PHASE-008/IX-014 | E | 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral. | P4B-E018; P3C-E052 |
| PHASE-008/IX-015 | E | 4B write tools honor SSB file locks (orchestrator concurrency invariant). | P4B-E008 |
| PHASE-008/IX-016 | E | 4F reuses 2D FeasibilityOutcome and 4A Tier-C agent process with restricted AgentDispatchConfig. | P4F-E010, P4F-E016, P4F-E018 |
| PHASE-008/IX-017 | E | 4F skip path PLANNING→TESTING_FIRST and novel_steps marking come from 4E structural validation. | P4F-E011, P4F-E012 |
| PHASE-008/IX-018 | E | 4F decision Escalate vs Replan depends on execution mode introduced by 3A/4G. | P4F-E009 |
| PHASE-008/IX-019 | E | 4F SSB branch tags compose with 3B SSB and 3D branch semantics (tag=branch:…). | P4F-E020 |
| PHASE-008/IX-020 | E | 4F baby-step feasibility requires 4E DecompositionTree/BabyStepSpec and runs before concept execution in 4G. | P4F-E029–P4F-E032 |
| PHASE-008/IX-021 | E | 4J event bridge writes Empire telemetry into 3B SSB for GRC grounding. | P4J-E016, P4J-E032 |
| PHASE-008/IX-022 | E | 4J ElaiNeuron capability path reuses 4A IPC proxy + ToolFilter trust boundary. | P4J-E017, P4J-E019 |
| PHASE-008/IX-023 | E | 4J repair pipelines are generated by 4D pipeline factory and validated with 4H TestOutcomeReport. | P4J-E018, P4J-E025 |
| PHASE-008/IX-024 | E | 4J autonomous unattended Empire operation depends on 4G autonomous mode. | P4J-E001 |
| PHASE-008/IX-025 | E | Index 2C role budgets are the same numbers 3C adversarial expects at spawn. | IDX-E019; P3C-E026 |
| PHASE-008/IX-026 | E | Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase. | IDX-E023; P3C-E012 |
| PHASE-008/IX-027 | E | Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs). | IDX-E011; P3E-E003 |
| PHASE-008/IX-028 | E | Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts. | P3C-E014; P3E-E034; P4B-E034; P4F-E024 |
| PHASE-008/IX-029 | E | RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J. | P4B-E032; P4F-E035; P4J-E037 |
| PHASE-008/IX-030 | E | blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A). | P3C-E053; P4F-E005 |
| PHASE-008/IX-031 | I | 4J Tauri/Svelte Empire panel will eventually sit beside 4K dashboard surfaces but 4K is out of this chunk. | P4J-E033 |
| PHASE-008/IX-032 | E | 3C SubstituteModel is type-ready but gated until 4G; 4F/4B do not enable it. | P3C-E047 |

# Namespaced ambiguity/conflict index (all AC)

| Register ID | Kind | Issue | Atoms |
|---|---|---|---|
| PHASE-001/AC-001 | C | TokenUsage field set differs design (4 fields) vs 0.2c shim (2 fields). | P0-C001 |
| PHASE-001/AC-002 | A | prompt.rs REPLACE vs STRIP open. | P0-A001 |
| PHASE-001/AC-003 | A | remote.rs DEFERRED vs STRIP open. | P0-A002 |
| PHASE-001/AC-004 | A | sandbox whole-file vs types-only open. | P0-A003 |
| PHASE-001/AC-005 | C | 1C model path onnx/ vs Step 5 per-model subdirs. | P1C-C001 |
| PHASE-001/AC-006 | C | SPLADE deferral 1G (source) vs 1H (MASTER summary). | P1C-C002 |
| PHASE-001/AC-007 | C | AUDITING recovery transitions need orchestrator.txt SPEC_UPDATE. | P3A-C001 |
| PHASE-001/AC-008 | A | UNGROUNDED COMPLETE/IDLE tool policies and gate traits. | P3A-E016, P3A-I001 |
| PHASE-001/AC-009 | A | REDIRECT invalid-target and full risk matrix incomplete. | P3A-A001–P3A-A002 |
| PHASE-001/AC-010 | A | 3J-BASELINE fixture prompts and compression tolerance unspecified. | P3JB-A001–P3JB-A002 |
| PHASE-001/AC-011 | A | 4K submit_task path, projects precedence, diff modal vs replace. | P4K-A001–P4K-A003 |
| PHASE-001/AC-012 | A | 4M adapter alias, panel placement, name list, hybrid/auto detail. | P4M-A001–P4M-A003 |
| PHASE-002/AC-001 | conflict | R20 ID collision: PageRank density vs phase-0 entropy quarantine; renumber out of scope for 1D. | P1D-E003 |
| PHASE-002/AC-002 | conflict | Anti-spiral 3× vs same_step_count>3; 4G pins 3rd-failure interpretation. | P4G-C001 |
| PHASE-002/AC-003 | qualification | HIGH-risk human pause vs auditor tool review are different layers; both remain binding. | P4G-C002 |
| PHASE-002/AC-004 | ambiguity | 1A partial parse on syntax errors lacks completeness floor. | P1A-A003 |
| PHASE-002/AC-005 | ambiguity | 1D “mentioned in task” personalization matching algorithm unspecified. | P1D-A002 |
| PHASE-002/AC-006 | ambiguity | 3D evidence_gate_passes sufficiency predicate incomplete in phase file. | P3D-A003 |
| PHASE-002/AC-007 | dependency | 3D ChannelRegistry + memory_monitoring + Topic(false) + checkpoint placement need orchestrator.txt amendments before/with implement. | P3D-E007, P3D-A002 |
| PHASE-002/AC-008 | ambiguity | 3G plan_phases SQL shorthand vs SSB-only interim; view amendment needs user confirmation. | P3G-A001 |
| PHASE-002/AC-009 | ambiguity | 3G coherence “text reference” vs structured implemented_contracts dual wording. | P3G-A003 |
| PHASE-002/AC-010 | ambiguity | 4DOR query normalization for blake3 key undefined. | P4DOR-A001 |
| PHASE-002/AC-011 | ambiguity | 4G SessionState ownership and definition still open; execution_mode attachment site TBD. | P4G-A001 |
| PHASE-002/AC-012 | dependency | 4G requires new APIs: can_fit_llm, AgentPool::get_role, SSB escalation type — all require post-approval spec propagation. | P4G-E011, P4G-E013, P4G-E014 |
| PHASE-002/AC-013 | ambiguity | parallel_dispatch default/TOML key for persona fan-out unspecified. | P4G-A003 |
| PHASE-002/AC-014 | evidence | 4G runtime UX flags appear in MASTER summary more explicitly than in phase-4G body; do not over-claim source atoms for --local/--hybrid without addendum. | P4G-I001 |
| PHASE-002/AC-015 | qualification | blake2 (1A file hash) vs blake3 (4DOR/4G) is intentional layer split, not a single-stack rule — document when consolidating hash policy. | IX-015 |
| PHASE-003/AC-001 | conflict/schema | tantivy_v21 label vs INDEX_FORMAT_VERSION u32. | P1B-C001, P1B-A001 |
| PHASE-003/AC-002 | ambiguity/config | Exact tantivy/ort pin versions deferred to execution decision docs. | P1B-A001, P1G-A003 |
| PHASE-003/AC-003 | conflict/fallback | L-6 reranker tier needs amendment vs skip-only spec. | P1G-C001, P1G-E011 |
| PHASE-003/AC-004 | ambiguity/measure | 1G skip threshold and 50 ms pass/fail after levers. | P1G-A001, P1G-E018 |
| PHASE-003/AC-005 | conflict/schema | 3H per-message snapshot_context vs orchestrator blob schema. | P3H-C001, P3H-A001 |
| PHASE-003/AC-006 | ambiguity/event | branch_abandon new type vs checkpoint payload. | P3H-A002, PIDX-C001 |
| PHASE-003/AC-007 | ambiguity/data | SSB merge cleanup logical vs physical wording. | P3H-A003 |
| PHASE-003/AC-008 | ambiguity/merge | File-overlap Conflict variant ungrounded. | P3H-A004 |
| PHASE-003/AC-009 | ambiguity/crypto | blake3 stub hash algorithm choice. | P3H-A005 |
| PHASE-003/AC-010 | ambiguity/surface | SurgicalDrawer full UI fields; step-cap merge notes. | P4DS-A001, P4DS-A002 |
| PHASE-003/AC-011 | ambiguity/enum | CycleDetected vs CyclicDependency naming. | P4E-A001 |
| PHASE-003/AC-012 | ambiguity/repair | Weakest-dependency definition for REORDER. | P4E-A003 |
| PHASE-003/AC-013 | ambiguity/order | Index 3K listed before 3J vs HyDE dependency on 3J. | PIDX-A001 |
| PHASE-003/AC-014 | ambiguity/index | Index omits C-30 surgical CLI/drawer present in 4D-SURGICAL. | PIDX-I001, P4DS-E002 |
| PHASE-003/AC-015 | ambiguity/dep | 3E total_allocated_tokens exposure for TrimConfig. | P3H-A006 |
| PHASE-004/AC-001 | conflict | 1E "sub-second" goal vs 2s hard interactive budget. | P1E-C001 |
| PHASE-004/AC-002 | ambiguity | 1E heuristic edge-score aggregation into 0–10 before phi not fully formal. | P1E-A002 |
| PHASE-004/AC-003 | ambiguity | 1E SPLADE in knowledge order vs no 1E splade stage. | P1E-A001 |
| PHASE-004/AC-004 | ambiguity | 2B claim regex / Uncertain path-vs-symbol heuristics incomplete. | P2B-A001, P2B-A002 |
| PHASE-004/AC-005 | conflict | reasoning.txt 4-field AgentGrcConfig vs 2C 8-field requirement + GroundingLevel order fix in 2A. | P2C-C001, P2C-E013, P2C-E020 |
| PHASE-004/AC-006 | ambiguity | FINDING: prefix vs future structured FindingReport. | P2C-A001 |
| PHASE-004/AC-007 | ambiguity | PI_T2_CIRCUIT_BREAKER_LINES exact value; full conventions SQL columns. | P3L-A001, P3L-A002 |
| PHASE-004/AC-008 | conflict | 4C adversarial "prefer ruff library" vs decided subprocess+breaker. | P4C-C001 |
| PHASE-004/AC-009 | ambiguity | T2 deny-list / lazy-code pattern catalog incomplete in phase text. | P4C-A001 |
| PHASE-004/AC-010 | conflict | architecture-overview media models stale vs 4I Mode A/B authority. | P4I-C001 |
| PHASE-004/AC-011 | ambiguity | QualitySpec threshold field names and max_iterations override policy. | P4I-A001, P4I-A002 |
| PHASE-004/AC-012 | qualification | MASTER marks phase-3L as unindexed inventory item; deep extract is authoritative for 3L atoms. | P3L-E001 |
| PHASE-005/AC-001 | conflict | 1F prerequisite Phase 0-independent vs 1C dependency graph. | P1F-C001 |
| PHASE-005/AC-002 | conflict | 1F full eval suite vs RESOLVED skip 1F.3-1F.5. | P1F-C002 |
| PHASE-005/AC-003 | conflict | 2A "grounding every step" summary line vs required subset. | P2A-C001 |
| PHASE-005/AC-004 | ambiguity | 2A T11 hard Err vs warning for missing scope. | P2A-A002 |
| PHASE-005/AC-005 | ambiguity | 2A full allowed_next table incomplete. | P2A-A001 |
| PHASE-005/AC-006 | conflict | 3F rules→core-only crate map vs AstEvaluator knowledge coupling. | P3F-C001 |
| PHASE-005/AC-007 | ambiguity | 3F evaluate return ALLOW\|BLOCK\|WARN vs ESCALATE enum arm. | P3F-A003 |
| PHASE-005/AC-008 | conflict | 3J step numbering vs MASTER summary bundles. | P3J-C001 |
| PHASE-005/AC-009 | conflict | 3J.14 31B in intro vs 3J.15 body renumber + E2E as 3J.14. | P3J-C002 |
| PHASE-005/AC-010 | ambiguity | DoLa full-logit access method still VERIFY. | P3J-A001 |
| PHASE-005/AC-011 | ambiguity | hybrid/auto BackendMode vs session removal of dead variants. | P3J-A003 |
| PHASE-005/AC-012 | dependency | 3J.COMPOSE/3J.LOCAL referenced but not defined in this file. | P3J-I001 |
| PHASE-005/AC-013 | qualification | MASTER can_fit_onnx for 3J.6 vs source can_colocate. | P3J-I002 |
| PHASE-005/AC-014 | ambiguity | SCHEMA_REQUIRED provider-tier escalation is MASTER-only, not 3F body. | P3F-I001 |
| PHASE-005/AC-015 | ambiguity | 3F.5 NaN/Inf metric guard appears in MASTER not phase-3F body. | MASTER:L314 vs P3F-E024 |
| PHASE-006/AC-001 | ambiguity | 1H 0.3 vocab threshold ungrounded; 0.2–0.4 band may still measure recall. | P1H-A001 |
| PHASE-006/AC-002 | ambiguity | 1H Candidate B missing path skips full A/B comparison. | P1H-A002 |
| PHASE-006/AC-003 | ambiguity/dependency | 3I custom SQLite VFS necessity vs madsim hooks unresolved until spike. | P3I-A001 |
| PHASE-006/AC-004 | ambiguity | 3I Windows madsim availability and manual scheduler equivalence open until 3I.1. | P3I-A002 |
| PHASE-006/AC-005 | ambiguity/config | 3K synonym lexicon and AUTO keyword lists incomplete. | P3K-A001–P3K-A002 |
| PHASE-006/AC-006 | ambiguity/measurement | 4A IPC/spawn/memory/overhead SLOs are ungrounded until 4A.6. | P4A-A001 |
| PHASE-006/AC-007 | ambiguity | 4A WARN 1–9 bodies not expanded in phase file despite required test mapping. | P4A-A002 |
| PHASE-006/AC-008 | ambiguity/design | 4H archive cap 50 and MSI thresholds are local design choices, not paper transfers. | P4H-A001–P4H-A002 |
| PHASE-006/AC-009 | ambiguity | 4H source-tier auto-promote thresholds (5 hits, 0.8 verified ratio) lack broader tier policy. | P4H-A003 |
| PHASE-006/AC-010 | conflict | DGM-style code self-mod vs 4H config/data-only must not be flattened. | P4H-C001 |
| PHASE-007/AC-001 | C | 1I requires adding missing 1G→1I edge to implementation-guide dep graph before implement. | P1I-C001, P1I-E002 |
| PHASE-007/AC-002 | A | 1I permanent/transient error enum variants incomplete; HALF_OPEN probe concurrency unspecified. | P1I-A001–P1I-A002 |
| PHASE-007/AC-003 | A | 1I session-end p50/p99 SSB aggregation payload shape unnamed. | P1I-A003 |
| PHASE-007/AC-004 | C | 2D design methods web_search/grep/api_registry vs SourceMatch-only type until 4F. | P2D-C001, P2D-A001 |
| PHASE-007/AC-005 | A | 2D T4 uses stored `.passed` field vs computed method design. | P2D-A002 |
| PHASE-007/AC-006 | A | 2D multi-word match threshold not a named constant. | P2D-A003 |
| PHASE-007/AC-007 | C | 3B SsbPayload "15 variants" wording vs 17 action_types. | P3B-C001, P3B-A001 |
| PHASE-007/AC-008 | A | 3B CURRENT_VERSION evolution policy beyond v1 not specified. | P3B-A002 |
| PHASE-007/AC-009 | C | 4D research_step / research_decompose_level / web_research_step absent from 3B 17-type CHECK. | P4D-C001, P4D-A001 |
| PHASE-007/AC-010 | A | 4D Interactive present_options concrete UI surface open. | P4D-A003, P4D-I001 |
| PHASE-007/AC-011 | A | 4D several VRAM context_ceiling values projected not measured. | P4D-A002 |
| PHASE-007/AC-012 | A | pre-phase-0 full cfg-gated test inventory not pre-enumerated. | PP0-A001 |
| PHASE-007/AC-013 | A | pre-phase-0 HookRunner /bin/sh hardcode status left as audit. | PP0-A002 |
| PHASE-008/AC-001 | ambiguity | 3C tool_results rolling window always-on vs only-if-slow. | P3C-A001 |
| PHASE-008/AC-002 | ambiguity | 3C SSB CHECK migration mechanism and spiral "last N" size. | P3C-A002, P3C-A003 |
| PHASE-008/AC-003 | ambiguity | 3E attempt-1 overlap threshold "e.g. 0.3" not fixed default. | P3E-I001, P3E-A001 |
| PHASE-008/AC-004 | status/conflict | 4B DONE status vs remaining measure/integration body. | P4B-C001, P4B-E002-E003 |
| PHASE-008/AC-005 | ambiguity | 4B deterministic commit message template unspecified. | P4B-A002 |
| PHASE-008/AC-006 | conflict | 4F FeasibilityProbe vs FeasibilityProber naming and sysinfo vs shell-out RAM strategy. | P4F-C001, P4F-C002 |
| PHASE-008/AC-007 | conflict | 4F FEASIBILITY tool set: stub_write+bash only vs +web search. | P4F-A001 |
| PHASE-008/AC-008 | conflict | 4J 5 SSB empire_* variants vs body 'empire_event' singular. | P4J-C001 |
| PHASE-008/AC-009 | conflict | 4J cargo package name elai_empire vs orchestrator --features empire. | P4J-C002 |
| PHASE-008/AC-010 | ambiguity | 4J Tauri/Svelte surface names not enumerated. | P4J-I001 |
| PHASE-008/AC-011 | ambiguity | Index pre0–2D omits completion and step-level gates. | IDX-I001, IDX-A001 |

# Namespaced summary-delta index (all SD)

| Delta ID | Change | Summary anchor | Source atoms |
|---|---|---|---|
| PHASE-001/SD-P0-001 | **ADD** full risk register R17–R25 with LOC/import facts and stale comparison-doc warning. | 01-phase:L24-L42 | P0-E004–P0-E012, P0-E068 |
| PHASE-001/SD-P0-002 | **ADD** workspace path migration (rust/→crates/), archive semantics, strip-before-restructure rationale. | 01-phase:L24-L33 | P0-E013–P0-E014 |
| PHASE-001/SD-P0-003 | **ADD** ONNX path, blake2 setup, DEGRADED BM25_ONLY, models/** hidden_paths. | 01-phase:L24-L42 | P0-E015–P0-E017 |
| PHASE-001/SD-P0-004 | **ADD** ordered strip sequence, KEEP/STRIP slash lists, TokenUsage shim vs core fields. | 01-phase:L29-L32 | P0-E024–P0-E029, P0-C001 |
| PHASE-001/SD-P0-005 | **ADD** module mapping LOCs, tools 2000-LOC cap, cli <500 soft, nine crates, contracts TDD red phase. | 01-phase:L31-L39 | P0-E032–P0-E048, P0-E066 |
| PHASE-001/SD-P0-006 | **ADD** LlmProvider streaming complete, Ollama endpoints, allowlist sanitization, hidden_paths inventory, smoke skip. | 01-phase:L36-L44 | P0-E049–P0-E059 |
| PHASE-001/SD-P0-007 | **ADD** ClockProvider/EntropySource shapes, RuntimeContext, CI exclusions, ELAI_SEED=42 dual-run proof. | 01-phase:L40-L41 | P0-E060–P0-E065 |
| PHASE-001/SD-P0-008 | **FLAG** open prompt/remote/sandbox extraction choices omitted from summary. | 01-phase:L24-L42 | P0-A001–P0-A004 |
| PHASE-001/SD-P1C-001 | **ADD** R22/R26/R27/R29 mitigations and HYBRID→SPARSE_ONLY→BM25_ONLY cascade. | 01-phase:L69-L83 | P1C-E003–P1C-E006, P1C-E012–P1C-E013 |
| PHASE-001/SD-P1C-002 | **CORRECT** SPLADE deferral: original 1G vs summary 1H. | 01-phase:L74-L74 | P1C-E007, P1C-C002 |
| PHASE-001/SD-P1C-003 | **ADD** TurboQuant grounded params and ban on 1C quantization. | 01-phase:L83-L83 | P1C-E015, P1C-E034 |
| PHASE-001/SD-P1C-004 | **ADD** DualVector/OrtSession/UsearchIndex APIs, 512 truncation, filtered_search rebuild, RRF k=60 fixed. | 01-phase:L75-L81 | P1C-E017–P1C-E022, P1C-E030, P1C-E035 |
| PHASE-001/SD-P1C-005 | **ADD** ≥22 tests, physical DB isolation, QueryTrace, knowledge_degradation triggers, write-before-query race rule. | 01-phase:L79-L81 | P1C-E024–P1C-E025, P1C-E032–P1C-E036 |
| PHASE-001/SD-P1C-006 | **ADD** Step 3.5 tensor gate, Step 5 fields, cache_hit BLOCK <90%, path naming tension. | 01-phase:L80-L82 | P1C-E027, P1C-E037–P1C-E039, P1C-C001 |
| PHASE-001/SD-P1C-007 | **QUALIFY** summary fastembed pin: original forbids adding fastembed with ort. | 01-phase:L74-L74 | P1C-E014 |
| PHASE-001/SD-P3A-001 | **ADD** full transition matrix including CONFIGURING, FEASIBILITY revise, ReplanRequest, SKILL.md AUDITING arms + SPEC_UPDATE. | 01-phase:L233-L244 | P3A-E005–P3A-E008, P3A-C001 |
| PHASE-001/SD-P3A-002 | **ADD** transition payload, token_budget_remaining, msgpack path, TransitionConflict, typed errors. | 01-phase:L237-L237 | P3A-E009–P3A-E014 |
| PHASE-001/SD-P3A-003 | **ADD** full tool-filter table including FEASIBILITY stub-write and UNGROUNDED COMPLETE/IDLE. | 01-phase:L238-L239 | P3A-E015–P3A-E017 |
| PHASE-001/SD-P3A-004 | **ADD** DAIRA max-3, EvidenceError shape, DEGRADED-allows-with-warning, FEASIBILITY ungated. | 01-phase:L240-L242 | P3A-E019–P3A-E023 |
| PHASE-001/SD-P3A-005 | **ADD** structured EvidenceSummary, trait mockability, no-timeout default, risk examples + overrides. | 01-phase:L243-L244 | P3A-E024–P3A-E030 |
| PHASE-001/SD-P3A-006 | **ADD** 81-pair tests, latency SLOs, SSB phase_transition non-deliverable note. | 01-phase:L233-L244 | P3A-E031–P3A-E033 |
| PHASE-001/SD-P3JB-001 | **ADD entire file**: MASTER does not index phase-3J-BASELINE.txt. | 01-phase:L360-L375, L655 | P3JB-E001–P3JB-E019 |
| PHASE-001/SD-P3JB-002 | **ADD** standalone stdlib harness, 8 tiers, 7 metrics, REPAIR_CONTEXT ±15%, JSON-schema 20-prompt, exit 0/1/2. | 01-phase:L655 | P3JB-E002–P3JB-E015 |
| PHASE-001/SD-P3JB-003 | **ADD** non-overwrite run_benchmark.py, optional CI bench-capability, ShortCoder ban + output_token_ratio. | 01-phase:L655 | P3JB-E004, P3JB-E014, P3JB-E017 |
| PHASE-001/SD-P4K-001 | **CORRECT/EXPAND** summary phase ribbon/commit history: MVP is project selector + task + plan tree + activity feed + diff viewer. | 01-phase:L528-L528 | P4K-E029–P4K-E053 |
| PHASE-001/SD-P4K-002 | **ADD** full visual tokens, window geometry, identifier, Vite :1420, elai --dashboard single-instance focus. | 01-phase:L522-L528 | P4K-E006, P4K-E018, P4K-E026–P4K-E027 |
| PHASE-001/SD-P4K-003 | **ADD** distinct Tauri commands including projects/templates/submit_task/get_action_detail. | 01-phase:L527-L527 | P4K-E019–P4K-E024, P4K-E033–P4K-E035, P4K-E053 |
| PHASE-001/SD-P4K-004 | **ADD** intervention options, per-hunk controls, multi-file nav, preview open, 500-entry cap, in-flight poll skip, 200-char truncate. | 01-phase:L525-L526 | P4K-E013–P4K-E014, P4K-E040–P4K-E055 |
| PHASE-001/SD-P4K-005 | **ADD** project name validation, min 5-char task, projects.json corrupt empty-list, post-MVP deferred list. | 01-phase:L528-L529 | P4K-E033, P4K-E036, P4K-E057 |
| PHASE-001/SD-P4K-006 | **ADD** WebView2 bootstrapper, <100KB JS, typed schema drift compile-fail, graceful open-fail dialog. | 01-phase:L529-L529 | P4K-E007–P4K-E010, P4K-E028 |
| PHASE-001/SD-P4M-001 | **ADD** summary stops at 4M.1; original continues 4M.2–4M.6 adapters/hybrid/CLI/dashboard/metrics/migration. | 01-phase:L531-L541, L653 | P4M-E014–P4M-E031 |
| PHASE-001/SD-P4M-002 | **ADD** PermissionMode defaults, ResolveBackendError, capabilities fields, factory timeouts. | 01-phase:L537-L539 | P4M-E006–P4M-E009 |
| PHASE-001/SD-P4M-003 | **ADD** default per-role map, runtime vs model fallback order, ESCALATE/HALT, per-session state. | 01-phase:L539-L539 | P4M-E021–P4M-E027 |
| PHASE-001/SD-P4M-004 | **ADD** /runtime show|set|fallback and RuntimePanel Test-backend canary. | 01-phase:L531-L541 | P4M-E023–P4M-E028 |
| PHASE-001/SD-P4M-005 | **ADD** Tier C mandate, stdout sanitization, missing-binary Unavailable, exit_code fail-closed, RuntimeClient wiring. | 01-phase:L531-L541 | P4M-E018–P4M-E019, P4M-E031 |
| PHASE-001/SD-P4M-006 | **ADD** [backend]→[runtime] deprecation, phase_4M.toml metrics, ouroboros grounding distinctions, R1–R4. | 01-phase:L533-L541 | P4M-E015, P4M-E029–P4M-E033 |
| PHASE-002/SD-1A-001 | **ADD** full COMPLIANCE SCOPE split of knowledge.txt Sec 8 items 1+5 vs deferred 2–4 and orphan-invariant block until 1B. | MASTER:L44-L55 | P1A-E009, P1A-E010, P1A-E043 |
| PHASE-002/SD-1A-002 | **ADD** readiness split dkg_readiness vs full KnowledgeEngine.readiness_check (Phase 1I). | MASTER:L48-L51 | P1A-E016, P1A-E017 |
| PHASE-002/SD-1A-003 | **ADD** complete schema tables dkg_files/dkg_meta, directed edge semantics, WIRING_CONTRACT symbols, index build SSB symbol_graph_updated, and 12 metric keys. | MASTER:L50-L55 | P1A-E022–P1A-E027, P1A-E038–P1A-E041 |
| PHASE-002/SD-1A-004 | **ADD** TDD red gate (≥20 tests, ≥10 fail), object-safety LanguageGrammar, and dynamic KNOW_PATH resolution. | MASTER:L48-L53 | P1A-E002, P1A-E011, P1A-E034 |
| PHASE-002/SD-1A-005 | **FLAG** ungrounded <15s build claim and designed-not-spec SQLite schema. | MASTER:L44-L55 | P1A-E006, P1A-E007 |
| PHASE-002/SD-1D-001 | **ADD** full T1–T18/P1–P3/A1–A3 catalog and make_snapshot helper requirements. | MASTER:L93-L93 | P1D-E010–P1D-E014 |
| PHASE-002/SD-1D-002 | **ADD** exact 13-item special-file list and config-driven deferral; qualify MASTER “such as” wording. | MASTER:L96-L96 | P1D-E016, P1D-A001 |
| PHASE-002/SD-1D-003 | **ADD** R20 ID collision note and Layer 3.5 vs Layer 4 correction. | MASTER:L85-L97 | P1D-E003, P1D-C001 |
| PHASE-002/SD-1D-004 | **ADD** aider not-adopted items (specificity/private/generic penalties) and 7-metric phase_1D.toml fields. | MASTER:L88-L97 | P1D-E019, P1D-E020 |
| PHASE-002/SD-1D-005 | **ADD** zero-budget T13, NaN/Inf clamp, and initial binary-search guess max_tokens/25. | MASTER:L94-L96 | P1D-E015, P1D-E017 |
| PHASE-002/SD-3D-001 | **ADD** four pre-implement orchestrator.txt amendments and corrected loop rationale (checkpoint after gate). | MASTER:L284-L288 | P3D-E007, P3D-E014 |
| PHASE-002/SD-3D-002 | **ADD** ephemeral clear SSB replay algorithm and Topic(false) two-phase snapshot+clear definition (spec gap). | MASTER:L281-L282, L287 | P3D-E009, P3D-E010, P3D-E018 |
| PHASE-002/SD-3D-003 | **ADD** DeepSizeOf/size_of_val ban, 10MB/2× force-clear, and isolation_metric separation from memory_monitoring. | MASTER:L283-L287 | P3D-E004, P3D-E017, P3D-E022 |
| PHASE-002/SD-3D-004 | **ADD** wall-time concurrency proof, VRAM max_concurrent formula, and simulation-only byte-identical determinism caveat for live LLMs. | MASTER:L277-L288 | P3D-E003, P3D-E019 |
| PHASE-002/SD-3D-005 | **ADD** baseline measurement section as non-exit UNGROUNDED estimates. | MASTER:L276-L288 | P3D-E020 |
| PHASE-002/SD-3G-001 | **ADD** 3E dependency correction for 3G.4 and partial-parallel recommendation. | MASTER:L321-L332 | P3G-E001, P3G-E002 |
| PHASE-002/SD-3G-002 | **ADD** visual_check/output_type media adapter rules and ClockProvider timestamp ungrounded fields. | MASTER:L325-L325 | P3G-E004, P3G-E005 |
| PHASE-002/SD-3G-003 | **ADD** 3600 safety margin, SkeletonBudgetExceeded, ContractConflict, and empty declared_interfaces fail-closed. | MASTER:L327-L330 | P3G-E008, P3G-E017, P3G-E023 |
| PHASE-002/SD-3G-004 | **ADD** split_phase preserve_contracts algorithm details and u32 checked_add overflow Err. | MASTER:L331-L331 | P3G-E014, P3G-E018 |
| PHASE-002/SD-3G-005 | **ADD** 100-run determinism with EntropyProvider pipeline_id and ban wall clocks in plan_skeleton. | MASTER:L332-L332 | P3G-E019 |
| PHASE-002/SD-4DOR-001 | **ADD** entire Oracle cache feature set: MASTER lists phase-4D-ORACLE only as unindexed file to inspect, without extracting atoms. | MASTER:L655-L655, L714-L714 | P4DOR-E001–P4DOR-E017 |
| PHASE-002/SD-4DOR-002 | **ADD** TTL/confidence mapping, LRU default 128, /research+/ask reachability, blake3 ban, C-30 SCHEMA 62 events. | MASTER:L655 | P4DOR-E006–P4DOR-E015 |
| PHASE-002/SD-4DOR-003 | **ADD** LiveCli lazy Option init and T4Rejected early-return stub_ban. | MASTER:L655 | P4DOR-E011, P4DOR-E014 |
| PHASE-002/SD-4DOR-004 | **FLAG** query normalization and expired-entry lazy eviction ambiguities absent from any summary. | MASTER:L655 | P4DOR-A001, P4DOR-A002 |
| PHASE-002/SD-4G-001 | **ADD** full EvidenceCheck/EvidenceVerdict types and TESTING_FIRST cargo heuristic caveats. | MASTER:L471-L471 | P4G-E005, P4G-E006, P4G-A002 |
| PHASE-002/SD-4G-002 | **ADD** 28+8 test catalog, InterventionCard exhaustion UI, and mode-switch TOCTOU RwLock rules. | MASTER:L476-L476 | P4G-E015, P4G-E028, P4G-E035 |
| PHASE-002/SD-4G-003 | **ADD** complete 4G.9 stagnation/persona/watchdog/goal-drift port (thresholds, SSB types, affinity, fingerprint, Jaccard bands). | MASTER:L477-L477 (compressed tree only) | P4G-E021–P4G-E027, P4G-E034 |
| PHASE-002/SD-4G-004 | **ADD** shipped addendum: reasoning-exhaustion 7th auto-advance check and CloudSpendTracker budget policy (default $5). | MASTER:L467-L478 | P4G-E031, P4G-E032 |
| PHASE-002/SD-4G-005 | **ADD** required new APIs can_fit_llm, get_role, SSB escalation type and SessionState open. | MASTER:L475-L475 | P4G-E011, P4G-E013, P4G-E014, P4G-A001 |
| PHASE-002/SD-4G-006 | **QUALIFY** MASTER runtime UX (--local/--hybrid, /model, backend config): more explicit in summary than phase body; keep as orientation/I. | MASTER:L478-L478 | P4G-I001, AC-014 |
| PHASE-002/SD-4G-007 | **ADD** anti-spiral pin L180 and HIGH-risk dual-path clarification vs auditor tool review. | MASTER:L472-L472 | P4G-C001, P4G-C002 |
| PHASE-003/SD-1B-001 | **ADD** full API surface IndexStats/BM25Hit snippet policy, single-writer batch lock, and explicit project_id rule. | MASTER:L57-L67 | P1B-E006-E013 |
| PHASE-003/SD-1B-002 | **ADD** WIRING_CONTRACT CLI `index build` + SSB `index_checkpoint` + call sites. | MASTER:L57-L67 | P1B-E018-E019 |
| PHASE-003/SD-1B-003 | **ADD** environment-aware CI 50 ms vs dev 10 ms gates and warm-up measurement. | MASTER:L67 | P1B-E017 |
| PHASE-003/SD-1B-004 | **ADD/CORRECT** tantivy_v21 vs INDEX_FORMAT_VERSION and DkgNode-vs-BM25Chunk ownership. | MASTER:L61-L63 | P1B-E004-E006, P1B-C001 |
| PHASE-003/SD-1B-005 | **ADD** 1I scoping-out of readiness/breaker/versioning from 1B. | MASTER:L67 | P1B-E020 |
| PHASE-003/SD-1G-001 | **ADD** full RerankerConfig field set, mpsc batching design, and oneshot offset split. | MASTER:L123-L133 | P1G-E004-E008 |
| PHASE-003/SD-1G-002 | **ADD** L-6 amendment path, cold-start SLA exception, and canary DEGRADED semantics. | MASTER:L129-L132 | P1G-E010-E014, P1G-C001 |
| PHASE-003/SD-1G-003 | **ADD** INT8 trigger p50>30 ms / NDCG delta <0.5 and quantization script path. | MASTER:L127, L133 | P1G-E015 |
| PHASE-003/SD-1G-004 | **ADD** complete metrics list including queue wait p99 and amendment-on-fail-50ms rule. | MASTER:L133 | P1G-E017-E018 |
| PHASE-003/SD-1G-005 | **ADD** 1F DROP independence note (MASTER has 1F section; 1G body reconfirms). | MASTER:L114-L121 | P1G-E001 |
| PHASE-003/SD-3H-001 | **ADD** full SQL column/index set, types, and DELETE-journal/FK/integrity_check safety. | MASTER:L334-L344 | P3H-E003-E008 |
| PHASE-003/SD-3H-002 | **EXPAND** three amendments (schema/event/cleanup) and Conflict/blake3 open items beyond MASTER [Q] one-liner. | MASTER:L337 | P3H-A001-A005, P3H-C001 |
| PHASE-003/SD-3H-003 | **ADD** recovery APIs, typed StubRecoveryFailed, and 100-run determinism/measure targets. | MASTER:L343-L344 | P3H-E014-E017 |
| PHASE-003/SD-3H-004 | **ADD** branch naming rules, close_branch, and immutable trunk-extension merge semantics detail. | MASTER:L342 | P3H-E012 |
| PHASE-003/SD-4DS-001 | **ADD** entire C-30 surgical path: MASTER only flags file as unindexed; full CLI/SSB/drawer/LOC/complexity atoms are deep-only. | MASTER:L655 | P4DS-E001-E015 |
| PHASE-003/SD-4DS-002 | **ADD** intercept-before-research runner semantics and DkgQuerierAdapter stub ban. | MASTER:L655 | P4DS-E003, P4DS-E007, P4DS-E013 |
| PHASE-003/SD-4E-001 | **ADD** full type inventory, nine builder stubs, and 8 ValidationErrorKind list. | MASTER:L442-L455 | P4E-E003-E008 |
| PHASE-003/SD-4E-002 | **ADD** Phase Sizing formula constants, fixpoint hash escalate, REORDER-before-SUBSTITUTE, RuntimeClient wiring rule. | MASTER:L445-L449 | P4E-E007, P4E-E009-E010, P4E-E013 |
| PHASE-003/SD-4E-003 | **ADD** complete baby-step 8-stage cycle, BubbleUpPolicy defaults, SSB replan_request, msgpack recovery. | MASTER:L451-L455 | P4E-E014-E023 |
| PHASE-003/SD-4E-004 | **ADD** test catalog counts (≥30+10+33) and 4E.12 context p95/overhead gates. | MASTER:L450-L455 | P4E-E011, P4E-E024-E025 |
| PHASE-003/SD-4E-005 | **FLAG** enum name CycleDetected vs CyclicDependency and weakest-dep open. | MASTER:L446-L448 | P4E-A001, P4E-A003 |
| PHASE-003/SD-IDX-001 | **ADD** this file is a second index surface parallel to MASTER; captures 3J wire substeps, `/model`, backend modes, 4H MSI, Empire caps not always expanded in MASTER bullets. | MASTER:L334-L478 region | PIDX-E012-E013, PIDX-E021-E024 |
| PHASE-003/SD-IDX-002 | **QUALIFY** index is not authoritative over deep phase files when they conflict (e.g. surgical absent here). | MASTER:L1-L8 | PIDX-I001, PIDX-A001, AC-014 |
| PHASE-003/SD-IDX-003 | **ADD** 3K FAST/STANDARD/DEEP tier routing and 4B IntegrationTestRunner ladder as index-compressed features. | MASTER:L399, retrieval ladder | PIDX-E011, PIDX-E015 |
| PHASE-004/SD-1E-001 | **ADD** full risk set R30–R34 with mitigation numbers (N_seed, K_edge, density 0.01, max_loops, ±2 lines, 2s budget). | MASTER:L99-L112 | P1E-E007 |
| PHASE-004/SD-1E-002 | **ADD** sparse CatRagOverlay type fields, QueryPipeline/PipelineStage schema, and measure_1e five-query catalog Q1–Q5 + phase_1E.toml keys. | MASTER:L103-L112 | P1E-E009, P1E-E015, P1E-E028 |
| PHASE-004/SD-1E-003 | **ADD** deterministic 1E A-RAG fixed tool order vs deferred ReAct Phase 2+; stage failure fall-through; RRF rank-from-1. | MASTER:L111 | P1E-E025, P1E-E027 |
| PHASE-004/SD-1E-004 | **ADD** match-cap 2*N_seed and >50% DKG match warning; optional LLM edge scoring via breaker only. | MASTER:L107 | P1E-E020, P1E-E021 |
| PHASE-004/SD-1E-005 | **FLAG** sub-second vs 2s acceptance tension absent from summary compression. | MASTER:L100, L112 | P1E-C001 |
| PHASE-004/SD-2B-001 | **ADD** PlanGroundingResult verify_needed semantics (training-only subset of ungrounded) and empty-plan trivially grounded. | MASTER:L194 | P2B-E008, P2B-E020 |
| PHASE-004/SD-2B-002 | **ADD** complete prefix dispatch table including grep→FileCheck and tool→SourceCheck; 30s cargo test timeout; BashSandbox API. | MASTER:L198-L201 | P2B-E017-P2B-E019 |
| PHASE-004/SD-2B-003 | **ADD** full T1–T16 catalog, network/timing bans, tempfile cleanup, and dependency gate (no knowledge in cargo tree). | MASTER:L197, L196 | P2B-E015, P2B-E016, P2B-E013 |
| PHASE-004/SD-2B-004 | **ADD** Uncertain auto-resolution during think() enhancement of 2A; 4A wrap-without-signature-change constraint. | MASTER:L191, L200 | P2B-E004, P2B-E005 |
| PHASE-004/SD-2B-005 | **QUALIFY** summary network stub string vs source "not implemented in phase 2B" — behavioral typed error is the requirement. | MASTER:L200 | P2B-C001 |
| PHASE-004/SD-2C-001 | **ADD** complete T1–T15 test catalog and role budgets (executor 8/1200, auditor 10/1200, memory_writer 2/400, prompt_engineer 5/800, designer distinct). | MASTER:L209, L215 | P2C-E011, P2C-E018 |
| PHASE-004/SD-2C-002 | **ADD** whitelist formula `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})` and with_agent_config topic reset. | MASTER:L216 | P2C-E012, P2C-E014 |
| PHASE-004/SD-2C-003 | **ADD** FINDING_PREFIX contract, scope enumeration WARN path, and five validation rules D2 including csq_required. | MASTER:L211-L212 | P2C-E005, P2C-E007, P2C-E008 |
| PHASE-004/SD-2C-004 | **ADD** full SPEC GAPS list (8-field type, GrcError variants, PartialOrd order, rubber-stamp invariant, methods vs quality complementarity). | MASTER:L207-L218 | P2C-E020, P2C-C001 |
| PHASE-004/SD-2C-005 | **ADD** metrics memory <2KB and malformed-table fail vs missing-table default distinction. | MASTER:L218 | P2C-E016, P2C-E014 |
| PHASE-004/SD-3L-001 | **ADD** entire Phase 3L feature set: MASTER only notes phase-3L.txt as unindexed file to inspect (Q-line), without atom extraction. | MASTER:L655, L713 | P3L-E001–P3L-E021 |
| PHASE-004/SD-3L-002 | **ADD** Pi-prefixed hook naming, LazyLock D2, conventions_migration_v1 flat path, seed_rules API, profile.toml + /profile. | MASTER:L655 | P3L-E003–P3L-E008 |
| PHASE-004/SD-3L-003 | **ADD** telescope priorities 100/80/60, TELESCOPE_MAX_CHARS 1200, pi_telescope_contract five invariants, calibration gates 50/100 ms. | MASTER:L655 | P3L-E015–P3L-E017 |
| PHASE-004/SD-3L-004 | **ADD** four-language determinism matrix and CONVENTIONS prompt block budget 200 tokens. | MASTER:L655 | P3L-E010, P3L-E012 |
| PHASE-004/SD-3L-005 | **ADD** 3K predecessor correction (not Tier0 scaffold) and reuse of plugins HookVerdict without forking 4C tiers. | MASTER:L655 | P3L-E002, P3L-E003 |
| PHASE-004/SD-4C-001 | **ADD** complete 30+10 test catalogs, cross-session locks, SCHEMA_REQUIRED Escalate, ShortCoder AST-equivalent tests. | MASTER:L421 | P4C-E011, P4C-E012 |
| PHASE-004/SD-4C-002 | **ADD** PreCompact 0.85 handoff threshold, ruff >10000 breaker, T2 >5000 breaker, BSP cache invalidation timing. | MASTER:L413-L418 | P4C-E003, P4C-E007, P4C-E005 |
| PHASE-004/SD-4C-003 | **ADD** exact sanitization string "Tool call rejected by policy", plugins dependency direction, RuntimeClient ssb/fsm/clock rule. | MASTER:L419 | P4C-E004, P4C-E014, P4C-E016 |
| PHASE-004/SD-4C-004 | **ADD** 4A WARN-1/WARN-6 mitigations and metrics nine categories with coverage >80%. | MASTER:L422 | P4C-E017, P4C-E015 |
| PHASE-004/SD-4C-005 | **ADD** deferred skills/MCP rationale (no 4D–4G dependency on 4C for those). | MASTER:L423 | P4C-E002 |
| PHASE-004/SD-4I-001 | **ADD** full veto matrix including schema-invalid Degraded and VLM-unavailable Degraded; Mode A post-validation field checks + retry. | MASTER:L506-L507 | P4I-E007, P4I-E011 |
| PHASE-004/SD-4I-002 | **ADD** complete adversarial set (injection, 500MB, headers, path/symlink, sandbox, queue 65, ComfyUI 600s, CLAP noise). | MASTER:L496-L507 | P4I-E009 |
| PHASE-004/SD-4I-003 | **ADD** full RCE type system (MediaRenderAdapter, five adapters, VisualFeedbackLoop, VisualIssue, convergence by severity/category/location). | MASTER:L505 | P4I-E015–P4I-E017 |
| PHASE-004/SD-4I-004 | **ADD** SSB visual_feedback liveness, lazy register_lazy factory, ToolCapabilityFlags, MockVlm Phase0, ollama ≥0.12.7. | MASTER:L498-L502 | P4I-E012, P4I-E004, P4I-E005, P4I-E018 |
| PHASE-004/SD-4I-005 | **ADD** RCE latency targets (web <3s, Blender <10s, loop web <10s/iter), convergence ≥70% measure, total ≥43 tests, Phase 4 overall exit block. | MASTER:L506-L507 | P4I-E014, P4I-E008, P4I-E022 |
| PHASE-004/SD-4I-006 | **ADD** stale architecture-overview LLaVA note and MoE VRAM clarification conflict with media-validator L60. | MASTER:L501-L502 | P4I-E021, P4I-C001 |
| PHASE-004/SD-4I-007 | **QUALIFY** summary "bounded N" iterations: source default max_iterations=3 with early stop on unchanged issues. | MASTER:L505 | P4I-E017 |
| PHASE-005/SD-1F-001 | **ADD** full RESOLVED timeline: ollama 0.20.2 NOT_AVAILABLE, PR #8134 never merged, consolidated phase_1F.toml decision=UNAVAILABLE, EXIT satisfied via DROP. | MASTER:L114-L117 | P1F-E002–P1F-E006, P1F-E015 |
| PHASE-005/SD-1F-002 | **ADD** complete decision gate order, boundary DROP at ==1.3x/3000MB, prompt-lookup preference, acceptance early kill, k-sweep, GPU temp rules, marginal retest. | MASTER:L120-L120 | P1F-E020, P1F-E032–P1F-E037, P1F-E041 |
| PHASE-005/SD-1F-003 | **ADD** hardware scenarios A/B remaining VRAM, nvidia-smi polling, Phase-0-hardware-only ADOPT caveat, TIDE conditional later opt. | MASTER:L121-L121 | P1F-E010-P1F-E011, P1F-E040, P1F-E043 |
| PHASE-005/SD-1F-004 | **ADD** adversarial serde/num_draft_tokens=0, silent-ignore false positive, cache uniqueness, and all metrics file paths. | MASTER:L118-L120 | P1F-E022-P1F-E031, P1F-E039 |
| PHASE-005/SD-1F-005 | **QUALIFY** "Phase 2+" revisit as post-Phase-1/vLLM (3J), not a Phase-2 numbered dependency. | MASTER:L117, L659 | P1F-I001, P1F-E004 |
| PHASE-005/SD-2A-001 | **ADD** design D1-D5 paper grounding (BudgetThinker, RARR/SAFE, DSPy Assert/Suggest) and check-then-increment rationale. | MASTER:L172-L186 | P2A-E005-P2A-E009, P2A-E027 |
| PHASE-005/SD-2A-002 | **ADD** full T1-T12 catalog notes including max_calls=3 trick, free Status, default 5/800, scope three-way tests. | MASTER:L182-L182 | P2A-E020-P2A-E023 |
| PHASE-005/SD-2A-003 | **ADD** valid grounding type list and level mapping including tool→High and search→Verified. | MASTER:L184-L184 | P2A-E025, P2A-E026 |
| PHASE-005/SD-2A-004 | **ADD** metrics methodology (1000-iter median, no criterion without approval) and reset_topic full removal. | MASTER:L186-L186 | P2A-E028, P2A-E030 |
| PHASE-005/SD-2A-005 | **FLAG** T11 Err-vs-warn, incomplete allowed_next table, and "every step grounding" wording conflict. | MASTER:L172-L186 | P2A-A001-P2A-A002, P2A-C001 |
| PHASE-005/SD-3F-001 | **ADD** WARN preservation with BLOCK, severity DESC within type, and evaluation-order rationale. | MASTER:L305-L306 | P3F-E007, P3F-E008 |
| PHASE-005/SD-3F-002 | **ADD** full transition thresholds, mutable=0 dual invariants, transition log fields, insufficient-shadow-data error. | MASTER:L311-L313 | P3F-E018-P3F-E023 |
| PHASE-005/SD-3F-003 | **ADD** PostToolUse seed GREP surfaces (img/table/TODO strings), merge override rules, seed contract-test minimums. | MASTER:L317-L317 | P3F-E036-P3F-E038 |
| PHASE-005/SD-3F-004 | **ADD** proptest De Morgan/associativity, READ-LOCK on contract_tests, COMPOSITE depth 8 UNGROUNDED note, counterfactual runtime deferral. | MASTER:L318-L319 | P3F-E039-P3F-E042, P3F-E044 |
| PHASE-005/SD-3F-005 | **CORRECT/QUALIFY** MASTER SCHEMA_REQUIRED "escalates provider tier" and NaN/Inf guard: not stated in phase-3F body (schema validate + metric ops only). | MASTER:L306, L314 | P3F-I001, AC-014, AC-015 |
| PHASE-005/SD-3F-006 | **ADD** AstEvaluator injection option (b) recommended, domain_mapping config, scoring [0,15], blake2s discovery. | MASTER:L303-L316 | P3F-E006, P3F-E029, P3F-E032, P3F-E034 |
| PHASE-005/SD-3J-001 | **CORRECT** step numbering: multiplex+preload are 3J.6; profiles 3J.7; MILP 3J.8; MCP 3J.9; hooks 3J.10; AppContainer 3J.11; SSB 3J.12; verify 3J.13; E2E 3J.14; 31B 3J.15. | MASTER:L370-L374 | P3J-C001, P3J-C002, P3J-E020-P3J-E063 |
| PHASE-005/SD-3J-002 | **CORRECT** 3J.6 API is can_colocate + MultiplexingPolicy fields, not can_fit_onnx in this file. | MASTER:L370 | P3J-E020, P3J-I002 |
| PHASE-005/SD-3J-003 | **ADD** complete user-facing surface set: --local, elai.bat, /mode,/model,/plan,/resume,/session fork,/compact, InteractiveGate four choices, degraded prefix, startup banner. | MASTER:L675-L681 | P3J-E038, P3J-E042, P3J-E045, P3J-E050, P3J-E052, P3J-E058, P3J-E061, P3J-E068 |
| PHASE-005/SD-3J-004 | **ADD** full WIRE.* chain FOUNDATION→BUDGET→RULES→SESSION→PLAN→STUBS and UNIFIED RuntimeClient constraints. | MASTER:L675-L681 | P3J-E040-P3J-E055 |
| PHASE-005/SD-3J-005 | **ADD** MCP security/retry, hook sanitization string, AppContainer job limits/env allowlist, isolation_metric formula, SSB ≥17 write-time validation. | MASTER:L373-L373 | P3J-E025-P3J-E034 |
| PHASE-005/SD-3J-006 | **ADD** 3J.14 Ollama fallback re-probe, QueryPipeline Slot 4, BM25 update-on-edit, 60s REPL SLO, release binary checks. | MASTER:L675-L681 | P3J-E057-P3J-E062 |
| PHASE-005/SD-3J-007 | **ADD** 3J.15 31B buckets, EAGLE-3 skip reason, optional 100B SUBSTITUTE_MODEL gate, metrics file names. | MASTER:L374 | P3J-E063, P3J-E064 |
| PHASE-005/SD-3J-008 | **ADD** entire 3J.ADDENDUM shipped extensions A-8/9/10/17/25/26/27/28/31 with file paths. | MASTER:L360-L375 (absent) | P3J-E066, P3J-E067 |
| PHASE-005/SD-3J-009 | **ADD** hardware/model install snapshot (vLLM 0.19.0 WSL2, gemma4 sizes, native vision) and R1-R11 risk register. | MASTER:L363-L364 | P3J-E003-P3J-E008 |
| PHASE-005/SD-3J-010 | **FLAG** undefined 3J.COMPOSE/LOCAL steps, TBD contrast pairs, NF4 detail, hybrid mode history. | MASTER:L675-L681 | P3J-I001, P3J-A002-P3J-A004, AC-011-AC-012 |
| PHASE-006/SD-1H-001 | **ADD** full risk register R24–R30 including air-gap ort download, tokenizers separate crate, and statistical power of 10 vs 50 queries. | MASTER:L135-L143 | P1H-E007–P1H-E009 |
| PHASE-006/SD-1H-002 | **ADD** PrePooled vs RawMLM ONNX output shapes, RRF k=60 0.5/0.5 weights, circuit-breaker empty-expansion fallback, and SPLADE_MODEL_PATH default. | MASTER:L141 | P1H-E016–P1H-E021 |
| PHASE-006/SD-1H-003 | **ADD** early-drop 0.2–0.4 calibration note, quant Jaccard≥0.95 prefer int8, cross-model within-1% pick-faster, Wilcoxon paired protocol, and ≥20 non-zero pairs power gate. | MASTER:L142-L143 | P1H-E022–P1H-E026, P1H-A001 |
| PHASE-006/SD-1H-004 | **ADD** SPARSE_ONLY→BM25_ONLY cascade verification when splade disabled and full measurement field list including binary_size_delta_mb. | MASTER:L143 | P1H-E028–P1H-E029 |
| PHASE-006/SD-1H-005 | **ADD** knowledge.txt typo/lock note and UNGROUNDED code-search recall labeling. | MASTER:L135-L143 | P1H-E006, P1H-E010 |
| PHASE-006/SD-3I-001 | **ADD** full VCR cassette design (blake2s HTTP body hash, msgpack, prompt-drift fail) and TraceOps trajectory layer. | MASTER:L349 | P3I-E004–P3I-E006 |
| PHASE-006/SD-3I-002 | **ADD** custom SQLite VFS details, madsim timer coordination, and spike-to-possibly-skip VFS. | MASTER:L349 | P3I-E007–P3I-E008, P3I-A001 |
| PHASE-006/SD-3I-003 | **ADD** complete FsmEvent ten-name list, SuperstepBoundary atomic substeps, AgentCleanup lifecycle, and by-value SUT apply semantics. | MASTER:L351 | P3I-E012–P3I-E014 |
| PHASE-006/SD-3I-004 | **ADD** R1–R5 risk block, PROPTEST_CASES 10000/256 split, bare-fn check_determinism constraint, and proptest-regressions synthetic seed. | MASTER:L350-L354 | P3I-E009, P3I-E017 |
| PHASE-006/SD-3I-005 | **ADD** entropy-quarantine BLOCK PRE_GATE, burst-loss 5×100%, whitespace anti-spiral, zero-byte evidence rejection, and ops/sec ungrounded 100K target. | MASTER:L355-L358 | P3I-E016, P3I-E018–P3I-E021 |
| PHASE-006/SD-3I-006 | **ADD** SPEC_GAP that orchestrator.txt has no DST section; design lives in external-analysis. | MASTER:L346-L358 | P3I-E023 |
| PHASE-006/SD-3I-007 | **ADD** exit note AUTONOMOUS not enabled (Phase 4G) and HITL walking-skeleton is multi-phase. | MASTER:L358 | P3I-E022 |
| PHASE-006/SD-3K-001 | **ADD** current-stack inventory (1A–1I) and explicit missing list before steps. | MASTER:L159-L161 | P3K-E003 |
| PHASE-006/SD-3K-002 | **ADD** chunk metadata fields, Tantivy schema extension, format-version reindex migration, and 1000-token hard cap check. | MASTER:L162 | P3K-E005–P3K-E007 |
| PHASE-006/SD-3K-003 | **ADD** exact HyDE sidecar prompt/max_tokens/temperature, 3s timeout, 5-minute cache, and rerank-against-original adversarial mitigation. | MASTER:L164 | P3K-E010–P3K-E011 |
| PHASE-006/SD-3K-004 | **ADD** per-tier latency budgets FAST<20ms STANDARD<200ms DEEP<5s and AUTO word-count rules. | MASTER:L165 | P3K-E012–P3K-E013 |
| PHASE-006/SD-3K-005 | **ADD** A-RAG Score_kw formula, 3× oversample MAX grouping, progressive disclosure ≥50% tokens, and length-domination adversarial. | MASTER:L167 | P3K-E015–P3K-E018 |
| PHASE-006/SD-3K-006 | **ADD** epsilon/beta formulas, entity-extraction 0-entity adversarial, beta dominance adversarial, and post-improvement tokens ≥30% reduction criterion. | MASTER:L168-L170 | P3K-E019–P3K-E022 |
| PHASE-006/SD-3K-007 | **QUALIFY** paper list and independence from 4D web research. | MASTER:L159-L170 | P3K-E015 |
| PHASE-006/SD-4A-001 | **ADD** per-type frame size caps, postcard version framing details, and exactly-15 const_assert. | MASTER:L380-L381 | P4A-E013–P4A-E015 |
| PHASE-006/SD-4A-002 | **ADD** full listen-before-spawn sequence, UUID v4 agent_id, and version-handshake kill. | MASTER:L385 | P4A-E019–P4A-E020 |
| PHASE-006/SD-4A-003 | **ADD** complete functional isolation test battery including env sanitization, /proc/sys masks, RLIMIT behaviors, concurrent agent isolation, and role-method grounding matrix. | MASTER:L392 | P4A-E021–P4A-E025 |
| PHASE-006/SD-4A-004 | **ADD** full adversarial IPC/replan/self-mod/crash/Windows test inventory and WARN/BLOCK coverage requirement. | MASTER:L389 | P4A-E026–P4A-E031 |
| PHASE-006/SD-4A-005 | **ADD** Linux clone flags, rlimits defaults, heartbeat 5s/15s, and Windows Job object numbers. | MASTER:L382-L388 | P4A-E032–P4A-E034 |
| PHASE-006/SD-4A-006 | **ADD** phase_complete SSB fields and RuntimeClient wiring exit rule. | MASTER:L391 | P4A-E018, P4A-E038 |
| PHASE-006/SD-4A-007 | **ADD** separation-cost planning numbers (0.1ms/50ms/20MB) vs SLO targets. | MASTER:L392 | P4A-E036–P4A-E037, P4A-A001 |
| PHASE-006/SD-4A-008 | **QUALIFY** feasibility GRC 3/600 stub-write/bash timeouts appear in MASTER dispatch bullet but are not restated as 4A step numbers in phase-4A.txt (they live in later 4F); do not invent them as 4A.x steps. | MASTER:L390 | P4A-E017 |
| PHASE-006/SD-4H-001 | **ADD** revision history 2026-04-15 audit resolution (4 BLOCK/8 WARN), paper refs DGM/DarwinNet/SWE-World, and code-vs-config safety divergence. | MASTER:L480-L482 | P4H-E001–P4H-E003, P4H-C001 |
| PHASE-006/SD-4H-002 | **ADD** complete PipelineOutcome field set including ETC/staleness/truncation/miss vectors and CatchLayer enum. | MASTER:L486-L488 | P4H-E007–P4H-E010 |
| PHASE-006/SD-4H-003 | **ADD** full AllowedConfigKey Parameter/DataTable/LOCKED lists including cloud spend locks and Freshness* keys. | MASTER:L493 | P4H-E015–P4H-E017 |
| PHASE-006/SD-4H-004 | **ADD** DataTableValidation FPR defaults, ProjectScope, ReasoningTraceRef, ProposedChange variants, and StructuralChange type set. | MASTER:L479-L493 | P4H-E018–P4H-E026 |
| PHASE-006/SD-4H-005 | **ADD** ouroboros OSC/REG/EVO algorithms, SSB action types, exploration boost, key freeze, and MSI force-STABILIZING on Withhold. | MASTER:L480-L494 | P4H-E022–P4H-E025, P4H-E040–P4H-E042, P4H-E045 |
| PHASE-006/SD-4H-006 | **ADD** staged-eval archive handling differences Stage1 discard vs Stage2/3 retain, trigger_task.toml schema, and mode-aware evaluation. | MASTER:L484 | P4H-E034–P4H-E036 |
| PHASE-006/SD-4H-007 | **ADD** Layer1A four growth paths with numeric thresholds (3 catches, FPR 0.05, domain 5+, verified 0.8) and no-staged-eval path. | MASTER:L483 | P4H-E031–P4H-E033 |
| PHASE-006/SD-4H-008 | **ADD** MSI frequency schedules (1/3 and 1/5 pipelines), per-mode pause, anti-fragility N=10 recovery bound, and exponential weight series. | MASTER:L491-L492 | P4H-E039 |
| PHASE-006/SD-4H-009 | **ADD** anti-staleness `elai dep-audit`, reasoning fix-template loop, token-efficiency slot proposals, and optional FM cross-transfer. | MASTER:L494 | P4H-E043–P4H-E046 |
| PHASE-006/SD-4H-010 | **ADD** conditional learned-reward EVALUATE section with domain-mismatch warnings and R53. | MASTER:L480-L494 | P4H-E047 |
| PHASE-006/SD-4H-011 | **ADD** four-layer enforcement detail, lock-list unreadability, objective-hacking trigger, and FPR threshold lock. | MASTER:L494 | P4H-E027, P4H-E038 |
| PHASE-007/SD-PP0-001 | **ADD** exact file/line sites for PermissionsExt and set_mode (7 sites / 3 files) and Windows no-op chmod pattern. | 01-phase:L13-L14 | PP0-E006–PP0-E008 |
| PHASE-007/SD-PP0-002 | **ADD** named failing plugin tests and Windows .cmd / cmd.exe shell dispatch requirements. | 01-phase:L15-L16 | PP0-E010–PP0-E013 |
| PHASE-007/SD-PP0-003 | **ADD** named tools failures (path ends_with, chmod, python3→python, env PoisonError) and 10-run adversarial stability. | 01-phase:L17-L18 | PP0-E014–PP0-E019 |
| PHASE-007/SD-PP0-004 | **ADD** exact warning sites, pre_phase_0.toml fields, measurements, grounding table, and WIRING_CONTRACT archive-verify surface. | 01-phase:L19-L22 | PP0-E020–PP0-E025 |
| PHASE-007/SD-PP0-005 | **ADD** R-pre1/R-pre2 mitigations and cfg-gate-not-delete rule. | 01-phase:L10-L11 | PP0-E003–PP0-E004 |
| PHASE-007/SD-PP0-006 | **FLAG** incomplete pre-enumeration of all cfg-gated tests and HookRunner shell audit. | 01-phase:L21-L22 | PP0-A001–PP0-A002 |
| PHASE-007/SD-P1I-001 | **ADD** ACTION-REQUIRED 1G→1I dep-graph fix and compile-failure scenario if ignored. | 01-phase:L147-L147 | P1I-E002, P1I-C001 |
| PHASE-007/SD-P1I-002 | **ADD** permanent-vs-transient classification, 2-success HALF_OPEN, fresh InferenceSession, deterministic model_id jitter details. | 01-phase:L148-L149 | P1I-E007–P1I-E011 |
| PHASE-007/SD-P1I-003 | **ADD** full cascade chain including TEXT-only/CODE-only intermediate modes and SPLADE-disabled collapse. | 01-phase:L150-L151 | P1I-E013–P1I-E018 |
| PHASE-007/SD-P1I-004 | **ADD** newline-accepting validation tests, canary_reference_vec first-startup generation, DEGRADED⇒BM25_ONLY invariant tests. | 01-phase:L152-L154 | P1I-E020–P1I-E025 |
| PHASE-007/SD-P1I-005 | **ADD** batching merge/segregate rules for cross-encoder queue and overflow returns unranked non-empty candidates. | 01-phase:L155-L155 | P1I-E026–P1I-E027 |
| PHASE-007/SD-P1I-006 | **ADD** full QueryTrace/StageTrace field lists, nine stage names, DEBUG-not-SSB rule, session-end aggregates. | 01-phase:L156-L156 | P1I-E028–P1I-E031 |
| PHASE-007/SD-P1I-007 | **ADD** index_version schema fields, archive-on-failure abort, user notification, isolation-under-degradation verification ownership. | 01-phase:L157-L158 | P1I-E032–P1I-E036 |
| PHASE-007/SD-P1I-008 | **ADD** MEASURE targets and Phase-1 overall measurement list. | 01-phase:L145-L158 | P1I-E037–P1I-E039 |
| PHASE-007/SD-P2D-001 | **ADD** design decisions D1–D5 (no-DKG detection, severity classes, separate inline counter, ReplanRequest MAJOR, type inventory). | 01-phase:L220-L226 | P2D-E004–P2D-E011 |
| PHASE-007/SD-P2D-002 | **ADD** full T1–T10 TDD inventory including CONTRADICTS_PLAN: marker and BudgetExhausted topic. | 01-phase:L228-L229 | P2D-E013–P2D-E015 |
| PHASE-007/SD-P2D-003 | **ADD** exact Critical pattern list, case-insensitive compare, assumption/consequence templates, guard_added=false default. | 01-phase:L227-L227 | P2D-E016–P2D-E018 |
| PHASE-007/SD-P2D-004 | **ADD** phase_2D.toml fields and overhead/false-positive SLOs; architecture-overview update requirement. | 01-phase:L231-L231 | P2D-E019 |
| PHASE-007/SD-P2D-005 | **ADD** grounding summary distinguishing GROUNDED/VERIFIED/UNGROUNDED claims and 4F deferrals. | 01-phase:L230-L230 | P2D-E020–P2D-E022 |
| PHASE-007/SD-P2D-006 | **FLAG** D3 method list vs SourceMatch-only type conflict and T4 `.passed` field wording. | 01-phase:L224-L229 | P2D-C001, P2D-A001–P2D-A002 |
| PHASE-007/SD-P3B-001 | **ADD** R10/R-3B-1/2/3 risks, r2d2 WAL cleanup, CHECK migration drop+recreate session semantics. | 01-phase:L246-L249 | P3B-E006–P3B-E007 |
| PHASE-007/SD-P3B-002 | **ADD** full SQL schema, indexes, ssb_files junction, and note that orchestrator.txt §3 synopsis is incomplete. | 01-phase:L249-L252 | P3B-E009–P3B-E012 |
| PHASE-007/SD-P3B-003 | **ADD** complete typed payload struct inventory including PhaseComplete fsync-before-unload semantics. | 01-phase:L256-L256 | P3B-E023–P3B-E024 |
| PHASE-007/SD-P3B-004 | **ADD** write-loop BEGIN IMMEDIATE, oneshot seq, shutdown drain, broadcast-after-commit, Arc payload, CustomizeConnection. | 01-phase:L253-L254 | P3B-E017–P3B-E021, P3B-E027 |
| PHASE-007/SD-P3B-005 | **ADD** SsbTraceable boundary, msgpack disk serialization, 1MB payload reject, full unit/property/adversarial/determinism suite and measure SLOs. | 01-phase:L255-L257 | P3B-E025–P3B-E029 |
| PHASE-007/SD-P3B-006 | **ADD** audit-fix history BLOCK1–5 WARN1–6 from s-1427264. | 01-phase:L246-L257 | P3B-E031 |
| PHASE-007/SD-P3B-007 | **FLAG** internal 15-vs-17 payload wording conflict. | 01-phase:L250-L250 | P3B-C001, P3B-A001 |
| PHASE-007/SD-P4D-001 | **ADD** Designer role config, SystemConstraints sizing defaults, confidence/ranking formulas, adaptive complexity table, info_gain prune rule. | 01-phase:L426-L434 | P4D-E004–P4D-E016 |
| PHASE-007/SD-P4D-002 | **ADD** full 4D.1–4D.6 TDD/adversarial gates, FORMULATE-SEARCH-EXTRACT-REFLECT loop, cloud-only speculation, measure keys, RuntimeClient wire rule. | 01-phase:L435-L435 | P4D-E019–P4D-E027 |
| PHASE-007/SD-P4D-003 | **ADD** recursive 4-level protocol, all decomposition/baby-step block types, ResearchCache, DecisionSource, package anti-hallucination gate, Interactive/Autonomous selection. | 01-phase:L436-L440 | P4D-E028–P4D-E037 |
| PHASE-007/SD-P4D-004 | **ADD entire web-research extension 4D.11–4D.18**: SearXNG client, WebFetcher/robots/readability, SourceQualityTable ≥200 domains, Web A-RAG, six VRAM tiers, budget manager, WebResearchOrchestrator, E2E metrics file. | 01-phase:L425-L440 | P4D-E038–P4D-E056 |
| PHASE-007/SD-P4D-005 | **ADD** CLI `--vram-tier`, INTERACTIVE progress via research_step events, design principles from survey papers. | 01-phase:L440-L440 | P4D-E038, P4D-E049, P4D-E057 |
| PHASE-007/SD-P4D-006 | **FLAG CONFLICT** research_* SSB action types not in 3B 17-type schema; Interactive present_options UI unspecified; projected VRAM ceilings. | 01-phase:L425-L440 | P4D-C001, P4D-A001–P4D-A003 |
| PHASE-007/SD-P4D-007 | **CORRECT/EXPAND** summary DecisionSource labels: source uses Research/UserChoice/AutoSelected (not RESEARCH\|EXISTING_PROJECT\|USER_SPECIFIED\|FALLBACK). | 01-phase:L439-L439 | P4D-E032 |
| PHASE-008/SD-3C-001 | **ADD** full input/output contracts (3B/3A/2C→3D/3E/3F/4A/ReplanRequest max-1). | 01-MASTER:L259-L274 | P3C-E005–P3C-E012 |
| PHASE-008/SD-3C-002 | **ADD** claw-code mapping details (UUID EntropySource, ClockProvider, TrustRequired drop). | 01-MASTER:L262 | P3C-E013–P3C-E015 |
| PHASE-008/SD-3C-003 | **ADD** R6 VRAM serialize rule and R31 NotFound indistinguishability tests. | 01-MASTER:L268 | P3C-E016–P3C-E017, P3C-E030 |
| PHASE-008/SD-3C-004 | **ADD** five checkpoint files, UntrackedValue rejection, Windows atomic rename, metadata-only SSB checkpoint. | 01-MASTER:L269-L271 | P3C-E034–P3C-E038 |
| PHASE-008/SD-3C-005 | **ADD** recovery recipe map, corrupt degrade, monotonic step_index, no tool replay. | 01-MASTER:L272 | P3C-E040–P3C-E044 |
| PHASE-008/SD-3C-006 | **ADD** spiral UI options string, N-3 recovery_target, blake3 B-16A supersession. | 01-MASTER:L274 | P3C-E053–P3C-E057 |
| PHASE-008/SD-3C-007 | **ADD** Tier B bash gap deferral, isolation_metric aggregation out-of-scope, measure targets. | 01-MASTER:L259-L274 | P3C-E058–P3C-E060 |
| PHASE-008/SD-3E-001 | **ADD** full AdaptiveLevel enum including Terminal; exact compute remainder-to-CodeContext rule. | 01-MASTER:L290-L300 | P3E-E007, P3E-E026 |
| PHASE-008/SD-3E-002 | **ADD** BSP call timing, channel→slot6 rules, handoff BP2 recompute, Display surface. | 01-MASTER:L290-L300 | P3E-E016–P3E-E017, P3E-E024, P3E-E035 |
| PHASE-008/SD-3E-003 | **ADD** floor steal order, BudgetInfeasible at dynamic<700, CodeContext<200 warning. | 01-MASTER:L294 | P3E-E027–P3E-E028 |
| PHASE-008/SD-3E-004 | **ADD** numbered steps 3E.1–3E.7 acceptance detail MASTER only points to phase file for. | 01-MASTER:L300 | P3E-E036 |
| PHASE-008/SD-3E-005 | **ADD** reduction logs excluded from current context; 100x determinism; no Instant::now. | 01-MASTER:L297-L299 | P3E-E032–P3E-E034 |
| PHASE-008/SD-4B-001 | **ADD** full DAIRA marker set (15 forms) and MarkerReport counts. | 01-MASTER:L402-L403 | P4B-E013, P4B-E015 |
| PHASE-008/SD-4B-002 | **ADD** force_full INTERACTIVE user-decide vs AUTONOMOUS auto-outline. | 01-MASTER:L407 | P4B-E019 |
| PHASE-008/SD-4B-003 | **ADD** R1–R4 risks, paper ingest PRE_GATE, RuntimeClient wiring exit rule. | 01-MASTER:L395-L409 | P4B-E005–P4B-E008, P4B-E027, P4B-E032 |
| PHASE-008/SD-4B-004 | **ADD** IntegrationTestRunner tool_id AUDITING-only, IntegrationResult stdout_hash. | 01-MASTER:L409 | P4B-E028–P4B-E030 |
| PHASE-008/SD-4B-005 | **ADD** metrics key list and UNGROUNDED labels on latency/BLOCK reduction projections. | 01-MASTER:L409 | P4B-E022–P4B-E026 |
| PHASE-008/SD-4B-006 | **QUALIFY** recorded DONE status applies to 4B.1–4.4 only; measure+runner remain deferred. | 01-MASTER:L398-L399 | P4B-E002–P4B-E003, P4B-C001 |
| PHASE-008/SD-4F-001 | **ADD** complete type field lists (GpuInfo uuid, stderr 512, ResourceDelta, ConstraintResult margin). | 01-MASTER:L461 | P4F-E002–P4F-E008 |
| PHASE-008/SD-4F-002 | **ADD** 16+8 named tests T1–T16/A1–A8 and dual-timeout kill_on_drop detail. | 01-MASTER:L463 | P4F-E021–P4F-E024 |
| PHASE-008/SD-4F-003 | **ADD** E-22 sysinfo rejection / env fallback path and SD2 flat file layout. | 01-MASTER:L457-L465 | P4F-E034, P4F-C001–P4F-C002 |
| PHASE-008/SD-4F-004 | **ADD** metrics targets and 12 testable invariants summary. | 01-MASTER:L463 | P4F-E028, P4F-E036 |
| PHASE-008/SD-4F-005 | **ADD** DispatchStage.agent AUTHORITATIVE override rule and SSB branch merge/discard. | 01-MASTER:L462 | P4F-E018, P4F-E020 |
| PHASE-008/SD-4F-006 | **FLAG** web_search inclusion conflict in FEASIBILITY tool set. | 01-MASTER:L462 | P4F-A001 |
| PHASE-008/SD-4J-001 | **ADD** EMPIRE-PENDING pre-resolved decisions 1–4 (scaffold, submodule path, 5 SSB variants, EmpireIpc namespace). | 01-MASTER:L508-L517 | P4J-E002–P4J-E005 |
| PHASE-008/SD-4J-002 | **ADD** Tauri command surface + Svelte panel scaffolding (user-facing). | 01-MASTER:L508-L517 | P4J-E033 |
| PHASE-008/SD-4J-003 | **ADD** rate-limit global vs high-priority elevated cap, generic rejection, reconnect Register. | 01-MASTER:L514-L515 | P4J-E020–P4J-E023 |
| PHASE-008/SD-4J-004 | **ADD** full measure keys/targets, FSB evaluate trigger, multi-brain evaluate trigger. | 01-MASTER:L517 | P4J-E028–P4J-E030 |
| PHASE-008/SD-4J-005 | **ADD** repair blast-radius, TYPE_CHECKING false-positive guard, one-issue-per-pipeline. | 01-MASTER:L512 | P4J-E025 |
| PHASE-008/SD-4J-006 | **FLAG CONFLICT** summary/body empire_event singular vs decision 5 variants; elai_empire crate name drift. | 01-MASTER:L514-L516 | P4J-C001–P4J-C002 |
| PHASE-008/SD-IDX-001 | **ADD** PRE0 Windows-only scope and baseline LOC/test counts not expanded in MASTER's PRE0 section compression. | 01-MASTER:L13 | IDX-E002–IDX-E003 |
| PHASE-008/SD-IDX-002 | **ADD** Phase 0 entropy-quarantine occurrence count (60) and 9-crate/one-tool skeleton. | 01-MASTER:L24 | IDX-E004–IDX-E005 |
| PHASE-008/SD-IDX-003 | **ADD** 1F ADOPT threshold 1.3x and 1H vocab/recall KEEP gates as compact numbers. | 01-MASTER:L168-L181 (1F), related | IDX-E012, IDX-E014 |
| PHASE-008/SD-IDX-004 | **ADD** 2C six role budget table and anti-rubber-stamp FINDING rule. | 01-MASTER:L340-L361 area | IDX-E019–IDX-E021 |
| PHASE-008/SD-IDX-005 | **QUALIFY** index is not a substitute for numbered phase files' PRE_GATE/TASK/ACCEPTANCE. | 01-MASTER:L5 | IDX-I001 |

---

# Closing audit block

## Ledgers merged

- Deep ledgers: **8/8** (`DEEP-PHASE-001` … `DEEP-PHASE-008`).
- Family summary: **1/1** (`01-phase-index-MASTER.discovery.md`).
- Total inputs merged: **9/9** (8/8 + summary).

## Per-ledger atom counts

| Ledger | Declared feature | Parsed feature | IX | SD | AC |
|---|---:|---:|---:|---:|---:|
| PHASE-001 | 278 | 278 | 18 | 36 | 12 |
| PHASE-002 | 189 | 189 | 20 | 31 | 15 |
| PHASE-003 | 151 | 151 | 20 | 24 | 15 |
| PHASE-004 | 157 | 157 | 20 | 32 | 12 |
| PHASE-005 | 213 | 213 | 20 | 26 | 15 |
| PHASE-006 | 238 | 238 | 22 | 38 | 10 |
| PHASE-007 | 198 | 198 | 20 | 34 | 13 |
| PHASE-008 | 253 | 253 | 32 | 35 | 11 |
| **Deep subtotal** | **1677** | **1677** | **172** | **256** | **103** |
| MASTER summary bullets | — | **515** | 0 | 0 | 0 |
| SUP-* supersession notes | — | **12** | — | — | — |
| PREC-* plan-number notes | — | **10** | — | — | — |

## Aggregate distinct record count

- Distinct feature atom IDs (deep): **1677** (parsed rows **1677**; ledger-declared sum **1677**).
- Namespaced IX records: **172** (declared sum 172).
- Namespaced SD records: **256** (ledger-declared sum 250; PHASE-008 table has 35 SD rows while its audit line said 29; aggregate uses table rows).
- Namespaced AC records: **103** (declared sum 103).
- MASTER summary atoms: **515**.
- Supersession notes (SUP-*): **12**.
- Plan-number precedence notes (PREC-*): **10**.
- SURFACE-domain atoms indexed: **103**.
- **Aggregate distinct record count (feature + IX + SD + AC + MASTER + SUP + PREC): 2745**.

## Superseded notes count

- Explicit SUP-* notes: **12**.
- Plan-number PREC-* notes: **10**.
- **Total superseded/precedence notes: 22**.

## Zero-omission statement

All eight deep PHASE ledgers and the family MASTER summary were merged into this aggregate. Every atomic feature table row from each deep ledger is included under its ledger section with original Atomic ID and exact source anchor. Every IX, AC, and SD relational record is included with ledger namespace (`PHASE-00N/…`). Every MASTER inventory bullet marked `[E]`/`[I]`/`[A]`/`[C]`/`[D]`/`[Q]` is included as a `MASTER-*` atom. No minor atom was dropped for brevity. Losing descriptions under newer-plan precedence remain in place in the atomic tables; winners are recorded only as SUP-*/PREC-* notes. User-facing surfaces are collected by ID for dashboard DESIGN.md without replacing the full atom text.

## Parse fidelity

- Feature ID uniqueness: unique=1677, rows=1677, match=True.
- Duplicate feature IDs across ledgers: **0**.
