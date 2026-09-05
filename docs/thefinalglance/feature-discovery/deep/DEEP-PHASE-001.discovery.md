# DEEP-PHASE-001 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1331-L1342`, heading `### DEEP-PHASE-001`; the next chunk starts at L1344.
- Declared chunk: family PHASE, **6 files**, **2,692 logical lines**, **165,923 bytes**, all six classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all six listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `phase-0:Lx-Ly`, `phase-1C:Lx-Ly`, `phase-3A:Lx-Ly`, `phase-3J-BASELINE:Lx-Ly`, `phase-4K:Lx-Ly`, and `phase-4M:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 780 | 49,096 | 53aba5d44d1ee36a23b8b71cbd8e2f3109f971bbaa2002d55f642b5619d66aaa | docs/specs/guide/steps/phase-0.txt | exact |
| numbered/index original | 531 | 37,863 | 79001cdb64eaac78430acef16e34874d838aebca6ec9fa2684825f1b50c4d1a2 | docs/specs/guide/steps/phase-1C.txt | exact |
| numbered/index original | 343 | 22,625 | 4cf40b6145687d1efd74f3e34b419c5eaefb8ffa3eea13ccaa4313816ac44b09 | docs/specs/guide/steps/phase-3A.txt | exact |
| numbered/index original | 146 | 6,516 | 89c25d16ba40711cb101015ac8daa3159a055ba687c34e72323499c15b77e1e4 | docs/specs/guide/steps/phase-3J-BASELINE.txt | exact |
| numbered/index original | 427 | 26,506 | 986b8dbdda09c2980a3703e34061b272552b29ef4f6635d8cf6e72415b6789a2 | docs/specs/guide/steps/phase-4K.txt | exact |
| numbered/index original | 465 | 23,317 | d4607fa920edeca49355cf134fecb5452d4ce40daf5ad740eb23647d9d0f6ffa | docs/specs/guide/steps/phase-4M.txt | exact |
| **Aggregate** | **2,692** | **165,923** | — | **6 exact paths** | **exact** |

## Per-file atomic ledgers

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

## Cross-source interactions and lifecycle

| Interaction ID | Class | Explicit discovery or bounded interpretation | Source atoms |
|---|---|---|---|
| IX-001 | E | Phase 0 nine-crate split, LlmProvider, minimal FSM, visibility, msgpack, contracts, entropy quarantine are substrate for later phases here. | P0-E002, P0-E046, P0-E049–P0-E065 |
| IX-002 | E | Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience. | P0-E015–P0-E017; P1C-E018, P1C-E029 |
| IX-003 | E | Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine. | P0-E060–P0-E064; P1C-E026 |
| IX-004 | E | Phase 1C SSB file_write invalidation and knowledge_degradation require Phase 3B SSB (outside chunk) but specify production wiring. | P1C-E021, P1C-E032 |
| IX-005 | E | Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G. | P0-E052; P3A-E001–P3A-E029 |
| IX-006 | E | Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG. | P3A-E019, P3A-E023; P1C-E029 |
| IX-007 | E | Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal. | P3A-E033; P4K-E012, P4K-E047 |
| IX-008 | E | Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI. | P3A-E024–P3A-E025; P4K-E013–P4K-E014, P4K-E049–P4K-E052 |
| IX-009 | E | Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface. | P4K-E011–P4K-E026 |
| IX-010 | E | Phase 3J-BASELINE fills capability_profile.toml for ModelCapabilityProfile without booting 3A/4K. | P3JB-E001–P3JB-E002, P3JB-E007–P3JB-E009 |
| IX-011 | E | 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability. | P3JB-E011; P4M-E006 |
| IX-012 | E | Phase 4M migrates 3J backends and Ollama/vLLM into RuntimeAdapter; requires 4A Tier C for CLI adapters. | P4M-E001, P4M-E014–P4M-E020 |
| IX-013 | E | Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace. | P0-E055; P4K-E026; P4M-E023–P4M-E028 |
| IX-014 | E | Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback. | P3A-E001; P4M-E022 |
| IX-015 | E | Slash command evolution: Phase 0 /task|/config|/continue|/explore|/status; 3A /config|/workflow CONFIGURING; 4M /runtime family. | P0-E055; P3A-E008; P4M-E023–P4M-E025 |
| IX-016 | I | RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here. | P1C-E029; P4M-E014, P4M-E028 |
| IX-017 | E | 1C three-ranker RRF stays additive for later SPLADE/cross-encoder; semaphore overflow unranked. | P1C-E007, P1C-E030–P1C-E031 |
| IX-018 | E | Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis. | P0-E053, P0-E056; P3A-E017 |

## Ambiguity and conflict register

| Register ID | Class | Issue | Source atoms |
|---|---|---|---|
| AC-001 | C | TokenUsage field set differs design (4 fields) vs 0.2c shim (2 fields). | P0-C001 |
| AC-002 | A | prompt.rs REPLACE vs STRIP open. | P0-A001 |
| AC-003 | A | remote.rs DEFERRED vs STRIP open. | P0-A002 |
| AC-004 | A | sandbox whole-file vs types-only open. | P0-A003 |
| AC-005 | C | 1C model path onnx/ vs Step 5 per-model subdirs. | P1C-C001 |
| AC-006 | C | SPLADE deferral 1G (source) vs 1H (MASTER summary). | P1C-C002 |
| AC-007 | C | AUDITING recovery transitions need orchestrator.txt SPEC_UPDATE. | P3A-C001 |
| AC-008 | A | UNGROUNDED COMPLETE/IDLE tool policies and gate traits. | P3A-E016, P3A-I001 |
| AC-009 | A | REDIRECT invalid-target and full risk matrix incomplete. | P3A-A001–P3A-A002 |
| AC-010 | A | 3J-BASELINE fixture prompts and compression tolerance unspecified. | P3JB-A001–P3JB-A002 |
| AC-011 | A | 4K submit_task path, projects precedence, diff modal vs replace. | P4K-A001–P4K-A003 |
| AC-012 | A | 4M adapter alias, panel placement, name list, hybrid/auto detail. | P4M-A001–P4M-A003 |

## Deltas against `01-phase-index-MASTER.discovery.md`

Summary artifact: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. Anchors use `01-phase:` line numbers.

### phase-0 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P0-001 | **ADD** full risk register R17–R25 with LOC/import facts and stale comparison-doc warning. | 01-phase:L24-L42 | P0-E004–P0-E012, P0-E068 |
| SD-P0-002 | **ADD** workspace path migration (rust/→crates/), archive semantics, strip-before-restructure rationale. | 01-phase:L24-L33 | P0-E013–P0-E014 |
| SD-P0-003 | **ADD** ONNX path, blake2 setup, DEGRADED BM25_ONLY, models/** hidden_paths. | 01-phase:L24-L42 | P0-E015–P0-E017 |
| SD-P0-004 | **ADD** ordered strip sequence, KEEP/STRIP slash lists, TokenUsage shim vs core fields. | 01-phase:L29-L32 | P0-E024–P0-E029, P0-C001 |
| SD-P0-005 | **ADD** module mapping LOCs, tools 2000-LOC cap, cli <500 soft, nine crates, contracts TDD red phase. | 01-phase:L31-L39 | P0-E032–P0-E048, P0-E066 |
| SD-P0-006 | **ADD** LlmProvider streaming complete, Ollama endpoints, allowlist sanitization, hidden_paths inventory, smoke skip. | 01-phase:L36-L44 | P0-E049–P0-E059 |
| SD-P0-007 | **ADD** ClockProvider/EntropySource shapes, RuntimeContext, CI exclusions, ELAI_SEED=42 dual-run proof. | 01-phase:L40-L41 | P0-E060–P0-E065 |
| SD-P0-008 | **FLAG** open prompt/remote/sandbox extraction choices omitted from summary. | 01-phase:L24-L42 | P0-A001–P0-A004 |

### phase-1C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P1C-001 | **ADD** R22/R26/R27/R29 mitigations and HYBRID→SPARSE_ONLY→BM25_ONLY cascade. | 01-phase:L69-L83 | P1C-E003–P1C-E006, P1C-E012–P1C-E013 |
| SD-P1C-002 | **CORRECT** SPLADE deferral: original 1G vs summary 1H. | 01-phase:L74-L74 | P1C-E007, P1C-C002 |
| SD-P1C-003 | **ADD** TurboQuant grounded params and ban on 1C quantization. | 01-phase:L83-L83 | P1C-E015, P1C-E034 |
| SD-P1C-004 | **ADD** DualVector/OrtSession/UsearchIndex APIs, 512 truncation, filtered_search rebuild, RRF k=60 fixed. | 01-phase:L75-L81 | P1C-E017–P1C-E022, P1C-E030, P1C-E035 |
| SD-P1C-005 | **ADD** ≥22 tests, physical DB isolation, QueryTrace, knowledge_degradation triggers, write-before-query race rule. | 01-phase:L79-L81 | P1C-E024–P1C-E025, P1C-E032–P1C-E036 |
| SD-P1C-006 | **ADD** Step 3.5 tensor gate, Step 5 fields, cache_hit BLOCK <90%, path naming tension. | 01-phase:L80-L82 | P1C-E027, P1C-E037–P1C-E039, P1C-C001 |
| SD-P1C-007 | **QUALIFY** summary fastembed pin: original forbids adding fastembed with ort. | 01-phase:L74-L74 | P1C-E014 |

### phase-3A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P3A-001 | **ADD** full transition matrix including CONFIGURING, FEASIBILITY revise, ReplanRequest, SKILL.md AUDITING arms + SPEC_UPDATE. | 01-phase:L233-L244 | P3A-E005–P3A-E008, P3A-C001 |
| SD-P3A-002 | **ADD** transition payload, token_budget_remaining, msgpack path, TransitionConflict, typed errors. | 01-phase:L237-L237 | P3A-E009–P3A-E014 |
| SD-P3A-003 | **ADD** full tool-filter table including FEASIBILITY stub-write and UNGROUNDED COMPLETE/IDLE. | 01-phase:L238-L239 | P3A-E015–P3A-E017 |
| SD-P3A-004 | **ADD** DAIRA max-3, EvidenceError shape, DEGRADED-allows-with-warning, FEASIBILITY ungated. | 01-phase:L240-L242 | P3A-E019–P3A-E023 |
| SD-P3A-005 | **ADD** structured EvidenceSummary, trait mockability, no-timeout default, risk examples + overrides. | 01-phase:L243-L244 | P3A-E024–P3A-E030 |
| SD-P3A-006 | **ADD** 81-pair tests, latency SLOs, SSB phase_transition non-deliverable note. | 01-phase:L233-L244 | P3A-E031–P3A-E033 |

### phase-3J-BASELINE deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P3JB-001 | **ADD entire file**: MASTER does not index phase-3J-BASELINE.txt. | 01-phase:L360-L375, L655 | P3JB-E001–P3JB-E019 |
| SD-P3JB-002 | **ADD** standalone stdlib harness, 8 tiers, 7 metrics, REPAIR_CONTEXT ±15%, JSON-schema 20-prompt, exit 0/1/2. | 01-phase:L655 | P3JB-E002–P3JB-E015 |
| SD-P3JB-003 | **ADD** non-overwrite run_benchmark.py, optional CI bench-capability, ShortCoder ban + output_token_ratio. | 01-phase:L655 | P3JB-E004, P3JB-E014, P3JB-E017 |

### phase-4K deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P4K-001 | **CORRECT/EXPAND** summary phase ribbon/commit history: MVP is project selector + task + plan tree + activity feed + diff viewer. | 01-phase:L528-L528 | P4K-E029–P4K-E053 |
| SD-P4K-002 | **ADD** full visual tokens, window geometry, identifier, Vite :1420, elai --dashboard single-instance focus. | 01-phase:L522-L528 | P4K-E006, P4K-E018, P4K-E026–P4K-E027 |
| SD-P4K-003 | **ADD** distinct Tauri commands including projects/templates/submit_task/get_action_detail. | 01-phase:L527-L527 | P4K-E019–P4K-E024, P4K-E033–P4K-E035, P4K-E053 |
| SD-P4K-004 | **ADD** intervention options, per-hunk controls, multi-file nav, preview open, 500-entry cap, in-flight poll skip, 200-char truncate. | 01-phase:L525-L526 | P4K-E013–P4K-E014, P4K-E040–P4K-E055 |
| SD-P4K-005 | **ADD** project name validation, min 5-char task, projects.json corrupt empty-list, post-MVP deferred list. | 01-phase:L528-L529 | P4K-E033, P4K-E036, P4K-E057 |
| SD-P4K-006 | **ADD** WebView2 bootstrapper, <100KB JS, typed schema drift compile-fail, graceful open-fail dialog. | 01-phase:L529-L529 | P4K-E007–P4K-E010, P4K-E028 |

### phase-4M deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P4M-001 | **ADD** summary stops at 4M.1; original continues 4M.2–4M.6 adapters/hybrid/CLI/dashboard/metrics/migration. | 01-phase:L531-L541, L653 | P4M-E014–P4M-E031 |
| SD-P4M-002 | **ADD** PermissionMode defaults, ResolveBackendError, capabilities fields, factory timeouts. | 01-phase:L537-L539 | P4M-E006–P4M-E009 |
| SD-P4M-003 | **ADD** default per-role map, runtime vs model fallback order, ESCALATE/HALT, per-session state. | 01-phase:L539-L539 | P4M-E021–P4M-E027 |
| SD-P4M-004 | **ADD** /runtime show|set|fallback and RuntimePanel Test-backend canary. | 01-phase:L531-L541 | P4M-E023–P4M-E028 |
| SD-P4M-005 | **ADD** Tier C mandate, stdout sanitization, missing-binary Unavailable, exit_code fail-closed, RuntimeClient wiring. | 01-phase:L531-L541 | P4M-E018–P4M-E019, P4M-E031 |
| SD-P4M-006 | **ADD** [backend]→[runtime] deprecation, phase_4M.toml metrics, ouroboros grounding distinctions, R1–R4. | 01-phase:L533-L541 | P4M-E015, P4M-E029–P4M-E033 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **6/6 files**, **2,692/2,692 logical lines**, **165,923/165,923 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **278 unique atoms** = **249 explicit**, **006 inferred**, **019 ambiguous**, **004 conflicting/corrective**.
- Per-file atom counts: phase-0 **074**; phase-1C **045**; phase-3A **038**; phase-3J-BASELINE **022**; phase-4K **061**; phase-4M **038**. Sum **278**.
- Relational interaction records: **18** (`IX-001`–`IX-018`). Summary-delta records: **36** (`SD-*`). Ambiguity/conflict register: **12** (`AC-001`–`AC-012`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P0/P1C/P3A/P3JB/P4K/P4M-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: every distinct CLI verb, Tauri command, window/panel/control named in phase-4K/4M/0/3A was given its own SURFACE atom rather than folded into backend bullets.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the six exact DEEP-PHASE-001 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0483`; `output_bytes=057190`.
- Canonical normalized output SHA-256: 033443514e1e6d6029c5322136abdca3ea36e2975830975d40c200c5252be87b.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
