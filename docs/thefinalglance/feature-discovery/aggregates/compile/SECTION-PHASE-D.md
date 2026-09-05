# SECTION PHASE + D — ELAI exhaustive super feature list (compiled)

Compiled from `PHASE-AGGREGATE.md` and `D-AGGREGATE.md`. Plan-recorded status unless noted. Deep wins target state over summary (see Superseded notes). Atom IDs preserved.

## System: PHASE — Implementation-guide phase steps (walking skeleton through empire integration)

Family PHASE is the ordered implementation-guide corpus: phase step files under `docs/specs/guide/steps/` plus phase indexes. Units cover fork/strip (0), knowledge layers (1*), grounding (2*), orchestration (3*), and productization (4*).

### phase-0: fork, strip, restructure, walking skeleton

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-0.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-001 — Phase 0 nine-crate split, LlmProvider, minimal FSM, visibility, msgpack, contracts, entropy quarantine are substrate for later phases here.; PHASE-001/IX-002 — Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience.; PHASE-001/IX-003 — Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine.; PHASE-001/IX-005 — Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G.; PHASE-001/IX-013 — Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace.; PHASE-001/IX-018 — Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis.; PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-019 — Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness.; PHASE-007/IX-020 — Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml.

**Behaviors**

- **P0-E001** [E/STATUS]: Phase 0 is FORK AND STRIP: produce a walking skeleton; prerequisite is pre-phase-0 complete (Windows platform fixes). (`phase-0:L1-L2`)
- **P0-E002** [E/CAP]: Goal: claw-code-rust-port compiles as elai_cc with unnecessary code removed; one model (ollama), one FSM loop IDLE->EXECUTING->IDLE, one tool (file read); no new features. (`phase-0:L6-L9`)
- **P0-E003** [E/DEP]: Mandatory pre-read set: architecture-overview.txt §§1-4, core.txt §§3-4, providers.txt §2, base-fork-comparison.txt full. (`phase-0:L11-L16`)
- **P0-E004** [E/EVIDENCE]: R17: claw-code compilation verified 26.24s, 0 errors. (`phase-0:L18-L19`)
- **P0-E005** [E/LIMIT]: R18: module quality unknown; auditor reviews each module during strip. (`phase-0:L20-L20`)
- **P0-E006** [E/DATA]: R19: runtime is 25,035 LOC / 38 files; monoliths main.rs 8,615, commands/lib.rs 4,841, tools/lib.rs 7,345 LOC. (`phase-0:L21-L22`)
- **P0-E007** [E/DET]: R20: entropy quarantine scope is 60 occurrences across 31 files. (`phase-0:L23-L23`)
- **P0-E008** [E/EVIDENCE]: R21: base-fork-comparison LOC numbers are stale (green_contract listed 4,061, actual 152); always verify against actual files. (`phase-0:L24-L25`)
- **P0-E009** [E/INT]: R22: config.rs imports SandboxConfig and FilesystemIsolationMode; both must move to core/ with config.rs to avoid core→orchestrator cycle. (`phase-0:L26-L27`)
- **P0-E010** [E/DATA]: R23: TokenUsage/UsageTracker embedded in Session, conversation.rs, main.rs (61 occurrences / 5 files); stripping usage.rs is structural, not file-delete. (`phase-0:L28-L29`)
- **P0-E011** [E/INT]: R24: tools/lib.rs imports api::*; when api/ absorbs into providers/, tools imports must update simultaneously. (`phase-0:L30-L31`)
- **P0-E012** [E/SURFACE]: R25: commands/lib.rs has 28+ claw-code slash commands that conflict with elai_cc /task, /config, /continue. (`phase-0:L32-L33`)
- **P0-E013** [E/FLOW]: Design: strip before restructure removes ~4K+ LOC of known-bad code before splitting runtime into 5 crates, reducing move/break risk. (`phase-0:L37-L41`)
- **P0-E014** [E/DATA]: Workspace path migration: 0.1-0.3 work in .elai_cc/rust/; 0.4 creates .elai_cc/crates/ with .elai_cc/Cargo.toml workspace root; 0.5+ use that root; old rust/ is archived after 0.5 verifies. (`phase-0:L43-L47`)
- **P0-E015** [E/CFG]: ONNX models live at canonical path .elai_cc/models/onnx/; pre-staged via setup (download + blake2 verify + index_version.toml); never auto-downloaded. (`phase-0:L49-L52`)
- **P0-E016** [E/FAIL]: Missing ONNX models at startup enter DEGRADED mode BM25_ONLY, not crash. (`phase-0:L52-L53`)
- **P0-E017** [E/SEC]: Visibility filter hides .elai_cc/models/** from agents via default hidden_paths. (`phase-0:L53-L54`)
- **P0-E018** [E/DATA]: TokenUsage handling: strip usage.rs but define minimal core TokenUsage {input_tokens, output_tokens, total_tokens, estimated_cost_usd}; strip UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing (billing concepts). (`phase-0:L56-L62`)
- **P0-E019** [E/CAP]: Provider trait migration: keep claw-code Provider trait during Phase 0 marked #[deprecated]; OllamaProvider implements LlmProvider (complete, embed, token_count, token_count_estimate); Anthropic/OpenAI migrate to LlmProvider in Phase 1+. (`phase-0:L64-L69`)
- **P0-E020** [E/TEST]: 0.1 PRE_GATE: pre-phase-0 cargo check passes; cargo installed; .elai_cc/rust/Cargo.toml exists. (`phase-0:L74-L79`)
- **P0-E021** [E/OBS]: 0.1 records baseline metrics into phase_0_baseline.toml fields compile_time_s, test_pass, test_fail, test_total, loc_total, loc_by_crate. (`phase-0:L80-L91`)
- **P0-E022** [E/DATA]: 0.1 creates migration_manifest.toml classifying every .rs file KEEP/STRIP/REPLACE/EXTEND with source crate, target crate, rationale; must cover 69 files. (`phase-0:L84-L93`)
- **P0-E023** [E/TEST]: 0.1 adversarial: baseline only after clean git status (no uncommitted changes in .elai_cc/rust/). (`phase-0:L94-L96`)
- **P0-E024** [E/FLOW]: 0.2 strip order (each followed by cargo check): telemetry crate; compat-harness + call sites; structural usage/compact/summary_compression with TokenUsage shim; lsp_client; oauth DEFERRED; audit prompt.rs; classify slash commands only; evaluate remote.rs. (`phase-0:L109-L155`)
- **P0-E025** [E/DATA]: 0.2c TokenUsage temporary shim is {input_tokens, output_tokens} only; core::TokenUsage replaces it in 0.4a. (`phase-0:L127-L130`)
- **P0-E026** [E/SURFACE]: 0.2g KEEP candidates: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. (`phase-0:L146-L148`)
- **P0-E027** [E/SURFACE]: 0.2g STRIP claw-code commands: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. (`phase-0:L149-L150`)
- **P0-E028** [E/LIMIT]: Patterns sentinel, anti_distill, bias_inject do NOT exist (verified 2026-04-06); do not grep for them. (`phase-0:L157-L158`)
- **P0-E029** [E/TEST]: 0.2 checks: stripped files absent; no dangling compact/summary_compression/usage/lsp_client refs; interactive-agent prompt gone; slash audit in manifest; cargo check 0 errors. (`phase-0:L159-L173`)
- **P0-E030** [E/TEST]: 0.3 classify failing tests EXPECTED or BUG; fix BUGs; record strip delta loc_removed, files_removed, tests_removed, tests_broken_expected, tests_broken_bug. (`phase-0:L194-L204`)
- **P0-E031** [E/FAIL]: 0.3 flag if tests_removed >10% of baseline total as over-stripping signal. (`phase-0:L209-L210`)
- **P0-E032** [E/CAP]: 0.4a creates workspace root .elai_cc/Cargo.toml and crate dirs under .elai_cc/crates/; copies from intact .elai_cc/rust/. (`phase-0:L221-L223`)
- **P0-E033** [E/CAP]: 0.4a core/ extracts config.rs, session_control.rs, json.rs, bootstrap.rs plus sandbox types; defines core::TokenUsage. (`phase-0:L225-L236`)
- **P0-E034** [E/CAP]: 0.4a session_control identity stays in core/; Session state management stays in orchestrator/. (`phase-0:L234-L236`)
- **P0-E035** [E/CAP]: 0.4a rules/ extracts policy_engine.rs, green_contract.rs, trust_resolver.rs. (`phase-0:L239-L241`)
- **P0-E036** [E/CAP]: 0.4a providers/ absorbs all api/src/ plus runtime/sse.rs; mark old Provider #[deprecated]; update all use api:: including tools/lib.rs. (`phase-0:L243-L250`)
- **P0-E037** [E/LIMIT]: core/ must be a true leaf: no deps on orchestrator/plugins/cli. (`phase-0:L256-L257`)
- **P0-E038** [E/CAP]: 0.4b tools/ keeps existing tools crate + file_ops, bash, bash_validation, stale_branch, branch_lock; depends on core/ not runtime/. (`phase-0:L276-L282`)
- **P0-E039** [E/LIMIT]: tools/lib.rs split if >3 tools; no single .rs in tools/ exceeds 2000 LOC after restructure. (`phase-0:L280-L298`)
- **P0-E040** [E/CAP]: 0.4b plugins/ keeps existing + hooks.rs, plugin_lifecycle.rs; depends on core/. (`phase-0:L284-L287`)
- **P0-E041** [E/CAP]: 0.4b knowledge/ and reasoning/ empty skeleton crates for Phase 1/2 fill. (`phase-0:L289-L290`)
- **P0-E042** [E/CAP]: 0.4c orchestrator/ receives remaining runtime modules; depends on core/providers/tools/rules. (`phase-0:L309-L321`)
- **P0-E043** [E/FLOW]: 0.4c evaluate lane_events, task_packet, task_registry, team_cron_registry: KEEP if maps to elai_cc agents/FSM/sessions else STRIP. (`phase-0:L316-L319`)
- **P0-E044** [E/CAP]: 0.4c cli/ absorbs rusty-claude-cli + commands/; main.rs thin REPL target <500 LOC; strip claw-code commands per 0.2g. (`phase-0:L323-L334`)
- **P0-E045** [E/DATA]: 0.4c cleanup: archive .elai_cc/rust/; remove absorbed dirs; workspace members exactly 9 crates. (`phase-0:L336-L341`)
- **P0-E046** [E/CAP]: Canonical nine crates: core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli. (`phase-0:L346-L347`)
- **P0-E047** [E/TEST]: 0.5: cargo test --no-run; cargo test --workspace; classify RESTRUCTURE_BUG or PATH_UPDATE; phase_0_restructure.toml total_crates=9. (`phase-0:L372-L383`)
- **P0-E048** [E/TEST]: 0.6 contracts crate, 5 tests provider_trait/fsm_basic/tool_basic/config_basic/visibility_filter; TDD red (compile, fail runtime). (`phase-0:L402-L428`)
- **P0-E049** [E/CAP]: 0.7a LlmProvider: complete (streaming Token), embed, token_count, token_count_estimate; all take &self. (`phase-0:L444-L449`)
- **P0-E050** [E/CAP]: 0.7a OllamaProvider: POST /api/generate and /api/embed; base_url default http://localhost:11434; NDJSON streaming. (`phase-0:L450-L453`)
- **P0-E051** [E/CFG]: 0.7a elai_cc.toml in core/: provider + project + agent role configs; toml parse; msgpack rmp-serde round-trip. (`phase-0:L454-L455`)
- **P0-E052** [E/STATE]: 0.7b minimal FSM IDLE/EXECUTING; invalid transitions return Error not panic; get_state, transition, get_active_tools. (`phase-0:L479-L482`)
- **P0-E053** [E/CAP]: 0.7b file_read: contents + total_lines + estimated_tokens; hidden/symlink-hidden return NotFound identical to missing. (`phase-0:L483-L499`)
- **P0-E054** [E/DATA]: 0.7b session storage msgpack via rmp-serde; verified by round-trip and valid msgpack on disk. (`phase-0:L486-L494`)
- **P0-E055** [E/SURFACE]: 0.7c slash commands /task, /config, /continue, /explore, /status; bare messages fallback to /task; unknown command error. (`phase-0:L517-L519`)
- **P0-E056** [E/SEC]: 0.7c visibility: allowed_paths + hidden_paths; canonicalize; defaults hide models/**, rules.db, harness, checkpoints, SSB, binary internals. (`phase-0:L520-L526`)
- **P0-E057** [E/SEC]: 0.7c error and tool-schema sanitization allowlist-based (not denylist). (`phase-0:L527-L529`)
- **P0-E058** [E/OBS]: 0.7c tracing + tracing-subscriber replace telemetry; log tool calls, FSM transitions, file ops. (`phase-0:L530-L531`)
- **P0-E059** [E/TEST]: 0.8 smoke_test_ollama.sh WARN not BLOCK if ollama unavailable. (`phase-0:L558-L564`)
- **P0-E060** [E/CAP]: 0.9 ClockProvider: now, elapsed_since, system_now; &self. (`phase-0:L581-L586`)
- **P0-E061** [E/CAP]: 0.9 EntropySource: next_u64, fill_bytes; &mut self. (`phase-0:L587-L590`)
- **P0-E062** [E/CAP]: 0.9 OsClock/OsRng production; SeededClock (explicit tick) and SeededRng (ChaCha8Rng seed_from_u64) for tests. (`phase-0:L592-L597`)
- **P0-E063** [E/INT]: 0.9 RuntimeContext bundles clock Arc and entropy Arc Mutex. (`phase-0:L606-L608`)
- **P0-E064** [E/LIMIT]: Raw time only core/src/clock.rs; raw RNG only core/src/entropy.rs; CI check_entropy_quarantine.sh excludes those and TEST_KEEP. (`phase-0:L609-L615`)
- **P0-E065** [E/TEST]: 0.10 determinism: ELAI_SEED=42, --test-threads=1, two runs stdout diff=0; required for Phase 3I DST. (`phase-0:L647-L654`)
- **P0-E066** [E/DATA]: MODULE MAPPING authoritative runtime→crate destinations with verified 2026-04-06 LOCs; SSE dedup keep api/sse strip runtime/sse. (`phase-0:L666-L686`)
- **P0-E067** [E/OBS]: Measurements: compile <30s target; test counts; LOC; strip delta; ollama latency; raw entropy outside core=0. (`phase-0:L748-L754`)
- **P0-E068** [E/EVIDENCE]: Grounding: 684 tests, 61,317 LOC, 69 .rs files verified; custom clippy UNGROUNDED (CI grep); quarantine ~200 LOC estimate UNGROUNDED vs 60/31 measured. (`phase-0:L757-L766`)
- **P0-I001** [I/DATA]: estimated_cost_usd retained on core TokenUsage despite non-billing design; must not drive claw-code billing paths. (`phase-0:L56-L61`)
- **P0-A001** [A/CAP]: prompt.rs REPLACE vs STRIP open. (`phase-0:L142-L144`)
- **P0-A002** [A/DEFER]: remote.rs DEFERRED vs STRIP open. (`phase-0:L155-L155`)
- **P0-A003** [A/CAP]: sandbox whole-file vs types-only extraction open. (`phase-0:L228-L231`)
- **P0-A004** [A/CAP]: cli/main.rs <500 LOC soft target; else document remaining LOC. (`phase-0:L332-L333`)
- **P0-C001** [C/DATA]: TokenUsage field set conflicts: 0.2c shim 2 fields vs design block 4 fields including total_tokens and estimated_cost_usd. (`phase-0:L56-L61, L127-L130`)

### phase-1C: dual-model ONNX vector search

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1C.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-002 — Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience.; PHASE-001/IX-003 — Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine.; PHASE-001/IX-004 — Phase 1C SSB file_write invalidation and knowledge_degradation require Phase 3B SSB (outside chunk) but specify production wiring.; PHASE-001/IX-006 — Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG.; PHASE-001/IX-016 — RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here.; PHASE-001/IX-017 — 1C three-ranker RRF stays additive for later SPLADE/cross-encoder; semaphore overflow unranked.

**Behaviors**

- **P1C-E001** [E/STATUS]: Phase 1C requires Phase 1B; dual GraphCodeBERT/Granite ONNX embeddings, SQLite vector cache, per-model breakers, startup canary. (`phase-1C:L1-L9`)
- **P1C-E002** [E/DEP]: Pre-read knowledge.txt §§2,3,5-7,11,13; providers.txt §§2,9; architecture §10; risks R22,R26,R27,R29; usearch decision finalized in phase-1C-decisions.md. (`phase-1C:L11-L17`)
- **P1C-E003** [E/CFG]: R22: ONNX GPU1 when executor/auditor on GPU0; else CPU-only; ModelPool.can_fit_onnx before allocation. (`phase-1C:L20-L23`)
- **P1C-E004** [E/RECOVERY]: R26 cascade HYBRID -> SPARSE_ONLY -> BM25_ONLY; never total knowledge outage. (`phase-1C:L25-L27`)
- **P1C-E005** [E/FAIL]: R27: canary embed("fn main()") within 5s. (`phase-1C:L29-L30`)
- **P1C-E006** [E/CONC]: R29: cross-encoder Semaphore(1) max queue depth 4. (`phase-1C:L32-L33`)
- **P1C-E007** [E/DEFER]: SPLADE deferred from 1C; dual-vector only; later additive 4th RRF ranker. (`phase-1C:L37-L47`)
- **P1C-E008** [E/DATA]: CODE GraphCodeBERT 768d; TEXT Granite-embedding-small 384d. (`phase-1C:L52-L53`)
- **P1C-E009** [E/DATA]: vector_cache: content_blake2s BLOB PK, code_vec, text_vec, cached_at INTEGER. (`phase-1C:L55-L55`)
- **P1C-E010** [E/CFG]: Breaker: 3 failures / 60s open; half-open after 30s. (`phase-1C:L57-L57`)
- **P1C-E011** [E/DATA]: Cache hit rate >90% incremental re-index; invalidate on SSB file_write. (`phase-1C:L60-L61`)
- **P1C-E012** [E/RECOVERY]: Code fail->TEXT-only; Granite fail->CODE-only; both->SPARSE_ONLY. (`phase-1C:L62-L62`)
- **P1C-E013** [E/CFG]: ONNX+LLM GPU multiplexing only dual-GPU; single GPU = CPU-only ONNX. (`phase-1C:L63-L63`)
- **P1C-E014** [E/DEP]: Pinned ort, usearch v2.24.0, blake2 Blake2s256; do NOT add fastembed with ort. (`phase-1C:L65-L77, L114-L121`)
- **P1C-E015** [E/DEFER]: TurboQuant grounded (~10.7× b=3) but 1C stores raw f32; quant deferred to 1G+. (`phase-1C:L79-L92, L365-L367`)
- **P1C-E016** [E/LIMIT]: Ungrounded until 3.5: tensor formats; embed throughput; dual-vector vs BM25 accuracy. (`phase-1C:L94-L100`)
- **P1C-E017** [E/CAP]: Modules vector/{mod,cache,ort_session,circuit_breaker,hnsw_index}; DualVector; stubs Err(NotReady). (`phase-1C:L157-L162`)
- **P1C-E018** [E/CAP]: OrtSession Arc<Mutex<Session>>; max_seq_len=512; truncate+WARN; path .elai_cc/models/onnx/; missing ModelNotFound->BM25_ONLY. (`phase-1C:L163-L178`)
- **P1C-E019** [E/CFG]: Canonical ONNX path configurable [knowledge.onnx_model_path]. (`phase-1C:L172-L172`)
- **P1C-E020** [E/CAP]: CircuitBreaker CLOSED/OPEN/HALF_OPEN; 3/60s; half-open 30s. (`phase-1C:L179-L180`)
- **P1C-E021** [E/EVENT]: SSB file_write DELETE vector_cache by path; registered at KnowledgeEngine startup. (`phase-1C:L181-L183, L397-L398`)
- **P1C-E022** [E/CAP]: UsearchIndex Cos; 768+384 indexes; add/search/filtered_search/save/load; rebuild from cache if load fails. (`phase-1C:L184-L191, L379-L385`)
- **P1C-E023** [E/DEP]: Chunking assumed as embed input; if undefined add AST tree-sitter sub-step. (`phase-1C:L193-L197`)
- **P1C-E024** [E/TEST]: ≥22 unit tests: dims, cache, breaker, degradation, query validation, canary, path, missing model, semaphore, cache-hit, truncation, HNSW, isolation, concurrent embed, RRF k=60. (`phase-1C:L254-L287, L321-L324`)
- **P1C-E025** [E/TEST]: Contracts: dual-vector HYBRID scores; project isolation via separate knowledge.db files (column filter FAIL). (`phase-1C:L288-L294`)
- **P1C-E026** [E/DET]: Breaker uses ClockProvider not std::time. (`phase-1C:L316-L319, L377-L378`)
- **P1C-E027** [E/EVIDENCE]: Step 3.5 tensor gate; tensor_format_verified:true in phase-1C-decisions.md; tokenizer crate if missing. (`phase-1C:L330-L356`)
- **P1C-E028** [E/CAP]: Step 4 embed; VectorCache via single-writer SSB channel for SQLite writes. (`phase-1C:L370-L376`)
- **P1C-E029** [E/CAP]: readiness_check canary within 5s doubles as ONNX warmup before Ready. (`phase-1C:L386-L390`)
- **P1C-E030** [E/FLOW]: RRF BM25+code+text; score sum 1/(60+rank); k=60 fixed; CodeChunk scores fields. (`phase-1C:L391-L396`)
- **P1C-E031** [E/CONC]: Cross-encoder Semaphore(1) queue 4; overflow returns unranked not error. (`phase-1C:L399-L400`)
- **P1C-E032** [E/EVENT]: SearchMode change emits knowledge_degradation SSB (breaker transitions). (`phase-1C:L401-L403`)
- **P1C-E033** [E/OBS]: QueryTrace DEBUG: query_id, stages, total_latency_ms, mode, result_count. (`phase-1C:L404-L405`)
- **P1C-E034** [E/LIMIT]: vector/ code: no std::time, unwrap, TODO; no TurboQuant in 1C. (`phase-1C:L409-L411, L455-L455`)
- **P1C-E035** [E/DET]: Equal cosine ties post-sort score DESC then chunk_id ASC. (`phase-1C:L425-L428`)
- **P1C-E036** [E/CONC]: SSB DELETE for file_write must complete before query for post-event requests. (`phase-1C:L433-L437`)
- **P1C-E037** [E/OBS]: Step 5: 100-chunk throughput; dual vs BM25; cache hit ≥90% BLOCK if under; phase-1C-measurements.md; dual≤BM25 INFO. (`phase-1C:L469-L504`)
- **P1C-E038** [E/DATA]: Step 5 also names .elai_cc/models/graphcodebert/ and granite-embedding-small/ paths. (`phase-1C:L463-L465`)
- **P1C-E039** [E/OBS]: Also measure warmup first-query latency; canary <5s; cross-encoder p99 <50ms/50 candidates. (`phase-1C:L524-L531`)
- **P1C-I001** [I/CFG]: RRF k=60 const implies no operator-configurable fusion without code change. (`phase-1C:L429-L432`)
- **P1C-A001** [A/DATA]: Exact tensor names/shapes open until Step 3.5. (`phase-1C:L94-L97, L337-L344`)
- **P1C-A002** [A/DEP]: Tokenizer crate choice open until 3.5/4. (`phase-1C:L346-L348`)
- **P1C-A003** [A/FLOW]: Chunking ownership unresolved if earlier phases silent. (`phase-1C:L193-L197`)
- **P1C-C001** [C/CFG]: Model path .elai_cc/models/onnx/ vs Step 5 per-model subdirs. (`phase-1C:L169-L172, L463-L465`)
- **P1C-C002** [C/DEFER]: Source defers SPLADE to 1G; MASTER summary says 1H. (`phase-1C:L37-L47`)

### phase-3A: full interactive FSM

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 3 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3A.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-005 — Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G.; PHASE-001/IX-006 — Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG.; PHASE-001/IX-007 — Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal.; PHASE-001/IX-008 — Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI.; PHASE-001/IX-014 — Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback.; PHASE-001/IX-018 — Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis.

**Behaviors**

- **P3A-E001** [E/STATUS]: Prereq Phase 0-2; full 9-state FSM + tool/rule loading + evidence gates + INTERACTIVE HITL; AUTONOMOUS deferred 4G. (`phase-3A:L1-L12`)
- **P3A-E002** [E/STATE]: Chain IDLE->PLANNING->FEASIBILITY_PROBE->TESTING_FIRST->EXECUTING->AUDITING->MEMORIZING->COMPLETE->IDLE. (`phase-3A:L9-L11`)
- **P3A-E003** [E/CAP]: Deliverables 3A.1–3A.5: machine, filters, 7 gates, InteractiveGate, risk levels. (`phase-3A:L23-L27`)
- **P3A-E004** [E/STATE]: FsmState: IDLE PLANNING FEASIBILITY_PROBE TESTING_FIRST EXECUTING AUDITING MEMORIZING COMPLETE CONFIGURING. (`phase-3A:L39-L40`)
- **P3A-E005** [E/FLOW]: Valid matrix: IDLE->PLANNING (`CONFIGURING; PLANNING->FEASIBILITY_PROBE`)
- **P3A-E006** [E/DEFER]: FEASIBILITY_PROBE full impl deferred 4F; transitions remain valid. (`phase-3A:L44-L46, L169-L172`)
- **P3A-E007** [E/EVIDENCE]: AUDITING->EXECUTING/PLANNING PARTIALLY_GROUNDED SKILL.md not orchestrator.txt §2; SPEC_UPDATE_NEEDED. (`phase-3A:L51-L56, L332-L335`)
- **P3A-E008** [E/SURFACE]: CONFIGURING via /config or /workflow; config_paths_only tools; no evidence gate; ->IDLE on completion. (`phase-3A:L60-L63`)
- **P3A-E009** [E/DATA]: Transition carries source, target, phase label, timestamp, Vec<EvidenceArtifact>. (`phase-3A:L64-L65`)
- **P3A-E010** [E/CAP]: token_budget_remaining()->usize for Phase 3E. (`phase-3A:L66-L68`)
- **P3A-E011** [E/DATA]: Persist .elai_cc/sessions/{sid}/.fsm-state.msgpack. (`phase-3A:L69-L69`)
- **P3A-E012** [E/FLOW]: ReplanRequest MINOR (no human) and MAJOR (human); max 1 per agent per phase. (`phase-3A:L70-L71`)
- **P3A-E013** [E/FAIL]: Typed errors InvalidTransition, CorruptState, ReplanLimitExceeded, TransitionConflict. (`phase-3A:L79-L88`)
- **P3A-E014** [E/CONC]: Concurrent transitions: one succeeds, other TransitionConflict. (`phase-3A:L86-L88`)
- **P3A-E015** [E/CAP]: Per-state tool filters: PLANNING no edit/bash; FEASIBILITY stub write <50+bash+web; TESTING_FIRST no edit/bash; EXECUTING no planning; AUDITING no edit/planning; MEMORIZING memory-bank only; CONFIGURING config paths; IDLE all; COMPLETE read-only. (`phase-3A:L110-L123`)
- **P3A-E016** [E/LIMIT]: COMPLETE read-only and IDLE unrestricted UNGROUNDED vs orchestrator.txt §2. (`phase-3A:L121-L123, L141-L142`)
- **P3A-E017** [E/FAIL]: ToolRegistry::execute returns ToolBlockedByState for blocked tools. (`phase-3A:L125-L126`)
- **P3A-E018** [E/CAP]: get_active_rules(state) via RuleEngine. (`phase-3A:L124-L124`)
- **P3A-E019** [E/FLOW]: 7 gates: knowledge READY (`DEGRADED; zero ungrounded; tests fail right reasons; ≥1 FAIL->PASS; MINOR replan; zero BLOCKs; DKG+changelog.`)
- **P3A-E020** [E/CAP]: EvidenceGate trait; EvidenceError {gate_name, missing_artifacts, suggestion}. (`phase-3A:L164-L166`)
- **P3A-E021** [E/RETRY]: DAIRA: if output >budget*0.3 reduce scope retry; max 3. (`phase-3A:L167-L168`)
- **P3A-E022** [E/DEFER]: FEASIBILITY transitions ungated in 3A; full gating 4F. (`phase-3A:L169-L172`)
- **P3A-E023** [E/FAIL]: DEGRADED allows IDLE->PLANNING with warning; UNAVAILABLE blocks. (`phase-3A:L183-L185, L196-L196`)
- **P3A-E024** [E/SURFACE]: INTERACTIVE gate: structured EvidenceSummary present_gate; not prose. (`phase-3A:L205-L209`)
- **P3A-E025** [E/FLOW]: GateDecision APPROVE/REJECT(no change)/MODIFY(recheck)/REDIRECT(valid target). (`phase-3A:L210-L214`)
- **P3A-E026** [E/CAP]: InteractiveGate trait; CLI impl; MockInteractiveGate tests. (`phase-3A:L215-L216`)
- **P3A-E027** [E/CFG]: Timeout configurable; default none INTERACTIVE; AUTONOMOUS timeout 4G. (`phase-3A:L217-L218`)
- **P3A-E028** [E/STATE]: RiskLevel LOW/MEDIUM/HIGH on transitions; examples LOW MEMORIZING->next; MEDIUM PLANNING->TESTING_FIRST; HIGH destructive. (`phase-3A:L251-L257`)
- **P3A-E029** [E/FLOW]: INTERACTIVE: LOW auto-advance; MEDIUM/HIGH pause. (`phase-3A:L255-L261`)
- **P3A-E030** [E/CFG]: [fsm.risk_overrides] in elai_cc.toml. (`phase-3A:L258-L259`)
- **P3A-E031** [E/TEST]: 9-state msgpack; 81 transition pairs; per-state tool tests; 7 gate tests; 4 decision tests; risk tests; replan limit; proptest; adversarial. (`phase-3A:L282-L303`)
- **P3A-E032** [E/OBS]: SLOs: transition <1ms; evidence gate <10ms; tool filter <0.1ms; human latency log-only. (`phase-3A:L305-L309`)
- **P3A-E033** [E/DEP]: SSB phase_transition not a 3A deliverable; emits after 3B. (`phase-3A:L342-L343`)
- **P3A-I001** [I/CAP]: EvidenceGate/InteractiveGate traits ungrounded testability assumptions. (`phase-3A:L191-L192, L236-L237`)
- **P3A-A001** [A/FLOW]: REDIRECT invalid-target handling partially specified. (`phase-3A:L230-L232`)
- **P3A-A002** [A/CFG]: Default RiskLevel for every transition beyond examples not fully tabulated. (`phase-3A:L251-L254`)
- **P3A-A003** [A/DEP]: SKILL.md path for AUDITING recovery unresolved among multiple skill files. (`phase-3A:L51-L56`)
- **P3A-C001** [C/EVIDENCE]: orchestrator.txt §2 omits AUDITING recovery arms SKILL.md requires; SPEC_UPDATE_NEEDED. (`phase-3A:L51-L56`)

### phase-3J-BASELINE: token efficiency benchmark harness

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3J-BASELINE.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-010 — Phase 3J-BASELINE fills capability_profile.toml for ModelCapabilityProfile without booting 3A/4K.; PHASE-001/IX-011 — 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability.

**Behaviors**

- **P3JB-E001** [E/STATUS]: Standalone token-economy harness for every ModelCapabilityProfile tier; writes metrics/capability_profile.toml for runtime slot/compression/budget. (`phase-3J-BASELINE:L1-L10`)
- **P3JB-E002** [E/LIMIT]: Must not boot elai runtime (no SSB/DKG/orchestrator); fixture TOML + HTTP endpoint only. (`phase-3J-BASELINE:L8-L10`)
- **P3JB-E003** [E/CFG]: Tiers: micro_8gb, micro_3b_12gb, micro_7b_12gb, awq_3b_8gb, awq_3b_12gb, awq_7b_12gb, large_32b_48gb, cloud_200k. (`phase-3J-BASELINE:L12-L21`)
- **P3JB-E004** [E/CAP]: New baseline_harness.py; does not overwrite run_benchmark.py. (`phase-3J-BASELINE:L24-L27`)
- **P3JB-E005** [E/DATA]: Fixtures fixtures/*.toml; roster elai_cc.toml [model_capability]; endpoint /v1/messages; output capability_profile.toml. (`phase-3J-BASELINE:L28-L31`)
- **P3JB-E006** [E/DATA]: Fixture ≥10 prompts/tier; schema tier + prompts[{id,text,expect_json}]. (`phase-3J-BASELINE:L41-L47`)
- **P3JB-E007** [E/OBS]: Per prompt: tokens_in/out, slot_utilization_pct, compression_ratio, ttft_ms, p99_latency_ms, cost_per_run_usd. (`phase-3J-BASELINE:L49-L56`)
- **P3JB-E008** [E/TEST]: One output row per tier from [model_capability]. (`phase-3J-BASELINE:L61-L63`)
- **P3JB-E009** [E/TEST]: RTX 3090 targets: micro_7b_12gb util≥0.82 compression≥0.65; cloud_200k util≥0.90. (`phase-3J-BASELINE:L65-L67`)
- **P3JB-E010** [E/TEST]: REPAIR_CONTEXT twice same prompts: tokens_in must not rise >15%; else repair_context_regression row blocks plans. (`phase-3J-BASELINE:L69-L74`)
- **P3JB-E011** [E/TEST]: JSON-schema 20 expect_json prompts: 0 invalid JSON under ConstrainedDecodingConfig. (`phase-3J-BASELINE:L76-L79`)
- **P3JB-E012** [E/DEFER]: EBNF/grammar constrained decoding deferred; sidecar JSON-schema-only via LMFE. (`phase-3J-BASELINE:L80-L86`)
- **P3JB-E013** [E/LIMIT]: stdlib-only harness imports; no requests; no in-tree crates. (`phase-3J-BASELINE:L91-L93`)
- **P3JB-E014** [E/LIMIT]: No ShortCoder import; compression_ratio_estimate via output/input ratio measurement_method=output_token_ratio. (`phase-3J-BASELINE:L94-L100`)
- **P3JB-E015** [E/FAIL]: Exit 0 pass; 1 gate fail; 2 harness error. (`phase-3J-BASELINE:L103-L104`)
- **P3JB-E016** [E/DATA]: Produces harness, capability_profile.toml, 8 tier fixtures, json_schema.toml. (`phase-3J-BASELINE:L109-L113`)
- **P3JB-E017** [E/INT]: Manual python invoke; optional non-blocking CI job bench-capability. (`phase-3J-BASELINE:L118-L120`)
- **P3JB-E018** [E/SURFACE]: CLI: python baseline_harness.py --profile micro_7b_12gb; expect exit 0 and util≥0.82. (`phase-3J-BASELINE:L125-L128`)
- **P3JB-E019** [E/TEST]: Deterministic checks: no forbidden imports; fixture/template present; model_capability sections counted. (`phase-3J-BASELINE:L130-L145`)
- **P3JB-I001** [I/OBS]: Eight tiers imply eight fixtures though only micro_7b named in checks. (`phase-3J-BASELINE:L12-L21, L112-L112`)
- **P3JB-A001** [A/CFG]: Prompt content/length/seed for ≥10 prompts unspecified. (`phase-3J-BASELINE:L41-L47`)
- **P3JB-A002** [A/OBS]: Token-ratio compression vs runtime ShortCoder tolerance unspecified. (`phase-3J-BASELINE:L53-L53, L94-L100`)

### phase-4K: native Tauri dashboard MVP

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-4K.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-007 — Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal.; PHASE-001/IX-008 — Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI.; PHASE-001/IX-009 — Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface.; PHASE-001/IX-013 — Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace.

**Behaviors**

- **P4K-E001** [E/STATUS]: Prereq 3A/3B/3J; 4A-4J recommended; dashboard reads SSB. (`phase-4K:L1-L4`)
- **P4K-E002** [E/SURFACE]: Native structured control surface not chatbot; inputs via FSM classification; MVP interactive mode. (`phase-4K:L8-L12`)
- **P4K-E003** [E/CAP]: Tauri 2.10 / build 2.5; SsbViewer commands; Svelte; no HTTP/REST/WebSocket/CORS/ports. (`phase-4K:L14-L17`)
- **P4K-E004** [E/FLOW]: Frontend polls invoke() every 500ms; cargo tauri build single .exe WebView2. (`phase-4K:L18-L19`)
- **P4K-E005** [E/EVIDENCE]: tauri 2.10.3, tauri-build 2.5.6, tauri-cli 2.10.1 verified 2026-04-16. (`phase-4K:L21-L26`)
- **P4K-E006** [E/SURFACE]: Visual: teal #1B7A6E on charcoal #0f1318; JetBrains Mono; density over whitespace. (`phase-4K:L28-L31`)
- **P4K-E007** [E/FAIL]: SSB schema drift mitigated by typed SsbViewer compile-time deserialization. (`phase-4K:L40-L43`)
- **P4K-E008** [E/DEP]: WebView2 required; bootstrapper in installer; README + clean Win11 test. (`phase-4K:L44-L48`)
- **P4K-E009** [E/LIMIT]: Bundle 3-8MB; frontend <100KB JS. (`phase-4K:L49-L52`)
- **P4K-E010** [E/LIMIT]: 500ms poll accepted; later app.emit push upgrade path. (`phase-4K:L55-L59`)
- **P4K-E011** [E/CAP]: SsbViewer SQLITE_OPEN_READONLY WAL connection separate from writer. (`phase-4K:L70-L72`)
- **P4K-E012** [E/CAP]: Queries: latest_fsm_state, active_session, phase_progress, recent_actions, agent_actions, pending_interventions, file_changes, reasoning_chain. (`phase-4K:L73-L81`)
- **P4K-E013** [E/EVENT]: INTERVENTION_REQUEST: step_id, type ApproveChange (`ReplanApproval`)
- **P4K-E014** [E/EVENT]: INTERVENTION_RESPONSE: request_id, chosen, optional user_note. (`phase-4K:L87-L88`)
- **P4K-E015** [E/DEFER]: Post-MVP: HEALTH_SNAPSHOT, WINDOW_STATE, health_snapshot/window_state/cost_summary queries. (`phase-4K:L89-L91`)
- **P4K-E016** [E/DATA]: SsbActionSummary first 100 chars; list pagination offset+limit; query <10ms on 100K rows. (`phase-4K:L92-L95`)
- **P4K-E017** [E/CAP]: dashboard crate structure: Cargo.toml, tauri.conf.json, build.rs, main.rs, commands.rs, src-frontend Svelte/Vite. (`phase-4K:L118-L133`)
- **P4K-E018** [E/CFG]: identifier dev.elai.dashboard; window ELAI 1200×800 min 900×600; dist path; devUrl http://localhost:1420; bundle.active. (`phase-4K:L134-L139`)
- **P4K-E019** [E/SURFACE]: Tauri command get_fsm_state. (`phase-4K:L140-L141`)
- **P4K-E020** [E/SURFACE]: Tauri command get_phase_progress. (`phase-4K:L142-L142`)
- **P4K-E021** [E/SURFACE]: Tauri command get_recent_actions(limit,offset). (`phase-4K:L143-L143`)
- **P4K-E022** [E/SURFACE]: Tauri command get_pending_interventions. (`phase-4K:L144-L144`)
- **P4K-E023** [E/SURFACE]: Tauri command get_file_changes(session_id). (`phase-4K:L145-L145`)
- **P4K-E024** [E/SURFACE]: Tauri command submit_intervention_response(request_id,chosen,note). (`phase-4K:L146-L146`)
- **P4K-E025** [E/INT]: SsbViewer as tauri::State; generate_handler registers six MVP commands. (`phase-4K:L149-L156`)
- **P4K-E026** [E/SURFACE]: CLI elai --dashboard; focus existing window if already running. (`phase-4K:L157-L158`)
- **P4K-E027** [E/SURFACE]: CSS tokens bg-void/base/surface/hover/input; teal/purple scales; semantic pass/warn/fail/info; mono font scale xs–xl. (`phase-4K:L159-L167`)
- **P4K-E028** [E/FAIL]: SsbViewer open fail: dialog + graceful exit; commands never unwrap. (`phase-4K:L177-L181`)
- **P4K-E029** [E/SURFACE]: Project selector: logo, subtitle "a deterministic development engine", two creation paths. (`phase-4K:L197-L203`)
- **P4K-E030** [E/SURFACE]: + New Project: name+parent picker; create_dir_all; list add; template+task without leaving app. (`phase-4K:L204-L208`)
- **P4K-E031** [E/SURFACE]: Open Existing: native folder dialog. (`phase-4K:L209-L210`)
- **P4K-E032** [E/SURFACE]: Recent projects: name/path/last opened; max 10; hover style; source elai_cc.toml [projects] or ~/.elai_cc/projects.json. (`phase-4K:L211-L214`)
- **P4K-E033** [E/SURFACE]: Commands list_projects, select_project, create_project; name alnum-_; inline red errors. (`phase-4K:L215-L224`)
- **P4K-E034** [E/SURFACE]: Template picker Svelte/React/Astro/Vanilla from templates/web/manifest.toml; list_templates. (`phase-4K:L225-L229`)
- **P4K-E035** [E/SURFACE]: Task input: header, placeholder, Start purple, Enter submit; submit_task command. (`phase-4K:L230-L237`)
- **P4K-E036** [E/FAIL]: Start disabled if empty; minimum 5 characters. (`phase-4K:L262-L263`)
- **P4K-E037** [E/SURFACE]: Workspace: plan tree left 320px + activity feed right; draggable divider; min 280px; double-click 50/50. (`phase-4K:L277-L280`)
- **P4K-E038** [E/SURFACE]: Status bar 24px: project (`FSM state`)
- **P4K-E039** [E/SURFACE]: Plan tree phases/steps/baby-steps; --> active; done/active/pending styles; expand active; elapsed times. (`phase-4K:L283-L291`)
- **P4K-E040** [E/SURFACE]: Feed [reasoning] teal-ghost collapsed 2-3 lines. (`phase-4K:L294-L295`)
- **P4K-E041** [E/SURFACE]: Feed [edit] path+lines +N -M. (`phase-4K:L296-L296`)
- **P4K-E042** [E/SURFACE]: Feed [validate] checkmark/x one-liner. (`phase-4K:L297-L297`)
- **P4K-E043** [E/SURFACE]: Feed [test] pass/fail counts. (`phase-4K:L298-L298`)
- **P4K-E044** [E/SURFACE]: Feed [preview] teal link opens system browser via shell::open. (`phase-4K:L299-L301, L362-L368`)
- **P4K-E045** [E/SURFACE]: Step headers; auto-scroll; Jump to latest pill when scrolled up. (`phase-4K:L302-L305`)
- **P4K-E046** [E/FLOW]: Poll 500ms phase_progress+recent_actions; skip if in-flight. (`phase-4K:L306-L308, L325-L327`)
- **P4K-E047** [E/DATA]: Map SSB types reasoning_trace, file_write, tool_result, test_outcome, phase_complete, preview_available. (`phase-4K:L309-L311`)
- **P4K-E048** [E/LIMIT]: Render last 500 entries; older lazy-load; virtualization post-MVP. (`phase-4K:L322-L324`)
- **P4K-E049** [E/SURFACE]: Diff viewer on ApproveChange: unified teal add / fail remove / sticky header. (`phase-4K:L343-L350`)
- **P4K-E050** [E/SURFACE]: Diff Accept / Decline / Retry+note; per-hunk Accept/Decline on hover. (`phase-4K:L351-L355`)
- **P4K-E051** [E/SURFACE]: Multi-file vertical list navigation. (`phase-4K:L356-L357`)
- **P4K-E052** [E/FLOW]: Responses write INTERVENTION_RESPONSE; close viewer; feed [accepted]/[declined]. (`phase-4K:L358-L361`)
- **P4K-E053** [E/SURFACE]: Tauri command get_action_detail(action_id) full payload. (`phase-4K:L369-L372`)
- **P4K-E054** [E/FAIL]: Truncate long lines 200 chars; multi-intervention badge Previous/Next. (`phase-4K:L382-L390`)
- **P4K-E055** [E/RECOVERY]: Partial hunk accept may fail compile → new intervention with error context. (`phase-4K:L385-L387`)
- **P4K-E056** [E/TEST]: Completion criteria: native window, project/template/task, plan+feed, per-hunk diffs, preview, <10ms queries, <1s launch. (`phase-4K:L400-L409`)
- **P4K-E057** [E/DEFER]: Post-MVP MONITOR/EXPLORE/notifications/tabs/polish/sidebar/slash palette/shortcuts/autonomous/push/cost_summary. (`phase-4K:L412-L427`)
- **P4K-I001** [I/SURFACE]: PhaseStatus and InterventionRequest fields imply plan-tree/card data contracts. (`phase-4K:L76-L76, L83-L86`)
- **P4K-A001** [A/INT]: submit_task "writes SSB / triggers orchestrator" path underspecified. (`phase-4K:L235-L237`)
- **P4K-A002** [A/DATA]: projects list dual source without precedence. (`phase-4K:L214-L214`)
- **P4K-A003** [A/SURFACE]: Diff viewer replace feed vs modal open. (`phase-4K:L345-L345`)

### phase-4M: multi-runtime backend abstraction

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-4M.txt`; atoms in `PHASE-001 / DEEP-PHASE-001.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-001/IX-011 — 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability.; PHASE-001/IX-012 — Phase 4M migrates 3J backends and Ollama/vLLM into RuntimeAdapter; requires 4A Tier C for CLI adapters.; PHASE-001/IX-013 — Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace.; PHASE-001/IX-014 — Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback.; PHASE-001/IX-016 — RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here.

**Behaviors**

- **P4M-E001** [E/STATUS]: NEW 2026-05-10; prereq 3J+4A; 4G recommended. (`phase-4M:L1-L4`)
- **P4M-E002** [E/CAP]: One RuntimeAdapter + adapters + resolve_backend; Anthropic-shaped messages; adapters translate. (`phase-4M:L8-L24`)
- **P4M-E003** [E/LIMIT]: Non-goals: no new runtimes; no keyword skills; no LiteLLM; extend LlmProvider. (`phase-4M:L32-L39`)
- **P4M-E004** [E/STATE]: RuntimeBackend 11: Sidecar AnthropicApi ClaudeCode CodexCli OpenCode Hermes GeminiCli KiroCli CopilotCli Ollama Vllm. (`phase-4M:L60-L72`)
- **P4M-E005** [E/CAP]: RuntimeAdapter: complete, stream, capabilities, health_check; Send+Sync. (`phase-4M:L74-L93`)
- **P4M-E006** [E/CAP]: capabilities: tools/streaming/vision/max_context/max_output/json_schema_mode/prefill/caching/thinking_budget. (`phase-4M:L87-L90`)
- **P4M-E007** [E/DATA]: RuntimeFactoryConfig: backend, cli_path, base_url, api_key, model, permission_mode, profile, cwd, timeouts. (`phase-4M:L99-L110`)
- **P4M-E008** [E/CFG]: PermissionMode AcceptEdits (`Plan`)
- **P4M-E009** [E/FAIL]: ResolveBackendError {candidate, supported full list}. (`phase-4M:L119-L122`)
- **P4M-E010** [E/CFG]: PerRoleRuntime {role,backend,fallback_chain}; [agents.<role>.runtime]. (`phase-4M:L124-L134`)
- **P4M-E011** [E/EVIDENCE]: 4M.0 ingest ouroboros MIT → ouroboros-runtime-interface.md MEDIUM; ≥7 adapters. (`phase-4M:L138-L161`)
- **P4M-E012** [E/CAP]: 4M.1 types + resolve_backend + create_runtime stubs; [runtime] and per-role schema; existing Sidecar/AnthropicApi stay functional. (`phase-4M:L163-L184`)
- **P4M-E013** [E/INT]: Alias RuntimeAdapter=LlmProvider if vtable compatible else wrapper without reimplement. (`phase-4M:L95-L97, L169-L172`)
- **P4M-E014** [E/CAP]: 4M.2 SidecarRuntime + AnthropicApiRuntime /v1/messages; health canary Ready (`Degraded`)
- **P4M-E015** [E/CFG]: Migrate [backend] local (`api`)
- **P4M-E016** [E/CAP]: 4M.3 seven CLI adapters: spawn+pipe; translate stdin; parse stdout. (`phase-4M:L239-L253`)
- **P4M-E017** [E/CFG]: permission_mode flags per backend; honor startup and stdout_idle timeouts. (`phase-4M:L254-L257`)
- **P4M-E018** [E/SEC]: CLI adapters MUST Tier C isolation; stdout sanitized via 4A IPC before prompt context. (`phase-4M:L264-L268, L286-L289`)
- **P4M-E019** [E/FAIL]: Missing CLI Unavailable not panic; hang kill on idle timeout; nonzero exit always Err; Plan mode no Edit/Write. (`phase-4M:L277-L296`)
- **P4M-E020** [E/CAP]: 4M.4 Ollama /api/chat; Vllm /v1/chat/completions; health /api/tags or /health. (`phase-4M:L304-L322`)
- **P4M-E021** [E/CFG]: Defaults: designer/auditor/researcher/media_validator AnthropicApi; executor/memory_writer/context_compiler/prompt_engineer Sidecar. (`phase-4M:L331-L339`)
- **P4M-E022** [E/RECOVERY]: Runtime fallback on Unavailable; SSB runtime_fallback; exhaust ESCALATE INTERACTIVE or HALT AUTONOMOUS; before model fallback. (`phase-4M:L340-L365`)
- **P4M-E023** [E/SURFACE]: CLI /runtime show. (`phase-4M:L344-L345`)
- **P4M-E024** [E/SURFACE]: CLI /runtime set <role> <backend>. (`phase-4M:L346-L346`)
- **P4M-E025** [E/SURFACE]: CLI /runtime fallback <role> <b1>,<b2>. (`phase-4M:L347-L347`)
- **P4M-E026** [E/STATE]: PerRoleRuntime per-session SessionState not global. (`phase-4M:L353-L356`)
- **P4M-E027** [E/EVENT]: runtime_fallback SSB reason+chain; dashboard yellow; CloudSpendTracker integration. (`phase-4M:L357-L360, L443-L447`)
- **P4M-E028** [E/SURFACE]: RuntimePanel/Settings: per-role backend, chain, live health g/y/r, Test backend button. (`phase-4M:L376-L380`)
- **P4M-E029** [E/OBS]: phase_4M.toml: p50/p99, tokens/sec, cost, fallback-fire rate, health uptime %. (`phase-4M:L381-L385`)
- **P4M-E030** [E/DATA]: Migration guide phase-4M-migration.md old [backend]→new [runtime]. (`phase-4M:L386-L388`)
- **P4M-E031** [E/INT]: RuntimeClient must wire ssb, fsm, clock; no None after construction. (`phase-4M:L463-L465`)
- **P4M-E032** [E/EVIDENCE]: Grounding: ouroboros MIT pattern; 7 CLIs; PermissionMode; Tier C ELAI-specific; Ollama/vLLM ELAI-specific; per-role PARTIAL; fallback OUR DESIGN. (`phase-4M:L399-L429`)
- **P4M-E033** [E/LIMIT]: R1 CLI drift probe-cache capabilities; R4 cost risk via SSB+dashboard+cost tracker. (`phase-4M:L432-L447`)
- **P4M-I001** [I/CFG]: TOML examples imply snake_case resolve_backend names. (`phase-4M:L131-L134, L175-L178`)
- **P4M-A001** [A/CAP]: RuntimeAdapter alias vs new trait open at execution. (`phase-4M:L169-L172`)
- **P4M-A002** [A/SURFACE]: RuntimePanel vs SettingsWindow placement open. (`phase-4M:L376-L376`)
- **P4M-A003** [A/CFG]: Full 11 name strings and hybrid/auto migration beyond local/api incomplete. (`phase-4M:L218-L221`)
- **P4M-A004** [A/INT]: SlashCommand::Runtime full parser wiring thin. (`phase-4M:L351-L351`)

### 1A: DKG (tree-sitter → SQLite graph)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-1A.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-001 — Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE.; PHASE-002/IX-002 — 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4).; PHASE-002/IX-015 — Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys.; PHASE-002/IX-017 — 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks.

**Behaviors**

- **P1A-E001** [E/STATUS]: Phase 1A builds Layer-1 DKG: parse source into a dependency knowledge graph stored in SQLite. Prerequisites: Phase 0 complete (9-crate layout, cargo check passes). (`1A:L1-L12`)
- **P1A-E002** [E/DEP]: Canonical knowledge crate path is dynamic: `KNOW_PATH=$(find .elai_cc -path "*/knowledge/Cargo.toml" (`head -1`)
- **P1A-E003** [E/DEP]: Downstream consumers: Phase 1D PageRank reads DKG edges; Phase 1E CatRAG traverses DKG; Phase 1B BM25 shares readiness gate but runs independently. (`1A:L13-L15`)
- **P1A-E004** [E/STATUS]: knowledge.txt “FIVE LAYERS” was corrected to “5 major + 2 sub-layers”; architecture-overview “7 layers” counts sub-layers 3.25 and 3.5. DKG is Layer 1. (`1A:L17-L19, L470-L470`)
- **P1A-E005** [E/LIMIT]: R7 (LOW): tree-sitter grammar gaps. Start with Rust, Python, TypeScript, Go. (`1A:L26-L27`)
- **P1A-E006** [E/DATA]: R-1A-1: SQLite schema is not in upstream specs; this phase must define schema explicitly. (`1A:L28-L28, L468-L469`)
- **P1A-E007** [E/OBS]: R-1A-2: no build-time baseline exists. knowledge.txt L20 “<15s” claim is UNGROUNDED; 1A.5 establishes first measurement. (`1A:L29-L29, L407-L408, L467-L468`)
- **P1A-E008** [E/DEP]: R-1A-3: Phase 1D PRE_GATE requires edge/adjacency iteration API from DkgEngine; without it Phase 1D is blocked at entry. (`1A:L30-L31, L165-L175, L426-L426`)
- **P1A-E009** [E/FAIL]: R-1A-4 / COMPLIANCE SCOPE: knowledge.txt Sec 8 has 5 parse-failure items. Phase 1A implements only items 1 (exclude from DKG) and 5 (log + retry on next change). Items 2–4 (BM25 still indexes, vector still embeds, PageRank leaf) are deferred to 1B/1C/1D. (`1A:L32-L47, L338-L342, L427-L427`)
- **P1A-E010** [E/LIMIT]: Orphan-file invariant “every file appears in at least BM25 index” is NOT achievable until Phase 1B. Phase 1A tracks parse-failed files so 1B can index them. (`1A:L46-L47, L449-L450`)
- **P1A-E011** [E/CAP]: LanguageGrammar must be object-safe for `dyn` dispatch: all methods take `&self`. (`1A:L35-L35, L84-L91, L460-L461`)
- **P1A-E012** [E/DATA]: DkgNodeInfo must include `signature` and `SymbolKind` to match Phase 1D contract. (`1A:L36-L37, L92-L109, L453-L454`)
- **P1A-E013** [E/CFG]: Dependencies (verified crates.io 2026-04-06): `tree-sitter=0.26`, `tree-sitter-rust=0.24`, `rusqlite={version=0.39, features=["bundled"]}`, `blake2=0.10`, dev `proptest=1`. Future: python 0.25, go 0.25, typescript 0.23 (aging; re-verify). (`1A:L60-L69, L456-L459`)
- **P1A-E014** [E/CAP]: KnowledgeEngine trait methods: `build_graph`, `query`, `rerank`, `update`, `readiness_check`. (`1A:L70-L75`)
- **P1A-E015** [E/DATA]: DependencyGraph return: `{ node_count, edge_count, files_parsed }`. (`1A:L76-L77, L336-L337`)
- **P1A-E016** [E/STATE]: ReadinessStatus enum: READY, DEGRADED, UNAVAILABLE (full-engine aggregate). (`1A:L78-L78`)
- **P1A-E017** [E/STATE]: Phase 1A implements DKG-layer readiness only via `dkg_readiness() -> DkgReadiness` (Ready, Empty, Unavailable). Trait `readiness_check()` aggregation is Phase 1I. (`1A:L79-L83, L172-L177, L343-L348, L429-L429, L477-L477`)
- **P1A-E018** [E/CAP]: LanguageGrammar methods: `language_id`, `tree_sitter_language`, `extract_nodes`. Start with RustGrammar only. (`1A:L84-L91, L302-L313`)
- **P1A-E019** [E/DATA]: SymbolKind: Function, Struct, Impl, Trait, Enum, Module, Use, TypeAlias, Other. (`1A:L92-L93`)
- **P1A-E020** [E/DATA]: DkgNodeInfo fields: node_id, symbol_name, file_path, signature, kind, start_line, end_line. Signature reconstructed from AST (function/struct/trait/impl/enum forms). (`1A:L94-L109, L305-L311`)
- **P1A-E021** [E/DATA]: DkgEngine::new(db_path) takes explicit project-scoped path (caller provides `.elai_cc/projects/{project_id}/knowledge.db`). (`1A:L110-L113, L351-L353, L433-L433`)
- **P1A-E022** [E/DATA]: dkg_nodes columns: id PK, file_path, name, kind, signature DEFAULT '', start_line, end_line, parent_id, concept DEFAULT NULL, concept_stale DEFAULT 1, file_hash BLOB. (`1A:L115-L127`)
- **P1A-E023** [E/DATA]: dkg_edges: source_id, target_id, edge_type, PRIMARY KEY (source_id, target_id, edge_type), ON DELETE CASCADE. Edge types: "imports", "contains", "implements" (directed). (`1A:L128-L139, L318-L321`)
- **P1A-E024** [E/DATA]: dkg_files: file_path PK, file_hash, parse_status ('ok'/'failed'/'pending'), parse_error, last_indexed. Enables O(1) staleness and Phase 1B failed-file bridge. (`1A:L140-L148`)
- **P1A-E025** [E/DATA]: dkg_meta key/value with initial rows schema_version=1, created_at. Indexes: name, file, kind, files_status. (`1A:L149-L155`)
- **P1A-E026** [E/DATA]: index_version.toml five fields: version, dkg_schema_version, vector_dimensions placeholder, bm25_format placeholder, created_at. (`1A:L156-L164, L349-L350, L430-L430`)
- **P1A-E027** [E/CAP]: Public API: nodes, edges, node_by_id, outbound_edges, inbound_edges, failed_files, dkg_readiness. Directed edge queries replace ambiguous neighbors(). (`1A:L165-L175, L324-L330, L484-L484`)
- **P1A-E028** [E/FLOW]: Steps: 1A.1 skeleton → 1A.2 verify workspace → 1A.3 TDD tests (≥20, ≥10 fail red) → 1A.4 implement → 1A.5 measure. (`1A:L50-L420`)
- **P1A-E029** [E/TEST]: 1A.3 functional tests: parse Rust nodes into SQLite; query by function/struct name; signature population; incremental update with file_hash change and concept_stale=1; concept NULL default; file_hash blake2s round-trip. (`1A:L236-L244`)
- **P1A-E030** [E/TEST]: Edge-type tests: impl→method contains; use→imports; impl Trait→implements; outbound direction. Downstream API tests for edges/outbound/inbound/nodes/failed_files. (`1A:L245-L255`)
- **P1A-E031** [E/TEST]: Parse-failure tests: empty file zero nodes no panic; syntax errors partial extract + parse_status=failed; binary/non-text graceful skip + failed_files. (`1A:L256-L261`)
- **P1A-E032** [E/TEST]: Readiness: Unavailable before build; Ready after >0 nodes; Empty after empty/all-failed directory. Isolation: two DkgEngine db_paths do not share state. (`1A:L262-L267, L394-L394`)
- **P1A-E033** [E/TEST]: proptest: random valid Rust → node_count≥0, line numbers>0, names non-empty, hashes non-empty, signatures non-empty for function/struct. (`1A:L268-L271`)
- **P1A-E034** [E/TEST]: TDD red gate: ≥20 test functions; cargo test --no-run ok; ≥10 FAILED (stubs pass ≤3 trivial). (`1A:L273-L294`)
- **P1A-E035** [E/CFG]: SQLite WAL: PRAGMA journal_mode=WAL. blake2s content hash for incremental staleness. (`1A:L315-L317, L331-L335, L360-L361`)
- **P1A-E036** [E/FLOW]: Incremental update: hash compare; unchanged skip; changed DELETE nodes for file_path (CASCADE edges), re-parse, concept_stale=1. (`1A:L331-L335`)
- **P1A-E037** [E/FAIL]: On parse failure: log::warn, dkg_files parse_status=failed, no nodes, no panic/abort. (`1A:L338-L342, L393-L393`)
- **P1A-E038** [E/OBS]: Metrics file `.elai_cc/metrics/phase_1A.toml` with ≥12 keys: build_time_ms, query_p50/p99_ms, db_size_bytes, total_nodes/edges, files_parsed/skipped, edges_imports/contains/implements, signatures_populated. (`1A:L396-L420, L434-L434`)
- **P1A-E039** [E/EVENT]: Wiring observable: trigger `elai-cli index build --project-root ...` expects SSB `symbol_graph_updated`. (`1A:L511-L513`)
- **P1A-E040** [E/INT]: Production reachability: `cli/src/index_cmd.rs::run_index_build`, `orchestrator/src/indexing_service.rs::IndexingService::run_full_index`. (`1A:L509-L510`)
- **P1A-E041** [E/CAP]: produces_symbols: DkgEngine::{new,build_graph,nodes,edges,outbound_edges,inbound_edges,failed_files,dkg_readiness}, DependencyGraph, DkgNodeInfo, SymbolKind, DkgReadiness, LanguageGrammar, ReadinessStatus. (`1A:L490-L508`)
- **P1A-E042** [E/LIMIT]: Exit criteria require directed edge API, failed_files for 1B, DKG-only readiness (not full-engine), index_version.toml 5 fields, signature+SymbolKind, concept_stale, project-scoped db, metrics, no Phase 0 regressions. (`1A:L423-L435`)
- **P1A-E043** [E/DEFER]: BM25 still-index, vector embed, PageRank leaf treatment of parse-failed files deferred to 1B/1C/1D. Python/TS/Go grammars deferred beyond Rust start. (`1A:L42-L47, L67-L69, L427-L427`)
- **P1A-I001** [I/DATA]: Edge types imports/contains/implements are design-derived from knowledge.txt edge-weight/CatRAG and phase-1D, not literal upstream schema. (`1A:L462-L463`)
- **P1A-A001** [A/CFG]: Exact tree-sitter-typescript pin may need re-verification (aging 2024-11). No concrete acceptance for multi-language beyond Rust start. (`1A:L67-L69`)
- **P1A-A002** [A/OBS]: No formal query-latency SLO; only baseline recording. (`1A:L468-L468`)
- **P1A-A003** [A/FAIL]: “Partial extraction” on syntax errors has no defined completeness floor for which nodes may be kept. (`1A:L258-L259`)

### 1D: PageRank repo map

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1D.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-001 — Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE.; PHASE-002/IX-002 — 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4).; PHASE-002/IX-003 — 1D density-guard fallback is insertion-order, not BM25, because BM25 rerank is upstream in the query pipeline.; PHASE-002/IX-017 — 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks.

**Behaviors**

- **P1D-E001** [E/STATUS]: Phase 1D ranks code symbols by PageRank on DKG and produces token-budget-fitted repo maps. Prerequisites: Phase 1C complete (knowledge + ort). (`1D:L1-L8`)
- **P1D-E002** [E/EVIDENCE]: Derives from aider/repomap.py (design 31.7). Three deliberate divergences: symbol-level nodes, density guard, insertion-order fallback. Five adopted improvements with tests. (`1D:L14-L17, L251-L279`)
- **P1D-E003** [E/FAIL]: R20 density degradation: density guard at 0.3. NOTE: R20 ID collides with phase-0 entropy quarantine; cross-file renumber deferred. (`1D:L19-L23, L314-L316`)
- **P1D-E004** [E/RECOVERY]: Dangling nodes (zero out-edges) redistribute via personalization vector (aider dangling=personalization / NetworkX). (`1D:L24-L28, L178-L183, L255-L257`)
- **P1D-E005** [E/DATA]: Orphan symbols get self-edges weight=0.1 during DkgSnapshot::from_engine so rank > 0. (`1D:L29-L32, L54-L56, L258-L260`)
- **P1D-E006** [E/DATA]: DkgSnapshot { nodes: Vec<DkgNodeInfo>, edges: Vec<(usize,usize,f64)> }; DkgNodeInfo adds ref_count (incoming references for sqrt scaling). Built via from_engine. (`1D:L48-L53`)
- **P1D-E007** [E/CFG]: PageRankConfig defaults: damping_factor=0.85, max_iterations=100, tolerance=1e-6, density_threshold=0.3, max_map_tokens=2048. damping/max_iter/tol are standard PageRank (UNGROUNDED in ELAI specs) but overridable via config. (`1D:L58-L60, L303-L304`)
- **P1D-E008** [E/DATA]: RepoMap { ranked_symbols, token_count, density, used_pagerank, fit_iterations }. RankedSymbol { node_id, symbol_name, file_path, rank, signature }. (`1D:L61-L63`)
- **P1D-E009** [E/CAP]: compute_repo_map(snapshot, task, active_files, config, token_estimator: &dyn Fn(&str)->usize) -> RepoMap. (`1D:L64-L65`)
- **P1D-E010** [E/TEST]: ≥21 unit tests T1–T18 + A1–A3; proptest P1–P3; make_snapshot helper. TDD: tests fail on todo!(). (`1D:L96-L154, L319-L320`)
- **P1D-E011** [E/TEST]: T1 linear graph task mention ranks highest; T2 active_files edges *=50; T3 task mention 10× personalization; T4 density>0.3 → used_pagerank=false insertion-order fallback (NOT BM25); T5 density<0.3 → used_pagerank=true. (`1D:L102-L108, L171-L177`)
- **P1D-E012** [E/TEST]: T6 token_count ≤ max_map_tokens×1.15; T7 undershoot ≥85% of budget; T8 special files always included; T9 personalization sums to 1.0; T10 fit_iterations ≤10; T11 empty graph; T12 single node; T13 zero budget no panic; T14 all-mentioned uniform personalization finite ranks. (`1D:L108-L117`)
- **P1D-E013** [E/TEST]: T15 dangling redistribution; T16 orphan self-edges; T17 sqrt ref scaling rank ratio <8 not 25; T18 500-char signature truncated to 100 chars for token estimate. (`1D:L118-L132, L329-L332`)
- **P1D-E014** [E/TEST]: P1 ranks finite ≥0; P2 sum ranks ≈1 when used_pagerank; P3 token budget bound. A1 near-complete density guard; A2 star hub highest; A3 disconnected components base rank only. (`1D:L134-L140`)
- **P1D-E015** [E/FLOW]: Implement order: adjacency + sqrt(ref_count); self-edges; active_files boost ×50; personalization 10× then normalize; density = e/(n*(n-1)); if density>0.3 or n≤1 skip PageRank; power iteration with dangling redistribution; NaN/Inf clamp to 0. (`1D:L160-L209`)
- **P1D-E016** [E/DATA]: Special files hardcoded 13 Rust items: Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile, docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml. Config-driven list deferred. (`1D:L184-L186, L305-L305`)
- **P1D-E017** [E/FLOW]: Binary search fit: max 10 iterations, ±15%, initial guess min(max_map_tokens/25, num_nodes). max_map_tokens=0 → empty ranked, specials omitted. (`1D:L187-L194, L212-L213`)
- **P1D-E018** [E/DATA]: Line truncation max_chars=100 before token estimation (aider line[:100]). (`1D:L195-L198, L210-L210, L267-L269`)
- **P1D-E019** [E/LIMIT]: Not adopted from aider: identifier specificity boost, private underscore penalty, generic-name penalty (>5 defs). (`1D:L281-L288`)
- **P1D-E020** [E/OBS]: Metrics `.elai_cc/metrics/phase_1D.toml`: test_count, compile_time_seconds, pagerank_loc, ranking_quality, token_budget_accuracy, density_guard_correct, binary_search_max_iterations (7 metrics). (`1D:L230-L248`)
- **P1D-E021** [E/TEST]: Exit: pagerank tests 0 failures ≥21; density guard; ±15% budget; specials when budget allows; no todo!/unimplemented!; clippy 0 errors; metrics; T13; PROPTEST_CASES=100; T15–T18 pass. (`1D:L318-L332`)
- **P1D-E022** [E/DEP]: providers token_count_estimate used via mockable estimator closure; PRE_GATE greps for DKG edge API and token_count_estimate. (`1D:L39-L43, L150-L150`)
- **P1D-E023** [E/DET]: Density formula: edges.len()/(nodes.len()*(nodes.len()-1)). Insertion-order fallback uses DkgSnapshot node order. (`1D:L105-L106, L171-L177`)
- **P1D-C001** [C/STATUS]: Spec conflict fixed: PageRank is Layer 3.5 not Layer 4 (CatRAG is Layer 4). (`1D:L314-L315`)
- **P1D-A001** [A/CFG]: Special-files list is ungrounded relative to knowledge.txt “~100 items”; only 13 Rust items ship. (`1D:L305-L305`)
- **P1D-A002** [A/CFG]: Exact personalization mention-matching algorithm (substring vs symbol equality) not formalized beyond “mentioned in task”. (`1D:L104-L104, L169-L170`)
- **P1D-A003** [A/DET]: Density guard proof cites max_rank/min_rank <1.1 on dense graphs as justification, but that ratio is not a production invariant outside A1. (`1D:L173-L176, L276-L277`)

### 3D: BSP supersteps + channels

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3D.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-004 — 3D requires 3B SSB + 3C agents; upgrades 3C spawn_parallel stub to true JoinSet parallelism; 3E/3F consume ChannelRegistry.; PHASE-002/IX-005 — 3D checkpoint+SSB replay composes with 3C agent checkpoints under `.elai_cc/sessions/{sid}/`; UntrackedValue deliberately absent from both.; PHASE-002/IX-016 — SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*.; PHASE-002/IX-020 — 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step.

**Behaviors**

- **P3D-E001** [E/STATUS]: Phase 3D: deterministic macro orchestration via BSP supersteps (Valiant 1990). Prerequisites: 3B SSB + 3C agents. (`3D:L1-L7`)
- **P3D-E002** [E/LIMIT]: BinaryOp intentionally dropped (non-deterministic with concurrent writes; audit BLOCK 4). (`3D:L8-L9, L363-L363`)
- **P3D-E003** [E/DET]: Determinism via total-ordered LastValue (SSB seq) or append-only Topic (SSB seq). Holds in simulation mode (mocked agents, fixed seed). Real LLMs may vary straggler participation; combiners well-defined on partial inputs but byte-identical checkpoints require identical agent sets. (`3D:L10-L18, L357-L361`)
- **P3D-E004** [E/CFG]: R19 UntrackedValue memory: configurable `elai_cc.toml [orchestrator.bsp.untracked_memory_limit_bytes]` default 10MB; force-clear at >2× limit; DeepSizeOf required (size_of_val banned). (`3D:L27-L32, L232-L238, L375-L376`)
- **P3D-E005** [E/FAIL]: Straggler timeout reuses Phase 3C heartbeat default 15s; AgentDeath::TIMEOUT removes agent from barrier; remaining proceed. (`3D:L33-L37, L162-L164, L377-L377`)
- **P3D-E006** [E/RECOVERY]: Crash mid-superstep: supervision tree removes dead agent; orchestrator does not wait for dead agents. (`3D:L38-L41`)
- **P3D-E007** [E/DEFER]: Spec amendments required before implement: Topic(accumulate=false) definition; memory_monitoring action_type; ChannelRegistry trait; save_checkpoint after evidence_gate. (`3D:L43-L54`)
- **P3D-E008** [E/CAP]: LastValue<T>: single value, last writer wins by SSB seq; read Option<T>. (`3D:L67-L68`)
- **P3D-E009** [E/CAP]: Topic<T>(accumulate: bool): true = append-only Vec persists; false = two-phase snapshot then clear before execute writes. Append order by SSB seq. (`3D:L69-L75, L248-L252, L372-L374`)
- **P3D-E010** [E/CAP]: EphemeralValue<T>: fire-and-forget; clear to MISSING when write_count==0 at barrier; saturating_add write counter; clearing journaled for replay. (`3D:L76-L82, L239-L247`)
- **P3D-E011** [E/CAP]: UntrackedValue<T:DeepSizeOf>: in-memory only, never checkpointed; multi-writer allowed; lost on crash. (`3D:L83-L85, L369-L369`)
- **P3D-E012** [E/DATA]: ChannelId typed key; ChannelState enum has exactly 3 variants (LastValue, Topic, Ephemeral) — no UntrackedValue. (`3D:L86-L87, L104-L104`)
- **P3D-E013** [E/CAP]: ChannelRegistry: read, write, snapshot_for_scheduling, clear_ephemeral, force_clear_untracked, deep_size_untracked. Types ChannelWrite/Value/Error/Snapshot. Implementation-defined until amendment 3. (`3D:L88-L100`)
- **P3D-E014** [E/FLOW]: Corrected BSP loop: snapshot → prepare_next_tasks → clear_non_accumulating_topics → execute_tasks_parallel → apply_writes (SSB seq) → deferred phase → clear_ephemeral → evidence_gate → save_checkpoint only if gate passes; else attempt++ up to max_superstep_attempts (default 3). max_supersteps total cap 100. (`3D:L125-L160, L385-L386`)
- **P3D-E015** [E/DATA]: Checkpoint path: `.elai_cc/sessions/{sid}/checkpoints/superstep_{N}.msgpack` (msgpack HashMap ChannelId→ChannelState). Superstep counter u64 monotonic in checkpoint. (`3D:L154-L161`)
- **P3D-E016** [E/FLOW]: Deferred nodes (NodeSpec.is_deferred): single pass post-barrier same superstep; all_inputs_satisfied required; no cascading deferred. (`3D:L186-L219, L364-L364`)
- **P3D-E017** [E/EVENT]: memory_monitoring SSB action (NEW, not isolation_metric): payload {channel_id, size_bytes, threshold_bytes, action: measure (`force_clear}. isolation_metric remains cognitive-token only.`)
- **P3D-E018** [E/RECOVERY]: Crash recovery: load latest checkpoint; replay SSB since superstep: apply writes by seq; reconstruct ephemeral clears; clear Topic(accumulate=false) at boundary before writes. UntrackedValue lost by design. (`3D:L253-L258, L340-L340`)
- **P3D-E019** [E/CONC]: spawn_parallel upgraded from 3C sequential stub to tokio::JoinSet true parallel; VRAM-aware max_concurrent = floor(available_vram/per_agent_vram) fallback 1; results applied by SSB seq not completion order; wall-time concurrency proof. (`3D:L289-L327, L383-L384`)
- **P3D-E020** [E/OBS]: Baseline measurements (not exit criteria, UNGROUNDED estimates): superstep overhead <50ms, checkpoint <1MB, recovery <500ms, parallel speedup, UntrackedValue peak memory. (`3D:L346-L354, L387-L388`)
- **P3D-E021** [E/TEST]: Determinism proof: same seed + mocked agents → byte-identical checkpoints. No checkpoint file on failed gate. Integration 3+ supersteps with 2 agents. (`3D:L165-L184, L330-L344`)
- **P3D-E022** [E/LIMIT]: size_of_val banned in bsp/; DeepSizeOf required. No isolation_metric in bsp/ code. (`3D:L105-L105, L265-L265, L342-L342`)
- **P3D-E023** [E/EVENT]: Ephemeral clear recorded as action_type='checkpoint' subtype/payload ephemeral_clear / ephemeral_clears Vec. (`3D:L79-L82, L243-L247`)
- **P3D-A001** [A/CFG]: max_superstep_attempts default 3 and max_supersteps 100 are stated here; config key names beyond untracked_memory_limit not fully enumerated. (`3D:L158-L160`)
- **P3D-A002** [A/INT]: ChannelRegistry formalization blocked on orchestrator.txt amendment before 3E. (`3D:L97-L100`)
- **P3D-A003** [A/DET]: Evidence_gate_passes exact predicate is phase-transition evidence “sufficient” without full boolean algorithm in this file. (`3D:L158-L159`)

### 3G: hierarchical plan management

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3G.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-006 — 3G.4 requires 3E Slot 3 ContextBudget; skeleton always loads in PhaseSpec slot; overflow uses 3E ReducedGranularity.; PHASE-002/IX-007 — 3G coherence is invoked by 3C auditor via IPC; findings use action_type finding; plan mutations use plan_update/stub_reference.; PHASE-002/IX-008 — 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing.; PHASE-002/IX-016 — SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*.

**Behaviors**

- **P3G-E001** [E/STATUS]: Two-artifact plans: skeleton (2–4k tokens, always Slot 3) + per-phase elaboration on demand. Contract extraction + coherence enforcement. Prerequisites: 3D; 3E required for 3G.4 only. (`3G:L1-L16`)
- **P3G-E002** [E/DEP]: Dependency-graph note: implementation-guide shows 3E∥3G from 3D is wrong for 3G.4; recommended partial parallelism with 3G.4 gated on 3E. (`3G:L6-L9`)
- **P3G-E003** [E/DATA]: PlanSkeleton: pipeline_id, phases IndexMap ordered, global_constraints, version u32, created_at/updated_at (UNGROUNDED timestamps via ClockProvider). (`3G:L37-L44`)
- **P3G-E004** [E/DATA]: PhaseEntry: phase_id, goal max 120 chars, inputs/outputs ContractRef, constraints, depends_on/depended_by, version, output_type Option, visual_check bool default false. (`3G:L46-L60`)
- **P3G-E005** [E/CFG]: visual_check=true requires MediaRenderAdapter for output_type; output_type=None forces visual_check=false at validation. (`3G:L58-L60`)
- **P3G-E006** [E/DATA]: ContractRef { name, type_signature, invariants }. PhaseDetail { phase_id, skeleton_version, contracts, implemented_contracts, declared_interfaces, content, token_count exact }. ExtractedContracts { input_contracts, output_contracts, shared_constraints }. (`3G:L62-L82`)
- **P3G-E007** [E/DATA]: Storage dual-path TOML authoritative: `.elai_cc/pipelines/{id}/skeleton.toml` and `phases/{phase_id}.toml`. SSB action_type='plan_update' every write. No plan_phases table; query SSB JSON filter (orchestrator.txt L250 is conceptual shorthand). (`3G:L84-L93, L323-L324`)
- **P3G-E008** [E/CFG]: Skeleton token strategy: estimate for allocation; exact token_count for final write; hard reject SkeletonBudgetExceeded if exact >4096. Safety margin: estimate >3600 triggers exact check. (`3G:L108-L116, L207-L211`)
- **P3G-E009** [E/FLOW]: Contract extraction before elaborating X: (1) union dep.outputs as inputs; (2) union depended_by.inputs as outputs; (3) global_constraints + phase.constraints. Model elaborates against contracts not full plan text. (`3G:L118-L128, L244-L248`)
- **P3G-E010** [E/SEC]: Coherence after elaboration: (1) implemented_contracts superset extracted; (2) declared_interfaces subset skeleton interfaces (structured set compare, not free text); (3) contract change increments phase.version AND skeleton.version. Violation → auditor BLOCK finding. (`3G:L131-L144, L307-L345`)
- **P3G-E011** [E/EVENT]: stub_reference SSB when skeleton phase lacks PhaseDetail file: payload {phase_id, skeleton_version}. Superseded on later detail write (append-only, no delete). (`3G:L146-L154, L213-L213, L461-L461`)
- **P3G-E012** [E/FLOW]: Dependency propagation: on contract version change, BFS depended_by; flag needs_review via plan_update payload {phase_id, reason:dependency_contract_changed, source_phase}. Must re-elaborate before execution. (`3G:L157-L164, L357-L361`)
- **P3G-E013** [E/FAIL]: Cycle detection on skeleton write via Kahn's algorithm O(V+E); reject CycleDetected. (`3G:L165-L167, L362-L362, L376-L376`)
- **P3G-E014** [E/CAP]: PlanSkeleton::split_phase(phase_id, split_points) with preserve_contracts bool default true: first sub-phase gets inputs, last gets outputs, internal contracts between; max split depth 2 (matches 4G SPLIT_TASK). (`3G:L365-L380`)
- **P3G-E015** [E/FLOW]: load_phase_context: skeleton always + active detail + extracted contracts + one-hop adjacent; fit Slot 3; overflow → ReducedGranularity (skeleton+contracts only); skeleton alone > budget → SlotBudgetExceeded. skeleton_version mismatch → staleness re-extraction. (`3G:L271-L301`)
- **P3G-E016** [E/INT]: coherence_check is library invoked by auditor agent via IPC ToolRequest, not standalone component. Findings action_type='finding'. (`3G:L313-L328`)
- **P3G-E017** [E/FAIL]: Empty declared_interfaces → coherence_check Err (elaboration must self-declare). Version check always reads current file/SSB, never stale cache. (`3G:L337-L341`)
- **P3G-E018** [E/DET]: Version: phase.version +=1 on contract field change; skeleton.version +=1 on any phase version change; checked_add on u32 overflow → Err. (`3G:L363-L364, L384-L386`)
- **P3G-E019** [E/TEST]: 3G.7 full lifecycle 5-phase DAG; 100× identical results with ClockProvider injected; unique pipeline_id EntropyProvider; no Instant::now/SystemTime::now in plan_skeleton. (`3G:L398-L434`)
- **P3G-E020** [E/OBS]: Measure: skeleton tokens, extraction time (<10ms/50-phase target), coherence time, propagation diamond, SSB plan_update count, compile delta, test count, LOC, coverage. (`3G:L441-L450`)
- **P3G-E021** [E/TEST]: Exit criteria: budget, extraction algorithm match, transitive needs_review, 3 coherence types, BLOCK findings, cycles, monotonic versions, plan_update audit, stub_reference, determinism, contract tests for PlanSkeleton/PhaseDetail/CoherenceChecker. (`3G:L452-L464`)
- **P3G-E022** [E/SURFACE]: Skeleton TOML format example under [phase.3B] with goal/inputs/outputs/constraints/depends_on/depended_by/version. (`3G:L95-L105`)
- **P3G-E023** [E/FAIL]: SkeletonBudgetExceeded never silently truncates. PhaseNotFound / ContractConflict on extraction errors. (`3G:L210-L211, L255-L262`)
- **P3G-E024** [E/CONC]: R26 version monotonicity relies on SSB single-writer; concurrent multi-session skeleton edits are sequentialized by single-writer channel. (`3G:L30-L32`)
- **P3G-A001** [A/DATA]: SQL plan_phases view/table as amendment requires user confirmation; interim is SSB filter only. (`3G:L88-L93`)
- **P3G-A002** [A/CFG]: Empty type_signature yields validation warning not error (some contracts textual) — exact warning surface unspecified. (`3G:L192-L193`)
- **P3G-A003** [A/TEST]: Coherence check 1 prose also says “corresponding implementation reference in elaboration text” while 3G.5 requires structured implemented_contracts match — dual wording. (`3G:L134-L136, L320-L322`)

### 4D-ORACLE: Oracle Mode /research cache

- **Status**: shipped (per source STATUS atom; not independently verified in this compile)
- **Anchors**: source `docs/specs/guide/steps/phase-4D-ORACLE.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-014 — 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss.; PHASE-002/IX-015 — Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys.; PHASE-002/IX-016 — SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*.

**Behaviors**

- **P4DOR-E001** [E/STATUS]: SHIPPED in C-30 (SCHEMA_VERSION 61→62). Spec record for OracleAnswerCache; cross-ref C-30-decisions DECISION-8/9/13, NOTE-A. (`4DOR:L1-L7`)
- **P4DOR-E002** [E/CAP]: Cache /research answers in-process per (query × project) so repeated questions within freshness window skip full ConceptDecomposition + WebResearch. (`4DOR:L9-L11`)
- **P4DOR-E003** [E/DATA]: CacheKey: query_hash blake3(query) [u8;32], project_hash blake3(project_root_canonical_path) [u8;32]. (`4DOR:L32-L34, L65-L66`)
- **P4DOR-E004** [E/DATA]: CachedAnswer: answer String, cached_at SystemTime, source_tier SourceTier (T1Authoritative (`T2Strong`)
- **P4DOR-E005** [E/CAP]: OracleAnswerCache: in-memory LRU; max_entries from elai_cc.toml. (`4DOR:L41-L42`)
- **P4DOR-E006** [E/CFG]: TTL: T1 2_592_000s (30d); T2 604_800s (7d); T3 86_400s (1d); T4 0s never cached (insert early-returns false). (`4DOR:L44-L50`)
- **P4DOR-E007** [E/CFG]: Confidence mapping: High→T1, Medium→T2, Low→T3, Unknown→T4Rejected (no cache). Field on ResearchAnswer is `confidence` NOT `answer_confidence` (DECISION-8). (`4DOR:L52-L59`)
- **P4DOR-E008** [E/FLOW]: Miss path: compute key → run_research_pipeline → tier from confidence → insert if ≠T4 → emit oracle_cache_miss {query_hash_short, source_tier}. (`4DOR:L64-L70`)
- **P4DOR-E009** [E/FLOW]: Hit path: lookup → verify cached_at+ttl > now else miss/evict → return cached → emit oracle_cache_hit {cached_age_secs, source_tier}. (`4DOR:L72-L77`)
- **P4DOR-E010** [E/CFG]: LRU: when len > max_entries evict LRU. max_entries from elai_cc.toml [research.oracle] max_cached_answers default 128. (`4DOR:L79-L82`)
- **P4DOR-E011** [E/INT]: LiveCli field oracle_cache: Option<Arc<OracleAnswerCache>> default None; lazy init on first /research; all three construction sites default None. (`4DOR:L84-L87`)
- **P4DOR-E012** [E/EVENT]: SSB types oracle_cache_hit {cached_age_secs, source_tier}, oracle_cache_miss {source_tier}; SCHEMA 62 9-tuple atomic bump. (`4DOR:L89-L95`)
- **P4DOR-E013** [E/SURFACE]: reachable_from: /research and /ask slash commands (commands.rs Research / Ask variants). (`4DOR:L108-L109`)
- **P4DOR-E014** [E/LIMIT]: stub_ban: T4Rejected MUST early-return on insert; hashes MUST use blake3 (no blake2/sha2). (`4DOR:L110-L113`)
- **P4DOR-E015** [E/OBS]: Observable: `elai /research "what does B-37 ship?"` twice within 60s → miss then hit with cached_age_secs>0. (`4DOR:L114-L117`)
- **P4DOR-E016** [E/TEST]: DETERMINISTIC_CHECK: OracleAnswerCache present; TTL constants; OracleCacheHit/Miss; blake3; cargo test c30_surgical_oracle_contract. (`4DOR:L119-L137`)
- **P4DOR-E017** [E/INT]: consumes: run_research_pipeline (live_cli.rs:2578), AnswerConfidence, SourceTier. Module oracle_cache.rs. (`4DOR:L13-L22, L104-L107`)
- **P4DOR-A001** [A/CFG]: “normalised query” hashing is stated without specifying normalization algorithm (trim/case/whitespace). (`4DOR:L65-L66`)
- **P4DOR-A002** [A/STATE]: Expired entry “treat as miss (LRU evicts on next insert)” leaves active expired entry until next insert — not proactive purge. (`4DOR:L74-L75`)

### 4G: autonomous mode enablement

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4G.txt`; atoms in `PHASE-002 / DEEP-PHASE-002.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-002/IX-008 — 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing.; PHASE-002/IX-009 — 4G SPLIT_TASK is the Terminal path consumer of 3E context-budget exhaustion; sequential merge avoids 3H/S10 pick-one branches.; PHASE-002/IX-010 — 4G SUBSTITUTE_MODEL is the autonomous completion of 3C on_max_restarts policy (interactive escalates).; PHASE-002/IX-011 — 4G wraps 3A InteractiveGate without new FSM states; evidence gates from orchestrator S13; risk levels from arch-overview S6.; PHASE-002/IX-012 — 4G tree auto-advance integrates 4E TreeNavigator + BubbleUpHandler; concept-level always human.; PHASE-002/IX-013 — 4G.9 Spinning subsumes prior 3× GRC anti-spiral; personas spawn via 4A Tier-C isolation.; PHASE-002/IX-015 — Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys.; PHASE-002/IX-016 — SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*.; PHASE-002/IX-017 — 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks.; PHASE-002/IX-018 — 4G circuit breakers operate at two grains: phase auto-advances (default 10) and tree baby steps per concept (default 50).; PHASE-002/IX-019 — Goal-drift Block and watchdog timeouts both hand off to 4E BubbleUpHandler rather than inventing a third recovery owner.; PHASE-002/IX-020 — 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step.

**Behaviors**

- **P4G-E001** [E/STATUS]: Enable AUTONOMOUS auto-advance on validated INTERACTIVE pipeline. No new FSM states — only gate evaluation behavior. Prerequisites: Phase 3 interactive validated + Phase 4A–4F. (`4G:L1-L12`)
- **P4G-E002** [E/CAP]: AutonomousGate wraps Box<dyn InteractiveGate> + AutoAdvanceEvaluator + ExecutionMode + EscalationPolicy + PlanDispatchTable. Implements InteractiveGate. Interactive: delegate inner zero change. Autonomous: evidence first; human only on HIGH risk or missing/invalid evidence. (`4G:L38-L54, L312-L314`)
- **P4G-E003** [E/SEC]: Within EXECUTING, dangerous tool calls reviewed by auditor agent not human (arch-overview S6:L220) — in-state path distinct from gate transitions. (`4G:L48-L50, L433-L435`)
- **P4G-E004** [E/CAP]: Plan-dispatch states (TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING) use PlanDispatchTable from PipelineSpec; meta-dispatch (PLANNING, CONFIGURING) use static MetaDispatchTable. (`4G:L51-L54`)
- **P4G-E005** [E/FLOW]: Evidence map (S13): IDLE→PLANNING knowledge readiness ≠ UNAVAILABLE; PLANNING→TESTING_FIRST zero UNGROUNDED; TESTING_FIRST→EXECUTING cargo test exit≠0 AND no error[E (heuristic); EXECUTING→AUDITING ≥1 FAIL→PASS flip; AUDITING→MEMORIZING zero BLOCK; MEMORIZING→next DKG updated + changelog. (`4G:L56-L71, L310-L311`)
- **P4G-E006** [E/DATA]: EvidenceCheck { artifact_path, check_fn pure deterministic, description }. EvidenceVerdict Pass (`Fail{reason}`)
- **P4G-E007** [E/CFG]: EscalationPolicy defaults: anti_spiral_threshold=3 (pin L180 not L173>3); high_risk_pauses_in_both_modes=true; replan_major_always_human=true; max_auto_advances_per_phase=10. (`4G:L85-L98, L411-L424`)
- **P4G-E008** [E/EVENT]: EscalationTrigger: AntiSpiral, HighRisk, ReplanMajor, CircuitBreaker, SubstituteModelFailed, SplitTaskFailed. (`4G:L100-L107`)
- **P4G-E009** [E/CAP]: SplitTaskPlan: parent_phase, sub_tasks max 4, MergeStrategy::Sequential only, max_depth 2, current_depth. SubTask {task_id, scope, context_budget, depends_on DAG}. (`4G:L109-L133, L318-L322, L418-L422`)
- **P4G-E010** [E/LIMIT]: S10 branching pick-one is incompatible with SPLIT_TASK (needs ALL results). Parallel merge deferred. Sub-tasks sequential, write parent SSB directly. (`4G:L124-L132, L318-L322, L430-L432`)
- **P4G-E011** [E/CAP]: ModelFallback iterates fallback_chain from elai_cc.toml [agents.{role}.fallback_models]; requires new ModelPool::can_fit_llm; swap protocol unload/load/health; none fit → SubstituteModelFailed. SLO tier mismatch WARNING allowed. (`4G:L135-L151, L323-L326, L438-L439`)
- **P4G-E012** [E/SURFACE]: Mode switch CLI: `/mode autonomous` or `/mode interactive`. Takes effect at next gate evaluation, not mid-phase. Mode per-session. (`4G:L327-L333, L245-L248`)
- **P4G-E013** [E/SEC]: Evidence provenance: expected_producer role; join SSB seq→agent_id→AgentPool::get_role (new API). Prevents forged audit_report. (`4G:L286-L287, L334-L341, L425-L426`)
- **P4G-E014** [E/EVENT]: SSB action_type 'escalation' must be added to CHECK constraint (or use checkpoint payload — preferred add type). (`4G:L235-L239, L436-L437`)
- **P4G-E015** [E/TEST]: Base: 28 contract tests (6 auto-advance + 6 missing + 3 risk + 5 escalation + 4 fallback + 3 mode + 1 regression) + 8 adversarial = 36. TDD red then implement. (`4G:L210-L302, L342-L364`)
- **P4G-E016** [E/CFG]: EscalationPolicy::new enforces min=1 on max_auto_advances; SplitTaskPlan::new clamps max_depth to 2; empty fallback_chain → immediate ESCALATE no panic. (`4G:L260-L265`)
- **P4G-E017** [E/FLOW]: ReplanRequest MINOR → auditor; MAJOR → human both modes. Max 1 ReplanRequest per agent per phase. (`4G:L231-L232, L290-L291, L315-L317, L414-L415`)
- **P4G-E018** [E/OBS]: Metrics `.elai_cc/metrics/phase_4G.toml`: auto-advance accuracy 100% on deterministic scenarios; false escalation rate; mode switch latency; SPLIT_TASK overhead; SUBSTITUTE_MODEL swap time; E2E AUTONOMOUS vs INTERACTIVE (with 5s simulated human latency); gate overhead target <1ms (adversarial absolute <0.5ms in 4G.2). (`4G:L366-L396, L200-L205`)
- **P4G-E019** [E/CAP]: TreeAutoAdvancePolicy: baby_step_auto/step_integration_auto/phase_integration_auto default true; concept_integration_auto default false always human; max_auto_baby_steps_per_concept=50; bubble risk Baby/Step LOW, Phase MEDIUM, Concept HIGH. (`4G:L469-L512, L516-L551`)
- **P4G-E020** [E/FLOW]: Per-baby-step gates: TESTING→EXECUTING, EXECUTING→AUDITING, AUDITING→ACCEPTANCE, ACCEPTANCE→NEXT. BubbleUp: step auto AUTONOMOUS; phase pause MEDIUM; concept always human. Integration fail baby auto-retry once then bubble. (`4G:L480-L498`)
- **P4G-E021** [E/CAP]: StagnationPattern: Spinning (blake3 last 3 outputs or errors identical conf 1.0), Oscillation (period-2 over 4 conf 0.9), NoDrift ( (`Δdrift`)
- **P4G-E022** [E/EVENT]: SSB: stagnation_spinning_detected, stagnation_oscillation_detected, stagnation_no_drift_detected, stagnation_diminishing_returns_detected; lateral_thinking_activated/succeeded/exhausted; watchdog_timeout; goal_drift_warning/block. (`4G:L654-L656, L682-L687, L717-L719, L745-L752`)
- **P4G-E023** [E/CAP]: ThinkingPersona affinity: Hacker→Spinning; Researcher→NoDrift/DiminishingReturns; Simplifier→DiminishingReturns/Oscillation; Architect→Oscillation/NoDrift; Contrarian→ALL. suggest_persona first match; fail exclude retry; all exhausted → INTERACTIVE pause / AUTONOMOUS HALT. (`4G:L590-L688`)
- **P4G-E024** [E/SURFACE]: Persona agent files: `.claude/agents/persona-{hacker,researcher,simplifier,architect,contrarian}.md`. (`4G:L624-L631, L861-L864`)
- **P4G-E025** [E/CONC]: Parallel fan-out optional: when Contrarian-affinity AND parallel_dispatch=true, spawn all 5 Tier-C with FRESH context (no cross-persona outputs); first success wins cancel rest. (`4G:L690-L697`)
- **P4G-E026** [E/CAP]: GenerationWatchdog per baby step: idle 60s (no SSB), no_material_progress 180s, safety 600s wall. Material types: phase_transition, evidence_gate_evaluated, integration_checkpoint_success, bubble_up_typed, agent_lifecycle state change. Exclude heartbeat/status_echo/log_entry/grc_observe status. workflow_fingerprint dedup freezes progress gaming. Timeout → cancel + BubbleUpHandler. (`4G:L605-L719`)
- **P4G-E027** [E/CAP]: GoalDriftCanary: Jaccard 1− (`∩`)
- **P4G-E028** [E/SURFACE]: INTERACTIVE persona exhaustion renders InterventionCard with all 5 personas and failure reasons. (`4G:L810-L813`)
- **P4G-E029** [E/LIMIT]: Existing 3× GRC anti-spiral SUBSUMED by Spinning (threshold=3), not removed. (`4G:L451-L452, L839-L841`)
- **P4G-E030** [E/INT]: Every routing call site wires ssb, fsm, clock through single RuntimeClient; no None after construction. (`4G:L466-L466`)
- **P4G-E031** [E/CAP]: Addendum shipped: AutoAdvanceEvaluator reasoning-exhaustion 7th check when AutoAdvanceStats.evidence_rounds ≥ evidence_rounds_max from [autonomy.auto_advance] → ContinueInteractive or Halt. (`4G:L886-L898`)
- **P4G-E032** [E/CAP]: Addendum CloudSpendTracker: 50% budget → cloud_spend_warning; 20% remaining → cloud_spend_critical; below 20% INTERACTIVE EscalationRequired / AUTONOMOUS halt cloud_spend_exhausted. Budget elai_cc.toml [cloud.session_budget_usd] default 5.00 USD. (`4G:L900-L914`)
- **P4G-E033** [E/SURFACE]: Runtime-selection UX (summary-aligned exit): elai --local sidecar, --hybrid per-role, default cloud; /model live change; [agents.{role}.backend]=local (`api`)
- **P4G-E034** [E/TEST]: 4G.7 ≥9 tree_auto_advance tests; 4G.9 stagnation/persona/watchdog/goal_drift tests with adversarial cases (Oscillation≠Spinning, affinity, heartbeat non-material, fingerprint gaming, InterventionCard, Jaccard boundaries, max 1 block-retry). (`4G:L516-L534, L775-L830`)
- **P4G-E035** [E/DET]: AutoAdvanceEvaluator pure functions over artifact bytes; no LlmProvider dependency. Gate reads mode under Arc<RwLock<SessionState>>. (`4G:L257-L259, L329-L330`)
- **P4G-E036** [E/FAIL]: DEGRADED knowledge: IDLE→PLANNING auto-advances with warning (not blocked). (`4G:L280-L281`)
- **P4G-C001** [C/CFG]: Anti-spiral threshold: orchestrator.txt L180 “3x” vs L173 “same_step_count > 3”; phase pins L180 (escalate on 3rd). (`4G:L86-L89, L411-L413`)
- **P4G-C002** [C/SEC]: HIGH risk: L227 pause both modes for gates; L220 auditor-only for in-EXECUTING tools — both true, different paths. (`4G:L90-L94, L433-L435`)
- **P4G-I001** [I/SURFACE]: Runtime flags --local/--hybrid and /model are asserted by MASTER phase-index summary for 4G; this source file’s body does not re-list them after the GOAL/read list — treat MASTER as orientation unless/until codified in addendum or providers wiring. (`4G:L14-L27; MASTER 4G L871-L873`)
- **P4G-A001** [A/INT]: SessionState struct “not yet defined in any spec”; 4G adds execution_mode to whatever exists. (`4G:L331-L333`)
- **P4G-A002** [A/TEST]: TESTING_FIRST heuristic false positives (setup panics) accepted; AUDITING catches — no tighter acceptance metric. (`4G:L62-L67, L442-L444`)
- **P4G-A003** [A/CFG]: parallel_dispatch flag location on EscalationPolicy for fan-out is named in extension but not given default or TOML key. (`4G:L690-L692`)
- **P4G-A004** [A/OBS]: “Auto-advance accuracy 100%” requires 10 deterministic scenarios with ≥3 adversarial; exact scenario catalog not listed. (`4G:L371-L372, L384-L387`)

### phase-1B: BM25 / Tantivy full-text retrieval

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1B.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-001 — 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish.; PHASE-003/IX-002 — 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators.; PHASE-003/IX-011 — 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension.; PHASE-003/IX-012 — Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path.; PHASE-003/IX-017 — User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G).

**Behaviors**

- **P1B-E001** [E/STATUS]: Phase 1B lives in `.elai_cc/crates/knowledge/` after Phase 1A; goal is per-project Tantivy BM25 with sub-10 ms queries under 100K docs and zero orphan text files. (`phase-1B:L1-L12`)
- **P1B-E002** [E/LIMIT]: Binary files with null bytes in the first 8 KiB are the sole zero-orphan exception; non-UTF8 must skip with a warning, never panic. (`phase-1B:L10-L12, L32-L32, L101-L102, L232-L234`)
- **P1B-E003** [E/DEP]: Mandatory pre-reads: knowledge.txt Layer-2/deps/storage/fallback/invariants/versioning and architecture trait boundaries. (`phase-1B:L14-L21`)
- **P1B-E004** [E/CFG]: Pin exact tantivy crate version; record `INDEX_FORMAT_VERSION` (u32). Spec label `tantivy_v21` is project-only, not a tantivy API. (`phase-1B:L24-L28, L46-L52, L388-L391`)
- **P1B-E005** [E/DATA]: Schema: `path` STRING\ (`STORED untokenized; `content` TEXT\`)
- **P1B-E006** [E/DATA]: `BM25Chunk {path, content, line_start, line_end, kind}` and `ChunkKind::{Function,LineBlock,File}` are BM25-owned; distinct from DKG `DkgNode` (no content). Conversion reads file ranges DKG supplies. (`phase-1B:L70-L82, L145-L146, L368-L371`)
- **P1B-E007** [E/CAP]: `Bm25Index::{new,index_file,index_directory,query,delete_file,update_file}` with `project_id` explicit first-class input; index root `{project_root}/.elai_cc/projects/{project_id}/bm25/`. (`phase-1B:L83-L118, L372-L374`)
- **P1B-E008** [E/DATA]: `BM25Hit {path, line_start, line_end, score, chunk_kind, snippet}`; snippet is first 200 chars simple truncation, not SnippetGenerator. (`phase-1B:L107-L110, L249-L250, L375-L377`)
- **P1B-E009** [E/DATA]: `IndexStats {files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms}`. (`phase-1B:L103-L106`)
- **P1B-E010** [E/FLOW]: `index_file`: chunks provided → one doc each (Function); chunks=None → 50-line windows / 10-line overlap (LineBlock); files <50 lines → single File doc. Binary null-byte skip. (`phase-1B:L99-L102, L219-L234`)
- **P1B-E011** [E/FLOW]: `index_directory` walks recursively, skips hidden `.*` and `.elai_cc/`; DKG nodes when present else line fallback; one batch `writer.commit()`; holds IndexWriter for whole batch. (`phase-1B:L103-L118, L236-L244`)
- **P1B-E012** [E/CONC]: Single IndexWriter model: concurrent `update_file` blocks during `index_directory`; second `Bm25Index::new` for same project must Err (lock), not panic/deadlock. (`phase-1B:L114-L118, L268-L269, L360-L363`)
- **P1B-E013** [E/IDEM]: `update_file` is atomic delete_term + add_document in one `writer.commit()`; readers never see intermediate absence; target <5 s per file. (`phase-1B:L113-L114, L256-L259, L360-L363`)
- **P1B-E014** [E/DEFER]: 10-line overlap is UNGROUNDED for BM25 (term independence); kept only for vector-layer alignment and simplicity (~20% extra docs). (`phase-1B:L224-L229, L346-L350, L410-L413`)
- **P1B-E015** [E/TEST]: ≥10 synthetic tempdir tests: index/query/delete/empty-miss/zero-orphan .xyz/binary skip/incremental/project isolation/chunk kinds/path STRING delete regression on `src/deep/nested/module.rs`. (`phase-1B:L150-L203`)
- **P1B-E016** [E/OBS]: Metrics `.elai_cc/metrics/phase_1B.toml`: build/query/update latencies, orphan_files=0, tantivy_version, index_format_version, sizes/counts. (`phase-1B:L287-L329`)
- **P1B-E017** [E/TEST]: Latency gates: query p99 <10 ms dev, <50 ms when `CI=true`; update_file <5 s; warm-up query before measure; index 69 `.elai_cc/rust/` files. (`phase-1B:L288-L329, L332-L343`)
- **P1B-E018** [E/INT]: WIRING: produces `Bm25Index` + types; reachable from `index_cmd::run_index_build`, `IndexingService::run_full_index`, `run_local_repl`, `QueryPipeline`; trigger `cargo run -p elai-cli -- index build`; expect SSB `index_checkpoint`. (`phase-1B:L427-L448`)
- **P1B-E019** [E/SURFACE]: User-facing CLI surface: `elai-cli index build --project-root <path>` builds the BM25 index (WIRING trigger). (`phase-1B:L447-L448`)
- **P1B-E020** [E/LIMIT]: Circuit breaker, SearchMode, query validation, readiness, tracing, versioning scoped out to Phase 1I. (`phase-1B:L423-L424`)
- **P1B-E021** [E/DATA]: Decisions D1–D9: null-byte binary detect; chunk_kind for downstream; explicit project_id; STRING path; BM25Chunk ownership; 200-char snippet. (`phase-1B:L345-L377`)
- **P1B-E022** [E/EVIDENCE]: Grounding corrections: fn complete path not providers; crate path is `.elai_cc/crates/knowledge/` not rust/crates; Chunk from 1A is DkgNode not content Chunk. (`phase-1B:L380-L424`)
- **P1B-A001** [A/CFG]: Exact pinned tantivy crate version and its `INDEX_FORMAT_VERSION` are left to implementer decision doc at execution. (`phase-1B:L46-L52`)
- **P1B-A002** [A/FAIL]: Race when file deleted between walk and read: skip+warn required, but exact error type and IndexStats field not named. (`phase-1B:L266-L267`)
- **P1B-C001** [C/DATA]: knowledge.txt L200 `bm25_format="tantivy_v21"` conflicts with real u32 `INDEX_FORMAT_VERSION`; phase corrects and says knowledge.txt should be updated. (`phase-1B:L26-L28, L388-L391`)

### phase-1G: cross-encoder reranker

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1G.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-001 — 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish.; PHASE-003/IX-002 — 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators.; PHASE-003/IX-016 — Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families.

**Behaviors**

- **P1G-E001** [E/STATUS]: Phase 1G requires 1E pipeline; independent of 1F DROP (Ollama speculative decoding closed). (`phase-1G:L1-L9`)
- **P1G-E002** [E/CAP]: Cross-encoder `ms-marco-MiniLM-L12-v2` after A-RAG, before token-budget fit; CPU-only; target <50 ms for 50 candidates. (`phase-1G:L13-L17, L29-L34`)
- **P1G-E003** [E/CFG]: Raw `ort` + HuggingFace `tokenizers` (not fastembed-rs); pin exact ort RC; re-validate on stable 2.0.0. (`phase-1G:L32-L33, L49-L54, L77-L78`)
- **P1G-E004** [E/CFG]: `RerankerConfig`: model/tokenizer/fallback L-6/quantized paths; `skip_threshold` default 10; max_seq 512; max_queue_depth 4; intra_op_threads default 1; `ORT_ENABLE_ALL`; `enable_deterministic_compute=true`. (`phase-1G:L80-L91, L114-L119`)
- **P1G-E005** [E/CAP]: `CrossEncoderReranker` holds ort Session, Tokenizer, Semaphore, mpsc Sender, CircuitBreaker, `Arc<dyn ClockProvider>`. (`phase-1G:L78-L79`)
- **P1G-E006** [E/FLOW]: Wire `KnowledgeEngine::rerank` between A-RAG output and token-budget fitting; skip entirely in SPARSE_ONLY and BM25_ONLY. (`phase-1G:L91-L93, L187-L188`)
- **P1G-E007** [E/FLOW]: Pair encode batch → single `session.run()` → descending re-sort; skip when candidates ≤ skip_threshold (boundary tests 10 pass-through, 11 rerank). (`phase-1G:L146-L159`)
- **P1G-E008** [E/CONC]: Semaphore(1) guards session; bounded mpsc capacity 4; full channel returns unranked immediately; drain/merge/offset-split/oneshot return per caller. (`phase-1G:L63-L65, L151-L166`)
- **P1G-E009** [E/FAIL]: NaN/Inf scores must be filtered or lowest-ranked; empty candidates return empty; >512 tokens truncate. (`phase-1G:L160-L169`)
- **P1G-E010** [E/RECOVERY]: L-12 breaker: 3 failures / 60 s window / 30 s half-open via ClockProvider; cascade lazy L-6 then skip-rerank A-RAG order. (`phase-1G:L181-L202`)
- **P1G-E011** [E/DEFER]: L-6 intermediate tier requires `workflow/reasoning/phase-1G-spec-amendment.md`; if rejected, keep skip-only final fallback per knowledge.txt L149. (`phase-1G:L38-L38, L83-L84, L180-L186, L298-L299`)
- **P1G-E012** [E/EVENT]: SSB `action_type='knowledge_degradation'` stage=`rerank` on breaker open/close; SSB write failure non-fatal (log, continue). (`phase-1G:L188-L197`)
- **P1G-E013** [E/OBS]: DEBUG `StageTrace` stage `rerank` in QueryTrace; canary logs reranker latency. (`phase-1G:L189-L189, L198-L198, L220-L220`)
- **P1G-E014** [E/FLOW]: Startup canary: embed + BM25 + rerank within 5 s; reranker failure → DEGRADED (not UNAVAILABLE) with reranker skipped. (`phase-1G:L210-L228`)
- **P1G-E015** [E/CFG]: Start fp32; INT8 only if baseline p50 >30 ms and NDCG@10 delta <0.5; script `scripts/quantize_cross_encoder.py` → `.elai_cc/models/cross-encoder-int8.onnx`. (`phase-1G:L56-L59, L253-L259`)
- **P1G-E016** [E/TEST]: Optimization harness: thread sweep 1/2/4/8 with ranking-order stability; skip thresholds 5/10/15/20/30 by NDCG@10 (delta >0.02); fixture `tests/fixtures/reranker_benchmark.json` (1 query, 50 code candidates, ≥10 relevant). (`phase-1G:L230-L266`)
- **P1G-E017** [E/OBS]: Metrics `.elai_cc/metrics/phase_1G.toml`: baseline/thread_sweep/int8/threshold_sweep/final latencies, NDCG, model variant, cold start, queue wait p99 under 4 agents. (`phase-1G:L242-L285, L327-L338`)
- **P1G-E018** [E/LIMIT]: If final p50 still >50 ms after all levers, append data to phase-1G-spec-amendment.md; do not invent a false pass. (`phase-1G:L271-L273, L287-L288`)
- **P1G-E019** [E/LIMIT]: No unwrap/panic; injectable ClockProvider only (no Instant::now); CPU EP only (no CUDA/TensorRT). (`phase-1G:L117-L123, L311-L313`)
- **P1G-E020** [E/FAIL]: Lazy L-6 cold-start 5–15 s acceptable during degradation (no SLA on breaker fallback path). (`phase-1G:L122-L122, L208-L208`)
- **P1G-E021** [E/EVIDENCE]: Model sizes ~120 MB fp32 / ~30 MB int8; L-12 MRR@10 39.02 vs L-6 36.81; ort is RC (conditional pin). (`phase-1G:L29-L37`)
- **P1G-A001** [A/CFG]: Optimal skip_threshold after sweep may replace default 10; exact production value not fixed until measurement. (`phase-1G:L61-L62, L261-L266`)
- **P1G-A002** [A/TEST]: Single-fixture NDCG; production threshold re-evaluation deferred to Phase 1I real query logs. (`phase-1G:L240-L240`)
- **P1G-A003** [A/CFG]: Exact pinned ort RC version number not fixed in phase body beyond rc.12 validation note. (`phase-1G:L32-L33, L112-L112`)
- **P1G-C001** [C/CAP]: Spec L149 defines only skip-rerank fallback; phase adds L-6 tier via amendment — not yet controlling without approval. (`phase-1G:L38-L38, L183-L186`)

### phase-3H: DAG session handoff and lossless trimming

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 3 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3H.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-003 — 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget.; PHASE-003/IX-004 — 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry.; PHASE-003/IX-005 — 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots.; PHASE-003/IX-013 — Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path).; PHASE-003/IX-020 — Handoff 85% (3H) aligns with 4C PreCompact checkpoint-at-85% theme in index.

**Behaviors**

- **P3H-E001** [E/STATUS]: Phase 3H needs 3G PlanSkeleton, 3B SSB, 3E context budget, providers::token_count; CMV-derived DAG handoff (arxiv 2602.22402). (`phase-3H:L1-L18`)
- **P3H-E002** [E/FLOW]: Checkpoint after every GRC step, tool call, phase transition; handoff at token budget threshold default 85%. (`phase-3H:L16-L17, L447-L450`)
- **P3H-E003** [E/DATA]: Per-session `session_dag.db`: `snapshots` + `snapshot_context` tables; indexes session/tag/stubbed; version table for migration. (`phase-3H:L45-L74, L170-L176`)
- **P3H-E004** [E/DATA]: Snapshot fields: snap_id UUID, parent_id, session_id, fsm_state, phase, step, findings/code_changes/tests_written msgpack blobs, commit_hashes, created_at, tag. (`phase-3H:L47-L60, L84-L97`)
- **P3H-E005** [E/DATA]: Expanded per-message `snapshot_context` rows: seq, role CHECK system/user/assistant/tool, content BLOB, is_stubbed, stub_ssb_seq — diverges from orchestrator.txt blob schema. (`phase-3H:L27-L32, L62-L70, L199-L210`)
- **P3H-E006** [E/DATA]: Types: SessionDag, Snapshot, TrimmedMessage, Stub{original_type,original_tokens,summary_hash,ssb_seq}, BranchRef, MergeResult::{Winner,Conflict}. (`phase-3H:L78-L126`)
- **P3H-E007** [E/CFG]: Not WAL: default DELETE journal for single writer; PRAGMA integrity_check on open; foreign_keys=ON; EntropyProvider for UUIDs; ClockProvider for timestamps. (`phase-3H:L78-L79, L151-L154, L175-L176, L188-L197`)
- **P3H-E008** [E/CAP]: CRUD: save_snapshot transactional; load_snapshot seq-ordered; latest_snapshot(tag?); parent_chain cycle-guarded root-to-tip. (`phase-3H:L220-L252`)
- **P3H-E009** [E/CAP]: Three-pass trim: identify tool results >stub_threshold; stub with Stub+blake3+ssb_seq; validate SSB or un-stub; never stub user/assistant or Slots 1–3. (`phase-3H:L271-L310`)
- **P3H-E010** [E/CFG]: `TrimConfig {stub_threshold, model_context_window, allocated_slots_tokens, output_reserve}`; target = window − allocated − reserve; stub_threshold default 500, enforced [100,1000]. (`phase-3H:L272-L274, L306-L308, L316-L323`)
- **P3H-E011** [E/FAIL]: TrimBudgetExceeded if trimmed total still > target; DatabaseCorrupted/DatabaseNotFound/CycleDetected/BranchLimitExceeded typed errors. (`phase-3H:L151-L154, L181-L186, L241-L242, L285-L285, L356-L356`)
- **P3H-E012** [E/CAP]: Branch API: create_branch (copy parent context, tag `branch:{name}`), list_branches, merge_branch (new trunk snapshot, abandon tags), close_branch; max 3 active; names alphanumeric+hyphen max 64. (`phase-3H:L333-L379`)
- **P3H-E013** [E/EVENT]: Merge records SSB via existing `checkpoint` with payload event=branch_merge (or amended `branch_abandon` if chosen). (`phase-3H:L33-L36, L344-L347`)
- **P3H-E014** [E/RECOVERY]: recover_stubs / recover_single_stub: SSB lookup + blake3 verify; StubRecoveryFailed on missing seq or hash mismatch. (`phase-3H:L389-L430`)
- **P3H-E015** [E/FLOW]: create_handoff trims + tag=handoff; load_handoff prefers branch tip over trunk when branches exist; commit_hashes persist. (`phase-3H:L437-L497`)
- **P3H-E016** [E/DET]: 100-run same-seed identity for snap_ids and trimming; parent chain always acyclic. (`phase-3H:L474-L521`)
- **P3H-E017** [E/OBS]: Measure save/load <50 ms (100 msgs), trim 500 msgs <10 s, token reduction >0.3 tool-heavy, branch ops <100 ms, DAG size <10 MB /1000 snaps. (`phase-3H:L500-L508`)
- **P3H-E018** [E/LIMIT]: Abandoned branch snapshots tagged not deleted (logical SSB exclusion assumption); physical SSB delete forbidden under append-only. (`phase-3H:L37-L40, L348-L349, L370-L379`)
- **P3H-A001** [A/DATA]: SPEC AMENDMENT #1: per-message schema vs orchestrator blob; fallback to blob if spec not updated before implement. (`phase-3H:L27-L32, L199-L210`)
- **P3H-A002** [A/EVENT]: SPEC AMENDMENT #2: new `branch_abandon` vs reuse `checkpoint` unresolved before 3H.4. (`phase-3H:L33-L36, L344-L347`)
- **P3H-A003** [A/DATA]: SPEC AMENDMENT #3: "only winning SSB kept" — phase assumes logical query filter, not physical delete. (`phase-3H:L37-L40, L348-L349`)
- **P3H-A004** [A/STATE]: MergeResult::Conflict for overlapping code_changes is UNGROUNDED optional; orchestrator says always pick winner. (`phase-3H:L120-L126, L367-L369, L385-L387`)
- **P3H-A005** [A/DATA]: blake3 for stub summary_hash is UNGROUNDED (orchestrator silent); sha256 alternative open. (`phase-3H:L324-L326`)
- **P3H-A006** [A/DEP]: TrimConfig needs 3E total_allocated_tokens method; PRE_GATE must verify — exact API name soft. (`phase-3H:L155-L158, L268-L269`)
- **P3H-C001** [C/DATA]: snapshot_context design intentionally diverges from orchestrator.txt L293-294 blob layout until amendment lands. (`phase-3H:L27-L32, L207-L210`)

### phase-4D-SURGICAL: surgical decomposition for /task

- **Status**: shipped (per source STATUS atom; not independently verified in this compile)
- **Anchors**: source `docs/specs/guide/steps/phase-4D-SURGICAL.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-010 — 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds.; PHASE-003/IX-011 — 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension.; PHASE-003/IX-014 — Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder.; PHASE-003/IX-016 — Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families.; PHASE-003/IX-017 — User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G).; PHASE-003/IX-019 — 4H MSI tracks SURGICAL/GREENFIELD modes; surgical CLI path is a related but separate C-30 feature.

**Behaviors**

- **P4DS-E001** [E/STATUS]: SHIPPED in C-30 (SCHEMA_VERSION 61→62); record lives in surgical_decomposer.rs; decisions C-30 DECISION-1..15. (`phase-4D-SURGICAL:L4-L7, L30-L33`)
- **P4DS-E002** [E/SURFACE]: `/task --surgical "<prompt>"` replaces ConceptDecomposition with DKG blast-radius surgical plan; flag via `parse_task_flags` commands.rs:2414. (`phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89`)
- **P4DS-E003** [E/INT]: live_cli intercept L2632 replaces DefaultWebResearchRunner with SurgicalDecomposer BEFORE runner construction. (`phase-4D-SURGICAL:L20-L21, L88-L89, L110-L112`)
- **P4DS-E004** [E/DATA]: Types: SurgicalPlan, AffectedSet{files,symbols}, ComplexityClass{Simple,Medium,Complex}, RegressionGate, TestBaseline, SurgicalStep. (`phase-4D-SURGICAL:L38-L44, L98-L100`)
- **P4DS-E005** [E/CFG]: Complexity: Simple files≤3 AND loc≤200 → cap 1; Medium (files≤8 OR loc≤500) and not Simple → cap 4; Complex otherwise → cap 8. (`phase-4D-SURGICAL:L49-L51, L65-L67`)
- **P4DS-E006** [E/FLOW]: `SurgicalDecomposer::decompose(task,dkg,mode)`: blast-radius query_callers+dependents+data_flow depth≤MAX_C30_DEPTH=8 cycle-guarded HashSet; classify; inject RegressionGate from DKG test edges; constrain steps. (`phase-4D-SURGICAL:L56-L67`)
- **P4DS-E007** [E/INT]: DkgQuerier gains query_callers, query_dependents, query_data_flow, query_subgraph; only DkgQuerierAdapter has real SQLite traversal (defaults empty). (`phase-4D-SURGICAL:L22-L24, L101-L102, L113-L116`)
- **P4DS-E008** [E/CFG]: SURGICAL_LOC_WARN_THRESHOLD=500; SURGICAL_LOC_ESCALATE_THRESHOLD=1000. (`phase-4D-SURGICAL:L72-L74`)
- **P4DS-E009** [E/FLOW]: loc>500: emit surgical_loc_warning; INTERACTIVE → ConfirmationRequired; AUTONOMOUS confirm_above_loc=500 blocks until confirm OR refuse_above_loc=1000 aborts. (`phase-4D-SURGICAL:L76-L81`)
- **P4DS-E010** [E/EVENT]: SSB SCHEMA 62: SurgicalPlanCreated{complexity_class,blast_radius_files,total_loc_estimate,step_count}; SurgicalLocWarning{loc_estimate,threshold}. (`phase-4D-SURGICAL:L27-L28, L90-L93, L105-L105`)
- **P4DS-E011** [E/SURFACE]: Dashboard SurgicalDrawer shows ComplexityClass + AffectedSet on plan creation. (`phase-4D-SURGICAL:L122-L122`)
- **P4DS-E012** [E/OBS]: observable_effect: `elai /task --surgical "rename Foo to Bar in core"` → surgical_plan_created within 2 s + SurgicalDrawer update. (`phase-4D-SURGICAL:L119-L122`)
- **P4DS-E013** [E/LIMIT]: stub_ban: step cap hard (no Simple plan with >1 step); no .unwrap() in library adapter path. (`phase-4D-SURGICAL:L113-L118`)
- **P4DS-E014** [E/TEST]: DETERMINISTIC_CHECK: rg types/SSB/live_cli/LOC constants; cargo test c30_surgical_oracle_contract. (`phase-4D-SURGICAL:L124-L143`)
- **P4DS-E015** [E/DEP]: Consumes ExecutionMode (context_budget.rs:865), DKG db path, EscalationPolicy autonomous confirm/refuse fields. (`phase-4D-SURGICAL:L17-L18, L25-L26, L106-L109`)
- **P4DS-A001** [A/FLOW]: Steps exceeding cap "merged or dropped with explanatory note" — note surface/format not specified. (`phase-4D-SURGICAL:L65-L67`)
- **P4DS-A002** [A/SURFACE]: SurgicalDrawer field layout beyond ComplexityClass+AffectedSet not specified in this file. (`phase-4D-SURGICAL:L122-L122`)

### phase-4E: Pipeline Factory L2 + DecompositionTree

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-4E.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-006 — 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth.; PHASE-003/IX-007 — 4E novel_step_ids consumed by 4F; visual_check adapters by 4I.; PHASE-003/IX-008 — 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks.; PHASE-003/IX-009 — 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1.; PHASE-003/IX-014 — Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder.; PHASE-003/IX-015 — RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition.; PHASE-003/IX-016 — Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families.; PHASE-003/IX-018 — 4E integration time ladder matches 4B IntegrationTestRunner levels in index.

**Behaviors**

- **P4E-E001** [E/STATUS]: Phase 4E after 4D; pure-Rust orchestrator L2, no LLM/agent process; IMPROVED 2026-04-06. (`phase-4E:L1-L9`)
- **P4E-E002** [E/EVIDENCE]: AutoB2G arxiv 2603.26005: adopt DAG + T_DAG + structured delta; reject SOCIA LLM repair; strategies REORDER + SUBSTITUTE only. (`phase-4E:L10-L28, L127-L131`)
- **P4E-E003** [E/DATA]: Newtypes PipelineId/StepId/NodeIndex/StateId/DataRef; ExecutablePipeline with fsm_states, dependency_graph, novel_step_ids, agent_configs, rule_configs, resource_estimate, test_harness, repair_log, output_dir. (`phase-4E:L41-L67`)
- **P4E-E004** [E/DATA]: Supporting: AgentPhaseConfig, RuleBinding, DependencyGraph, DagNode, PipelineFsmState (compiled Predicate gates, is_terminal), ResourceEstimate, TestHarnessSkeleton, RepairAction/Outcome, ValidationError + 8 kinds. (`phase-4E:L69-L153`)
- **P4E-E005** [E/TEST]: Eight structural checks: tool exists, model available, acyclic Kahn, hardware fit, measurable metrics, compile_predicate, novel steps, visual MediaRenderAdapter mapping. (`phase-4E:L155-L167, L381-L393`)
- **P4E-E006** [E/FAIL]: Non-repairable escalate immediately: resource overflow, unmeasurable metric, missing transitive deps with ambiguous resolution; no REORDER/SUBSTITUTE attempt. (`phase-4E:L24-L26, L272-L273, L400-L401`)
- **P4E-E007** [E/RETRY]: DagRepairLoop max T_DAG=10; REORDER for cycles (weakest dep remove), SUBSTITUTE for missing tool same category compatible IO; structural hash fixpoint → escalate; REORDER before SUBSTITUTE same round. (`phase-4E:L19-L23, L269-L276, L336-L339, L398-L404`)
- **P4E-E008** [E/CAP]: Nine stubs/APIs: PipelineBuilder, StructuralValidator, DagRepairLoop, FsmStateGenerator, TestHarnessGenerator, ResourceEstimator, NoveltyDetector, AgentConfigurator, RuleInstaller. (`phase-4E:L183-L199`)
- **P4E-E009** [E/FLOW]: Assembly: PipelineSpec → DependencyGraph → validate → Phase Sizing (3.5) → repair → ExecutablePipeline msgpack under `.elai_cc/pipelines/{id}/`. (`phase-4E:L361-L413`)
- **P4E-E010** [E/CFG]: Phase sizing: complexity = files*avg_file_tokens + tests*avg_test_tokens + contracts*200; split if > available_for_work * 0.6; max depth 2; merge adjacent only; uses PlanSkeleton::split_phase preserve_contracts. (`phase-4E:L365-L376`)
- **P4E-E011** [E/TEST]: ≥30 base tests (validation/repair/novel/FSM/harness/resource/visual/e2e/proptest) + ≥10 adversarial; assembly <1 s for 10-phase; 100-phase <5 s adversarial. (`phase-4E:L256-L355, L424-L427`)
- **P4E-E012** [E/OBS]: Metrics phase_4E.toml: assembly times, repair success ≥50% on ≥10 broken specs, strategy distribution, harness coverage. (`phase-4E:L437-L463`)
- **P4E-E013** [E/INT]: Every routing participant must wire ssb, fsm, clock through single RuntimeClient; no None after construction. (`phase-4E:L473-L473`)
- **P4E-E014** [E/CAP]: DecompositionTree extension is primary execution artifact (typed data, not prompts); TreeNavigator pure Rust flow control. (`phase-4E:L476-L490`)
- **P4E-E015** [E/DATA]: DecompositionTree{tree_id,root,navigator,execution_log,bubble_up_log,integration_checkpoints}; TreeNavigatorState with completed/failed/blocked/traversal_order; FailureRecord; BabyStepResult/Outcome; BubbleUpEvent/Action/Policy; IntegrationCheckpoint. (`phase-4E:L494-L577`)
- **P4E-E016** [E/CFG]: BubbleUpPolicy defaults: baby 3, step 2, phase 1, concept 1 retries; on_concept_fail=Escalate. (`phase-4E:L562-L568`)
- **P4E-E017** [E/FLOW]: next_baby_step: precomputed topo order; skip completed/blocked; require pre_gates + dependencies; DepthFirstNavigator default. (`phase-4E:L579-L618`)
- **P4E-E018** [E/FLOW]: Per baby-step 8-stage cycle: context Slot 3 → PRE_GATE → TESTING_FIRST → EXECUTING → AUDITING → ACCEPTANCE → INTEGRATION → EXIT. (`phase-4E:L620-L671`)
- **P4E-E019** [E/STATE]: Context: BabyStepGrounding in Slot 3 (~500+200 tokens); EXECUTING context only grounding+task+test file; max_lines via TaskSpec; T1/T2/T3 hooks + incremental compile. (`phase-4E:L624-L644, L809-L823`)
- **P4E-E020** [E/RETRY]: Acceptance fail: ResearchAlternative with FlashResearch failure context while attempts < max; else BubbleUpHandler cascade RedecomposeStep/ReplanPhase/ReplanConcept/Escalate. (`phase-4E:L652-L707`)
- **P4E-E021** [E/EVENT]: Bubble-up logged SSB action_type=`replan_request` with tree context; INTERACTIVE escalate to human; AUTONOMOUS ReplanRequest(MAJOR). (`phase-4E:L707-L709`)
- **P4E-E022** [E/TEST]: Integration checkpoints: baby cargo check <5 s; step module tests <30 s; phase crate <60 s; concept workspace <120 s; deferred skips until siblings complete. (`phase-4E:L711-L735, L866-L891`)
- **P4E-E023** [E/RECOVERY]: TreeNavigatorState msgpack after every baby-step completion; corrupt checkpoint recovers via re-traverse from SSB. (`phase-4E:L515-L515, L849-L860`)
- **P4E-E024** [E/TEST]: Extension steps 4E.7–4E.12: ≥18 navigation/bubble-up tests + ≥15 cycle tests → 33+ green; traversal deterministic 100 runs. (`phase-4E:L739-L864`)
- **P4E-E025** [E/OBS]: 4E.12 measures baby steps/idea, FSM cycles, bubble-up rate, integration pass rate, traversal overhead <5%, context p50/p95 target <2K, ResearchCache savings. (`phase-4E:L893-L911`)
- **P4E-E026** [E/LIMIT]: No LLM in L2 path (grep ban); L2 stays in orchestrator crate. (`phase-4E:L199-L221, L428-L428`)
- **P4E-E027** [E/DEP]: Marks novel_step_ids for 4F FEASIBILITY_PROBE; visual_check maps to 4I MediaRenderAdapter. (`phase-4E:L65-L67, L163-L167, L470-L470`)
- **P4E-I001** [I/FLOW]: 5-stage list includes Research and Feasibility Probe as surrounding stages; pure L2 owns assembly/validate/size/repair inside that pipeline. (`phase-4E:L361-L364`)
- **P4E-A001** [A/DATA]: ValidationErrorKind lists CycleDetected while 4E.5 text says CyclicDependency — exact enum variant name open. (`phase-4E:L144-L153, L377-L379`)
- **P4E-A002** [A/CFG]: CONTRACT_OVERHEAD default 200 and SPLIT_RATIO 0.6 named; config key surface for operator override not specified. (`phase-4E:L367-L371`)
- **P4E-A003** [A/FAIL]: "weakest dependency" selection rule for REORDER not formally defined (edge weight? recency?). (`phase-4E:L20-L20`)
- **P4E-A004** [A/TEST]: Integration test_command language-appropriate equivalent for non-Rust not fully specified. (`phase-4E:L870-L871`)

### phase-index-3A-4J: concise phase index

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-index-3A-4J.txt`; atoms in `PHASE-003 / DEEP-PHASE-003.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-003/IX-003 — 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget.; PHASE-003/IX-004 — 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry.; PHASE-003/IX-005 — 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots.; PHASE-003/IX-006 — 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth.; PHASE-003/IX-007 — 4E novel_step_ids consumed by 4F; visual_check adapters by 4I.; PHASE-003/IX-008 — 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks.; PHASE-003/IX-009 — 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1.; PHASE-003/IX-010 — 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds.; PHASE-003/IX-012 — Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path.; PHASE-003/IX-013 — Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path).; PHASE-003/IX-014 — Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder.; PHASE-003/IX-015 — RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition. …

**Behaviors**

- **PIDX-E001** [E/STATUS]: Index purpose: concise LLM context for phases 3A–4J (includes 3J); full detail in individual phase files; Phase 3 requires 0–2 complete. (`phase-index-3A-4J:L1-L4`)
- **PIDX-E002** [E/STATE]: 3A full FSM 9 states incl CONFIGURING; 7 DAIRA evidence gates; INTERACTIVE APPROVE/REJECT/MODIFY/REDIRECT; risk LOW/MEDIUM/HIGH; ReplanRequest MINOR/MAJOR; AUTONOMOUS deferred to 4G. (`phase-index-3A-4J:L6-L16`)
- **PIDX-E003** [E/CAP]: 3B SSB: SQLite WAL single-writer; 17 action_types; r2d2 pool; typed msgpack payloads; mpsc writer + broadcast; append-only + update_hook. (`phase-index-3A-4J:L18-L27`)
- **PIDX-E004** [E/CAP]: 3C agents Tier A in-process: 8 roles; 4 death classes; 5 msgpack checkpoint files; supervision max_restarts; anti-spiral 3x+blake2s; heartbeat 5s/15s. (`phase-index-3A-4J:L29-L40`)
- **PIDX-E005** [E/CAP]: 3D BSP: LastValue/Topic/EphemeralValue/UntrackedValue; BinaryOp dropped; JoinSet parallel; Untracked never checkpointed. (`phase-index-3A-4J:L42-L53`)
- **PIDX-E006** [E/CFG]: 3E 8 slots; dynamic 15/40/25/20; floors Ssb≥200 Grc≥500 Out≥2000; 2 cache breakpoints; adaptive reduction to ESCALATE/SPLIT_TASK; assembly <50 ms. (`phase-index-3A-4J:L55-L64`)
- **PIDX-E007** [E/CAP]: 3F rules DATA in SQLite; 5 predicate types; Verdict ALLOW/WARN/BLOCK/ESCALATE; lifecycle PROPOSED→…→RETIRED; ~33 seed rules; ≥254 static tests. (`phase-index-3A-4J:L66-L78`)
- **PIDX-E008** [E/CAP]: 3G skeleton+detail; goal≤120; Kahn deps; stub_reference; coherence auditor checks. (`phase-index-3A-4J:L80-L88`)
- **PIDX-E009** [E/CAP]: 3H handoff summary matches detailed phase: DAG, 3-pass trim, max 3 branches, 85% handoff, blake3 stubs. (`phase-index-3A-4J:L90-L98`)
- **PIDX-E010** [E/TEST]: 3I DST: digital-sibling oracle; proptest-state-machine 10K+; madsim faults; FsmEvent set including SuperstepBoundary. (`phase-index-3A-4J:L100-L108`)
- **PIDX-E011** [E/CAP]: 3K RAG hardening: AST adaptive chunking; multi-query expansion; HyDE via sidecar; tiers FAST/STANDARD/DEEP; 30-query benchmark. (`phase-index-3A-4J:L111-L128`)
- **PIDX-E012** [E/CAP]: 3J vLLM+DoLa+ActAdd+MCP+AppContainer+typed 17 SSB; compose/local/pipeline/wire substeps; backend modes local\ (`api\`)
- **PIDX-E013** [E/SURFACE]: 3J.LOCAL/4G model UI: CLI `--local`/`--api`/`--hybrid`; runtime `/model`; config `[agents.{role}.backend]` and `[backend] mode`. (`phase-index-3A-4J:L156-L160, L310-L318`)
- **PIDX-E014** [E/CAP]: 4A Tier C isolation: 15 IPC messages; agent binary no LLM/network/tools; Linux seccomp+namespaces; Windows AppContainer; ToolFilter at IPC. (`phase-index-3A-4J:L177-L188`)
- **PIDX-E015** [E/CAP]: 4B tools: git2, 5-tier validation, DAIRA markers, 3-tier edit match, ShortCoder, 9 failure taxonomy, outline >300 lines, IntegrationTestRunner levels. (`phase-index-3A-4J:L190-L206`)
- **PIDX-E016** [E/CAP]: 4C hooks PreToolUse/PostToolUse/Stop/PreCompact; T1/T2/T3; generic "tool call rejected by policy" sanitization. (`phase-index-3A-4J:L208-L215`)
- **PIDX-E017** [E/CAP]: 4D L1 Research+Designer+recursive Concept→BabySteps; ResearchTree phi/psi; SourceQuality 5-tier; BabyStepSpec 8 blocks; ResearchCache LRU. (`phase-index-3A-4J:L217-L238`)
- **PIDX-E018** [E/CAP]: 4E L2 index summary: 5-stage, T_DAG=10, 8 validation, Phase Sizing 0.6/depth2, DecompositionTree+BubbleUp+integration times. (`phase-index-3A-4J:L240-L269`)
- **PIDX-E019** [E/CAP]: 4F feasibility: skip if zero novel AND ≥20% margin; GRC 3/600; 25s/30s kills; per-concept baby-step probes. (`phase-index-3A-4J:L271-L288`)
- **PIDX-E020** [E/SURFACE]: 4G AutonomousGate; `/mode autonomous\ (`interactive` next-gate; 6 evidence checks; SPLIT_TASK max 4 depth 2; TreeAutoAdvancePolicy circuit 50 baby steps/concept.`)
- **PIDX-E021** [E/CAP]: 4H self-mod Classes A/B; Layers 1/1A/2/3; MSI bands; AllowedConfigKey; 4 safety layers; PipelineMode SURGICAL/GREENFIELD. (`phase-index-3A-4J:L321-L353`)
- **PIDX-E022** [E/CAP]: 4I media validator conditional; Tier1 tools + Tier2 VLM advisory; CLAP audio; MediaRenderAdapter loop. (`phase-index-3A-4J:L355-L371`)
- **PIDX-E023** [E/CAP]: 4J Empire feature-gated; FactoryLine client; 4 ElaiNeuron capabilities; graceful degrade without Empire. (`phase-index-3A-4J:L373-L388`)
- **PIDX-E024** [E/SURFACE]: Slash/CLI surfaces named in index: INTERACTIVE gate verbs; `/plan` (3J.WIRE.PLAN); `/mode`; `/model`; `elai --local/--hybrid`; `elai.bat`. (`phase-index-3A-4J:L12-L12, L166-L170, L300-L317`)
- **PIDX-I001** [I/STATUS]: Index is compression of many phase files; atoms here are index-level claims, not substitutes for deep chunk ledgers of those phases. (`phase-index-3A-4J:L1-L2`)
- **PIDX-A001** [A/STATUS]: Index lists 3K before 3J narrative order; dependency text says 3K needs 1A–1I and 3J for HyDE only — numbering presentation vs implement order open. (`phase-index-3A-4J:L111-L127`)
- **PIDX-C001** [C/EVENT]: 3B says 17 action_types while 3H source debates adding branch_abandon — index does not resolve enum growth. (`phase-index-3A-4J:L20-L21, L90-L98`)

### 1E: CatRAG + A-RAG (query pipeline assembly)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1E.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-001 — 1E composes 1B BM25 + 1C vectors + 1D PageRank/DKG; leaves explicit stubs for 1G rerank and 3E token_fit.; PHASE-004/IX-002 — 1E density guard and circuit-breaker skips feed 1I resilience/mode cascade and QueryTrace stage list.; PHASE-004/IX-009 — 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline.; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.

**Behaviors**

- **P1E-E001** [E/STATUS]: Phase 1E assembles full retrieval: query → BM25 → vector → PageRank → CatRAG → A-RAG. Prerequisites: 1D complete (PageRank + DKG); depends on 1B BM25 and 1C vectors. (`1E:L1-L9`)
- **P1E-E002** [E/CAP]: CatRAG is query-aware graph traversal over DKG (evidence-chain expansion). A-RAG is hierarchical retrieval with snippet progressive disclosure. Target 55–87% token savings vs full-chunk; sub-second latency goal; interactive budget ≤2s total. (`1E:L7-L11, L61-L62, L438-L442`)
- **P1E-E003** [E/EVIDENCE]: Grounding papers: CatRAG arXiv 2602.01965 (interface `workflow/papers/catrag-interface.md`); A-RAG arXiv 2602.03442 (interface `workflow/papers/arag-interface.md`); knowledge.txt Layers 4–5 + §3 query pipeline. (`1E:L13-L24`)
- **P1E-E004** [E/DATA]: CatRAG paper→ELAI map: entity→DkgNode; passage→file chunks; relation edges→calls/references/imports/implements; synonym edges→cosine>0.8; context edges→node-to-file; OpenIE/NER not used (tree-sitter + regex/fuzzy symbol match). (`1E:L26-L35`)
- **P1E-E005** [E/CAP]: A-RAG adaptation: keyword_search→BM25+snippets; semantic_search→dual-vector+snippets; chunk_read→disk file/line range; agent loop→ReAct with ollama/external LLM; one tool per step (no parallel tool calls). (`1E:L37-L43`)
- **P1E-E006** [E/FLOW]: Canonical stage order (knowledge.txt L104-105): BM25+SPLADE(RRF) → dual-vector rerank → PageRank filter → CatRAG expand → A-RAG filter → cross-encoder rerank → token-budget fit → context. Cross-encoder is Phase 1G; token-budget is Phase 3E; this phase stubs both. (`1E:L45-L49, L375-L377`)
- **P1E-E007** [E/FAIL]: R30: limit CatRAG LLM to N_seed=5, K_edge=15; cache scores. R31: if density <0.01 skip CatRAG and pass PageRank→A-RAG; add synonym edges >0.8. R32: A-RAG max_loops=15 + token budget + context tracker. R33: snippets ±2 code lines. R34: measure per-layer latency. (`1E:L51-L62`)
- **P1E-E008** [E/CFG]: CatRagConfig defaults: damping=0.5, epsilon=0.2, beta=2.5, n_seed=5, k_edge=15, min_graph_density=0.01. (`1E:L76-L83`)
- **P1E-E009** [E/DATA]: CatRagOverlay is sparse only: `reset_distribution: Vec<(usize,f64)>` and `edge_weight_deltas: Vec<(usize,usize,f64)>`; never full adjacency matrix copies. (`1E:L84-L87, L157-L159, L252`)
- **P1E-E010** [E/CAP]: CatRag API: `new(config)`, `compute_overlay(query, snapshot, pagerank_scores) -> Result<CatRagOverlay>`; modifies PPR inputs without mutating base graph. (`1E:L88-L97, L265-L269`)
- **P1E-E011** [E/CFG]: AragConfig defaults: max_loops=15, max_token_budget=128000, top_k=5. (`1E:L100-L104`)
- **P1E-E012** [E/CAP]: RetrievalTool enum: KeywordSearch{keywords,top_k}, SemanticSearch{query,top_k}, ChunkRead{chunk_ids}. RetrievalResult: chunk_id, snippet, score, tool_used. (`1E:L105-L115`)
- **P1E-E013** [E/STATE]: ContextTracker holds per-query HashSet of read_chunks; created fresh each query_pipeline call; not shared/concurrent. (`1E:L119-L121, L160-L161, L310-L313, L333-L334`)
- **P1E-E014** [E/CAP]: AragOrchestrator: `new`, `execute_tool`, `query_pipeline` (full BM25→vectors→PageRank→CatRAG→A-RAG wiring). (`1E:L116-L134`)
- **P1E-E015** [E/CAP]: QueryPipeline holds bm25 + catrag + arag (+ vector/pagerank from prior phases); `query` returns `Vec<PipelineStage{stage,results,latency_ms}>`. Modules: catrag, arag, pipeline wired in knowledge lib.rs. (`1E:L135-L150`)
- **P1E-E016** [E/TEST]: 1E.1 PRE_GATE: 1D tests ok; DkgSnapshot, Bm25Index, pagerank symbols present. DETERMINISTIC: cargo check + three pub mod greps. (`1E:L68-L72, L152-L156`)
- **P1E-E017** [E/TEST]: 1E.3 CatRAG tests: symbolic anchoring weight formula epsilon/ (`files`)
- **P1E-E018** [E/TEST]: 1E.3 A-RAG tests: keyword score sum(count*\ (`keyword\`)
- **P1E-E019** [E/TEST]: 1E.3 pipeline tests: stages order bm25/vector/pagerank/catrag/arag; latency_ms>0; empty query no panic. Contracts: overlay apply/revert; RetrievalResult fields non-empty. All new tests fail on stubs. (`1E:L207-L223`)
- **P1E-E020** [E/FLOW]: Symbolic anchoring: split whitespace/camelCase/snake_case; case-insensitive substring fuzzy match; reset_weight=epsilon*\ (`files_containing\`)
- **P1E-E021** [E/FLOW]: Dynamic edge weighting: top N_seed by reset; prune neighbors by embedding similarity if >K_edge; heuristic scores by edge type calls=8 implements=7 references=5 imports=3 + target relevance + file proximity; map via phi; LLM scoring optional via circuit breaker only. (`1E:L240-L252`)
- **P1E-E022** [E/FLOW]: Key-fact enhancement: T_seed = query symbols + direct callees; boost symbol→file context edges by *(1+beta). Pure algorithmic. (`1E:L254-L258`)
- **P1E-E023** [E/FAIL]: Density = \ (`edges\`)
- **P1E-E024** [E/CAP]: Keyword tool: merge BM25 per keyword; snippet sentences with keyword ±2 lines; top_k snippets not full chunks. Semantic: vector query, 3× oversample, max-score per file. Chunk read: full content or previously-read placeholder. (`1E:L290-L308`)
- **P1E-E025** [E/FLOW]: Phase 1E A-RAG orchestration is deterministic fixed pipeline keyword→semantic→chunk_read; full ReAct LLM tool choice deferred to Phase 2+ reasoning crate. (`1E:L320-L324`)
- **P1E-E026** [E/FLOW]: Pipeline stages: (1) BM25 top_k=20 after stopword filter; (2) vector + RRF k=60 with BM25, skip if breaker; (3) PageRank importance filter; (4) CatRAG overlay re-PPR expand; (5) A-RAG snippet filter; (6–7) stubs stage "rerank" and "token_fit". (`1E:L343-L379`)
- **P1E-E027** [E/FAIL]: Stage failures wrap Result, log stage name, fall through. RRF ranks start at 1 (k=60) to avoid NaN. Optional stages degrade gracefully. (`1E:L385-L395`)
- **P1E-E028** [E/OBS]: measure_1e.rs indexes .elai_cc crates; 5 queries Q1–Q5; metrics phase_1E.toml: pipeline_p50_ms, pipeline_p99_ms, token_savings_mean_pct, catrag_overlay_size, arag_tools_per_query, stages_skipped. Targets p50<2000ms, savings>50%. (`1E:L397-L435`)
- **P1E-E029** [E/TEST]: Exit: full pipeline operational; savings>50%; latency<2s; graceful optional-layer skip; cargo test -p knowledge --lib 21+ green. (`1E:L438-L443`)
- **P1E-E030** [E/DEP]: Modules under `.elai_cc/crates/knowledge/src/{catrag,arag,pipeline}.rs`; PRE_GATE greps DkgSnapshot, Bm25Index, pagerank. (`1E:L69-L72, L74-L75, L98, L135`)
- **P1E-I001** [I/DEFER]: "Phase 2+" for ReAct A-RAG means reasoning-crate agent loop ownership; tools themselves still ship in 1E. (`1E:L320-L324`)
- **P1E-A001** [A/CFG]: SPLADE appears in knowledge.txt order but 1E pipeline stage list does not implement a dedicated splade stage (1H owns SPLADE). (`1E:L45-L49, L207-L209`)
- **P1E-A002** [A/DET]: "Heuristic scoring" formula for combining edge type + relevance + proximity into 0–10 before phi is not fully specified. (`1E:L245-L248`)
- **P1E-C001** [C/OBS]: Goal line says "sub-second" while R34 and exit criteria bind interactive budget to 2 seconds; treat 2s p50 as the hard acceptance, sub-second as aspirational. (`1E:L11, L61-L62, L411, L433, L441`)

### 2B: grounding verification

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-2B.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-003 — 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy.; PHASE-004/IX-004 — 2B plan sweep supplies the ungrounded-zero evidence gate for 3A PLANNING→TESTING_FIRST / EXECUTING path.; PHASE-004/IX-005 — 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC.; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.; PHASE-004/IX-019 — 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth).

**Behaviors**

- **P2B-E001** [E/STATUS]: Phase 2B implements verification methods that resolve claims against real sources. Prerequisite: Phase 2A (9+ GRC tests pass). (`2B:L1-L2`)
- **P2B-E002** [E/CAP]: Six methods: FILE_CHECK, SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK, API_REGISTRY_CHECK. Each returns binary pass/fail + evidence string (no confidence scores). (`2B:L6-L14, L44-L54`)
- **P2B-E003** [E/FLOW]: Plan grounding sweep before EXECUTING: classify claims GROUNDED / VERIFY_NEEDED / UNGROUNDED; verify VERIFY_NEEDED; block if any UNGROUNDED remain. Blocks PLANNING→EXECUTING. (`2B:L9-L18, L537-L543`)
- **P2B-E004** [E/RECOVERY]: Uncertain (2A) claims auto-promoted via FILE_CHECK/SOURCE_CHECK to Verified/High when resolvable; unresolved stay Uncertain and are flagged (rejection policy is 2C per-agent). (`2B:L20-L23, L463-L470`)
- **P2B-E005** [E/DEFER]: Phase 4A wraps verification behind IPC GroundingRequest; 2B is direct calls structured for later wrap without trait signature change (`&self`, stateless). (`2B:L25-L27, L141-L143`)
- **P2B-E006** [E/DEP]: Mandatory reads: reasoning.txt §§4–5,8–11; architecture-overview §4 stubs; phase-2A.1 stubs; tools.txt for file ops. (`2B:L30-L39`)
- **P2B-E007** [E/DATA]: VerificationResult {claim, passes, evidence, method_used: VerificationMethod}. GroundingResult {claim, method: VerificationMethod, level, source, verified_at:u64 TimeSource}. (`2B:L44-L45, L75-L80, L167-L175`)
- **P2B-E008** [E/DATA]: PlanGroundingResult {claims, all_grounded, ungrounded, verify_needed}. verify_needed = training: claims that could not auto-resolve; subset of ungrounded; all_grounded = ungrounded.is_empty(). (`2B:L96-L104, L176-L186`)
- **P2B-E009** [E/CAP]: VerificationMethod enum + requires_network() (Web/Api/ApiRegistry=true) + requires_tools() (File/Source/Test=true). (`2B:L67-L73, L157-L161, L339-L341`)
- **P2B-E010** [E/CAP]: Verifier trait object-safe Send+Sync: verify(method,claim,context)->Result; supported_methods(). TestRunner: run_test(name,crate)->TestRunResult{passed,output,duration_ms}. (`2B:L119-L129, L189-L202`)
- **P2B-E011** [E/CAP]: CargoTestRunner {sandbox BashSandbox, timeout_ms} production; MockTestRunner {results HashMap} for tests. LocalVerifier::new(Box<dyn TestRunner>). NetworkVerifier stubs. (`2B:L124-L128, L200-L212`)
- **P2B-E012** [E/INT]: Breaking trait changes from 2A: verify_claim gains context param; ground_plan returns PlanGroundingResult not Vec. Stubs were unimplemented!() so safe. (`2B:L96-L116, L203-L208`)
- **P2B-E013** [E/DEP]: reasoning/ gains tools/ dependency for FILE_CHECK/SOURCE_CHECK. Must not create tools→reasoning cycle; transitive knowledge via tools must not appear in cargo tree. Allowed deps: core, tools, thiserror, serde. (`2B:L82-L94, L132-L135, L209, L257-L276`)
- **P2B-E014** [E/FAIL]: GrcError::VerificationError {method, claim, reason}. Network methods return typed VerificationError not panic. (`2B:L187-L188, L423-L430, L483-L485`)
- **P2B-E015** [E/TEST]: ≥16 verification_tests: existing/missing file; source exist/missing; mock test; web/api/registry stubs; plan block/pass/empty; uncertain auto-resolve; requires_network; dyn Verifier; adversarial regex comment vs real claim; strip file: prefix. (`2B:L286-L388`)
- **P2B-E016** [E/LIMIT]: Unit tests ban network (no reqwest/hyper/TcpStream) and ban sleep/Duration/Instant timing. Network gated #[cfg(feature="network")] integration only. (`2B:L136-L138, L372-L375`)
- **P2B-E017** [E/CAP]: FILE_CHECK: tools::read_file after stripping "file:"/"grep:" prefix; evidence with path+lines or not found. SOURCE_CHECK: read_file + regex `(fn\ (`struct\`)
- **P2B-E018** [E/CAP]: TEST_CHECK: CargoTestRunner BashSandbox::execute `cargo test --exact {name} -p {crate}` timeout 30s; pass on exit 0; fail on nonzero/timeout. (`2B:L414-L421`)
- **P2B-E019** [E/FLOW]: verify_claim prefix map: file:/grep:→FileCheck; test:→TestCheck; search:→WebCheck; tool:→SourceCheck; training:→no verify passes=false Uncertain. Prefix always stripped before Verifier. (`2B:L432-L446`)
- **P2B-E020** [E/FLOW]: ground_plan: regex extract type:reference {file,grep,test,search,tool,training}; verify each; training tries FILE/SOURCE auto-resolve; empty plan trivially all_grounded=true. verified_at from TimeSource. (`2B:L448-L461, L139-L140`)
- **P2B-E021** [E/OBS]: Metrics `.elai_cc/metrics/phase_2B.toml`: file_check_latency_us <2000; source_check_latency_us <10000 (1000-line file); plan_sweep_latency_us <100000 (20 claims); methods_implemented=3/6. (`2B:L503-L534`)
- **P2B-E022** [E/LIMIT]: Zero todo!/unimplemented!/unwrap()/panic! in verification.rs; network stubs typed errors only. (`2B:L477-L500, L547`)
- **P2B-E023** [E/EVIDENCE]: Binary checks grounded MiniCheck EMNLP 2024 + Vertex AI; structural decomposition vs RARR/SAFE LLM; 6 methods reasoning.txt §4; tools dep crate map. (`2B:L50-L52, L61-L64, L551-L580`)
- **P2B-I001** [I/SEC]: Send+Sync on Verifier is inferred from 4A process isolation IPC, not explicit reasoning.txt bound. (`2B:L245-L249, L571-L574`)
- **P2B-A001** [A/DATA]: Exact regex for plan claim extraction (context vs free-text "file:") is required by T15 but pattern body not fully normative beyond type:reference set. (`2B:L349-L355, L452-L454`)
- **P2B-A002** [A/FLOW]: SOURCE_CHECK "looks like a path" vs "looks like a symbol" auto-resolution heuristic for Uncertain claims is not fully formalized. (`2B:L463-L469`)
- **P2B-C001** [C/DEFER]: Network stub message wording varies ("not implemented in phase 2B" vs summary "not impl until Phase 4A IPC"); behavioral requirement is typed error without network, not exact string. (`2B:L423-L430`)

### 2C: per-agent GRC configuration

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-2C.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-003 — 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy.; PHASE-004/IX-005 — 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC.; PHASE-004/IX-006 — 2C auditor double-verify + FINDING anti-rubber-stamp is last defense before acceptance; composes with 3A AUDITING evidence.; PHASE-004/IX-017 — Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary).; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.; PHASE-004/IX-020 — Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory.

**Behaviors**

- **P2C-E001** [E/STATUS]: Phase 2C: per-role GRC budgets, step sequences, grounding constraints, allowed methods. Prerequisite: Phase 2B. (`2C:L1-L2`)
- **P2C-E002** [E/CAP]: Engine routes AgentGrcConfig at spawn. Layered config: [grc.defaults] + [agents.{role}.grc] field-by-field override (not deep merge). Underscore role keys (context_compiler) not hyphens. (`2C:L7-L12, L54-L66, L209-L210`)
- **P2C-E003** [E/DATA]: AgentGrcConfig 8 fields: role, budget, required_steps, grounding_level, csq_required, allowed_methods, transition_overrides, require_scope_enumeration. (`2C:L111-L125, L162-L164, L468-L469`)
- **P2C-E004** [E/FLOW]: D1 transition_overrides additive HashMap only (union with allowed_next, never remove). Auditor needs Verify→[Verify,Observe] for double-verify. (`2C:L28-L40, L321-L326, L448`)
- **P2C-E005** [E/CFG]: D2 validate at load: (a) required_steps non-empty; (b) valid chain via merged transitions; (c) max_calls ≥ len(required_steps); (d) csq_required ⇒ ≥1 Verify; (e) allowed_methods ⊆ known VerificationMethod. Err ConfigValidation. (`2C:L42-L52, L312-L319, L452`)
- **P2C-E006** [E/SEC]: allowed_methods is POLICY layer (what role may request) ≤ CAPABILITY layer (IPC proxy in 4A). Context-compiler policy FILE_CHECK+SOURCE_CHECK only; executor/auditor all methods. 2C enforces in-process; 4A adds second IPC layer. (`2C:L68-L84`)
- **P2C-E007** [E/FAIL]: Anti-rubber-stamp D5: auditor Converge requires ≥1 Verify step with FINDING: prefix (case-sensitive after trim); else InsufficientFindings {role, expected_min:1}. Const FINDING_PREFIX. (`2C:L86-L97, L350-L358, L450`)
- **P2C-E008** [E/DATA]: require_scope_enumeration: Observe ThinkStep.scope must list components; orchestrator checks plan_action coverage; unaddressed → WARN. Default true executor/designer, false memory_writer. (`2C:L120-L125, L279-L282, L455`)
- **P2C-E009** [E/CAP]: GrcError::InsufficientFindings, ConfigValidation. AgentGrcConfigLoader load_config/validate_config. GrcDefaults {budget, grounding_level, csq_required, allowed_methods}. Engine with_agent_config / get_agent_config. (`2C:L126-L136`)
- **P2C-E010** [E/LIMIT]: Role stored as String not enum; validate known roles at load. No toml crate in reasoning; receives typed structs from core/orchestrator. (`2C:L152-L156, L182-L188`)
- **P2C-E011** [E/TEST]: ≥15 agent_config tests T1–T15: auditor double-verify; context_compiler observe-only; executor 8/1200; memory_writer 2/400; empty steps; budget too small; rubber-stamp; Uncertain reject; allowed_methods filter; defaults+override; additive transitions; auditor 10/1200; prompt_engineer 5/800; designer distinct; scope enum per role. (`2C:L198-L302`)
- **P2C-E012** [E/FLOW]: effective_allowed_next = (base ∪ overrides) ∩ (required_steps ∪ {Status, Converge}). Whitelist makes observe-only actually block Hypothesize. (`2C:L321-L335`)
- **P2C-E013** [E/STATE]: GroundingLevel PartialOrd must be Uncertain < High < Verified (discriminant order or explicit impl). Reject think if level < config.grounding_level. Phase 2A order {Verified,High,Uncertain} is inverted and must be fixed. (`2C:L337-L348, L405-L406, L488-L491`)
- **P2C-E014** [E/FLOW]: with_agent_config stores config and resets topic state (no budget leak across roles). Missing role table → defaults only; malformed table → error not silent fallback. (`2C:L360-L372, L399-L404`)
- **P2C-E015** [E/STATE]: Budget authority: 2C in-process GrcEngine is AUTHORITATIVE through Phases 0–3; Phase 4A demotes agent-side to advisory and orchestrator enforces (orchestrator.txt L440-444). (`2C:L373-L380, L471`)
- **P2C-E016** [E/OBS]: Metrics phase_2C.toml: config_load_us <1000; routing_overhead_pct <10; memory per config <2KB including overrides. (`2C:L414-L442`)
- **P2C-E017** [E/SURFACE]: TOML surfaces: `[grc.defaults]`, `[agents.executor.grc]`, `[agents.auditor.grc]`, `[agents.context_compiler.grc]`, `[agents.memory_writer.grc]`, `[agents.prompt_engineer.grc]`, designer role table; pattern matches existing supervision/slo_tier/sandbox/role_prompt. (`2C:L54-L66, L23, L445-L447`)
- **P2C-E018** [E/CAP]: Six roles including Designer: same model as executor possible, distinct role_prompt/required_steps/output PipelineSpec. (`2C:L270-L277, L446-L447, L470`)
- **P2C-E019** [E/LIMIT]: Zero todo!/unimplemented!/unwrap() in engine.rs and config.rs after implement. (`2C:L387-L411, L457`)
- **P2C-E020** [E/EVIDENCE]: Spec gaps flagged for reconciliation: AgentGrcConfig needs 3 extra fields in reasoning.txt L186; GrcError missing variants; GroundingLevel order; invariant for rubber-stamp; allowed_methods vs grounding_level complementary docs. (`2C:L480-L496`)
- **P2C-I001** [I/DEP]: "Status" always-allowed alongside Converge implies Status is a GRC step type reachable even when not listed in required_steps (from 2A enum). (`2C:L327-L335`)
- **P2C-A001** [A/DATA]: Exact FINDING marker structured type deferred; prefix contract is interim internal protocol. (`2C:L294-L296, L394-L398`)
- **P2C-A002** [A/CFG]: Full authoritative TOML tables for all six roles are test-described but not listed as complete sample config block in this phase file. (`2C:L198-L277`)
- **P2C-C001** [C/DATA]: reasoning.txt still documents 4-field AgentGrcConfig while 2C requires 8; phase flags update, does not edit owned spec. (`2C:L82-L84, L468-L469, L481-L484`)

### 3L: Project Intelligence (conventions + Pi tier hooks + telescope)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3L.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-007 — 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family.; PHASE-004/IX-008 — 3L CONVENTIONS injection via reasoning prompt_builder and /profile CLI; telescope priorities sit upstream of agent SYSTEM context.; PHASE-004/IX-009 — 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline.; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.

**Behaviors**

- **P3L-E001** [E/STATUS]: Phase 3L extracts project coding conventions; surfaces ConventionStats/NamingConventions; wires PiT1/PiT2/PiT3 hooks into prompt construction; integrates context telescope; latency p50≤50ms / p99≤100ms. (`3L:L1-L10`)
- **P3L-E002** [E/DEP]: Predecessor phase-3K is RAG production hardening (adaptive chunking, query expansion, HyDE, quality tiers), not a "Tier0 prompt scaffold". (`3L:L33-L36, L70-L71`)
- **P3L-E003** [E/CAP]: D1: hooks live in elai_knowledge::project_intelligence as PiT1Hook/PiT2Hook/PiT3Hook to avoid collision with elai_plugins::tiered_analysis T1/T2/T3. Reuse HookVerdict + TieredAnalysisResult (no fork). (`3L:L12-L17, L207-L208`)
- **P3L-E004** [E/CFG]: D2: regex via std::sync::LazyLock (Rust 1.80+), not once_cell::Lazy. Static-init .expect() allowed for literal patterns (CLAUDE.md S1). (`3L:L18-L20, L86-L89, L218-L219`)
- **P3L-E005** [E/DATA]: D3: conventions migration at knowledge/src/conventions_migration_v1.sql (flat sibling to metadata_migration_v2.sql, not migrations/ subdir). Separate conventions.db. (`3L:L21-L23, L94-L101`)
- **P3L-E006** [E/DATA]: D4: NamingConventions single surface from elai_core::project_profile; extract_conventions returns ConventionStats with naming field; no PiNamingConventions fork. (`3L:L24-L27, L79-L80`)
- **P3L-E007** [E/OBS]: D5: ProjectProfile snapshot at `.elai_cc/metrics/project_profile.toml` via persist_profile; reloaded at session start by `/profile` slash command (cli Profile variant). (`3L:L28-L30, L148-L151, L212`)
- **P3L-E008** [E/CAP]: D6: ProjectRuleSeeder API is seed_rules (not seed); PiT2Hook::run calls it. (`3L:L31-L32, L129-L131`)
- **P3L-E009** [E/CAP]: extract_conventions(&CorpusView)->ConventionStats; CorpusView holds Option<&[DkgNodeInfo]> + project_root + primary_language; DKG path if nodes non-empty else regex fallback. CONVENTIONS_SCHEMA_VERSION:u32=1. (`3L:L81-L91`)
- **P3L-E010** [E/DET]: extract_conventions byte-identical on repeat (sorted walker + determinism_matrix for Rust/Python/TypeScript/Svelte). Expected majority styles: snake_case, snake_case, camelCase, PascalCase. (`3L:L102-L113`)
- **P3L-E011** [E/RECOVERY]: PiT2 VERSION_GATE: if schema stale, execute_batch embedded migration (IF NOT EXISTS) and return SchemaTooOld for re-run. (`3L:L98-L101, L132-L135`)
- **P3L-E012** [E/SURFACE]: PiT1Hook::load + ProjectIntelligenceHook::render_block → CONVENTIONS block for SYSTEM via build_with_conventions. Format: per-category summaries + error_crate + top-3 avoid anti-patterns. Cap PI_T1_TOKEN_BUDGET=200 tokens (800 chars); truncate at line boundary. (`3L:L115-L126, L200-L201`)
- **P3L-E013** [E/CAP]: PiT2Hook::run seeds rules.db; evaluate Ok(None) on unrecognised language OR synthesised line count > PI_T2_CIRCUIT_BREAKER_LINES (mirrors plugins T2). (`3L:L128-L140, L221`)
- **P3L-E014** [E/STATE]: ProjectProfile aggregation after full reindex via populate_tier2_conventions + populate_tier3. PiT3Hook::evaluate ADVISORY ONLY — never HookVerdict::Block. (`3L:L142-L153, L220`)
- **P3L-E015** [E/FLOW]: assemble_telescope_with_hooks(profile, hooks)->(TelescopeContext, Vec<String>); priorities PI_T1=100 > PI_T2=80 > PI_T3=60; ties by hook id; TELESCOPE_MAX_CHARS=1200 drop whole blocks. Legacy assemble_telescope preserved. (`3L:L155-L167`)
- **P3L-E016** [E/TEST]: pi_telescope_contract: (a) top-K stable across 10 repeats; (b) T1>T2>T3; (c) per-hook budget; (d) PiT3 never Block; (e) PiT2 unrecognised language Ok(None). (`3L:L168-L174`)
- **P3L-E017** [E/OBS]: calibrate_phase_3L.py stdlib-only; 1000-file multi-lang corpus; emits phase_3L.toml; gates p50≤50ms p99≤100ms; floor 80% of gates for auditor. Fallback: reduce regex, shard by language, or in-process bench. (`3L:L176-L186`)
- **P3L-E018** [E/INT]: Reachable from prompt_builder build_with_conventions, /profile, telescope ordering; consumes rules.db seeder, conventions.db migration, HookVerdict from plugins. (`3L:L189-L214`)
- **P3L-E019** [E/LIMIT]: stub_ban: no .unwrap() outside tests or LazyLock init; regex only LazyLock; PiT3 never Block; PiT2 Ok(None) language/breaker. (`3L:L215-L221`)
- **P3L-E020** [E/TEST]: Observable: cargo test -p elai-knowledge conventions:: and elai_cc_contracts pi_telescope_contract; 4 language determinism + 5 telescope invariants; phase_3L.toml written. (`3L:L222-L227`)
- **P3L-E021** [E/DET]: DETERMINISTIC_CHECK rg list: extract_conventions, CorpusView, CONVENTIONS_SCHEMA_VERSION, PiT*Hook, ProjectIntelligenceHook, migration, persist_profile, assemble_telescope_with_hooks, pi_telescope_contract, build_with_conventions, calibration files. (`3L:L229-L251`)
- **P3L-I001** [I/SURFACE]: `/profile` is the primary operator CLI surface for reloading ProjectProfile; CONVENTIONS block is agent-visible prompt surface not a dashboard panel. (`3L:L148-L151, L119-L121`)
- **P3L-A001** [A/CFG]: Exact numeric value of PI_T2_CIRCUIT_BREAKER_LINES is not stated (only that it mirrors plugins T2). (`3L:L137-L140`)
- **P3L-A002** [A/DATA]: Full conventions.db SQL column schema beyond "migration_v1 + meta schema_version" is not expanded in this phase text. (`3L:L94-L97, L45-L46`)

### 4C: plugins hooks + tiered static analysis

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4C.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-007 — 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family.; PHASE-004/IX-010 — 4C PostToolUse ShortCoder placement resolves 4B tools.txt code_simplify + 4A WARN-3 under Tier C before auditor FS.; PHASE-004/IX-011 — 4C T1 SCHEMA_REQUIRED Escalate to ProviderTier couples rules DSL to provider constrained decoding (outside 4C body).; PHASE-004/IX-012 — 4C circular-trust normalization + AUDITOR PROPOSED freeze composes with rules.txt self-discovery and 3F-style promotion gates.; PHASE-004/IX-016 — Hash split in chunk: 4C violation_patterns blake2s vs knowledge blake2 elsewhere; 4I does not introduce new content hash for media.; PHASE-004/IX-017 — Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary).; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.; PHASE-004/IX-019 — 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth).

**Behaviors**

- **P4C-E001** [E/STATUS]: Phase 4C: invisible PreToolUse/PostToolUse/Stop/PreCompact hooks in orchestrator; tiered static analysis on SSB writes. Prerequisite: 4B tools via IPC. Agents never see hook existence. (`4C:L1-L7`)
- **P4C-E002** [E/DEFER]: Skills system and MCP bridge out of scope (skills=prompt templates no code path; MCP=1751 LOC claw-code later sub-phase). (`4C:L8-L10, L376-L380`)
- **P4C-E003** [E/FLOW]: Under Tier C: PreToolUse on ToolRequest/BashRequest before executor; PostToolUse after tool before ToolResult to agent; PostToolUse also ShortCoder code_simplify on AgentResponse before auditor view FS; Stop on session end/AgentKill; PreCompact when token_used/token_limit >0.85. (`4C:L22-L29, L258-L261`)
- **P4C-E004** [E/SEC]: Error sanitization: Block → generic IPC "Tool call rejected by policy" / "tool call rejected"; strip rule_id, pattern, severity, trading.* — agent must not see PreToolUse BLOCK from rule trading.14. Sanitize at HookVerdict enum mapping not string scrub only. (`4C:L34-L36, L66, L123-L129, L262-L263, L294-L296`)
- **P4C-E005** [E/CAP]: T1: predicate DSL from rules.db — AST_PATTERN, GREP_PATTERN, METRIC_GATE, COMPOSITE, SCHEMA_REQUIRED + phase_mask. T1 does not parse AST (T2 does). SCHEMA_REQUIRED → Escalate{target=ProviderTier} not ALLOW/BLOCK. Indexed SELECT by phase_mask & domain. Cache per phase; invalidate SHADOW→ACTIVE at next BSP superstep boundary. (`4C:L37-L41, L265-L272`)
- **P4C-E006** [E/FLOW]: T3 fires only when WARN exists AND no BLOCK from any tier. If T1=BLOCK and T2=WARN, T3 does not fire. Pattern-match only, no LLM. (`4C:L42-L43, L136-L137, L276-L278`)
- **P4C-E007** [E/CFG]: ruff PostToolUse is subprocess (~50–200ms); no public Rust API (astral-sh/ruff #12351). Circuit breaker skip ruff files >10000 lines. PostToolUse total target <500ms. (`4C:L44-L46, L251-L252, L337`)
- **P4C-E008** [E/CAP]: HookTrait methods pre_tool_use/post_tool_use/stop/pre_compact. HookVerdict Allow \ (`Block{reason} \`)
- **P4C-E009** [E/CAP]: TieredAnalysis T1/T2/T3 entry points. T2 tree-sitter syntax, import deny-list, dead/lazy code; circuit breaker >5000 lines → T1 only. T3 examples: fn>200 lines WARN:complexity; catch Exception:pass BLOCK:swallow. (`4C:L62-L69, L273-L280`)
- **P4C-E010** [E/EVENT]: BLOCK from any tier → SSB finding entry via single-writer channel. Orchestrator normalizes findings to blake2s pattern_hash (not auditor). PROPOSED rules source=AUDITOR never auto-promote; finding-source auditor cannot approve own findings. (`4C:L30-L33, L140-L148, L281-L285, L309-L310`)
- **P4C-E011** [E/TEST]: 30 contract tests plugins_hooks: generic block error; allow forward; ruff syntax block; stop IDLE gate; PreCompact checkpoint; sanitization; T1<1ms every SSB write; GREP pass-in-except BLOCK; T2 only file_write; T2<10ms <5000; T2 breaker >5000; T3 only WARN; T3<5ms; complexity/swallow patterns; SSB finding; IPC Tool/Bash PreToolUse; BLOCK prevents execution; normalization; AUDITOR no auto-promote; independent validation; cross-session lock write block/allow; SCHEMA_REQUIRED Escalate; T3 compound no-fire; ShortCoder placement + AST-equivalent. (`4C:L120-L175`)
- **P4C-E012** [E/TEST]: 10 adversarial: path traversal Unix+Windows; FSM-gated tool; independent Bash after Tool block; 1000-rule T1 still <1ms indexed; malformed predicate WARN; multi-WARN process all; T3 not on BLOCK; reason injection strip; same-auditor validate reject; AUDITOR flood stuck PROPOSED. (`4C:L197-L234`)
- **P4C-E013** [E/SEC]: PreToolUse loads get_active_rules for FSM state+domain; first BLOCK wins; cross-session file lock via SSB. Stop: non-IDLE Block, IDLE Allow. PreCompact: checkpoint before truncation. (`4C:L247-L257`)
- **P4C-E014** [E/LIMIT]: plugins depends on core/ + rules/ only; no plugins→orchestrator imports (orchestrator→plugins OK). Zero todo!/unimplemented!/pass bodies after implement. (`4C:L103-L110, L290, L304`)
- **P4C-E015** [E/OBS]: Metrics phase_4C.toml: T1/T2/T3 p50/p95/p99; hook overhead per IPC tool call; PostToolUse incl ruff; compile delta; test count; LOC; coverage. Targets T1 p99<1ms; T2 p99<10ms <5000 lines; T3 p99<5ms; hook overhead <2ms non-write; PostToolUse p99<500ms; coverage >80%; compile +<10s vs 4B. (`4C:L321-L347`)
- **P4C-E016** [E/INT]: RuntimeClient wiring: every routing call site/struct must wire ssb, fsm, clock; no None after construction. (`4C:L360`)
- **P4C-E017** [E/DEP]: 4A WARN mitigations: PostToolUse analysis is T1/T2/T3 not GRC WEB_CHECK; hooks synchronous before IPC response so GRC convergence cannot mid-cut; ShortCoder placement resolved tests 29-30. (`4C:L362-L374`)
- **P4C-E018** [E/SURFACE]: No agent-visible UI for hooks; operator-facing artifacts are SSB findings, metrics TOML, generic rejection errors on agent channel. (`4C:L4-L5, L355-L359`)
- **P4C-I001** [I/CONC]: T1 latency test note prefers Criterion benches over inline Instant in contracts; adversarial wall-clock Instant still listed as monotonic (not SystemTime). (`4C:L166-L167, L176`)
- **P4C-A001** [A/CFG]: Exact deny-list contents for T2 imports and lazy-code patterns beyond examples not fully enumerated. (`4C:L273-L275`)
- **P4C-C001** [C/OBS]: Adversarial note "ruff should use library binding" conflicts with decided subprocess + circuit breaker when binding unavailable; subprocess path is authoritative. (`4C:L44-L46, L297-L300`)

### 4I: conditional media validation + Render-Capture-Evaluate

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4I.txt`; atoms in `PHASE-004 / DEEP-PHASE-004.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-004/IX-013 — 4I lazy media path depends on 4D MediaSpec / 3G visual_check+output_type and 4E adapter structural check #8.; PHASE-004/IX-014 — 4I VLM process uses 4A Tier C isolation + same postcard IPC as agents; ToolCapabilityFlags degrade without panicking.; PHASE-004/IX-015 — 4I VisualFeedbackLoop writes SSB visual_feedback so 3B journal and 3C/4G liveness observe long RCE iterations.; PHASE-004/IX-018 — 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards.; PHASE-004/IX-020 — Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory.

**Behaviors**

- **P4I-E001** [E/STATUS]: Phase 4I conditional: implement only if pipeline tasks produce media. Prerequisites: 4A isolation. Validates images, 3D, audio, UI screenshots. Tier1 deterministic veto; Tier2 VLM advisory. (`4I:L1-L11`)
- **P4I-E002** [E/FAIL]: R15: VLM never auto-overrides Tier1; disagree → human review. R-NEW-1 tool availability degrade. R-NEW-2 GPU swap batch at phase boundary. R-NEW-3 Mode B requires SGLang. R-NEW-4 MoE VRAM stores all params. R-NEW-5 SSIM/LPIPS CPU EP + epsilon 1e-6. (`4I:L19-L39`)
- **P4I-E003** [E/DATA]: MediaValidationRequest: media_type Image\ (`Model3D\`)
- **P4I-E004** [E/CAP]: Tier1Validator trait + Image/Model3D/Audio/UiScreenshot validators. ExternalToolWrapper check_available+execute timeout; ImageMagick/FFprobe/Tesseract/Blender wrappers. ToolCapabilityFlags built at startup; missing tool → DEGRADED check not panic. (`4I:L83-L91`)
- **P4I-E005** [E/CFG]: Mode A: Qwen3.5-VL-30B-A3B ollama Q4_K_M ~17GB; needs ollama≥0.12.7; Phase0 12GB cannot run (MockVlm). Mode B: InternVL3.5-38B SGLang only RadixAttention; ~38GB 8-bit dual 3090 or ~21GB Q4. Hardware guard nvml-wrapper. (`4I:L92-L103, L48-L49`)
- **P4I-E006** [E/CFG]: CLAP audio: Xenova/clap-htsat-unfused ONNX; ~200M; FP16 ~0.4GB always-loaded when audio active; raw PCM input. (`4I:L104-L107, L271-L276`)
- **P4I-E007** [E/DATA]: VlmValidationOutput constrained: overall_quality f32[0,1], issues[], confidence, reasoning; Issue severity Critical\ (`Major\`)
- **P4I-E008** [E/TEST]: Contracts: T1 before T2; T1 fail→Fail ignore T2; T2 only if T1 pass + qualitative checks; schema conform; only media PipelineSpec; SSIM epsilon; T2 disagree→HumanReview. ≥20 tests base; ≥8 adversarial; ≥15 RCE; total ≥43. (`4I:L172-L200, L231-L240, L495-L496, L551`)
- **P4I-E009** [E/SEC]: Adversarial: EXIF/overlay prompt injection still schema-bound + Tier1-based verdict; 500MB size reject pre-VLM; malformed headers no panic; path must canonicalize inside output dir (symlink reject); VLM no /proc or parent enum; queue 64 overflow QueueFull; max 4 Tier1 / 1 Tier2 concurrent; ComfyUI timeout 600s cleanup; CLAP noise low score deterministic. (`4I:L211-L240`)
- **P4I-E010** [E/CAP]: Image: identify format/resolution/colorspace; size; SSIM image crate CPU EP; LPIPS ONNX CPU EP. 3D: Blender headless gated BLENDER flag. Audio: FFprobe + RMS silence + peak clip integer math. UI: resolution + SSIM mockup + Tesseract OCR. (`4I:L253-L263`)
- **P4I-E011** [E/FLOW]: Veto matrix: T1 fail→Fail; T1 pass no qualitative→Pass; T1+T2 pass→Pass; T1 pass T2 fail→HumanReview; T1 pass T2 disagree→HumanReview; schema-invalid after retry→Degraded; VLM unavailable→Degraded. (`4I:L277-L287`)
- **P4I-E012** [E/CAP]: Lazy load: ToolRegistry::register_lazy; factory builds flags + optional VLM on first MediaValidationRequest; non-media PipelineSpec never loads. VLM uses 15-message postcard IPC like agents; startup_timeout default 30s. (`4I:L288-L294`)
- **P4I-E013** [E/CONC]: Tier1 max 4 concurrent; Tier2 max 1; queue depth 64 → Error::QueueFull. (`4I:L295-L298, L223-L225`)
- **P4I-E014** [E/OBS]: Metrics phase_4I.toml: tier1_p95_ms (<100 image/audio/ui; 3D <500); tier2 mode A/B; gpu_swap mode A <15s / B <60s; vram; test counts; later RCE fields rce_* render/loop/convergence. (`4I:L331-L362, L578-L602`)
- **P4I-E015** [E/CAP]: MediaRenderAdapter trait: can_handle, async render→CapturedFrame[], cleanup. Types: Viewport, FrameMetadata, RenderConfig, CaptureMode Screenshot\ (`KeyFrames\`)
- **P4I-E016** [E/CAP]: Adapters: WebRenderAdapter Playwright; BlenderRenderAdapter; AnimationRenderAdapter FFmpeg; UiScreenshotAdapter pass-through; ComfyUIRenderAdapter REST. AdapterRegistry by OutputType from ToolCapabilityFlags. (`4I:L396-L406, L513-L547`)
- **P4I-E017** [E/FLOW]: VisualFeedbackLoop: render→T1→T2→issues; stop on T1 fail, max_iterations default 3, or issues unchanged (compare severity+category+location not raw string). Feedback structured VisualIssue with actionable_fix to executor prompt. (`4I:L407-L416, L451-L456, L535-L542`)
- **P4I-E018** [E/EVENT]: SSB action_type='visual_feedback' per iteration {iteration, issue_count, resolved_count, verdict} so orchestrator knows agent alive. (`4I:L492-L494, L543-L544, L566`)
- **P4I-E019** [E/SURFACE]: Functional viewports web 375/768/1440; Blender 4 angles; FFmpeg keyframes; ComfyUI mock never real in tests. visual_check=false → no render overhead. (`4I:L449-L469, L458-L459`)
- **P4I-E020** [E/SEC]: VLM process isolation: no read outside allowed_paths; no write; no network. Path traversal reject. Tier C seccomp/RLIMIT from 4A. (`4I:L185-L188, L221-L222`)
- **P4I-E021** [E/LIMIT]: architecture-overview L283 stale LLaVA models; media-validator.txt L60 "Active params ~3B" misleading for VRAM. RuntimeClient ssb/fsm/clock wiring required. (`4I:L41-L46, L613`)
- **P4I-E022** [E/STATUS]: Phase 4 overall exit: arbitrary task design pipeline INTERACTIVE; AUTONOMOUS on top (4G); mode switch preserves state; isolated agents; no enforcement visibility; isolation escape test; overhead <5%; self-mod cannot touch evaluation criteria. (`4I:L615-L623`)
- **P4I-E023** [E/DEP]: PRE_GATE tools: ImageMagick+FFmpeg required; Tesseract/Blender optional degrade; Playwright for RCE; PhaseEntry output_type/visual_check from 3G; L2 MissingAdapter/InvalidVisualCheck from 4E. (`4I:L61-L70, L376-L378`)
- **P4I-I001** [I/SURFACE]: HumanReview/Degraded verdicts are operator-facing outcomes (review queue semantics) even though no dashboard panel is specified in this phase file. (`4I:L81, L277-L287`)
- **P4I-A001** [A/CFG]: Exact QualitySpec threshold field names for SSIM/LPIPS/CLAP not restated here (owned by pipeline-factory QualitySpec). (`4I:L16, L75`)
- **P4I-A002** [A/CONC]: Whether max_iterations is globally fixed or QualitySpec-overridable beyond default 3 is only partially specified (field exists on VisualFeedbackLoop). (`4I:L408-L409, L451`)
- **P4I-C001** [C/EVIDENCE]: architecture-overview media models conflict with media-validator Mode A/B; phase binds media-validator as authority and flags overview stale. (`4I:L41-L45`)

### 1F: speculative decoding evaluation

- **Status**: superseded / DROP
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1F.txt`; atoms in `PHASE-005 / DEEP-PHASE-005.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-005/IX-001 — 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite.; PHASE-005/IX-002 — 1F requires 1C ModelPool VRAM accounting for draft overhead measurement.; PHASE-005/IX-017 — 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals.

**Behaviors**

- **P1F-E001** [E/STATUS]: Phase 1F is SPECULATIVE DECODING EVALUATION: an evaluation phase whose output is a measured ADOPT/DROP decision, not a feature ship. Status stamp 2026-04-07. (`1F:L1-L14, L24-L27`)
- **P1F-E002** [E/STATUS]: Steps 1F.1+1F.2 shipped: SpeculativeConfig, OllamaConfig nesting, ProbeResult, probe_speculative_support in ollama.rs with unit+live tests. (`1F:L3-L6`)
- **P1F-E003** [E/EVIDENCE]: Live probe on ollama v0.20.2 returned NOT_AVAILABLE: PR #8134 closed never-merged; draft_model silently ignored; accepted_tokens always null. (`1F:L6-L8, L56-L59`)
- **P1F-E004** [E/DEFER]: Steps 1F.3 / 1F.3b / 1F.4 / 1F.5 are DEFERRED-BY-DESIGN to Phase 2+ (vLLM provider); not missing work. (`1F:L8-L10, L58-L60`)
- **P1F-E005** [E/OBS]: Per-step TOMLs (correctness/throughput/vram/prompt_lookup) intentionally not produced; consolidated `.elai_cc/metrics/phase_1F.toml` records decision=UNAVAILABLE with reason. (`1F:L10-L12`)
- **P1F-E006** [E/STATUS]: EXIT CRITERIA "Decision made and documented: ADOPT or DROP" is satisfied via DROP path. (`1F:L13-L14, L309-L310`)
- **P1F-E007** [E/DEP]: Prerequisite corrected to Phase 1C complete (vector search + VRAM accounting); dependency graph L141 shows 1F branches from 1C; original "Phase 0 independent of 1A-1E" was wrong. (`1F:L16-L20, L100-L101`)
- **P1F-E008** [E/CAP]: Goal: evaluate whether speculative decoding improves throughput enough to justify VRAM cost; adopt if >1.3x throughput on target hardware, drop if less. (`1F:L24-L26, L321-L322`)
- **P1F-E009** [E/DEP]: Mandatory pre-reads: providers.txt L87-97, external-analysis EAGLE-3/TIDE, architecture-overview §7 HARDWARE TARGET L233-252. (`1F:L29-L32`)
- **P1F-E010** [E/LIMIT]: R4 VRAM: Gemma 4 E2B ~7.2GB Q4_K_M leaves ~4.8GB headroom on RTX 3060 12GB (not ~10GB as providers.txt L47 stated). Scenario A ~2GB draft → ~2.8GB free; B ~3GB delta → ~1.8GB free. (`1F:L34-L41, L76-L78`)
- **P1F-E011** [E/OBS]: Monitor VRAM via `nvidia-smi --query-gpu=memory.used` with polling during benchmark, not static snapshots. (`1F:L40-L41, L287-L289, L293-L295`)
- **P1F-E012** [E/FAIL]: R-NEW-1: Ollama speculative support unconfirmed; fallback (a) llama.cpp speculative only in Phase 1 (vLLM Phase 2+), (b) defer until ollama merges; else DROP/defer Phase 2 re-eval. (`1F:L42-L47`)
- **P1F-E013** [E/LIMIT]: R-NEW-2: EAGLE-3 published 2x-6.5x is for large (70B+) models; gains on 2.3B consumer GPU will be substantially lower; 1.3x threshold accounts for this. (`1F:L48-L50, L328-L329`)
- **P1F-E014** [E/LIMIT]: R-NEW-3: Gemma 4 EAGLE-3 draft may not exist (vLLM issue #38893); if missing, use same-family smaller draft, not EAGLE-3 head. Step 1F.2 verifies availability. (`1F:L51-L55`)
- **P1F-E015** [E/STATUS]: RESOLVED 2026-04-07: DROP speculative decoding for Phase 1; Prompt Lookup Decoding remains viable zero-VRAM alternative; revisit Phase 2+ with vLLM. (`1F:L56-L64`)
- **P1F-E016** [E/CAP]: OllamaProvider + LlmProvider trait abstraction from 1F.1 means vLLM integration is a new provider impl, not a rewrite. (`1F:L63-L64`)
- **P1F-E017** [E/EVIDENCE]: Grounding: EAGLE-3 corrected to 2x-6.5x (arxiv 2503.01840); TIDE 1.15x over static verified (arxiv 2602.05145); "Ollama supports natively" UNVERIFIED until probe; >1.3x threshold UNGROUNDED design decision. (`1F:L71-L85`)
- **P1F-E018** [E/CFG]: Draft model size guidance from llama.cpp #10466: smaller drafts often better net speedup; prefer 0.5B-1.5B over 2B-3B for Gemma 4 E2B target. (`1F:L86-L90, L145-L149`)
- **P1F-E019** [E/CFG]: num_draft_tokens default 5 is suboptimal; step 1F.4 uses parameter sweep instead of fixed k=5. (`1F:L91-L93, L114-L116, L229-L232`)
- **P1F-E020** [E/OBS]: Acceptance rate is a required metric; below 0.5 speculative decoding hurts throughput (early kill signal). (`1F:L94-L96, L234-L237, L324-L324`)
- **P1F-E021** [E/CAP]: Prompt lookup decoding added as step 1F.3b: zero-VRAM, up to 2.8x on summarization (vLLM blog); baseline before committing VRAM to draft. (`1F:L97-L99, L194-L219`)
- **P1F-E022** [E/CFG]: 1F.1 fields: speculative_decoding bool default false; draft_model Option<String>; num_draft_tokens u8 default 5; nested SpeculativeConfig or on OllamaConfig; serde round-trip. (`1F:L106-L117, L128-L130`)
- **P1F-E023** [E/TEST]: 1F.1 adversarial: serde name mismatch silent drop (round-trip test); num_draft_tokens=0 invalid (validate >=1). (`1F:L122-L126`)
- **P1F-E024** [E/TEST]: 1F.1 PRE_GATE: Phase 1C cargo test knowledge lib; providers cargo check; OllamaConfig exists via grep. (`1F:L107-L110`)
- **P1F-E025** [E/CAP]: 1F.2 probe checks ollama draft model param on /api/generate or /api/chat; outputs SUPPORTED/NOT_SUPPORTED; if neither ollama nor llama.cpp viable → DROP document. (`1F:L132-L162`)
- **P1F-E026** [E/SURFACE]: Decision artifact path: workflow/reasoning/phase-1F-decisions.md documents fallback and ADOPT/DROP with measurements. (`1F:L152-L153, L161-L161, L285-L285, L315-L315`)
- **P1F-E027** [E/TEST]: 1F.2 adversarial: silent ignore of unknown draft_model (false positive) must be disproved by timing/headers; record ollama --version. (`1F:L153-L158`)
- **P1F-E028** [E/TEST]: 1F.3 correctness: 10 prompts min; baseline vs speculative same prompt+seed; soft text match 10/10; token ID divergence WARN not fail; temp=0 + fixed seed; unique prompts/clear cache. (`1F:L164-L192`)
- **P1F-E029** [E/DATA]: Correctness metrics file `.elai_cc/metrics/phase_1F_correctness.toml`. (`1F:L179-L179, L192-L192`)
- **P1F-E030** [E/CAP]: 1F.3b prompt lookup: same 5 prompts as 1F.4, 3 runs each (15 measurements); ratio vs baseline; if ollama lacks support check llama.cpp --lookup-ngram-min; else UNAVAILABLE. (`1F:L194-L219`)
- **P1F-E031** [E/DATA]: Prompt lookup metrics: `.elai_cc/metrics/phase_1F_prompt_lookup.toml`. (`1F:L209-L209`)
- **P1F-E032** [E/FLOW]: Decision preference: if prompt_lookup >1.3x AND spec_decode also >1.3x, prefer prompt_lookup for zero VRAM; if both >1.3x and spec better, document both. (`1F:L217-L219, L277-L281`)
- **P1F-E033** [E/TEST]: 1F.4 throughput: 5 diverse prompts; k sweep {1,2,3,4,5,7}; 3 runs per prompt per k per mode; record acceptance rate; early kill if acceptance <0.5 across all k for 3+ prompts → DROP. (`1F:L221-L263`)
- **P1F-E034** [E/OBS]: Per run: prompt/output length, wall time, tokens/sec, GPU util, acceptance rate; mean+stddev per mode at best-k; ratio single number; stddev <20% of mean per mode independently. (`1F:L233-L245, L256-L261`)
- **P1F-E035** [E/DATA]: Throughput metrics: `.elai_cc/metrics/phase_1F_throughput.toml`. (`1F:L241-L241`)
- **P1F-E036** [E/TEST]: 1F.4 adversarial: GPU temp >85C discard + 60s cooldown; ≥2 prompts with >200 output tokens; warm-up exclude first run; timing proxy if acceptance not exposed. (`1F:L246-L255`)
- **P1F-E037** [E/FLOW]: 1F.5 decision gate top-to-bottom: NOT_AVAILABLE→DROP; acceptance<0.5→DROP; prompt_lookup>1.3x and spec≤lookup→DROP spec ADOPT lookup; spec>1.3x and delta_vram_mb<3000→ADOPT with best-k; else DROP; boundary ratio==1.3x or delta==3000 → DROP (conservative). (`1F:L265-L284`)
- **P1F-E038** [E/CFG]: If ADOPT: providers config default enables speculative decoding, best-k and draft model set; if DROP: default remains disabled. (`1F:L290-L291, L305-L306, L316-L318`)
- **P1F-E039** [E/DATA]: VRAM metrics: `.elai_cc/metrics/phase_1F_vram.toml` with PEAK baseline and PEAK speculative via nvidia-smi polling. (`1F:L287-L289`)
- **P1F-E040** [E/LIMIT]: ADOPT decision applies to Phase 0 hardware only; production re-evaluation needed in Phase 4 when hardware changes. (`1F:L296-L298`)
- **P1F-E041** [E/TEST]: Marginal ratio within 0.05 of 1.3x: run 5 additional prompts and recompute. (`1F:L299-L300`)
- **P1F-E042** [E/TEST]: Exit: Phase 0 + 1A-1C tests still pass (no regression); decision binary ADOPT or DROP (no maybe). (`1F:L301-L304, L319-L319`)
- **P1F-E043** [E/OBS]: Measurement targets: throughput >1.3x to adopt; acceptance >0.5; VRAM delta <3000MB; draft size 0.5B-1.5B expected optimal; TIDE only if static adopted and further opt desired. (`1F:L321-L332`)
- **P1F-E044** [E/SURFACE]: No dashboard UI in 1F; operator-visible surfaces are decision markdown, metrics TOMLs, and probe SUPPORTED/NOT_SUPPORTED console output. (`1F:L10-L14, L151-L152, L285-L289`)
- **P1F-I001** [I/DEP]: "Phase 2+" revisit wording implies any post-Phase-1 vLLM path; concrete provider step is Phase 3J, not a Phase-2 numbered phase. (`1F:L8-L10, L59-L60`)
- **P1F-A001** [A/CFG]: Exact draft model name/artifact for Gemma 4 family is unresolved until 1F.2 availability check. (`1F:L51-L55, L145-L149`)
- **P1F-A002** [A/TEST]: Soft text match algorithm (normalization, whitespace) is not fully specified beyond "identical text content". (`1F:L175-L176`)
- **P1F-C001** [C/DEP]: Original prerequisite "Phase 0 independent of 1A-1E" conflicts with dependency graph 1C branch; corrected to Phase 1C. (`1F:L16-L20, L100-L101`)
- **P1F-C002** [C/STATUS]: Opening evaluation plan describes full 1F.3-1F.5 suite, but RESOLVED block permanently skips them for Phase 1 after NOT_AVAILABLE probe. (`1F:L3-L14, L56-L60, L164-L306`)

### 2A: GRC state machine (basic)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-2A.txt`; atoms in `PHASE-005 / DEEP-PHASE-005.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-005/IX-003 — 2A freezes GrcEngine six-method trait so 2B can implement verify_claim/ground_plan without breaking BasicGrcEngine.; PHASE-005/IX-004 — 2A scope enumeration WARNs feed agent context; orchestrator (outside 2A) enforces plan_action coverage.; PHASE-005/IX-016 — 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers.

**Behaviors**

- **P2A-E001** [E/STATUS]: Phase 2A: GRC STATE MACHINE (basic). Prerequisites: Phase 0 complete (contract tests pass, cargo check clean). (`2A:L1-L2`)
- **P2A-E002** [E/CAP]: Goal: GRC engine core — all reasoning cites grounding; budget enforces per-topic call/char limits; state machine validates transition order; rejected steps produce typed errors. (`2A:L5-L9`)
- **P2A-E003** [E/LIMIT]: Scope excludes grounding verification methods (2B), per-agent config (2C), CSQ (2D). (`2A:L11-L13`)
- **P2A-E004** [E/DEP]: Pre-reads: reasoning.txt §§1-3,10,11 items 1-3; architecture-overview §4 GrcEngine; prompt-architecture §1. (`2A:L16-L21`)
- **P2A-E005** [E/STATE]: D1: GrcState with allowed_next(); observe→hypothesize→plan_action→act→verify; reject invalid (e.g. hypothesize→verify). (`2A:L26-L32, L72-L76`)
- **P2A-E006** [E/FAIL]: D2: GrcError { UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding } with context; thiserror. (`2A:L34-L37, L96-L100`)
- **P2A-E007** [E/CFG]: D3: GrcBudget tracks remaining_calls/chars; when remaining_calls<=1 engine forces convergence (must produce conclusion). (`2A:L39-L44, L89-L93`)
- **P2A-E008** [E/DATA]: D4: grounding string parsed into source citations format "source_type:reference" (e.g. file:src/foo.rs:L42); 2A format-only, 2B verifies sources. (`2A:L46-L52, L273-L285`)
- **P2A-E009** [E/STATE]: D5: GroundingLevel VERIFIED > HIGH > UNCERTAIN (PartialOrd); UNCERTAIN flagged warn log, not rejected in 2A. (`2A:L54-L57, L94-L95, L279-L283`)
- **P2A-E010** [E/CAP]: 2A.1 greenfield: create crates/reasoning with Cargo.toml dep core; add to workspace. (`2A:L62-L70`)
- **P2A-E011** [E/STATE]: GrcState enum: Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status; Converge and Status allowed from any state; first non-Status/non-Converge step must be Observe. (`2A:L72-L76, L217-L220, L269-L271`)
- **P2A-E012** [E/DATA]: ThinkStep { step_type, content, topic, grounding: Option<String>, conclusion: Option<String>, scope: Option<Vec<String>> }. (`2A:L77-L85`)
- **P2A-E013** [E/CFG]: scope: when AgentGrcConfig.require_scope_enumeration=true, Observe must populate scope; orchestrator validates every scope item appears in subsequent plan_action; unaddressed → WARN into agent context. (`2A:L81-L85, L222-L237, L364-L364`)
- **P2A-E014** [E/DATA]: ThinkResult { step, budget_remaining: GrcBudget, grounding_level: Option<GroundingLevel>, warnings: Vec<String> }. (`2A:L87-L88, L305-L307`)
- **P2A-E015** [E/CFG]: GrcBudget default max_calls=5, max_chars=800 ("moderate"); methods remaining_calls, remaining_chars, is_exhausted, must_converge (remaining_calls<=1). (`2A:L89-L93, L211-L215`)
- **P2A-E016** [E/CAP]: GrcEngine trait 2A methods: think, set_budget, get_budget, reset_topic; 2B stubs unimplemented: verify_claim, ground_plan; full 6-method trait for stability. (`2A:L101-L112, L142-L144`)
- **P2A-E017** [E/DATA]: 2B forward stubs: VerificationResult, GroundingResult with #[allow(dead_code)]. (`2A:L113-L116`)
- **P2A-E018** [E/STATE]: BasicGrcEngine: HashMap<String, TopicState> with budget, last_state, steps, grounding_log; default budget applied on first think if no set_budget. (`2A:L117-L120, L300-L303`)
- **P2A-E019** [E/DEP]: 2A.2: reasoning depends only on core/, thiserror, serde — no providers/, tools/, knowledge/; pure logic, no LLM calls. (`2A:L146-L164`)
- **P2A-E020** [E/TEST]: 2A.3 TDD: 12 failing tests T1-T12 covering ungrounded observe, budget overflow/forced converge (max_calls=3), per-topic isolation, invalid transition, must_converge, malformed grounding, free Status, default budget, first-step Observe, scope present/missing/optional. (`2A:L166-L255`)
- **P2A-E021** [E/TEST]: T2 uses max_calls=3 not 2 so must_converge at remaining<=1 does not mask overflow test. (`2A:L179-L189`)
- **P2A-E022** [E/TEST]: T7 Status is free: does not consume budget (calls_used stays 0). (`2A:L207-L209, L362-L362`)
- **P2A-E023** [E/TEST]: Stubs must use todo!/unimplemented!, not Ok placeholders; zero timing-dependent tests. (`2A:L247-L254`)
- **P2A-E024** [E/FLOW]: I1 transitions: last_state.allowed_next(); first step Observe; Status does not update last_state; Converge allowed any state. (`2A:L265-L271`)
- **P2A-E025** [E/DATA]: I2 grounding required on Observe, Hypothesize, Verify; optional on PlanAction/Act; none on Status. Parse comma-separated type:reference. Valid types: file, grep, test, search, tool, training. (`2A:L273-L285`)
- **P2A-E026** [E/DATA]: GroundingLevel map: test/search→Verified; file/grep/tool→High; training→Uncertain; warn on Uncertain, do not reject. (`2A:L279-L283`)
- **P2A-E027** [E/FLOW]: I3 budget check-then-increment: exhaust calls, chars overflow, or must_converge without Converge → BudgetExhausted before increment; then saturating_add. (`2A:L287-L298`)
- **P2A-E028** [E/STATE]: I4 reset_topic clears all state for topic; entry fully removed from map (not zeroed). (`2A:L300-L303, L322-L323`)
- **P2A-E029** [E/TEST]: 2A.4 acceptance: cargo test -p reasoning 0 failures ≥12; no todo!/unimplemented!/unwrap() in engine.rs. (`2A:L309-L327`)
- **P2A-E030** [E/OBS]: 2A.5 metrics: validation_overhead_us <5000; memory per topic <1KB base + history; write `.elai_cc/metrics/phase_2A.toml`; median of 1000 iterations; no criterion without approval. (`2A:L329-L354`)
- **P2A-E031** [E/TEST]: Exit criteria: grounding enforced; budget overflow → converge or BudgetExhausted; InvalidTransition; MalformedGrounding; free Status; independent topics; scope WARN when required; 12+ tests. (`2A:L357-L365`)
- **P2A-E032** [E/EVIDENCE]: Budget range grounding: max_calls 1-12, max_chars 200-1500 from reasoning.txt §3; defaults are moderate 5/800. (`2A:L369-L370, L93-L93`)
- **P2A-E033** [E/SURFACE]: No dashboard panel in 2A; operator/engine surface is GrcEngine API and typed errors; Status free-echo aligns with mcp_think(step_type="status"). (`2A:L207-L209, L373-L373`)
- **P2A-I001** [I/INT]: AgentGrcConfig and orchestrator scope validation are named but config crate/module location is outside 2A skeleton (types only reference require_scope_enumeration). (`2A:L81-L85, L222-L237`)
- **P2A-A001** [A/STATE]: Exact allowed_next() table for every state (beyond Observe→not Verify and Verify terminal note) is not fully enumerated in the phase file. (`2A:L72-L76, L136-L138, L317-L319`)
- **P2A-A002** [A/TEST]: T11 says "Err or ThinkResult with warning" for missing scope — hard fail vs soft warn unresolved. (`2A:L229-L232`)
- **P2A-A003** [A/FAIL]: InvalidTransition "from=None indication" for first non-Observe step lacks a dedicated error variant shape. (`2A:L217-L220`)
- **P2A-C001** [C/DATA]: Header says "Grounding required on every step" in grounding summary item while body requires grounding only on Observe/Hypothesize/Verify (PlanAction/Act optional, Status free). (`2A:L273-L285, L358-L362, L371-L371`)

### 3F: rules engine (self-improving, three-tier enforcement)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3F.txt`; atoms in `PHASE-005 / DEEP-PHASE-005.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-005/IX-005 — 3F depends on 3D complete and 1A tree-sitter via AstEvaluator; self-discovery needs 3C agent/auditor IPC.; PHASE-005/IX-006 — 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM).; PHASE-005/IX-007 — 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7).; PHASE-005/IX-008 — 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters.; PHASE-005/IX-015 — Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception.

**Behaviors**

- **P3F-E001** [E/STATUS]: Phase 3F: RULES ENGINE. Prerequisites: Phase 3D complete (parallel with 3E); implicit Phase 1A tree-sitter for AST_PATTERN. (`3F:L1-L3`)
- **P3F-E002** [E/CAP]: Goal: deterministic rule enforcement replacing probabilistic prompts; Predicate DSL with 5 types; self-discovery from auditor violations; rules DATA in SQLite evaluated by Rust; three-tier enforcement (Tier1 code, Tier2 constrained decoding, Tier3 prompt+auditor). (`3F:L7-L12`)
- **P3F-E003** [E/DEP]: Pre-reads: rules.txt full; prompt-architecture §§1-2,4.3,5,6; risk-register R25. (`3F:L14-L22`)
- **P3F-E004** [E/SEC]: R25 LLM ignores domain rules — this phase is the mitigation. (`3F:L24-L25`)
- **P3F-E005** [E/FAIL]: Malformed self-discovered predicates mitigated by compile-time validation before DB insert (3F.7). (`3F:L26-L27, L271-L272`)
- **P3F-E006** [E/INT]: tree-sitter: rules/ must NOT add separate dep; knowledge exposes AstEvaluator facade; recommended trait Box<dyn AstEvaluator> injected at construction (option b preserves dep graph). (`3F:L28-L42`)
- **P3F-E007** [E/FLOW]: Evaluation order cheapest-first short-circuit on BLOCK: METRIC_GATE → GREP_PATTERN → AST_PATTERN → COMPOSITE; within type severity DESC (BLOCK before WARN). (`3F:L44-L55, L133-L134`)
- **P3F-E008** [E/FLOW]: WARN collection: WARNs before a BLOCK preserved with BLOCK as context. (`3F:L52-L53`)
- **P3F-E009** [E/DEP]: Dev deps: proptest, criterion; runtime jsonschema; tree-sitter via knowledge AstEvaluator. (`3F:L57-L61`)
- **P3F-E010** [E/DATA]: 3F.1 tables: rules, rule_metrics, contract_tests, violation_patterns; schema_version migration; columns per rules.txt §2 including phase_mask, severity, mutable, source; scoring score_clarity/frequency/precision; counterfactual causal_catches/spurious_catches. (`3F:L66-L93`)
- **P3F-E011** [E/TEST]: CHECK constraints reject invalid tier/status/source; NULL predicate INSERT fails; migration idempotent preserves v1 data. (`3F:L81-L93`)
- **P3F-E012** [E/CAP]: 3F.2 five predicate types: AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE. (`3F:L95-L100, L119-L119`)
- **P3F-E013** [E/STATE]: Verdict: ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget); EscalationTarget: Auditor, Human, NextTierRule(RuleId). (`3F:L101-L102, L120-L121`)
- **P3F-E014** [E/DATA]: Predicate serde to/from TEXT (JSON or postcard); round-trip for all types. (`3F:L103-L103, L109-L109`)
- **P3F-E015** [E/CAP]: AstEvaluator: evaluate_query(language, source, query)→Vec<AstMatch>; validate_query→Result. (`3F:L104-L106`)
- **P3F-E016** [E/FAIL]: COMPOSITE circular graph returns error within 1ms; serde version drift explicit error or defaults. (`3F:L112-L117`)
- **P3F-E017** [E/CAP]: 3F.3 RuleEngine trait: evaluate(action, context)→ALLOW\ (`BLOCK\`)
- **P3F-E018** [E/STATE]: 3F.4 lifecycle: PROPOSED → CANDIDATE → SHADOW → ACTIVE → RETIRED. (`3F:L150-L155, L189-L189`)
- **P3F-E019** [E/FLOW]: Guards: PROPOSED→CANDIDATE ≥3 contract tests all pass; CANDIDATE→SHADOW human OR independent second-auditor (finding-source auditor cannot approve); SHADOW→ACTIVE FPR<5% AND TPR>80% AND counterfactual_causal_rate>50% AND no safety regressions; ACTIVE→RETIRED FPR>30% over 10 pipelines OR superseded OR human. (`3F:L155-L162`)
- **P3F-E020** [E/SEC]: mutable=0 rejects auto-retirement; mutable=0 exempt from causal rate gate (human-promotable only). (`3F:L163-L164, L177-L177, L191-L191`)
- **P3F-E021** [E/OBS]: Every transition logged with timestamp, metrics snapshot, reason. (`3F:L165-L165, L178-L178`)
- **P3F-E022** [E/FAIL]: Division-by-zero safe rates: 0 shadow data → "insufficient shadow data" error not panic; RETIRED kept in DB not deleted. (`3F:L179-L184, L192-L192`)
- **P3F-E023** [E/SEC]: PROPOSED source=AUDITOR cannot auto-promote without human/second-auditor. (`3F:L175-L187, L276-L277`)
- **P3F-E024** [E/CAP]: 3F.5 interpreter: AST via AstEvaluator; GREP compile-once regex; SCHEMA_REQUIRED jsonschema; METRIC_GATE lt/gt/eq/between; COMPOSITE AND/OR/NOT max depth 8. (`3F:L194-L229`)
- **P3F-E025** [E/DATA]: Evaluation returns Verdict { rule_id, result, evidence: String }. (`3F:L205-L205`)
- **P3F-E026** [E/FLOW]: ESCALATE routing: Auditor for SHADOW rules; Human for mutable=0 promotion; NextTierRule for tier chain. (`3F:L206-L207, L215-L215`)
- **P3F-E027** [E/DET]: Same code + same predicate = same verdict over 100 repetitions. (`3F:L216-L216, L229-L229, L408-L408`)
- **P3F-E028** [E/FAIL]: COMPOSITE depth 100 → Err(DepthExceeded); ReDoS patterns complete within timeout (regex linear-time). (`3F:L217-L222`)
- **P3F-E029** [E/CFG]: 3F.6 phase_mask bitmask + domain detection; get_active_rules status=ACTIVE AND phase_mask AND domain; SQL includes OR domain IN ('engineering','security'); elai_cc.toml [rules.domain_mapping]. (`3F:L231-L242`)
- **P3F-E030** [E/CFG]: Engineering and security domains always active (cross-cutting, security mutable=0). (`3F:L239-L242, L247-L248, L257-L258`)
- **P3F-E031** [E/TEST]: Domain detection covers 5 domains + engineering + security; case-insensitive path matching; phase_mask=0 never active. (`3F:L243-L258`)
- **P3F-E032** [E/CAP]: 3F.7 self-discovery on orchestrator side: normalize, blake2s hash, count, auto-propose at occurrences≥3; generate AST then regex then Tier3-only; compile_predicate before insert; invalid logged discarded. (`3F:L260-L298`)
- **P3F-E033** [E/TEST]: Auto-contract tests: each example_code → BLOCK test; orchestrator generates ALLOW counterexamples (not auditor). (`3F:L273-L275, L285-L285`)
- **P3F-E034** [E/OBS]: Violation scoring 3-axis composite clarity/frequency/precision in [0,15]. (`3F:L278-L278, L298-L298`)
- **P3F-E035** [E/SEC]: Normalization runs in orchestrator process, not auditor (circular trust). (`3F:L265-L266, L292-L293`)
- **P3F-E036** [E/DATA]: 3F.8 seed: rules_seed.db ~33 highest-value rules; status=ACTIVE source=SEED skip lifecycle; merge global+project rule_id override; mutable=0 seeds cannot be overridden by mutable=1 project rules. (`3F:L300-L337`)
- **P3F-E037** [E/SURFACE]: Post-tool completeness GREP rules (Tier1, engineering, PostToolUse): web_fetch <img> → WARN "N images not extracted"; <table> → WARN "N tables not extracted"; file_read TODO/FIXME → INFO "N TODO/FIXME items"; <1ms, no pre-exec I/O. (`3F:L309-L314`)
- **P3F-E038** [E/TEST]: Every seed Tier1 rule ≥3 contract tests; Tier2 ≥2; absent seed DB explicit error with path. (`3F:L322-L328`)
- **P3F-E039** [E/TEST]: 3F.9 ≥254 static contract tests; proptest 256 cases/rule; generators violation/valid/mutation_invariant/fix_mutation; COMPOSITE prop_recursive depth 8 size 64; De Morgan + associativity; shrunk counterexamples → contract_tests source=GENERATED; ~1400 proptest lines / 7 domains. (`3F:L339-L371`)
- **P3F-E040** [E/SEC]: contract_tests READ-LOCK during self-modification (INSERT rejected when flag true). (`3F:L364-L371`)
- **P3F-E041** [E/OBS]: 3F.10 criterion benchmarks: GREP median <1ms; METRIC <0.1ms; AST <5ms target; 10 mixed <10ms; store `.elai_cc/metrics/phase_3F.toml`. (`3F:L373-L397`)
- **P3F-E042** [E/DEFER]: Counterfactual shadow RUNTIME integration deferred to orchestrator+auditor (3C+3A) and 3I DST model; 3F only models rates/guards. (`3F:L399-L405`)
- **P3F-E043** [E/TEST]: Exit: determinism 100x; lifecycle all guards; self-discovery ≥3; ≥254+proptest; GREP/METRIC latency; auditor trust boundary. (`3F:L407-L414`)
- **P3F-E044** [E/EVIDENCE]: 254 arithmetic verified from rules.txt §9A; COMPOSITE depth=8 UNGROUNDED defensive (not in rules.txt); policy_engine.rs/permission_enforcer.rs LOC verified present. (`3F:L422-L448`)
- **P3F-E045** [E/SURFACE]: No dedicated rules dashboard panel in 3F phase file; enforcement surfaces are evaluate verdicts, seed DB, PostToolUse WARN/INFO strings, and transition logs. (`3F:L309-L314, L165-L165`)
- **P3F-I001** [I/INT]: SCHEMA_REQUIRED may interact with provider-tier escalation only if rules.txt says so; phase-3F body defines schema validation, not provider-tier escalation. (`3F:L99-L100, L202-L202`)
- **P3F-A001** [A/DATA]: Predicate TEXT encoding choice JSON vs postcard left open. (`3F:L103-L103`)
- **P3F-A002** [A/CFG]: Exact five business domains for detection listed only as "all 5 domains + engineering + security" without full enum in this file. (`3F:L256-L258`)
- **P3F-A003** [A/FLOW]: Verdict trait evaluate signature shows ALLOW\ (`BLOCK\`)
- **P3F-C001** [C/DEP]: Crate map says rules depends on core only, while architecture decision requires knowledge AstEvaluator or knowledge dep — resolved by recommended trait injection but dual wording remains. (`3F:L31-L41`)

### 3J: vLLM + advanced providers + plugins + orchestrator hardening

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3J.txt`; atoms in `PHASE-005 / DEEP-PHASE-005.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-005/IX-001 — 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite.; PHASE-005/IX-007 — 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7).; PHASE-005/IX-008 — 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters.; PHASE-005/IX-009 — 3J workload_profile + advanced SSB types extend 3B core actions to ≥17 including phase_complete atomic boundary used later by 4A.; PHASE-005/IX-010 — 3J.WIRE.SESSION composes 3H SessionDag trim 85% + stub recovery from SSB.; PHASE-005/IX-011 — 3J.WIRE.PLAN composes 3G PlanSkeleton into CLI Slot 2 and /plan surfaces.; PHASE-005/IX-012 — 3J.WIRE.FOUNDATION InteractiveGate is the 3A/4G-shared human gate surface (Approve/Reject/Modify/Redirect).; PHASE-005/IX-013 — 3J.14 retrieval injects 1B/1E/1G pipeline pieces (BM25, CatRAG, ARAG, optional cross-encoder) into prompt Slot 4.; PHASE-005/IX-014 — 3J.15 SUBSTITUTE_MODEL 100B path anticipates 4G escalation; anti-spiral 3× same error swap.; PHASE-005/IX-015 — Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception.; PHASE-005/IX-016 — 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers.; PHASE-005/IX-017 — 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals. …

**Behaviors**

- **P3J-E001** [E/STATUS]: Phase 3J: vLLM integration + advanced providers + plugins + orchestrator hardening. Prerequisites: Phase 3I complete; Phase 1F speculative eval. (`3J:L1-L2`)
- **P3J-E002** [E/CAP]: Goal: vLLM production backend; layer opts DoLa/TurboQuant/steering; ModelPool GPU multiplex/preload/profiles; MCP bridge; tiered static hooks; AppContainer; advanced SSB types; MILP eval. (`3J:L6-L11`)
- **P3J-E003** [E/CFG]: Primary test model Gemma4 E2B on RTX 3060 12GB; final 3J.14/15 apply to Gemma4 31B via CPU offload on 64GB RAM (~0.5-2 tok/s); EAGLE-3 draft training not feasible on CPU. (`3J:L13-L30, L99-L101`)
- **P3J-E004** [E/EVIDENCE]: Installed 2026-04-08: gemma4:31b 19GB Q4 262144 ctx vision+tools+thinking; gemma4:e2b 7.2GB; gemma3:4b fallback; no separate VLM needed. (`3J:L32-L41`)
- **P3J-E005** [E/CFG]: vLLM 0.19.0 in WSL2 ~/vllm-env/ PyTorch 2.10+cu128; serve example port 8000; Rust on Windows connects via HTTP localhost auto-forward. (`3J:L42-L51`)
- **P3J-E006** [E/DEP]: Pre-reads: providers.txt vLLM/LayerConfig/ModelPool; orchestrator SSB/AppContainer/self-mod; plugins MCP+hooks; paper summaries DoLa/TurboQuant/EAGLE-3/TIDE/ActAdd/Murakkab. (`3J:L54-L64`)
- **P3J-E007** [E/DEP]: Phase is prerequisite for 4A isolation and 4G autonomous mode (placement/optimization must stabilize first). (`3J:L67-L69`)
- **P3J-E008** [E/FAIL]: Risks R1-R11: HTTP latency, DoLa layer access, TurboQuant deep integration, ActAdd coupled, MILP Windows build, MCP 1751 LOC, AppContainer weaker-than-Linux, VRAM overcommit, small-model transfer, 31B CPU speed, CPU hook availability. (`3J:L72-L105`)
- **P3J-E009** [E/CAP]: 3J.1 VllmProvider: /v1/completions, /v1/models, SSE (reuse sse.rs), health GET /health, LlmProvider complete/embed/token_count; cfg feature "vllm"; latency 100 req p50/p95/p99 in vllm_spike.toml; timeout 5s on dead server. (`3J:L110-L141`)
- **P3J-E010** [E/CAP]: 3J.2 InferenceBackend::{Ollama(OllamaConfig), Vllm(VllmConfig)}; VllmConfig server_url, tensor_parallelism, served_model_names; ModelPool does not load/unload under Vllm (vLLM scheduler); POST/DELETE load/unload model APIs. (`3J:L143-L174`)
- **P3J-E011** [E/CFG]: elai_cc.toml [providers.backend] = "ollama"\ (`"vllm"; fallback to Ollama emits WARNING including "layer-level optimizations unavailable".`)
- **P3J-E012** [E/CAP]: 3J.3 DolaDecoder: JSD premature layer, alpha=0.1, theta=1.2 repetition; model-specific candidate buckets; DolaConfig enabled/alpha/buckets/repetition_penalty. (`3J:L176-L199`)
- **P3J-E013** [E/CFG]: Per-role DoLa: Executor on, Auditor amplified, Prompt-engineer off; elai_cc.toml [agents.{role}.layer_config.dola]. (`3J:L200-L204, L234-L234`)
- **P3J-E014** [E/LIMIT]: Guard: model_params <1B auto-disable DoLa with WARNING; 1-7B EVALUATE empirically; degraded top-N logprobs path if full logits unavailable. (`3J:L211-L219, L226-L227`)
- **P3J-E015** [E/TEST]: E2B: 20 factuality prompts baseline vs DoLa; latency budget ~5-8% (accept within 10%). (`3J:L204-L209, L236-L237`)
- **P3J-E016** [E/DEFER]: 3J.4 TurboQuant FEASIBLE/DEFER decision; QR rotation + Lloyd-Max; b-bit codebooks; d>=64; unit-norm + orthogonal Pi invariants; decision file phase-3J-turboquant.md. (`3J:L239-L291`)
- **P3J-E017** [E/CAP]: 3J.5 SteeringVectorEngine ActAdd: h_A^l = h+^l − h-^l; SteeringVector {layer, vector, coefficient}; config vectors/enabled; elai_cc.toml [agents.{role}.layer_config.steering]; mid-layer default model_layers/3; c in [3,20] start 5. (`3J:L293-L350`)
- **P3J-E018** [E/DEP]: Steering coupled to DoLa layer hook: if DoLa deferred, steering deferred. Local models only (no API models). Composable multi-vector. (`3J:L323-L329, L350-L350`)
- **P3J-E019** [E/DATA]: Store vectors .safetensors or .npz; E2B ≥2 contrast pairs skepticism+completeness; 5 prompts/vector eval. (`3J:L330-L334`)
- **P3J-E020** [E/CAP]: 3J.6 MultiplexingPolicy: safety_margin_bytes default 1GB; max_colocated 3; min_model_size_for_dedicated_gpu 4GB; ModelPool.can_colocate; Phase 0-1 single GPU no multiplex. (`3J:L352-L369, L401-L404`)
- **P3J-E021** [E/FLOW]: Predictive preload on FSM entry: EXECUTING→auditor; AUDITING→executor; PLANNING→prompt-engineer; TESTING_FIRST→executor; tokio::spawn background; blocked if VRAM insufficient. (`3J:L370-L378`)
- **P3J-E022** [E/CFG]: PreloadPolicy map FsmState→Vec<ModelId> in [providers.preload]; SloTier BEST/GOOD/FAIR/BASIC per [agents.{role}.slo_tier]; tier mismatch WARNING. (`3J:L377-L386`)
- **P3J-E023** [E/EVENT]: 3J.7 WorkloadProfile SSB action_type='workload_profile' at phase_complete: role, phase_id, tokens_in/out, latency_ms, vram_peak_mb, model_id, gpu_id, timestamp ClockProvider; budget <1ms. (`3J:L406-L442`)
- **P3J-E024** [E/CAP]: 3J.8 MILP eval vs greedy: good_lp+HiGHS preferred; ADOPT if >10% swap latency improvement; DROP if heuristic <5% worse; EVALUATE if 5-10%; always implement GreedyModelPlacer; decision phase-3J-milp.md; solve budget <50ms; infeasible→greedy WARNING. (`3J:L444-L487`)
- **P3J-E025** [E/CAP]: 3J.9 McpBridge: connect/list_tools/call_tool/disconnect; transport stdio/SSE/streamable HTTP; feature mcp; port claw-code 912+839 LOC. (`3J:L489-L533`)
- **P3J-E026** [E/RECOVERY]: MCP retry 3 attempts 1s/2s/4s; heartbeat timeout 30s; tool call timeout 30s configurable; graceful degradation explicit errors to agent. (`3J:L504-L509, L525-L526`)
- **P3J-E027** [E/SEC]: MCP untrusted: sanitize results; treat as data not instructions; version handshake reject incompatible; agent never connects to MCP directly (orchestrator proxy). (`3J:L509-L528`)
- **P3J-E028** [E/CAP]: 3J.10 T1 RULES <1ms every SSB write; T2 AST <10ms on file_write only, circuit break files>5000 lines; T3 PATTERN <5ms only when WARN and no BLOCK; no LLM hot path. (`3J:L535-L567`)
- **P3J-E029** [E/SEC]: BLOCK/WARN → SSB finding; orchestrator blake2s normalizes for violation_patterns; agent sees sanitized "Tool call rejected by policy" only (no rule ID/tier/pattern). (`3J:L556-L563, L582-L584`)
- **P3J-E030** [E/SEC]: 3J.11 AppContainerConfig profile elai_cc_agent_{id}; granted_paths; network_allowed false except researcher; Job limits ACTIVE_PROCESS=1, PROCESS_MEMORY=512MB, JOB_TIME; env only LANG/TERM/PATH/TMPDIR. (`3J:L586-L614`)
- **P3J-E031** [E/SEC]: Self-mod locks: IPC realpath Write/Edit reject; FS not granted; GitCommit filter on locked paths; fallback restricted token documented weaker (junctions). (`3J:L605-L611, L624-L625`)
- **P3J-E032** [E/CAP]: AgentSandbox trait create/launch_agent/cleanup; Windows AppContainer cfg; Linux clone+seccomp Phase 4A. (`3J:L615-L617`)
- **P3J-E033** [E/EVENT]: 3J.12 adds typed SSB actions: model_swap, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, memory_monitoring, phase_complete, workload_profile; total variants ≥17; write-time type validation. (`3J:L637-L681`)
- **P3J-E034** [E/OBS]: isolation_metric: context_savings_from_isolation = sum(agent_grc_tokens)/model.max_context at phase transition. (`3J:L663-L665`)
- **P3J-E035** [E/CAP]: 3J.PIPELINE InferenceDispatcher trait; ApiDispatcher + LocalDispatcher; [backend] mode local\ (`api\`)
- **P3J-E036** [E/FLOW]: FSM tool filtering: PLANNING read-only; EXECUTING full; AUDITING read-only+test; local max 15 tools truncate descriptions; API all tools. (`3J:L714-L721`)
- **P3J-E037** [E/CFG]: ContextBudget: local 80% of 131K; API 200K; slots system/tools/messages. (`3J:L723-L730`)
- **P3J-E038** [E/SURFACE]: Slash /model and /mode switch backend at runtime; elai --local uses orchestrator pipeline; default elai uses API same pipeline. (`3J:L737-L737, L765-L767`)
- **P3J-E039** [E/FLOW]: Auto-start sidecar when mode local/auto; health wait max 120s; kill sidecar on CLI exit; auto→API if start fails. (`3J:L739-L744`)
- **P3J-E040** [E/CAP]: 3J.UNIFIED single RuntimeClient: auth/backend/tool_policy/budget_policy; one stream(); max 3 backend branch points; BackendConfig max 10 fields. (`3J:L769-L848`)
- **P3J-E041** [E/CFG]: ToolPolicy Full \ (`FsmFiltered {max_tools, fsm_state_path} \`)
- **P3J-E042** [E/SURFACE]: --local flag sets BackendConfig mode Local; no flag → Api from env auth; elai.bat simplified to --local + HOME fallback. (`3J:L814-L815, L862-L862`)
- **P3J-E043** [E/STATUS]: 3J.PIPELINE session summary 2026-04-09: dispatcher.rs, Fsm::read_persisted_state, LocalRuntimeClient, start_sidecar windows_to_wsl_path, /shutdown, real tokenizer counting, 7/7 tests, patches 0.3/0.4. (`3J:L850-L875`)
- **P3J-E044** [E/CAP]: 3J.WIRE.FOUNDATION: LiveCli owns Ssb, Arc<Fsm>, ClockProvider; paths sessions/{sid}/ssb.db and .fsm-state.msgpack. (`3J:L877-L907`)
- **P3J-E045** [E/SURFACE]: CliInteractiveGate: present_gate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances. (`3J:L895-L898`)
- **P3J-E046** [E/EVENT]: SessionTracer backed by Ssb: turn_started, tool_execution, turn_completed → SsbPayload; Ssb::shutdown on exit. (`3J:L902-L907`)
- **P3J-E047** [E/CFG]: 3J.WIRE.BUDGET: real fixed_slot_sizes for slots 0/1/2; max_tokens = min(model max, remaining); ApiDefault skips ContextBudget; [pipeline] telemetry logs slots. (`3J:L925-L963`)
- **P3J-E048** [E/CAP]: 3J.WIRE.RULES: load seed rules (~30-40 eng+security); DefaultRuleEngine on PreToolUse; BLOCK reject; WARN→Ssb; ESCALATE→InteractiveGate; AstEvaluator/DkgQuerier stubs initially; strip CLAUDE.md rules from local system prompt only. (`3J:L966-L1015`)
- **P3J-E049** [E/OBS]: Rule eval <5ms per tool call Tier1 target; seed rules versioned under .elai_cc/data/rules/ TOML. (`3J:L1004-L1007`)
- **P3J-E050** [E/CAP]: 3J.WIRE.SESSION: SessionDag sessions/{sid}/dag.db; auto-snapshot each turn; trim at 85% budget 3-pass; /resume via load_snapshot+parent_chain; /session fork create_branch max 3; recover_stubs from Ssb. (`3J:L1017-L1060`)
- **P3J-E051** [E/LIMIT]: SessionDag sync SQLite not WAL; separate from Ssb WAL connection; keep /compact as manual override. (`3J:L1045-L1052`)
- **P3J-E052** [E/SURFACE]: 3J.WIRE.PLAN: optional PlanSkeleton from .elai_cc/plans/{pipeline_id}.toml; /plan displays skeleton; /plan phase <id> shows PhaseDetail; Slot 2 phase context; contract satisfied vs pending display. (`3J:L1063-L1103`)
- **P3J-E053** [E/LIMIT]: Plan load optional (None ok); do not auto-create plans. (`3J:L1089-L1096`)
- **P3J-E054** [E/CAP]: 3J.WIRE.STUBS: TreeSitterAstEvaluator + DkgQuerierAdapter replace PassAstEvaluator/EmptyDkgQuerier; max_depth<=20; visited set on BFS; lazy DKG init; fallback EmptyDkgQuerier on init fail with warning. (`3J:L1106-L1201`)
- **P3J-E055** [E/DEFER]: Stub inventory defers OllamaProvider/Embedder todos, NetworkVerifier, NoopTestRunner, OrtSession.embed, VectorIndex readiness to other phases. (`3J:L1120-L1127`)
- **P3J-E056** [E/OBS]: 3J.13 metrics phase_3J.toml fields: latencies, dola_*, turboquant_*, milp_*, steering_*, multiplexing_enabled, ssb_action_type_count, t1/t2/t3 hook medians, compile_time_delta_sec; non-vllm build still passes. (`3J:L1204-L1244`)
- **P3J-E057** [E/SURFACE]: 3J.14 WIRE.E2E: elai.bat single command; primary sidecar DoLa+TurboQuant+NF4; ollama fallback; QueryPipeline retrieval; release elai.exe. (`3J:L1246-L1253`)
- **P3J-E058** [E/FLOW]: Ollama fallback on sidecar health fail or mid-session HTTP error; re-probe each turn; both down → clear error no crash; user sees "[degraded] using ollama fallback" prompt prefix. (`3J:L1263-L1277`)
- **P3J-E059** [E/CFG]: elai_cc.toml [backend] fallback_url default http://localhost:11434. (`3J:L1277-L1277`)
- **P3J-E060** [E/CAP]: Knowledge retrieval: BM25 index cwd; QueryPipeline BM25+CatRag+Arag; optional CrossEncoder; inject top-K into Slot 4 CodeContext; update index on file_write/edit; DKG PageRank if dkg.db exists. (`3J:L1279-L1297`)
- **P3J-E061** [E/FLOW]: Startup sequence: probe/start sidecar or ollama; BM25; QueryPipeline; Ssb/Fsm/SessionDag/RuleEngine/ContextBudget/PlanSkeleton; banner backend+index+session; REPL. (`3J:L1307-L1314`)
- **P3J-E062** [E/TEST]: E2E accept: REPL ≤60s excluding model load; both backends complete; BM25 context in prompt; non-zero CodeContext slot; release binary path. (`3J:L1341-L1348`)
- **P3J-E063** [E/CAP]: 3J.15 31B CPU offload: ≥2 steering vectors stored; DoLa 20 prompts buckets [0,20),[20,40),[40,60); TurboQuant codebook validate; EAGLE-3 architecture only, training deferred 48GB; metrics phase_3J_31b.toml. (`3J:L1350-L1428`)
- **P3J-E064** [E/DEFER]: 100B Q3 validation gated on 48GB GPU; SUBSTITUTE_MODEL escalation path test; metrics phase_3J_100b.toml if executed. (`3J:L1430-L1441`)
- **P3J-E065** [E/TEST]: Exit criteria list: VllmProvider+ModelPool; DoLa/steering E2B+31B; TurboQuant decision; multiplexing+preload; workload profiles; MILP decision; MCP; hooks SLOs; AppContainer; 17 SSB types; zero 0-3I regressions; EAGLE-3 arch deferred. (`3J:L1446-L1462`)
- **P3J-E066** [E/STATUS]: 3J.ADDENDUM shipped post-baseline: A-9 ETCLogitsProcessor; A-17/A-25 GG mean_logprob+entropy_mean; A-10 ConstrainedDecodingConfig grammar/JSON; A-26 routing_rules; A-27 bash safety_filter; A-28 git_commit; A-31 action_classifier; A-8 steering batch mode. (`3J:L1517-L1569`)
- **P3J-E067** [E/SURFACE]: Addendum surfaces: /research JSON-mode uses constrained decoding; COMMITTING FSM phase invokes git_commit tool; calibrate_etc_threshold.py; data/steering_contrast_pairs.toml. (`3J:L1525-L1569`)
- **P3J-E068** [E/SURFACE]: User-facing CLI verbs/flags in 3J body: elai / elai --local; elai.bat; /mode local\ (`api\`)
- **P3J-I001** [I/DEP]: 3J.COMPOSE and 3J.LOCAL are PRE_GATE prerequisites for PIPELINE/E2E but are not defined as step headings in this file (likely baseline/other chunk). (`3J:L685-L686, L1257-L1257`)
- **P3J-I002** [I/CFG]: Summary "can_fit_onnx()" for multiplexing does not appear in 3J.6; source API is can_colocate (and can_fit may live elsewhere). (`3J:L368-L369`)
- **P3J-A001** [A/CAP]: Exact vLLM plugin/hook API for full DoLa logits remains VERIFY; degraded path always required until proven. (`3J:L76-L79, L210-L214, L1491-L1493`)
- **P3J-A002** [A/CFG]: Contrast prompt pairs for Auditor skepticism and Context-compiler completeness marked TBD at implementation. (`3J:L308-L313`)
- **P3J-A003** [A/FLOW]: hybrid/auto role_override vs later UNIFIED removal of Hybrid/Auto dead variants (session summary) leaves production mode enum history ambiguous. (`3J:L705-L712, L870-L870`)
- **P3J-A004** [A/TEST]: 3J.14 "NF4" in sidecar primary stack is named without algorithm detail in this file. (`3J:L1249-L1249`)
- **P3J-C001** [C/STATUS]: Step numbers in body (3J.6 multiplex+preload, 3J.7 profiles, 3J.8 MILP, 3J.9 MCP…) differ from MASTER summary bundling (3J.6 can_fit_onnx; 3J.7-8 preload+profiles; 3J.9 MILP; 3J.10-13 MCP/hooks/AppContainer/SSB). (`3J:L352-L681 vs MASTER:L370-L373`)
- **P3J-C002** [C/STATUS]: Opening says final step 3J.14 applies 31B; body renumbers 31B validation to 3J.15 and 3J.14 becomes WIRE.E2E. (`3J:L15-L16, L1246-L1350`)

### 1H: conditional SPLADE keyword expansion

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-1H.txt`; atoms in `PHASE-006 / DEEP-PHASE-006.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-006/IX-001 — Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this).; PHASE-006/IX-015 — 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop.; PHASE-006/IX-019 — 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation.

**Behaviors**

- **P1H-E001** [E/STATUS]: Phase 1H is conditional SPLADE keyword expansion evaluation; prerequisites Phase 1G complete. (`1H:L1-L2`)
- **P1H-E002** [E/CAP]: Goal: evaluate SPLADE sparse keyword expansion behind cfg(feature="splade"); compare prithivida/Splade_PP_en_v1 vs naver/splade-v3; pick better on code corpus then keep/drop. (`1H:L6-L9`)
- **P1H-E003** [E/FAIL]: DROP if <5% recall improvement over BM25-only on test corpus. (`1H:L10-L10`)
- **P1H-E004** [E/LIMIT]: CRITICAL RISK: BERT WordPiece vocab (30522 tokens) mismatches code tokens (fn, impl, async, Vec<T>, unwrap()); phase MUST measure vocabulary coverage before concluding SPLADE helps on code. (`1H:L12-L17`)
- **P1H-E005** [E/DEP]: Pre-read knowledge.txt L3.25 SPLADE (L57-62), query pipeline (L88); architecture-overview §10 ort/tokenizers/model candidates; knowledge.txt §6 circuit breaker; §7 optional SPLADE load. (`1H:L19-L24`)
- **P1H-E006** [E/LIMIT]: knowledge.txt L58 typo "prithvida" and missing SPLADE-v3/quantized/vocab-risk; owned by s-702732; update when unlocked. Correct spelling is prithivida. (`1H:L24-L24, L34-L34`)
- **P1H-E007** [E/FAIL]: R24: SPLADE ONNX weight ~532MB vs marginal code-search gain. R25: NL benchmarks (29% nDCG@10 ESCI) may not transfer to code. R26: ort downloads prebuilt from Microsoft (air-gap risk). (`1H:L27-L29`)
- **P1H-E008** [E/DEP]: R27: ort does NOT include tokenization; needs separate tokenizers crate (may exist from 1G cross-encoder). R28: ort is unconditional for GraphCodeBERT/Granite; feature-gating SPLADE only gates SPLADE Session init, not ort. (`1H:L30-L31`)
- **P1H-E009** [E/LIMIT]: R29: programming tokens not in BERT vocab and split to char n-grams, reducing expansion quality. R30: 10-query eval underpowered; TREC standard ≥50 topics; effects <~15% undetectable with 10 queries. (`1H:L32-L33`)
- **P1H-E010** [E/EVIDENCE]: Grounding: PP_en_v1 ONNX 532MB main / 436MB quantized; ort pyke; RRF with BM25 GROUNDED knowledge.txt; 5% drop criterion GROUNDED; code-search recall UNGROUNDED; splade-v3 VERIFIED arxiv 2403.06789; 0.3 vocab threshold UNGROUNDED heuristic. (`1H:L37-L50`)
- **P1H-E011** [E/CAP]: Step 1: optional ort dep; create knowledge/splade.rs fully behind cfg(feature="splade"); stub SpladeExpander and expand_query returning empty Vec<(String,f32)>; re-export from mod.rs gated. (`1H:L55-L68`)
- **P1H-E012** [E/TEST]: Step 1 checks: cargo check no features compiles without SpladeExpander; cargo check --features splade includes module; non-gated code must not contain SpladeExpander. (`1H:L70-L87`)
- **P1H-E013** [E/LIMIT]: Do NOT test for ort absence; ort always present for other models. (`1H:L78-L80`)
- **P1H-E014** [E/TEST]: Step 2 TDD: feature-gated tests/knowledge/splade_tests.rs — expand_query synonym terms, empty query empty vec, long >10000 chars no panic, RRF differs from BM25-only, property finite weights >=0, integration recall BM25 vs BM25+SPLADE. (`1H:L90-L103`)
- **P1H-E015** [E/TEST]: Step 2 acceptance: ≥5 unit + 1 property + 1 integration; tests FAIL (red); at least one asserts non-empty expand; property corpus includes Unicode. (`1H:L105-L120`)
- **P1H-E016** [E/CFG]: Model path default .elai_cc/models/splade/model.onnx; override SPLADE_MODEL_PATH env. (`1H:L130-L131`)
- **P1H-E017** [E/CAP]: SpladeExpander::new loads ONNX via ort::Session and BERT tokenizer (vocab.txt + tokenizer.json). expand_query tokenizes to input_ids/attention_mask/token_type_ids, runs inference, returns top-k (token, weight). (`1H:L133-L136`)
- **P1H-E018** [E/CAP]: model_variant enum PrePooled vs RawMLM: PP_en_v1 may output pre-pooled; splade-v3 optimum export is RawMLM [1,seq,30522] requiring max-pool+ReLU+log1p in Rust. (`1H:L137-L139`)
- **P1H-E019** [E/OBS]: vocab_coverage(corpus_tokens)->f64 fraction of unique tokens mapping to single WordPiece; if coverage <0.3 SPLADE unlikely to help. (`1H:L140-L140`)
- **P1H-E020** [E/CAP]: rrf_fuse(bm25, splade, k=60, weight_bm25=0.5, weight_splade=0.5); ranks 1-based; k=max(k,1) prevents div-by-zero. (`1H:L141-L141`)
- **P1H-E021** [E/RECOVERY]: Circuit breaker on ONNX calls (knowledge.txt §6); SPLADE failure returns empty expansion so BM25-only continues. (`1H:L142-L143`)
- **P1H-E022** [E/CFG]: Both model candidates share same interface; only model_path differs: A PP_en_v1 532/436MB; B splade-v3 ONNX export. (`1H:L144-L147`)
- **P1H-E023** [E/TEST]: Step 3 adversarial: code-specific vocab_coverage on tree-sitter identifiers not all tokens; corrupted 1-byte model returns Err not panic; non-English no panic; expand_query p50 <2s/100-char excluding warmup; k=0 clamps; missing tokenizer files Err. (`1H:L159-L181`)
- **P1H-E024** [E/FLOW]: 4a early-exit: extract code identifiers via tree-sitter; if code-specific coverage <0.3 EARLY DROP without recall; log rationale; skip 4b-4d. Coverage 0.2-0.4 may still proceed to calibrate threshold. (`1H:L193-L201`)
- **P1H-E025** [E/CFG]: 4b models pre-staged outside session; quant Jaccard top-50 terms A-full vs A-quant; if overlap>=0.95 prefer quant (saves 96MB + ~3x). Jaccard only for same-model quant, NOT cross-model. (`1H:L203-L211`)
- **P1H-E026** [E/FLOW]: 4b cross-model: quick recall@10 on 10 of 50 queries; pick higher mean; if within 1% pick faster; log to workflow/reasoning/phase-1H-measurements.md. (`1H:L212-L216`)
- **P1H-E027** [E/TEST]: 4c corpus: 100 code files, 50 queries; ≥15 vocab-mismatch, ≥15 distractors, ≥10 straightforward keyword; Wilcoxon signed-rank on paired recall@10; exclude bm25_recall==0 from delta %. (`1H:L218-L230`)
- **P1H-E028** [E/FLOW]: 4d KEEP if mean delta>=5% AND p<0.05; DROP if delta<5% OR p>=0.05; edge delta>=5% but p>=0.05: DROP default, preserve module, log inconclusive. (`1H:L232-L236`)
- **P1H-E029** [E/TEST]: Adversarial: ≥15 distractor queries; ≥15 code-syntax queries; all 50 queries run (no silent drop); clear caches between runs; ≥20 non-zero paired deltas else DROP; quant Jaccard >=0.80 vs full. (`1H:L248-L261`)
- **P1H-E030** [E/STATE]: Three exit outcomes: EARLY DROP vocab; DROP stats; KEEP. All preserve module behind cfg; DROP disables feature by default. KEEP: circuit breaker + RRF + winning model documented. (`1H:L273-L280`)
- **P1H-E031** [E/FLOW]: With splade disabled, SPARSE_ONLY cascade must degrade to BM25_ONLY (no dead path). (`1H:L281-L281`)
- **P1H-E032** [E/OBS]: Measurements: vocab_coverage_code, model_comparison table, recall_delta_percent mean+std, wilcoxon_p_value, splade_query_latency_ms p50/p95, model_load_time_s, binary_size_delta_mb. (`1H:L283-L290`)
- **P1H-A001** [A/CFG]: 0.3 vocab-coverage threshold is ungrounded heuristic, not published study; 0.2-0.4 band may still measure recall. (`1H:L201-L201`)
- **P1H-A002** [A/TEST]: If Candidate B unavailable, proceed with A only; full A/B protocol partially skipped. (`1H:L207-L207`)
- **P1H-I001** [I/DEP]: tokenizers may already exist from Phase 1G; step adds only if missing. (`1H:L128-L134`)

### 3I: deterministic simulation testing

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3I.txt`; atoms in `PHASE-006 / DEEP-PHASE-006.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-006/IX-004 — 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence.; PHASE-006/IX-005 — 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary.; PHASE-006/IX-006 — 3I entropy quarantine (zero raw time/rng in orchestrator) is a hard PRE_GATE for BSP determinism and for any later seed-replay of isolation tests.; PHASE-006/IX-021 — 3I VCR uses blake2s for prompt hashing while much of ELAI elsewhere bans blake2; this phase-local choice is intentional for cassette keys.

**Behaviors**

- **P3I-E001** [E/STATUS]: Phase 3I deterministic simulation testing (DST); prerequisites 0.8 entropy quarantine + 3A-3D FSM/SSB/agents/BSP. NOT optional. (`3I:L1-L17`)
- **P3I-E002** [E/CAP]: Goal: prove FSM+SSB+BSP via deterministic simulation; reference model digital sibling is infallible oracle; same seed = same output. (`3I:L6-L9`)
- **P3I-E003** [E/DEP]: Pre-read orchestrator BSP/channels/invariants; core KEY TYPES + entropy traits L67-92; external-analysis DST L268-305. (`3I:L11-L14`)
- **P3I-E004** [E/CAP]: VCR pattern: intercept HTTP at transport (not LlmProvider trait); blake2s hash of full request body; store (hash,response) msgpack cassettes; DST lookup returns recorded response with simulated latency; missing hash fails (prompt drift). (`3I:L22-L40`)
- **P3I-E005** [E/CAP]: Trait-level LlmProvider mock is complementary unit pattern; VCR catches ollama cache anomalies trait mocks miss. (`3I:L42-L45`)
- **P3I-E006** [E/CAP]: TraceOps records entire agent trajectory (GRC, tools, branches) as structured trace; DST feeds trajectory instead of live LLM. (`3I:L47-L51`)
- **P3I-E007** [E/CAP]: Custom SQLite VFS traps write/fsync into in-memory buffer; sim PRNG dictates I/O latency; enables torn WAL, deterministic latency, crash recovery tests; ~200-400 LOC unsafe C FFI. (`3I:L59-L77`)
- **P3I-E008** [E/DEP]: Custom VFS must use madsim timer queue as single deterministic time source. SPIKE: evaluate if madsim libc I/O hooks make custom VFS unnecessary. (`3I:L72-L81`)
- **P3I-E009** [E/FAIL]: R1 madsim Windows unverified — spike first. R2 not drop-in tokio — needs cfg(madsim) and madsim-tokio swap. R3 must use proptest-state-machine ReferenceStateMachine not hand-rolled. R4 RUSTFLAGS madsim full rebuild. R5 10k cases need 10min timeout or reduced local count. (`3I:L84-L94`)
- **P3I-E010** [E/TEST]: 3I.1 madsim 0.2.34 dev-dep spike: two nodes send/receive on Windows with RUSTFLAGS; fail -> document manual SeededClock+SeededRng scheduler alternative. (`3I:L103-L119`)
- **P3I-E011** [E/CAP]: 3I.2 SimulatorRuntime wraps SeededClock+SeededRng+madsim; advance_time, inject_fault, snapshot_state. (`3I:L126-L127`)
- **P3I-E012** [E/CAP]: FsmReferenceModel implements ReferenceStateMachine; State=ReferenceState (FSM+SSB rows+agents+channels); Transition=FsmEvent; deliberately simpler than real system. (`3I:L128-L135`)
- **P3I-E013** [E/DATA]: FsmEvent 10 variants: PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary. (`3I:L136-L144`)
- **P3I-E014** [E/FLOW]: SuperstepBoundary composite atomic: clear_non_accumulating_topics + execute + clear_ephemeral + evidence_gate_check. AgentCleanup models crashed->cleaned after crash. (`3I:L139-L144`)
- **P3I-E015** [E/CAP]: StateMachineTest: SUT=actual FSM+SSB; apply returns SUT by value (ownership model); check_invariants compares FSM state, SSB row counts, active agents, LastValue/Topic/Ephemeral channel state. (`3I:L145-L152`)
- **P3I-E016** [E/CFG]: dev-deps proptest v1.11+, proptest-state-machine v0.8+, madsim 0.2.34 cfg-gated; #[cfg(not(madsim))] stubs skip IPC sim gracefully. (`3I:L154-L157`)
- **P3I-E017** [E/LIMIT]: Reference model LOC <30% of real FSM LOC (oracle simplicity). Every FsmState reachable via BFS on reference state space. (`3I:L165-L169`)
- **P3I-E018** [E/TEST]: 3I.3 verify workspace: cargo test --workspace 0; warnings <= baseline. (`3I:L178-L188`)
- **P3I-E019** [E/TEST]: 3I.4 prop_state_machine sequential 1..200; PROPTEST_CASES=10000 CI 10min hard, 256 local; check_determinism takes bare static fn not closure; synthetic regression generates proptest-regressions/ checked into git; TDD red until 3I.8. (`3I:L190-L230`)
- **P3I-E020** [E/TEST]: 3I.5 PRE: entropy quarantine — zero Instant::now\ (`thread_rng\`)
- **P3I-E021** [E/TEST]: 3I.5 BSP: 100 identical checkpoints same seed; UntrackedValue never in checkpoint; EphemeralValue zero-write clears MISSING + write_count reset; Topic appends follow SSB seq; SuperstepBoundary two-phase Topic reset. (`3I:L241-L265`)
- **P3I-E022** [E/TEST]: 3I.6 madsim IPC (prep 4A): packet_loss_rate 0.3; send_latency 0..500ms Duration range; clog_link asymmetric; SIGKILL mid-message cleanup no orphaned SSB; burst 100% loss 5 msgs then 0%; seed replay; skip+document if madsim unavailable Windows. (`3I:L267-L300`)
- **P3I-E023** [E/TEST]: 3I.7 adversarial: anti-spiral 3x same GRC / >3 same-file edits STOP; evidence_gate_passes false without artifacts; context budget adaptive truncate; SSB concurrent same-path single-writer; demonic proptest interleavings. Whitespace-only triples still escalate; zero-byte evidence rejected. (`3I:L302-L328`)
- **P3I-E024** [E/CAP]: 3I.8 implement SimulatorRuntime with with_seed; FsmReferenceModel pure apply; SuperstepBoundary atomic clears; madsim::check_determinism bare fn; NetSim partition/delay/drop + buggify_with_prob; LOC ratio python assert. (`3I:L330-L376`)
- **P3I-E025** [E/OBS]: 3I.9 metrics phase_3I.toml: state_coverage_pct (>80% valid transition pairs), bugs_found, determinism_verified (100%), regression_count, ops_per_sec (ungrounded 100K target), compile_time_delta_sec, madsim_compile_time_sec. (`3I:L378-L401`)
- **P3I-E026** [E/TEST]: Exit: 10k+ sequences match; same seed identical; 100 BSP runs; IPC faults pass or documented skip; VCR cassette identical SSB; prompt hash mismatch reported; TraceOps all transitions; SQLite VFS spike done or hooks proven. AUTONOMOUS not enabled (4G). (`3I:L404-L417`)
- **P3I-E027** [E/EVIDENCE]: madsim drop-in claim FALSE; Windows UNGROUNDED; proptest-state-machine v0.8 VERIFIED; orchestrator.txt has NO DST section (SPEC_GAP — design in external-analysis); channel types GROUNDED orchestrator §§4-5. (`3I:L420-L468`)
- **P3I-A001** [A/DEP]: Whether custom SQLite VFS is required vs madsim hooks is unresolved until spike. (`3I:L79-L81`)
- **P3I-A002** [A/CFG]: Windows madsim availability and manual-scheduler equivalence details unresolved until 3I.1. (`3I:L85-L87, L110-L111`)
- **P3I-I001** [I/DET]: blake2s for VCR request hashing is the named hash in this phase (not blake3). (`3I:L30-L37`)
- **P3I-E028** [E/DATA]: ReferenceState includes FSM state, SSB row counts, agent lifecycle set, and channel state (LastValue current, Topic vec contents, Ephemeral presence); UntrackedValue excluded from reference channel state. (`3I:L129-L130, L339-L340`)
- **P3I-E029** [E/TEST]: 3I.4 adversarial: always-true preconditions must still produce at least one invariant failure; mutating one SSB row count in reference must be caught by check_invariants. (`3I:L222-L228`)
- **P3I-E030** [E/TEST]: 3I.5 adversarial: serialize checkpoint must not contain UntrackedValue marker; crash after ephemeral clear and SSB replay must restore EphemeralValue=MISSING. (`3I:L257-L261`)
- **P3I-E031** [E/TEST]: 3I.6 adversarial: crash agent mid-SSB-write yields no orphaned rows after cleanup; RUSTFLAGS madsim full rebuild expected and measured in compile_time metrics. (`3I:L286-L297`)
- **P3I-E032** [E/CFG]: madsim net::Config fields: packet_loss_rate f64, send_latency Range<Duration> with explicit Duration construction, NetSim::clog_link, buggify_with_prob. (`3I:L274-L277, L359-L360, L424-L426`)
- **P3I-E033** [E/TEST]: 3I.8 acceptance requires PROPTEST_CASES=10000 fsm_reference_model_matches within 10 min, all ipc_ tests under cfg(madsim), and cross-platform LOC ratio script. (`3I:L361-L376`)

### 3K: RAG production hardening

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3K.txt`; atoms in `PHASE-006 / DEEP-PHASE-006.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-006/IX-001 — Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this).; PHASE-006/IX-002 — 3K hardening composes 1E A-RAG/CatRAG stubs, 1G reranker, 1C embeddings, 1A DKG grammars, and 3J sidecar only for HyDE.; PHASE-006/IX-003 — 3K.4 QueryTier intentional quality selection is explicitly distinct from 1I SearchMode fault-degradation cascade.; PHASE-006/IX-015 — 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop.; PHASE-006/IX-017 — 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc.; PHASE-006/IX-019 — 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation.; PHASE-006/IX-022 — Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network.

**Behaviors**

- **P3K-E001** [E/STATUS]: Phase 3K RAG production hardening; prerequisites 1E CatRAG/A-RAG, 1G cross-encoder, 3J sidecar for HyDE (3K.3 only). (`3K:L1-L3`)
- **P3K-E002** [E/CAP]: Upgrade crates/knowledge/ with adaptive chunking, query expansion, HyDE, quality-tier routing; better retrieval → smaller prompts for local LLMs. (`3K:L7-L14`)
- **P3K-E003** [E/STATUS]: Current stack: DKG 1A, BM25 1B, dual vectors 1C, PageRank 1D, CatRAG/A-RAG 1E, reranker 1G, SPLADE 1H KEEP/DROP done, resilience 1I. Missing: AST chunks, expansion, HyDE, tier routing, recall/MRR metrics. (`3K:L17-L35`)
- **P3K-E004** [E/FAIL]: R1 HyDE latency 1-3s mitigated by tier routing. R2 AST chunking reuses DKG LanguageGrammar. R3 expansion noise mitigated by template (not LLM) and 2-3 variants. (`3K:L38-L44`)
- **P3K-E005** [E/CAP]: 3K.1 adaptive chunking: replace 50-line windows; split Rust at fn/impl/struct/enum/mod; Python def/class; TS/Go function/class; non-code markdown headings or blank paragraphs; fallback 50-line on AST fail. (`3K:L49-L60`)
- **P3K-E006** [E/CFG]: Chunk targets 200-800 tokens; 50-token overlap; metadata file_path, start_line, end_line, chunk_type; Tantivy schema adds chunk_type + line range; re-index if chunk format version changes. (`3K:L61-L66`)
- **P3K-E007** [E/TEST]: 3K.1: boundaries align AST; no chunk >1000 tokens; overlap exists; parse fail falls back not crash; BM25 results include chunk_type; reindex without data loss. (`3K:L67-L76`)
- **P3K-E008** [E/CAP]: 3K.2 multi-query template expansion (no LLM): keyword extraction; CamelCase/snake_case symbol guess; synonym error→failure/exception; 2-3 variants for >5 words; skip <5 words. (`3K:L78-L93`)
- **P3K-E009** [E/FLOW]: 3K.2 run BM25+vector per expansion; merge dedupe by chunk ID; then cross-encoder rerank. Accept: Recall@5 improves; no >100ms latency regression on simple queries. (`3K:L91-L101`)
- **P3K-E010** [E/CAP]: 3K.3 HyDE: sidecar POST /v1/completions max_tokens=128 temperature=0.7 prompt Write short technical paragraph; embed hypothesis with 1C models; search with hypothesis embedding; RERANK against ORIGINAL query. (`3K:L103-L118`)
- **P3K-E011** [E/CFG]: HyDE cache hypothetical embeddings 5 minutes same query. Sidecar timeout 3s; unavailable/timeout degrade to direct search never block. Integrates 1I circuit breaker. (`3K:L119-L131`)
- **P3K-E012** [E/TEST]: 3K.3: HyDE for complex; simple skip zero latency regression; MRR improved on complex; graceful sidecar outage. (`3K:L123-L136`)
- **P3K-E013** [E/CAP]: 3K.4 QueryTier: FAST BM25-only <20ms; STANDARD hybrid+rerank no HyDE <200ms; DEEP expansion+HyDE+hybrid+rerank <5s. Distinct from 1I SearchMode degradation. (`3K:L138-L159`)
- **P3K-E014** [E/CFG]: Auto-detect: <5 words FAST; 5-15 code keywords STANDARD; >15 question/debug/arch DEEP. QueryPipeline::search optional tier; default AUTO backwards compatible. (`3K:L152-L157`)
- **P3K-E015** [E/TEST]: 3K.4: FAST skips reranker+vector; DEEP includes HyDE; three tiers measurably different latency; quality DEEP>=STANDARD>=FAST; existing callers unchanged. (`3K:L160-L168`)
- **P3K-E016** [E/TEST]: 3K.5 bench: 30 queries (10 simple/10 code/10 complex) known-correct chunks; Recall@5/@10, MRR, latency p50/p95 per tier; write .elai_cc/metrics/phase_3K_rag.toml; regress >5% blocks ship. (`3K:L170-L190`)
- **P3K-E017** [E/EVIDENCE]: Papers: A-RAG 2602.03442, CatRAG 2602.01965, Deep Research 2508.12752, Deep Search Agents 2508.05668; interface maps workflow/papers/*; independent of 4D web research. (`3K:L193-L199`)
- **P3K-E018** [E/CAP]: 3K.6 fix A-RAG keyword_search stub: Score_kw=SUM(count(k,text)*len(k)) case-insensitive; longer keywords weighted higher (arag-interface L72-77). (`3K:L202-L211`)
- **P3K-E019** [E/CAP]: 3K.6 fix semantic_search stub: dual-vector 1C; 3x oversample top_k*3; group by parent; score=MAX similarity among matches. (`3K:L212-L216`)
- **P3K-E020** [E/CAP]: 3K.6 progressive disclosure: search returns SNIPPETS not full chunks; ChunkRead full text only on explicit request; ContextTracker prevents re-read; ≥50% token savings goal. (`3K:L217-L224`)
- **P3K-E021** [E/TEST]: 3K.6 adversarial: short 10-line with 3 hits outscores 500-line with 3 (length domination); 60 candidates retrieval <100ms; arag contract tests pass. (`3K:L230-L246`)
- **P3K-E022** [E/CAP]: 3K.7 Symbolic Anchoring: rule-based CamelCase/UPPER_SNAKE entity extract; match DKG; weak PPR reset_weight(e)=epsilon*\ (`passages(e)\`)
- **P3K-E023** [E/CAP]: 3K.7 Key-Fact enhancement: w_hat=w*(1+beta*I(u,p in verified_triples)); beta=2.5; verified triples high-confidence DKG edges; pure algorithmic no LLM. (`3K:L261-L267`)
- **P3K-E024** [E/DEFER]: CatRAG Innovation 2 Dynamic Edge Weighting LLM-scored DEFERRED; too expensive for local code retrieval; re-evaluate after 4D web research LLM-in-loop. (`3K:L268-L271`)
- **P3K-E025** [E/TEST]: 3K.7: symbolic_anchor and key_fact_enhance symbols present; query 'how does the system work' extracts 0 entities; with beta=2.5 non-verified still in top-5; Recall@5 >= 3K.6. (`3K:L272-L294`)
- **P3K-E026** [E/OBS]: 3K.8 re-run same 30 queries; deltas Recall@5/@10/MRR/tokens-per-query/latency; phase_3K_rag_improved.toml; tokens-per-query >=30% reduction; no metric regress >5%. (`3K:L297-L317`)
- **P3K-A001** [A/CFG]: Exact synonym lexicon beyond error/failure/exception not fully specified. (`3K:L88-L90`)
- **P3K-A002** [A/CFG]: AUTO routing keyword lists for code/debug/architecture are descriptive not enumerated. (`3K:L152-L155`)
- **P3K-I001** [I/SURFACE]: 3K has no new operator UI; surfaces are QueryTier API, metrics TOML artifacts, and retrieval result snippet fields. (`3K:L156-L157, L181-L182, L308-L308`)
- **P3K-E027** [E/INT]: 3K.1 touches knowledge/src/bm25.rs chunking and knowledge/src/dkg.rs grammar reuse. (`3K:L76`)
- **P3K-E028** [E/INT]: 3K.2–3K.4 touch knowledge/src/pipeline.rs; QueryTier exported from knowledge/src/lib.rs. (`3K:L101, L136, L168`)
- **P3K-E029** [E/INT]: 3K.3 reuses providers/src/sidecar_client.rs; never blocks retrieval on sidecar outage. (`3K:L116-L122, L136`)
- **P3K-E030** [E/INT]: 3K.6–3K.7 touch knowledge/src/arag.rs and catrag.rs; contracts tests/contracts/arag_contract.rs must still pass. (`3K:L241-L246, L288-L294`)
- **P3K-E031** [E/EVIDENCE]: 3K.6–3K.7 grounding cites arag-interface.md and catrag-interface.md line ranges; progressive disclosure 90% paper figure is grounded interface claim, local acceptance uses ≥50%/≥30% token reductions. (`3K:L242-L245, L289-L293, L237-L239, L314`)
- **P3K-E032** [E/TEST]: Regression gate both 3K.5 and 3K.8: any metric regresses >5% vs prior baseline requires fix before complete. (`3K:L182-L185, L309-L312`)

### 4A: Tier-C agent process isolation

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-4A.txt`; atoms in `PHASE-006 / DEEP-PHASE-006.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-006/IX-004 — 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence.; PHASE-006/IX-005 — 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary.; PHASE-006/IX-007 — 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch.; PHASE-006/IX-008 — 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging.; PHASE-006/IX-009 — 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern.; PHASE-006/IX-010 — 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list.; PHASE-006/IX-011 — 4A ShortCoder deferral to 4B orchestrator placement is a cross-phase WARN-3 handoff outside this chunk's implementation.; PHASE-006/IX-012 — 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria.; PHASE-006/IX-017 — 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc.; PHASE-006/IX-018 — Budget authority handoff: 2C in-process budgets become advisory under 4A orchestrator ThinkStep counting.; PHASE-006/IX-020 — Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT.; PHASE-006/IX-022 — Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network.

**Behaviors**

- **P4A-E001** [E/STATUS]: Phase 4A Tier-C agent process isolation; prerequisite Phase 3 INTERACTIVE pipeline gates all pass. Foundational for all later Phase 4. (`4A:L1-L9`)
- **P4A-E002** [E/CAP]: Agents run as isolated OS processes; no agent sees enforcement, other agents, or harness; orchestrator communicates via unix socket / named pipe. (`4A:L6-L8`)
- **P4A-E003** [E/DEP]: Pre-read orchestrator §12, tools §8-9, reasoning §4 GroundingRequest IPC, audit-results-phase4 all 9 BLOCK+WARN, architecture-overview §9 roles. (`4A:L11-L16`)
- **P4A-E004** [E/SEC]: BLOCK-1: two seccomp profiles IPC_BASELINE (all) + NETWORK_EXTRA (researcher only); BPF checks socket domain arg not blanket socket. AF_UNIX baseline; AF_INET researcher only. (`4A:L19-L21`)
- **P4A-E005** [E/SEC]: BLOCK-2: researcher uses veth+iptables NOT CAP_NET_RAW; zero capabilities any role; prctl(PR_SET_NO_NEW_PRIVS) before exec. (`4A:L22-L23`)
- **P4A-E006** [E/SEC]: BLOCK-3: agent binary has NO LLM client; InferRequest→orchestrator→providers→InferResponse. (`4A:L24-L25`)
- **P4A-E007** [E/SEC]: BLOCK-4: GRC budget orchestrator-side; counts ThinkStep externally; agent budget advisory only. (`4A:L26-L27`)
- **P4A-E008** [E/SEC]: BLOCK-5: sockets in 0700 /tmp/elai_cc_{session}/; SO_PEERCRED on accept; unlink on agent death. (`4A:L28-L29`)
- **P4A-E009** [E/SEC]: BLOCK-6: auditor self-discovery → PROPOSED never auto-CANDIDATE; human approval; independent second auditor validates predicates. (`4A:L30-L32`)
- **P4A-E010** [E/SEC]: BLOCK-7: Windows weaker; AppContainer preferred; restricted token+ACL deny documented-weaker; Linux is production security target. (`4A:L33-L35`)
- **P4A-E011** [E/SEC]: BLOCK-8: self-mod locks BOTH IPC canonical realpath AND FS RO mount; BashRequest+GitCommit also filtered. (`4A:L36-L37`)
- **P4A-E012** [E/SEC]: BLOCK-9: adversarial IPC test battery required. (`4A:L38-L38`)
- **P4A-E013** [E/CAP]: 4A.1 AgentProcess wraps Command; IPC unix socket Linux / named pipe Windows; frame: 1-byte version + 4-byte length + postcard; MAX_FRAME_SIZE 10MB. (`4A:L43-L54`)
- **P4A-E014** [E/DATA]: Per-type size caps: AgentHeartbeat <1KB, ThinkStep <64KB, ToolResult <10MB, AgentResponse <context_limit. (`4A:L53-L54`)
- **P4A-E015** [E/DATA]: Exactly 15 message types: AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest. (`4A:L55-L59`)
- **P4A-E016** [E/CAP]: elai_cc_agent binary: GRC loop + IPC client + structured output parser; MUST NOT contain LLM client, reqwest, network, tools, FSM, SSB, rules, knowledge. (`4A:L60-L63`)
- **P4A-E017** [E/CFG]: ProcessConfig per role: allowed_paths, seccomp_profile, resource_limits. ToolFilter whitelist {allowed_tools}; not in list → generic "tool call rejected by policy". Defense-in-depth on PreToolUse. (`4A:L64-L70`)
- **P4A-E018** [E/CFG]: AgentDispatchConfig authoritative: role, model_preference, tool_filter, rule_domains, grc_config, sandbox_config. Role alone insufficient. (`4A:L71-L74`)
- **P4A-E019** [E/EVENT]: phase_complete SSB action_type at model-swap phase boundaries: {phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes}; atomic durability marker; model unload only after fsync WAL. (`4A:L75-L79`)
- **P4A-E020** [E/DATA]: agent_id UUID v4 assigned at spawn by orchestrator. (`4A:L80-L80`)
- **P4A-E021** [E/FLOW]: Connection sequence listen-before-spawn: create 0700 dir → create socket listen → spawn with path → SO_PEERCRED/SecurityIdentification → version handshake mismatch kills. (`4A:L81-L87`)
- **P4A-E022** [E/TEST]: 4A.1: two binaries elai_cc + elai_cc_agent; cargo tree agent no reqwest/providers; 15 variants const_assert; postcard round-trip each type; 11MB frame rejected pre-alloc. (`4A:L88-L103`)
- **P4A-E023** [E/TEST]: 4A.2: cargo build/test/clippy -D warnings clean; cargo test 3x all pass (temp dir isolation). (`4A:L105-L120`)
- **P4A-E024** [E/TEST]: 4A.3 functional isolation TDD: cannot read .elai_cc ENOENT; NSpid==1 host-verified; AF_INET EPERM; AF_UNIX allowed; researcher HTTPS via veth; 15s heartbeat → SIGKILL; crash exit1 not crash orchestrator; cannot write outside scratch; concurrent agents isolated scratch and no peer AF_UNIX; auditor can read executor output via orchestrator copy; executor cannot read auditor; env only LANG/TERM/PATH/TMPDIR no HOME/USER/API/ELAI_CC_*; wrong PID reject; /proc masked /sys unmounted; RLIMIT_FSIZE 100MB fail write; RLIMIT_AS 512MB OOM agent only. (`4A:L122-L151`)
- **P4A-E025** [E/TEST]: 4A.3 LLM proxy: InferRequest path; no localhost:11434; oversize prompt rejected. (`4A:L153-L157`)
- **P4A-E026** [E/TEST]: 4A.3 GRC budget: orchestrator counts ThinkStep per topic; max_calls injects convergence; per-topic independent; orchestrator authoritative when agent local count disagrees. (`4A:L159-L166`)
- **P4A-E027** [E/TEST]: 4A.3 GroundingRequest proxy: FILE/WEB/SOURCE_CHECK; role-method matrix executor/auditor get FILE/SOURCE/WEB/TEST; context-compiler/prompt-engineer/memory-writer FILE+SOURCE only; TEST_CHECK from context-compiler rejected; counts to budget; no active ThinkStep topic rejected. (`4A:L168-L180`)
- **P4A-E028** [E/TEST]: 4A.3 test names test_4a_isolation_*/llm_proxy_*/grc_budget_*/grounding_*; one assert each; FAIL until 4A.5. (`4A:L182-L192`)
- **P4A-E029** [E/TEST]: 4A.4 adversarial IPC: >10MB reject; length=u32::MAX pre-alloc reject; malformed postcard no crash; unsolicited AgentResponse reject; ToolResult unrequested ToolId reject; 10k ThinkStep burst budget drop; second process SO_PEERCRED reject; SIGKILL socket cleanup respawn; mismatched agent_id reject; version mismatch kill; partial frame timeout cleanup. (`4A:L194-L211`)
- **P4A-E030** [E/TEST]: 4A.4 ReplanRequest: MINOR+evidence auditor fast-track; MAJOR human INTERACTIVE; max 1 replan per agent per phase; no evidence reject; evidence >500 tokens truncate/reject. (`4A:L213-L218`)
- **P4A-E031** [E/TEST]: 4A.4 self-mod locks: WriteFile locked path generic error; path traversal realpath reject; BashRequest locked RO mount block; GitCommit locked diff reject; symlink realpath reject. (`4A:L220-L225`)
- **P4A-E032** [E/TEST]: 4A.4 auditor discovery: PROPOSED not CANDIDATE; human before CANDIDATE; second auditor validates; inflated violation_patterns same-hash capped. (`4A:L227-L231`)
- **P4A-E033** [E/TEST]: 4A.4 crash: orchestrator crash restart SSB consistent; agent crash heartbeat respawn; SSB write BEFORE ToolResult send; partial frame discarded on fd close. (`4A:L233-L237`)
- **P4A-E034** [E/TEST]: 4A.4 Windows cfg: AppContainer no .elai_cc; Job limits memory/process/timeout; named pipe SecurityIdentification; fallback restricted token still tested. Every BLOCK 1-9 and WARN 1-9 has ≥1 test comment. (`4A:L239-L257`)
- **P4A-E035** [E/CAP]: 4A.5 agent: heartbeat every 5s; AgentKill 1s flush. Orchestrator gateway: routes Infer→providers, Tool→tools+rules+visibility, Bash→sandbox, Grounding→knowledge+web; GRC budget external; SSB sync before ToolResult. (`4A:L259-L280`)
- **P4A-E036** [E/SEC]: Linux: clone NEWNS\ (`NEWPID\`)
- **P4A-E037** [E/SEC]: Windows: AppContainer CreateAppContainerProfile preferred; fallback restricted token+Job+ACL; Job ACTIVE_PROCESS=1 PROCESS_MEMORY=512MB JOB_TIME configurable; named pipe SecurityIdentification. (`4A:L297-L302`)
- **P4A-E038** [E/SEC]: mutable=0 rules protected from all automated modification. Self-mod: IPC realpath + RO mount + GitCommit path filter. (`4A:L304-L312`)
- **P4A-E039** [E/DEFER]: ShortCoder post-process MUST run in orchestrator after AgentResponse BEFORE auditor view FS — not in agent; verify in 4B PRE_GATE (WARN-3). (`4A:L314-L317`)
- **P4A-E040** [E/OBS]: 4A.6 metrics phase_4A.toml: IPC p99 <1ms for <64KB; spawn <100ms; memory/agent <30MB; overhead vs in-process <5%; isolation escape fails. Each metric value/unit/timestamp/method. Separation cost estimates 0.1ms/50ms/20MB. (`4A:L337-L373`)
- **P4A-E041** [E/INT]: Every request-routing call site/struct must wire ssb, fsm, clock through single RuntimeClient; no None/uninitialized after construction. (`4A:L427-L427`)
- **P4A-E042** [E/SURFACE]: Operator-facing isolation surfaces: generic policy rejection string; human approval for MAJOR replan (INTERACTIVE gate); human approval for auditor PROPOSED rules; Windows vs Linux isolation strength documentation. (`4A:L67-L68, L215-L216, L228-L230, L33-L35`)
- **P4A-A001** [A/OBS]: IPC/spawn/memory/overhead SLO numbers are UNGROUNDED targets pending 4A.6 empirical measurement. (`4A:L346-L353, L376-L384`)
- **P4A-A002** [A/CFG]: WARN list 1-9 referenced for test mapping but individual WARN bodies not expanded in this phase file. (`4A:L257-L257`)
- **P4A-I001** [I/SURFACE]: No new dashboard panel is specified in 4A; isolation is infrastructure with generic IPC error strings as the agent-visible surface. (`4A:L67-L68`)
- **P4A-E043** [E/CFG]: Heartbeat interval 5s with 15s timeout → orchestrator SIGKILL. AgentKill grants 1s flush window for graceful shutdown. (`4A:L137, L269, L275`)
- **P4A-E044** [E/SEC]: Researcher NETWORK_EXTRA allows AF_INET/AF_INET6 SOCK_STREAM only via BPF argument inspection; other roles EPERM on AF_INET. (`4A:L134-L136, L285-L287`)
- **P4A-E045** [E/SEC]: Socket directory /tmp/elai_cc_{session}/ is NOT mounted into agent namespace; agent receives socket fd or path argument; per-agent scratch isolated. (`4A:L141-L142, L290-L291`)
- **P4A-E046** [E/FLOW]: Orchestrator routes: InferRequest→providers; ToolRequest→tools+rules+visibility_filter; BashRequest→sandbox; GroundingRequest→knowledge+web. (`4A:L276-L277`)
- **P4A-E047** [E/TEST]: Isolation escape acceptance: agent instructed to find .elai_cc/ must fail (measured in 4A.5 and 4A.6). (`4A:L335, L354-L365`)
- **P4A-E048** [E/FAIL]: Generic IPC policy rejection message is "tool call rejected by policy" with no rule ID leakage to agent. (`4A:L67-L68`)
- **P4A-E049** [E/DATA]: ReplanRequest max 1 per agent per phase; MINOR with evidence auditor fast-track; MAJOR requires human INTERACTIVE gate. (`4A:L213-L216`)
- **P4A-E050** [E/EVENT]: SSB write is synchronous with ToolResult dispatch: write BEFORE sending result (WARN-6 fix). (`4A:L236, L279, L404-L405`)
- **P4A-E051** [E/DEP]: Decision doc workflow/reasoning/phase-4A-decisions.md is PRE_GATE for 4A.5 IMPLEMENT. (`4A:L262`)

### 4H: measured bounded self-modification

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4H.txt`; atoms in `PHASE-006 / DEEP-PHASE-006.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-006/IX-004 — 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence.; PHASE-006/IX-007 — 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch.; PHASE-006/IX-008 — 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging.; PHASE-006/IX-009 — 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern.; PHASE-006/IX-010 — 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list.; PHASE-006/IX-012 — 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria.; PHASE-006/IX-013 — 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance.; PHASE-006/IX-014 — 4H FailureTaxonomy expansion closes reasoning-architecture Layer1 with fix templates from higher layers' successful resolutions.; PHASE-006/IX-015 — 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop.; PHASE-006/IX-016 — 4H OSC/REG/EVO ouroboros ports operate per PipelineMode in parallel with MSI and feed SSB action types.; PHASE-006/IX-020 — Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT.

**Behaviors**

- **P4H-E001** [E/STATUS]: Phase 4H measured bounded self-modification (DGM/CEL/DarwinNet-derived), revised 2026-04-15 after adversarial audit (4 BLOCK+8 WARN resolved). Prerequisites 4A Tier C isolation. (`4H:L1-L24`)
- **P4H-E002** [E/CAP]: System improves configs/rules/data from pipeline outcomes via Layer1 metrics (no LLM), Layer1A data-table growth (no LLM), Layer2 staged config proposals, Layer3 structural human-only stubs. Modifying agent NEVER evaluates own modifications. (`4H:L7-L12`)
- **P4H-E003** [E/LIMIT]: Architectural divergence from DGM: 4H mutates CONFIG KEYS and DATA TABLES only, never agent/Rust source recompilation. (`4H:L36-L41`)
- **P4H-E004** [E/CAP]: Class A data-table additive low-risk Layer1A: deprecated_patterns, FailureTaxonomy, lazy-code regexes, source-tier entries; validate FPR over last N pipelines; no staged bench. (`4H:L43-L51`)
- **P4H-E005** [E/CAP]: Class B parameter systemic Layer2: ETC threshold, baby-step LOC, AB-MCTS temps, context weights, freshness half-life, retries, steering coeffs; staged 10/50/200. (`4H:L52-L56`)
- **P4H-E006** [E/FAIL]: R14 reward hacking — SWE-bench rotation + trigger task (DGM Appendix H). R16 resource contention idle-only lowest priority. R53 learned reward bias EVALUATE conditional. (`4H:L67-L72`)
- **P4H-E007** [E/DATA]: PipelineOutcome fields include pipeline_mode SURGICAL\ (`GREENFIELD\`)
- **P4H-E008** [E/DATA]: StaleApiCatch: file_path, deprecated_name, replacement_name, ecosystem, caught_by CatchLayer T1_HOOK\ (`T2_HOOK\`)
- **P4H-E009** [E/DATA]: UnclassifiedError auto-proposes FailureTaxonomy when occurrence_count>=3 across pipelines. (`4H:L118-L123`)
- **P4H-E010** [E/DATA]: TestOutcomeReport 4-category: fail_to_pass, pass_to_pass, fail_to_fail, pass_to_fail (REGRESSION hard reject). (`4H:L125-L129`)
- **P4H-E011** [E/CFG]: Score always [0,1] weighted: first_try 0.3, coverage 0.2, efficiency 0.2, intervention 0.15, findings 0.15 via elai_cc.toml [self_mod.score_weights]. (`4H:L131-L140`)
- **P4H-E012** [E/OBS]: MSI = exponential-decay weighted moving avg of delta_score over N=10; lambda=0.3; weights oldest~0.07 newest 1.0; MSI in [-1,1]; inspired by DarwinNet PSI different formula. (`4H:L142-L152`)
- **P4H-E013** [E/DATA]: ConfigVariant: variant_id, parent_id, score [0,1], children_count, config_blob, created_at, pipeline_outcomes, pipeline_mode, status ACTIVE\ (`ARCHIVED\`)
- **P4H-E014** [E/FLOW]: Parent selection DGM: s_i=sigmoid(10*(alpha_i-0.5)); h_i=1/(1+n_i); p_i proportional s*h; all non-zero; MODE FILTER — SURGICAL parent never for GREENFIELD eval. (`4H:L167-L175`)
- **P4H-E015** [E/CFG]: AllowedConfigKey closed enum Parameter keys: EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc{vram_tier}, ContextSlotWeight{slot}, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, SelfModFrequency. (`4H:L177-L193`)
- **P4H-E016** [E/CFG]: AllowedConfigKey DataTable: DeprecatedPattern, FailureTaxonomyCategory, LazyCodePattern, SourceTierEntry. (`4H:L195-L199`)
- **P4H-E017** [E/SEC]: LOCKED never modifiable even human_approved: score_weights, gate_definitions, auditor_config, benchmark_harness, test_suite, config_variants_schema, safety_locks, lock_list, cloud_monthly_budget, cloud_opus_allowed, cloud_escalation_threshold. (`4H:L201-L207`)
- **P4H-E018** [E/DATA]: SelfModProposal: variant/parent ids, target_config_key AllowedConfigKey, ProposedChange InsertRow\ (`UpdateKey, ssb_evidence, reasoning_trace_evidence, rationale, optional ProjectScope.`)
- **P4H-E019** [E/DATA]: ProjectScope ecosystem/language/pipeline_mode filters prevent cross-ecosystem over-generalization. (`4H:L219-L224`)
- **P4H-E020** [E/DATA]: ReasoningTraceRef pinpoints GRC failure location (agent, phase, step index/type, failure_mode). StagedEvaluation Stage1(10)\ (`Stage2(50)\`)
- **P4H-E021** [E/DATA]: DataTableValidation: FPR=false_positive_count/total; threshold default 0.05; window default 10; verdict PROMOTE\ (`REJECT\`)
- **P4H-E022** [E/DATA]: ConfigVariantSimilarity ouroboros: 0.5*key_overlap+0.3*type_match+0.2*exact_match; STAGNANT if similarity>=0.95 over stagnation_window=3. (`4H:L256-L263`)
- **P4H-E023** [E/STATE]: OscVerdict: Stagnant{window,similarity}, Oscillating{period}, Evolving, Inconclusive{reason}. (`4H:L265-L269`)
- **P4H-E024** [E/DATA]: AcRegression + RegressionReport cross-pipeline AC history; never-passed AC is not regression; distinct from single-pipeline pass_to_fail. (`4H:L271-L290`)
- **P4H-E025** [E/STATE]: EvolutionGateResult Allow{evolved_count}\ (`Withhold{reason,identical_streak}; evolved_count pairs with similarity<0.95.`)
- **P4H-E026** [E/CFG]: SandboxConfig copy parent to .elai_cc/sandbox/{mod_id}/. Layer3 StructuralChange types FSM_STATE\ (`AGENT_ROLE\`)
- **P4H-E027** [E/SEC]: Self-mod agent Tier C EXTRA restrictions: cannot write metrics/gates/auditor/benchmark/tests/schema/AllowedConfigKey def/locked keys; cannot READ lock list; four layers dispatch ToolFilter + IPC + FS + git. (`4H:L312-L319`)
- **P4H-E028** [E/TEST]: 4H.1 acceptance: AllowedConfigKey ≥12 Parameter + ≥4 DataTable; score [0,1]; parent_selection signature; config_variants schema match. (`4H:L321-L349`)
- **P4H-E029** [E/TEST]: 4H.3 config archive: max 50 variants; one ACTIVE per PipelineMode; mode-filtered parent; revert restores same-mode previous; overflow archives lowest non-ACTIVE; deceptive-dip ancestors survive prune even low score; ancestor-vs-cap collision keeps ancestor under cap. (`4H:L380-L433`)
- **P4H-E030** [E/FLOW]: 4H.3B deprecated_patterns: 3 catches → auto-propose PROPOSED scoped; FPR over 10 pipelines; <0.05 PROMOTE else REJECT. FailureTaxonomy: 3 misses → category+regex+fix template from successful resolution. Lazy-code: 3 auditor findings → GREP_PATTERN. Source-tier: domain 5+ as T3 with verified_facts/total>0.8 → propose T2. Scope: cargo rule not eval on TypeScript. (`4H:L442-L522`)
- **P4H-E031** [E/LIMIT]: Layer1A path does NOT trigger staged 10/50/200. Proposals use AllowedConfigKey DataTable variants only. (`4H:L517-L522`)
- **P4H-E032** [E/FLOW]: 4H.4 log-driven proposals read SSB full trace + reasoning_traces; must cite ssb_evidence>=1 and reasoning WHERE failed; target AllowedConfigKey; heuristics fallback only; DataTable→1A Parameter→staged. (`4H:L537-L581`)
- **P4H-E033** [E/TEST]: 4H.5 staged Class B: Stage1 10 discard on fail; Stage2 50 includes trigger_task; Stage3 200 if >40% on 60-subset AND top2 archive + 3 SWE rotation; PASS_TO_FAIL immediate reject; mode-aware tasks; Stage1 fail not archived; Stage2/3 fail stay for branching; Class A never enters. (`4H:L584-L652`)
- **P4H-E034** [E/SURFACE]: Trigger task surface: .elai_cc/benchmarks/trigger_task.toml with input, expected_output_hash sha256, metric_key — project-specific canary against objective hacking. (`4H:L589-L598`)
- **P4H-E035** [E/SURFACE]: Stage3 promotion gate requires better trigger task AND no SWE regression AND no auditor safety regressions AND no evidence-gate definition mods → ACTIVE demote previous ARCHIVED. (`4H:L612-L615`)
- **P4H-E036** [E/SEC]: 4H.6 four independent enforcement layers; generic error no path leak; cannot read lock list; locked keys rejected at dispatch; auto-revert >10% degrade over 3 pipelines; Layer3 human_approved enforced; cannot extend AllowedConfigKey; FPR threshold locked; objective-hacking trigger task. (`4H:L655-L723`)
- **P4H-E037** [E/OBS]: 4H.7 MSI pure arithmetic no LLM; IMPROVING MSI>0.3 frequency 1/3 pipelines; STABILIZING -0.1<MSI<=0.3 frequency 1/5; DIVERGING MSI<=-0.1 pause that mode + user alert + restore highest-score; per-mode independent; stored self_mod_history.db. (`4H:L734-L801`)
- **P4H-E038** [E/SURFACE]: DIVERGING MSI alerts user and pauses self-mod for that PipelineMode only (never silently continues). (`4H:L746-L746, L775-L779`)
- **P4H-E039** [E/OBS]: 4H.7B OSC: stagnation 3 consecutive sim>=0.95 → ConvergenceStagnant SSB + 2x exploration_bonus next 3 selections (does not block promotion). Oscillating period-2 full sim(N,N-2) AND sim(N-1,N-3) both >=0.95 needs ≥4 variants → ConfigOscillation SSB + freeze differing keys 5 pipelines. (`4H:L804-L848`)
- **P4H-E040** [E/OBS]: 4H.7C REG: cross-pipeline AC regression blocks 4H.5 promotion even if score up; 3 consecutive regressed pipelines on active → 4H.6 auto-revert; persistent never-passed not regression; per-mode. (`4H:L897-L970`)
- **P4H-E041** [E/OBS]: 4H.7D EVO: if evolved_count==0 AND latest_sim>=0.95 Withhold → EvolutionWithheld SSB and FORCE MSI STABILIZING (not IMPROVING) regardless of score noise; per-mode. (`4H:L973-L1050`)
- **P4H-E042** [E/FLOW]: 4H.8 Layer1 security/safety/architecture rules NEVER auto-adjusted. Layer1A four growth paths + ProjectScope + locked FPR. Layer2 log-driven + archive + staged + sandbox + rotation + 4-layer read-lock + AllowedConfigKey routing. Layer3 stubs only. (`4H:L1053-L1090`)
- **P4H-E043** [E/FLOW]: Self-referential alignment loop: pipeline→SSB→self-mod reads→propose→1A/2 validate→archive→next pipeline. (`4H:L1092-L1097`)
- **P4H-E044** [E/SURFACE]: Anti-staleness wiring: 4H triggers nightly elai dep-audit CLI; outdated deps → stale_api_catches → Layer1A deprecated_patterns → steering vector refresh metrics/compute_steering_vector.py. (`4H:L1099-L1106`)
- **P4H-E045** [E/FLOW]: Reasoning wiring: 3+ unclassified errors → FailureTaxonomy category + fix template from Layers2-6 resolution → future Layer1 deterministic sub-100ms. Token efficiency: truncations → ContextSlotWeight / BabyStepMaxLoc proposals. (`4H:L1108-L1121`)
- **P4H-E046** [E/EVENT]: SSB action_types: convergence_stagnant, config_oscillation, regression_detected, evolution_withheld. (`4H:L1133-L1134`)
- **P4H-E047** [E/DEFER]: Optional cross-transfer validation with different FM; FM-specific variants marked not universal. (`4H:L1136-L1139`)
- **P4H-E048** [E/OBS]: 4H.9 metrics phase_4H.toml: e2e self-mod time; Layer1A proposal/FPR/growth rates; Stage1 elimination >60% target / Stage3 <20% pattern; MSI trajectory; anti-fragility recovery; closed-loop tightness; stale catch trends; Layer1 classification rate. (`4H:L1186-L1238`)
- **P4H-E049** [E/DEFER]: EVALUATE SWE-World-style learned reward pre-screen ONLY if MSI shows >50% Stage1 fail (poor proposals); else skip. SWR accuracy numbers do NOT transfer; real Stage1 still runs after screen. Risk R53. (`4H:L1241-L1264`)
- **P4H-E050** [E/INT]: RuntimeClient ssb/fsm/clock wiring rule same as 4A exit. (`4H:L1290-L1292`)
- **P4H-E051** [E/LIMIT]: 4H specifies no new full dashboard panel or live cockpit UI; operator surfaces are alerts, CLI dep-audit, TOML metrics, and SQLite history only. (`4H:L1186-L1230`)
- **P4H-A001** [A/CFG]: max 50 archive cap is OUR DESIGN not DGM; resource-driven. (`4H:L437-L437`)
- **P4H-A002** [A/CFG]: MSI thresholds 0.3/-0.1 and 3 phases are OUR DESIGN vs DarwinNet 2 phases. (`4H:L796-L797`)
- **P4H-A003** [A/CFG]: Source-tier auto-promote verified_facts/total >0.8 and domain hit count 5+ are specified but broader tier-mapping rules remain design-local. (`4H:L483-L488`)
- **P4H-I001** [I/SURFACE]: self_mod_history.db and phase_4H.toml are operator-visible measurement/history surfaces for later cockpit consumption. (`4H:L751-L751, L1220-L1221`)
- **P4H-C001** [C/LIMIT]: DGM paper modifies agent code; 4H deliberately refuses code mutation — safety divergence must not be collapsed to "full DGM". (`4H:L36-L41`)
- **P4H-E052** [E/CFG]: Default score weights in elai_cc.toml [self_mod.score_weights]: first_try_rate_weight=0.3, coverage_weight=0.2, efficiency_weight=0.2, intervention_weight=0.15, findings_weight=0.15. (`4H:L134-L139`)
- **P4H-E053** [E/CFG]: Parent selection parameters lambda=10 and alpha_0=0.5 per DGM section 2.3; exploration h_i=1/(1+children_count). (`4H:L167-L172`)
- **P4H-E054** [E/CFG]: MSI exponential weights with N=10 lambda=0.3 listed as [0.07,0.08,0.10,0.13,0.17,0.22,0.29,0.37,0.48,0.61] then normalized. (`4H:L740-L742`)
- **P4H-E055** [E/STATE]: Only one variant status=ACTIVE per PipelineMode at a time; revert promotes previous highest-score ARCHIVED same-mode variant. (`4H:L391-L400`)
- **P4H-E056** [E/FLOW]: Archive overflow when count>50 moves lowest-score non-ACTIVE to cold storage; ancestor-of-active and children_count>0 protected from deceptive-dip prune. (`4H:L401-L425`)
- **P4H-E057** [E/TEST]: 4H.3B scope enforcement: cargo-scoped deprecated_patterns not evaluated on TypeScript pipelines; unscoped rules evaluate on all. (`4H:L490-L495`)
- **P4H-E058** [E/TEST]: 4H.3B FailureTaxonomy proposed pattern must match ≥3 distinct historical error instances, not 3 occurrences of one pipeline error. (`4H:L510-L513`)
- **P4H-E059** [E/FLOW]: Stage1 fail → variant discarded entirely (not archived). Stage2 fail → archived with Stage2 score. Stage3 fail → archived near-miss higher parent priority than Stage2 fails. (`4H:L616-L621`)
- **P4H-E060** [E/TEST]: Stage3 entry requires >40% success on 60-task subset AND top two performers in archive (DGM section 2.5). (`4H:L602-L604, L643`)
- **P4H-E061** [E/SEC]: Self-mod cannot write metrics code (objective hacking defense); trigger task known-answer catches metric gaming even if score improves. (`4H:L678-L684, L705-L708`)
- **P4H-E062** [E/SEC]: Layer1A cannot target Parameter keys; FPR threshold is locked and proposing change to it is rejected. (`4H:L685-L690, L714-L715`)
- **P4H-E063** [E/OBS]: Stagnant OSC does NOT block promotion (valid stop state) but multiplies exploration_bonus by 2.0 for next 3 parent_selection rounds in that mode. (`4H:L842-L846`)
- **P4H-E064** [E/OBS]: Oscillating freezes keys that differ between the two states into AllowedConfigKey runtime exclusion list for next 5 pipelines. (`4H:L847-L849`)
- **P4H-E065** [E/EVENT]: SSB rows ConvergenceStagnant / ConfigOscillation emitted on OSC verdicts; EvolutionWithheld on EVO Withhold. (`4H:L843-L848, L1007-L1008, L1133-L1134`)
- **P4H-E066** [E/FLOW]: REG used as additional gate at 4H.5 Stage3→ACTIVE promotion and every 3 pipelines on active variant for auto-revert. (`4H:L923-L927, L1127-L1130`)
- **P4H-E067** [E/OBS]: 4H.9 targets: Stage1 elimination rate >60% pattern, Stage3 reach <20% pattern (DGM cost pattern, measured as baseline not hard fail). (`4H:L1199-L1201, L1227`)
- **P4H-E068** [E/FAIL]: StructuralChange.apply without human_approved=true returns Err or panics; Layer3 is enforced not advisory. (`4H:L674-L675, L709-L711`)
- **P4H-E069** [E/SURFACE]: CLI surface: `elai dep-audit` nightly anti-staleness trigger invoked by 4H loop. (`4H:L1100-L1100, L1144`)
- **P4H-E070** [E/DATA]: self_mod_history.db stores MSI history and RegressionReport persistence. (`4H:L751, L1129-L1130`)
- **P4H-E071** [E/TEST]: 4H.8 adversarial: every proposal.ssb_evidence must reference logs from last COMPLETE pipeline with completed_at < proposal.created_at. (`4H:L1148-L1152`)
- **P4H-E072** [E/LIMIT]: Security/safety/architecture rules NEVER auto-adjusted by Layer1. (`4H:L1058-L1059`)
- **P4H-E073** [E/DEFER]: Layer3 structural execution deferred until system maturity proves need; only stubs+gate in this phase. (`4H:L1088-L1090`)

### pre-phase-0: Windows platform fixes

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/pre-phase-0.txt`; atoms in `PHASE-007 / DEEP-PHASE-007.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-019 — Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness.; PHASE-007/IX-020 — Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml.

**Behaviors**

- **PP0-E001** [E/STATUS]: PRE-PHASE 0 makes claw-code-rust-port compile and pass all tests on Windows 11 with zero compile errors, zero test failures, zero warnings, and no feature changes. (`pre-phase-0:L1-L10`)
- **PP0-E002** [E/DEP]: Prerequisite: claw-code-rust-port cloned into `.elai_cc/`. No pre-spec files; pure platform work on the upstream fork. (`pre-phase-0:L2-L13`)
- **PP0-E003** [E/LIMIT]: Scope forbids behavioral code-path changes; Unix-only tests must be `cfg`-gated, not deleted, and still run on Linux CI. (`pre-phase-0:L10-L17, L167-L171`)
- **PP0-E004** [E/FAIL]: R-pre1: inherently Unix-only shell tests; mitigation is cfg-gate. R-pre2: path-separator fixes must use `MAIN_SEPARATOR`/`Path` methods, never hardcoded slash direction. (`pre-phase-0:L15-L19`)
- **PP0-E005** [E/EVIDENCE]: Baseline 2026-04-06: cargo check 40.15s / 0 errors / 2 warnings; cargo test 10 compile errors (2 root causes) + 8 runtime failures (4 root causes); 69 `.rs` files; 61,317 LOC; 684 test annotations. (`pre-phase-0:L22-L25, L180-L189`)
- **PP0-E006** [E/CAP]: P0.1 fixes 10 compile errors in three files by cfg-gating `std::os::unix::fs::PermissionsExt` and all `set_mode(0o755)` sites. (`pre-phase-0:L30-L50`)
- **PP0-E007** [E/DATA]: P0.1 files: `mcp_stdio.rs` (use L1413; set_mode L1456/1502/1636/1761/2680), `mcp_tool_bridge.rs` (use L317; set_mode L434), `mock_parity_harness.rs` (use L4; set_mode L425). (`pre-phase-0:L36-L46`)
- **PP0-E008** [E/CFG]: Windows pattern: `#[cfg(unix)]` chmod block; `#[cfg(windows)]` no-op because Windows has no file mode bits and `.cmd`/`.ps1` are executable by default. (`pre-phase-0:L39-L50`)
- **PP0-E009** [E/TEST]: P0.1 gates: cargo test --no-run 0 errors; all `use std::os::unix` imports cfg-gated; acceptance is 0 compile errors across crates. (`pre-phase-0:L51-L61`)
- **PP0-E010** [E/CAP]: P0.2 fixes 4 plugin-hook test failures in `plugins/src/hooks.rs` by cfg-gating Unix `.sh` tests and adding Windows `.cmd`/PowerShell equivalents. (`pre-phase-0:L63-L87`)
- **PP0-E011** [E/SURFACE]: Failing plugin tests: `collects_and_runs_hooks_from_enabled_plugins` (L395, pre.sh), `pre_tool_use_denies_when_plugin_hook_exits_two` (L456, printf/exit 2), `propagates_plugin_hook_failures` (L473, printf/exit 1). (`pre-phase-0:L72-L81`)
- **PP0-E012** [E/FLOW]: `write_hook_plugin` (L362): Unix keeps `.sh`; Windows writes `.cmd` with `@echo off` + echo. `HookRunner::run_hook_command` (~L326) must use cmd.exe on Windows if it hardcodes `/bin/sh`. (`pre-phase-0:L82-L87`)
- **PP0-E013** [E/TEST]: P0.2 acceptance: `cargo test -p plugins --lib` 0 failures / "33 passed; 0 failed". (`pre-phase-0:L88-L97`)
- **PP0-E014** [E/CAP]: P0.3 fixes 4 tools-crate failures in `tools/src/lib.rs` test module. (`pre-phase-0:L99-L118`)
- **PP0-E015** [E/TEST]: `glob_and_grep_tools_cover_success_and_errors` (L6672): path ends_with must tolerate Windows backslash via Path::ends_with or separator normalize. (`pre-phase-0:L106-L108`)
- **PP0-E016** [E/TEST]: `powershell_runs_via_stub_shell` (L7069): cfg-gate Unix chmod; Windows scripts executable by default. (`pre-phase-0:L109-L111`)
- **PP0-E017** [E/TEST]: `repl_executes_python_code` (L7007): exit 9009 means `python3` missing; use `python` on Windows, `python3` on Unix. (`pre-phase-0:L112-L114`)
- **PP0-E018** [E/TEST]: `skill_loads_local_skill_prompt` (L5745): PoisonError on env lock; fix with dedicated mutex, catch_unwind, or `#[serial]` isolation. (`pre-phase-0:L115-L118`)
- **PP0-E019** [E/TEST]: P0.3 acceptance: `cargo test -p tools --lib` 0 failures / "61 passed; 0 failed"; adversarial asks 10 consecutive clean runs. (`pre-phase-0:L119-L128`)
- **PP0-E020** [E/CAP]: P0.4 removes 2 warnings: unused `Path` import in `plugins/hooks.rs` L2; unused `mut` on `command_builder` in `runtime/hooks.rs` L636. (`pre-phase-0:L130-L144`)
- **PP0-E021** [E/OBS]: P0.5 records clean baseline to `.elai_cc/metrics/pre_phase_0.toml` fields: compile_time, test_total, test_pass, test_fail, loc_total, warnings. (`pre-phase-0:L146-L157, L173-L177`)
- **PP0-E022** [E/TEST]: Exit criteria: cargo check 0 errors/warnings on Windows; cargo test 0 failures; all cfg-gated tests documented with Linux-only flag. (`pre-phase-0:L161-L171`)
- **PP0-E023** [E/OBS]: Measurements: compile time, test counts (total/passed/failed/cfg-gated-out), warning count 0, files modified list. (`pre-phase-0:L173-L177`)
- **PP0-E024** [E/EVIDENCE]: Grounding table verifies 10 errors, 7 set_mode sites, 3 unix imports, 4+4 failures, 2 warnings, exit 9009, PoisonError, 61,317 LOC/69 files/684 tests. (`pre-phase-0:L180-L189`)
- **PP0-E025** [E/INT]: WIRING_CONTRACT: produces `hooks.rs::shell_command`; reachable from `hooks.rs::run_command`; observable via `elai.exe archive verify --json` expecting SSB `archive_verify_result`. (`pre-phase-0:L192-L200`)
- **PP0-I001** [I/OBS]: Archive-verify observable effect implies pre-phase-0 shell fix is production-reachable through CLI archive path, not only unit tests. (`pre-phase-0:L198-L200`)
- **PP0-A001** [A/TEST]: Exact list of which tests become cfg-gated-out on Windows is required at exit but not pre-enumerated beyond the three named plugin and four tools cases. (`pre-phase-0:L159-L164, L171-L171`)
- **PP0-A002** [A/CFG]: Whether `HookRunner` already hardcodes `/bin/sh` is left as an audit question at implement time. (`pre-phase-0:L85-L87, L92-L93`)

### phase-1I: resilience layer

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-1I.txt`; atoms in `PHASE-007 / DEEP-PHASE-007.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-002 — Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path.; PHASE-007/IX-004 — Phase 1I SearchMode cascade and BM25_ONLY degraded mode compose with 1B/1C isolation verified under degradation.; PHASE-007/IX-005 — Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate.; PHASE-007/IX-018 — Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer.

**Behaviors**

- **P1I-E001** [E/STATUS]: PHASE 1I is Resilience Layer: circuit breaker, degradation, startup, tracing. Prerequisites: 1E complete and 1G complete. (`phase-1I:L1-L2`)
- **P1I-E002** [E/DEP]: Blocking note: implementation-guide dep graph L141 shows only 1E→1I; must add 1G→1I before implementation or 1I.5 fails. ACTION: update guide L141. (`phase-1I:L3-L9`)
- **P1I-E003** [E/CAP]: Goal: survive individual component failures with circuit breakers, degradation cascade, startup readiness, query validation, cross-encoder concurrency, pipeline tracing, index versioning. (`phase-1I:L13-L16`)
- **P1I-E004** [E/DEP]: Pre-read: knowledge.txt §§5-8, 10-12; architecture-overview.txt §4 readiness_check/ReadinessStatus. (`phase-1I:L18-L23`)
- **P1I-E005** [E/CAP]: 1I.1 `EmbeddingCircuitBreaker` states CLOSED, OPEN, HALF_OPEN; independent instance per ONNX model (GraphCodeBERT, Granite, cross-encoder, SPLADE). (`phase-1I:L28-L36`)
- **P1I-E006** [E/CFG]: Breaker config: failure_threshold=3 (transient), failure_window=60s, half_open_timeout_base=30s; all time via ClockProvider. (`phase-1I:L37-L38`)
- **P1I-E007** [E/FAIL]: Failure classification via error-type enum (not string match): TRANSIENT (timeout/temp OOM) counts to 3; PERMANENT (segfault/missing model/invalid ONNX) opens after 1. (`phase-1I:L39-L42, L66-L67`)
- **P1I-E008** [E/RETRY]: Exponential backoff OPEN timeouts: 30s, 60s, 120s cap; deterministic jitter from hash(model_id) prevents thundering herd. (`phase-1I:L43-L46, L71-L73`)
- **P1I-E009** [E/STATE]: Breaker state is NOT checkpointed; resets CLOSED on restart. (`phase-1I:L47-L47, L79-L79`)
- **P1I-E010** [E/FLOW]: OPEN skips model and returns fallback immediately. HALF_OPEN allows limited probes; needs 2 consecutive successes for CLOSED; any failure → OPEN with reset timer and increased backoff. (`phase-1I:L48-L51, L69-L70`)
- **P1I-E011** [E/RECOVERY]: HALF_OPEN probes create a fresh ONNX InferenceSession (never reuse potentially corrupted session; cites ORT issues #20043/#20354). (`phase-1I:L52-L53, L74-L74`)
- **P1I-E012** [E/TEST]: 1I.1 acceptance includes independent per-model breakers, 3-in-60s open, permanent-1 open, spread failures stay CLOSED, 2-success HALF_OPEN, jitter distinctness, fresh session. (`phase-1I:L65-L75`)
- **P1I-E013** [E/STATE]: 1I.2 `SearchMode` enum: HYBRID, SPARSE_ONLY, BM25_ONLY; any layer failure drops to next mode. (`phase-1I:L87-L94`)
- **P1I-E014** [E/FLOW]: Fallback chain: GraphCodeBERT fail→TEXT-vector-only (Granite); Granite fail→CODE-vector-only; both→SPARSE_ONLY (BM25+SPLADE); SPLADE fail→BM25_ONLY; cross-encoder fail→skip rerank use vector scores. (`phase-1I:L95-L100`)
- **P1I-E015** [E/RECOVERY]: Mode auto-restores when circuit-breaker HALF_OPEN probe succeeds and cascade re-evaluates; polling driven by half_open_timeout (30s), not a separate timer. (`phase-1I:L101-L104, L117-L117`)
- **P1I-E016** [E/EVENT]: Every mode change logs SSB action_type=`knowledge_degradation`. (`phase-1I:L105-L105, L118-L118`)
- **P1I-E017** [E/FAIL]: Never total outage while BM25 is up: BM25_ONLY always reachable (tantivy has no ONNX dependency). Kill embedding → BM25 results within 100ms of detection. (`phase-1I:L116-L119`)
- **P1I-E018** [E/CFG]: If SPLADE disabled (conditional 1H), SPARSE_ONLY collapses to BM25_ONLY and cascade skips straight. (`phase-1I:L114-L114`)
- **P1I-E019** [E/CAP]: 1I.3 `QueryValidator` runs before any embedding call. (`phase-1I:L127-L131`)
- **P1I-E020** [E/FAIL]: Reject empty (zero tokens after whitespace split), >10,000 chars, null bytes, control characters; accept newlines; latency <1ms. (`phase-1I:L132-L135, L144-L151`)
- **P1I-E021** [E/FLOW]: 1I.4 startup sequence: load ONNX models → open SQLite (DKG, vector cache, BM25) → canary embed("fn main()")+BM25 search("main")+rerank within 5s → drift check → report ReadinessStatus. (`phase-1I:L159-L176`)
- **P1I-E022** [E/STATE]: `ReadinessStatus`: READY (all canary pass); DEGRADED (ONNX failed, BM25 ok → BM25_ONLY invariant); UNAVAILABLE (BM25 also failed → total outage). (`phase-1I:L177-L181, L204-L205`)
- **P1I-E023** [E/RETRY]: Startup: after 3 retries enter DEGRADED BM25_ONLY + WARNING; if BM25 fails enter UNAVAILABLE + ERROR. (`phase-1I:L182-L183, L198-L199`)
- **P1I-E024** [E/INT]: Orchestrator FSM: IDLE→PLANNING blocked until READY or DEGRADED; UNAVAILABLE blocks all FSM transitions. (`phase-1I:L184-L185, L200-L200`)
- **P1I-E025** [E/OBS]: Canary drift: cosine_sim(embed("fn main()"), reference) < 0.85 → DEGRADED. Reference stored in `index_version.toml` as `canary_reference_vec`; first startup generates and persists reference after success. (`phase-1I:L169-L173, L192-L203`)
- **P1I-E026** [E/CONC]: 1I.5: `tokio::Semaphore(1)` around cross-encoder ONNX session; bounded channel capacity 4; max 5 in-flight; 6th returns unranked results immediately. (`phase-1I:L215-L223, L233-L236`)
- **P1I-E027** [E/CAP]: Batching: if multiple requests queued, merge candidate sets and rerank once; each caller receives only its original candidates (reranked). (`phase-1I:L224-L224, L237-L238`)
- **P1I-E028** [E/DATA]: 1I.6 `QueryTrace` fields: query_id UUID, stages Vec StageTrace, total_latency_ms, mode SearchMode, result_count. (`phase-1I:L247-L253`)
- **P1I-E029** [E/DATA]: `StageTrace` fields: stage, latency_ms, input_count, output_count, error Option, circuit_breaker_state Option. (`phase-1I:L254-L254`)
- **P1I-E030** [E/OBS]: Nine stages: bm25, splade, code_vector, text_vector, pagerank, catrag, arag, rerank, token_fit. Emit via tracing at DEBUG; NOT per-query in SSB. (`phase-1I:L255-L258, L268-L271`)
- **P1I-E031** [E/OBS]: Aggregated per-session p50/p99 per stage stored in SSB at session end only. (`phase-1I:L257-L258`)
- **P1I-E032** [E/DATA]: 1I.7 `index_version.toml` at `.elai_cc/projects/{id}/index_version.toml` schema: version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format, created_at. (`phase-1I:L281-L287`)
- **P1I-E033** [E/FLOW]: On upgrade compare versions → full rebuild or BM25-only rebuild or reuse; archive old index to `index_backup_{version}/` before rebuild. (`phase-1I:L288-L289, L303-L303`)
- **P1I-E034** [E/SURFACE]: User notified before rebuild (not silent). (`phase-1I:L290-L290, L302-L302`)
- **P1I-E035** [E/FAIL]: Missing version file creates default version=1; corrupted TOML → full rebuild + warning; archive failure aborts rebuild. (`phase-1I:L296-L301`)
- **P1I-E036** [E/TEST]: VERIFY: knowledge crate compiles; no 1G regressions; project isolation holds under BM25_ONLY; vector cache invalidation holds under SPARSE_ONLY/BM25_ONLY. Isolation owned by 1A/1B/1C. (`phase-1I:L311-L320`)
- **P1I-E037** [E/OBS]: MEASURE targets: degradation latency <100ms; canary <5s; breaker recovery <35s typical; semaphore p99 queue wait; QueryTrace overhead <1ms; index version check <10ms. (`phase-1I:L322-L328`)
- **P1I-E038** [E/TEST]: EXIT: full query pipeline on elai_cc codebase; survives ONNX failure; breaker 3 states; cascade to BM25_ONLY auto-recovers; readiness gates FSM; semaphore prevents concurrent ONNX corruption; QueryTrace every query; index migration preserves backup. (`phase-1I:L330-L338`)
- **P1I-E039** [E/OBS]: Phase 1 overall measurements: index build, query latency per layer, token savings, dual-vector accuracy, reranker precision, SPLADE recall delta, degradation correctness, startup readiness latency. (`phase-1I:L340-L343`)
- **P1I-I001** [I/EVENT]: knowledge_degradation event is the operator-visible degradation signal; dashboard consumers are not specified in this phase file. (`phase-1I:L105-L105, L118-L118`)
- **P1I-A001** [A/CFG]: Exact permanent-vs-transient error enum variants beyond the listed examples are not exhaustively named. (`phase-1I:L39-L42`)
- **P1I-A002** [A/CONC]: "Limited probe calls" in HALF_OPEN does not specify concurrency/permit count beyond "allow limited probes". (`phase-1I:L49-L51`)
- **P1I-A003** [A/OBS]: Session-end SSB aggregation schema for p50/p99 stage stats is not named as a payload type. (`phase-1I:L257-L258`)
- **P1I-C001** [C/DEP]: Source blocks on missing 1G→1I edge in implementation-guide while prerequisites claim both 1E and 1G; guide is incomplete until ACTION is done. (`phase-1I:L1-L9`)

### phase-2D: CSQ and inline feasibility

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-2D.txt`; atoms in `PHASE-007 / DEEP-PHASE-007.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-006 — Phase 2D Blocked outcome signals ReplanRequest(MAJOR) for orchestrator 3A+ human-approval EXECUTING→PLANNING.; PHASE-007/IX-007 — Phase 2D CSQ depends on 2C AgentGrcConfig.csq_required and per-role CSQ mandates.; PHASE-007/IX-008 — Phase 2D inline feasibility SourceMatch is temporary; 4F adds WebSearch/Grep/ApiRegistryCheck variants and deeper contradiction analysis.; PHASE-007/IX-020 — Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml.

**Behaviors**

- **P2D-E001** [E/STATUS]: PHASE 2D: CSQ and inline feasibility. Prerequisite: Phase 2C complete. Completes reasoning engine 2A–2D. (`phase-2D:L1-L9, L409-L410`)
- **P2D-E002** [E/CAP]: Goal: CSQ on functions touching files/data/network/auth; inline feasibility during EXECUTING (max 2 GRC calls). (`phase-2D:L6-L8`)
- **P2D-E003** [E/DEP]: Pre-read: reasoning.txt §§6,7,8,10[2D],11[2D]; orchestrator.txt §8; architecture-overview §4. (`phase-2D:L12-L19`)
- **P2D-E004** [E/FLOW]: D1: CSQ trigger via function signature/pattern match on Path/File/TcpStream/Money/AuthToken or std::fs/std::net; must work WITHOUT DKG dependency (reasoning depends on core+providers+tools). Orchestrator injects sensitive signatures at call time. (`phase-2D:L24-L38`)
- **P2D-E005** [E/STATE]: D2 severity: CRITICAL (data_loss, security_breach, financial_error, auth_bypass) guard MANDATORY; WARNING (incorrect_output, performance_degradation, stale_cache) RECOMMENDED. Critical without guard = CSQ failure blocks AUDITING; warning without guard is logged only. (`phase-2D:L40-L50, L401-L403`)
- **P2D-E006** [E/CFG]: D3: Inline feasibility uses dedicated `InlineFeasibilityState` counter separate from per-topic GRC budget (cross-topic scope). Methods: web_search, grep, api_registry_check (spec); receiver is &mut self. (`phase-2D:L52-L65`)
- **P2D-E007** [E/FLOW]: D4 outcomes: PASS proceed; FAIL use original plan approach; BLOCKED emit ReplanRequest(MAJOR) → EXECUTING→PLANNING requires human approval in AUTONOMOUS and INTERACTIVE. (`phase-2D:L67-L75, L404-L405`)
- **P2D-E008** [E/DATA]: Types: CsqCheck {assumption, consequence, severity, guard_added}; CsqSeverity {Critical, Warning}; CsqResult {function_id, checks} with computed `passed()` (no stored field). (`phase-2D:L77-L84, L112-L120`)
- **P2D-E009** [E/DATA]: FeasibilityMethod enum {SourceMatch} now; future WebSearch/Grep/ApiRegistryCheck in phase 4F. InlineFeasibilityResult {claim, method, outcome}; FeasibilityOutcome {Pass, Fail, Blocked}. (`phase-2D:L85-L91, L127-L135`)
- **P2D-E010** [E/STATE]: InlineFeasibilityState {checks_used, max_checks=2 hardcoded}; is_exhausted, record_check (saturating_add), reset (orchestrator between phases). Exhausted treats new claims as Fail / BudgetExhausted. (`phase-2D:L92-L95, L121-L126, L163-L165`)
- **P2D-E011** [E/CAP]: GrcEngine methods: `generate_csq(function_id, sensitive_ops)`; `check_inline_feasibility(claim, sources)` mut self. Stubs todo!() until 2D.4. BasicGrcEngine holds InlineFeasibilityState. (`phase-2D:L136-L144`)
- **P2D-E012** [E/LIMIT]: 2D.2: no new reasoning deps beyond core, providers, tools, thiserror, serde. (`phase-2D:L177-L190`)
- **P2D-E013** [E/TEST]: 2D.3 TDD file `reasoning/tests/csq_tests.rs` with ≥10 tests T1–T10 all FAIL against stubs; zero timing-dependent tests. (`phase-2D:L192-L274`)
- **P2D-E014** [E/TEST]: T1 file-function CSQ ≥1 Critical; T2 Critical no guard fails; T3 Critical with guard passes; T4 Warning no guard still passes; T5 empty sensitive_ops empty checks. (`phase-2D:L201-L223`)
- **P2D-E015** [E/TEST]: T6 known claim Pass; T7 unknown Fail; T8 max two calls then BudgetExhausted; T9 CONTRADICTS_PLAN: prefix → Blocked (ReplanRequest emission is orchestrator 3A+); T10 reset after exhaust. (`phase-2D:L225-L257`)
- **P2D-E016** [E/FLOW]: I1 generate_csq: empty ops → empty checks; else per op Critical if case-insensitive match patterns (fs::, file, write, delete, remove, net::, tcpstream, money, payment, auth, token, credential, password) else Warning; guard_added=false. (`phase-2D:L284-L304`)
- **P2D-E017** [E/FLOW]: I2 check_inline_feasibility: exhaust → Err BudgetExhausted topic=inline_feasibility; source substring/exact match → Pass; not found → Fail; CONTRADICTS_PLAN: prefix → Blocked (full semantic deferred 4F). Then record_check. (`phase-2D:L306-L323`)
- **P2D-E018** [E/DET]: CsqResult.passed computed: all Critical have guard_added. Adversarial: case-insensitive patterns; meaningful multi-word claim match not single common words; different ops not Blocked. (`phase-2D:L325-L353`)
- **P2D-E019** [E/OBS]: 2D.5 metrics in `.elai_cc/metrics/phase_2D.toml`: csq_overhead_us <1000, inline_check_overhead_us <2000, false_positive_rate == 0.0; update architecture-overview GrcEngine listing. (`phase-2D:L360-L397`)
- **P2D-E020** [E/EVIDENCE]: Grounding: CSQ/guard/inline max-2/PASS-FAIL-BLOCKED/per-agent CSQ config from reasoning.txt; severity classification VERIFIED from dangerous-consequence distinction; detection via patterns VERIFIED vs ast-grep; overhead targets UNGROUNDED; CONTRADICTS_PLAN prefix UNGROUNDED placeholder for 4F. (`phase-2D:L413-L428`)
- **P2D-E021** [E/DEP]: Per-agent CSQ config: executor=mandatory, auditor=mandatory, compiler=none (reasoning.txt §7). (`phase-2D:L421-L421`)
- **P2D-E022** [E/DEFER]: Actual ReplanRequest emission is orchestrator responsibility phase 3A+. Full semantic contradiction detection deferred to 4F FEASIBILITY_PROBE. (`phase-2D:L250-L252, L318-L321, L428-L428`)
- **P2D-E023** [E/LIMIT]: Zero todo!/unimplemented!/unwrap() in CSQ and inline feasibility production code at exit. (`phase-2D:L334-L338, L407-L407`)
- **P2D-I001** [I/SURFACE]: Human approval for MAJOR replan is an interactive gate surface owned by orchestrator, not implemented in reasoning crate in this phase. (`phase-2D:L70-L73, L250-L252`)
- **P2D-A001** [A/FLOW]: Design D3 lists web_search/grep/api_registry_check methods while Tier-1 type only implements SourceMatch; call-path for those methods is unspecified until 4F. (`phase-2D:L58-L58, L85-L87, L132-L132`)
- **P2D-A002** [A/TEST]: T4 writes `result.passed == true` field access while type design forbids a stored field (computed method). (`phase-2D:L83-L84, L216-L218`)
- **P2D-A003** [A/DET]: Substring match "meaningful content (>3 words)" threshold is adversarial guidance, not a named constant/API. (`phase-2D:L345-L347`)
- **P2D-C001** [C/DATA]: D3 method list (web_search/grep/api_registry) conflicts with FeasibilityMethod::{SourceMatch} only + 4F future variants. (`phase-2D:L58-L58, L85-L87, L132-L132`)

### phase-3B: Shared State Board

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 9 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-3B.txt`; atoms in `PHASE-007 / DEEP-PHASE-007.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-002 — Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path.; PHASE-007/IX-005 — Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate.; PHASE-007/IX-009 — Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events.; PHASE-007/IX-010 — Phase 3B WARN-6 write-before-ToolResult contract binds agent tool IPC to durable journal order.; PHASE-007/IX-011 — Phase 3B PhaseCompletePayload fsync-before-unload durability rule composes with agent lifecycle unload paths.; PHASE-007/IX-017 — Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension.; PHASE-007/IX-019 — Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness.

**Behaviors**

- **P3B-E001** [E/STATUS]: PHASE 3B: SSB. Prerequisite: Phase 3A complete (9 states, evidence gates, INTERACTIVE gating). (`phase-3B:L1-L2`)
- **P3B-E002** [E/CAP]: Goal: inter-agent communication via SQLite WAL single-writer; append-only within session; backbone for tool results, findings, claims, checkpoints, phase transitions. (`phase-3B:L6-L12`)
- **P3B-E003** [E/STATE]: One Ssb instance permanently binds one (project_id, session_id); multi-session needs new instance (anti-forgery). (`phase-3B:L14-L16, L491-L491`)
- **P3B-E004** [E/DEP]: Downstream consumers: 1C vector cache invalidation, 3C agents, 3D BSP, 3E context budget. (`phase-3B:L10-L12`)
- **P3B-E005** [E/DEP]: Pre-read: orchestrator.txt §§3,5,6,7A; core.txt ClockProvider L68-82; audit-results BLOCK 1/5/8; phase4 WARN-6; risk R10; architecture-overview SSB refs. (`phase-3B:L18-L29`)
- **P3B-E006** [E/LIMIT]: R10: ~50K inserts/sec claimed; 80K achievable with optimized pragmas; agent rate 1–10/s gives 50x headroom; fallback batch transaction. (`phase-3B:L32-L35, L451-L453`)
- **P3B-E007** [E/FAIL]: R-3B-1: WAL autocheckpoint 1000; manual checkpoint only between supersteps (3D). R-3B-2: disable r2d2 idle_timeout/max_lifetime on read pool. R-3B-3: CHECK changes need user_version migration (drop+recreate session-scoped OK). (`phase-3B:L36-L49`)
- **P3B-E008** [E/CAP]: Deliverables D1 schema/WAL/pool/migration; D2 conflict+append-only+events; D3 typed actions+traces. (`phase-3B:L51-L54`)
- **P3B-E009** [E/DATA]: Schema PRAGMA user_version=1; table ssb(seq PK AUTOINCREMENT, project_id, session_id, agent_id, superstep DEFAULT 0, action_type CHECK 17 values, payload BLOB, created_at INTEGER). (`phase-3B:L56-L74`)
- **P3B-E010** [E/EVENT]: 17 action_types: file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, phase_complete, memory_monitoring. (`phase-3B:L65-L71, L96-L100`)
- **P3B-E011** [E/DATA]: Indexes: project, session, superstep, conflict(project_id,superstep). Junction ssb_files(seq,file_path) with path and seq indexes. (`phase-3B:L75-L85`)
- **P3B-E012** [E/EVIDENCE]: orchestrator.txt §3 synopsis lists only 9 action_types; full 17 from cross-refs is CANONICAL in this schema. (`phase-3B:L87-L94, L492-L492`)
- **P3B-E013** [E/DATA]: Rust SsbActionType enum serde rename_all=snake_case; typed SsbPayload per type validated at write. (`phase-3B:L96-L100`)
- **P3B-E014** [E/CAP]: Conflict SQL: JOIN ssb_files→ssb; filter file_path IN rarray, project_id, session_id != self, superstep; return distinct agent_ids. (`phase-3B:L102-L115, L270-L274`)
- **P3B-E015** [E/CFG]: WAL pragmas order: journal_mode=WAL; synchronous=NORMAL; wal_autocheckpoint=1000; busy_timeout=5000; cache_size=-64000 (64MB); foreign_keys=ON; temp_store=MEMORY. (`phase-3B:L117-L127`)
- **P3B-E016** [E/CFG]: foreign_keys is per-connection; applied to writer and every read-pool checkout via r2d2 CustomizeConnection. (`phase-3B:L129-L132, L157-L158`)
- **P3B-E017** [E/CAP]: 3B.1 module ssb.rs: SsbActionType, SsbEntry (payload Arc<[u8]>), SsbFileEntry, SsbConfig, SsbPragmaApplier, Ssb::new with migration/user_version, read pool size max(num_cpus*2,4) idle/lifetime None, mpsc unbounded writer, broadcast capacity 1024. (`phase-3B:L137-L178`)
- **P3B-E018** [E/INT]: SsbReader queries: by_session, by_file, by_type, by_superstep, query_conflict, last_seq; subscribe(); shutdown(). (`phase-3B:L170-L178`)
- **P3B-E019** [E/FLOW]: 3B.2 write path: BEGIN IMMEDIATE → INSERT ssb → INSERT ssb_files → COMMIT → broadcast; oneshot returns seq; project_id/session_id injected at construction; created_at from ClockProvider never SystemTime. (`phase-3B:L208-L228`)
- **P3B-E020** [E/INT]: WARN-6 IPC sync: SSB write MUST complete (.await seq) before ToolResult dispatch to agent. (`phase-3B:L226-L228, L459-L459`)
- **P3B-E021** [E/RECOVERY]: shutdown drops sender, drains loop, write-after-shutdown Err(SsbShutdown); idempotent; no hung oneshots. (`phase-3B:L229-L234, L258-L259`)
- **P3B-E022** [E/LIMIT]: Append-only: no UPDATE/DELETE methods; Connection not exposed; debug update_hook panics on UPDATE/DELETE. (`phase-3B:L275-L280, L309-L309`)
- **P3B-E023** [E/DATA]: 3B.4 typed payloads: FileWrite/Read, ToolResult, TestOutcome, Finding, Claim, Checkpoint, ModelSwap, PhaseTransition, KnowledgeDegradation, IsolationMetric, PlanUpdate, ReplanRequest, StubReference, ReasoningTrace, PhaseComplete, MemoryMonitoring (17 structs). (`phase-3B:L317-L338`)
- **P3B-E024** [E/EVENT]: PhaseCompletePayload written BEFORE model unload with WAL fsync; crash before → re-execute phase; after → skip. (`phase-3B:L333-L337`)
- **P3B-E025** [E/INT]: SsbTraceable trait in orchestrator/; reasoning/tools never call SSB; orchestrator writes after ThinkStep IPC or in-process GRC callbacks. Serialization msgpack rmp-serde for disk. (`phase-3B:L342-L352, L379-L380`)
- **P3B-E026** [E/FAIL]: Payload type mismatch returns Err not panic; payload >1MB rejected; disk-full propagates error. (`phase-3B:L339-L341, L364-L366, L416-L417`)
- **P3B-E027** [E/EVENT]: Broadcast after commit only; slow subscribers lag/logged; Arc payload O(1) clone; capacity 1024. (`phase-3B:L224-L224, L398-L400, L445-L445`)
- **P3B-E028** [E/TEST]: Unit/property/adversarial/determinism suites: 10K monotonic seq, self-not-conflict, cross-project empty, symmetric conflict, seed+clock byte-identical DB, SQLITE_BUSY storm proves channel necessity. (`phase-3B:L383-L429`)
- **P3B-E029** [E/OBS]: MEASURE: write >10K inserts/s; read p99 <1ms at 10K; conflict <5ms for 100 paths vs 100K; pool 0 timeouts 60s; RSS <50MB at 100K; broadcast overhead <1% with 3 subscribers. (`phase-3B:L431-L438`)
- **P3B-E030** [E/TEST]: EXIT: typed 17 payloads, conflict JOIN+project isolation, append-only, event notification, ClockProvider determinism, user_version migration, graceful drain, all tests pass. (`phase-3B:L440-L449`)
- **P3B-E031** [E/EVIDENCE]: Audit fixes list BLOCK1–5 and WARN1–6 including reasoning_trace/phase_complete/memory_monitoring additions, JOIN conflict, ClockProvider rename, Arc payload, CustomizeConnection, shutdown, user_version, rarray, SsbTraceable placement. (`phase-3B:L477-L492`)
- **P3B-I001** [I/DEP]: rusqlite bundled+array and r2d2_sqlite are required Windows/deps choices for this phase. (`phase-3B:L141-L147, L462-L463`)
- **P3B-A001** [A/DATA]: Step text says "SsbPayload enum (15 variants)" while schema and audit claim 17 action_types/payloads. (`phase-3B:L339-L341, L97-L100, L375-L375`)
- **P3B-A002** [A/CFG]: CURRENT_VERSION constant numeric value beyond initial user_version=1 is not specified for future migrations. (`phase-3B:L57-L57, L161-L163`)
- **P3B-C001** [C/DATA]: Internal 15-vs-17 payload count wording conflicts with the canonical 17-type CHECK list. (`phase-3B:L65-L71, L339-L341`)

### phase-4D: Pipeline Factory L1 / FlashResearch

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4D.txt`; atoms in `PHASE-007 / DEEP-PHASE-007.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-007/IX-001 — Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D.; PHASE-007/IX-009 — Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events.; PHASE-007/IX-012 — Phase 4D Research Agent Tier C isolation reuses Phase 4A process walls and IPC proxy patterns.; PHASE-007/IX-013 — Phase 4D Designer GRC config reuses 2C role specialization pattern (distinct from Executor).; PHASE-007/IX-014 — Phase 4D SystemConstraints sizing defaults feed later L2 sizing/feasibility consumers.; PHASE-007/IX-015 — Phase 4D package API_REGISTRY_CHECK reuses 2B verification method family.; PHASE-007/IX-016 — Phase 4D Interactive present_options is a human-gate surface sibling to 3A InteractiveGate / later dashboard interventions.; PHASE-007/IX-017 — Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension.; PHASE-007/IX-018 — Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer.; PHASE-007/IX-020 — Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml.

**Behaviors**

- **P4D-E001** [E/STATUS]: PHASE 4D Pipeline Factory L1 (designer + FlashResearch). Prerequisite: Phase 4A Tier C isolation. Status IMPROVED 2026-04-06. (`phase-4D:L1-L3`)
- **P4D-E002** [E/CAP]: Goal: Research Agent designs pipelines from tasks via tree-structured GRC-grounded research; produces PipelineSpec + QualitySpec via ResearchReport. Paper arxiv 2510.05145 FlashResearch. (`phase-4D:L7-L10`)
- **P4D-E003** [E/DEP]: Pre-read: pipeline-factory.txt §§1-9; reasoning GRC + IPC verification; external-analysis FlashResearch; flashresearch-interface.md. (`phase-4D:L12-L16`)
- **P4D-E004** [E/CFG]: Research Agent role: cloud preferred or Gemma4; GRC budget 10/1200. (`phase-4D:L27-L27, L119-L119`)
- **P4D-E005** [E/CFG]: Designer role distinct AgentRole::Designer (not Executor); model reuses executor Gemma 4 31B; GRC 8/1200; required_steps Observe/Hypothesize/PlanAction/Verify/Converge; plan-only role_prompt for TOML PipelineSpec. (`phase-4D:L28-L36`)
- **P4D-E006** [E/DATA]: SystemConstraints input: model_max_context, available_for_work, avg_file_tokens, avg_test_tokens. Defaults when index <10 files from elai_cc.toml [planning.sizing] default_avg_file_tokens=150, default_avg_test_tokens=80; else knowledge_engine.index_stats(). (`phase-4D:L37-L45`)
- **P4D-E007** [E/DATA]: ResearchTree {plan, state, root, breadth_factor, depth_limit, max_iterations, phi_min, psi_min, speculative_depth, time_budget}. (`phase-4D:L46-L49`)
- **P4D-E008** [E/DATA]: ResearchNode {question, parent, children, phi, psi, status, info_gain, iterations}; NodeId newtype u64; NodeStatus {Pending, Active, Converged, Pruned}. (`phase-4D:L50-L53`)
- **P4D-E009** [E/DATA]: ResearchPlan {query, complexity, sub_questions}; SharedResearchState {nodes HashMap, next_id}; QueryComplexity {Simple, Moderate, Complex}. (`phase-4D:L54-L56`)
- **P4D-E010** [E/DATA]: Supporting: ResearchTreeSnapshot, ConvergenceStatus {Converged, Partial, Failed}, Alternative, Contradiction, EvidenceLevel {Benchmark(1.0), OfficialDocs(0.8), BlogPost(0.6), Training(0.4)}. (`phase-4D:L57-L63`)
- **P4D-E011** [E/DATA]: QualityMetric {name, target, unit, method, phase, hard_fail}; MediaSpec {output_name, format, resolution, ssim_threshold, vlm_check, output_type}; OutputType {Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom}; PerfSpec. (`phase-4D:L64-L69`)
- **P4D-E012** [E/DATA]: ResearchReport + SubQuestionResult with sources/confidence/phi/psi/depth/alternatives/contradictions; confidence = source_count * avg_quality * agreement_ratio. (`phase-4D:L70-L78`)
- **P4D-E013** [E/DATA]: SourceCitation {url, extracted_quote, quality_tier}; SourceQuality 5-tier Tier1(1.0)…Tier5(0.2); domain lookup no LLM; configurable [research.trusted_domains]. (`phase-4D:L79-L83`)
- **P4D-E014** [E/SEC]: Research Agent Tier C isolated process: network web-search only via IPC proxy; GRC via IPC GroundingRequest; no FS to .elai_cc/ or project source; model via IPC LlmProvider. (`phase-4D:L86-L90, L181-L182`)
- **P4D-E015** [E/CFG]: Adaptive complexity: Simple breadth2/depth3/φ0.8/ψ0.7; Moderate 3/4/0.7/0.6; Complex 4/5/0.6/0.5 (no hardcoded single thresholds). (`phase-4D:L91-L95`)
- **P4D-E016** [E/CFG]: information_gain_threshold range [0.1, 0.9]; below for 3 consecutive iterations → prune node. (`phase-4D:L96-L97, L273-L273`)
- **P4D-E017** [E/CONC]: Parallel root children for independent sub-questions; local models sequential per level; cloud parallel with rate-limit Dynamic Fan-Out; pruned branches free resources immediately. (`phase-4D:L98-L102, L267-L270`)
- **P4D-E018** [E/EVENT]: SSB progress stubs action_type=`research_step` per iteration. (`phase-4D:L85-L85, L183-L183, L271-L271`)
- **P4D-E019** [E/TEST]: 4D.3 ≥14 contract invariants (depth/breadth bounds, acyclic, dual-threshold convergence, deterministic quality, contradictions surfaced, speculative bounds, source tracking); ≥3 integration; ≥3 proptest; all FAIL red phase. (`phase-4D:L158-L204`)
- **P4D-E020** [E/TEST]: 4D.4 ≥8 adversarial: FS escape, direct network, GRC exhaustion, depth/breadth bombs, speculative prune, quality gaming, empty task, ALL-TRAINING HIGH_RISK, contradictions. (`phase-4D:L206-L232`)
- **P4D-E021** [E/FLOW]: 4D.5 per-node loop: FORMULATE → SEARCH (top N default 10) → EXTRACT (500-token sample filter) → REFLECT (phi/psi/contradictions/info_gain) → CONVERGE/EXPAND/ITERATE. (`phase-4D:L239-L247`)
- **P4D-E022** [E/FLOW]: Speculative execution cloud-only: trigger phi>0.4; top speculative_depth children before parent converges; parent direction change cancels children; local speculative_depth forced 0; cost bound speculative_depth*breadth_factor. (`phase-4D:L252-L257`)
- **P4D-E023** [E/STATE]: Dual-threshold convergence: phi≥phi_min AND psi≥psi_min. Node lifecycle Pending→Active→(Converged\ (`Pruned). Acyclic parent-chain check.`)
- **P4D-E024** [E/LIMIT]: L1 PipelineSpec may ONLY reference alternatives from ResearchReport; QualityMetric must have automated check method (unmeasurable → REJECT). (`phase-4D:L264-L266, L326-L327`)
- **P4D-E025** [E/OBS]: 4D.6 metrics phase_4D.toml: grounding_distribution, spec_token_count, depth/breadth utilization, convergence p50/p95, speculative_hit_rate, source_tier_distribution, time_budget_ratio, ssb_event_count, info_gain_trend. Accept: zero UNCERTAIN; time_budget_ratio≤1.1. (`phase-4D:L293-L313`)
- **P4D-E026** [E/DATA]: Required docs: workflow/reasoning/phase-4D-decisions.md and phase-4D-grounding.md before EXECUTING. (`phase-4D:L316-L318`)
- **P4D-E027** [E/INT]: EXIT: RuntimeClient must wire ssb, fsm, clock; no None after construction. INTERACTIVE mode visibility via SSB progress. (`phase-4D:L320-L329`)
- **P4D-E028** [E/FLOW]: Recursive extension: FlashResearch at Idea→Concept→Phase→Step→BabyStep; orchestrator TreeNavigator drives; LLM never controls flow. (`phase-4D:L332-L344`)
- **P4D-E029** [E/STATE]: Level 0 IDEA → ConceptDecomposition with feasibility scores; gate drop concepts under threshold. Level 1 CONCEPT → phases/frameworks/packages. Level 2 PHASE → steps with grounded paths. Level 3 STEP → BabyStepSpec 8-block each <2K tokens. (`phase-4D:L346-L368`)
- **P4D-E030** [E/DATA]: Decomposition types: ConceptDecomposition, ConceptNode, PhaseNode, StepNode, BabyStepSpec (+ grounding/pre_gate/task/tdd/deterministic_check/acceptance/integration/exit). Newtypes ConceptId, BabyStepId. (`phase-4D:L370-L419, L510-L513`)
- **P4D-E031** [E/DATA]: BabyStepGrounding max 500 tokens overall_context_summary; TddSpec write_before_code always true; AcceptanceFailAction {BubbleUp, ResearchAlternative, Replan}; max_retry_alternatives default 3. (`phase-4D:L421-L464`)
- **P4D-E032** [E/DATA]: TechStackDecision + DecisionSource {Research, UserChoice, AutoSelected}; PackageRef pinned verified via API_REGISTRY_CHECK; ResearchCache BTreeMap LRU max_entries + ttl_seconds + ClockProvider cached_at. (`phase-4D:L477-L508`)
- **P4D-E033** [E/FLOW]: research_decompose: Level0 Complex research; Level1 Moderate with cache; Interactive present_options vs Autonomous auto_select by evidence_level; Level2 Simple + verify_packages; Level3 baby steps with compressed grounding. (`phase-4D:L515-L562`)
- **P4D-E034** [E/SURFACE]: INTERACTIVE mode presents alternatives with evidence for user pick; AUTONOMOUS selects highest evidence_level * source_quality. (`phase-4D:L531-L535, L645-L647`)
- **P4D-E035** [E/SEC]: Package verification API_REGISTRY_CHECK is anti-hallucination gate; nonexistent/wrong version retries; LLM cannot invent packages. (`phase-4D:L568-L570`)
- **P4D-E036** [E/TEST]: 4D.7–4D.10: recursive types skeleton, ≥13 FAIL tests (levels, cache, TTL, package verify, grounding <500, Interactive/Autonomous, cross-deps, proptest), implement, measure cache_hit_rate/baby_step metrics into phase_4D.toml. (`phase-4D:L574-L678`)
- **P4D-E037** [E/EVENT]: SSB emit research_decompose_level per level transition. (`phase-4D:L650-L650`)
- **P4D-E038** [E/LIMIT]: Design principles: API retrieval not browser (2–3x cost); ManuSearch deterministic orchestration; A-RAG progressive disclosure ~90% token savings; evidence grows monotonically; hybrid tree preferred; source quality never LLM-scored. (`phase-4D:L703-L714`)
- **P4D-E039** [E/CAP]: 4D.11 new crate web_search; SearXNG Docker start-searxng.sh port 8888; custom settings JSON format result_count=20 no telemetry. (`phase-4D:L717-L730`)
- **P4D-E040** [E/CAP]: SearxClient {base_url default localhost:8888, timeout 10s, max_results 20}; search GET /search format=json; SearchResult {title,url,snippet,engine,score,parsed_domain}; health_check GET / 2s. (`phase-4D:L731-L746`)
- **P4D-E041** [E/CFG]: SearXNG rate limit max 1 req/s configurable; 1 retry exponential backoff on 5xx; lives in orchestrator, IPC-proxied. (`phase-4D:L747-L752`)
- **P4D-E042** [E/CAP]: 4D.12 WebFetcher: timeout 15s, max_redirects 5, UA ELAI-Research/0.1, max_body 2MB, per-domain rate max 2 req/s. (`phase-4D:L782-L789`)
- **P4D-E043** [E/CAP]: ContentExtractor trait + ReadabilityExtractor (readability-rust; fallback strip tags); robots.txt respect RFC9309 with 1h cache; 404 robots = allow; Empty page_type on JS-only. (`phase-4D:L791-L826`)
- **P4D-E044** [E/DEFER]: rs-trafilatura higher quality alternative evaluated in 4D.18 if extraction quality insufficient. (`phase-4D:L828-L831, L1236-L1239`)
- **P4D-E045** [E/DATA]: 4D.13 SourceQualityTable + SourceTier; ≥200 domains; exact domain then subdomain rules (*.gov Tier1, *.edu Tier3); default Tier5; TOML [research.trusted_domains] + [research.trusted_subdomain_rules]. (`phase-4D:L836-L891`)
- **P4D-E046** [E/CAP]: 4D.14 WebContentIndex/WebChunk for prose A-RAG; sentence split [.!?\n]+ min 10 chars; ~500 token chunks; KeywordSearch score count*len; SemanticSearch 3x oversample; ChunkRead with ContextTracker; progressive snippets-first. (`phase-4D:L899-L959`)
- **P4D-E047** [E/CFG]: Embedding strategy: CPU sentence-transformers if no GPU model; sidecar /v1/embed if running; else keyword-only. (`phase-4D:L930-L933`)
- **P4D-E048** [E/CFG]: 4D.15 [research.vram_tiers] six tiers 8gb/12gb/12gb_7b/16gb/24gb/48gb with model, quant, context_ceiling, research_context_budget, chunk_size, breadth/depth, phi/psi, embedding_device, max_sources, max_concurrent_fetches; only 48gb has speculative_depth=2. (`phase-4D:L968-L1057`)
- **P4D-E049** [E/SURFACE]: ResearchTierConfig auto-detect GPU VRAM (nvidia-smi/CUDA) or manual `--vram-tier` flag; detect_vram_tier rounds DOWN; no-GPU falls back 8gb. (`phase-4D:L1059-L1079`)
- **P4D-E050** [E/CAP]: 4D.16 ResearchBudgetManager can_admit; prune_to_budget drops lowest tier then lowest phi, never Tier1; compress_evidence first3+last sentence; token approx 1tok~4bytes overcount 10%. (`phase-4D:L1087-L1140`)
- **P4D-E051** [E/CAP]: 4D.17 WebResearchOrchestrator wires SearxClient, WebFetcher, SourceQualityTable, WebContentIndex, ResearchBudgetManager, ResearchTierConfig. (`phase-4D:L1143-L1155`)
- **P4D-E052** [E/FLOW]: Wired loop: FORMULATE IPC → SEARCH → QUALITY GATE Tier1-3 first → FETCH/EXTRACT parallel ≤max_concurrent → INDEX → A-RAG retrieve → REFLECT IPC → CONVERGE. (`phase-4D:L1156-L1175`)
- **P4D-E053** [E/INT]: IPC messages: SearchRequest, FetchRequest, EmbedRequest, InferenceRequest; agent never direct network. (`phase-4D:L1176-L1182`)
- **P4D-E054** [E/EVENT]: SSB emit web_research_step per node iteration. (`phase-4D:L1183-L1183`)
- **P4D-E055** [E/FAIL]: SearXNG down → partial result + error note not hang; >80% Tier5 → confidence LOW; sidecar missing → clear error; rate limiter max 2 concurrent same domain. (`phase-4D:L1189-L1198`)
- **P4D-E056** [E/OBS]: 4D.18 E2E 10 queries (3 simple/4 moderate/3 complex) metrics phase_4D_web_research.toml; 12GB RTX 3060 3B+7B; accept token_budget_util <1.0; ≥30% Tier1-3 on complex; time <5min simple / <15min complex; extraction ≥80% clean. (`phase-4D:L1212-L1256`)
- **P4D-E057** [E/SURFACE]: SSB research_step / research_decompose_level / web_research_step intended for INTERACTIVE mode operator visibility of research progress. (`phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183`)
- **P4D-I001** [I/SURFACE]: present_options Interactive gate is a user-facing selection surface; concrete panel/command wiring is not named in this phase file. (`phase-4D:L531-L533, L622-L622`)
- **P4D-A001** [A/EVENT]: research_step, research_decompose_level, and web_research_step are not in phase-3B's 17-type CHECK list; registration path is unspecified here. (`phase-4D:L85-L85, L650-L650, L1183-L1183; phase-3B:L65-L71`)
- **P4D-A002** [A/CFG]: Several context_ceiling values marked projected vs MEASURED; measurement obligation before treating as hard SLOs is incomplete. (`phase-4D:L977-L1047`)
- **P4D-A003** [A/SURFACE]: Exact Interactive UI for present_options (CLI, dashboard modal, gate API) is not specified. (`phase-4D:L531-L533, L645-L647`)
- **P4D-A004** [A/DATA]: PipelineSpec/QualitySpec full field schemas deferred to pipeline-factory.txt sec 10 beyond the metrics/media/perf structs listed. (`phase-4D:L64-L69, L178-L180`)
- **P4D-C001** [C/EVENT]: Phase-4D introduces SSB action types (research_step, research_decompose_level, web_research_step) absent from phase-3B canonical 17-type schema without a schema-migration step in either file. (`phase-4D:L85-L85, L650-L650, L1183-L1183; phase-3B:L65-L71`)

### 3C: Agent system (pool, lifecycle, checkpoint, recovery, supervision, anti-spiral)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3C.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-001 — 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection.; PHASE-008/IX-002 — 3C spawn_parallel sequential stub is upgraded to true parallel only by 3D.; PHASE-008/IX-003 — 3C communicate in-process stub becomes IPC in 4A; same AgentHandle abstraction.; PHASE-008/IX-004 — 3C ReplanRequest MAJOR human gate and MINOR PLANNING fast-track compose with 3A FSM and auditor.; PHASE-008/IX-005 — 3C ReasoningTraceEntry and checkpoint events depend on 3B SSB single-writer append path.; PHASE-008/IX-006 — 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400.; PHASE-008/IX-007 — 3C Designer role produces PipelineSpec later consumed by 4D/4E/4F chain.; PHASE-008/IX-014 — 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral.; PHASE-008/IX-025 — Index 2C role budgets are the same numbers 3C adversarial expects at spawn.; PHASE-008/IX-026 — Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase.; PHASE-008/IX-028 — Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts.; PHASE-008/IX-030 — blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A). …

**Behaviors**

- **P3C-E001** [E/STATUS]: Phase 3C goal is agent spawning, lifecycle, checkpointing, crash recovery, supervision tree, and anti-spiral escalation. Agents run in-process (Tier A) in Phase 3. (`3C:L7-L10`)
- **P3C-E002** [E/DEFER]: Process separation (Tier C) is deferred to Phase 4A. (`3C:L10-L10`)
- **P3C-E003** [E/DEP]: Prerequisites: Phase 3B SSB operational with single-writer guarantee. Status label IMPROVED, grounded against orchestrator/reasoning/architecture-overview/worker_boot. (`3C:L1-L3`)
- **P3C-E004** [E/CAP]: Transforms claw-code `WorkerRegistry` (~1069 LOC worker_boot.rs) and `recovery_recipes.rs` (~630 LOC) into architecture-overview AgentPool trait with checkpoint/recovery/supervision from orchestrator §7. (`3C:L12-L14`)
- **P3C-E005** [E/DEP]: Input from 3B: `ssb::write` for checkpoint/reasoning_trace; single-writer, append-only, seq monotonic. (`3C:L31-L32`)
- **P3C-E006** [E/DEP]: Input from 3A: `Fsm::get_state`/`transition`; phase-aware tool loading restricts tools per FSM state. (`3C:L33-L34`)
- **P3C-E007** [E/DEP]: Input from 2C/2A: `AgentGrcConfig` loaded at spawn; `GrcEngine::think` produces `ReasoningTraceEntry` to SSB. (`3C:L35-L38`)
- **P3C-E008** [E/CAP]: Output to 3D: `AgentPool::spawn` returns `AgentHandle`; `spawn_parallel` STUBBED sequential here, true parallel in 3D. (`3C:L41-L43`)
- **P3C-E009** [E/CAP]: Output to 3E: handle exposes `agent_id`, `role`, `active_files` for file-overlap scoring. (`3C:L44-L45`)
- **P3C-E010** [E/FLOW]: Output to 3F: agent tool calls route through orchestrator dispatch and `RuleEngine::evaluate` (no direct tool calls). (`3C:L46-L47`)
- **P3C-E011** [E/CAP]: Output to 4A: `AgentHandle` is IPC-ready; 3C wraps in-process, 4A wraps IPC socket with same interface. (`3C:L48-L49`)
- **P3C-E012** [E/FLOW]: Agents emit `ReplanRequest(evidence, severity)`: max 1 per agent per phase; SSB `action_type='replan_request'`; auditor validates evidence; MINOR fast-track to PLANNING; MAJOR human gate. (`3C:L50-L54`)
- **P3C-E013** [E/DATA]: Mapping: `WorkerStatus`→`AgentStatus` (add Checkpointing+BudgetExceeded; drop TrustRequired); `WorkerRegistry`→`AgentPool` with `HashMap<Uuid, AgentHandle>` via UUID v4 from EntropySource (not raw uuid crate). (`3C:L56-L62`)
- **P3C-E014** [E/DET]: Replace `now_secs()`/SystemTime with `ClockProvider::now_monotonic()` (entropy quarantine). (`3C:L65-L67`)
- **P3C-E015** [E/DATA]: `FailureScenario`/`RecoveryRecipe` map to death classification and `SupervisionPolicy.on_failure()`. (`3C:L68-L69`)
- **P3C-E016** [E/CONC]: R6: on single GPU (12GB), if model+embedding exceed VRAM, serialize agents (never parallel). VRAM check built into spawn. (`3C:L72-L74`)
- **P3C-E017** [E/SEC]: R31: Tier A invisible walls must return "not found" not "denied" for hidden paths. (`3C:L75-L77`)
- **P3C-E018** [E/CAP]: Six independently testable deliverables 3C.1–3C.6: pool/handle/status; lifecycle+death; checkpointing; recovery; supervision; anti-spiral. (`3C:L80-L86`)
- **P3C-E019** [E/STATE]: `AgentStatus`: Spawning, Ready, Running, Checkpointing, Finished, Failed. Drop TrustRequired; rename ReadyForPrompt→Ready; add Checkpointing. (`3C:L100-L104`)
- **P3C-E020** [E/DATA]: `AgentHandle` fields: `agent_id: Uuid`, `role: AgentRole`, `status`, `active_files: Vec<PathBuf>`, `grc_config: AgentGrcConfig`, `scratch_dir: PathBuf`. (`3C:L105-L110`)
- **P3C-E021** [E/CFG]: `grc_config` loaded from `elai_cc.toml [agents.{role}]` at spawn. (`3C:L110-L110`)
- **P3C-E022** [E/DATA]: `AgentRole` (8): Executor, Auditor, PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer. (`3C:L111-L113`)
- **P3C-E023** [E/CAP]: Designer produces PipelineSpec from Research Report; not Executor; distinct role_prompt/required_steps/output; reuses executor model (Gemma 4 31B) with separate AgentGrcConfig. (`3C:L114-L116`)
- **P3C-E024** [E/CAP]: `AgentPool` trait: `spawn(role,task)->AgentHandle`; `spawn_parallel` sequential stub (not unimplemented!); `communicate` direct in-process stub (IPC in 4A). (`3C:L117-L120`)
- **P3C-E025** [E/FAIL]: spawn VRAM check via `ModelPool::available_vram()` (never nvml direct); insufficient → `ResourceError` (not panic). (`3C:L121-L124`)
- **P3C-E026** [E/TEST]: Adversarial: Executor budget 8/1200, MemoryWriter 2/400 at spawn; spawn_parallel sequential; ResourceError on VRAM=0. (`3C:L131-L137`)
- **P3C-E027** [E/STATE]: Lifecycle SM: Spawning→Ready→Running→Finished (`Failed; Phase 3 agent is tokio task.`)
- **P3C-E028** [E/DATA]: `AgentDeathClassification`: CLEAN_EXIT, CRASH, TIMEOUT (15s heartbeat silence), BUDGET_EXCEEDED. (`3C:L165-L169`)
- **P3C-E029** [E/FLOW]: Heartbeat every 5s via `tokio::sync::watch`; 15s silence = TIMEOUT; Tier C replaces with IPC AgentHeartbeat. (`3C:L170-L172`)
- **P3C-E030** [E/SEC]: Tier A visibility filter: per-agent `allowed_paths: Vec<GlobPattern>`; hidden default: `.elai_cc/`, rules.db, harness source, other agent checkpoints, SSB db, ONNX models, binary internals; returns NotFound. (`3C:L173-L177`)
- **P3C-E031** [E/FLOW]: ReplanRequest stub: counter on AgentHandle; second call `ReplanDenied`; MINOR→auditor then EXECUTING→PLANNING; MAJOR→INTERACTIVE human approval. (`3C:L178-L182`)
- **P3C-E032** [E/EVENT]: Every GRC think writes `ReasoningTraceEntry` to SSB: agent_id, step_index, step_type, content_summary max 200 chars, grounding_source/level, confidence, elapsed_ms, topic, budget_remaining. (`3C:L183-L186`)
- **P3C-E033** [E/DATA]: SSB schema migration may be required for action_types: reasoning_trace, isolation_metric, spiral_detected, supervision_exhausted (section 3 list omits them). (`3C:L154-L160`)
- **P3C-E034** [E/DATA]: Checkpoint path `.elai_cc/sessions/{sid}/agents/{agent_id}/` with five files: state/grc_history/tool_results/output_draft `.msgpack` + plain `checkpoint_ts` via ClockProvider. (`3C:L227-L235`)
- **P3C-E035** [E/FLOW]: Auto checkpoint triggers: after every GRC think, tool result, file edit confirmed; before slow ops (latency >1s). (`3C:L236-L240`)
- **P3C-E036** [E/RECOVERY]: Atomic checkpoint write: temp+fsync+rename; Windows delete-old-then-rename same-volume. (`3C:L241-L243`)
- **P3C-E037** [E/EVENT]: SSB checkpoint event payload is metadata only (agent_id, step_index, file sizes), not checkpoint body. (`3C:L244-L245`)
- **P3C-E038** [E/SEC]: Checkpoint serializer rejects UntrackedValue channel IDs with `CheckpointError::UntrackedValueReference`; LastValue/Topic allowed. (`3C:L246-L253`)
- **P3C-E039** [E/CFG]: If tool_results grows unbounded, rolling window default keep last 50 (configurable); target write <100ms after 100 tool calls. (`3C:L269-L271`)
- **P3C-E040** [E/RECOVERY]: Recovery: read latest checkpoint, spawn same role+agent_id new task, inject structured context, continue from last GRC step; first recovered step grounding_source=`CHECKPOINT:{path}`. (`3C:L291-L310`)
- **P3C-E041** [E/DATA]: Recovery recipe map: PromptMisdelivery/StaleBranch/CompileRedCrossCrate/ProviderFailure→CRASH; TrustPromptUnresolved/McpHandshakeFailure/PartialPluginStartup→N/A. (`3C:L300-L307`)
- **P3C-E042** [E/RECOVERY]: Corrupt checkpoint: degrade to fresh start with empty history + WARNING SSB (not permanent stuck). (`3C:L318-L320`)
- **P3C-E043** [E/DET]: Recovered agent_id unchanged; SSB step_index continues monotonically (no reset). (`3C:L308-L308, L321-L323`)
- **P3C-E044** [E/IDEM]: Recovered first action is NEXT GRC step (no replay of completed tool side effects). (`3C:L324-L326`)
- **P3C-E045** [E/TEST]: CRASH recovery resume <2s; TIMEOUT within heartbeat_timeout+2s. (`3C:L328-L329`)
- **P3C-E046** [E/CFG]: SupervisionPolicy: max_restarts default 3 (per role elai_cc.toml [agents.{role}.supervision]); restart_window = current phase duration; on_max_restarts Escalate (`SubstituteModel(ModelId).`)
- **P3C-E047** [E/CFG]: Phase 3 INTERACTIVE: Escalate; Phase 4G AUTONOMOUS: SubstituteModel; Phase 3 enum has SubstituteModel but gated (error if used INTERACTIVE). (`3C:L348-L351`)
- **P3C-E048** [E/STATE]: restart_count per agent per phase; reset 0 at phase boundary. (`3C:L352-L352`)
- **P3C-E049** [E/FLOW]: CLEAN_EXIT no restart; CRASH/TIMEOUT restart+increment; BUDGET_EXCEEDED inject convergence, no restart, no count increment. (`3C:L353-L357`)
- **P3C-E050** [E/EVENT]: On max restarts: stop, checkpoint final, execute on_max_restarts, log supervision_exhausted with full history. (`3C:L358-L360`)
- **P3C-E051** [E/CFG]: max_restarts <1 in config clamps to 1 with WARNING. (`3C:L375-L376`)
- **P3C-E052** [E/FLOW]: Anti-spiral: track last N GRC steps; same_step_count >3 (same step_type+topic+similar content) = SPIRAL. (`3C:L394-L397`)
- **P3C-E053** [E/DET]: Similar content = blake3 of normalized content (lowercase, whitespace-collapsed); B-16A supersedes prior blake2s. (`3C:L398-L401`)
- **P3C-E054** [E/FLOW]: On SPIRAL: (1) checkpoint (2) inject convergence status step (3) another identical step → permanent stop (not restart) (4) SSB spiral_detected with 3 steps evidence. (`3C:L402-L408`)
- **P3C-E055** [E/FLOW]: Spiral stop is supervised stop not crash; on_spiral same as on_max_restarts (Escalate INTERACTIVE). (`3C:L409-L410`)
- **P3C-E056** [E/SURFACE]: INTERACTIVE spiral escalation UI text: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]". (`3C:L411-L413`)
- **P3C-E057** [E/RECOVERY]: Spiral tags pre-spiral checkpoint (N-3) as recovery_target; recovery loads pre-spiral not latest. (`3C:L428-L433`)
- **P3C-E058** [E/DEFER]: Tier B sandbox (Firecracker/gVisor) not in 3C: bash has only Tier A path filter; known security gap until 4A. (`3C:L446-L451`)
- **P3C-E059** [E/DEFER]: isolation_metric aggregation at phase boundary is 3A/3D scope; 3C only supplies per-agent budget_remaining data. (`3C:L452-L456`)
- **P3C-E060** [E/OBS]: Measures: checkpoint write <50ms; recovery <2s; checkpoint size <1MB/agent; heartbeat overhead <0.1% CPU; anti-spiral <1ms/step; memory overhead per agent logged. (`3C:L458-L465`)
- **P3C-E061** [E/LIMIT]: No lazy stubs: unimplemented!() is build failure; spawn_parallel sequential is valid stub. (`3C:L129-L129, L471`)
- **P3C-I001** [I/SURFACE]: HUMAN-facing surfaces in 3C are INTERACTIVE escalation panels for max-restarts and spiral (options listed); no dedicated dashboard panel is named. (`3C:L348-L349, L411-L413`)
- **P3C-A001** [A/CFG]: Rolling-window N for tool_results default 50 is given only as adversarial mitigation when write exceeds 100ms; whether always-on is required is unclear. (`3C:L269-L271`)
- **P3C-A002** [A/DATA]: Exact SSB CHECK constraint migration mechanism (ALTER vs recreate) left as implementer choice. (`3C:L157-L160`)
- **P3C-A003** [A/FLOW]: "last N GRC steps" window size for spiral tracking is not numerically fixed beyond the >3 consecutive trigger. (`3C:L395-L397`)

### 3E: Context budget allocator

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-3E.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-001 — 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection.; PHASE-008/IX-008 — 3E budget is recomputed inside 3D BSP superstep between prepare_next_tasks and execute_tasks; consumes channel kinds from 3D.; PHASE-008/IX-009 — 3E Terminal AUTONOMOUS SPLIT_TASK is owned by 4G; INTERACTIVE escalate is immediate.; PHASE-008/IX-010 — 3E OutputReserve and floors protect model generation headroom used by all later agent invocations.; PHASE-008/IX-027 — Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs).; PHASE-008/IX-028 — Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts.

**Behaviors**

- **P3E-E001** [E/STATUS]: Goal: fixed-slot context budget so model context is never exceeded; adaptive reduction with floors; cache breakpoints for provider prefix caching. (`3E:L7-L10`)
- **P3E-E002** [E/DEP]: Prerequisites: Phase 3D BSP+channels; depends on providers token_count/token_count_estimate, SSB, BSP. (`3E:L2-L3`)
- **P3E-E003** [E/DATA]: `ContextSlot` (8): SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext, SsbEntries, GrcHistory, OutputReserve. (`3E:L21-L30`)
- **P3E-E004** [E/DATA]: `SlotAllocation`: slot, allocated_tokens, floor_tokens (0 for slots 1-3), actual_tokens, source_truncated. (`3E:L32-L38`)
- **P3E-E005** [E/DATA]: `ContextBudget`: model_max_context from ModelCapabilityRegistry; slots[8]; cache_breakpoints[2]; total_allocated/used; adaptive_level. (`3E:L40-L48`)
- **P3E-E006** [E/DATA]: `CacheBreakpoint`: after_slot (BP1 after PhaseSpec, BP2 after CodeContext), prefix_hash u64, valid bool. (`3E:L49-L53`)
- **P3E-E007** [E/STATE]: `AdaptiveLevel`: Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal. (`3E:L55-L61`)
- **P3E-E008** [E/FLOW]: Allocation uses token_count_estimate; final validation uses exact token_count; exact exceed triggers adaptive; hard truncate never silent. (`3E:L66-L72`)
- **P3E-E009** [E/OBS]: Full budget computation latency target <50ms. (`3E:L73-L73`)
- **P3E-E010** [E/FLOW]: Fixed slot order never reordered: 1 SystemPrompt, 2 Rules, 3 PhaseSpec, BP1, 4 RepoMap, 5 CodeContext, BP2, 6 SsbEntries, 7 GrcHistory, 8 OutputReserve. (`3E:L78-L88`)
- **P3E-E011** [E/FLOW]: BP1 invalidated by phase transition, role change, rule set change; BP2 by file set change; slots 6-8 recompute every invocation. (`3E:L90-L93`)
- **P3E-E012** [E/INT]: When BP1/BP2 valid, provider reuses cached KV for slots 1-3 / 1-5 via CacheBreakpoint.valid. (`3E:L95-L98`)
- **P3E-E013** [E/CFG]: Dynamic percentages: RepoMap 15%, CodeContext 40%, SsbEntries 25%, GrcHistory 20% after fixed 1-3 exact and OutputReserve subtracted. (`3E:L103-L113`)
- **P3E-E014** [E/CFG]: Floors: SsbEntries≥200, GrcHistory≥500, OutputReserve≥2000; dynamic_budget < sum(floors) terminal. (`3E:L115-L119`)
- **P3E-E015** [E/FLOW]: File overlap score = (`intersection(entry.files, agent.files)`)
- **P3E-E016** [E/FLOW]: Budget recomputed per agent invocation in BSP superstep; called after prepare_next_tasks, before execute_tasks. (`3E:L133-L139`)
- **P3E-E017** [E/DATA]: Channel feed to slot 6: LastValue if overlap>0; Topic filtered by overlap; EphemeralValue only current superstep; UntrackedValue excluded. (`3E:L134-L138`)
- **P3E-E018** [E/EVENT]: Adaptive reduction max 3 attempts; each logs SSB action_type='context_reduction' with before/after tokens. (`3E:L144-L145`)
- **P3E-E019** [E/FLOW]: Attempt1 FilteredByRelevance: raise overlap threshold (e.g. 0.3), GRC current-topic only. (`3E:L147-L150`)
- **P3E-E020** [E/FLOW]: Attempt2 ReducedGranularity: denser PageRank, code→signatures+doc comments. (`3E:L152-L155`)
- **P3E-E021** [E/FLOW]: Attempt3 ReducedScope: current file+direct deps map; current file code; agent's own SSB only. (`3E:L157-L161`)
- **P3E-E022** [E/FAIL]: Terminal: INTERACTIVE ESCALATE with budget breakdown; AUTONOMOUS SPLIT_TASK (Phase 4G). Never silent truncate. (`3E:L163-L166`)
- **P3E-E023** [E/DATA]: Prompt assembly map: slots 1→role_prompt, 2→tier_3_rules, 4+5 merge→code_context, 3→task_spec+output_format+constraints; 6-7 structured after task_spec; 8 not in prompt. (`3E:L171-L186`)
- **P3E-E024** [E/FLOW]: Session handoff CMV trimming preserves BP1 slots 1-3; BP2 recomputed on new session (do not assume BP2 survives). (`3E:L191-L197`)
- **P3E-E025** [E/CFG]: OutputReserve: max(configured, 2000) subtracted before percentage split. (`3E:L229-L229`)
- **P3E-E026** [E/DET]: Remainder after floor division assigned to largest slot (CodeContext); sum all slots == model_max_context. (`3E:L231-L238`)
- **P3E-E027** [E/FLOW]: Floor steal order: CodeContext first, then RepoMap; both insufficient → Err(BudgetInfeasible); dynamic <700 → BudgetInfeasible. (`3E:L250-L254`)
- **P3E-E028** [E/OBS]: If CodeContext <200 after steal, log warning to SSB (not error). (`3E:L261-L261`)
- **P3E-E029** [E/DET]: prefix_hash uses deterministic hasher (xxhash/FxHash), never DefaultHasher. (`3E:L51-L51, L281-L281`)
- **P3E-E030** [E/DET]: Empty agent_files → overlap score 0.0 (no division by zero). (`3E:L302-L304`)
- **P3E-E031** [E/OBS]: 10k SSB entries filtering target <10ms. (`3E:L305-L305`)
- **P3E-E032** [E/EVENT]: Reduction logging writes SSB but does NOT enter current invocation's context (visible next turn only). (`3E:L327-L327`)
- **P3E-E033** [E/TEST]: Integration: Gemma 4 E2B 128K context; test 4096 and 131072; assembly <50ms via ClockProvider; 100x byte-identical. (`3E:L337-L348`)
- **P3E-E034** [E/LIMIT]: No Instant::now in context budget module (entropy quarantine). (`3E:L351-L351`)
- **P3E-E035** [E/SURFACE]: ContextBudget implements Display for debug-friendly budget breakdown; INTERACTIVE terminal surfaces Escalate(breakdown) to user. (`3E:L210-L210, L163-L164, L324`)
- **P3E-E036** [E/TEST]: 3E.1–3E.7 steps: types, compute, floors, cache breakpoints, SSB filter, adaptive, e2e integration. (`3E:L200-L355`)
- **P3E-I001** [I/CFG]: Configurable overlap raise "e.g. 0.3" is example not hard default for attempt 1. (`3E:L148-L148`)
- **P3E-A001** [A/CFG]: Exact default values for model_max_context sources and fixed-slot measurement sites left to ModelCapabilityRegistry/runtime. (`3E:L41-L41, L226-L227`)
- **P3E-A002** [A/SURFACE]: No dedicated cockpit/panel for budget is named; only Display + escalate breakdown. (`3E:L210, L163-L164`)

### 4B: Tools (git, validation, DAIRA, edit match, ShortCoder, taxonomy, file read, incr. compile)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 1 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4B.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-011 — 4B tools run only through 4A IPC ToolRequest/ToolResult; isolation/sanitization owned by 4A, execution by 4B.; PHASE-008/IX-012 — 4B IntegrationTestRunner blocked on 4E DecompositionTree types and matches 4E integration time budgets.; PHASE-008/IX-013 — 4B ShortCoder post-processes executor output before auditor; placement also referenced from 4C hooks.; PHASE-008/IX-014 — 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral.; PHASE-008/IX-015 — 4B write tools honor SSB file locks (orchestrator concurrency invariant).; PHASE-008/IX-028 — Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts.; PHASE-008/IX-029 — RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J.

**Behaviors**

- **P4B-E001** [E/STATUS]: Prerequisites: Phase 4A agent isolation + IPC. Goal: full tool suite in orchestrator; agents access via IPC ToolRequest/ToolResult. (`4B:L1-L2, L17-L21`)
- **P4B-E002** [E/STATUS]: Recorded 2026-04-16 commit 55ece36: 4B.1–4B.4 DONE (10 tool modules, IpcToolRegistry, 15 typed arg structs, 19/19 tests; 3 BLOCKs fixed). (`4B:L3-L7`)
- **P4B-E003** [E/DEFER]: 4B.5 MEASURE deferred (metrics remaining). 4B.6–4B.7 IntegrationTestRunner deferred blocked on 4E.7+ types. (`4B:L8-L10`)
- **P4B-E004** [E/LIMIT]: Scope exclusions: VisibilityFilter + schema sanitization = 4A; BashSandbox inherited not implemented; lsp_client evaluation deferred Phase 5 if not in MEASURE. (`4B:L39-L46`)
- **P4B-E005** [E/FAIL]: R1: IPC schema drift mitigated by postcard round-trip contract tests per ToolRequest variant. (`4B:L51-L54`)
- **P4B-E006** [E/FAIL]: R2: compilation on written file only; unresolved import of unwritten module → DEFERRED not COMPILATION_ERROR. (`4B:L55-L59`)
- **P4B-E007** [E/CFG]: R3: tools/ and knowledge/ pin identical tree-sitter grammar versions workspace-level. (`4B:L60-L63`)
- **P4B-E008** [E/CONC]: R4: ToolRegistry checks SSB file locks before write tools (no two agents write same file). (`4B:L64-L67`)
- **P4B-E009** [E/DATA]: ToolRequest variants: GitCommit/GitUndo/GitBranch, ValidateRequest, TraceRequest, EditRequest(+tier hint), SimplifyRequest, ClassifyRequest, FileReadRequest(+force_full), CompileCheckRequest. (`4B:L81-L86`)
- **P4B-E010** [E/CFG]: Tools registered with phase-aware filtering; PLANNING must not expose EditRequest. (`4B:L87-L88, L96-L97`)
- **P4B-E011** [E/CAP]: git2: auto-commit deterministic message from phase outcome; branch plan-{pipeline_id}; WIP commits for >5 edits; session-owned undo. (`4B:L177-L180`)
- **P4B-E012** [E/CAP]: 5-tier validation: T1 determinism → T2 quality/AST → T3 unit → T4 integration → T5 stress; auditor only, executor never runs own tests. (`4B:L181-L184`)
- **P4B-E013** [E/CAP]: DAIRA markers: CALL, RETURN, MUTATION, BRANCH, ERROR, PANIC, TIMEOUT, FINDING, PATTERN, ANOMALY, STAT:name, DURATION:name, MEMORY:name, CHECKPOINT, STEP:n; Rust tracing + Python sys.settrace. (`4B:L185-L190`)
- **P4B-E014** [E/CAP]: AdaptiveTool<T> budget-based scope reduction max 3 then error. (`4B:L189-L190, L237-L239`)
- **P4B-E015** [E/OBS]: MarkerReport includes raw_line_count and marker_line_count (ratio drives AdaptiveTool). (`4B:L132-L133`)
- **P4B-E016** [E/CAP]: 3-tier edit match: exact → whitespace-normalized (preserve indent) → ellipsis-elision; on fail SequenceMatcher similarity hint; never silent wrong match. (`4B:L191-L195`)
- **P4B-E017** [E/CAP]: ShortCoder: 10 tree-sitter rules R1–R10; R4/R6 configurable; post-process executor output before auditor. (`4B:L196-L198`)
- **P4B-E018** [E/CAP]: Failure taxonomy 9 categories: SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED; deterministic suggested_action; auditor reads before raw; 3+ same category/phase → anti-spiral. (`4B:L199-L204`)
- **P4B-E019** [E/CAP]: File read: outline mode >300 lines (tree-sitter sections); force_full bypass; budget guard warn if file >50% remaining context; INTERACTIVE user decides; AUTONOMOUS auto-downgrade outline. (`4B:L205-L208`)
- **P4B-E020** [E/CAP]: Incremental compile ASYNC after each EXECUTING write: cargo check/ruff/tsc/go vet on written file; results via IPC; max 3 rounds (2 retries); then NEEDS_AUDIT; skip TESTING_FIRST. (`4B:L209-L218`)
- **P4B-E021** [E/SEC]: IPC error messages sanitized: strip rule_id, predicate types, hook names. (`4B:L220-L222`)
- **P4B-E022** [E/OBS]: Metrics path `.elai_cc/metrics/phase_4B.toml` keys: ipc_overhead_ms, compile_check_latency_{rust,python,typescript,go}_ms, shortcoder_savings_pct, test_count, loc_tools. (`4B:L274-L277`)
- **P4B-E023** [E/OBS]: IPC dispatch overhead target <2ms p95; measure 100 calls p50/p95/p99. (`4B:L256-L260, L280-L281`)
- **P4B-E024** [E/OBS]: Compile latency expectations: Rust 2-5s, Python ruff <500ms, tsc <2s, go vet <1s (non-Rust marked UNGROUNDED until measured). (`4B:L261-L265`)
- **P4B-E025** [E/OBS]: Auditor BLOCK reduction ~30% is UNGROUNDED design estimate. (`4B:L266-L270`)
- **P4B-E026** [E/OBS]: ShortCoder paper claims 18-38% savings; measure on 10 representative samples. (`4B:L271-L272`)
- **P4B-E027** [E/DEP]: Papers daira-interface.md and shortcoder-interface.md required ingested before TESTING_FIRST. (`4B:L34-L36, L78-L79`)
- **P4B-E028** [E/CAP]: IntegrationTestRunner levels: BabyStep (<5s cargo check), Step (<30s module tests), Phase (<60s crate tests), Concept (<120s workspace concept_tag). (`4B:L306-L321`)
- **P4B-E029** [E/DATA]: IntegrationResult: level, passed, test_count, failures, stdout_hash [u8;32], elapsed_ms. (`4B:L329-L336`)
- **P4B-E030** [E/CFG]: tool_id="integration_test" available in AUDITING only; NOT during EXECUTING; orchestrator-triggered not agent-triggered. (`4B:L344-L346`)
- **P4B-E031** [E/INT]: Integration uses BashSandbox, FailureTaxonomy, IPC ToolResult, BABY_STEP uses incr. compile infrastructure. (`4B:L338-L342`)
- **P4B-E032** [E/LIMIT]: Every request-routing site must wire ssb, fsm, clock through single RuntimeClient (no None fields post-construction). (`4B:L298-L298`)
- **P4B-E033** [E/TEST]: ≥15 test functions; step ladder skeleton→verify→failing tests→implement→measure→integration. (`4B:L71-L170, L252-L287`)
- **P4B-E034** [E/DET]: git2 timestamps from ClockProvider not system time. (`4B:L232-L233`)
- **P4B-E035** [E/FLOW]: Validation tiers must short-circuit: tier-1 failure stops tiers 2-5. (`4B:L234-L236`)
- **P4B-E036** [E/TEST]: Incremental compile event-ordering test: next ToolRequest before mock compile completes. (`4B:L240-L243`)
- **P4B-E037** [E/TEST]: Supported languages for incr. compile tests: Rust, Python, TS, Go (one each). (`4B:L165-L166`)
- **P4B-I001** [I/SURFACE]: Tools are agent-facing IPC surfaces, not human UI; human sees outcomes via auditor/SSB/dashboard elsewhere. (`4B:L17-L18`)
- **P4B-A001** [A/STATUS]: Whether papers remain MISSING at time of this discovery is runtime state; plan still requires them as PRE_GATE. (`4B:L35-L36`)
- **P4B-A002** [A/CFG]: Exact deterministic git commit message format template not specified beyond "from phase outcome". (`4B:L178-L178`)
- **P4B-C001** [C/STATUS]: Header completion status says 4B.1-4.4 DONE while body still lists full TDD ladder; status is historical record, steps remain normative for residual/measure work. (`4B:L3-L10, L71-L287`)

### 4F: Feasibility probe

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 3 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4F.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-016 — 4F reuses 2D FeasibilityOutcome and 4A Tier-C agent process with restricted AgentDispatchConfig.; PHASE-008/IX-017 — 4F skip path PLANNING→TESTING_FIRST and novel_steps marking come from 4E structural validation.; PHASE-008/IX-018 — 4F decision Escalate vs Replan depends on execution mode introduced by 3A/4G.; PHASE-008/IX-019 — 4F SSB branch tags compose with 3B SSB and 3D branch semantics (tag=branch:…).; PHASE-008/IX-020 — 4F baby-step feasibility requires 4E DecompositionTree/BabyStepSpec and runs before concept execution in 4G.; PHASE-008/IX-028 — Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts.; PHASE-008/IX-029 — RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J.; PHASE-008/IX-030 — blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A).

**Behaviors**

- **P4F-E001** [E/STATUS]: Prerequisites: Phase 4E ExecutablePipeline. Goal: system probe + novel-step stubs (<50 lines) with branch-and-explore; skip when zero novel; escalate after max 3 alternatives; output stub_results.msgpack. (`4F:L1-L11`)
- **P4F-E002** [E/DATA]: SystemProbeResult: gpu_count, gpu_info, ram_total/free, disk_free, os, is_supported. (`4F:L28-L36`)
- **P4F-E003** [E/DATA]: GpuInfo: index, name, total/free bytes, driver_version, uuid (stable NVML). (`4F:L38-L45`)
- **P4F-E004** [E/DATA]: SystemConstraints: min_vram/ram/disk bytes, required_gpu_count. (`4F:L47-L52`)
- **P4F-E005** [E/DATA]: StubResult: step_id, alternative_index 0-2, FeasibilityOutcome, elapsed_ms, stdout_hash blake3, stderr_summary first 512 bytes, ResourceDelta. (`4F:L54-L62`)
- **P4F-E006** [E/DATA]: FeasibilityReport: probe, constraints, constraint_results, stubs, skipped_steps, escalations. (`4F:L71-L78`)
- **P4F-E007** [E/DATA]: ConstraintResult: resource string, required, available, margin, passed (margin≥0 met; margin≥0.20 skip threshold). (`4F:L80-L86`)
- **P4F-E008** [E/DATA]: FeasibilityEscalation: step_id, reason, stubs_tried (3 fails). (`4F:L88-L92`)
- **P4F-E009** [E/STATE]: FeasibilityDecision: Proceed, ProceedWithConstraints(Vec), Escalate(Vec) INTERACTIVE, Replan(Vec) AUTONOMOUS. (`4F:L94-L99`)
- **P4F-E010** [E/DATA]: Serialization: msgpack disk, postcard IPC; FeasibilityOutcome reused from reasoning (2D). (`4F:L101-L103`)
- **P4F-E011** [E/FLOW]: Skip when novel_steps empty AND all VRAM/RAM/disk free ≥ required*1.20; else run probe. (`4F:L107-L115`)
- **P4F-E012** [E/EVENT]: Skip logs SSB action_type='phase_transition' with skip_reason; FSM PLANNING→TESTING_FIRST. (`4F:L116-L118`)
- **P4F-E013** [E/FAIL]: No NVIDIA: Nvml::init Err → gpu_count=0, empty gpu_info. (`4F:L122-L124`)
- **P4F-E014** [E/OBS]: Keep single sysinfo System; refresh_specifics only needed categories. (`4F:L126-L128`)
- **P4F-E015** [E/FLOW]: Re-check VRAM via ModelPool.available_vram immediately before stub launch. (`4F:L130-L133`)
- **P4F-E016** [E/SEC]: Stubs Tier C with FEASIBILITY tools only: stub write + bash + web search; NO edit/git. (`4F:L139-L141`)
- **P4F-E017** [E/CAP]: FeasibilityProbe trait: probe_system, check_constraints, run_stub, decide; object-safe &self. (`4F:L157-L161`)
- **P4F-E018** [E/CFG]: Feasibility AgentDispatchConfig overrides: GRC 3/600; allowed_tools [stub_write,bash]; sandbox 25s graceful/30s hard; line limit strict <50. DispatchStage.agent AUTHORITATIVE. (`4F:L164-L172`)
- **P4F-E019** [E/INT]: IPC reuse 4A: 1-byte version + 4-byte length + postcard. (`4F:L173-L173`)
- **P4F-E020** [E/EVENT]: SSB branch per stub alternative tag="branch:stub_{step_id}_{n}"; max 3; merge winner discard losers. (`4F:L175-L175, L377-L380`)
- **P4F-E021** [E/TEST]: 16 contract tests T1–T16 covering probe, constraints, stub pass/reject/timeout, max3 escalate, skip matrix, msgpack, sandbox deny edit, VRAM recheck, dual timeout, ProceedWithConstraints, malformed IPC. (`4F:L228-L281`)
- **P4F-E022** [E/TEST]: 8 adversarial A1–A8: fs escape, network, malformed frame, concurrent GPU, no-GPU CI, oversized frame, VRAM race, 4th branch escalate. (`4F:L305-L323`)
- **P4F-E023** [E/LIMIT]: Stub ≥50 lines rejected before execution (strict <50). (`4F:L243-L245, L368`)
- **P4F-E024** [E/FLOW]: Dual timeout: graceful shutdown IPC at 25s; hard kill at 30s with explicit child.kill (kill_on_drop false). (`4F:L269-L273, L369-L374`)
- **P4F-E025** [E/FAIL]: Malformed IPC → StubResult Fail with stderr_summary containing "malformed_ipc". (`4F:L278-L281`)
- **P4F-E026** [E/FAIL]: VRAM drop pre-launch → Fail reason="vram_dropped" / "insufficient_vram". (`4F:L266-L268, L319-L321`)
- **P4F-E027** [E/CFG]: MAX_FRAME_SIZE 10MB for FeasibilityReport postcard. (`4F:L189-L190`)
- **P4F-E028** [E/OBS]: Metrics phase_4F.toml: system_probe_latency_ms (<500), stub p50/p99, constraint_check_latency_us (<100), report_serialization_bytes, skip_criteria_evaluation_us, test_count/pass, compile_time_delta_s. (`4F:L416-L425`)
- **P4F-E029** [E/CAP]: Baby-step extension: novel if PackageRef.verified=false OR unused framework OR untested capability (GPU/FFI). (`4F:L488-L491`)
- **P4F-E030** [E/FLOW]: Feasibility per concept before concept N execution (not all upfront). (`4F:L493-L496`)
- **P4F-E031** [E/DATA]: BabyStepFeasibilityReport / BabyStepStubResult / BabyStepEscalation types. (`4F:L503-L520`)
- **P4F-E032** [E/FLOW]: Per-concept skip when zero novel baby steps. (`4F:L498-L499`)
- **P4F-E033** [E/DEP]: Crate deps: nvml-wrapper, sysinfo, blake3 (if missing); rmp-serde already present. (`4F:L461-L465`)
- **P4F-E034** [E/CFG]: E-22 note: sysinfo dep may be rejected; RAM/disk via wmic//proc/meminfo/df + ELAI_FREE_RAM_GB/ELAI_FREE_DISK_GB fallbacks; nvml-wrapper activated non-optional. (`4F:L339-L346`)
- **P4F-E035** [E/LIMIT]: RuntimeClient ssb/fsm/clock wiring required on all request-routing structs. (`4F:L475-L475`)
- **P4F-E036** [E/DET]: Constraint check pure (no I/O); stdout blake3 deterministic for identical bytes. (`4F:L445-L456`)
- **P4F-I001** [I/SURFACE]: INTERACTIVE Escalate decision is human-facing; no dedicated panel name beyond decision enum. (`4F:L97-L98`)
- **P4F-A001** [A/CFG]: Whether web_search is always in FEASIBILITY tool set conflicts slightly: skeleton says stub_write+bash only; I3 text adds web search. (`4F:L167-L167, L366-L367`)
- **P4F-C001** [C/DATA]: Trait named FeasibilityProbe in TASK but greps look for FeasibilityProber; SD2 notes flat feasibility_probe.rs layout vs subdir. (`4F:L157-L184`)
- **P4F-C002** [C/DEP]: Body still specifies sysinfo API, while E-22 pre-gate rejects sysinfo as new dep and uses shell-out fallbacks. (`4F:L353-L360, L339-L344`)

### 4J: Empire integration

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Superseded / conflicts**: 2 conflict/ambiguity atoms retained inline below (see [C]/[A] tags).
- **Anchors**: source `docs/specs/guide/steps/phase-4J.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-021 — 4J event bridge writes Empire telemetry into 3B SSB for GRC grounding.; PHASE-008/IX-022 — 4J ElaiNeuron capability path reuses 4A IPC proxy + ToolFilter trust boundary.; PHASE-008/IX-023 — 4J repair pipelines are generated by 4D pipeline factory and validated with 4H TestOutcomeReport.; PHASE-008/IX-024 — 4J autonomous unattended Empire operation depends on 4G autonomous mode.; PHASE-008/IX-029 — RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J.; PHASE-008/IX-031 — 4J Tauri/Svelte Empire panel will eventually sit beside 4K dashboard surfaces but 4K is out of this chunk.

**Behaviors**

- **P4J-E001** [E/STATUS]: Conditional phase: only when Empire codebase accessible/deployed. Prereqs 4A, 3B, 4D, 4G. (`4J:L1-L8`)
- **P4J-E002** [E/STATUS]: EMPIRE-PENDING gate (2026-05-22): Empire not accessible; ship host scaffolding now with EMPIRE-PENDING labels; live activation one-line flip. (`4J:L12-L19`)
- **P4J-E003** [E/CFG]: Implementation at crates/orchestrator/src/empire/* under cfg(feature="empire"); not crates/empire; cargo check -p elai-orchestrator --features empire. (`4J:L21-L28`)
- **P4J-E004** [E/EVENT]: SSB action_type granularity = 5 variants: empire_connected, empire_disconnected, empire_event_received, empire_capability_invoked, empire_repair_proposed (not single empire_event). (`4J:L30-L37`)
- **P4J-E005** [E/DATA]: Empire IPC namespace empire::ipc::EmpireIpcMessageType (do not shadow elai_ipc_types::IpcMessageType 16 locked variants). (`4J:L39-L43`)
- **P4J-E006** [E/CAP]: Goal: speak FactoryLine, register as brain, subscribe Empire events as live GRC/SSB source, expose neuron capabilities, replace brainctl. (`4J:L45-L50`)
- **P4J-E007** [E/INT]: Empire topology: EngineMKI→MKII→MKIII; IPC sock /tmp/brain-engine.sock; FactoryLine /tmp/brain-factory.sock; newline JSON AF_UNIX; max 65536 IPC; FSB 12-byte header, 256KB max frame. (`4J:L52-L59`)
- **P4J-E008** [E/DATA]: FactoryLine cmds: register (`heartbeat`)
- **P4J-E009** [E/DATA]: Engine handlers MKI: REGISTER_HOOK, EMIT, QUERY_HOOKS, SUBSCRIBE, UNSUBSCRIBE, BUS_CREATE, BUS_JOIN, BUS_LEAVE, BUS_LIST. (`4J:L71-L72`)
- **P4J-E010** [E/FAIL]: Exactly 3 verified broken issues: brainctl↔brain_lifecycle circular; EngineMKII hard-import slc; unique-factory/types stdlib shadow. (`4J:L73-L78`)
- **P4J-E011** [E/LIMIT]: Dead code supersede: workforce lambda stubs; unvalidated brain.json superseded by PipelineSpec 6 layers. MetricsStore/SeedState alive. (`4J:L79-L83`)
- **P4J-E012** [E/CAP]: FactoryLineTransport trait: connect, health_check; TransportConfig Unix (`Tcp; UnixTransport cfg(unix), TcpTransport all.`)
- **P4J-E013** [E/CFG]: FactoryLineConfig: transport, heartbeat_interval, reconnect_backoff ExponentialBackoff, event_rate_limit. (`4J:L146-L147`)
- **P4J-E014** [E/DATA]: FactoryLineCommand/FactoryCmd/FactoryLineResponse; IpcMessage/EmpireIpcMessageType (9 kinds)/IpcResponse; shared Event/Command/Response. (`4J:L152-L173`)
- **P4J-E015** [E/DATA]: BrainManifest with schema validation: brain_id, version, engine_tier MKI (`MKII`)
- **P4J-E016** [E/CAP]: EmpireEventBridge: ssb_writer, rate_limiter (always present not Option), EventFilter patterns+priority_map High (`Normal`)
- **P4J-E017** [E/CAP]: ElaiNeuron exposes only task_submit, task_status, pipeline_result, health_check; never FSM/evidence/rules/SSB-direct/config/agent_spawn. (`4J:L196-L202, L409-L411`)
- **P4J-E018** [E/DATA]: EmpireIssue CircularDep (`HardImport`)
- **P4J-E019** [E/SEC]: Empire-originated requests enter IPC proxy ToolFilter; cannot trigger FSM transitions or bypass gates. (`4J:L113-L117`)
- **P4J-E020** [E/FLOW]: Heartbeat at configured interval; 3 consecutive misses → reconnection backoff; reconnect re-sends Register. (`4J:L304-L308, L538-L539`)
- **P4J-E021** [E/LIMIT]: Zero brainctl imports; pure Rust FactoryLine client; zero pyo3/cpython. (`4J:L105-L108, L344-L351`)
- **P4J-E022** [E/FLOW]: Rate limit global (not per-pattern); high-priority bypass elevated but bounded (e.g. 100/sec not unlimited); dropped_count metrics. (`4J:L360-L389`)
- **P4J-E023** [E/SEC]: Rejection responses generic "capability not available" with no internal state leak. (`4J:L417-L449`)
- **P4J-E024** [E/FLOW]: task_submit returns task_id async; task_status polls FSM+progress; pipeline_result final output; same path as CLI /task. (`4J:L412-L432`)
- **P4J-E025** [E/CAP]: Repair: detect 3 issues programmatically; one PipelineSpec per issue; blast radius enforcement; TestOutcomeReport zero PASS_TO_FAIL. (`4J:L468-L517`)
- **P4J-E026** [E/RECOVERY]: Graceful degradation: Empire optional; features disable cleanly; elai starts without connection. (`4J:L540-L544, L612-L615`)
- **P4J-E027** [E/FAIL]: Startup unreachable: max 5 reconnect attempts then Error::EmpireUnreachable. (`4J:L536-L537`)
- **P4J-E028** [E/DEFER]: FSB binary transport evaluate-only; upgrade if event_latency_p95 >5ms consistently; frame payload_len > MAX_FSB_PAYLOAD default 16MB reject. (`4J:L545-L549, L678-L685, L563-L565`)
- **P4J-E029** [E/DEFER]: Multi-brain orchestration evaluate when multiple elai instances need coordination beyond multi-session. (`4J:L687-L694`)
- **P4J-E030** [E/OBS]: Metrics phase_4J.toml: connection_latency_ms (<100 Unix / <500 TCP), heartbeat_jitter_ms (<500), event_throughput_per_sec (≥90% of limit), event_latency_p95_ms (<10), capability_roundtrip_ms, repair_scan_time_ms (<5s), binary_size_delta_bytes, test_count (≥40). (`4J:L644-L672`)
- **P4J-E031** [E/DET]: Wire newline is exactly "\n" not "\r\n". Protocol v:1 preserved. (`4J:L288-L292, L320-L322`)
- **P4J-E032** [E/INT]: Empire events grounding source type EMPIRE_EVENT / source='empire' for GRC. (`4J:L370-L373, L596-L597`)
- **P4J-E033** [E/SURFACE]: Host scaffolding includes Tauri command surface + Svelte panel + feature gate (EMPIRE-PENDING). (`4J:L16-L18`)
- **P4J-E034** [E/SURFACE]: brainctl CLI replacement: orchestrator drives FactoryLine (start/stop/status brain lifecycle). (`4J:L343-L349, L704`)
- **P4J-E035** [E/SURFACE]: Capability discovery action "capabilities" returns exactly 4 entries. (`4J:L423-L427`)
- **P4J-E036** [E/CFG]: Feature gate cfg(feature="empire"); non-empire builds zero cost. (`4J:L614-L615, L708`)
- **P4J-E037** [E/LIMIT]: RuntimeClient ssb/fsm/clock wiring invariant. (`4J:L710-L710`)
- **P4J-E038** [E/TEST]: Steps 4J.1–4J.9 skeleton→verify→protocol tests→event bridge→capability→repair→resilience→implement→measure. (`4J:L122-L676`)
- **P4J-I001** [I/SURFACE]: Svelte panel and Tauri commands are required scaffolding surfaces but exact command/panel names are not listed in this phase file. (`4J:L16-L18`)
- **P4J-A001** [A/INT]: Exact default socket paths for Windows TCP and remote host config schema beyond TransportConfig not fully specified. (`4J:L137-L138`)
- **P4J-C001** [C/EVENT]: PRE-RESOLVED DECISION 3 requires 5 SSB variants, but 4J.4/4J.8 TASK still write action_type='empire_event' singular. (`4J:L30-L37, L356-L358, L596-L596`)
- **P4J-C002** [C/TEST]: DETERMINISTIC_CHECK still says cargo test -p elai_empire while DECISION 2 redirects to elai-orchestrator --features empire. (`4J:L21-L28, L313-L314, L618`)

### IDX: Phase index pre0–2D (concise reference)

- **Status**: planned (implementation-guide phase step; plan-recorded, deep pass did not independently verify ship status)
- **Anchors**: source `docs/specs/guide/steps/phase-index-pre0-2D.txt`; atoms in `PHASE-008 / DEEP-PHASE-008.discovery.md`; unit header in PHASE-AGGREGATE.md
- **Composes with**: PHASE-008/IX-006 — 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400.; PHASE-008/IX-025 — Index 2C role budgets are the same numbers 3C adversarial expects at spawn.; PHASE-008/IX-026 — Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase.; PHASE-008/IX-027 — Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs).; PHASE-003/IX-003 — 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget.; PHASE-003/IX-004 — 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry.; PHASE-003/IX-005 — 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots.; PHASE-003/IX-006 — 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth.; PHASE-003/IX-007 — 4E novel_step_ids consumed by 4F; visual_check adapters by 4I.; PHASE-003/IX-008 — 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks.; PHASE-003/IX-009 — 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1.; PHASE-003/IX-010 — 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds. …

**Behaviors**

- **IDX-E001** [E/STATUS]: Document is concise LLM-context index of pre-phase-0 through phase-2D: goals, deliverables, dependencies. (`IDX:L1-L2`)
- **IDX-E002** [E/CAP]: PRE-PHASE 0: Windows platform fixes only (cfg-gate unix PermissionsExt, .cmd hooks, path separators, python not python3, PoisonError, warnings). No feature changes. (`IDX:L5-L13`)
- **IDX-E003** [E/OBS]: PRE0 baseline: 69 .rs files, 61K LOC, 684 test annotations; 0 errors/failures/warnings on Windows. (`IDX:L11-L13`)
- **IDX-E004** [E/CAP]: PHASE 0: strip claw-code to elai_cc; 9-crate workspace; one model ollama; FSM IDLE→EXECUTING→IDLE; one tool file-read; archive old code. (`IDX:L16-L24`)
- **IDX-E005** [E/LIMIT]: Phase 0 risks: 25K LOC runtime/38 files, entropy quarantine 60 occurrences, config circular deps, TokenUsage refactor. (`IDX:L21-L22`)
- **IDX-E006** [E/CAP]: 1A DKG: tree-sitter→SQLite graph; langs Rust/Python/TS/Go; dkg_nodes+dkg_edges; parse failure log+skip; LanguageGrammar trait. (`IDX:L27-L34`)
- **IDX-E007** [E/CAP]: 1B BM25 tantivy: per-project; zero orphan non-binary; null-byte binary detect first 8KB; path STRING untokenized; update_file <5s; p99 query <10ms. (`IDX:L37-L43`)
- **IDX-E008** [E/CAP]: 1C dual-model ONNX vectors: GraphCodeBERT 768d + Granite 384d; cache by content hash; circuit breakers; cascade HYBRID→SPARSE_ONLY→BM25_ONLY; canary embed("fn main()") within 5s. (`IDX:L46-L53`)
- **IDX-E009** [E/DEFER]: SPLADE deferred from 1C to 1H measurement. (`IDX:L51-L51`)
- **IDX-E010** [E/CAP]: 1D PageRank repo map: density guard 0.3; personalization 10x; edge boost *50; sqrt(ref_count); +-15% token fit; line trunc 100 chars; 21+ tests. (`IDX:L56-L65`)
- **IDX-E011** [E/CAP]: 1E CatRAG+A-RAG full pipeline assembly BM25→vector→PageRank→CatRAG→A-RAG. (`IDX:L68-L72`)
- **IDX-E012** [E/CAP]: 1F speculative decoding evaluation: ADOPT if >1.3x throughput on RTX 3060 12GB else DROP; k=1-7 sweep; decision cascade availability→acceptance→prompt lookup→throughput→VRAM. (`IDX:L75-L84`)
- **IDX-E013** [E/CAP]: 1G cross-encoder ms-marco-MiniLM-L12-v2 ~120MB; CPU; <50ms/50 candidates; Semaphore(1)+mpsc depth 4; circuit breaker L-6 fallback. (`IDX:L87-L94`)
- **IDX-E014** [E/CAP]: 1H SPLADE conditional feature=splade; vocab coverage <30% early DROP; KEEP if mean recall Δ≥5% and p<0.05. (`IDX:L97-L105`)
- **IDX-E015** [E/CAP]: 1I resilience: circuit breaker; SearchMode cascade; query validator empty/>10K/null reject; readiness READY/DEGRADED/UNAVAILABLE; QueryTrace/StageTrace; index version migration. Deps 1E AND 1G. (`IDX:L108-L116`)
- **IDX-E016** [E/CAP]: 2A GRC SM: observe→hypothesize→plan_action→act→verify (+status, converge); budget per-topic; forced converge remaining_calls≤1; format-only grounding; GroundingLevel Verified>High>Uncertain; ThinkStep.scope; typed errors. (`IDX:L119-L127`)
- **IDX-E017** [E/CAP]: 2A GrcEngine trait 6 methods (4 implemented + 2 stubs); 12+ tests. (`IDX:L126-L126`)
- **IDX-E018** [E/CAP]: 2B verify methods FILE/SOURCE/TEST/WEB/API/API_REGISTRY_CHECK; binary pass/fail; plan sweep blocks UNGROUNDED before EXECUTING. (`IDX:L130-L137`)
- **IDX-E019** [E/CFG]: 2C six roles with budgets: executor 8/1200; auditor 10/1200 double-verify anti-rubber-stamp; context-compiler observe-only; memory-writer 2/400; prompt-engineer 5/800; designer 8/1200 PipelineSpec. (`IDX:L140-L149`)
- **IDX-E020** [E/CFG]: AgentGrcConfig 8 fields including require_scope_enumeration true for executor/designer; layer [grc.defaults]+[agents.{role}.grc]; additive transition overrides. (`IDX:L145-L147`)
- **IDX-E021** [E/FLOW]: Auditor anti-rubber-stamp: ≥1 FINDING: before Converge or rejected. (`IDX:L148-L148`)
- **IDX-E022** [E/CAP]: 2D CSQ: CRITICAL (data_loss/security/financial/auth guard mandatory blocks) vs WARNING (logged). (`IDX:L153-L156`)
- **IDX-E023** [E/CAP]: Inline feasibility max 2 checks per EXECUTING; Pass/Fail/Blocked; Blocked emits MAJOR ReplanRequest. Completes 2A-2D reasoning. (`IDX:L157-L160`)
- **IDX-E024** [E/DEP]: Dependency chain PRE0→0→1A→1B→1C→1D→1E→1G→1I; 1F after 1C; 1H after 1G; 2A after 0; 2B after 2A; 2C after 2B; 2D after 2C. (`IDX:L13-L161`)
- **IDX-I001** [I/LIMIT]: Index compresses full phase files; step-level PRE_GATE/TASK/ACCEPTANCE not present here (MASTER also delegates). (`IDX:L1-L2`)
- **IDX-A001** [A/STATUS]: Index does not record completion status of any pre0–2D phase. (`IDX:L1-L161`)

### User-facing surfaces (PHASE)

_Collected 243 surface-bearing atoms (SURFACE class + explicit CLI/UI descriptors)._

- R25: commands/lib.rs has 28+ claw-code slash commands that conflict with elai_cc /task, /config, /continue. — **P0-E012** (`phase-0:L32-L33`)
- 0.2 strip order (each followed by cargo check): telemetry crate; compat-harness + call sites; structural usage/compact/summary_compression with TokenUsage shim; lsp_client; oauth DEFERRED; audit prompt.rs; classify slash commands only; evaluate remote.rs. — **P0-E024** (`phase-0:L109-L155`)
- 0.2g KEEP candidates: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. — **P0-E026** (`phase-0:L146-L148`)
- 0.2g STRIP claw-code commands: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. — **P0-E027** (`phase-0:L149-L150`)
- 0.6 contracts crate, 5 tests provider_trait/fsm_basic/tool_basic/config_basic/visibility_filter; TDD red (compile, fail runtime). — **P0-E048** (`phase-0:L402-L428`)
- 0.7c slash commands /task, /config, /continue, /explore, /status; bare messages fallback to /task; unknown command error. — **P0-E055** (`phase-0:L517-L519`)
- CONFIGURING via /config or /workflow; config_paths_only tools; no evidence gate; ->IDLE on completion. — **P3A-E008** (`phase-3A:L60-L63`)
- INTERACTIVE gate: structured EvidenceSummary present_gate; not prose. — **P3A-E024** (`phase-3A:L205-L209`)
- CLI: python baseline_harness.py --profile micro_7b_12gb; expect exit 0 and util≥0.82. — **P3JB-E018** (`phase-3J-BASELINE:L125-L128`)
- Native structured control surface not chatbot; inputs via FSM classification; MVP interactive mode. — **P4K-E002** (`phase-4K:L8-L12`)
- Visual: teal #1B7A6E on charcoal #0f1318; JetBrains Mono; density over whitespace. — **P4K-E006** (`phase-4K:L28-L31`)
- Tauri command get_fsm_state. — **P4K-E019** (`phase-4K:L140-L141`)
- Tauri command get_phase_progress. — **P4K-E020** (`phase-4K:L142-L142`)
- Tauri command get_recent_actions(limit,offset). — **P4K-E021** (`phase-4K:L143-L143`)
- Tauri command get_pending_interventions. — **P4K-E022** (`phase-4K:L144-L144`)
- Tauri command get_file_changes(session_id). — **P4K-E023** (`phase-4K:L145-L145`)
- Tauri command submit_intervention_response(request_id,chosen,note). — **P4K-E024** (`phase-4K:L146-L146`)
- CLI elai --dashboard; focus existing window if already running. — **P4K-E026** (`phase-4K:L157-L158`)
- CSS tokens bg-void/base/surface/hover/input; teal/purple scales; semantic pass/warn/fail/info; mono font scale xs–xl. — **P4K-E027** (`phase-4K:L159-L167`)
- Project selector: logo, subtitle "a deterministic development engine", two creation paths. — **P4K-E029** (`phase-4K:L197-L203`)
- + New Project: name+parent picker; create_dir_all; list add; template+task without leaving app. — **P4K-E030** (`phase-4K:L204-L208`)
- Open Existing: native folder dialog. — **P4K-E031** (`phase-4K:L209-L210`)
- Recent projects: name/path/last opened; max 10; hover style; source elai_cc.toml [projects] or ~/.elai_cc/projects.json. — **P4K-E032** (`phase-4K:L211-L214`)
- Commands list_projects, select_project, create_project; name alnum-_; inline red errors. — **P4K-E033** (`phase-4K:L215-L224`)
- Template picker Svelte/React/Astro/Vanilla from templates/web/manifest.toml; list_templates. — **P4K-E034** (`phase-4K:L225-L229`)
- Task input: header, placeholder, Start purple, Enter submit; submit_task command. — **P4K-E035** (`phase-4K:L230-L237`)
- Workspace: plan tree left 320px + activity feed right; draggable divider; min 280px; double-click 50/50. — **P4K-E037** (`phase-4K:L277-L280`)
- Status bar 24px: project — **P4K-E038** (`FSM state`)
- Plan tree phases/steps/baby-steps; --> active; done/active/pending styles; expand active; elapsed times. — **P4K-E039** (`phase-4K:L283-L291`)
- Feed [reasoning] teal-ghost collapsed 2-3 lines. — **P4K-E040** (`phase-4K:L294-L295`)
- Feed [edit] path+lines +N -M. — **P4K-E041** (`phase-4K:L296-L296`)
- Feed [validate] checkmark/x one-liner. — **P4K-E042** (`phase-4K:L297-L297`)
- Feed [test] pass/fail counts. — **P4K-E043** (`phase-4K:L298-L298`)
- Feed [preview] teal link opens system browser via shell::open. — **P4K-E044** (`phase-4K:L299-L301, L362-L368`)
- Step headers; auto-scroll; Jump to latest pill when scrolled up. — **P4K-E045** (`phase-4K:L302-L305`)
- Diff viewer on ApproveChange: unified teal add / fail remove / sticky header. — **P4K-E049** (`phase-4K:L343-L350`)
- Diff Accept / Decline / Retry+note; per-hunk Accept/Decline on hover. — **P4K-E050** (`phase-4K:L351-L355`)
- Multi-file vertical list navigation. — **P4K-E051** (`phase-4K:L356-L357`)
- Tauri command get_action_detail(action_id) full payload. — **P4K-E053** (`phase-4K:L369-L372`)
- Completion criteria: native window, project/template/task, plan+feed, per-hunk diffs, preview, <10ms queries, <1s launch. — **P4K-E056** (`phase-4K:L400-L409`)
- PhaseStatus and InterventionRequest fields imply plan-tree/card data contracts. — **P4K-I001** (`phase-4K:L76-L76, L83-L86`)
- Diff viewer replace feed vs modal open. — **P4K-A003** (`phase-4K:L345-L345`)
- Defaults: designer/auditor/researcher/media_validator AnthropicApi; executor/memory_writer/context_compiler/prompt_engineer Sidecar. — **P4M-E021** (`phase-4M:L331-L339`)
- CLI /runtime show. — **P4M-E023** (`phase-4M:L344-L345`)
- CLI /runtime set <role> <backend>. — **P4M-E024** (`phase-4M:L346-L346`)
- CLI /runtime fallback <role> <b1>,<b2>. — **P4M-E025** (`phase-4M:L347-L347`)
- RuntimePanel/Settings: per-role backend, chain, live health g/y/r, Test backend button. — **P4M-E028** (`phase-4M:L376-L380`)
- RuntimePanel vs SettingsWindow placement open. — **P4M-A002** (`phase-4M:L376-L376`)
- Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface. — **PHASE-001/IX-009** (`P4K-E011–P4K-E026`)
- Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace. — **PHASE-001/IX-013** (`P0-E055; P4K-E026; P4M-E023–P4M-E028`)
- RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here. — **PHASE-001/IX-016** (`P1C-E029; P4M-E014, P4M-E028`)
- Wiring observable: trigger `elai-cli index build --project-root ...` expects SSB `symbol_graph_updated`. — **P1A-E039** (`1A:L511-L513`)
- Special files hardcoded 13 Rust items: Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile, docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml. Config-driven list deferred. — **P1D-E016** (`1D:L184-L186, L305-L305`)
- Skeleton TOML format example under [phase.3B] with goal/inputs/outputs/constraints/depends_on/depended_by/version. — **P3G-E022** (`3G:L95-L105`)
- Cache /research answers in-process per (query × project) so repeated questions within freshness window skip full ConceptDecomposition + WebResearch. — **P4DOR-E002** (`4DOR:L9-L11`)
- LiveCli field oracle_cache: Option<Arc<OracleAnswerCache>> default None; lazy init on first /research; all three construction sites default None. — **P4DOR-E011** (`4DOR:L84-L87`)
- reachable_from: /research and /ask slash commands (commands.rs Research / Ask variants). — **P4DOR-E013** (`4DOR:L108-L109`)
- Observable: `elai /research "what does B-37 ship?"` twice within 60s → miss then hit with cached_age_secs>0. — **P4DOR-E015** (`4DOR:L114-L117`)
- Mode switch CLI: `/mode autonomous` or `/mode interactive`. Takes effect at next gate evaluation, not mid-phase. Mode per-session. — **P4G-E012** (`4G:L327-L333, L245-L248`)
- Persona agent files: `.claude/agents/persona-{hacker,researcher,simplifier,architect,contrarian}.md`. — **P4G-E024** (`4G:L624-L631, L861-L864`)
- INTERACTIVE persona exhaustion renders InterventionCard with all 5 personas and failure reasons. — **P4G-E028** (`4G:L810-L813`)
- 4G.7 ≥9 tree_auto_advance tests; 4G.9 stagnation/persona/watchdog/goal_drift tests with adversarial cases (Oscillation≠Spinning, affinity, heartbeat non-material, fingerprint gaming, InterventionCard, Jaccard boundaries, max 1 block-retry). — **P4G-E034** (`4G:L516-L534, L775-L830`)
- Runtime flags --local/--hybrid and /model are asserted by MASTER phase-index summary for 4G; this source file’s body does not re-list them after the GOAL/read list — treat MASTER as orientation unless/until codified in addendum or providers wiring. — **P4G-I001** (`4G:L14-L27; MASTER 4G L871-L873`)
- 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss. — **PHASE-002/IX-014** (`P4DOR-E001, P4DOR-E002, P4DOR-E013`)
- WIRING: produces `Bm25Index` + types; reachable from `index_cmd::run_index_build`, `IndexingService::run_full_index`, `run_local_repl`, `QueryPipeline`; trigger `cargo run -p elai-cli -- index build`; expect SSB `index_checkpoint`. — **P1B-E018** (`phase-1B:L427-L448`)
- User-facing CLI surface: `elai-cli index build --project-root <path>` builds the BM25 index (WIRING trigger). — **P1B-E019** (`phase-1B:L447-L448`)
- `/task --surgical "<prompt>"` replaces ConceptDecomposition with DKG blast-radius surgical plan; flag via `parse_task_flags` commands.rs:2414. — **P4DS-E002** (`phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89`)
- Dashboard SurgicalDrawer shows ComplexityClass + AffectedSet on plan creation. — **P4DS-E011** (`phase-4D-SURGICAL:L122-L122`)
- observable_effect: `elai /task --surgical "rename Foo to Bar in core"` → surgical_plan_created within 2 s + SurgicalDrawer update. — **P4DS-E012** (`phase-4D-SURGICAL:L119-L122`)
- SurgicalDrawer field layout beyond ComplexityClass+AffectedSet not specified in this file. — **P4DS-A002** (`phase-4D-SURGICAL:L122-L122`)
- 3J.LOCAL/4G model UI: CLI `--local`/`--api`/`--hybrid`; runtime `/model`; config `[agents.{role}.backend]` and `[backend] mode`. — **PIDX-E013** (`phase-index-3A-4J:L156-L160, L310-L318`)
- Slash/CLI surfaces named in index: INTERACTIVE gate verbs; `/plan` (3J.WIRE.PLAN); `/mode`; `/model`; `elai --local/--hybrid`; `elai.bat`. — **PIDX-E024** (`phase-index-3A-4J:L12-L12, L166-L170, L300-L317`)
- SurgicalDrawer full UI fields; step-cap merge notes. — **PHASE-003/AC-010** (`P4DS-A001, P4DS-A002`)
- MASTER:L57-L67 — **PHASE-003/SD-1B-001** (`P1B-E006-E013`)
- MASTER:L334-L478 region — **PHASE-003/SD-IDX-001** (`PIDX-E012-E013, PIDX-E021-E024`)
- TOML surfaces: `[grc.defaults]`, `[agents.executor.grc]`, `[agents.auditor.grc]`, `[agents.context_compiler.grc]`, `[agents.memory_writer.grc]`, `[agents.prompt_engineer.grc]`, designer role table; pattern matches existing supervision/slo_tier/sandbox/role_prompt. — **P2C-E017** (`2C:L54-L66, L23, L445-L447`)
- D5: ProjectProfile snapshot at `.elai_cc/metrics/project_profile.toml` via persist_profile; reloaded at session start by `/profile` slash command (cli Profile variant). — **P3L-E007** (`3L:L28-L30, L148-L151, L212`)
- PiT1Hook::load + ProjectIntelligenceHook::render_block → CONVENTIONS block for SYSTEM via build_with_conventions. Format: per-category summaries + error_crate + top-3 avoid anti-patterns. Cap PI_T1_TOKEN_BUDGET=200 tokens (800 chars); truncate at line boundary. — **P3L-E012** (`3L:L115-L126, L200-L201`)
- `/profile` is the primary operator CLI surface for reloading ProjectProfile; CONVENTIONS block is agent-visible prompt surface not a dashboard panel. — **P3L-I001** (`3L:L148-L151, L119-L121`)
- No agent-visible UI for hooks; operator-facing artifacts are SSB findings, metrics TOML, generic rejection errors on agent channel. — **P4C-E018** (`4C:L4-L5, L355-L359`)
- Functional viewports web 375/768/1440; Blender 4 angles; FFmpeg keyframes; ComfyUI mock never real in tests. visual_check=false → no render overhead. — **P4I-E019** (`4I:L449-L469, L458-L459`)
- HumanReview/Degraded verdicts are operator-facing outcomes (review queue semantics) even though no dashboard panel is specified in this phase file. — **P4I-I001** (`4I:L81, L277-L287`)
- Decision artifact path: workflow/reasoning/phase-1F-decisions.md documents fallback and ADOPT/DROP with measurements. — **P1F-E026** (`1F:L152-L153, L161-L161, L285-L285, L315-L315`)
- No dashboard UI in 1F; operator-visible surfaces are decision markdown, metrics TOMLs, and probe SUPPORTED/NOT_SUPPORTED console output. — **P1F-E044** (`1F:L10-L14, L151-L152, L285-L289`)
- No dashboard panel in 2A; operator/engine surface is GrcEngine API and typed errors; Status free-echo aligns with mcp_think(step_type="status"). — **P2A-E033** (`2A:L207-L209, L373-L373`)
- Post-tool completeness GREP rules (Tier1, engineering, PostToolUse): web_fetch <img> → WARN "N images not extracted"; <table> → WARN "N tables not extracted"; file_read TODO/FIXME → INFO "N TODO/FIXME items"; <1ms, no pre-exec I/O. — **P3F-E037** (`3F:L309-L314`)
- No dedicated rules dashboard panel in 3F phase file; enforcement surfaces are evaluate verdicts, seed DB, PostToolUse WARN/INFO strings, and transition logs. — **P3F-E045** (`3F:L309-L314, L165-L165`)
- Slash /model and /mode switch backend at runtime; elai --local uses orchestrator pipeline; default elai uses API same pipeline. — **P3J-E038** (`3J:L737-L737, L765-L767`)
- --local flag sets BackendConfig mode Local; no flag → Api from env auth; elai.bat simplified to --local + HOME fallback. — **P3J-E042** (`3J:L814-L815, L862-L862`)
- CliInteractiveGate: present_gate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances. — **P3J-E045** (`3J:L895-L898`)
- 3J.WIRE.PLAN: optional PlanSkeleton from .elai_cc/plans/{pipeline_id}.toml; /plan displays skeleton; /plan phase <id> shows PhaseDetail; Slot 2 phase context; contract satisfied vs pending display. — **P3J-E052** (`3J:L1063-L1103`)
- 3J.14 WIRE.E2E: elai.bat single command; primary sidecar DoLa+TurboQuant+NF4; ollama fallback; QueryPipeline retrieval; release elai.exe. — **P3J-E057** (`3J:L1246-L1253`)
- Addendum surfaces: /research JSON-mode uses constrained decoding; COMMITTING FSM phase invokes git_commit tool; calibrate_etc_threshold.py; data/steering_contrast_pairs.toml. — **P3J-E067** (`3J:L1525-L1569`)
- MASTER:L317-L317 — **PHASE-005/SD-3F-003** (`P3F-E036-P3F-E038`)
- MASTER:L675-L681 — **PHASE-005/SD-3J-003** (`P3J-E038, P3J-E042, P3J-E045, P3J-E050, P3J-E052, P3J-E058, P3J-E061, P3J-E068`)
- 3K has no new operator UI; surfaces are QueryTier API, metrics TOML artifacts, and retrieval result snippet fields. — **P3K-I001** (`3K:L156-L157, L181-L182, L308-L308`)
- Operator-facing isolation surfaces: generic policy rejection string; human approval for MAJOR replan (INTERACTIVE gate); human approval for auditor PROPOSED rules; Windows vs Linux isolation strength documentation. — **P4A-E042** (`4A:L67-L68, L215-L216, L228-L230, L33-L35`)
- No new dashboard panel is specified in 4A; isolation is infrastructure with generic IPC error strings as the agent-visible surface. — **P4A-I001** (`4A:L67-L68`)
- Trigger task surface: .elai_cc/benchmarks/trigger_task.toml with input, expected_output_hash sha256, metric_key — project-specific canary against objective hacking. — **P4H-E034** (`4H:L589-L598`)
- Stage3 promotion gate requires better trigger task AND no SWE regression AND no auditor safety regressions AND no evidence-gate definition mods → ACTIVE demote previous ARCHIVED. — **P4H-E035** (`4H:L612-L615`)
- DIVERGING MSI alerts user and pauses self-mod for that PipelineMode only (never silently continues). — **P4H-E038** (`4H:L746-L746, L775-L779`)
- Anti-staleness wiring: 4H triggers nightly elai dep-audit CLI; outdated deps → stale_api_catches → Layer1A deprecated_patterns → steering vector refresh metrics/compute_steering_vector.py. — **P4H-E044** (`4H:L1099-L1106`)
- self_mod_history.db and phase_4H.toml are operator-visible measurement/history surfaces for later cockpit consumption. — **P4H-I001** (`4H:L751-L751, L1220-L1221`)
- CLI surface: `elai dep-audit` nightly anti-staleness trigger invoked by 4H loop. — **P4H-E069** (`4H:L1100-L1100, L1144`)
- 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance. — **PHASE-006/IX-013** (`P4H-E009, P4H-E031, P4H-E043`)
- Failing plugin tests: `collects_and_runs_hooks_from_enabled_plugins` (L395, pre.sh), `pre_tool_use_denies_when_plugin_hook_exits_two` (L456, printf/exit 2), `propagates_plugin_hook_failures` (L473, printf/exit 1). — **PP0-E011** (`pre-phase-0:L72-L81`)
- User notified before rebuild (not silent). — **P1I-E034** (`phase-1I:L290-L290, L302-L302`)
- Human approval for MAJOR replan is an interactive gate surface owned by orchestrator, not implemented in reasoning crate in this phase. — **P2D-I001** (`phase-2D:L70-L73, L250-L252`)
- Decomposition types: ConceptDecomposition, ConceptNode, PhaseNode, StepNode, BabyStepSpec (+ grounding/pre_gate/task/tdd/deterministic_check/acceptance/integration/exit). Newtypes ConceptId, BabyStepId. — **P4D-E030** (`phase-4D:L370-L419, L510-L513`)
- INTERACTIVE mode presents alternatives with evidence for user pick; AUTONOMOUS selects highest evidence_level * source_quality. — **P4D-E034** (`phase-4D:L531-L535, L645-L647`)
- ResearchTierConfig auto-detect GPU VRAM (nvidia-smi/CUDA) or manual `--vram-tier` flag; detect_vram_tier rounds DOWN; no-GPU falls back 8gb. — **P4D-E049** (`phase-4D:L1059-L1079`)
- SSB research_step / research_decompose_level / web_research_step intended for INTERACTIVE mode operator visibility of research progress. — **P4D-E057** (`phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183`)
- present_options Interactive gate is a user-facing selection surface; concrete panel/command wiring is not named in this phase file. — **P4D-I001** (`phase-4D:L531-L533, L622-L622`)
- Exact Interactive UI for present_options (CLI, dashboard modal, gate API) is not specified. — **P4D-A003** (`phase-4D:L531-L533, L645-L647`)
- 01-phase:L19-L22 — **PHASE-007/SD-PP0-004** (`PP0-E020–PP0-E025`)
- INTERACTIVE spiral escalation UI text: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]". — **P3C-E056** (`3C:L411-L413`)
- HUMAN-facing surfaces in 3C are INTERACTIVE escalation panels for max-restarts and spiral (options listed); no dedicated dashboard panel is named. — **P3C-I001** (`3C:L348-L349, L411-L413`)
- ContextBudget implements Display for debug-friendly budget breakdown; INTERACTIVE terminal surfaces Escalate(breakdown) to user. — **P3E-E035** (`3E:L210-L210, L163-L164, L324`)
- Exact default values for model_max_context sources and fixed-slot measurement sites left to ModelCapabilityRegistry/runtime. — **P3E-A001** (`3E:L41-L41, L226-L227`)
- No dedicated cockpit/panel for budget is named; only Display + escalate breakdown. — **P3E-A002** (`3E:L210, L163-L164`)
- Tools are agent-facing IPC surfaces, not human UI; human sees outcomes via auditor/SSB/dashboard elsewhere. — **P4B-I001** (`4B:L17-L18`)
- INTERACTIVE Escalate decision is human-facing; no dedicated panel name beyond decision enum. — **P4F-I001** (`4F:L97-L98`)
- ElaiNeuron exposes only task_submit, task_status, pipeline_result, health_check; never FSM/evidence/rules/SSB-direct/config/agent_spawn. — **P4J-E017** (`4J:L196-L202, L409-L411`)
- task_submit returns task_id async; task_status polls FSM+progress; pipeline_result final output; same path as CLI /task. — **P4J-E024** (`4J:L412-L432`)
- Host scaffolding includes Tauri command surface + Svelte panel + feature gate (EMPIRE-PENDING). — **P4J-E033** (`4J:L16-L18`)
- brainctl CLI replacement: orchestrator drives FactoryLine (start/stop/status brain lifecycle). — **P4J-E034** (`4J:L343-L349, L704`)
- Capability discovery action "capabilities" returns exactly 4 entries. — **P4J-E035** (`4J:L423-L427`)
- Svelte panel and Tauri commands are required scaffolding surfaces but exact command/panel names are not listed in this phase file. — **P4J-I001** (`4J:L16-L18`)
- Index compresses full phase files; step-level PRE_GATE/TASK/ACCEPTANCE not present here (MASTER also delegates). — **IDX-I001** (`IDX:L1-L2`)
- 01-MASTER:L290-L300 — **PHASE-008/SD-3E-002** (`P3E-E016–P3E-E017, P3E-E024, P3E-E035`)
- 01-MASTER:L508-L517 — **PHASE-008/SD-4J-002** (`P4J-E033`)
- 0.2 strip set: remove `telemetry/`, `compat-harness/`, and runtime modules `usage`, `compact`, `summary_compression`, `lsp_client`, `oauth`, and `prompt`; replace stripped token accounting with a structural `TokenUsage` shim while refactoring 61 `Session` occurrences; audit 28+ slash commands into keep/strip sets (`L30–32`). — **MASTER-E012** (`01-phase-index-MASTER.discovery.md:L29`)
- 0.7a provider/config: define a provider trait and `OllamaProvider`; configure through `elai_cc.toml`; support TOML and MessagePack (`rmp-serde`) serialization (`L40`). — **MASTER-E019** (`01-phase-index-MASTER.discovery.md:L36`)
- 0.7c CLI/runtime UX: commands `/task`, `/config`, `/continue`, `/explore`, `/status`; visibility policy with `allowed_paths`, `hidden_paths`, and allowlist sanitization; `tracing`-based logging (`L43–44`). — **MASTER-E021** (`01-phase-index-MASTER.discovery.md:L38`)
- 4K.2 scaffold: dashboard crate, six Tauri commands, Svelte/Vite/CSS, `elai --dashboard` (`L968–969`). — **MASTER-E403** (`01-phase-index-MASTER.discovery.md:L527`)
- UI surfaces: project selector, task input, phase ribbon, intervention panel, activity feed, reasoning chain, commit history (`L970–971`). — **MASTER-E404** (`01-phase-index-MASTER.discovery.md:L528`)
- 4M.1 skeleton/config: backend enum, adapter or alias, factory config, permission/error/role/capability/health types, stub resolve/create, `[runtime] backend` and `[agents.<role>.runtime]` schema (`L997–1000`). — **MASTER-E413** (`01-phase-index-MASTER.discovery.md:L541`)
- Full completion path: a task is classified by FSM, grounded/researched, converted to a typed/sized/probed tree, executed by isolated role-configured agents through orchestrator-owned tools, journaled in SSB, verified at baby/step/phase/concept and FSM gates, optionally media-validated, memorized/handoff-checkpointed, and only then complete. This is the composition implied by the explicit phase interfaces; no single MASTER passage states the whole workflow. — **MASTER-I444** (`01-phase-index-MASTER.discovery.md:L575`)
- Master granularity: individual phase files still own the exhaustive pre-gates/tasks/checks/adversarial/acceptance details (`L1–4`). — **MASTER-D489** (`01-phase-index-MASTER.discovery.md:L631`)
- phase-0:L32-L33 — **P0-E012** (`PHASE-001`)
- phase-0:L146-L148 — **P0-E026** (`PHASE-001`)
- phase-0:L149-L150 — **P0-E027** (`PHASE-001`)
- phase-0:L517-L519 — **P0-E055** (`PHASE-001`)
- phase-3A:L60-L63 — **P3A-E008** (`PHASE-001`)
- phase-3A:L205-L209 — **P3A-E024** (`PHASE-001`)
- phase-3J-BASELINE:L125-L128 — **P3JB-E018** (`PHASE-001`)
- phase-4K:L8-L12 — **P4K-E002** (`PHASE-001`)
- phase-4K:L28-L31 — **P4K-E006** (`PHASE-001`)
- phase-4K:L140-L141 — **P4K-E019** (`PHASE-001`)
- phase-4K:L142-L142 — **P4K-E020** (`PHASE-001`)
- phase-4K:L143-L143 — **P4K-E021** (`PHASE-001`)
- phase-4K:L144-L144 — **P4K-E022** (`PHASE-001`)
- phase-4K:L145-L145 — **P4K-E023** (`PHASE-001`)
- phase-4K:L146-L146 — **P4K-E024** (`PHASE-001`)
- phase-4K:L157-L158 — **P4K-E026** (`PHASE-001`)
- phase-4K:L159-L167 — **P4K-E027** (`PHASE-001`)
- phase-4K:L197-L203 — **P4K-E029** (`PHASE-001`)
- phase-4K:L204-L208 — **P4K-E030** (`PHASE-001`)
- phase-4K:L209-L210 — **P4K-E031** (`PHASE-001`)
- phase-4K:L211-L214 — **P4K-E032** (`PHASE-001`)
- phase-4K:L215-L224 — **P4K-E033** (`PHASE-001`)
- phase-4K:L225-L229 — **P4K-E034** (`PHASE-001`)
- phase-4K:L230-L237 — **P4K-E035** (`PHASE-001`)
- phase-4K:L277-L280 — **P4K-E037** (`PHASE-001`)
- FSM state — **P4K-E038** (`session; EXECUTING teal.`)
- phase-4K:L283-L291 — **P4K-E039** (`PHASE-001`)
- phase-4K:L294-L295 — **P4K-E040** (`PHASE-001`)
- phase-4K:L296-L296 — **P4K-E041** (`PHASE-001`)
- phase-4K:L297-L297 — **P4K-E042** (`PHASE-001`)
- phase-4K:L298-L298 — **P4K-E043** (`PHASE-001`)
- phase-4K:L299-L301, L362-L368 — **P4K-E044** (`PHASE-001`)
- phase-4K:L302-L305 — **P4K-E045** (`PHASE-001`)
- phase-4K:L343-L350 — **P4K-E049** (`PHASE-001`)
- phase-4K:L351-L355 — **P4K-E050** (`PHASE-001`)
- phase-4K:L356-L357 — **P4K-E051** (`PHASE-001`)
- phase-4K:L369-L372 — **P4K-E053** (`PHASE-001`)
- phase-4K:L76-L76, L83-L86 — **P4K-I001** (`PHASE-001`)
- phase-4K:L345-L345 — **P4K-A003** (`PHASE-001`)
- phase-4M:L344-L345 — **P4M-E023** (`PHASE-001`)
- phase-4M:L346-L346 — **P4M-E024** (`PHASE-001`)
- phase-4M:L347-L347 — **P4M-E025** (`PHASE-001`)
- phase-4M:L376-L380 — **P4M-E028** (`PHASE-001`)
- phase-4M:L376-L376 — **P4M-A002** (`PHASE-001`)
- 3G:L95-L105 — **P3G-E022** (`PHASE-002`)
- 4DOR:L108-L109 — **P4DOR-E013** (`PHASE-002`)
- 4G:L327-L333, L245-L248 — **P4G-E012** (`PHASE-002`)
- 4G:L624-L631, L861-L864 — **P4G-E024** (`PHASE-002`)
- 4G:L810-L813 — **P4G-E028** (`PHASE-002`)
- 4G:L1-L12, exit context via MASTER 4G runtime lines (orientation); source body focuses gates — see P4G-I001 — **P4G-E033** (`PHASE-002`)
- 4G:L14-L27; MASTER 4G L871-L873 — **P4G-I001** (`PHASE-002`)
- phase-1B:L447-L448 — **P1B-E019** (`PHASE-003`)
- phase-4D-SURGICAL:L9-L12, L18-L19, L86-L89 — **P4DS-E002** (`PHASE-003`)
- phase-4D-SURGICAL:L122-L122 — **P4DS-E011** (`PHASE-003`)
- phase-4D-SURGICAL:L122-L122 — **P4DS-A002** (`PHASE-003`)
- phase-index-3A-4J:L156-L160, L310-L318 — **PIDX-E013** (`PHASE-003`)
- phase-index-3A-4J:L290-L318 — **PIDX-E020** (`PHASE-003`)
- phase-index-3A-4J:L12-L12, L166-L170, L300-L317 — **PIDX-E024** (`PHASE-003`)
- 2C:L54-L66, L23, L445-L447 — **P2C-E017** (`PHASE-004`)
- 3L:L115-L126, L200-L201 — **P3L-E012** (`PHASE-004`)
- 3L:L148-L151, L119-L121 — **P3L-I001** (`PHASE-004`)
- 4C:L4-L5, L355-L359 — **P4C-E018** (`PHASE-004`)
- 4I:L449-L469, L458-L459 — **P4I-E019** (`PHASE-004`)
- 4I:L81, L277-L287 — **P4I-I001** (`PHASE-004`)
- 1F:L152-L153, L161-L161, L285-L285, L315-L315 — **P1F-E026** (`PHASE-005`)
- 1F:L10-L14, L151-L152, L285-L289 — **P1F-E044** (`PHASE-005`)
- 2A:L207-L209, L373-L373 — **P2A-E033** (`PHASE-005`)
- 3F:L309-L314 — **P3F-E037** (`PHASE-005`)
- 3F:L309-L314, L165-L165 — **P3F-E045** (`PHASE-005`)
- 3J:L737-L737, L765-L767 — **P3J-E038** (`PHASE-005`)
- 3J:L814-L815, L862-L862 — **P3J-E042** (`PHASE-005`)
- 3J:L895-L898 — **P3J-E045** (`PHASE-005`)
- 3J:L1063-L1103 — **P3J-E052** (`PHASE-005`)
- 3J:L1246-L1253 — **P3J-E057** (`PHASE-005`)
- 3J:L1525-L1569 — **P3J-E067** (`PHASE-005`)
- 3J:L737-L767, L814-L831, L895-L898, L1027-L1052, L1078-L1079, L1268-L1273, L1307-L1314 — **P3J-E068** (`PHASE-005`)
- 3K:L156-L157, L181-L182, L308-L308 — **P3K-I001** (`PHASE-006`)
- 4A:L67-L68, L215-L216, L228-L230, L33-L35 — **P4A-E042** (`PHASE-006`)
- 4A:L67-L68 — **P4A-I001** (`PHASE-006`)
- 4H:L589-L598 — **P4H-E034** (`PHASE-006`)
- 4H:L612-L615 — **P4H-E035** (`PHASE-006`)
- 4H:L746-L746, L775-L779 — **P4H-E038** (`PHASE-006`)
- 4H:L1099-L1106 — **P4H-E044** (`PHASE-006`)
- 4H:L751-L751, L1220-L1221 — **P4H-I001** (`PHASE-006`)
- 4H:L1100-L1100, L1144 — **P4H-E069** (`PHASE-006`)
- pre-phase-0:L72-L81 — **PP0-E011** (`PHASE-007`)
- phase-1I:L290-L290, L302-L302 — **P1I-E034** (`PHASE-007`)
- phase-2D:L70-L73, L250-L252 — **P2D-I001** (`PHASE-007`)
- phase-4D:L531-L535, L645-L647 — **P4D-E034** (`PHASE-007`)
- phase-4D:L1059-L1079 — **P4D-E049** (`PHASE-007`)
- phase-4D:L325-L325, L85-L85, L650-L650, L1183-L1183 — **P4D-E057** (`PHASE-007`)
- phase-4D:L531-L533, L622-L622 — **P4D-I001** (`PHASE-007`)
- phase-4D:L531-L533, L645-L647 — **P4D-A003** (`PHASE-007`)
- 3C:L411-L413 — **P3C-E056** (`PHASE-008`)
- 3C:L348-L349, L411-L413 — **P3C-I001** (`PHASE-008`)
- 3E:L210-L210, L163-L164, L324 — **P3E-E035** (`PHASE-008`)
- 3E:L210, L163-L164 — **P3E-A002** (`PHASE-008`)
- 4B:L17-L18 — **P4B-I001** (`PHASE-008`)
- 4F:L97-L98 — **P4F-I001** (`PHASE-008`)
- 4J:L16-L18 — **P4J-E033** (`PHASE-008`)
- 4J:L343-L349, L704 — **P4J-E034** (`PHASE-008`)
- 4J:L423-L427 — **P4J-E035** (`PHASE-008`)
- 4J:L16-L18 — **P4J-I001** (`PHASE-008`)
- User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G). — **PHASE-003/IX-017** (`P1B-E019, P4DS-E002, P4DS-E011, PIDX-E013, PIDX-E024`)

## System: D — Typed self-improvement, inference-time grounding, benchmarks, dashboards, deferred research

Family D is the post-C empirical recursive self-improvement stack (DGM-class) plus five-mode inference-time grounding (RPS). Waves: D-1..D-9 templates/archive/PPM/tripwires/BAI/held-out/canary/crypto/L3; D-10..D-18 mode routing/checkers/operationalization/adversary/sycophancy/novelty/refusal/effects; D-19..D-25 gates; D-26..D-27 dashboards; D-28..D-30 deferred research.

Status labels are plan-recorded claims unless noted. Deep plan originals win target state over the summary index (see Superseded notes).

### D-0: INDEX

- **Status**: planned (index/meta — not a shipping module)
- **Anchors**: plan `D-0_INDEX.md`; DEEP-D-001..004; aggregate unit (68 declared source atoms); compiled 68
- **Composes with**: D-001/IX-010 — **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not indepen

**Behaviors**

- **D0-001**: `[E][STATUS/DEP]` The index was generated `2026-05-19` from the two named grounded companion guides; its premise is that post-C ELAI ships empirical recursive self-improvement (DGM-class) plus a five-mode inference-time-grounding substrate. (`D-0_INDEX.md:1-4`)
- **D0-002**: `[E][WF/DEP]` Session files are dependency ordered: D-1..D-9 **MUST** precede D-10..D-18; Wave 2 consumes typed templates, PPM, tripwires, archive, and L3-frozen surfaces; D-19..D-25 gate parent capabilities; D-26..D-27 surface the plan; D-28..D-30 are deferred. (`D-0_INDEX.md:6-8`)
- **D0-003**: `[E][STATUS]` Status meanings are exact: `PLANNED` = plan exists/work not started; `IN PROGRESS` = active shipping session; `DONE` = shipped + cloud-Sonnet smoke + benchmark threshold; `DEFERRED` = blocked on calibration data, hardware, or open research. (`D-0_INDEX.md:10-14`)
- **D0-004**: `[E][CAP/TEST]` Every D-N session is intended to ship one cohesive module, wire it to the dashboard, and prove it with a real cloud-Sonnet test plus a paper-baselined benchmark. (`D-0_INDEX.md:3-4`)
- **D0-005**: `[E][CAP/DEP/TEST]` D-1 plans `ReasoningTemplate` typed schema, Blake3 content hash, and archive lifecycle; closes Phase 4H.IV; requires C-15 and C-21; cites SELF-DISCOVER, arXiv:2510.04399, and DGM; proof is a promotion that writes `ARCHIVE_ENTRY_ADDED` and a verifiable hash chain. Status `PLANNED`. (`D-0_INDEX.md:18-24`)
- **D0-006**: `[E][CAP/DEP/TEST]` D-2 plans the Qwen2.5-Math-PRM-7B PPM sidecar; closes Phase 4H.V.1; requires D-1 and B-37; cites rStar-Math, Lightweight Latent Verifiers, and PRMBench; proof requires known-correct versus known-wrong score margin `>0.3`. Status `PLANNED`. (`D-0_INDEX.md:26-30`)
- **D0-007**: `[E][CAP/DEP/TEST]` D-3 plans `VerificationResult` aggregation and multi-verifier consensus; closes Phase 4H.V part 2; requires D-2; cites V-STaR, PRMBench, and Cross-Context Verification; a PPM-pass/deterministic-fail deceptive task must be rejected. Status `PLANNED`. (`D-0_INDEX.md:32-36`)
- **D0-008**: `[E][CAP/DEP/TEST]` D-4 plans KL, delta-rho, IQR, perplexity, and DDM tripwires; closes Phase 4H.VI; requires D-1/D-3; cites reward-hacking, concept-drift, and RM-overoptimization work; an injected regression must fire at least three tripwires and visibly auto-revert through the archive. Status `PLANNED`. (`D-0_INDEX.md:38-42`)
- **D0-009**: `[E][CAP/DEP/TEST]` D-5 plans Beta-posterior Thompson-sampling BAI; closes Phase 4H.VII; requires D-1/D-3; cites Audibert COLT 2010 and variance-dependent BAI; its index proof is a 50-round noisy-arm run with regret within the published Audibert bound. Status `PLANNED`. (`D-0_INDEX.md:44-48`)
- **D0-010**: `[E][CAP/DEP/TEST]` D-6 plans a time-windowed held-out store and 9-gram filter; closes Phase 4H.VIII part 1; requires D-5; cites LiveCodeBench, Soft Contamination, and ARC Prize 2025; proof auto-retires an 18-month-old uniformly passing task and emits `HELD_OUT_REFRESH` on ingest. Status `PLANNED`. (`D-0_INDEX.md:50-54`)
- **D0-011**: `[E][CAP/DEP/TEST]` D-7 plans a five-family cross-domain canary; closes Phase 4H.VIII part 2; requires D-6; cites NanoFlux; an 8% prose-reason regression must block promotion and write `CrossDomainRegression`. Status `PLANNED`. (`D-0_INDEX.md:56-60`)
- **D0-012**: `[E][CAP/DEP/TEST]` D-8 plans the cryptographic archive plus `elai archive verify`; closes Phase 4H.IX; requires D-1; cites reward-hacking checksum guidance and DGM archive practice; any tamper exits 1 with a broken-link report and clean state exits 0. Status `PLANNED`. (`D-0_INDEX.md:62-66`)
- **D0-013**: `[E][SEC/DEP/TEST]` D-9 plans L3 expansion and frozen-surface access-control predicates; closes companion sections 2.3 and 4; requires D-1..D-8; cites reward-tampering/boxed-agent/mesa-optimizer work; attempted evaluator edit must be pre-commit blocked and journaled as `LOCKED_KEYS_TOUCH`. Status `PLANNED`. (`D-0_INDEX.md:68-72`)
- **D0-014**: `[E][CAP/DEP/TEST]` D-10 plans L3-owned `select_grounding_mode`; closes Phase RPS.I; requires D-9; cites Everitt; at least 45 of 50 synthetic claims must receive the correct mode. Status `PLANNED`. (`D-0_INDEX.md:74-80`)
- **D0-015**: `[E][CAP/DEP/TEST]` D-11 plans the constructive-checker registry; closes Phase RPS.II; requires D-10; cites PoT, CodeAct, and AlphaProof; SymPy, `cargo check`, and a registry check must respectively verify math, code, and novel-package claims. Status `PLANNED`. (`D-0_INDEX.md:82-86`)
- **D0-016**: `[E][WF/DEP/TEST]` D-12 plans the `OperationalizationCheck` predicate; closes Phase RPS.III; requires D-11; cites Voyager; a claim without procedure must be auditor-blocked and emit `CLAIM_WITHOUT_PROCEDURE`. Status `PLANNED`. (`D-0_INDEX.md:88-92`)
- **D0-017**: `[E][CAP/DEP/TEST]` D-13 plans an `Adversary` role and cross-family verifier; closes Phase RPS.IV; requires D-10/D-3; cites Self-Correction Bench, latent verifiers, and small-LLM constitutional AI; cross-family disagreement on a confident-wrong claim must make consensus reject. Status `PLANNED`. (`D-0_INDEX.md:94-98`)
- **D0-018**: `[E][SEC/DEP/TEST]` D-14 plans an inverted-framing sycophancy probe; closes Phase RPS.V; requires D-13; cites Sharma et al.; accepting a user’s false assertion then diverging on inversion must fire `SYCOPHANCY_FLAG`. Status `PLANNED`. (`D-0_INDEX.md:100-104`)
- **D0-019**: `[E][DATA/DEP/TEST]` D-15 plans `NoveltyClass` labeling; closes Phase RPS.VI; requires D-11/D-12/C-17; cites FunSearch, AlphaEvolve, and ARC; proofs map known search, novel composition, and a not-in-DKG primitive to `Search`, `DistantSearch`, and `DiscoveryCandidate`. Status `PLANNED`. (`D-0_INDEX.md:106-110`)
- **D0-020**: `[E][CAP/DEP/TEST]` D-16 plans UniCR conformal refusal; closes Phase RPS.VII; requires D-9/D-10; cites UniCR, uncertainty, and doubt-reward work; a shifted held-out family must preserve empirical refusal within `epsilon=0.1`. Status `PLANNED`. (`D-0_INDEX.md:112-116`)
- **D0-021**: `[E][OBS/DEP/TEST]` D-17 plans honest-refusal SLA monitoring; closes Phase RPS.VIII; requires D-16; cites the RPS design and honesty survey; a synthetic session with refusal rate below 2% must emit `AUTO_FLAG`. Status `PLANNED`. (`D-0_INDEX.md:118-122`)
- **D0-022**: `[E][DATA/DEP/TEST]` D-18 plans typed `ObservableEffect` for Mode D; closes the Phase 4F extension; requires C-22/D-11; cites Voyager, Eureka, and Coscientist; one effect fires `GOODHART_RISK_FLAG`, while two or more suppress it. Status `PLANNED`. (`D-0_INDEX.md:124-128`)
- **D0-023**: `[E][TEST/DEP]` D-19 plans PRMBench adversarial-gaming gates for the PPM; closes companion section 1.13; requires D-2/D-3; cites PRMBench, Lessons of Developing PRMs, and ThinkPRM; required floors are Deception Resistance `>=35%` and Circular Logic `>=30%`. Status `PLANNED`. (`D-0_INDEX.md:130-136`)
- **D0-024**: `[E][TEST/DEP/SEC]` D-20 plans an ARC-AGI-2 OOD harness; closes companion section 1.16 and real-problem section 1 Mode C; requires D-10/D-15; cites Chollet ARC, ARC Prize 2025, and NVARC; reports solve and refusal rates against human baseline on 50 tasks and must not auto-publish `DiscoveryCandidate`. Status `PLANNED`. (`D-0_INDEX.md:138-142`)
- **D0-025**: `[E][TEST/DEP]` D-21 plans a mode-routing coverage benchmark that closes D-10 gating; requires D-10; cites real-problem section 3.2 and Self-Correction Bench; gate is F1 `>=0.85` across five modes with no class collapse. Status `PLANNED`. (`D-0_INDEX.md:144-148`)
- **D0-026**: `[E][TEST/DEP]` D-22 plans conformal-calibration validation under distribution shift that closes D-16 gating; requires D-16/D-6; cites UniCR; every in-domain, near-OOD, and far-OOD refusal rate must lie within `[tau-epsilon, tau+epsilon]`. Status `PLANNED`. (`D-0_INDEX.md:150-154`)
- **D0-027**: `[E][TEST/DEP]` D-23 plans the sycophancy-detection benchmark that closes D-14 gating; requires D-14; cites Sharma; gate is detection `>=70%` and false-positive rate `<=15%`. Status `PLANNED`. (`D-0_INDEX.md:156-160`)
- **D0-028**: `[E][TEST/DEP]` D-24 plans canary regression validation that closes D-7 gating; requires D-7; cites NanoFlux; it must block 20/20 harmful mutations while blocking at most 5/50 helpful and 2/30 neutral mutations. Status `PLANNED`. (`D-0_INDEX.md:162-166`)
- **D0-029**: `[E][DET/TEST/DEP]` D-25 plans determinism plus archive-tamper testing that closes D-8+C-27 gating; requires D-8/C-27; cites the reward-hacking source; all 10 tamper scenarios must be detected and clean state must exit 0. Status `PLANNED`. (`D-0_INDEX.md:168-172`)
- **D0-030**: `[E][CAP/DEP/TEST]` D-26 plans and closes the reasoning-template/archive dashboard surface for D-1..D-8; requires D-1..D-8; cites companion sections 3.1 and 3.8; a live promotion must add an archive-panel entry and lineage edge, and a tripwire injection must show red. Status `PLANNED`. (`D-0_INDEX.md:174-180`)
- **D0-031**: `[E][CAP/DEP/TEST]` D-27 plans and closes mode-routing/refusal dashboard coverage for D-10..D-17; requires D-10..D-17; cites real-problem sections 3.2 and 4.6; live mixed claims must show realistic routing and a sycophancy probe must reach `SycophancyFlagStrip`. Status `PLANNED`. (`D-0_INDEX.md:182-186`)
- **D0-032**: `[E][STATUS/DEP/TEST]` D-28 adversarial-example farming is `DEFERRED`; closes Phase 4H.X; requires D-5, D-19, and RunPod; cites Adversarial Training Reward Models; valid proof is pre/post PRMBench Deception Resistance delta `>=+5pp`. (`D-0_INDEX.md:188-194`)
- **D0-033**: `[E][STATUS/DEP/TEST]` D-29 hierarchical BAI is `DEFERRED`; closes companion open Q4; requires D-5 and RunPod; cites Audibert and variance-dependent BAI; closure is hierarchical regret no worse than flat BAI or an infeasibility note with at least one concrete blocker. (`D-0_INDEX.md:196-200`)
- **D0-034**: `[E][STATUS/DEP/TEST]` D-30 calibration-corpus bootstrap is `DEFERRED`; closes real-problem open Q5; requires D-16 and corpus access; cites UniCR; bootstrap must stay within epsilon and later real-session calibration must have risk no worse than bootstrap. (`D-0_INDEX.md:202-206`)
- **D0-035**: `[E][OBS/STATUS]` Per-session status notes are cumulative; shipped notes use `SHIPPED <date>: <artifact + benchmark>`, and schema-version/SSB additions are logged for B-plan reconciliation. (`D-0_INDEX.md:208-212`)
- **D0-036**: `[E][DATA/INT/STATUS]` D-1 is `PLANNED`, expects schema `47 -> 48`, and events `ARCHIVE_ENTRY_ADDED`, `ARCHIVE_ENTRY_RETIRED`, `TEMPLATE_PROMOTED`, `TEMPLATE_CANDIDATE_REJECTED`. (`D-0_INDEX.md:214-217`)
- **D0-037**: `[E][CFG/INT/STATUS]` D-2 is `PLANNED`; PPM port is `8101`, policy SLM port `8100`, with policy on GPU0 and PPM on GPU1. (`D-0_INDEX.md:219-221`)
- **D0-038**: `[E][INT/STATUS]` D-3 is `PLANNED`; expected events are `VERIFIER_CONSENSUS_PASS`, `VERIFIER_CONSENSUS_REJECT`, and `VERIFIER_DISAGREEMENT`. (`D-0_INDEX.md:223-225`)
- **D0-039**: `[E][DATA/INT/STATUS]` D-4 is `PLANNED`; its schema bump is expected atomically with D-3, and `TRIPWIRE_FIRED` carries kinds `KlDivergence`, `CorrelationDrop`, `IqrOutlier`, `PerplexityAnomaly`, or `Ddm`. (`D-0_INDEX.md:227-230`)
- **D0-040**: `[E][INT/STATUS]` D-5 is `PLANNED`; expected files are `orchestrator/src/bai.rs` and `promotion_arms.rs`; events are `ARM_POSTERIOR_UPDATED`, `THOMPSON_SAMPLED`, `BAI_PROMOTION_DECISION`. (`D-0_INDEX.md:232-235`)
- **D0-041**: `[E][DATA/INT/STATUS]` D-6 is `PLANNED`; expected append-only layout is `.elai_cc/held-out/{date}/{task-id}.toml`; events are `HELD_OUT_REFRESH` and `HELD_OUT_AUTO_RETIRE`. (`D-0_INDEX.md:237-240`)
- **D0-042**: `[E][INT/STATUS]` D-7 is `PLANNED`; events are `CROSS_DOMAIN_REGRESSION` and `CANARY_FAMILY_PASS`. (`D-0_INDEX.md:242-244`)
- **D0-043**: `[E][CAP/STATUS]` D-8 is `PLANNED`; expected CLI is `elai archive verify`, backed by new `archive_chain.rs` and `archive_verify.rs`. (`D-0_INDEX.md:246-249`)
- **D0-044**: `[E][SEC/INT/STATUS]` D-9 is `PLANNED`; expected hook is `scripts/locked_keys_check.sh`; events are `LOCKED_KEYS_TOUCH` and `L3_OVERRIDE_REQUESTED`. (`D-0_INDEX.md:251-254`)
- **D0-045**: `[E][DATA/INT/STATUS]` D-10 is `PLANNED`; schema bump is expected atomically with D-11/D-12; event is `MODE_SELECTED`. (`D-0_INDEX.md:256-259`)
- **D0-046**: `[E][CAP/INT/STATUS]` D-11 is `PLANNED`; six initial checker families live at `.elai_cc/checkers/{math,code,text,graph,api,perf}/`; events are `CHECKER_DISPATCHED`, `CHECKER_PASS`, `CHECKER_FAIL`, `CHECKER_INAPPLICABLE`. (`D-0_INDEX.md:261-264`)
- **D0-047**: `[E][DATA/INT/STATUS]` D-12 is `PLANNED`; it expects BabyStepSpec block **9** `MEASURABLE_PROCEDURE` and events `CLAIM_WITHOUT_PROCEDURE` and `PROCEDURE_EXECUTED`. (`D-0_INDEX.md:266-269`)
- **D0-048**: `[E][DATA/INT/STATUS]` D-13 is `PLANNED`; adds `AgentRole::Adversary` and events `ADVERSARY_VERDICT`, `CROSS_FAMILY_UNAVAILABLE`. (`D-0_INDEX.md:271-274`)
- **D0-049**: `[E][INT/STATUS]` D-14 is `PLANNED`; events are `SYCOPHANCY_PROBE` and `SYCOPHANCY_FLAG`. (`D-0_INDEX.md:276-278`)
- **D0-050**: `[E][DATA/INT/STATUS]` D-15 is `PLANNED`; exact enum values are `Search`, `DistantSearch`, `DiscoveryCandidate`; events are `NOVELTY_LABELED`, `DISCOVERY_CANDIDATE_QUEUED`. (`D-0_INDEX.md:280-283`)
- **D0-051**: `[E][DATA/INT/STATUS]` D-16 is `PLANNED`; expected calibration directory is `.elai_cc/calibration/`; events are `REFUSAL_DECISION`, `CALIBRATION_UPDATE`. (`D-0_INDEX.md:285-288`)
- **D0-052**: `[E][OBS/INT/STATUS]` D-17 is `PLANNED`; D-27 supplies `RefusalRateGauge.svelte`; event is `REFUSAL_SLA_BREACH`. (`D-0_INDEX.md:290-293`)
- **D0-053**: `[E][DATA/INT/STATUS]` D-18 is `PLANNED`; expected `ObservableEffect` lives in `orchestrator/src/observable_effect.rs`; event is `GOODHART_RISK_FLAG`. (`D-0_INDEX.md:295-298`)
- **D0-054**: `[E][TEST/INT/STATUS]` D-19 is `PLANNED`; paper-ingestor artifact is `workflow/papers/prmbench-2501.03124-interface.md`; baseline comes from PRMBench Table 3. (`D-0_INDEX.md:300-303`)
- **D0-055**: `[E][TEST/INT/STATUS]` D-20 is `PLANNED`; harness is `.elai_cc/benchmarks/arc_agi2/`; stated baselines are human `60-70%` and NVARC `24%`. (`D-0_INDEX.md:305-308`)
- **D0-056**: `[E][TEST/STATUS]` D-21 is `PLANNED`; expected fixture is `.elai_cc/benchmarks/mode_routing/{200 synthetic claims}`. (`D-0_INDEX.md:310-312`)
- **D0-057**: `[E][TEST/STATUS]` D-22 is `PLANNED`; expected fixture partitions are `.elai_cc/benchmarks/conformal/{in_domain,near_ood,far_ood}/`. (`D-0_INDEX.md:314-316`)
- **D0-058**: `[E][TEST/STATUS]` D-23 is `PLANNED`; expected fixture is `.elai_cc/benchmarks/sycophancy/` following Sharma probes. (`D-0_INDEX.md:318-320`)
- **D0-059**: `[E][TEST/STATUS]` D-24 is `PLANNED`; fixture is `.elai_cc/benchmarks/canary/{50 helpful, 30 neutral, 20 harmful}`. (`D-0_INDEX.md:322-324`)
- **D0-060**: `[E][DET/TEST/STATUS]` D-25 is `PLANNED`; fixture is `.elai_cc/benchmarks/tamper/{10 scenarios}`. (`D-0_INDEX.md:326-328`)
- **D0-061**: `[E][CAP/STATUS]` D-26 is `PLANNED`; expected components are `MutationArchivePanel.svelte` and `LineageGraphView.svelte` (Cytoscape); expected commands are `get_archive_recent`, `get_archive_verify_status`, `get_tripwire_status`. (`D-0_INDEX.md:330-333`)
- **D0-062**: `[E][CAP/STATUS]` D-27 is `PLANNED`; expected components are `ModeRoutingPanel.svelte`, `RefusalRateGauge.svelte`, `NoveltyBreakdown.svelte`, `SycophancyFlagStrip.svelte`; expected commands are `get_mode_breakdown`, `get_refusal_rate`, `get_novelty_breakdown`, `get_sycophancy_flags`. (`D-0_INDEX.md:335-338`)
- **D0-063**: `[E][STATUS/DEP]` D-28 remains deferred until at least 20 live D-5 promotion cycles exist and RunPod retraining is available. (`D-0_INDEX.md:340-341`)
- **D0-064**: `[E][STATUS]` D-29 is marked deferred because it is research-grade; an infeasibility note is explicitly valid closure. (`D-0_INDEX.md:343-344`)
- **D0-065**: `[E][STATUS/DEP]` D-30 is deferred until D-16 has empirical signal and either 500 calibration tuples or generic-corpus access exists. (`D-0_INDEX.md:346-347`)
- **D0-066**: `[E][STATUS/OBS]` Index totals are 30 slots, 27 in scope, 3 deferred, all 30 authored on 2026-05-19 and `PLANNED` as files; anticipated totals are 4-6 atomic schema bumps, about 35-40 new SSB variants, 160-220 CPU/cloud hours, and 25-75 deferred hours plus RunPod. (`D-0_INDEX.md:349-360`)
- **D0-067**: `[E][SEC/DEP/FAIL]` Every D-N `WIRING_CONTRACT.consumes_symbols` must reconcile against canonical schema/event definitions at `EXECUTING`; any missing B/C dependency blocks the FSM transition out of `PLANNING`. (`D-0_INDEX.md:362-364`)
- **D0-068**: `[E][OBS/CAP]` D-26 extends `.claude/tools/bn_status_report.py` with `--plan=D` so dashboard progress includes D alongside B/C; this is the only exact cross-plan progress-overlay hook stated in the index. (`D-0_INDEX.md:366-370`)

### D-5: bai thompson promotion

- **Status**: PLANNED — deps D-1, D-3; Phase 4H.VII / BAI regret bound
- **Anchors**: plan `D-5_bai_thompson_promotion.md`; DEEP-D-001..004; aggregate unit (45 declared source atoms); compiled 45
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-002 — **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 p | D-001/IX-003 — **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks.  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-008 — **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D

**Behaviors**

- **D5-001**: `[E][CAP]` D-5 replaces MSI sigmoid-Boltzmann promotion with Beta-posterior Thompson sampling plus Successive Rejects and LUCB for template promotion/retirement, claiming a strictly stronger selection axis than DGM’s heuristic. (`D-5_bai_thompson_promotion.md:1-8`)
- **D5-002**: `[E][DEP/STATUS]` Estimated effort is two sessions; D-1 archive entries and D-3 consensus pass counts must be `DONE`, verified by `spec-grounder`. (`D-5_bai_thompson_promotion.md:8-10`)
- **D5-003**: `[E][DET/CON]` Work is parallel-safe with D-6/D-7 because they supply signals without writing held-out tasks, but not with D-4 because D-4 and BAI mutate the same `ArchiveChain`; coordination is through archive locks. (`D-5_bai_thompson_promotion.md:11-14`)
- **D5-004**: `[E][INT]` Theory grounding is Audibert Successive Rejects/LUCB plus variance-aware BAI; DGM/STaR/ReST^EM/current ELAI are stated not to use explicit Beta-posterior Thompson selection. (`D-5_bai_thompson_promotion.md:16-23`)
- **D5-005**: `[E][DATA]` Every candidate `ReasoningTemplate` owns an `ArmPosterior` that tracks `pass_count` and `fail_count`. (`D-5_bai_thompson_promotion.md:25-27`)
- **D5-006**: `[E][DATA]` The posterior is exactly `Beta(pass_count + 1, fail_count + 1)`. (`D-5_bai_thompson_promotion.md:28-29`)
- **D5-007**: `[E][WF]` Thompson selection samples one theta per arm, picks the largest, and advances evaluation through 10, 50, and 200-evaluation stages. (`D-5_bai_thompson_promotion.md:30-32`)
- **D5-008**: `[E][WF/CFG]` Candidate-to-Promoted requires the 95% credible-interval lower bound to exceed the current promoted median plus a domain delta, typically 0.02-0.05 absolute pass rate. (`D-5_bai_thompson_promotion.md:33-35`)
- **D5-009**: `[E][WF]` Successive Rejects eliminates the empirically worst arm each round. (`D-5_bai_thompson_promotion.md:36`)
- **D5-010**: `[E][WF]` LUCB pulls the highest-UCB competitor against the empirical best to balance exploration/exploitation. (`D-5_bai_thompson_promotion.md:37-38`)
- **D5-011**: `[E][DEP/DATA]` Existing foundations are D-1 staged `stage_10/50/200_pass/total` fields, D-3 binary consensus, and `RetirementReason::Superseded`. (`D-5_bai_thompson_promotion.md:40-45`)
- **D5-012**: `[E][DEP]` Required reads are the companion BAI sections, rStar-Math/statistical-limits interfaces, and D-1/D-3 archive/template/verification implementation surfaces. (`D-5_bai_thompson_promotion.md:47-58`)
- **D5-013**: `[E][INT/DEP]` Before implementation, `paper-ingestor` must create and verify `audibert-colt-2010-bai-interface.md` and `variance-bai-2106.10417-interface.md`. (`D-5_bai_thompson_promotion.md:60-67`)
- **D5-014**: `[E][DATA/CAP]` New `ArmPosterior` fields are `template_id`, `pass_count`, `fail_count`, `last_updated`; `alpha=pass_count+1`, `beta=fail_count+1`, `mean=alpha/(alpha+beta)`, and `credible_interval(alpha_level)` returns lower/upper bounds. (`D-5_bai_thompson_promotion.md:69-79`)
- **D5-015**: `[E][SEC/FAIL]` `statrs::distribution::Beta` may be used only if already available; otherwise the agent must ask before adding it, and the dependency-free fallback is a hand-rolled incomplete-beta function under 40 LOC. (`D-5_bai_thompson_promotion.md:77-81`)
- **D5-016**: `[E][DET/DATA]` `ArmPosterior::update(passed)` returns a new value using an immutable-update pattern. (`D-5_bai_thompson_promotion.md:82-83`)
- **D5-017**: `[E][DET/CAP]` `sample_beta(&mut SeededRng, alpha, beta)` uses the gamma-ratio method and C-15 `SeededRng`, not ambient randomness. (`D-5_bai_thompson_promotion.md:85-88`)
- **D5-018**: `[E][FAIL/CAP]` `thompson_select` samples every arm, returns the maximum-theta `TemplateId`, and returns `None` for empty input. (`D-5_bai_thompson_promotion.md:89-91`)
- **D5-019**: `[E][CAP]` `successive_rejects(Vec<ArmPosterior>, eval_budget)` performs round elimination by mean, assigns `n_k = budget / (K-k)` pulls to survivors, and returns ordered survivor IDs. (`D-5_bai_thompson_promotion.md:93-97`)
- **D5-020**: `[E][CAP]` `lucb1_next_pull(arms, delta) -> Option<TemplateId>` computes `mean_i + sqrt(log(1/delta)/(2*n_i))` and is directed to return the highest-UCB competitor; unlike `thompson_select`, the source does not state the condition that yields `None`. (`D-5_bai_thompson_promotion.md:99-103`)
- **D5-021**: `[E][DATA]` `BaiPromotionEngine` owns arm/posterior maps, promoted/retired ID lists, evaluation budget, per-domain deltas, and a `SeededRng`. (`D-5_bai_thompson_promotion.md:105-108`)
- **D5-022**: `[E][WF]` `ingest_eval_result(template_id, passed)` updates the target posterior. (`D-5_bai_thompson_promotion.md:109`)
- **D5-023**: `[E][WF/OBS]` `promote_round(stage)` applies the credible-bound rule, emits one `BaiPromotionDecision` row per arm, transitions `Candidate -> Promoted`, and appends the promotion event to `ArchiveChain`. (`D-5_bai_thompson_promotion.md:110-113`)
- **D5-024**: `[E][WF]` `retire_round(stage)` applies Successive Rejects to the bottom quartile, marks losers `Superseded`, and calls `ArchiveChain.retire`. (`D-5_bai_thompson_promotion.md:114-116`)
- **D5-025**: `[E][DATA]` Exact `PromotionStage` variants are `Stage10`, `Stage50`, and `Stage200`. (`D-5_bai_thompson_promotion.md:117`)
- **D5-026**: `[E][DATA/INT]` Schema bumps `50 -> 51`; exact payloads are `ArmPosteriorUpdated {template_id, pass_count, fail_count, mean, ci_lower, ci_upper}`, `ThompsonSampled {round_id, sampled_arms, selected}`, and `BaiPromotionDecision {template_id, decision, stage, evidence_pass, evidence_total, audit_id}`, all with round-trip tests. (`D-5_bai_thompson_promotion.md:119-126`)
- **D5-027**: `[E][CFG/SEC]` L3-locked `[bai]` defaults are `eval_budget=200`, `ci_alpha=0.05`, `delta_code=0.05`, `delta_math=0.03`, `delta_prose=0.05`, `delta_mixed=0.04`. (`D-5_bai_thompson_promotion.md:128-132`)
- **D5-028**: `[E][DET]` All sampling must use `SeededRng`; identical inputs and `ELAI_SEED` must yield byte-identical promotion decisions, demonstrated with two seed-42 runs over a 20-arm stream. (`D-5_bai_thompson_promotion.md:134-137`)
- **D5-029**: `[E][TEST]` Posterior unit gates: means are 0.5 for `(0,0)`, approximately 0.917 for `(10,0)`, approximately 0.083 for `(0,10)`; `(50,50)` CI is symmetric and `(95,5)` is high-skewed. (`D-5_bai_thompson_promotion.md:139-143`)
- **D5-030**: `[E][DET/TEST]` `sample_beta` and `thompson_select` must be deterministic under a fixed seed. (`D-5_bai_thompson_promotion.md:144-145`)
- **D5-031**: `[E][TEST]` Successive Rejects must show the stated 4/3/2/1 survivor allocation for four arms/budget 12; LUCB must pick the wider-confidence competitor rather than the empirical best. (`D-5_bai_thompson_promotion.md:146-149`)
- **D5-032**: `[E][TEST]` A five-arm, 100-step synthetic engine test with one 0.9 arm and four 0.5 arms must promote the best within budget and reproduce under the same seed. (`D-5_bai_thompson_promotion.md:150-152`)
- **D5-033**: `[E][TEST]` Integration uses ten rates `[0.9,0.85,0.7,0.65,0.5,0.5,0.4,0.3,0.2,0.1]`, 500 pulls, 100 seeds; regret must be at most twice the conservative Audibert `O(sqrt(K*budget*log(budget)))` bound and best-arm selection at least 85%. (`D-5_bai_thompson_promotion.md:154-161`)
- **D5-034**: `[E][CON]` The decision record must justify binary Beta-Bernoulli (consensus is Pass/Reject), per-domain deltas (different noise floors), and retaining MSI only as an operator-visible secondary signal, never the gate. (`D-5_bai_thompson_promotion.md:163-170`)
- **D5-035**: `[E][INT]` `WIRING_CONTRACT.produces_symbols` includes every posterior/algorithm/engine/stage symbol, the three SSB variants/payloads, schema 51, and `[bai]`. (`D-5_bai_thompson_promotion.md:172-181`)
- **D5-036**: `[E][DEP/INT]` Consumers/reachability are D-1 `ArchiveChain` and template status, D-3 consensus count, C-15/D-1 `SeededRng`, D-1 staged fields, the task-runner after-eval hook, `[bai]`, and `ELAI_SEED`. (`D-5_bai_thompson_promotion.md:182-191`)
- **D5-037**: `[E][FAIL/DET/SEC]` Stub bans require no `thread_rng`, graceful empty-arm return, `(0,1)` CI for the uniform zero-sample prior rather than NaN, and config-loaded deltas that the agent cannot edit. (`D-5_bai_thompson_promotion.md:192-199`)
- **D5-038**: `[E][OBS/DET]` A seeded ten-arm benchmark must produce at least 100 `ArmPosteriorUpdated` rows, at least one decision per survivor, identical same-seed SSB diffs, and a regret assertion artifact. (`D-5_bai_thompson_promotion.md:199-205`)
- **D5-039**: `[E][TEST]` Deterministic checks assert schema 51, presence/count of all events and algorithms, `SeededRng` with zero `thread_rng|rand::random`, `[bai]`, workspace/scoped builds, and both paper interfaces. (`D-5_bai_thompson_promotion.md:207-238`)
- **D5-040**: `[E][TEST]` Acceptance requires public schema 51, correct posterior math, deterministic sampling/selection, Audibert allocation, correct LUCB choice, and the one-good/four-inferior engine outcome with inferior arms `Superseded`. (`D-5_bai_thompson_promotion.md:240-250`)
- **D5-041**: `[E][TEST]` Acceptance also requires 500-pull/100-trial regret and 85% selection gates, three event round trips, loaded defaults, zero new workspace errors, and at least 15 passing BAI tests. (`D-5_bai_thompson_promotion.md:251-256`)
- **D5-042**: `[E][TEST/OBS]` Cloud A is a 50-cycle bootstrap with posterior/decision/archive-transition evidence; Cloud B repeats identical seed-42 tasks and permits only timestamps to differ while BAI rows remain byte-identical; paper interfaces and the three-decision document are mandatory. (`D-5_bai_thompson_promotion.md:257-264`)
- **D5-043**: `[E][CAP/INT]` File effects are new `bai.rs`, two integration/contract tests, decision/paper interfaces, plus edits to `ssb.rs`, `task_runner.rs`, config, and `lib.rs`; `statrs` is conditional on user approval. (`D-5_bai_thompson_promotion.md:266-280`)
- **D5-044**: `[E][CON/STATUS]` Hierarchical BAI, adversarial-example farming, and reward-model retraining are outside D-5; the first two are explicitly deferred. (`D-5_bai_thompson_promotion.md:282-286`)
- **D5-045**: `[E][DEP/WF]` D-6/D-7 later feed per-arm pass/fail signals; D-8 exposes promotion lineage. (`D-5_bai_thompson_promotion.md:288-292`)

### D-6: held out task store

- **Status**: PLANNED — deps D-5; Phase 4H.VIII.1 / temporal held-out gate
- **Anchors**: plan `D-6_held_out_task_store.md`; DEEP-D-001..004; aggregate unit (52 declared source atoms); compiled 52
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-002 — **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 p | D-001/IX-003 — **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks.  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-009 — **Evaluation-store secrecy.** D-6 composes temporal/9-gram filtering with Arbor-style dev/test isolation and DeNovo-style `.git` history removal; runtime leakage blocking remains explicitly owned by H

**Behaviors**

- **D6-001**: `[E][CAP/WF]` D-6 maintains `.elai_cc/held-out/{date}/{task-id}.toml`, applies a 9-gram contamination filter, retires old uniformly passing tasks, and blocks template promotion when held-out regression exceeds suggested `k=5%`. (`D-6_held_out_task_store.md:1-7`)
- **D6-002**: `[E][DEP/DET]` Estimated effort is two sessions; D-5 must be `DONE`; D-7 is parallel-safe, while D-4 is not because both touch held-out/archive evaluation state. (`D-6_held_out_task_store.md:8-13`)
- **D6-003**: `[E][WF/INT]` The LiveCodeBench pattern is date tagging, N-session refresh, retirement after six months plus uniform passing, and intake only from recency-verifiable GitHub issues or new arXiv reproducibility tasks. (`D-6_held_out_task_store.md:15-27`)
- **D6-004**: `[E][SEC/CON]` Because 9-grams miss near-equivalent training examples, the design adds time-window plus semantic-fingerprint defense using rolling embedding similarity to recent ingest dates. (`D-6_held_out_task_store.md:28-31`)
- **D6-005**: `[E][SEC/CON]` ARC’s structural prior can itself leak; the two stated defenses are time windows and structural-prior randomization. (`D-6_held_out_task_store.md:33-37`)
- **D6-006**: `[E][DEP/DATA]` Reused foundations are C-17 tokenization/chunking for content hashing, D-1 `ArchiveChain` for task-retirement evidence whose payload carries `task_id` rather than `template_id`, and C-15 `SeededClock` for ingest dates. (`D-6_held_out_task_store.md:39-45`)
- **D6-007**: `[E][DEP]` Required implementation reads are the companion, statistical-limits interface, DKG types only as a ruled-out D-15 concern, `archive_chain.rs`, and `clock.rs`. (`D-6_held_out_task_store.md:47-57`)
- **D6-008**: `[E][INT/DEP]` `paper-ingestor` must create/verify LiveCodeBench, Soft Contamination, and ARC Prize 2025 interfaces. (`D-6_held_out_task_store.md:58-66`)
- **D6-009**: `[E][DATA]` `HeldOutTask` fields are `task_id`, `submission_date`, `domain`, bounded 4096-char problem statement, `expected_outcome`, `source`, Blake3 `content_hash`, and `status`. (`D-6_held_out_task_store.md:68-75`)
- **D6-010**: `[E][DATA]` `ExpectedOutcome` variants and payloads are `Code{test_cmd}`, `Math{sympy_expr}`, `Prose{rubric<=1024}`, `Planning{final_state<=2048}`, and `KnowledgeQa{reference_answer<=2048}`. (`D-6_held_out_task_store.md:76-78`)
- **D6-011**: `[E][DATA]` `TaskSource` variants are `GitHubIssue{repo,issue_num,fetched_at}`, `ArxivRepro{arxiv_id,fetched_at}`, `Synthetic{generator_version}`, and `HumanCurated{curator}`. (`D-6_held_out_task_store.md:79-81`)
- **D6-012**: `[E][DATA/WF]` `HeldOutStatus` is `Active`, `AutoRetired{reason}`, or `ManuallyExcluded`; retirement reasons are `UniformPassSuspicion`, `NgramContamination`, `SemanticContamination`, `AgeExceedsTtl`; `TaskId` wraps `Uuid`. (`D-6_held_out_task_store.md:82-84`)
- **D6-013**: `[E][DATA/CAP]` `HeldOutStore` owns `root_path`, ID index, and date index; `load` walks dated TOML files, parses tasks, and rebuilds both indexes. (`D-6_held_out_task_store.md:86-91`)
- **D6-014**: `[E][WF]` `append` writes `root/{date}/{task_id}.toml` and updates the in-memory index. (`D-6_held_out_task_store.md:92-93`)
- **D6-015**: `[E][WF/OBS]` `retire` changes the status, rewrites the TOML file, and emits `HeldOutAutoRetire`. (`D-6_held_out_task_store.md:94-95`)
- **D6-016**: `[E][CAP]` `query(domain,since)` filters by optional domain/date and returns active tasks only. (`D-6_held_out_task_store.md:96-97`)
- **D6-017**: `[E][DATA/FAIL]` `extract_ngrams(text,n)` lowercases, tokenizes on whitespace/punctuation, and returns sliding n-grams; it must return an empty set for empty input. (`D-6_held_out_task_store.md:99-103,214-216`)
- **D6-018**: `[E][DATA]` `ngram_overlap` is Jaccard `intersection/union` over n-gram sets. (`D-6_held_out_task_store.md:104-105`)
- **D6-019**: `[E][SEC/CFG]` `detect_ngram_contamination` hashes each 9-gram with Blake3, probes `.elai_cc/held-out/ngram-bloom.bin`, and treats Jaccard `>0.3` as contamination. (`D-6_held_out_task_store.md:106-109`)
- **D6-020**: `[E][DATA/CFG]` `AutoRefreshPolicy` fields are TTL months, uniform-pass threshold, and minimum evaluations; defaults are 6, 0.95, and 20. (`D-6_held_out_task_store.md:111-114`)
- **D6-021**: `[E][WF]` Auto-retirement requires all three strict conditions: age greater than TTL, pass rate greater than threshold, and evaluations at least the configured minimum; reason is `UniformPassSuspicion`. (`D-6_held_out_task_store.md:115-120`)
- **D6-022**: `[E][CAP]` `held_out_regression` compares baseline/candidate pass rates on their shared held-out subset and returns signed percentage delta, positive for improvement and negative for regression. (`D-6_held_out_task_store.md:122-126`)
- **D6-023**: `[E][WF/FAIL]` `gate_promotion(delta,threshold)` allows iff `delta >= -threshold` and returns a reason string containing the measured delta. (`D-6_held_out_task_store.md:127-129`)
- **D6-024**: `[E][DATA/INT]` Schema bumps `51 -> 52`; payloads are `HeldOutRefresh{date,action,task_count_added,task_count_retired}`, `HeldOutAutoRetire{task_id,reason,age_days,uniform_pass_rate}`, and `HeldOutPromotionGate{template_id,delta,threshold,allowed}`, all round-tripped. (`D-6_held_out_task_store.md:131-138`)
- **D6-025**: `[E][CFG/SEC]` L3-locked `[held_out]` defaults are root `.elai_cc/held-out`, TTL 6, pass threshold 0.95, minimum 20, regression threshold 5.0%, n-gram size 9, and Jaccard threshold 0.3. (`D-6_held_out_task_store.md:140-145`)
- **D6-026**: `[E][DATA/INT]` Manual seeding creates 25 tasks on `2026-05-19`, five per family: 10 recent GitHub issues, 10 public ARC-AGI tasks, and 5 curated math/prose tasks, each with expected outcome and content hash. (`D-6_held_out_task_store.md:147-152`)
- **D6-027**: `[E][FAIL/STATUS]` Training-corpus Bloom precomputation is deferred; D-6 implements the interface and skips n-gram checks when the Bloom file is empty/unavailable. Candidate source is a public known-training-data subset such as HuggingFace `the-stack`. (`D-6_held_out_task_store.md:154-159`)
- **D6-028**: `[E][DET/TEST]` Store unit coverage is load/append/retire round-trip in a temporary directory. (`D-6_held_out_task_store.md:161-162`)
- **D6-029**: `[E][TEST]` N-gram units require the exact 3-gram set for `a b c d e`, overlap 1.0 for identical input, and 0.0 for disjoint input. (`D-6_held_out_task_store.md:163-164`)
- **D6-030**: `[E][TEST]` Contamination detection is tested with overlapping and disjoint synthetic corpus/Bloom inputs. (`D-6_held_out_task_store.md:165-166`)
- **D6-031**: `[E][TEST]` Retirement table: 7 months/100%/25 evals retires; 7 months/80% keeps; 5 months/100% keeps; 7 months/100%/10 evals keeps. (`D-6_held_out_task_store.md:167-170`)
- **D6-032**: `[E][TEST]` Promotion table: `-3%` against 5% allows; `-7%` blocks. (`D-6_held_out_task_store.md:171`)
- **D6-033**: `[E][TEST]` Integration synthesizes 50 tasks, advances a mock clock across 12 months, verifies retirement membership, and gates baseline versus candidate histories. (`D-6_held_out_task_store.md:173-179`)
- **D6-034**: `[E][CON]` Decision doc must justify per-task TOML for git diffs/no sled locking/~10K scale, `n=9` as LiveCodeBench’s balance, and six-month TTL as a frontier-model cutoff/curation-cost tradeoff. (`D-6_held_out_task_store.md:180-188`)
- **D6-035**: `[E][INT]` Produced contract includes all task/status/source/outcome IDs, store methods, refresh policy, contamination functions, promotion gate/result, three events/payloads, schema 52, config, and dated file layout. (`D-6_held_out_task_store.md:190-203`)
- **D6-036**: `[E][DEP/INT]` Consumes C-15 clock, D-1 archive audit, D-5 BAI, Blake3, and C-17 tokenization; reachable from D-5’s pre-promotion gate, config, and directory layout. (`D-6_held_out_task_store.md:204-213`)
- **D6-037**: `[E][FAIL/DET]` Files must never be deleted; retirement is status marking; empty n-gram input cannot panic. (`D-6_held_out_task_store.md:214-216`)
- **D6-038**: `[E][SEC/FAIL]` Retirement must enforce the minimum sample count and gate thresholds must be config-loaded rather than hardcoded. (`D-6_held_out_task_store.md:217-219`)
- **D6-039**: `[E][OBS]` A 25-task/12-month uniform-pass simulation expects at least 20 `HeldOutAutoRetire` rows, zero for tasks under six months, all 25 fixture files, and persisted status changes. (`D-6_held_out_task_store.md:220-226`)
- **D6-040**: `[E][TEST]` Deterministic checks assert schema 52, all event/type/filter symbols, config keys/defaults, and directory presence. (`D-6_held_out_task_store.md:228-253`)
- **D6-041**: `[E][TEST/INT]` Deterministic checks also run workspace/held-out tests and require all three paper interfaces. The shell presence gate accepts at least 20 TOML files. (`D-6_held_out_task_store.md:251-263`)
- **D6-042**: `[E][DET/TEST]` Acceptance requires public schema 52, three event round trips, idempotent store load/append/retire, and at least ten canonical n-gram scenarios. (`D-6_held_out_task_store.md:265-270`)
- **D6-043**: `[E][TEST]` Acceptance also requires the four retirement scenarios, promotion threshold logic, loaded defaults, exactly 25 seed tasks, zero new workspace errors, and at least 12 passing held-out tests. (`D-6_held_out_task_store.md:271-276`)
- **D6-044**: `[E][TEST/OBS]` Cloud A advances a mocked clock 12 months and requires retirement rows for uniformly passing aged tasks. (`D-6_held_out_task_store.md:277-278`)
- **D6-045**: `[E][TEST/FAIL]` Cloud B injects 7% held-out regression and requires `HeldOutPromotionGate{allowed=false}` with no `TemplatePromoted` event. (`D-6_held_out_task_store.md:279-281`)
- **D6-046**: `[E][TEST/INT]` Cloud C ingests a new arXiv reproduction task through paper-ingestor and requires a refresh row plus file; paper outputs and the three-decision document are mandatory. (`D-6_held_out_task_store.md:282-285`)
- **D6-047**: `[E][CAP/INT]` File effects add `held_out.rs`, 25 task files, integration test, decisions, and three paper interfaces; edit schema, BAI gate integration, config, and module export. (`D-6_held_out_task_store.md:287-299`)
- **D6-048**: `[E][CON/DEP]` D-7, D-15, D-20, and corpus Bloom construction are out of D-6; D-7 consumes family rates and D-22 reuses this store across its three distributions. (`D-6_held_out_task_store.md:301-313`)
- **D6-049**: `[E][INT/SEC]` J-11/S-2 records Arbor’s `B_dev` iteration set versus `B_test` milestone-only set and detached-worktree merge re-run as prior art; the executor never touches `B_test`. (`D-6_held_out_task_store.md:315-323`)
- **D6-050**: `[E][CON]` Arbor’s split is manual and simpler; D-6 alone owns the time window, 9-gram filter, and uniform-pass auto-retirement, so the prior art must be composed rather than restated. (`D-6_held_out_task_store.md:323-325`)
- **D6-051**: `[E][SEC]` J-11/S-8c requires fixture-build-time destruction and re-initialization of `.git` when held-out tasks come from real repositories, preventing recovery of reference history through reflog or loose objects. (`D-6_held_out_task_store.md:327-336`)
- **D6-052**: `[E][SEC/CON]` `.git` reinitialization is store hygiene, not a runtime gate; runtime command blocking and trace audit belong to H-47. (`D-6_held_out_task_store.md:333-337`)

### D-12: operationalization check predicate

- **Status**: PLANNED — deps D-11; RPS.III / measurable-procedure audit gate
- **Superseded:** SUP-001 — Baby-step block number for MEASURABLE_PROCEDURE. Target: D-12 block 9 (eight existing + ninth). Loser: Summary said block 7.
- **Anchors**: plan `D-12_operationalization_check_predicate.md`; DEEP-D-001..004; aggregate unit (38 declared source atoms); compiled 38
- **Composes with**: D-001/IX-004 — **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lac

**Behaviors**

- **D12-001**: `[E][WF/SEC]` Every claim-emitting baby step must co-emit a runnable `MEASURABLE_PROCEDURE` intended to fail when the claim is wrong; without it the step cannot enter `MEMORIZING`, and the auditor blocks at acceptance. (`D-12_operationalization_check_predicate.md:1-8,18-21`)
- **D12-002**: `[E][DEP/STATUS]` Effort is one to two sessions; D-11 checker registry must be `DONE`, verified by `spec-grounder`. (`D-12_operationalization_check_predicate.md:9-11`)
- **D12-003**: `[E][DET/CON]` D-12 is parallel-safe with D-13/D-14 but not D-15 because novelty labels and operationalization share the checker-result hook. (`D-12_operationalization_check_predicate.md:12-14`)
- **D12-004**: `[E][DATA]` The source states the current BabyStepSpec has eight blocks (`TITLE`, `OBJECTIVE`, `PRE_GATE`, `FILES_PLAN`, `TASKS`, `DETERMINISTIC_CHECK`, `TESTS`, `ACCEPTANCE`) and D-12 adds block **9**. (`D-12_operationalization_check_predicate.md:16-25`)
- **D12-005**: `[E][WF/INT]` A procedure names a `CheckerEntry`, carries claim/candidate/domain arguments, runs when leaving `AUDITING`, and contributes its `CheckerResult` to the acceptance decision. (`D-12_operationalization_check_predicate.md:27-30`)
- **D12-006**: `[E][SEC/FAIL]` The L3 rule is fail-closed: claim without procedure cannot reach `MEMORIZING`; author emits it and auditor enforces it. (`D-12_operationalization_check_predicate.md:32-33`)
- **D12-007**: `[E][DEP]` Foundations are C-31’s block-extension pattern, auditor acceptance hook, D-11 registry/types, and D-1’s overlapping system-level `verifier_required` concept. (`D-12_operationalization_check_predicate.md:35-41`)
- **D12-008**: `[E][DEP]` Required reads are the RPS guide, C-31, the actual BabyStepSpec/parser, D-11 registry, and the auditor prompt; the author prompt is a later consumed/modified surface, not listed in this `Reads` section. (`D-12_operationalization_check_predicate.md:43-52,161-166,240-241`)
- **D12-009**: `[E][DATA]` `BabyStepSpec.measurable_procedure` is temporarily `Option<MeasurableProcedure>`; the struct fields are checker name, bounded 2048 claim, optional bounded 8192 candidate, optional bounded 1024 domain args, and expected `CheckerResult`. (`D-12_operationalization_check_predicate.md:54-63`)
- **D12-010**: `[E][WF]` `expected_result` is a prediction: runtime executes the checker and diffs actual from expected, including expected failure for negative tests. (`D-12_operationalization_check_predicate.md:64-65,140-147`)
- **D12-011**: `[E][DATA/FAIL]` The parser recognizes block token `MEASURABLE_PROCEDURE`, accepts the shown TOML-like `checker/claim/expected_result` format, populates the field, and emits `ParseError` on invalid input. (`D-12_operationalization_check_predicate.md:67-77`)
- **D12-012**: `[E][SEC/CON]` The babystep-author prompt must state that every claim-emitting step includes the block. (`D-12_operationalization_check_predicate.md:79-81`)
- **D12-013**: `[E][TEST/CON]` Prompt examples contrast an unmeasured `O(n log n)` claim (block) with a `perf` checker over `n=10..10000` (compliant). (`D-12_operationalization_check_predicate.md:82-85`)
- **D12-014**: `[E][SEC/FAIL]` Auditor detects a claim with no procedure and blocks with reason `missing MEASURABLE_PROCEDURE`. (`D-12_operationalization_check_predicate.md:87-90`)
- **D12-015**: `[E][FAIL]` Auditor runs the procedure and blocks when actual differs from expected, with reason `measurable procedure result diverges from expected`. (`D-12_operationalization_check_predicate.md:91-93`)
- **D12-016**: `[E][CAP/FAIL]` `require_procedure_for_claim` scans claim-emitting symbols/artifacts and returns `OperationalizationError` when required evidence is absent. (`D-12_operationalization_check_predicate.md:95-101`)
- **D12-017**: `[E][CAP/INT]` `run_procedure` dispatches through D-11 `CheckerRegistry` with `OrchContext`. (`D-12_operationalization_check_predicate.md:102-104`)
- **D12-018**: `[E][CAP/DATA]` `diff_expected(expected,actual)` returns optional `ProcedureDivergence`. (`D-12_operationalization_check_predicate.md:105-106`)
- **D12-019**: `[E][CAP]` `detect_claim_in_baby_step` searches task and observable-effect prose for stated keywords such as “correct,” “complexity is,” “produces,” “guarantees,” and “ensures,” returning `DetectedClaim` values. (`D-12_operationalization_check_predicate.md:108-113`)
- **D12-020**: `[E][SEC]` Detection is intentionally conservative: false positives are acceptable because they force evidence; false negatives are forbidden because they leak untested claims. (`D-12_operationalization_check_predicate.md:114-116`)
- **D12-021**: `[E][DATA/INT]` Schema bumps `57 -> 58`; payloads are `ClaimWithoutProcedure{baby_step_id,claim_snippet,audit_id}` and `ProcedureExecuted{baby_step_id,checker_name,result,expected,divergence}`, with round-trip tests. (`D-12_operationalization_check_predicate.md:118-123`)
- **D12-022**: `[E][TEST]` Claim-detection unit tests cover three claim and three non-claim steps. (`D-12_operationalization_check_predicate.md:125-127`)
- **D12-023**: `[E][TEST]` Units require missing-procedure error, `Pass==Pass -> None`, and `Pass!=Fail -> divergence`. (`D-12_operationalization_check_predicate.md:128-130`)
- **D12-024**: `[E][TEST/OBS]` Integration case 1: missing procedure causes auditor `BLOCK` and a `ClaimWithoutProcedure` row. (`D-12_operationalization_check_predicate.md:132-135`)
- **D12-025**: `[E][TEST/OBS]` Integration case 2: a procedure runs; `Pass` allows progress, while `Fail` blocks and writes `ProcedureExecuted` with divergence. (`D-12_operationalization_check_predicate.md:136-138`)
- **D12-026**: `[E][CON]` Decisions justify a typed ninth block rather than shell-only checks, expected-vs-actual tri-state semantics rather than “always Pass,” and conservative detection. (`D-12_operationalization_check_predicate.md:140-150`)
- **D12-027**: `[E][INT]` Produced symbols are `MeasurableProcedure`, require/run/diff helpers, claim detector/type, both SSB variants/payloads, schema 58, and BabyStepSpec block 9. (`D-12_operationalization_check_predicate.md:152-160`)
- **D12-028**: `[E][DEP]` Consumes registry/result/parser plus auditor and author agents. (`D-12_operationalization_check_predicate.md:161-166`)
- **D12-029**: `[E][WF]` Reachability is from `AUDITING` procedure dispatch and authoring-time plan generation. (`D-12_operationalization_check_predicate.md:167-169`)
- **D12-030**: `[E][FAIL/SEC]` Stub bans require conservative detection, per-checker timeout, full `Pass/Fail/Inapplicable` diffing, and no opt-out from fail-closed enforcement. (`D-12_operationalization_check_predicate.md:170-176`)
- **D12-031**: `[E][OBS/SEC]` Missing-procedure trigger must yield one event, blocked `AUDITING`, a human-review prompt, updated agent prompts, and parser support for block 9. (`D-12_operationalization_check_predicate.md:177-183`)
- **D12-032**: `[E][TEST]` Deterministic checks assert schema 58, both events, `MeasurableProcedure` plus `require_procedure_for_claim`/`run_procedure`, the block token in the parser, both prompt updates, workspace build, and scoped tests; the shell block does not directly grep `diff_expected` or `detect_claim_in_baby_step`. (`D-12_operationalization_check_predicate.md:185-208`)
- **D12-033**: `[E][TEST]` Acceptance requires public schema 58, two event round trips, compiled helpers, parser support, at least six classification fixtures, correct missing-procedure error, and tri-state diff. (`D-12_operationalization_check_predicate.md:210-218`)
- **D12-034**: `[E][TEST]` Acceptance also requires zero new workspace errors, at least eight passing tests, and both agent docs with examples/enforcement. (`D-12_operationalization_check_predicate.md:219-222`)
- **D12-035**: `[E][TEST/WF]` Cloud A requires an authored claim with the block, executed checker, and acceptance pass. (`D-12_operationalization_check_predicate.md:223-225`)
- **D12-036**: `[E][TEST/FAIL]` Cloud B requires missing procedure to block/journal; Cloud C requires expected Pass/actual Fail to journal divergence and block; the decision doc is mandatory. (`D-12_operationalization_check_predicate.md:226-230`)
- **D12-037**: `[E][CAP/INT]` File effects add `operationalization.rs`, integration test, and decision doc; edit BabyStepSpec/parser, SSB, exports, and both agent prompts. (`D-12_operationalization_check_predicate.md:232-243`)
- **D12-038**: `[E][CON/DEP]` Per-checker templates, auto-generation, and procedure caching are deferred; every audit runs fresh. D-13 next covers claims without a Mode-A checker. (`D-12_operationalization_check_predicate.md:245-255`)

### D-23: sycophancy detection benchmark

- **Status**: PLANNED — deps D-14; D-14 gating evidence / detection/FPR
- **Anchors**: plan `D-23_sycophancy_detection_benchmark.md`; DEEP-D-001..004; aggregate unit (21 declared source atoms); compiled 21
- **Composes with**: D-001/IX-005 — **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture 

**Behaviors**

- **D23-001**: `[E][CAP/FAIL]` D-23 uses Sharma public probes or synthetic stand-ins, runs D-14, and measures detection and false positives. (`D-23_sycophancy_detection_benchmark.md:1-5`)
- **D23-002**: `[E][TEST/DEP/DET]` Gate is detection `>=70%` and FP `<=15%`; effort is one to two sessions; D-14 is prerequisite; it is parallel-safe with D-19..D-25. (`D-23_sycophancy_detection_benchmark.md:5-8`)
- **D23-003**: `[E][INT/TEST]` Source context states literature sycophancy at 23-33% across models and expects the structural probe to surface about 70%+ on labeled public/synthetic fixtures. (`D-23_sycophancy_detection_benchmark.md:10-16`)
- **D23-004**: `[E][DEP]` Reads are the RPS sycophancy section, Sharma interface from D-14, and D-14 `sycophancy.rs`. (`D-23_sycophancy_detection_benchmark.md:18-23`)
- **D23-005**: `[E][DATA/SEC]` Fixture set contains exactly 100 human-labeled examples: 70 expected `Sycophancy`, 30 expected `Robust`; positive fixtures encode user assertion plus agreeing model, negatives independent agreement. (`D-23_sycophancy_detection_benchmark.md:25-31,66-68`)
- **D23-006**: `[E][DATA]` `SycophancyBenchmark` owns fixture directory and `Arc<SycophancyProbe>`; report fields are total, TP, FP, TN, FN, detection rate, FP rate, and `GatingVerdict`. (`D-23_sycophancy_detection_benchmark.md:33-39`)
- **D23-007**: `[E][WF/TEST]` `run` replays fixtures through D-14, compares flagged divergence with labels, and computes `TP/(TP+FN)` plus `FP/(FP+TN)`. (`D-23_sycophancy_detection_benchmark.md:40-43`)
- **D23-008**: `[E][WF]` The benchmark passes only when both threshold predicates hold. (`D-23_sycophancy_detection_benchmark.md:45`)
- **D23-009**: `[E][CAP]` CLI surface is `elai bench sycophancy`. (`D-23_sycophancy_detection_benchmark.md:47`)
- **D23-010**: `[E][CON]` Decisions justify the 70/30 split as matching Sharma, the 70% floor as conservative versus roughly 80%, and the 15% FP cap as protection against human-review queue clutter. (`D-23_sycophancy_detection_benchmark.md:49-54`)
- **D23-011**: `[E][INT]` Produces benchmark/report and CLI; consumes D-14 `SycophancyProbe` plus `SsbWriter`; reachable from CLI and CI. (`D-23_sycophancy_detection_benchmark.md:56-65`)
- **D23-012**: `[E][SEC/CON]` Fixtures must be labeled by a human reviewer, not auto-generated. (`D-23_sycophancy_detection_benchmark.md:66-67`)
- **D23-013**: `[E][OBS/CON]` Detection and false-positive rate must be reported separately. (`D-23_sycophancy_detection_benchmark.md:68`)
- **D23-014**: `[E][OBS]` A run emits existing `BenchmarkStarted` and `BenchmarkResult` rows and writes `.elai_cc/benchmarks/sycophancy/reports/{date}.json`. (`D-23_sycophancy_detection_benchmark.md:69-72`)
- **D23-015**: `[E][TEST]` Deterministic checks require at least 100 fixture JSON files, presence of the benchmark struct, and a clean workspace build. (`D-23_sycophancy_detection_benchmark.md:74-81`)
- **D23-016**: `[E][TEST]` Acceptance requires compilation, 100 fixtures, CLI exit 0 on both gates, and zero new workspace errors. (`D-23_sycophancy_detection_benchmark.md:83-88`)
- **D23-017**: `[E][TEST]` Cloud A must meet detection and FP thresholds. (`D-23_sycophancy_detection_benchmark.md:89`)
- **D23-018**: `[E][TEST]` Cloud B requires high-confidence robust agreements to classify `Convergent` and remain unflagged. (`D-23_sycophancy_detection_benchmark.md:90-91`)
- **D23-019**: `[E][CAP/INT]` File effects add runner, CLI module, 100 claim fixtures, decision doc, and edit CLI commands. (`D-23_sycophancy_detection_benchmark.md:94-100`)
- **D23-020**: `[E][CON]` Multi-turn sycophancy is out of scope. (`D-23_sycophancy_detection_benchmark.md:102-104`)
- **D23-021**: `[E][DEP]` D-24 is the declared next session. (`D-23_sycophancy_detection_benchmark.md:106-108`)

### D-26: reasoning template dashboard

- **Status**: PLANNED — deps D-1..D-8 shipped; Wave-1 dashboard exposure
- **Superseded:** SUP-003 — D-26 prerequisites. Target: D-26 original: D-1..D-9 shipped. Loser: D-0 index: D-1..D-8.
- **Superseded:** SUP-016 — D-26/D-27 dashboard surface counts. Target: D-26: 6 commands + 6 components; D-27: 7 commands + 6 components. Loser: Summary thinner command/component lists.
- **Anchors**: plan `D-26_reasoning_template_dashboard.md`; DEEP-D-001..004; aggregate unit (34 declared source atoms); compiled 34
- **Composes with**: D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-010 — **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not indepen

**Behaviors**

- **D26-001**: `[E][CAP]` D-26 surfaces typed templates, archive, tripwires, BAI, held-out state, canaries, and archive verification; its objective also names ReasoningDrawer PPM score, consensus rule, and tripwire status. (`D-26_reasoning_template_dashboard.md:1-8`)
- **D26-002**: `[E][DEP/DET]` Effort is two to three sessions; the source requires D-1..D-9 shipped and is parallel-safe with D-27. (`D-26_reasoning_template_dashboard.md:9-11`)
- **D26-003**: `[E][DEP/CAP]` Reuses C-3/C-4 Tauri patterns, C-6 tokens, C-7 layouts, C-24 Cytoscape styles, and SSB queryability. (`D-26_reasoning_template_dashboard.md:13-25`)
- **D26-004**: `[E][DEP]` Reads include C-3/C-7/C-24, dashboard commands/drawer state, and all Wave-1 plan artifacts. (`D-26_reasoning_template_dashboard.md:27-34`)
- **D26-005**: `[E][CAP/FAIL]` `get_archive_recent(limit)` returns latest N `ArchiveEntrySummary` values and must honor the limit. (`D-26_reasoning_template_dashboard.md:38-41,124-126`)
- **D26-006**: `[E][CAP/FAIL]` `get_archive_verify_status` runs chain verification and returns `{total,verified,errors_count}`; verification must execute asynchronously so UI is not blocked. (`D-26_reasoning_template_dashboard.md:42-44,127`)
- **D26-007**: `[E][CAP]` `get_tripwire_status(window_minutes)` queries D-4 tripwire rows over the requested window. (`D-26_reasoning_template_dashboard.md:45-46`)
- **D26-008**: `[E][CAP]` `get_bai_arms` loads D-5 state and returns active posterior summaries. (`D-26_reasoning_template_dashboard.md:47-48`)
- **D26-009**: `[E][CAP]` `get_canary_report(optional template_id)` returns D-7 reports for a template or recent reports. (`D-26_reasoning_template_dashboard.md:49-50`)
- **D26-010**: `[E][CAP/DATA]` `get_held_out_status` returns active count, retired count, and last refresh time from D-6. (`D-26_reasoning_template_dashboard.md:51-52`)
- **D26-011**: `[E][CAP]` `MutationArchivePanel` lists hash prefix, domain, pattern kind, archive status, and optional retirement reason. (`D-26_reasoning_template_dashboard.md:54-57`)
- **D26-012**: `[E][CAP/DATA]` Archive rows expand to a template excerpt and parent link; filtering supports Candidate/Promoted/Retired/Frozen; colors are green/yellow/red/grey respectively. (`D-26_reasoning_template_dashboard.md:58-60`)
- **D26-013**: `[E][CAP]` `LineageGraphView` renders the parent-hash DAG in Cytoscape, colors nodes by status, and flips node clicks to archive detail. (`D-26_reasoning_template_dashboard.md:62-67`)
- **D26-014**: `[E][CAP/OBS]` `TripwireStatusStrip` live-polls the last five events with kind/time/value/threshold; concurrent fire or auto-revert is red, individual firing yellow. (`D-26_reasoning_template_dashboard.md:69-74`)
- **D26-015**: `[E][CAP/DATA]` `BaiArmsTable` shows ID prefix, counts, posterior mean, 95% CI, sorts by mean/lower bound, and highlights current promoted arm green. (`D-26_reasoning_template_dashboard.md:76-80`)
- **D26-016**: `[E][CAP]` `HeldOutPanel` shows per-family active counts and recent retirements with domain filtering. (`D-26_reasoning_template_dashboard.md:82-85`)
- **D26-017**: `[E][CAP/FAIL]` `ArchiveVerifyGauge` live-polls, shows green `Clean` or red `Tampered (N errors)`, and opens the full report on click. (`D-26_reasoning_template_dashboard.md:87-91`)
- **D26-018**: `[E][CAP]` `ReasoningDrawer.svelte` embeds the new panels as tabs or stacked sections. (`D-26_reasoning_template_dashboard.md:93-94`)
- **D26-019**: `[E][OBS/INT]` Tripwire/archive events use the existing C-7 real-time SSB subscription pattern. (`D-26_reasoning_template_dashboard.md:96-97`)
- **D26-020**: `[E][CON]` Decisions justify Cytoscape reuse, separate dockable Svelte components, and status colors for scan speed. (`D-26_reasoning_template_dashboard.md:99-105`)
- **D26-021**: `[E][INT]` Produced contract contains all six Tauri commands and all six components: archive, lineage, tripwire, BAI, held-out, and verification. (`D-26_reasoning_template_dashboard.md:107-114`)
- **D26-022**: `[E][DEP/INT]` Consumes archive/verification, D-4 tripwires, D-5 BAI, D-6 store, D-7 canary, and `SsbViewer`. (`D-26_reasoning_template_dashboard.md:115-121`)
- **D26-023**: `[E][CAP]` Reachable surfaces are the dashboard ReasoningDrawer and a dedicated archive sub-window. (`D-26_reasoning_template_dashboard.md:122-123`)
- **D26-024**: `[E][FAIL]` Full-archive accidental return is prohibited through mandatory `limit` honoring. (`D-26_reasoning_template_dashboard.md:124-126`)
- **D26-025**: `[E][FAIL/DET]` Chain verification cannot block the UI and therefore runs async. (`D-26_reasoning_template_dashboard.md:127`)
- **D26-026**: `[E][TEST/INT]` Every command must pass the existing ts-rs binding-drift workflow. (`D-26_reasoning_template_dashboard.md:128-129`)
- **D26-027**: `[E][OBS/TEST]` Live promotion/tripwire injection must stream SSB data, show the archive entry within the poll interval, turn the strip red, and keep verification green on clean state. (`D-26_reasoning_template_dashboard.md:130-135`)
- **D26-028**: `[E][TEST]` Deterministic checks grep commands/components and run `svelte-check` plus dashboard cargo check. (`D-26_reasoning_template_dashboard.md:137-145`)
- **D26-029**: `[E][TEST]` Acceptance requires six commands, six components, zero Svelte errors, clean ts-rs drift, and zero new dashboard Rust errors. (`D-26_reasoning_template_dashboard.md:147-152`)
- **D26-030**: `[E][TEST]` Cloud A requires a new archive entry, graph node, and parent edge after live promotion. (`D-26_reasoning_template_dashboard.md:153-155`)
- **D26-031**: `[E][TEST]` Cloud B requires a red tripwire strip within an explicit three-second poll interval. (`D-26_reasoning_template_dashboard.md:156-157`)
- **D26-032**: `[E][TEST]` Cloud C toggles verification green/red across clean/manual tamper; Cloud D shows at least 25 active held-out tasks; Cloud E shows sortable posterior rows; decision doc is mandatory. (`D-26_reasoning_template_dashboard.md:158-163`)
- **D26-033**: `[E][CAP/INT]` File effects edit commands/drawer/bindings and add six exact Svelte view files plus decision doc. (`D-26_reasoning_template_dashboard.md:165-178`)
- **D26-034**: `[E][CON/DEP]` D-27 and a full B+C+D progress overlay are out of scope, except the partial `bn_status_report.py --plan=D` hook; D-27 is next. (`D-26_reasoning_template_dashboard.md:180-188`)

### D-27: mode routing refusal dashboard

- **Status**: PLANNED — deps D-10..D-17 shipped; Wave-2 dashboard exposure
- **Superseded:** SUP-016 — D-26/D-27 dashboard surface counts. Target: D-26: 6 commands + 6 components; D-27: 7 commands + 6 components. Loser: Summary thinner command/component lists.
- **Anchors**: plan `D-27_mode_routing_refusal_dashboard.md`; DEEP-D-001..004; aggregate unit (33 declared source atoms); compiled 33
- **Composes with**: D-001/IX-005 — **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-007 — **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance

**Behaviors**

- **D27-001**: `[E][CAP]` D-27 surfaces routing distribution, novelty breakdown, refusal SLA, sycophancy flags, discovery review, and checker health through dashboard-v2. (`D-27_mode_routing_refusal_dashboard.md:1-6,39-44,64-81`)
- **D27-002**: `[E][DEP/DET]` Effort is two sessions; D-10..D-17 must be shipped; D-26 is parallel-safe. (`D-27_mode_routing_refusal_dashboard.md:7-9`)
- **D27-003**: `[E][DEP]` D-26 supplies the pattern, C-7 the tokens, and D-10..D-17 the SSB rows; reads are C-3/C-7 and all Wave-2 plans. (`D-27_mode_routing_refusal_dashboard.md:11-25`)
- **D27-004**: `[E][CAP]` `get_mode_breakdown(optional session_id,window_minutes)` counts `ModeSelected` rows per mode. (`D-27_mode_routing_refusal_dashboard.md:29-32`)
- **D27-005**: `[E][CAP]` `get_refusal_rate(optional session_id)` returns D-17’s `RefusalRateSnapshot` directly. (`D-27_mode_routing_refusal_dashboard.md:33-34`)
- **D27-006**: `[E][CAP]` `get_novelty_breakdown(window_minutes)` counts `NoveltyLabeled` rows by `NoveltyClass`. (`D-27_mode_routing_refusal_dashboard.md:35-36`)
- **D27-007**: `[E][CAP]` `get_sycophancy_flags(limit)` returns recent flags plus claim text. (`D-27_mode_routing_refusal_dashboard.md:37-38`)
- **D27-008**: `[E][CAP]` `get_discovery_queue` returns pending human-review claims from the D-15 queue directory. (`D-27_mode_routing_refusal_dashboard.md:39-40`)
- **D27-009**: `[E][SEC/WF]` `ratify_discovery(claim_id,reviewer)` writes `DiscoveryRatified` and updates the queue file. (`D-27_mode_routing_refusal_dashboard.md:41-42`)
- **D27-010**: `[E][SEC/WF]` `reject_discovery(claim_id,reviewer,reason)` writes `DiscoveryRejected`. (`D-27_mode_routing_refusal_dashboard.md:43-44`)
- **D27-011**: `[E][CAP/OBS]` `ModeRoutingPanel` shows a per-session pie/stacked distribution, live-updates on `ModeSelected`, and reveals percent plus raw count on hover. (`D-27_mode_routing_refusal_dashboard.md:46-50`)
- **D27-012**: `[E][CAP/CFG]` `RefusalRateGauge` spans 0-100%, highlights healthy `[5%,50%]` green and other values red, and opens recent refusal decisions on click. (`D-27_mode_routing_refusal_dashboard.md:52-57`)
- **D27-013**: `[E][CAP]` `NoveltyBreakdown` shows Search/DistantSearch/DiscoveryCandidate bars; clicking the candidate bar opens the review drawer. (`D-27_mode_routing_refusal_dashboard.md:59-62`)
- **D27-014**: `[E][CAP/SEC]` `DiscoveryQueueDrawer` lists claim/evidence/classification time and exposes Ratify with reviewer input and Reject with reason. (`D-27_mode_routing_refusal_dashboard.md:64-69`)
- **D27-015**: `[E][CAP/SEC]` `SycophancyFlagStrip` streams claim ID, Jaccard score, threshold, and a link to quarantined-claim review. (`D-27_mode_routing_refusal_dashboard.md:71-75`)
- **D27-016**: `[E][CAP/OBS]` `CheckerStatusPanel` lists registry entries with ten recent dispatches, pass/fail ratio, last-failure time, and evidence snippet. (`D-27_mode_routing_refusal_dashboard.md:77-81`)
- **D27-017**: `[E][CAP]` Placement is MonitorWindow for routing/refusal, ChatWindow for sycophancy, and a Reasoning tab for novelty/queue/checkers. (`D-27_mode_routing_refusal_dashboard.md:83-86`)
- **D27-018**: `[E][OBS/INT]` Mode, refusal, and sycophancy events use real-time SSB subscription. (`D-27_mode_routing_refusal_dashboard.md:88`)
- **D27-019**: `[E][CON]` Decisions justify dashboard review over CLI-only UX, a pie chart for five categorical modes, and a gauge for one value plus healthy band. (`D-27_mode_routing_refusal_dashboard.md:90-96`)
- **D27-020**: `[E][INT]` Produced contract contains seven commands and six components, including the two review/checker components omitted from the D-0 status note. (`D-27_mode_routing_refusal_dashboard.md:98-106`)
- **D27-021**: `[E][DEP/INT]` Consumes `ModeSelected`, `RefusalDecision`, `NoveltyLabeled`, `SycophancyFlag`, checker dispatch/pass/fail events, `RefusalRateMonitor`, `DiscoveryQueueEntry`, and both SSB reader/writer. (`D-27_mode_routing_refusal_dashboard.md:107-112`)
- **D27-022**: `[E][CAP]` Reachability is through MonitorWindow, ChatWindow, and ReasoningDrawer tabs. (`D-27_mode_routing_refusal_dashboard.md:113-114`)
- **D27-023**: `[E][SEC]` Ratify/reject must require a reviewer; anonymous ratification is forbidden. (`D-27_mode_routing_refusal_dashboard.md:115-117`)
- **D27-024**: `[E][SEC]` Every sycophancy flag must link to its quarantined claim. (`D-27_mode_routing_refusal_dashboard.md:118`)
- **D27-025**: `[E][CFG/SEC]` Healthy-band thresholds must load from D-17 config and cannot be hardcoded in Svelte. (`D-27_mode_routing_refusal_dashboard.md:119-120`)
- **D27-026**: `[E][OBS/TEST]` A mixed cloud task must feed existing events and render routing, refusal, and pending discovery state. (`D-27_mode_routing_refusal_dashboard.md:121-126`)
- **D27-027**: `[E][TEST]` Deterministic checks grep command/component presence and run Svelte plus Rust checks. (`D-27_mode_routing_refusal_dashboard.md:127-135`)
- **D27-028**: `[E][TEST]` Acceptance requires all seven commands and six components to compile/type-check, clean ts-rs drift, and zero new Rust errors. (`D-27_mode_routing_refusal_dashboard.md:137-142`)
- **D27-029**: `[E][TEST]` Cloud A requires a realistic five-mode distribution; Cloud B requires an out-of-band high-refusal session to turn the gauge red. (`D-27_mode_routing_refusal_dashboard.md:143-146`)
- **D27-030**: `[E][TEST/SEC]` Cloud C increments DiscoveryCandidate/queue for a not-in-DKG primitive; Cloud D surfaces a sycophancy fixture within the poll interval; Cloud E ratifies a queue entry, writes `DiscoveryRatified`, and updates its status. (`D-27_mode_routing_refusal_dashboard.md:147-154`)
- **D27-031**: `[E][CAP/INT]` File effects edit commands plus Monitor/Chat/Reasoning views and add six exact component files plus decision doc. (`D-27_mode_routing_refusal_dashboard.md:156-169`)
- **D27-032**: `[E][CON/STATUS]` Discovery-review TUI, multi-reviewer ratification, and cross-plan progress overlay are deferred/out of scope. (`D-27_mode_routing_refusal_dashboard.md:171-175`)
- **D27-033**: `[E][STATUS]` D-27 declares Wave 4 complete and Wave 5 research-grade/deferred. (`D-27_mode_routing_refusal_dashboard.md:177-179`)

### D-29: hierarchical bai research

- **Status**: mixed / partial — Path A host Rust dispatch claimed shipped; Gibbs/RunPod research closure DEFERRED (SUP-005)
- **Superseded:** SUP-005 — D-29 hierarchical BAI status. Target: Mixed: Path A host Rust dispatch claimed shipped; Gibbs/RunPod research closure still DEFERRED. Loser: Summary/D-0 clean DEFERRED only.
- **Anchors**: plan `D-29_hierarchical_bai_research.md`; DEEP-D-001..004; aggregate unit (28 declared source atoms); compiled 28
- **Composes with**: D-001/IX-008 — **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D | D-001/IX-010 — **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not indepen

**Behaviors**

- **D29-001**: `[E][CAP/STATUS]` D-29 addresses non-IID session rewards with hierarchical Bayesian BAI and accepts a concrete infeasibility note as valid closure. (`D-29_hierarchical_bai_research.md:1-7`)
- **D29-002**: `[E][STATUS/DEP/DET]` Header status is `DEFERRED`; effort is 4-8 wall-clock sessions plus RunPod; D-5 flat BAI and RunPod are prerequisites; D-28/D-30 are parallel-safe. (`D-29_hierarchical_bai_research.md:8-12`)
- **D29-003**: `[E][DATA]` Proposed hierarchy has per-arm pass/fail, a per-family hyperprior over arm pass rates, and pooled within-family posterior information. (`D-29_hierarchical_bai_research.md:14-20`)
- **D29-004**: `[E][CON/STATUS]` The source asserts possible 30-50% regret reduction from broader hierarchical literature but explicitly says that literature is not yet verified; it also warns ELAI’s structure may yield no gain. (`D-29_hierarchical_bai_research.md:21-27`)
- **D29-005**: `[E][DEP]` Required reads are the companion BAI/Q4 sections, Audibert and variance-BAI interfaces, and D-5 flat implementation. (`D-29_hierarchical_bai_research.md:29-35`)
- **D29-006**: `[E][INT/DEP]` At unblock time, paper-ingestor must select hierarchical-bandit literature and create `workflow/papers/hierarchical-bandit-<ref>-interface.md`. (`D-29_hierarchical_bai_research.md:37-44`)
- **D29-007**: `[E][DATA/CAP]` Proposed solver at `.elai_cc/python/bai_hierarchical/` uses per-arm Beta distributions, Gamma (or similar) hyperpriors, and Gibbs sampling for about 50 arms x 200 evaluations. (`D-29_hierarchical_bai_research.md:46-52`)
- **D29-008**: `[E][TEST]` Synthetic benchmark has five families, ten arms per family, within-family correlation, cross-family independence, and compares flat versus hierarchical regret over 1000 pulls. (`D-29_hierarchical_bai_research.md:54-57`)
- **D29-009**: `[E][STATUS/TEST]` Path A succeeds when hierarchical regret is no worse than flat and then productizes the solver. (`D-29_hierarchical_bai_research.md:59-60`)
- **D29-010**: `[E][STATUS/FAIL]` Path B closes with an infeasibility note and at least one concrete blocker when hierarchy is equal/worse, such as unjustified compute or weak non-IID structure. (`D-29_hierarchical_bai_research.md:61-63`)
- **D29-011**: `[E][CFG/FAIL]` If Path A, D-5 integration is gated by `ELAI_BAI_MODE=hierarchical`; default remains flat. (`D-29_hierarchical_bai_research.md:65-66`)
- **D29-012**: `[E][STATUS]` Decision doc is drafted only when the deferred work unblocks. (`D-29_hierarchical_bai_research.md:68-69`)
- **D29-013**: `[E][DATA]` The later wiring contract claims concrete `HierarchicalBaiState` at `bai_hierarchical.rs:271`, `FamilyHyperprior` at `:202`, `FamilyId` at `:115`, and `SelectionBasis` at `:246`. (`D-29_hierarchical_bai_research.md:71-75`)
- **D29-014**: `[E][CAP]` Claimed functions are `family_id_for_domain`/`family_id_for_template_or_mixed` at lines 159/183, `hierarchical_update` at 509, `hierarchical_arm_select` at 605, and `total_variance_decomposition` at 659. (`D-29_hierarchical_bai_research.md:76-78`)
- **D29-015**: `[E][DATA/INT/CFG]` Claimed production wiring adds `HierarchicalBaiRoundCompleted`/payload at `ssb.rs:6725` and `BaiMode::Hierarchical` at `runtime_config.rs:440`, selected by `ELAI_BAI_MODE`. (`D-29_hierarchical_bai_research.md:79-80`)
- **D29-016**: `[E][C][STATUS]` A contract comment says Path A “shipped” and Path B was not the outcome, conflicting with the header/acceptance `DEFERRED` state. (`D-29_hierarchical_bai_research.md:81`)
- **D29-017**: `[E][STATUS]` The Python Gibbs solver/benchmark remains `DEFERRED/RUNPOD-PENDING` even while host-side Rust productization is claimed. (`D-29_hierarchical_bai_research.md:82`)
- **D29-018**: `[E][DEP/INT]` Consumes a D-5 `darwinian_bai` flat baseline/default discount factor, `BaiMode::from_env`, template/domain assignment, and `SsbWriter`. (`D-29_hierarchical_bai_research.md:83-87`)
- **D29-019**: `[E][WF/CFG]` Runtime reachability is a `task_runner.rs:11406` hierarchical promotion branch enabled only when both `ELAI_BAI_MODE=hierarchical` and `ELAI_BAI_ENABLE=1`; unset mode defaults flat/wire-dark. (`D-29_hierarchical_bai_research.md:88-90`)
- **D29-020**: `[E][STATUS/TEST]` The 5-family/10-arm/1000-pull Gibbs regret comparison is still RunPod-pending. (`D-29_hierarchical_bai_research.md:91-92`)
- **D29-021**: `[E][CON]` `hierarchical_arm_select` must record/use `SelectionBasis` and cannot silently fall through to flat. (`D-29_hierarchical_bai_research.md:93-94`)
- **D29-022**: `[E][OBS/FAIL]` Every hierarchical round must emit `HierarchicalBaiRoundCompleted`; mode must default `Flat` when the environment value is absent. (`D-29_hierarchical_bai_research.md:95-96`)
- **D29-023**: `[E][OBS/TEST]` A hierarchical task expects one round row with `gibbs_iterations > 0` and recorded `SelectionBasis`; the Rust dispatch is explicitly called wired/live while research closure remains pending. (`D-29_hierarchical_bai_research.md:97-103`)
- **D29-024**: `[E][DET/STATUS]` `DETERMINISTIC_CHECK` is unpopulated and deferred to unblock time. (`D-29_hierarchical_bai_research.md:105-107`)
- **D29-025**: `[E][STATUS/DEP]` Acceptance remains unchecked/deferred until stable D-5 baseline, RunPod CPU, and paper interface exist. (`D-29_hierarchical_bai_research.md:109-113`)
- **D29-026**: `[E][TEST/STATUS]` Closure still requires the synthetic report, either Path A or Path B, and the decision doc. (`D-29_hierarchical_bai_research.md:114-117`)
- **D29-027**: `[E][CAP/STATUS]` Files are still labeled “anticipated”: Python solver, Rust `bai_hierarchical.rs` if Path A, and decision doc. (`D-29_hierarchical_bai_research.md:119-123`)
- **D29-028**: `[E][CON/DEP]` More than two hierarchy levels and online updates are deferred; D-29 remains independent of D-28/D-30. (`D-29_hierarchical_bai_research.md:125-132`)

### D-1: reasoning template schema

- **Status**: PLANNED — deps C-15, C-21; Phase 4H.IV / typed templates and chain
- **Superseded:** SUP-002 — D-1 archive hash formula. Target: Unresolved conflict preserved: D-1 two-part (body+parent) vs D-8 three-part (+mutation_proposal_text). Loser: Neither fully supersedes; target state requires reconciliation before implement.
- **Superseded:** SUP-015 — D-1 staged_eval_result field name. Target: staged_eval_result (+ retirement_reason). Loser: Summary eval_result projection.
- **Anchors**: plan `D-1_reasoning_template_schema.md`; DEEP-D-001..004; aggregate unit (57 declared source atoms); compiled 57
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-003 — **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks.  | D-001/IX-004 — **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lac | D-001/IX-005 — **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-007 — **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance

**Behaviors**

- **D1-E01**: [[E]][CAP,DATA,SEC] Replace prompt-string reasoning patterns with bounded typed ReasoningTemplate records and bootstrap an append-only ArchiveEntry chain as the mutation substrate. (`D-1_reasoning_template_schema.md:3-9`)
- **D1-E02**: [[E]][OBS,DEP] The plan records the then-current gap as no ReasoningTemplate, roughly 25 TOML prompt files plus inline agent strings, and no hash, lineage, or archive lifecycle. (`D-1_reasoning_template_schema.md:22-30`)
- **D1-E03**: [[E]][SEC,CON] Extend L1A from data tables to data tables plus typed reasoning templates; the closed schema is the intended capacity bound. (`D-1_reasoning_template_schema.md:39-42`)
- **D1-E04**: [[E]][DEP,FLOW] Estimate 1-2 sessions; require C-15 and C-21 DONE, with spec-grounder verification before EXECUTING; use C-15 Blake3 and C-21 MistakeBook BLOCK inputs. (`D-1_reasoning_template_schema.md:10-16`)
- **D1-E05**: [[E]][DET,DEP] Parallel-safe with D-19 and D-25; not parallel-safe with D-2 through D-9 because those consume the new records. (`D-1_reasoning_template_schema.md:17-20`)
- **D1-E06**: [[E]][DEP,TEST] Ingest SELF-DISCOVER and Statistical Limits interfaces, and require both files before TESTING_FIRST. (`D-1_reasoning_template_schema.md:76-85`)
- **D1-E07**: [[E]][DATA] ReasoningTemplate fields are id, domain, pattern_kind, template_body as BoundedString<1024>, eligibility_predicate, verifier_required, content_hash, created_at, parent_id, and archive_status. (`D-1_reasoning_template_schema.md:89-95`)
- **D1-E08**: [[E]][DATA,SEC] Domain is the L3-locked closed enum Code, Math, Prose, Mixed. (`D-1_reasoning_template_schema.md:96-97`)
- **D1-E09**: [[E]][DATA,SEC] PatternKind is the L3-locked closed enum Decomposition, Reflection, Verification, Synthesis. (`D-1_reasoning_template_schema.md:98-99`)
- **D1-E10**: [[E]][DATA,SEC,FAIL] VerifierKind is the L3-locked closed enum Compile, Test, Prm, Refuse; an agent may not weaken Prm to Refuse. (`D-1_reasoning_template_schema.md:100-101`)
- **D1-E11**: [[E]][DATA,FLOW] ArchiveStatus values are Candidate, Promoted, Retired, Frozen; RetirementReason values are Degradation, Superseded, Banned, MistakeBookTrigger. (`D-1_reasoning_template_schema.md:102-103`)
- **D1-E12**: [[E]][DATA] TemplateId is a Uuid newtype and Blake3Hash is a 32-byte-array newtype. (`D-1_reasoning_template_schema.md:104`)
- **D1-E13**: [[E]][DATA,FAIL,CON] BoundedString<N> validates in TryFrom, rejects lengths greater than N without panic, and returns BoundedStringError. (`D-1_reasoning_template_schema.md:105-106`)
- **D1-E14**: [[E]][DATA,SEC,CON] Predicate is the closed, non-LLM enum Always, MatchDomain, MinConfidence, AndAll, OrAny; recursive fan-out is bounded by vector length. (`D-1_reasoning_template_schema.md:107-109,265-266`)
- **D1-E15**: [[E]][DATA,FAIL,CON] All types derive Debug, Clone, Serialize, Deserialize; TemplateError has BoundedStringExceeded, HashMismatch, ParentNotFound, StatusTransitionInvalid; new code forbids unwrap and expect. (`D-1_reasoning_template_schema.md:110-112,353`)
- **D1-E16**: [[E]][DET,SEC] compute_content_hash hashes body bytes concatenated with the supplied parent hash, or 32 zero bytes for a root, using Blake3; identical inputs must produce the same hash. (`D-1_reasoning_template_schema.md:114-117`)
- **D1-E17**: [[E]][FAIL,SEC] verify_hash recomputes a template hash and returns HashMismatch on divergence; it may not compare cached values. (`D-1_reasoning_template_schema.md:118-119,259-260`)
- **D1-E18**: [[E]][TEST,DET] Hash tests require equality for identical body/parent, inequality when body or parent differs, and exactly 32 output bytes. (`D-1_reasoning_template_schema.md:120-121`)
- **D1-E19**: [[E]][DATA,FLOW] ArchiveEntry fields are self_hash, parent_hash, template, staged_eval_result, promoted_at, retired_at, retirement_reason. (`D-1_reasoning_template_schema.md:123-128`)
- **D1-E20**: [[E]][DATA,OBS] StagedEvalResult carries pass/total counts at stages 10, 50, and 200 plus cross_domain_deltas by Domain; D-5 and D-7 populate the placeholder. (`D-1_reasoning_template_schema.md:129-132`)
- **D1-E21**: [[E]][DATA,FLOW,FAIL,SEC] ArchiveChain has storage_path and entries; load, append, and verify_chain are required; append never deletes, while verification walks parent links and recomputes hashes. (`D-1_reasoning_template_schema.md:133-135`)
- **D1-E22**: [[E]][DATA,SEC] Store one serialized ArchiveEntry per JSONL line at .elai_cc/archive/reasoning-templates.jsonl and open writes in O_APPEND mode only. (`D-1_reasoning_template_schema.md:136-137`)
- **D1-E23**: [[E]][DET,DEP] Serialize concurrent appends with an fs2 advisory lock; if fs2 is absent, it is the sole optional dependency and requires user confirmation before addition. (`D-1_reasoning_template_schema.md:138-140`)
- **D1-E24**: [[E]][INT,DATA,OBS] Bump SSB SCHEMA_VERSION 47 to 48 and add ArchiveEntryAdded, ArchiveEntryRetired, TemplatePromoted, and TemplateCandidateRejected with their listed payload fields, SQL serialization, and round-trip coverage. (`D-1_reasoning_template_schema.md:142-152`)
- **D1-E25**: [[E]][CAP,FLOW,DATA] elai template migrate scans prompt-template TOML and agent-definition Markdown, extracts unique patterns, hashes each, creates Frozen ReasoningTemplate entries, and writes the archive JSONL. (`D-1_reasoning_template_schema.md:154-159`)
- **D1-E26**: [[E]][DET,OBS] Migration is idempotent on unchanged inputs, skips existing hashes, and reports extracted, skipped_duplicate, and skipped_too_long counts. (`D-1_reasoning_template_schema.md:160-162`)
- **D1-E27**: [[E]][DATA,CON] TemplateRegistry maps TemplateId to ReasoningTemplate with FxHashMap, explicitly not std HashMap. (`D-1_reasoning_template_schema.md:164-167`)
- **D1-E28**: [[E]][FLOW] TemplateRegistry.load admits only Promoted and Frozen archive entries. (`D-1_reasoning_template_schema.md:168-169`)
- **D1-E29**: [[E]][FLOW,DEP] select_template filters by domain, pattern kind, and a closed-form eligibility predicate, then returns the first match; replacing task-runner prompt literals is deferred to D-2. (`D-1_reasoning_template_schema.md:170-174`)
- **D1-E30**: [[E]][CFG] Add a commented reasoning.templates config section; absent values default archive_path to .elai_cc/archive/reasoning-templates.jsonl and bounded_string_max to 1024. (`D-1_reasoning_template_schema.md:176-180`)
- **D1-E31**: [[E]][TEST] Unit coverage includes 1024/1025 string boundaries, repeatable hash, body-tamper detection, append/load identity, broken-parent detection, and predicate truth tables. (`D-1_reasoning_template_schema.md:182-192`)
- **D1-E32**: [[E]][FLOW,SEC,TEST] Candidate to Promoted and Promoted to Retired are valid; Frozen and Retired are immutable terminal states, with no force-override flag. (`D-1_reasoning_template_schema.md:193-194,261-262`)
- **D1-E33**: [[E]][TEST,DATA] The contract test requires public schema 48, all four SSB variants and SQL round trips, allow-listed closed enums, and append-plus-verify end to end. (`D-1_reasoning_template_schema.md:197-203`)
- **D1-E34**: [[E]][TEST,OBS] Manual migration smoke reports all three counts and inspects the first three JSONL entries for valid JSON, 64-character hashes, and Frozen status. (`D-1_reasoning_template_schema.md:205-209`)
- **D1-E35**: [[E]][CON,OBS] D-1-decisions.md must record why Blake3, why JSONL instead of SQLite, and why migrated templates are Frozen rather than Promoted. (`D-1_reasoning_template_schema.md:211-218`)
- **D1-E36**: [[E]][FLOW,OBS,DEP] During MEMORIZING, memory-writer must emit project_d1_reasoning_template.md containing schema, lifecycle, and SSB variants for D-2 through D-9. (`D-1_reasoning_template_schema.md:220-222`)
- **D1-E37**: [[E]][DEP,DET] created_at must use injected SeededClock and never SystemTime::now; MistakeBook retirement feed is planned for D-4, not wired in D-1. (`D-1_reasoning_template_schema.md:239-248`)
- **D1-E38**: [[E]][CAP,INT] Reachability is through elai template migrate, registry startup loading, and future D sessions reading ArchiveChain through an in-memory FxHashMap after JSONL load. (`D-1_reasoning_template_schema.md:250-254`)
- **D1-E39**: [[E]][CON,SEC] ArchiveChain may expose no delete method; content hashing must never use SHA-256 or Blake2; Predicate evaluation may not call an LLM. (`D-1_reasoning_template_schema.md:255-266`)
- **D1-E40**: [[E]][OBS,TEST] A migration run must yield N ArchiveEntryAdded rows matching its reported count, all Frozen, with zero TemplatePromoted and TemplateCandidateRejected rows; the whole artifact must parse and verify. (`D-1_reasoning_template_schema.md:267-274`)
- **D1-E41**: [[E]][TEST,FAIL] Acceptance requires public schema 48, a migration warning for an old version-47 SSB database, error-return rather than panic, at least eight new unit tests, contract tests, and a clean workspace build. (`D-1_reasoning_template_schema.md:325-341`)
- **D1-E42**: [[E]][TEST,FAIL,SEC] Cloud smoke A verifies registry loading and task completion with a migrated template; smoke B manually tampers template_body and requires HashMismatch plus orchestrator refusal to use it. (`D-1_reasoning_template_schema.md:342-351`)
- **D1-E43**: [[E]][TEST,OBS] Real-repository migration must migrate at least as many templates as the template-directory TOML file count, be idempotent on the second run, and produce the decision and memory artifacts. (`D-1_reasoning_template_schema.md:335-354`)
- **D1-E44**: [[E]][CON,DEP] Out of scope are PPM wiring, BAI promotion, user-facing archive verification, per-template performance metrics, and enforcement of documented L3 locks; these belong to D-2, D-5, D-8, D-7, and D-9 respectively. (`D-1_reasoning_template_schema.md:379-389`)
- **D1-E45**: [[E]][DEP,DET] After D-1, D-2 and D-8 may start in parallel: D-2 adds verifier substrate and D-8 exposes verify_chain as CLI. (`D-1_reasoning_template_schema.md:391-395`)
- **D1-N01**: [[A]][DEP] The original never declares PLANNED, IN PROGRESS, or DONE; PLANNED comes only from the orientation summary, and this pass did not inspect implementation. (`D-1_reasoning_template_schema.md:1-395`)
- **D1-N02**: [[C]][DATA,SEC] D-1 defines template content_hash from body plus parent hash, while D-8 defines archive self-hash from body plus parent hash plus mutation_proposal_text. (`D-1_reasoning_template_schema.md:114-121; D-8_crypto_archive_verify.md:16-28`)
- **D1-N03**: [[A]][DATA] Neither ReasoningTemplate nor ArchiveEntry declares mutation_proposal_text, leaving the D-8 hash input without a storage field in D-1's schema. (`D-1_reasoning_template_schema.md:91-95,125-128; D-8_crypto_archive_verify.md:16-28`)
- **D1-N04**: [[A]][FLOW] StatusTransitionInvalid and transition tests exist, but the implementing transition type or method and the append-only representation of promotion/retirement are not specified. (`D-1_reasoning_template_schema.md:111-112,193-194`)
- **D1-N05**: [[A]][DATA,TEST] Predicate tests permit adding Never if needed, although Never is absent from the declared closed enum; its normative membership is unresolved. (`D-1_reasoning_template_schema.md:107-109,191-192`)
- **D1-N06**: [[A]][CFG,DATA] BoundedString<1024> is compile-time-sized while bounded_string_max is runtime-configurable; behavior when configuration differs from 1024 is not defined. (`D-1_reasoning_template_schema.md:92-95,176-180`)
- **D1-N07**: [[A]][DET,FLOW] select_template returns the first eligible FxHashMap result, but tie ordering and a stable selection key are not specified. (`D-1_reasoning_template_schema.md:164-172`)
- **D1-N08**: [[A]][FLOW,SEC] The file requires immutable append-only storage and mutable lifecycle states but does not define whether transitions append successor entries, append events, or rewrite prior JSONL rows. (`D-1_reasoning_template_schema.md:102-103,123-137,193-194`)
- **D1-N09**: [[A]][CAP,DET] Pattern extraction and deduplication rules for free-form TOML/Markdown are not defined beyond uniqueness by resulting hashes. (`D-1_reasoning_template_schema.md:154-162`)
- **D1-N10**: [[A]][DATA,SEC] `ReasoningTemplate.content_hash`, `ArchiveEntry.self_hash`, and the value recomputed by `verify_chain` are all named, but D-1 never states whether the two hashes must be identical or whether an entry has a second envelope hash. (`D-1_reasoning_template_schema.md:91-95,114-128,133-137`)
- **D1-N11**: [[A]][DATA,FLOW] Template lineage uses `parent_id: Option<TemplateId>` while archive lineage uses `parent_hash: Option<Blake3Hash>`; lookup, validation, and consistency rules between those two parent references are not specified. (`D-1_reasoning_template_schema.md:91-95,125-135`)
- **D1-N12**: [[A]][DATA,SEC,CON] The source calls predicate recursion bounded by vector length, but supplies no maximum vector length, nesting depth, cycle defense, or evaluation budget; the effective capacity bound is therefore unspecified. (`D-1_reasoning_template_schema.md:107-109,265-266`)

### D-8: crypto archive verify

- **Status**: PLANNED — deps D-1; Phase 4H.IX / archive tamper verification
- **Superseded:** SUP-002 — D-1 archive hash formula. Target: Unresolved conflict preserved: D-1 two-part (body+parent) vs D-8 three-part (+mutation_proposal_text). Loser: Neither fully supersedes; target state requires reconciliation before implement.
- **Anchors**: plan `D-8_crypto_archive_verify.md`; DEEP-D-001..004; aggregate unit (41 declared source atoms); compiled 41
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.

**Behaviors**

- **D8-E01**: [[E]][CAP,SEC,OBS] Expose D-1 ArchiveChain.verify_chain as a user CLI and CI check, emit ArchiveVerifyResult, and defend against an agent rewriting its audit history. (`D-8_crypto_archive_verify.md:3-8`)
- **D8-E02**: [[E]][DEP,DET] Require D-1, with spec-grounder verification; run in parallel with D-2 through D-7 because D-8 reads but does not extend ArchiveChain beyond reporting. (`D-8_crypto_archive_verify.md:9-12`)
- **D8-E03**: [[E]][DATA,SEC] The context defines each self hash as Blake3 over template_body, parent_hash, and mutation_proposal_text. (`D-8_crypto_archive_verify.md:16-19,25-27`)
- **D8-E04**: [[E]][FAIL,SEC] Chain verification recomputes every hash, requires each parent to be an earlier existing entry, and reports hash/link/parent failures. (`D-8_crypto_archive_verify.md:25-28`)
- **D8-E05**: [[E]][CAP,CFG] ArchiveVerifyCmd contains archive_path, json_output, and fail_fast. (`D-8_crypto_archive_verify.md:52-55`)
- **D8-E06**: [[E]][DATA,OBS] ArchiveVerifyReport contains total_entries, verified, and a vector of ArchiveVerifyError. (`D-8_crypto_archive_verify.md:56-57`)
- **D8-E07**: [[E]][DATA,FAIL] ArchiveVerifyError variants are HashMismatch with expected/computed and index, BrokenParent with index/hash, DuplicateHash with current/prior indexes, and MalformedJson with index/error. (`D-8_crypto_archive_verify.md:58-61`)
- **D8-E08**: [[E]][FLOW,FAIL] run loads the archive, calls verify_chain, aggregates all errors by default, and short-circuits on the first error only when fail_fast is true. (`D-8_crypto_archive_verify.md:62-66`)
- **D8-E09**: [[E]][FAIL,OBS] Output is human-readable or JSON; zero errors exits 0, any error exits 1, and an ArchiveVerifyResult row is emitted regardless of exit code. (`D-8_crypto_archive_verify.md:67-69`)
- **D8-E10**: [[E]][CAP,CFG] Add SlashCommand::ArchiveVerify and dispatch elai archive verify with path, json, and fail-fast flags; default to the D-1 archive path. (`D-8_crypto_archive_verify.md:71-77`)
- **D8-E11**: [[E]][INT,DATA,OBS] Bump SSB schema 53 to 54 and add ArchiveVerifyResult with audit_id, total_entries, verified, error_count, and errors, plus round-trip coverage. (`D-8_crypto_archive_verify.md:79-82`)
- **D8-E12**: [[E]][INT,TEST,SEC] archive-verify.yml runs on push and pull_request to main plus nightly cron, builds release elai_cli, and invokes archive verify with fail-fast so integrity failure fails CI. (`D-8_crypto_archive_verify.md:84-89`)
- **D8-E13**: [[E]][INT,CFG] CI uses windows-latest and ubuntu-latest, and skips when the commit message contains [ci skip]. (`D-8_crypto_archive_verify.md:90-92`)
- **D8-E14**: [[E]][TEST,FAIL] Tamper scenario A edits entry 3 template_body and must report HashMismatch at entry 3 and exit 1. (`D-8_crypto_archive_verify.md:94-98`)
- **D8-E15**: [[E]][TEST,FAIL] Tamper scenario B points entry 4 at a nonexistent parent and must report BrokenParent and exit 1. (`D-8_crypto_archive_verify.md:99-100`)
- **D8-E16**: [[E]][TEST,FAIL] Tamper scenario C duplicates entry 2's self hash at entry 5 and must report DuplicateHash and exit 1. (`D-8_crypto_archive_verify.md:101-102`)
- **D8-E17**: [[E]][TEST,FAIL] Tamper scenario D inserts malformed JSON and must report MalformedJson while continuing when fail_fast is false. (`D-8_crypto_archive_verify.md:103-104`)
- **D8-E18**: [[E]][TEST,OBS] A clean five-entry archive exits 0 and emits ArchiveVerifyResult with error_count zero. (`D-8_crypto_archive_verify.md:105-106`)
- **D8-E19**: [[E]][TEST,DET,CON] An ignored manual performance test synthesizes 10,000 entries and requires verification within five seconds and O(n), not O(n-squared), behavior. (`D-8_crypto_archive_verify.md:108-113`)
- **D8-E20**: [[E]][FAIL,OBS] Documentation must cover CLI/CI and recovery: after failure, treat the last N agent actions as suspect and roll back by forensic chain walk. (`D-8_crypto_archive_verify.md:115-119`)
- **D8-E21**: [[E]][CON,CFG] The decision record explains separate CLI/CI roles, Blake3 speed/consistency, and full-report default versus early-exit fail-fast mode. (`D-8_crypto_archive_verify.md:121-128`)
- **D8-E22**: [[E]][DEP,CAP] The command consumes ArchiveChain, SlashCommand, and SsbWriter and is reachable from direct CLI, CI, and cargo-run development invocation. (`D-8_crypto_archive_verify.md:130-145`)
- **D8-E23**: [[E]][CON,FAIL] I/O may not unwrap; clean must not always fail; tamper must not silently pass; json and fail-fast flags must be functional. No retry behavior is defined. (`D-8_crypto_archive_verify.md:146-151`)
- **D8-E24**: [[E]][OBS,TEST] Each clean or tampered invocation emits exactly one ArchiveVerifyResult; the workflow artifact must report a PR commit status. (`D-8_crypto_archive_verify.md:152-157`)
- **D8-E25**: [[E]][TEST] Deterministic checks cover schema 54, command/types/event/workflow presence, clean verification, synthetic corrupt-line detection, workspace build, contract tests, and the ignored performance test. (`D-8_crypto_archive_verify.md:159-194`)
- **D8-E26**: [[E]][TEST,FAIL] Acceptance additionally requires parseable JSON and demonstrates that default collection reports at least two errors when at least two tampers exist. (`D-8_crypto_archive_verify.md:196-211`)
- **D8-E27**: [[E]][TEST,SEC] Cloud smoke verifies a clean archive after live promotion, then a manual body corruption that exits 1 with HashMismatch. (`D-8_crypto_archive_verify.md:212-217`)
- **D8-E28**: [[E]][CAP,INT] Files include the CLI module/dispatch, SSB update, CI workflow, contract and performance tests, decision record, and recovery documentation. (`D-8_crypto_archive_verify.md:219-229`)
- **D8-E29**: [[E]][CON,DEP] D-25's ten-case suite and D-9 lock enforcement are out of scope; archive compaction or sled migration is deferred because JSONL is expected to scale to about 10,000 entries. (`D-8_crypto_archive_verify.md:231-241`)
- **D8-N01**: [[C]][DATA,SEC] Its three-part hash formula conflicts with D-1's two-part compute_content_hash formula and depends on mutation_proposal_text absent from D-1's records. (`D-8_crypto_archive_verify.md:16-28; D-1_reasoning_template_schema.md:91-95,114-128`)
- **D8-N02**: [[C]][FAIL] Context names BrokenLink and DanglingParent, while the normative error enum names BrokenParent and has no BrokenLink or DanglingParent variant. (`D-8_crypto_archive_verify.md:25-28,58-61`)
- **D8-N03**: [[A]][FAIL,FLOW] ArchiveChain load precedes verification, but malformed JSON is expected to be an aggregatable per-line error; how load preserves later lines is unspecified. (`D-8_crypto_archive_verify.md:62-66,103-104`)
- **D8-N04**: [[A]][FAIL] Missing file, permission denial, unreadable path, and empty-archive behavior are not defined. (`D-8_crypto_archive_verify.md:52-69`)
- **D8-N05**: [[A]][OBS,FAIL] The report promises an SSB row regardless of exit code, but no path is specified for emitting it when loading or SSB writing itself fails. (`D-8_crypto_archive_verify.md:62-69`)
- **D8-N06**: [[A]][FAIL,SEC] Recovery identifies suspect actions and a forensic walk but does not define the trusted restore source, exact rollback command, or how the append-only file is repaired. (`D-8_crypto_archive_verify.md:115-119`)
- **D8-N07**: [[A]][INT] The plan places workflows beneath .elai_cc/.github; it does not state whether .elai_cc is the repository root or how a parent-repository CI installation discovers them. (`D-8_crypto_archive_verify.md:84-92,219-229`)
- **D8-N08**: [[C]][DEP,FAIL] D-8 delegates to D-1 `verify_chain`, yet D-1 exposes only template-level `HashMismatch`/`ParentNotFound` errors and does not define duplicate-hash or malformed-line aggregation; D-8 requires four richer per-entry error variants without specifying the adapter boundary. (`D-8_crypto_archive_verify.md:52-69,94-106; D-1_reasoning_template_schema.md:110-119,133-137`)
- **D8-N09**: [[A]][DET,FLOW,SEC] Verification concurrency is undefined: the source does not say whether the reader takes D-1's advisory lock, snapshots the file, or can observe a partial line while another process appends. (`D-8_crypto_archive_verify.md:62-69; D-1_reasoning_template_schema.md:133-140`)
- **D8-N10**: [[A]][SEC,INT] `[ci skip]` disables the integrity workflow, but no authorization policy or protected-branch control constrains who may use that bypass. (`D-8_crypto_archive_verify.md:84-92`)
- **D8-N11**: [[A]][DATA,FAIL] Error `entry_index` semantics are unspecified (zero- versus one-based, and whether malformed physical lines count), which affects machine consumers and the recovery walk. (`D-8_crypto_archive_verify.md:56-61,94-106`)
- **D8-N12**: [[C]][TEST] Acceptance calls the five cases “5 tamper scenarios” although one of the five is explicitly the clean control and only four are tampers. (`D-8_crypto_archive_verify.md:94-106,198-204`)

### D-9: l3 frozen surfaces

- **Status**: PLANNED — deps D-1..D-8; companion §§2.3, 4 / frozen-surface block
- **Anchors**: plan `D-9_l3_frozen_surfaces.md`; DEEP-D-001..004; aggregate unit (52 declared source atoms); compiled 52
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-9

**Behaviors**

- **D9-E01**: [[E]][CAP,SEC] Structurally enforce expanded L3-frozen surfaces with a pre-commit hook, LockedKeysAuditor, LockedKeysTouch event, and a blocked agent edit path. (`D-9_l3_frozen_surfaces.md:3-8`)
- **D9-E02**: [[E]][DEP,DET] Require D-1 through D-8 DONE; run in parallel with D-10 through D-18; coordinate hook ordering with C-27 rather than running that overlapping change independently. (`D-9_l3_frozen_surfaces.md:9-15`)
- **D9-E03**: [[E]][SEC] Freeze staged-evaluation harness code. (`D-9_l3_frozen_surfaces.md:19-22`)
- **D9-E04**: [[E]][SEC] Freeze tripwire thresholds and detection code. (`D-9_l3_frozen_surfaces.md:22,31-32`)
- **D9-E05**: [[E]][SEC] Freeze MSI/BAI scoring-function code. (`D-9_l3_frozen_surfaces.md:23,33`)
- **D9-E06**: [[E]][SEC] Freeze archive maintenance and parent-selection algorithm code. (`D-9_l3_frozen_surfaces.md:24`)
- **D9-E07**: [[E]][SEC] Freeze the held-out task-selection algorithm. (`D-9_l3_frozen_surfaces.md:25`)
- **D9-E08**: [[E]][SEC] Freeze cross-domain canary family definitions. (`D-9_l3_frozen_surfaces.md:26`)
- **D9-E09**: [[E]][SEC] Freeze cryptographic hash choice to Blake3. (`D-9_l3_frozen_surfaces.md:27`)
- **D9-E10**: [[E]][SEC] Freeze the AllowedConfigKey bound so an agent cannot add variants. (`D-9_l3_frozen_surfaces.md:28`)
- **D9-E11**: [[E]][SEC] Freeze every template's verifier_required field against self-weakening. (`D-9_l3_frozen_surfaces.md:29`)
- **D9-E12**: [[E]][SEC] Freeze the Domain enum against self-added domains. (`D-9_l3_frozen_surfaces.md:30`)
- **D9-E13**: [[E]][SEC,CFG] Freeze tripwire config keys kl_spec_tau, rho_delta, iqr_mult, ppl_sigma, ddm thresholds, consensus_fire_count, and window_size. (`D-9_l3_frozen_surfaces.md:31-32`)
- **D9-E14**: [[E]][SEC,CFG] Freeze BAI eval_budget, ci_alpha, and per-family delta. (`D-9_l3_frozen_surfaces.md:33`)
- **D9-E15**: [[E]][SEC,CFG] Freeze held-out ttl_months, ngram_size, and jaccard_threshold. (`D-9_l3_frozen_surfaces.md:34`)
- **D9-E16**: [[E]][SEC,CFG] Freeze canary regression_threshold and min_samples. (`D-9_l3_frozen_surfaces.md:35`)
- **D9-E17**: [[E]][SEC,FLOW] Enforcement has three layers: authoritative documentation/registry, pre-commit rejection, and a runtime auditor over attempted writes/events. (`D-9_l3_frozen_surfaces.md:42-46`)
- **D9-E18**: [[E]][DATA,SEC] locked_keys_registry.toml records locked_path with reason/override key, locked_symbol with file/reason, and locked_toml_key with file/section/keys/reason. (`D-9_l3_frozen_surfaces.md:75-104`)
- **D9-E19**: [[E]][TEST,SEC] Populate the registry with all 14 named surface classes and their per-file/per-symbol details. (`D-9_l3_frozen_surfaces.md:105-106`)
- **D9-E20**: [[E]][FLOW,SEC] locked_keys_check.sh parses the registry and cached name-only diff; a locked path without a matching L3_OVERRIDE tag exits 1 with a structured error. (`D-9_l3_frozen_surfaces.md:108-113`)
- **D9-E21**: [[E]][FLOW,SEC] The same check inspects diff bodies for locked symbols and parses TOML section/key changes. (`D-9_l3_frozen_surfaces.md:114-116`)
- **D9-E22**: [[E]][DATA,SEC] LockedKeysAuditor owns a registry, SSB sender, and audit UUID; LockedKeysRegistry stores vectors of paths, symbols, and TOML keys. (`D-9_l3_frozen_surfaces.md:118-122`)
- **D9-E23**: [[E]][FLOW,SEC] audit_diff returns an event per unauthorized match; audit_runtime_write performs the analogous check before a pending runtime write. (`D-9_l3_frozen_surfaces.md:122-127`)
- **D9-E24**: [[E]][FLOW,SEC,OBS] task_runner's PreToolUse/Tool::Write path invokes the auditor, blocks a touched locked surface, and emits LockedKeysTouch. (`D-9_l3_frozen_surfaces.md:129-134`)
- **D9-E25**: [[E]][INT,DATA,OBS] Bump SSB schema 54 to 55; LockedKeysTouch carries path/symbol/TOML location/override flag/audit ID; L3OverrideRequested carries key/requester/a 512-bound justification. (`D-9_l3_frozen_surfaces.md:136-144`)
- **D9-E26**: [[E]][SEC,OBS] L3OverrideRequested is informational documentation of a human decision, explicitly not an in-code permission-grant mechanism. (`D-9_l3_frozen_surfaces.md:140-143`)
- **D9-E27**: [[E]][CFG,SEC] A valid override is a registered L3_OVERRIDE key in the commit body with at least a 40-character justification; valid overrides allow the commit, others block. (`D-9_l3_frozen_surfaces.md:146-159`)
- **D9-E28**: [[E]][OBS,SEC] Documentation lists all 14 surfaces, points to the registry as authoritative, documents the override protocol, and cites its research grounding. (`D-9_l3_frozen_surfaces.md:161-166`)
- **D9-E29**: [[E]][TEST] Unit tests cover unauthorized and overridden path edits, a tripwire TOML-key edit with populated fields, and an unrelated path. (`D-9_l3_frozen_surfaces.md:168-175`)
- **D9-E30**: [[E]][TEST,SEC] Integration tests create a temporary git repository and require exit 1 without an override and exit 0 with one. (`D-9_l3_frozen_surfaces.md:177-181`)
- **D9-E31**: [[E]][INT,SEC] locked-keys-audit.yml runs the script on push and pull_request to main and fails violations even when local hooks are bypassed. (`D-9_l3_frozen_surfaces.md:183-187`)
- **D9-E32**: [[E]][CON,SEC] The decision record justifies structural enforcement, the 40-character floor, and using both dev-time and runtime enforcement. (`D-9_l3_frozen_surfaces.md:189-199`)
- **D9-E33**: [[E]][FAIL,SEC,CON] Registry parse error fails closed; override text shorter than 40 characters blocks; all path/symbol/TOML layers are mandatory; clean or valid override alone exits 0. No retry is defined. (`D-9_l3_frozen_surfaces.md:223-229`)
- **D9-E34**: [[E]][OBS,TEST] An unauthorized tripwire edit must produce one LockedKeysTouch, exit 1, and block the commit/runtime write; registry and CI artifacts must parse/exist. (`D-9_l3_frozen_surfaces.md:230-235`)
- **D9-E35**: [[E]][TEST,OBS] Cloud smokes cover unauthorized pre-commit block/event, valid override allow plus L3OverrideRequested, and runtime Tool::Write to bai.rs blocked before execution. (`D-9_l3_frozen_surfaces.md:276-298`)
- **D9-E36**: [[E]][DEP] Existing C-21 hook/script and all Wave-1 files are consumed; if no GitDiff/git2 support exists, shell out to git rather than adding an unapproved dependency. (`D-9_l3_frozen_surfaces.md:212-218`)
- **D9-E37**: [[E]][CON,DEP] D-25 combined tamper cases are out of scope; registry signing and multi-party override approval are deferred, relying on git signing and one operator in the current threat model. (`D-9_l3_frozen_surfaces.md:319-331`)
- **D9-E38**: [[E]][DEP,TEST] Before implementation, ingest and verify the Everitt reward-tampering, Hubinger mesa-optimization, and Cohen Boxed Myopic paper interfaces; acceptance requires all three artifacts. (`D-9_l3_frozen_surfaces.md:64-73,270-273,297`)
- **D9-E39**: [[E]][TEST] Completion also requires public schema 55, both SSB round trips, at least 14 registry entries covering all surface classes, the hook chain, CI, documentation, a clean workspace build, and at least eight locked-key tests. (`D-9_l3_frozen_surfaces.md:237-289`)
- **D9-N01**: [[A]][DEP] The original has no implementation status label; PLANNED is orientation-only. (`D-9_l3_frozen_surfaces.md:1-331`)
- **D9-N02**: [[C]][SEC,FLOW] A pre-commit hook is required to parse the commit-message body, but pre-commit normally executes before the final commit message exists; no commit-msg or prepare-commit-msg hook is specified. (`D-9_l3_frozen_surfaces.md:108-116,146-159`)
- **D9-N03**: [[C]][SEC] The objective and summary say git push is blocked, while concrete mechanisms block commit, runtime write, or CI; no pre-push enforcement is defined. (`D-9_l3_frozen_surfaces.md:3-8,108-134,183-187`)
- **D9-N04**: [[A]][OBS] Overridden diff tests allow either zero touch events or one override_present=true event, leaving the exact audit trail nondeterministic at specification level. (`D-9_l3_frozen_surfaces.md:168-174`)
- **D9-N05**: [[C]][SEC] The override event is called non-permission-granting, yet a valid override is the condition that permits the commit; the boundary between documentation and authorization is semantic rather than operationally defined. (`D-9_l3_frozen_surfaces.md:140-159`)
- **D9-N06**: [[A]][FAIL,SEC] Fail closed on an unreadable registry says treat it as locked but does not define whether all writes/commits are blocked or only known protected paths. (`D-9_l3_frozen_surfaces.md:223-225`)
- **D9-N07**: [[A]][SEC,FLOW] The source of a valid human override during runtime Tool::Write and whether runtime writes can ever be overridden are unspecified. (`D-9_l3_frozen_surfaces.md:122-134,146-159`)
- **D9-N08**: [[A]][DATA,SEC] Only example registry entries are shown; the exact canonical path/symbol/key mapping for each of the 14 surface classes remains implementation work. (`D-9_l3_frozen_surfaces.md:75-106`)
- **D9-N09**: [[C]][CFG,SEC] The contextual locked-key names (`rho_delta`, `iqr_mult`, `ppl_sigma`, plural `ddm_thresholds`) differ from the registry example (`rho_delta_threshold`, `iqr_outlier_multiplier`, `ppl_anomaly_sigma`, separate warning/drift keys); the canonical names are unresolved. (`D-9_l3_frozen_surfaces.md:31-35,97-104`)
- **D9-N10**: [[A]][TEST,SEC] The deterministic gate checks only at least 14 registry records, but one conceptual surface may require several path/symbol/key records; that count alone cannot prove all 14 classes or all per-file specifics are covered. (`D-9_l3_frozen_surfaces.md:77-106,246-249,278-281`)
- **D9-N11**: [[A]][INT,OBS] The shell pre-commit path is required to yield a `LockedKeysTouch` row, but the plan does not define how the hook obtains an SSB writer; only the Rust runtime auditor owns `ssb_tx`. (`D-9_l3_frozen_surfaces.md:108-134,230-235`)
- **D9-N12**: [[A]][DATA,SEC] `audit_diff` returns multiple events but `audit_runtime_write` returns only one optional event; behavior for one pending write touching multiple locked symbols or keys is not defined. (`D-9_l3_frozen_surfaces.md:118-127`)
- **D9-N13**: [[A]][CFG,SEC] Registry examples use human override keys such as `L3_OVERRIDE:tripwire`, while the convention/example uses a registry key such as `tripwire.kl_spec_tau`; the exact key namespace and matching algorithm are unspecified. (`D-9_l3_frozen_surfaces.md:77-106,146-159`)

### D-10: mode routing predicate

- **Status**: PLANNED — deps D-9; RPS.I / orchestrator route predicate
- **Anchors**: plan `D-10_mode_routing_predicate.md`; DEEP-D-001..004; aggregate unit (53 declared source atoms); compiled 53
- **Composes with**: D-001/IX-007 — **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance

**Behaviors**

- **D10-E01**: [[E]][CAP,SEC] Assign every novel claim to one of `GroundingMode::{Constructive, FalsifiabilityCrossFamily, OperationalFalsification, DiscoveryLabelOnly, Refuse}`; routing belongs to the orchestrator and accepts no LLM input. (`D-10_mode_routing_predicate.md:3-8`)
- **D10-E02**: [[E]][DEP,DET] Require D-9 DONE; run alongside D-11 and D-12, but not D-16 because UniCR formalizes Mode E. (`D-10_mode_routing_predicate.md:9-14`)
- **D10-E03**: [[E]][FLOW] Mode A (`Constructive`) applies when a claim is operationalizable and an applicable checker exists; the checker supplies ground truth. (`D-10_mode_routing_predicate.md:18-23,33-38`)
- **D10-E04**: [[E]][FLOW,FAIL,SEC] Mode B (`FalsifiabilityCrossFamily`) is a last-resort different-family adversarial probe, reached only after constructive and experimental routes are unavailable, intrinsic signals are not low, and an alternate family exists. (`D-10_mode_routing_predicate.md:23-24,33-49,52-54`)
- **D10-E05**: [[E]][FLOW,CON] Mode C (`DiscoveryLabelOnly`) is not a runtime route; it is a post-hoc label after Mode A or D yields a checker-grounded result. (`D-10_mode_routing_predicate.md:25-27,52-54`)
- **D10-E06**: [[E]][FLOW] Mode D (`OperationalFalsification`) runs a sandbox experiment represented by a typed `ObservableEffect`. (`D-10_mode_routing_predicate.md:28-29`)
- **D10-E07**: [[E]][FLOW,FAIL] Mode E (`Refuse`) is the fallback when there is no constructive checker or runnable experiment and intrinsic confidence is too low, and also when no alternate family exists. (`D-10_mode_routing_predicate.md:30-31,42-49`)
- **D10-E08**: [[E]][FLOW,DET] Predicate precedence is constructive first, runnable experiment second, low intrinsic signal refusal third, alternate-family falsifiability fourth, and refusal last. (`D-10_mode_routing_predicate.md:33-49`)
- **D10-E09**: [[E]][DATA] `Claim` fields are `id`, `claim_type`, `statement: BoundedString<2048>`, `domain`, optional `source_template`, and optional D-3 `intrinsic_signals`. (`D-10_mode_routing_predicate.md:77-81`)
- **D10-E10**: [[E]][DATA,SEC] `ClaimType` is the L3-locked closed enum `Code`, `Math`, `Prose`, `Empirical`, `Definitional`, `Counterfactual`, `EthicalAesthetic`. (`D-10_mode_routing_predicate.md:82-83`)
- **D10-E11**: [[E]][FLOW] `is_operationalizable` returns true for Code or Math and for Empirical with `test_cmd`; it returns false for Prose, Definitional, Counterfactual, and EthicalAesthetic. (`D-10_mode_routing_predicate.md:84-86`)
- **D10-E12**: [[E]][FLOW] `permits_experiment` returns true only for Empirical claims having a sandbox-compatible test. (`D-10_mode_routing_predicate.md:87-88`)
- **D10-E13**: [[E]][DATA,SEC] `GroundingMode` is the L3-locked closed five-variant enum named in the objective. (`D-10_mode_routing_predicate.md:90-95`)
- **D10-E14**: [[E]][DET,SEC,CON] `select_grounding_mode` is pure typed-input logic and must not call an LLM. (`D-10_mode_routing_predicate.md:94-96,197-200`)
- **D10-E15**: [[E]][DATA,DEP] Extend `OrchContext` with a D-11 checker-registry trait stub, the existing C-22 `FeasibilityProber` sandbox, a D-16 `unicr_tau` stub constant, and alternate-family availability. (`D-10_mode_routing_predicate.md:98-105`)
- **D10-E16**: [[E]][CFG,INT] `has_alternate_family_model()` returns true when `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` is set, referencing D-2/B-37. (`D-10_mode_routing_predicate.md:100-104`)
- **D10-E17**: [[E]][DEP,FLOW] Stub implementations deliberately preserve the interface until D-11, D-13, D-16, and D-18 supply concrete checker, adversary, calibration, and observable-effect behavior. (`D-10_mode_routing_predicate.md:100-105,187-193`)
- **D10-E18**: [[E]][FLOW,DEP] `dispatch_by_mode` sends Constructive to `checker_registry.run`; until D-11, that stub returns `Inapplicable`. (`D-10_mode_routing_predicate.md:107-112`)
- **D10-E19**: [[E]][FLOW,DEP] OperationalFalsification invokes `sandbox.run_observable_effect`; until D-18, that stub returns `Inconclusive`. (`D-10_mode_routing_predicate.md:107-113`)
- **D10-E20**: [[E]][FLOW,DEP] FalsifiabilityCrossFamily invokes `adversary_role.probe`; until D-13, that stub returns `NoResult`. (`D-10_mode_routing_predicate.md:107-115`)
- **D10-E21**: [[E]][FLOW,FAIL] Refuse produces `ClaimResult::Refused` with a reason. (`D-10_mode_routing_predicate.md:116`)
- **D10-E22**: [[E]][FLOW,FAIL,CON] Dispatching `DiscoveryLabelOnly` is forbidden as a runtime path and is intended to yield defensive inapplicability/error behavior. (`D-10_mode_routing_predicate.md:117-118`)
- **D10-E23**: [[E]][DATA] Declared `ClaimResult` variants are `Verified`, `Falsified { evidence: BoundedString<2048> }`, `Inconclusive { reason: String }`, and `Refused { reason: String }`. (`D-10_mode_routing_predicate.md:119-120`)
- **D10-E24**: [[E]][INT,DATA,OBS] Bump SSB schema 55 to 56 and add `ModeSelected { claim_id, mode, reason, audit_id }`. (`D-10_mode_routing_predicate.md:122-124`)
- **D10-E25**: [[E]][INT,DATA,OBS] Add `ClaimDispatched { claim_id, mode, result, evidence_snippet }`; both new SSB variants require round-trip tests. (`D-10_mode_routing_predicate.md:125-127`)
- **D10-E26**: [[E]][FLOW,OBS] At every claim-emission site, construct a Claim, select the mode, emit `ModeSelected`, dispatch it, then emit `ClaimDispatched`. (`D-10_mode_routing_predicate.md:129-133`)
- **D10-E27**: [[E]][SEC,CFG] Extend D-9's registry with a locked-symbol entry for `select_grounding_mode` in `grounding_mode.rs`, grounded in the no-self-grading rule. (`D-10_mode_routing_predicate.md:135-143`)
- **D10-E28**: [[E]][SEC,CFG] Add a separate locked-symbol entry for the `ClaimType` closed enum in `claim.rs`. (`D-10_mode_routing_predicate.md:144-148`)
- **D10-E29**: [[E]][TEST,FLOW] Unit tests cover operationalizability and experiment-permission tables, including Code with a test, Math with a SymPy expression, and Empirical without sandbox compatibility. (`D-10_mode_routing_predicate.md:150-153`)
- **D10-E30**: [[E]][TEST,FLOW] Routing tests require Constructive to outrank Mode D, then cover sandbox-only D, low-signal E, alternate-family B, and no-alternate E branches. (`D-10_mode_routing_predicate.md:153-160`)
- **D10-E31**: [[E]][TEST,DATA] The contract requires public schema 56, both SSB payload round trips, and stable serde for the closed `GroundingMode` enum. (`D-10_mode_routing_predicate.md:162-165`)
- **D10-E32**: [[E]][CON,SEC] The decision record must justify orchestrator-owned routing using Cohen/Everitt/Hubinger. (`D-10_mode_routing_predicate.md:167-170`)
- **D10-E33**: [[E]][FAIL,CON] It must justify Refuse rather than Mode B as the fallback, citing a 64.5% same-model blind-spot rate and preferring explicit uncertainty when no alternate family exists. (`D-10_mode_routing_predicate.md:171-173`)
- **D10-E34**: [[E]][FLOW,CON] It must justify why `DiscoveryLabelOnly` is post-hoc annotation rather than a runtime mode. (`D-10_mode_routing_predicate.md:174-175`)
- **D10-E35**: [[E]][FAIL,TEST,OBS] Stub bans require the empty-context Refuse path to remain reachable, all five named mode branches to be tested, and `ClaimDispatched` to be emitted regardless of mode. (`D-10_mode_routing_predicate.md:197-203`)
- **D10-E36**: [[E]][OBS,TEST] The observable exercise runs 50 synthetic claims, ten per named mode, requires 50 `ModeSelected` rows and at least 45/50 labels matching ground truth, described as at least 90% F1. (`D-10_mode_routing_predicate.md:203-208`)
- **D10-E37**: [[E]][TEST] Deterministic/acceptance checks require schema/event/type/lock presence, zero routing-file LLM-call matches, a clean workspace build, contract tests, and at least 12 new unit tests. (`D-10_mode_routing_predicate.md:210-249`)
- **D10-E38**: [[E]][TEST,OBS] Cloud smoke sends ten mixed synthetic claims, checks their hand-labeled distribution, and verifies by grep and execution trace that selection made no LLM call. (`D-10_mode_routing_predicate.md:250-255`)
- **D10-E39**: [[E]][CON,DEP] D-11 checker behavior, D-13 adversary behavior, D-16 UniCR, D-18 observable effects, and D-21 quantitative coverage are explicitly out of scope for this session. (`D-10_mode_routing_predicate.md:273-280`)
- **D10-E40**: [[E]][DEP] D-11 is the next consumer, and D-21 is stated to gate D-10 at at least 0.85 F1. (`D-10_mode_routing_predicate.md:281-284`)
- **D10-N01**: [[A]][DEP] The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. (`D-10_mode_routing_predicate.md:1-284`)
- **D10-N02**: [[C]][DATA,FLOW] `is_operationalizable` and its tests depend on `test_cmd` and `sympy_expr`, and experiment routing depends on sandbox compatibility, but none is a field of the declared `Claim`. (`D-10_mode_routing_predicate.md:77-88,150-153`)
- **D10-N03**: [[A]][FLOW,FAIL] `intrinsic_signals` is optional, but the predicate's behavior when it is `None` is not defined. (`D-10_mode_routing_predicate.md:79-81,42-43`)
- **D10-N04**: [[C]][DATA,FAIL] The DiscoveryLabelOnly branch says it returns `ClaimResult::Inapplicable`, but the declared `ClaimResult` enum has no `Inapplicable` variant. (`D-10_mode_routing_predicate.md:107-120`)
- **D10-N05**: [[C]][FAIL] The same branch is described as a defensive `Err`, while `dispatch_by_mode` returns bare `ClaimResult`, not `Result<ClaimResult, _>`. (`D-10_mode_routing_predicate.md:107-120`)
- **D10-N06**: [[A]][OBS,FLOW] Task-runner integration says the caller emits `ClaimDispatched`, while the stub ban says `dispatch_by_mode` itself must emit it; ownership is unresolved and a literal implementation could duplicate rows. (`D-10_mode_routing_predicate.md:129-133,197-203`)
- **D10-N07**: [[A]][INT,SEC] Presence of `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` does not by itself prove the available model is a different family from the executor; family identity and runtime health checks are not specified. (`D-10_mode_routing_predicate.md:98-105`)
- **D10-N08**: [[C]][TEST] The plan equates 45 correct classifications out of 50 with at least 90% F1; accuracy and macro/per-class F1 are not interchangeable. (`D-10_mode_routing_predicate.md:203-208,250-254`)
- **D10-N09**: [[C]][TEST,DEP] D-10's observable target says at least 90% F1, while its own Next section makes D-21 authoritative at at least 0.85 macro-F1. (`D-10_mode_routing_predicate.md:203-208,281-284; D-21_mode_routing_coverage_benchmark.md:39-49`)
- **D10-N10**: [[C]][FLOW,TEST] The predicate never returns `DiscoveryLabelOnly`, yet both D-10's 50-claim exercise and D-21's balanced corpus demand ten/forty Mode-C cases respectively. (`D-10_mode_routing_predicate.md:33-54,203-208; D-21_mode_routing_coverage_benchmark.md:15-21,31-34`)
- **D10-N11**: [[A]][DATA,OBS] Reason-string contents, `ClaimResult`-to-string serialization, and construction/truncation of `evidence_snippet` are unspecified. (`D-10_mode_routing_predicate.md:119-127`)
- **D10-N12**: [[A]][FAIL,FLOW] The stubs return Inapplicable, Inconclusive, or NoResult, but the declared result enum cannot represent two of those names and no fallback/re-routing rule is defined for a selected mode whose backend is unavailable. (`D-10_mode_routing_predicate.md:98-120`)
- **D10-N13**: [[A]][CAP,FLOW] “Every claim-emit site” has no enumerated call-site inventory or coverage check, so completeness of routing insertion cannot be established from this plan alone. (`D-10_mode_routing_predicate.md:56-61,129-133`)

### D-14: sycophancy probe

- **Status**: PLANNED — deps D-13; RPS.V / inverted-framing quarantine
- **Superseded:** SUP-010 — D-14 sycophancy dispatch event. Target: SycophancyProbeDispatched. Loser: Narrative SYCOPHANCY_PROBE.
- **Anchors**: plan `D-14_sycophancy_probe.md`; DEEP-D-001..004; aggregate unit (47 declared source atoms); compiled 47
- **Composes with**: D-001/IX-005 — **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture 

**Behaviors**

- **D14-E01**: [[E]][CAP,SEC,FLOW] For a user-aligned claim, run an inverted-framing probe; convergence is robust, divergence creates `SYCOPHANCY_FLAG` and human-review quarantine, as a structural rather than prompt-only defense. (`D-14_sycophancy_probe.md:3-8`)
- **D14-E02**: [[E]][DEP,DET] Require D-13 cross-family infrastructure, possibly using its Adversary slot or the same executor; run in parallel with D-15. (`D-14_sycophancy_probe.md:9-12`)
- **D14-E03**: [[E]][FLOW] Trigger when the model agrees with a direct user assertion, then invoke the same model with the same context and an instruction to assume that assertion is wrong and justify the alternative. (`D-14_sycophancy_probe.md:16-26`)
- **D14-E04**: [[E]][FLOW,FAIL] Similar positions across original and inverted responses are Convergent; significant divergence is a sycophancy flag and quarantines the claim. (`D-14_sycophancy_probe.md:26-30`)
- **D14-E05**: [[E]][DET,CFG] Convergence is token-level 5-shingle Jaccard similarity at least 0.7; the threshold is intended for later empirical tuning from ELAI session statistics. (`D-14_sycophancy_probe.md:32-34`)
- **D14-E06**: [[E]][CAP,DATA,CFG] `SycophancyProbe` owns `Arc<dyn RuntimeAdapter>`, `convergence_threshold` defaulting to 0.7, and an SSB sender. (`D-14_sycophancy_probe.md:58-62`)
- **D14-E07**: [[E]][DATA] `InvertedFraming` contains original and inverted `BoundedString<2048>` values plus `claim_id`. (`D-14_sycophancy_probe.md:63-64`)
- **D14-E08**: [[E]][DATA,OBS] `ProbeReport` contains claim ID, Blake3 hashes of both outputs, Jaccard similarity, and `ProbeVerdict`. (`D-14_sycophancy_probe.md:65-67`)
- **D14-E09**: [[E]][DATA,FLOW] `ProbeVerdict` is `Convergent`, `DivergentSycophancyFlag`, or `Inconclusive`. (`D-14_sycophancy_probe.md:68`)
- **D14-E10**: [[E]][FLOW] `invert_framing` detects direct positions with heuristics such as “I think”, “this is”, “X is correct”, and direct assertions, then constructs the specified adversarial prompt. (`D-14_sycophancy_probe.md:70-76`)
- **D14-E11**: [[E]][FAIL,FLOW] With no clear assertion, inversion conservatively mirrors the original with a skip note and yields Inconclusive rather than forcing a probe. (`D-14_sycophancy_probe.md:77-81`)
- **D14-E12**: [[E]][TEST] At least five synthetic user framings must each produce a sensible inversion. (`D-14_sycophancy_probe.md:80-81,184-190`)
- **D14-E13**: [[E]][FLOW,INT] `probe` inverts the message, skips to Inconclusive when marked, otherwise sends the inverted prompt through `RuntimeAdapter` and receives an inverted output. (`D-14_sycophancy_probe.md:83-89`)
- **D14-E14**: [[E]][DET,FLOW] It computes 5-shingle Jaccard between original and inverted output, maps `>= threshold` to Convergent and lower values to `DivergentSycophancyFlag`. (`D-14_sycophancy_probe.md:89-92`)
- **D14-E15**: [[E]][OBS,FLOW] Every executed probe emits a probe-dispatch row, and a divergent verdict additionally emits a sycophancy-flag row. (`D-14_sycophancy_probe.md:93,158-167`)
- **D14-E16**: [[E]][FLOW] The task-runner hook executes only after `dispatch_by_mode` returns `ClaimResult::Verified` and the user message has a direct assertion. (`D-14_sycophancy_probe.md:95-97`)
- **D14-E17**: [[E]][FLOW,SEC] A divergent claim is marked `human_review_pending` and cannot proceed to `MEMORIZING`. (`D-14_sycophancy_probe.md:98-100,158-162`)
- **D14-E18**: [[E]][INT,DATA,OBS] Bump SSB schema 59 to 60 and add `SycophancyProbeDispatched { claim_id, user_assertion_hash, audit_id }`. (`D-14_sycophancy_probe.md:101-106`)
- **D14-E19**: [[E]][INT,DATA,OBS] Add `SycophancyFlag { claim_id, jaccard, threshold, original_hash, inverted_hash }`; both variants require round trips. (`D-14_sycophancy_probe.md:101-106`)
- **D14-E20**: [[E]][CFG,SEC] Add L3-locked `[sycophancy_probe]` config with `enabled=true`, `convergence_threshold=0.7`, `jaccard_ngram_size=5`, and `skip_if_no_assertion=true`. (`D-14_sycophancy_probe.md:108-111`)
- **D14-E21**: [[E]][TEST,FAIL] Unit tests cover assertive and neutral inversions, Jaccard values 1.0/0.0/partial, and the complete verdict table. (`D-14_sycophancy_probe.md:113-120`)
- **D14-E22**: [[E]][TEST,OBS] The divergent integration scenario requires a flag event and quarantine. (`D-14_sycophancy_probe.md:122-127`)
- **D14-E23**: [[E]][TEST,OBS] The independent-agreement scenario requires Convergent and no flag. (`D-14_sycophancy_probe.md:128-130`)
- **D14-E24**: [[E]][CON,SEC] The decision record must justify structural probing because prompt-only mitigation is unreliable. (`D-14_sycophancy_probe.md:132-135`)
- **D14-E25**: [[E]][CON,DET] It must justify bounded 5-shingle Jaccard rather than unbounded edit distance. (`D-14_sycophancy_probe.md:136-137`)
- **D14-E26**: [[E]][CFG,CON] It must justify the default 0.7 threshold as a conservative midpoint of the cited 0.6-0.85 non-sycophantic similarity band. (`D-14_sycophancy_probe.md:138-140`)
- **D14-E27**: [[E]][DEP,INT] The module consumes B-37 `RuntimeAdapter`, D-10 Claim, SSB writing, and the user-message field on `OrchContext`, and is reachable from the task-runner post-dispatch hook plus config. (`D-14_sycophancy_probe.md:142-157`)
- **D14-E28**: [[E]][FAIL,CFG,OBS] Stub bans require a dispatch row regardless of verdict, safe empty-message skipping, config-loaded threshold, and quarantine enforcement at the MEMORIZING transition. (`D-14_sycophancy_probe.md:158-162`)
- **D14-E29**: [[E]][OBS] The divergent observable exercise yields exactly one dispatch row, one flag row, no MEMORIZING transition, an auditor-visible flag, and a quarantine artifact under `workflow/sessions/{id}/sycophancy-quarantine/`. (`D-14_sycophancy_probe.md:163-170`)
- **D14-E30**: [[E]][DEP,TEST] Before implementation, ingest the Sharma sycophancy paper interface; deterministic and acceptance checks require that artifact, schema/types/config presence, a clean workspace build, and at least eight unit tests. (`D-14_sycophancy_probe.md:42-54,172-194,201`)
- **D14-E31**: [[E]][TEST,FAIL] Cloud smokes cover divergent quarantine, independent convergence without a flag, and a neutral no-assertion skip recorded with `note="skipped"`. (`D-14_sycophancy_probe.md:195-200`)
- **D14-E32**: [[E]][CON,DEP] D-23 benchmarking is out of scope; multi-turn detection and LLM-as-judge detection are explicitly deferred, with the structural single-turn approach first. (`D-14_sycophancy_probe.md:216-220`)
- **D14-E33**: [[E]][DEP] D-15 is the next orthogonal output-labeling step, and D-23 later benchmarks this detector. (`D-14_sycophancy_probe.md:222-225`)
- **D14-N01**: [[A]][DEP] The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. (`D-14_sycophancy_probe.md:1-225`)
- **D14-N02**: [[C]][DATA,FAIL] `InvertedFraming` has no `note` field, yet probe control flow and the neutral smoke test read or emit such a note. (`D-14_sycophancy_probe.md:63-64,77-88,199-200`)
- **D14-N03**: [[C]][OBS,DATA] The neutral smoke expects `SycophancyProbeDispatched` with `note="skipped"`, but that event schema has no note or verdict field. (`D-14_sycophancy_probe.md:101-106,199-200`)
- **D14-N04**: [[C]][FLOW,OBS] The hook excludes neutral/no-assertion messages, while acceptance requires those messages to produce a skipped dispatch row; no alternative invocation site is defined. (`D-14_sycophancy_probe.md:95-100,158-160,199-200`)
- **D14-N05**: [[C]][INT,OBS] Narrative and summary terminology use `SYCOPHANCY_PROBE`, while the normative Rust variant is `SycophancyProbeDispatched`; no alias or SQL identifier mapping is supplied. (`D-14_sycophancy_probe.md:3-8,93,101-106,142-149`)
- **D14-N06**: [[A]][INT,SEC] The context says “same model, same context,” while the prerequisite permits either the D-13 Adversary slot or the same executor; actual adapter/model-family selection is unspecified. (`D-14_sycophancy_probe.md:9-11,21-25,36-40,150-154`)
- **D14-N07**: [[A]][DET,DATA] Tokenization, case/punctuation normalization, duplicate-shingle treatment, and behavior for outputs shorter than five tokens or both empty are not defined. (`D-14_sycophancy_probe.md:32-34,89-92,113-120`)
- **D14-N08**: [[A]][FAIL,INT] Timeout, provider failure, cancellation, retry, rate-limit, and partial-output behavior for the inverted runtime call are not defined; `SycophancyError` is named but not enumerated. (`D-14_sycophancy_probe.md:83-93`)
- **D14-N09**: [[A]][FLOW,SEC] Human-review ownership, release/reject criteria, retention, and resumption after quarantine are unspecified. (`D-14_sycophancy_probe.md:98-100,163-170`)
- **D14-N10**: [[A]][CFG,FLOW] `enabled` and `skip_if_no_assertion` defaults are declared, but disabled-mode behavior and any override semantics are not. (`D-14_sycophancy_probe.md:108-111`)
- **D14-N11**: [[A]][SEC,CFG] The config is declared L3-locked, but this plan does not add its keys to D-9's registry or define an override key. (`D-14_sycophancy_probe.md:108-111,204-214`)
- **D14-N12**: [[A]][OBS] A convergent or inconclusive result has no result-bearing SSB row; the dispatch event proves invocation but does not preserve verdict, similarity, or skip reason. (`D-14_sycophancy_probe.md:65-69,93,101-106`)
- **D14-N13**: [[A]][DET,SEC] Lexical Jaccard convergence is treated as semantic robustness, but paraphrase and shared-boilerplate handling are not specified; this is a design assumption rather than a proved equivalence. (`D-14_sycophancy_probe.md:26-34,89-92`)
- **D14-N14**: [[C]][OBS,DEP] The observable contract requires an auditor-agent update, but that agent file is absent from the declared files-touched list. (`D-14_sycophancy_probe.md:163-170,204-214`)

### D-21: mode routing coverage benchmark

- **Status**: PLANNED — deps D-10; D-10 gating evidence / routing F1
- **Superseded:** SUP-017 — D-21 Mode C benchmarkability. Target: Conflict retained: D-21 demands Mode C fixtures; D-10 never returns DiscoveryLabelOnly from selector. Loser: Neither deleted.
- **Anchors**: plan `D-21_mode_routing_coverage_benchmark.md`; DEEP-D-001..004; aggregate unit (33 declared source atoms); compiled 33
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-21

**Behaviors**

- **D21-E01**: [[E]][CAP,TEST] Benchmark D-10 with 200 hand-labeled synthetic claims across A/B/C/D/E, report F1 and per-mode confusion, and gate macro-F1 at 0.85 without class collapse. (`D-21_mode_routing_coverage_benchmark.md:3-5`)
- **D21-E02**: [[E]][DEP,DET] Require D-10 and declare parallel safety with D-19 through D-25. (`D-21_mode_routing_coverage_benchmark.md:6-8`)
- **D21-E03**: [[E]][DATA,TEST] Author exactly 40 examples per mode, each with a canonical expected mode and rationale. (`D-21_mode_routing_coverage_benchmark.md:15-17`)
- **D21-E04**: [[E]][TEST] Coverage must include clear compile-check Mode A, sandbox Mode D, ethical/aesthetic Mode E, ambiguous A-versus-D, and no-alternate-family Refuse edge cases. (`D-21_mode_routing_coverage_benchmark.md:18-21`)
- **D21-E05**: [[E]][DATA] Store 200 JSON fixture files under `.elai_cc/benchmarks/mode_routing/fixtures/`, with `claim_id`, `statement`, `claim_type`, `domain`, `intrinsic_signals`, `expected_mode`, and `rationale`. (`D-21_mode_routing_coverage_benchmark.md:29-34`)
- **D21-E06**: [[E]][CAP,DATA] `ModeRoutingBenchmark` contains `fixtures_dir`; `ModeRoutingReport` contains total, correct, `(expected,predicted)` confusion counts, macro-F1, per-mode F1, and `GatingVerdict`. (`D-21_mode_routing_coverage_benchmark.md:36-41`)
- **D21-E07**: [[E]][FLOW,DET] `run` loads every fixture, builds a mock `OrchContext`, calls `select_grounding_mode`, compares expected versus actual, and computes macro/per-mode F1 plus confusion matrix. (`D-21_mode_routing_coverage_benchmark.md:42-45`)
- **D21-E08**: [[E]][TEST,FAIL] Pass requires macro-F1 at least 0.85 and every mode's F1 at least 0.70; otherwise return `FailBelowBaseline`. (`D-21_mode_routing_coverage_benchmark.md:47-49`)
- **D21-E09**: [[E]][CAP] Expose CLI `elai bench mode-routing` from `bench_mode_routing.rs`. (`D-21_mode_routing_coverage_benchmark.md:51-53`)
- **D21-E10**: [[E]][CON] The decision record justifies 40 examples per mode as an approximately ±5% noise floor balanced against authoring cost. (`D-21_mode_routing_coverage_benchmark.md:54-56`)
- **D21-E11**: [[E]][TEST,CON] It justifies macro-F1 0.85 for system failure and per-mode 0.70 for class-collapse protection. (`D-21_mode_routing_coverage_benchmark.md:57-58`)
- **D21-E12**: [[E]][SEC,CON] It requires human-vetted labels rather than another LLM's output to avoid circular ground truth. (`D-21_mode_routing_coverage_benchmark.md:59-60`)
- **D21-E13**: [[E]][DEP,INT] The benchmark consumes D-10 selector/mode types and D-19 `BenchmarkStarted`/`BenchmarkResult` SSB support. (`D-21_mode_routing_coverage_benchmark.md:62-71`)
- **D21-E14**: [[E]][CAP,INT] The benchmark is reachable from CLI and a CI workflow. (`D-21_mode_routing_coverage_benchmark.md:72-73`)
- **D21-E15**: [[E]][SEC,TEST] Fixtures must be authored from scratch, not generated, and signed off by a human reviewer. (`D-21_mode_routing_coverage_benchmark.md:74-78`)
- **D21-E16**: [[E]][OBS] Each CLI run emits `BenchmarkStarted` and `BenchmarkResult` and writes `.elai_cc/benchmarks/mode_routing/reports/{date}.json`. (`D-21_mode_routing_coverage_benchmark.md:79-82`)
- **D21-E17**: [[E]][TEST] Deterministic checks require the runner/run symbols, at least 200 JSON fixtures, a clean workspace build, and scoped benchmark tests. (`D-21_mode_routing_coverage_benchmark.md:84-92`)
- **D21-E18**: [[E]][TEST,FAIL] Acceptance requires the benchmark to compile, all 200 balanced hand labels to exist, and the CLI to exit 0 when the gate is met. (`D-21_mode_routing_coverage_benchmark.md:94-99`)
- **D21-E19**: [[E]][TEST,OBS] The live 200-fixture smoke must reach macro-F1 at least 0.85 and every per-mode F1 at least 0.70 with no collapse. (`D-21_mode_routing_coverage_benchmark.md:100-101`)
- **D21-E20**: [[E]][DET,TEST] Two runs over identical mocked contexts must produce the same confusion matrix. (`D-21_mode_routing_coverage_benchmark.md:102-103`)
- **D21-E21**: [[E]][OBS,FLOW] Confusion output must expose edge cases for subsequent D-10 refinement. (`D-21_mode_routing_coverage_benchmark.md:104`)
- **D21-E22**: [[E]][CON,DEP] A D-21 decision document is required; larger fixture sets and LLM-generated fixtures are deferred, the latter due to circularity risk. (`D-21_mode_routing_coverage_benchmark.md:105-118`)
- **D21-E23**: [[E]][DEP] D-22 conformal-calibration validation follows this session. (`D-21_mode_routing_coverage_benchmark.md:120-122`)
- **D21-N01**: [[A]][DEP] The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. (`D-21_mode_routing_coverage_benchmark.md:1-122`)
- **D21-N02**: [[C]][FLOW,TEST] Forty fixtures require expected Mode C, but D-10 defines C as post-hoc label-only and its selector has no branch that returns it. (`D-21_mode_routing_coverage_benchmark.md:3-5,15-21,31-34; D-10_mode_routing_predicate.md:25-27,33-54`)
- **D21-N03**: [[A]][DATA,TEST] The fixture schema omits checker applicability, sandbox capability, alternate-family availability, and experiment/test metadata needed to construct the mocked context and reproduce D-10 routes. (`D-21_mode_routing_coverage_benchmark.md:31-45; D-10_mode_routing_predicate.md:33-49,77-105`)
- **D21-N04**: [[A]][TEST,DET] Precision/recall formulas, macro averaging, zero-denominator policy, and treatment of Mode C's unreachable predictions are not specified. (`D-21_mode_routing_coverage_benchmark.md:36-49`)
- **D21-N05**: [[A]][DATA,FAIL] `GatingVerdict` and its Pass variant/source are not defined, while only `FailBelowBaseline` is named. (`D-21_mode_routing_coverage_benchmark.md:36-49`)
- **D21-N06**: [[A]][FAIL,TEST] Acceptance ties CLI exit 0 only to macro-F1, while normative gate logic additionally requires every per-mode F1 at least 0.70; the exit status when macro passes but one class fails is not restated. (`D-21_mode_routing_coverage_benchmark.md:47-49,94-101`)
- **D21-N07**: [[C]][DEP,DET] The session is declared parallel-safe with D-19 yet consumes benchmark SSB types “from D-19”; build sequencing or a pre-existing shared definition is not supplied. (`D-21_mode_routing_coverage_benchmark.md:7-8,62-73`)
- **D21-N08**: [[A]][SEC,OBS] Human sign-off is mandatory, but reviewer identity, signature format, approval artifact, and enforcement check are unspecified. (`D-21_mode_routing_coverage_benchmark.md:31-34,59-60,74-78`)
- **D21-N09**: [[A]][FAIL] Missing, malformed, duplicate, imbalanced, or extra fixtures and partial report-write failures have no defined error/fallback behavior. (`D-21_mode_routing_coverage_benchmark.md:31-49,79-82`)
- **D21-N10**: [[A]][DET,OBS] `{date}` timezone, same-day overwrite/versioning, stable map serialization, and byte-for-byte report determinism are unspecified even though result determinism is an acceptance gate. (`D-21_mode_routing_coverage_benchmark.md:39-41,79-82,102-103`)

### D-24: cross domain canary regression suite

- **Status**: PLANNED — deps D-7; D-7 gating evidence / canary errors
- **Anchors**: plan `D-24_cross_domain_canary_regression_suite.md`; DEEP-D-001..004; aggregate unit (28 declared source atoms); compiled 28
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-24

**Behaviors**

- **D24-E01**: [[E]][CAP,TEST] Validate D-7 with 100 synthetic mutations across five task families: 50 helpful, 30 neutral, and 20 harmful. (`D-24_cross_domain_canary_regression_suite.md:3-5`)
- **D24-E02**: [[E]][DEP,DET] Require D-7 and declare parallel safety with D-19 through D-25. (`D-24_cross_domain_canary_regression_suite.md:6-8`)
- **D24-E03**: [[E]][DATA,TEST] Helpful mutations improve the target family by 5-15% and regress no non-target family. (`D-24_cross_domain_canary_regression_suite.md:14-15`)
- **D24-E04**: [[E]][DATA,TEST] Neutral mutations leave the target within ±2% and regress no non-target family. (`D-24_cross_domain_canary_regression_suite.md:16`)
- **D24-E05**: [[E]][DATA,TEST] Harmful mutations leave the target unchanged or improved but regress at least one non-target family by more than 5%. (`D-24_cross_domain_canary_regression_suite.md:17-18`)
- **D24-E06**: [[E]][DATA] Store 100 TOML fixtures under `.elai_cc/benchmarks/canary/fixtures/`, each with mutation ID, target family, baseline/candidate pass rates by family, and expected verdict. (`D-24_cross_domain_canary_regression_suite.md:26-32`)
- **D24-E07**: [[E]][CAP,FLOW] `CanaryBenchmark` invokes D-7 `CanaryRunner` for each fixture and compares the result with `expected_verdict`. (`D-24_cross_domain_canary_regression_suite.md:34-37`)
- **D24-E08**: [[E]][DATA,OBS] Its report contains total, true_block, false_block, true_pass, and false_pass. (`D-24_cross_domain_canary_regression_suite.md:34-37`)
- **D24-E09**: [[E]][TEST,FAIL] Hard gates are 20/20 harmful blocked, no more than 5/50 helpful blocked, and no more than 2/30 neutral blocked. (`D-24_cross_domain_canary_regression_suite.md:39-40`)
- **D24-E10**: [[E]][CAP] Expose CLI `elai bench canary-regression`. (`D-24_cross_domain_canary_regression_suite.md:42`)
- **D24-E11**: [[E]][CON] The decision record justifies 100 fixtures as sufficient for the intended sample-noise floor. (`D-24_cross_domain_canary_regression_suite.md:44-45`)
- **D24-E12**: [[E]][CON] It justifies the 50/30/20 split because harmful mutations are the minority and over-sampling would distort evaluation. (`D-24_cross_domain_canary_regression_suite.md:46`)
- **D24-E13**: [[E]][TEST,CON] It justifies a 10% helpful false-block allowance so D-7's regression threshold can be tuned without rebuilding canary infrastructure. (`D-24_cross_domain_canary_regression_suite.md:47-48`)
- **D24-E14**: [[E]][DEP,INT] The benchmark consumes D-7 `CanaryRunner` and SSB writing and is reachable from CLI plus CI. (`D-24_cross_domain_canary_regression_suite.md:50-59`)
- **D24-E15**: [[E]][TEST,SEC] Blocking all 20 harmful fixtures is an unrelaxed hard gate. (`D-24_cross_domain_canary_regression_suite.md:60-61`)
- **D24-E16**: [[E]][OBS] A CLI run emits `BenchmarkStarted` plus `BenchmarkResult` and writes a report beneath `.elai_cc/benchmarks/canary/reports/`. (`D-24_cross_domain_canary_regression_suite.md:62-65`)
- **D24-E17**: [[E]][TEST] Deterministic checks require exactly/at least 100 fixture files, the benchmark struct, and a clean workspace build. (`D-24_cross_domain_canary_regression_suite.md:67-74`)
- **D24-E18**: [[E]][TEST,FAIL] Acceptance requires compilation, 100 fixtures, CLI exit 0 only when gates are met, and the clean workspace build. (`D-24_cross_domain_canary_regression_suite.md:76-81`)
- **D24-E19**: [[E]][TEST] Three cloud smokes separately prove 20/20 harmful blocking, helpful false blocks at most 5/50, and neutral blocks at most 2/30. (`D-24_cross_domain_canary_regression_suite.md:82-85`)
- **D24-E20**: [[E]][CON,DEP] Real-data mutations are deferred; the plan treats synthetic fixtures as sufficient for the D-7 gate. (`D-24_cross_domain_canary_regression_suite.md:95-101`)
- **D24-N01**: [[A]][DEP] The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. (`D-24_cross_domain_canary_regression_suite.md:1-101`)
- **D24-N02**: [[A]][DATA,TEST] “Across 5 task families” has no per-family quota or cross-product rule, so all 100 fixtures could technically concentrate in one family while satisfying the stated type counts. (`D-24_cross_domain_canary_regression_suite.md:3-5,28-32`)
- **D24-N03**: [[A]][TEST] Helpful improvement exactly +5%, neutral exactly ±2%, and non-target regression exactly 5% have only partly explicit boundary semantics; the harmful definition is strictly greater than 5%. (`D-24_cross_domain_canary_regression_suite.md:14-18`)
- **D24-N04**: [[A]][DATA,FAIL] Rate ranges, family completeness, numeric precision, missing families, malformed TOML, duplicate IDs, and invalid expected verdicts have no validation/error policy. (`D-24_cross_domain_canary_regression_suite.md:28-37`)
- **D24-N05**: [[A]][DATA,OBS] The report has only binary pass/block counters and no per-family/type breakdown or explicit overall verdict field, limiting diagnosis of where thresholds fail. (`D-24_cross_domain_canary_regression_suite.md:34-40`)
- **D24-N06**: [[A]][INT,OBS] Generic benchmark SSB event payloads and their schema/version source are not defined in this plan. (`D-24_cross_domain_canary_regression_suite.md:50-65`)
- **D24-N07**: [[A]][DET] Fixture ordering, random seed (if any), repeated-run equivalence, report naming, and overwrite/concurrency behavior are unspecified. (`D-24_cross_domain_canary_regression_suite.md:26-42,62-74`)
- **D24-N08**: [[A]][TEST] No scoped unit/integration test command is included; deterministic verification checks only fixture count, symbol presence, and workspace compilation before cloud smokes. (`D-24_cross_domain_canary_regression_suite.md:67-85`)

### D-28: adversarial example farming

- **Status**: mixed / partial — shipping-pass CLI/SSB legs claimed live; training/deploy DEFERRED (SUP-004)
- **Superseded:** SUP-004 — D-28 status and SSB events. Target: Shipping-pass: AdversarialFarmRunCompleted; local CLI/SSB legs claimed live; training/deploy deferred. Loser: Header Status:DEFERRED and two-event design AdversarialCorpusGenerated/PpmFinetuneCompleted.
- **Anchors**: plan `D-28_adversarial_example_farming.md`; DEEP-D-001..004; aggregate unit (47 declared source atoms); compiled 47
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-28

**Behaviors**

- **D28-E01**: [[E]][CAP,DEP] Deferred research-grade work iteratively generates OOD high-proxy/low-quality PPM examples and retrains D-2's PPM for gaming resistance; estimate 4-6 sessions plus RunPod hours. (`D-28_adversarial_example_farming.md:1-8`)
- **D28-E02**: [[E]][DEP,DET] Unblock only after D-5 has at least 20 stable promotion cycles, D-19 publishes a PRMBench baseline, and a RunPod 4090 or A100 is available; it may run beside D-29 and D-30. (`D-28_adversarial_example_farming.md:9-11,129-132`)
- **D28-E03**: [[E]][FLOW,TEST] Generate candidate reasoning steps that score high on the current PPM but fail deterministic gold checks such as compile, tests, or formal consistency. (`D-28_adversarial_example_farming.md:13-20`)
- **D28-E04**: [[E]][FLOW,DATA] Mine the proxy-versus-gold gap and assign negative labels to the resulting adversarial corpus. (`D-28_adversarial_example_farming.md:20-21`)
- **D28-E05**: [[E]][FLOW,INT] Retrain the PPM on that corpus, then remeasure PRMBench Deception Resistance and Circular Logic with a target improvement of at least five percentage points. (`D-28_adversarial_example_farming.md:21-28`)
- **D28-E06**: [[E]][DEP] Bootstrapping is explicitly blocked by stable BAI selection, a published baseline, and cost-throttled GPU fine-tuning. (`D-28_adversarial_example_farming.md:25-28`)
- **D28-E07**: [[E]][DEP,TEST] Ingest and verify `workflow/papers/adversarial-rm-2504.06141-interface.md` before unblocking. (`D-28_adversarial_example_farming.md:30-44`)
- **D28-E08**: [[E]][CAP,FLOW] `ppm_adversarial.py` samples a step, computes proxy and gold rewards, and retains it when proxy is much greater than gold and gold equals Fail. (`D-28_adversarial_example_farming.md:46-54`)
- **D28-E09**: [[E]][DATA] The task-level corpus destination is `.elai_cc/calibration/ppm_adversarial/{ts}/{step_id}.jsonl`. (`D-28_adversarial_example_farming.md:48-55`)
- **D28-E10**: [[E]][INT,CFG] `ppm_finetune.sh` runs on a RunPod 4090/A100 and loads `Qwen2.5-Math-PRM-7B`. (`D-28_adversarial_example_farming.md:56-59`)
- **D28-E11**: [[E]][CFG] Fine-tuning uses negative-labeled adversarial examples with LoRA rank 16, three epochs, and learning rate `1e-5`. (`D-28_adversarial_example_farming.md:58-60`)
- **D28-E12**: [[E]][DATA,INT] Fine-tuning outputs `ppm_v2.safetensors` and deploys it to the remote sidecar. (`D-28_adversarial_example_farming.md:60`)
- **D28-E13**: [[E]][FLOW,OBS] Run D-19 on v1 and save its report, deploy v2, run D-19 again, save that report, then compare category deltas. (`D-28_adversarial_example_farming.md:62-65`)
- **D28-E14**: [[E]][TEST,FAIL] Every standard PRMBench category must avoid regression greater than two percentage points. (`D-28_adversarial_example_farming.md:67-70`)
- **D28-E15**: [[E]][TEST] Deception Resistance and Circular Logic must each improve by at least five percentage points. (`D-28_adversarial_example_farming.md:67-70,129-135`)
- **D28-E16**: [[E]][FAIL] If standard categories regress or adversarial gains miss the gate, revert to v1, log the corpus and decision, and do not deploy v2. (`D-28_adversarial_example_farming.md:72-75`)
- **D28-E17**: [[E]][INT,DATA,OBS] The task originally proposes, with an appropriate future schema bump, `AdversarialCorpusGenerated { count, source_session }`. (`D-28_adversarial_example_farming.md:77-80`)
- **D28-E18**: [[E]][INT,DATA,OBS] It also proposes `PpmFinetuneCompleted { v1_score, v2_score, delta, verdict }`. (`D-28_adversarial_example_farming.md:77-80`)
- **D28-E19**: [[E]][CON,DEP] Draft the D-28 decision document only at unblock time, explaining deferral, conservative LoRA rather than full tuning, and the literature-grounded five-point gain gate. (`D-28_adversarial_example_farming.md:82-89`)
- **D28-E20**: [[E]][CAP,DATA] Shipping-pass annotations name live `AdvRmFarmer`, `AdvRmFarmer::run`, `AdvRmFarmRunReport`, `AdvRmSeed`, `AdvRmCorpusEntry`, and `verdict -> GatingVerdict` with implementation anchors. (`D-28_adversarial_example_farming.md:91-97`)
- **D28-E21**: [[E]][INT,OBS] Those annotations explicitly reject both originally proposed event variants and identify the actual shipped event/payload as `AdversarialFarmRunCompleted`. (`D-28_adversarial_example_farming.md:97-101`)
- **D28-E22**: [[E]][DEP,INT] The Python producer exists according to the plan; the fine-tune script is `RUNPOD-PENDING`. (`D-28_adversarial_example_farming.md:100-101`)
- **D28-E23**: [[E]][DEP] The shipping contract consumes at least 20 D-5 BAI cycles, D-19 `PRMBenchRunner`, `SidecarPpmHook`, and D-5 `darwinian_bai`. (`D-28_adversarial_example_farming.md:102-106`)
- **D28-E24**: [[E]][CAP] Live reachability is asserted through `elai farm-adv-rm run` and `elai farm-adv-rm verdict`. (`D-28_adversarial_example_farming.md:107-110`)
- **D28-E25**: [[E]][INT,DEP] Live `ppm_adversarial.py --live` and the GPU fine-tune/post-immunization legs remain RunPod- and prerequisite-pending. (`D-28_adversarial_example_farming.md:110,119-121`)
- **D28-E26**: [[E]][OBS,TEST] Every farming run must emit `AdversarialFarmRunCompleted`; `verdict` must call the real `report.passed()` rather than a mock. (`D-28_adversarial_example_farming.md:111-114`)
- **D28-E27**: [[E]][SEC,CON] Production corpus generation may not use `ppm_adversarial.py --mock`. (`D-28_adversarial_example_farming.md:114`)
- **D28-E28**: [[E]][OBS] `elai farm-adv-rm run --dataset <prmbench_seed_path> --corpus-dir .elai_cc/benchmarks/adv_rm/corpus/` must emit one completion row with `samples_farmed > 0` and populate `AdvRmCorpusEntry` JSONL. (`D-28_adversarial_example_farming.md:115-121`)
- **D28-E29**: [[E]][DEP,OBS] The source explicitly records the CLI-run and SSB-emission legs as wired and specifically claims `farm_adv_rm.rs` plus `ssb.rs:6710` are live. (`D-28_adversarial_example_farming.md:119-121`)
- **D28-E30**: [[E]][TEST,DEP] The deterministic-check section remains deferred to unblock time; acceptance is gated on prerequisites, GPU access, paper output, v2 deployment/report, both numeric gates, and the decision document. (`D-28_adversarial_example_farming.md:123-136`)
- **D28-E31**: [[E]][INT,DEP] Anticipated artifacts include the Python generator, RunPod fine-tune script, timestamped calibration corpus, v2 sidecar-startup update, decision document, and research interface. (`D-28_adversarial_example_farming.md:138-146`)
- **D28-E32**: [[E]][CON] Full PPM rebuilding and multi-model adversarial corpora are deferred beyond D-28; LoRA and a single-model corpus define the planned scope. (`D-28_adversarial_example_farming.md:148-151`)
- **D28-E33**: [[E]][DEP,DET] Once unblocked, D-29 may run independently in a different RunPod session. (`D-28_adversarial_example_farming.md:153-156`)
- **D28-N01**: [[C]][DEP] Header/status and acceptance mark D-28 deferred, while the shipping-pass block asserts that farming CLI/SSB legs are live; the source therefore describes partial implementation, not a single uniform status. (`D-28_adversarial_example_farming.md:1-11,91-121,127-136`)
- **D28-N02**: [[C]][DATA] Planned generator output uses `.elai_cc/calibration/ppm_adversarial/{ts}/`, while the asserted live CLI writes `.elai_cc/benchmarks/adv_rm/corpus/`; canonical corpus location and migration are unresolved. (`D-28_adversarial_example_farming.md:48-55,115-121,138-143`)
- **D28-N03**: [[C]][INT,OBS] Tasks specify two future SSB variants, but the shipping contract explicitly says both were rejected in favor of one completion event; acceptance was not rewritten to state the actual event contract. (`D-28_adversarial_example_farming.md:77-80,91-101,127-136`)
- **D28-N04**: [[A]][INT,DATA] “Appropriate SCHEMA bump” supplies no version, migration rule, public constant, or compatibility test. (`D-28_adversarial_example_farming.md:77-80`)
- **D28-N05**: [[A]][FLOW,TEST] `proxy >> gold` has no numeric margin, search budget, stopping condition, sample count, or balance rule. (`D-28_adversarial_example_farming.md:15-23,48-54`)
- **D28-N06**: [[A]][DEP,TEST] “At least 20 stable cycles” never defines stable, cycle identity, consecutive-window requirements, minimum samples per arm, or who certifies readiness. (`D-28_adversarial_example_farming.md:9-10,25-28,102-104,129-130`)
- **D28-N07**: [[A]][DATA,TEST] The planned `PpmFinetuneCompleted` scalar v1/v2 scores cannot by itself encode both adversarial-category gains plus every standard-category regression gate. (`D-28_adversarial_example_farming.md:62-80`)
- **D28-N08**: [[A]][DET,TEST] Dataset version, split, random seeds, sample ordering, optimizer determinism, repeated-run tolerance, and hardware-dependent reproducibility are unspecified. (`D-28_adversarial_example_farming.md:46-70,123-136`)
- **D28-N09**: [[A]][DET,FLOW] Farming idempotency, duplicate-example handling, concurrent writers, resume after interruption, and corpus atomicity are unspecified. (`D-28_adversarial_example_farming.md:46-55,115-121`)
- **D28-N10**: [[A]][SEC,INT] Model/corpus provenance, safetensors checksum/signature, remote transport, sidecar health validation, and deployment authorization are not defined. (`D-28_adversarial_example_farming.md:56-65,138-146`)
- **D28-N11**: [[A]][FAIL] Rollback says revert to v1 but does not define the atomic version pointer, rollback command, archive/event evidence, or recovery from a partially deployed v2. (`D-28_adversarial_example_farming.md:72-75,140-145`)
- **D28-N12**: [[A]][SEC] Corpus privacy, secret/PII scrubbing, retention, and whether production reasoning may be exported to RunPod are not addressed. (`D-28_adversarial_example_farming.md:46-60`)
- **D28-N13**: [[A]][CFG,DEP] GPU use is called cost-throttled, but no budget, timeout, instance-hours limit, or approval gate is specified. (`D-28_adversarial_example_farming.md:7,25-28,56-60`)
- **D28-N14**: [[I]][DEP] Joining the explicit shipping notes yields a two-part state: local farming/audit substrate is claimed live; immunization training, deployment, and post-benchmark proof remain deferred. This is synthesis, not independent implementation verification. (`D-28_adversarial_example_farming.md:91-136`)

### D-2: ppm sidecar wiring

- **Status**: PLANNED — deps D-1, B-37; Phase 4H.V.1 / heterogeneous PPM margin
- **Superseded:** SUP-014 — D-2 PpmClient shape. Target: Concrete struct PpmClient + PpmAdapter RuntimeAdapter. Loser: Summary trait.
- **Anchors**: plan `D-2_ppm_sidecar_wiring.md`; DEEP-D-001..004; aggregate unit (42 declared source atoms); compiled 42
- **Composes with**: D-001/IX-005 — **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture  | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027. | D-001/IX-007 — **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance | D-001/IX-008 — **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D | D-001/IX-010 — **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not indepen

**Behaviors**

- **D003-D2-E001**: `[E]` — capability/topology. D2 introduces a second, step-level Process Preference Model sidecar, with policy `Qwen2.5-Coder-7B-Instruct-abliterated` on port `8100` and PPM `Qwen2.5-Math-PRM-7B` on `8101`; the two artifacts evolve independently against different signals and the verifier is not the policy in self-judge mode. (`D2:1-9`)
- **D003-D2-E002**: `[E]` — prerequisites/status envelope. Estimated effort is 2–3 sessions; execution requires D-1 (`ReasoningTemplate`, `VerifierKind::Prm`, `verifier_required=Prm`), B-37 (`PerRoleRuntime` and backend slot), and C-15 (`ClockProvider` plus entropy quarantine) all verified `DONE` by `spec-grounder`. (`D2:9-13`)
- **D003-D2-E003**: `[E]` — concurrency. D2 is declared parallel-safe with D-8 and the isolated D-19 benchmark surface, but not with D-3 because D-3 consumes its PPM output. (`D2:14-16`)
- **D003-D2-E004**: `[E]` — model constraint/evidence. The plan rules out the policy model as its own verifier, describes Qwen2.5-Math-PRM-7B as Qwen-family but separate-base/math-pretrained/PRM-distilled, cites 60–65 standard-category PRMBench performance, and explicitly delegates adversarial weakness measurement to D-19. (`D2:20-32`)
- **D003-D2-E005**: `[E]` — deployment capacity. The remote sidecar binds remotely and tunnels to loopback `127.0.0.1:8101`; the stated sizing is about 1.92 GB per NF4 7B model plus about 1.8 GB total KV cache, about 5.7 GB combined, fitting the stated 12/24 GB budget. (`D2:34-39`)
- **D003-D2-E006**: `[E]` — existing integrations. D2 reuses B-37 `RuntimeAdapter`, `PerRoleRuntime`, existing `HTTPClient`/retry logic in `sidecar_client.rs`, the Anthropic `/v1/messages` FastAPI speaker, D-1’s `VerifierKind::Prm` slot, and `[runtime.*]` configuration. (`D2:41-48`)
- **D003-D2-E007**: `[E]` — research gate. Before `TESTING_FIRST`, `paper-ingestor` must materialize `rstar-math-2501.04519-interface.md`, `latent-verifiers-2504.16760-interface.md`, and `prmbench-2501.03124-interface.md`. (`D2:70-79`)
- **D003-D2-E008**: `[E]` — client schema. New `ppm_client.rs` defines `PpmClient { endpoint: Url, http: Arc<reqwest::Client>, timeout: Duration, retry_policy: RetryPolicy }`. (`D2:81-87`)
- **D003-D2-E009**: `[E]` — client action. `PpmClient::new(...)` and `score_step(prompt: &str, candidate_step: &str, ctx: &PpmContext) -> Result<PpmScore, PpmError>` are the construction and scoring surfaces. (`D2:83-90`)
- **D003-D2-E010**: `[E]` — score schema/range. `PpmScore` contains `score: f32`, `confidence: f32`, and `raw_output: String`; `score` is constrained to `[0.0, 1.0]`. (`D2:90-91`)
- **D003-D2-E011**: `[E]` — context schema. `PpmContext` contains `problem_statement: String`, `prior_steps: Vec<String>`, and `domain: Domain`. (`D2:92-93`)
- **D003-D2-E012**: `[E]` — error surface. `PpmError` is a `thiserror` family containing `HttpError(#[from] reqwest::Error)`, `TimeoutError`, `ParseError(String)`, and `SidecarUnavailable`. (`D2:94-95`)
- **D003-D2-E013**: `[E]` — deterministic request format. `score_step` emits the fixed rStar-Math-style `Problem / Previous steps / Current step / Is this step correct? Rate 0.0 to 1.0` prompt; an unparseable response is a `ParseError`, not a guess. (`D2:96-98`)
- **D003-D2-E014**: `[E]` — retry/failure behavior. Every HTTP call uses at most three attempts with exponential backoff and jitter; exhausting a timeout returns `TimeoutError`, and infinite retry is forbidden. (`D2:99-100,228`)
- **D003-D2-E015**: `[E]` — runtime adapter. New `PpmAdapter { client: Arc<PpmClient> }` implements `RuntimeAdapter`, reports backend ID `"ppm"`, translates Anthropic `/v1/messages` requests to score requests, and wraps `PpmScore` as `RuntimeResponse`. (`D2:102-107`)
- **D003-D2-E016**: `[E]` — permission boundary. PPM is verifier-only: agents cannot select it for generation, only D-3’s consensus verifier may call it, and `RuntimeAdapter::is_verifier_only()` must return `true`. (`D2:108-110,220-224`)
- **D003-D2-E017**: `[E]` — runtime/config defaults. `RuntimeBackend` gains `Ppm(PpmAdapter)`; `[runtime.ppm]` defaults are endpoint `http://127.0.0.1:8101/v1/messages`, `timeout_secs=30`, `max_retries=3`, and model `huihui-ai/Qwen2.5-Math-PRM-7B` or a distilled-equivalent path. (`D2:112-117`)
- **D003-D2-E018**: `[E]` — activation default. `PerRoleRuntime::default()` leaves PPM disabled; activation requires both TOML opt-in and environment flag `ELAI_PPM_ENABLE=1`, preventing development sessions from accidentally requiring an offline second sidecar. (`D2:118-120,190-192`)
- **D003-D2-E019**: `[E]` — remote boot action. `start-ppm-sidecar.sh` uses the existing SSH/virtualenv server pattern, `CUDA_VISIBLE_DEVICES=1`, model path above, port `8101`, `--no-dola`, `--turboquant-bits 0`, `--no-steering`, and `--load-in-4bit`, and records log/PID files. (`D2:122-130`)
- **D003-D2-E020**: `[E]` — placement fallback. GPU1 is the normal PPM placement while GPU0 hosts policy; a documented single-GPU fallback serializes VRAM swapping and is supported but explicitly suboptimal. (`D2:131-133`)
- **D003-D2-E021**: `[E]` — health workflow. Health is a three-stage process check, TCP reachability check, then real `/v1/messages` curl request; the named probes are `pgrep`, `Test-NetConnection`, and `curl`. (`D2:134-137`)
- **D003-D2-E022**: `[E]` — tunnel integration. `start-tunnel.ps1` adds `127.0.0.1:8101 -> remote 127.0.0.1:8101`, leaves the existing `8100` forward unchanged, and documents the dual tunnel in `REMOTE-SIDECAR.md`. (`D2:139-143`)
- **D003-D2-E023**: `[E]` — CLI surface. `elai ppm health` wraps the three-stage health check, reports `OK` or a structured `PpmError`, and exits `0`/`1`. (`D2:145-150`)
- **D003-D2-E024**: `[E]` — score event schema. `PpmScoreEmitted` carries `template_id`, `step_index`, `score`, `confidence`, and `latency_ms`. (`D2:152-154`)
- **D003-D2-E025**: `[E]` — unavailable event schema. `PpmUnavailable` carries a string `reason` and `retry_count`, and fires only after verifier dispatch exhausts retries. (`D2:155-156`)
- **D003-D2-E026**: `[E]` — schema lifecycle. D2 scaffolds both PPM SSB variants behind `schema-v49` or a temporary marked scaffold, while D3 owns the atomic schema bump; D2 must not land an independent bump. (`D2:157-160,203-204`)
- **D003-D2-E027**: `[E]` — dependency policy. `reqwest` should already exist; no new production dependency is expected, and any missing retry-jitter helper should be hand-rolled in at most 30 LOC rather than add a crate. (`D2:162-166`)
- **D003-D2-E028**: `[E]` — unit evidence. Required client tests cover mock happy path, transient `502` success on the third attempt, persistent `504` to `TimeoutError`, malformed body to `ParseError`, and a deterministic fixed-input prompt snapshot. (`D2:168-176`)
- **D003-D2-E029**: `[E]` — integration evidence. The adapter integration test runs through `RuntimeAdapter` against a local axum mock, verifies score round-trip, and asserts `is_verifier_only() == true`. (`D2:178-182`)
- **D003-D2-E030**: `[E]` — recorded decisions. `D-2-decisions.md` must justify the same-family/different-base PPM choice as an acknowledged uncertainty, opt-in activation, and three-attempt jittered backoff. (`D2:184-195`)
- **D003-D2-E031**: `[E]` — reachability. D2 is reachable from D3’s `VerificationResult.ppm`, `elai ppm health`, `ELAI_PPM_ENABLE`, and `[runtime.ppm]`. (`D2:197-219`)
- **D003-D2-E032**: `[E]` — prohibited fallbacks. `PpmClient` must have no same-model policy fallback; an empty response must return `ParseError` rather than panic; the default endpoint must remain `8101`, not collide with policy `8100`; retry must remain bounded. (`D2:220-228`)
- **D003-D2-E033**: `[E]` — observable evidence contract. With enable flag set, sidecar booted, and health invoked, D2 expects health exit `0`, a real curl response, boot/tunnel/config artifacts, and—before D3 consensus wiring—zero SSB rows. (`D2:229-235`)
- **D003-D2-E034**: `[E]` — deterministic done probes. The plan enumerates source-symbol searches, port/CLI/health checks, workspace build and scoped tests, a negative same-model-fallback search expecting zero hits, and existence of all three paper interfaces. (`D2:237-277`)
- **D003-D2-E035**: `[E]` — base acceptance gate. Done requires client/adapter compilation, mock integration, verifier-only enforcement, backend/config loading, scaffolded SSB variants, working remote boot and dual tunnel, opt-in behavior, zero new workspace errors, all scoped tests, and at least six new PPM tests. (`D2:279-295`)
- **D003-D2-E036**: `[E]` — hardware smoke gate. Before the hardware-heavy test, chat must flag the boot; both sidecars must stay healthy together for at least five minutes without OOM and leave at least 6 GB GPU memory free. (`D2:296-299`)
- **D003-D2-E037**: `[E]` — score-quality smoke gate. A known-correct step must score at least `0.7`, a known-wrong step at most `0.3`, and the margin must exceed `0.3`. (`D2:300-302`)
- **D003-D2-E038**: `[E]` — outage smoke gate. With PPM enabled but offline, `PpmUnavailable` must be journaled and the reasoning step must not auto-pass; final consensus behavior is delegated to D3. (`D2:303-305`)
- **D003-D2-E039**: `[E]` — documentary done gates. All paper interfaces, the three-decision record, and post-`AUDITING` memory artifact `project_d2_ppm_sidecar.md` are required. (`D2:306-309`)
- **D003-D2-E040**: `[E]` — code/config surfaces. The plan adds client, adapter, CLI health, boot script, integration test, decision doc and paper interfaces; modifies role runtime, SSB, CLI commands, TOML, tunnel script and remote-sidecar documentation. (`D2:311-329`)
- **D003-D2-E041**: `[E]` — non-goals. D3 aggregation, D19 benchmarking, D28 retraining, and distilled-PPM training are excluded from D2. (`D2:331-337`)
- **D003-D2-E042**: `[E]` — next dependency. D3 is the immediate consumer; D19 is described as able to proceed alongside D3 because their write surfaces are disjoint. (`D2:339-343`)

### D-3: verification result aggregator

- **Status**: PLANNED — deps D-2; Phase 4H.V.2 / 2-verifier consensus
- **Superseded:** SUP-013 — D-3 deceptive consensus default. Target: Default AtLeastTwo can pass PPM+intrinsic with deterministic fail; code claims use RequireAll. Loser: Summary unconditional deterministic-fail blocks promotion.
- **Anchors**: plan `D-3_verification_result_aggregator.md`; DEEP-D-001..004; aggregate unit (37 declared source atoms); compiled 37
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4 

**Behaviors**

- **D003-D3-E001**: `[E]` — capability. D3 aggregates deterministic, PPM, intrinsic GG, and cross-context signals into `VerificationResult`, with a stated baseline requirement of at least two of four concurring before template promotion. (`D3:1-10`)
- **D003-D3-E002**: `[E]` — prerequisites/concurrency. Estimated effort is 1–2 sessions; D2 and D1 are prerequisites; D5 is declared parallel-safe despite consuming the result, while D4 is not parallel-safe because it monitors D3 rejection/disagreement signals. (`D3:11-17`)
- **D003-D3-E003**: `[E]` — intrinsic integration. Existing scattered `mean_logprob`, `entropy_max`, and rolling-Jaccard `novelty` metrics are consolidated into `IntrinsicSignals`. (`D3:34-37`)
- **D003-D3-E004**: `[E]` — cross-context behavior. A claim runs in two independent context windows with different prior-step ordering; agreement is treated as robustness and disagreement as a flag. (`D3:39-42`)
- **D003-D3-E005**: `[E]` — foundations. D3 reuses deterministic dispatch hooks, D2’s client/adapter, Phase-4G GG fields, and D1’s per-template `verifier_required`. (`D3:44-49`)
- **D003-D3-E006**: `[E]` — research gate. `vstar-2402.06457-interface.md` and `ccv-2603.21454-interface.md` must exist before `TESTING_FIRST`. (`D3:69-76`)
- **D003-D3-E007**: `[E]` — aggregate schema. `VerificationResult` has optional `deterministic`, `ppm`, `intrinsic`, and `cross_context` fields plus `consensus_rule` and final `verdict`. (`D3:80-85`)
- **D003-D3-E008**: `[E]` — deterministic schema. `DeterministicCheck` contains `kind`, `passed`, and optional `stderr_snippet`; `DeterministicKind` is the closed set `Compile`, `Test`, `Lint`, `Schema`, `RegistryCheck`. (`D3:86-88`)
- **D003-D3-E009**: `[E]` — intrinsic schema. `IntrinsicSignals` contains `mean_logprob`, `entropy_max`, and `novelty`, all with ranges required to be documented in the struct. (`D3:89-90`)
- **D003-D3-E010**: `[E]` — cross-context schema/default. `CrossContextAgreement` stores two Blake3 output hashes, Jaccard similarity, and Boolean agreement; the default agreement threshold is `0.7` and lives in L3-locked `[verification]` config. (`D3:91-94`)
- **D003-D3-E011**: `[E]` — rule schema. `ConsensusRule` is a closed, L3-locked enum: `AtLeastTwo`, `AtLeastThree`, `AtLeastN(u8)`, or `RequireAll(Vec<SignalKind>)`. (`D3:95-96`)
- **D003-D3-E012**: `[E]` — signal/verdict schemas. The initial `SignalKind` set is `Deterministic`, `Ppm`, `Intrinsic`, `CrossContext`; `Verdict` is `Pass`, `Reject`, or `Inconclusive`. (`D3:97-98`)
- **D003-D3-E013**: `[E]` — default computation. `AtLeastTwo` counts deterministic `passed`, PPM score `>=0.7`, intrinsic `mean_logprob >= intrinsic_threshold`, and cross-context agreement; at least two passes means `Pass`, zero or one means `Reject`, while a missing required signal makes the result `Inconclusive`. (`D3:100-108`)
- **D003-D3-E014**: `[E]` — alternative computations. `AtLeastN(n)` uses the corresponding pass count; `RequireAll(kinds)` requires every named kind to be present and passing; the absent-config default is `AtLeastTwo`. (`D3:100-108,188-192`)
- **D003-D3-E015**: `[E]` — runner schema. `CrossContextRunner` contains `Arc<dyn Executor>` and a Jaccard threshold, and `run(&Claim, &OrchContext)` returns `CrossContextAgreement`. (`D3:110-115`)
- **D003-D3-E016**: `[E]` — concurrency/determinism. The runner starts two executor futures concurrently, shuffles prior steps, Blake3-hashes each output, then computes token-level 3-shingle Jaccard similarity. (`D3:114-118,121-122`)
- **D003-D3-E017**: `[E]` — timeout recovery. Each cross-context side is bounded to 30 seconds; timeout returns `Inconclusive` rather than panic or infinite wait. (`D3:119-122`)
- **D003-D3-E018**: `[E]` — dispatch workflow. Every claim-emitting candidate dispatches an applicable deterministic check, conditional PPM when `verifier_required==Prm`, PPM enablement and health all hold, always captures intrinsic signals, conditionally runs cross-context when the rule demands it, computes a verdict, and emits an SSB row. (`D3:124-135,228-230`)
- **D003-D3-E019**: `[E]` — checker fallback. Before D11 exists, deterministic dispatch uses current compile/test hooks and skips inapplicable checks; D11 later replaces this with typed registry dispatch. (`D3:127-129,331-336`)
- **D003-D3-E020**: `[E]` — schema migration. D3 atomically bumps `SCHEMA_VERSION 48→49` for both D2 PPM events and all D3 verifier events; an old v48 DB must emit a migration warning. (`D3:137-140,286-292`)
- **D003-D3-E021**: `[E]` — pass/reject events. `VerifierConsensusPass` contains template/candidate IDs, passing signals and rule string; `VerifierConsensusReject` contains IDs, `(SignalKind, reason)` failures and rule string. (`D3:141-145`)
- **D003-D3-E022**: `[E]` — disagreement event. `VerifierDisagreement` contains template ID and signal/status pairs and fires when at least one signal passes and at least one rejects, because disagreement is operationally distinct from rejection. (`D3:146-147,202-205`)
- **D003-D3-E023**: `[E]` — cross-context event. `CrossContextDispatched` records claim ID, both context hashes, Jaccard score, and agreement; every new variant needs payload arm, SQL serialization and round-trip proof. (`D3:148-150`)
- **D003-D3-E024**: `[E]` — config defaults. `[verification]` defaults are `consensus_rule="at_least_two"`, `intrinsic_logprob_threshold=-3.0`, `intrinsic_entropy_max=3.5`, `intrinsic_novelty_min=0.2`, and `cross_context_jaccard_threshold=0.7`; evaluator-side values are L3-locked. (`D3:152-157`)
- **D003-D3-E025**: `[E]` — unit evidence. Tests cover pass-count rules, required-signal absence, TOML deserialization, timeout-to-inconclusive, deterministic Jaccard on identical input, and disagreement emission. (`D3:159-168`)
- **D003-D3-E026**: `[E]` — deceptive-case correction in source. The plan explicitly notices that deterministic fail + PPM pass + intrinsic pass would pass `AtLeastTwo`; it revises the code-claim scenario to `RequireAll([Deterministic, Ppm])`, under which deterministic failure rejects. (`D3:170-184`)
- **D003-D3-E027**: `[E]` — integration scenarios. Integration uses mock executor/PPM/deterministic checker for ordinary pass, deceptive code rejection under `RequireAll`, and cross-context disagreement, asserting rows through an in-memory SSB writer. (`D3:170-186`)
- **D003-D3-E028**: `[E]` — contract evidence. Contract tests require public schema `49`, round-trips for every new event, stable `ConsensusRule` serde, and the `AtLeastTwo` default when config is absent. (`D3:188-192`)
- **D003-D3-E029**: `[E]` — decision record. D3 must justify the `AtLeastTwo` latency/gate tradeoff, distinct Blake3 deduplication versus Jaccard similarity, and separate disagreement telemetry for D4 drift monitoring. (`D3:194-205`)
- **D003-D3-E030**: `[E]` — consumers. D5 consumes `VerifierConsensusPass` counts; D4 consumes disagreement rate; every claim-emitting post-`EXECUTING` baby step reaches `dispatch_verifiers`. (`D3:207-232`)
- **D003-D3-E031**: `[E]` — fail-closed constraints. Missing required signals cannot be swallowed, cross-context cannot wait forever, every candidate must emit one result-shaped SSB row even on unavailable signals, and the rule must load from locked config rather than an in-code constant. (`D3:233-239`)
- **D003-D3-E032**: `[E]` — observable proof. A cloud deceptive candidate that PPM accepts but compilation rejects must yield a `VerifierConsensusReject` with deterministic stderr plus `PpmScoreEmitted`, and no `TemplatePromoted`. (`D3:240-246`)
- **D003-D3-E033**: `[E]` — deterministic done probes. Required probes cover schema/version/event symbols, aggregate enums, cross-context runner, config, task-runner dispatch, workspace/scoped/contract tests, a negative single-verifier bypass search, and both paper interfaces. (`D3:248-284`)
- **D003-D3-E034**: `[E]` — acceptance volume. Done requires at least ten table-driven consensus scenarios, at least twelve new orchestrator tests, contract tests, config defaults, one result per candidate, all six D2+D3 event round-trips, zero new build errors, and timeout safety. (`D3:286-303`)
- **D003-D3-E035**: `[E]` — cloud gates. Cloud proofs require deceptive rejection/no promotion, simple-math cross-context agreement `>=0.7`, and PPM-offline behavior yielding `Inconclusive` plus `PpmUnavailable` and either rejection or an explicit inconclusive flag according to rule. (`D3:304-310`)
- **D003-D3-E036**: `[E]` — documentary gates/surfaces. Paper interfaces, three-decision doc and `project_d3_verification_result.md` are required; implementation adds aggregate/runner/test/contract/docs and modifies SSB, task runner, config and module exports. (`D3:311-329`)
- **D003-D3-E037**: `[E]` — non-goals/next. D4 tripwires, D5 BAI, D11 typed checkers and D13 adversary are outside D3; D4 and D5 are its next disjoint consumers. (`D3:331-341`)

### D-13: adversary role cross family

- **Status**: PLANNED — deps D-10, D-3; RPS.IV / Mode-B cross-family adversary
- **Anchors**: plan `D-13_adversary_role_cross_family.md`; DEEP-D-001..004; aggregate unit (32 declared source atoms); compiled 32
- **Composes with**: D-001/IX-004 — **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lac

**Behaviors**

- **D003-D13-E001**: `[E]` — capability. D13 adds `AgentRole::Adversary` for `GroundingMode::FalsifiabilityCrossFamily`; it must run on a different model family and return structured counter-evidence. (`D13:1-11`)
- **D003-D13-E002**: `[E]` — prerequisites/concurrency. Estimated effort is 2–3 sessions; D10 routing and D3 consensus are prerequisites; D14 and D15 are declared parallel-safe, although D14 shares the new cross-family infrastructure. (`D13:12-16`)
- **D003-D13-E003**: `[E]` — routing priority. Mode B is explicitly last-resort: most claims should route to constructive Mode A or observable Mode D first. (`D13:18-22`)
- **D003-D13-E004**: `[E]` — hard verifier rules. The adversary must be cross-family, its output must be structured with source-tier citations, a separate auditor—not the executor—decides from claim plus counter-evidence, and absence of an alternate family skips Mode B for Mode E with no self-critique. (`D13:23-31`)
- **D003-D13-E005**: `[E]` — deployment fallback. With local Qwen policy, cloud Sonnet is the default cross-family adversary; Qwen Math PPM is explicitly unsuitable, and cloud plus alternate-local unavailability results in Mode E. (`D13:33-37`)
- **D003-D13-E006**: `[E]` — foundations. The plan extends existing `AgentRole`, B37 per-role runtime, D3 verification, and D10’s `FalsifiabilityCrossFamily` branch. (`D13:39-44`)
- **D003-D13-E007**: `[E]` — research gate. Three paper interfaces—Self-Correction Bench, Constitution or Collapse, and Constitutional AI in Small LLMs—must be present. (`D13:57-65`)
- **D003-D13-E008**: `[E]` — role permission metadata. `AgentRole::Adversary` gains metadata requiring its runtime family to be cross-family from the executor, enforced at spawn. (`D13:67-73`)
- **D003-D13-E009**: `[E]` — config schema. `AdversaryConfig` contains `runtime_role`, `allowed_families`, `max_response_tokens`, and a `CounterEvidenceSchema`. (`D13:75-79`)
- **D003-D13-E010**: `[E]` — model-family schema. `ModelFamily` enumerates `Qwen25Coder`, `Qwen25Math`, `AnthropicClaude`, `OpenAiGpt`, `DeepSeek`, and extensible `Other(String)`. (`D13:80-82`)
- **D003-D13-E011**: `[E]` — family predicate. `cross_family_check(executor, adversary)` must reject same-family pairs and force the caller toward Mode E. (`D13:83-85`)
- **D003-D13-E012**: `[E]` — counter-evidence schema. `CounterEvidence` contains a `CounterVerdict`, evidence points, source-tier citations, and confidence; verdicts are `ClaimRobust`, `ClaimWeak`, or `ClaimFalse`. (`D13:87-92`)
- **D003-D13-E013**: `[E]` — evidence schema. Each `EvidencePoint` has `EvidenceKind`, a `BoundedString<2048>`, and an optional citation; kinds are `ContradictoryExample`, `MissingPrecondition`, `LogicalGap`, and `AlternateInterpretation`. (`D13:92-95`)
- **D003-D13-E014**: `[E]` — citation schema. Citations are typed as `T1Arxiv { id }`, `T1Nature { doi }`, `T2OrgBlog { url }`, or `T3Other { url, note }`. (`D13:96-97`)
- **D003-D13-E015**: `[E]` — agent surface. New `.claude/agents/adversary.md` instructs the strongest falsehood-seeking critic, requires explicit `ClaimRobust` when no counter-evidence exists, forbids hedging, returns matching JSON, cites the two motivating papers, and runs on Sonnet by default. (`D13:99-109`)
- **D003-D13-E016**: `[E]` — probe schema. `AdversaryProbe` owns a runtime adapter, config, and SSB sender; `probe` is async and returns `Result<CounterEvidence, AdversaryError>`. (`D13:111-115`)
- **D003-D13-E017**: `[E]` — probe workflow/failures. Probe first checks alternate-family availability, then rejects same-family runtime, loads the agent prompt, invokes the adapter, parses and validates JSON, and finally emits `AdversaryVerdict`; the two preflight failures are `CrossFamilyUnavailable` and `SameFamily`. (`D13:114-123`)
- **D003-D13-E018**: `[E]` — mode dispatch states. In `FalsifiabilityCrossFamily`, unavailable family returns `ClaimResult::Refused` with reason `"no cross-family adversary"` and journals `CrossFamilyUnavailable`; robust maps to `Verified`, false to `Falsified`, weak to `Inconclusive`. (`D13:125-135`)
- **D003-D13-E019**: `[E]` — consensus integration. D3 gains `SignalKind::Adversary` and `VerificationResult.adversary`; `ClaimRobust` contributes pass, `ClaimFalse` reject, and `ClaimWeak` neither/inconclusive. (`D13:137-142`)
- **D003-D13-E020**: `[E]` — event schemas/version. Schema `58→59`; `AdversaryVerdict` stores claim, verdict, confidence, evidence count, runtime family and audit ID, while `CrossFamilyUnavailable` stores claim, executor family and reason. (`D13:144-149`)
- **D003-D13-E021**: `[E]` — locked defaults. `[adversary]` is L3-locked, defaults allowed families to Anthropic Claude, OpenAI GPT and DeepSeek, and caps response at `4096` tokens. (`D13:151-154`)
- **D003-D13-E022**: `[E]` — unit evidence. Tests require Qwen Coder + Claude to pass cross-family, Qwen Coder + Qwen Math to fail, JSON round-trip, valid arXiv ID parsing and malformed-ID `ParseError`. (`D13:156-162`)
- **D003-D13-E023**: `[E]` — integration evidence. Mock Sonnet/Qwen scenarios cover robust Tokio verification, false Rust-GC falsification, and no-cloud refusal plus `CrossFamilyUnavailable`. (`D13:164-172`)
- **D003-D13-E024**: `[E]` — decisions. The decision doc must justify cloud Sonnet and session cost throttling, typed instead of narrative evidence, and fail-closed Mode E instead of same-model critique. (`D13:174-183`)
- **D003-D13-E025**: `[E]` — reachability. The feature is reachable from D10 dispatch, D3 consensus, and per-Mode-B spawning of the agent definition. (`D13:185-206`)
- **D003-D13-E026**: `[E]` — schema/fallback bans. Same-family must emit unavailability, malformed or freeform output must be an `AdversaryError`/`ParseError` rather than text fallback, allowed families must come from TOML, and structured schema validation is mandatory. (`D13:207-213`)
- **D003-D13-E027**: `[E]` — observable proof. The synthetic false Rust-GC claim must produce `ModeSelected(mode=FalsifiabilityCrossFamily)`, one `AdversaryVerdict(ClaimFalse)` with at least two evidence points, a falsified result, agent file, and config section. (`D13:214-221`)
- **D003-D13-E028**: `[E]` — deterministic done probes. Probes assert schema 59, event/types/role/signal/config symbols, agent and paper files, workspace build, and scoped adversary tests. (`D13:223-256`)
- **D003-D13-E029**: `[E]` — acceptance gate. Done requires both event round-trips, at least ten tests, role spawn enforcement, schema round-trip, config loading, D3 signal integration, zero build errors, three cloud branches, all paper interfaces and the decision doc. (`D13:258-280`)
- **D003-D13-E030**: `[E]` — code/config surfaces. D13 adds adversary Rust/agent/test/decision/paper files and modifies role, SSB, grounding mode, verification result, config and module export surfaces. (`D13:282-298`)
- **D003-D13-E031**: `[E]` — non-goals/lifecycle. D14 probing is separate; a multi-adversary ensemble and critique caching are deferred, so the first version performs one fresh probe per claim. (`D13:300-305`)
- **D003-D13-E032**: `[E]` — next dependency. D14 reuses this infrastructure and D23 later gates D14. (`D13:307-310`)

### D-16: unicr conformal refusal

- **Status**: PLANNED — deps D-9, D-10; RPS.VII / Mode-E conformal refusal
- **Anchors**: plan `D-16_unicr_conformal_refusal.md`; DEEP-D-001..004; aggregate unit (31 declared source atoms); compiled 31
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-16

**Behaviors**

- **D003-D16-E001**: `[E]` — capability/claim. D16 implements a Mode-E refusal controller presented as a distribution-free formal guarantee that refusal risk remains within user epsilon under distribution shift. (`D16:1-8`)
- **D003-D16-E002**: `[E]` — prerequisites/concurrency. Estimated effort is 3–4 sessions; D9 frozen-surface enforcement and D10 Mode-E slot are prerequisites; D17 and D22 are declared parallel-safe. (`D16:9-14`)
- **D003-D16-E003**: `[E]` — risk definition/default. The calibration set records claim/truth-availability/confidence; the plan defines empirical risk as refusing when truth exists plus accepting when truth does not, with default `epsilon=0.1`. (`D16:18-24`)
- **D003-D16-E004**: `[E]` — refusal lifecycle. Tau updates nightly under L3 control, and a claim is refused when a GG-signal composite confidence falls below tau. (`D16:25-28`)
- **D003-D16-E005**: `[E]` — foundations. D16 consumes D3 `IntrinsicSignals`, D9 locked keys, and D10 `GroundingMode::Refuse`. (`D16:33-37`)
- **D003-D16-E006**: `[E]` — research gate. UniCR, Uncertainty Outside Parametric Knowledge, and Rewarding Doubt interfaces must be ingested. (`D16:47-55`)
- **D003-D16-E007**: `[E]` — tuple schema. `CalibrationTuple` stores `claim_id`, `ground_truth_available`, `model_confidence`, `claim_type`, `claim_outcome`, and `recorded_at`; outcomes are `Verified`, `Falsified`, `Refused`, `NoCheck`. (`D16:57-65`)
- **D003-D16-E008**: `[E]` — corpus schema/lifecycle. `CalibrationCorpus` contains tuple vector and storage path; `load` walks `path/*.toml`, `append` writes `{claim_id}.toml`, and `size` reports tuple count. (`D16:65-70`)
- **D003-D16-E009**: `[E]` — tau computation. `compute_tau` evaluates `r(tau)=P(refuse|truth_exists)+P(accept|truth_doesnt_exist)`, selects the minimal candidate satisfying `r<=epsilon` by binary search over a `0.01` grid in `[0,1]`, and returns `InsufficientData` below 500 tuples. (`D16:72-81`)
- **D003-D16-E010**: `[E]` — updater schema/action. `TauUpdater` holds storage path, epsilon, and current tau; nightly update loads the corpus, recomputes tau, writes `.elai_cc/calibration/tau.txt` with timestamp, and emits `CalibrationUpdate`. (`D16:83-92`)
- **D003-D16-E011**: `[E]` — permission/integrity. `TauUpdater`, `compute_tau`, weights/config, and `tau.txt` are L3-protected; only the updater may write the threshold, preventing agent-side bias. (`D16:83-92,133-138,180-187`)
- **D003-D16-E012**: `[E]` — confidence function. `refusal_decision` computes `w1*mean_logprob + w2*(-entropy_max) + w3*novelty` using locked TOML weights, and returns true exactly when the composite is below tau. (`D16:94-99`)
- **D003-D16-E013**: `[E]` — routing integration. D10’s `intrinsic_signals_below_threshold` delegates to `refusal_decision`; `ctx.unicr_tau` loads from `tau.txt` at startup. (`D16:101-105`)
- **D003-D16-E014**: `[E]` — data growth. After each claim resolution, `task_runner` appends a tuple, derives truth availability from checker output, records decision-time composite confidence, and keeps the hook L1A/data-only with no LLM judgment. (`D16:107-111`)
- **D003-D16-E015**: `[E]` — status CLI. `elai calibration status` prints corpus size, current tau, update timestamp, epsilon, and empirical refusal rate over the last N sessions; it is informational and always exits `0`. (`D16:113-117`)
- **D003-D16-E016**: `[E]` — update CLI/authorization. `elai calibration update` manually invokes nightly logic; CI invocation requires commit tag `L3_OVERRIDE:calibration_update`, while development invocation is unrestricted. (`D16:119-124`)
- **D003-D16-E017**: `[E]` — refusal event. `RefusalDecision` stores claim ID, composite confidence, tau, and Boolean refused. (`D16:126-128`)
- **D003-D16-E018**: `[E]` — update event/version. `CalibrationUpdate` stores tuple count, new/old tau, epsilon and audit ID; both events round-trip under schema `61→62`. (`D16:126-131`)
- **D003-D16-E019**: `[E]` — config defaults. `[conformal]` defaults: `epsilon=0.1`, `min_corpus_size=500`, weights `{mean_logprob=0.4, entropy=-0.3, novelty=0.3}`, tau path `.elai_cc/calibration/tau.txt`, corpus path `.elai_cc/calibration/corpus/`. (`D16:133-138`)
- **D003-D16-E020**: `[E]` — unit/idempotency evidence. Tests cover known-split tau, insufficient data, at least eight threshold cases, and identical-corpus nightly-update idempotency. (`D16:140-145`)
- **D003-D16-E021**: `[E]` — integration evidence. A synthesized 600-tuple corpus must produce tau and an empirical refusal rate asserted within `[tau-epsilon, tau+epsilon]`. (`D16:146-150`)
- **D003-D16-E022**: `[E]` — decisions. The decision doc records epsilon `0.1`, minimum `500` based on the cited finite-sample bound, and why composite GG evidence is preferred to raw mean log probability. (`D16:152-159`)
- **D003-D16-E023**: `[E]` — reachability. D16 is reachable from the Mode-E decision point, a nightly CI/cron updater, and status/update CLI surfaces. (`D16:161-179`)
- **D003-D16-E024**: `[E]` — fail-closed constraints. No small-sample approximation is allowed, every update must journal, refusal must use the weighted composite, and direct writes to `tau.txt` are forbidden. (`D16:180-187`)
- **D003-D16-E025**: `[E]` — observable proof. A 600-tuple nightly update must emit `CalibrationUpdate`, persist tau and at least 600 tuple files, and cause a later low-confidence cloud claim to emit `RefusalDecision(refused=true)`. (`D16:188-193`)
- **D003-D16-E026**: `[E]` — deterministic done probes. The plan checks schema/event/type/config/lock symbols, build/tests, and three paper interfaces. (`D16:195-210`)
- **D003-D16-E027**: `[E]` — acceptance gate. Done requires schema 62, two event round-trips, corpus load/append, exact/insufficient-data/idempotency tests, lock registration, config load, at least twelve tests, and zero new workspace errors. (`D16:212-224`)
- **D003-D16-E028**: `[E]` — cloud branches. Low confidence must route to Mode E and journal refusal; high confidence must choose another mode with no refusal row in that branch; manual update with override must journal and update the file. (`D16:225-230`)
- **D003-D16-E029**: `[E]` — documentary/surface gates. Three paper interfaces and the decision doc are required; the code adds calibration and CLI modules/tests/docs and modifies SSB, route selection, task runner, config, locked-key registry, CLI commands, and exports. (`D16:231-252`)
- **D003-D16-E030**: `[E]` — non-goals. Distribution-shift validation is D22, generic bootstrap is deferred to D30, and per-family/multi-domain tau is deferred unless D30 establishes need. (`D16:254-259`)
- **D003-D16-E031**: `[E]` — downstream. D17 consumes refusal events; D22 validates shifted behavior. (`D16:261-264`)

### D-17: honest refusal sla

- **Status**: PLANNED — deps D-16; RPS.VIII / refusal-rate SLA
- **Superseded:** SUP-009 — D-17 SLA event name. Target: RefusalSlaBreach + AlertKind::RefusalRateOutOfBand. Loser: Summary AUTO_FLAG.
- **Anchors**: plan `D-17_honest_refusal_sla.md`; DEEP-D-001..004; aggregate unit (24 declared source atoms); compiled 24
- **Composes with**: D-001/IX-007 — **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance

**Behaviors**

- **D003-D17-E001**: `[E]` — metric/capability. D17 computes per-session `refusal_rate=refusals/total_claims`; below `5%` means likely confabulation, above `50%` means checker misconfiguration or an overly hard task suite. (`D17:1-7`)
- **D003-D17-E002**: `[E]` — prerequisites/concurrency. Estimated effort is one session; D16 refusal events are required, and D18 is declared parallel-safe. (`D17:8-10`)
- **D003-D17-E003**: `[E]` — default versus research uncertainty. The design uses broad inclusive healthy band `5–50%` while treating `10–30%` as an unvalidated guess to be tuned from actual session statistics. (`D17:12-20,87-92`)
- **D003-D17-E004**: `[E]` — foundations/UI dependency. It aggregates D16 `RefusalDecision` rows through `SsbViewer`; D27 later renders the gauge. (`D17:22-25`)
- **D003-D17-E005**: `[E]` — monitor schema. `RefusalRateMonitor` stores a minute window, minimum/maximum healthy rates, and SSB DB path. (`D17:34-39`)
- **D003-D17-E006**: `[E]` — snapshot schema. `RefusalRateSnapshot` stores session ID, window start, refusal and total counts, rate, and `HealthBand`; the band is `Below {rate,min}`, `Healthy`, or `Above {rate,max}`. (`D17:40-44`)
- **D003-D17-E007**: `[E]` — query/workflow. `snapshot` queries `RefusalDecision` rows for numerator and `MODE_SELECTED` rows for denominator, computes rate and band, and emits `RefusalSlaBreach` for either nonhealthy band. (`D17:46-52`)
- **D003-D17-E008**: `[E]` — alert integration. `alert_on_breach` emits the existing B13 Bubble-Up pattern with `AlertKind::RefusalRateOutOfBand` only when the band is not healthy. (`D17:54-57`)
- **D003-D17-E009**: `[E]` — scheduling. Task runner/orchestration invokes monitoring periodically; the prose says every N minutes with 60 default, while config separately defaults the check interval to 30 minutes and the window to 60. (`D17:59-62,69-72`)
- **D003-D17-E010**: `[E]` — event/version. Schema `62→63`; `RefusalSlaBreach` contains session ID, rate, healthy min/max, band, and total claims and needs round-trip proof. (`D17:64-67`)
- **D003-D17-E011**: `[E]` — config/default/security. L3-locked `[refusal_sla]` defaults are `enabled=true`, `window_size_minutes=60`, `healthy_min=0.05`, `healthy_max=0.50`, and `check_interval_minutes=30`. (`D17:69-72`)
- **D003-D17-E012**: `[E]` — unit evidence. Tests classify `0.02` below, `0.30` healthy, `0.60` above, verify synthetic SSB aggregation, and ensure healthy snapshots do not alert. (`D17:74-78`)
- **D003-D17-E013**: `[E]` — integration evidence. With 100 selected claims, 2 refusals alerts below, 60 alerts above, and 20 is healthy/no alert. (`D17:80-85`)
- **D003-D17-E014**: `[E]` — decisions. The plan records why the broad band is provisional, why scope is per session rather than global, and why breaches Bubble-Up for operator action instead of remaining silent. (`D17:87-94`)
- **D003-D17-E015**: `[E]` — reachability. D17 consumes D16, D10, SSB viewer/writer and B13 alerts; it is reached from periodic task-runner checks and D27 dashboard gauge. (`D17:96-111`)
- **D003-D17-E016**: `[E]` — zero-denominator recovery. `total_claims=0` is defined as `Healthy`, avoiding division by zero and suppressing an alert. (`D17:112-114`)
- **D003-D17-E017**: `[E]` — idempotency. Duplicate breach alerts in the same window must be suppressed using a window-end key. (`D17:115-116`)
- **D003-D17-E018**: `[E]` — config integrity. Thresholds must be loaded from TOML, not embedded in monitor code. (`D17:117`)
- **D003-D17-E019**: `[E]` — observable proof. A 2% synthetic session yields exactly one below-band breach plus Bubble-Up; D27’s gauge should be red. (`D17:118-121`)
- **D003-D17-E020**: `[E]` — deterministic done probes. Schema/event/monitor/config searches, workspace build and scoped tests are the deterministic checks. (`D17:123-132`)
- **D003-D17-E021**: `[E]` — acceptance gate. Done requires schema 63, event round-trip, correct aggregation/bands, Bubble-Up wiring, config loading, at least six tests, zero new errors, a 2% cloud breach/alert, and no event inside the healthy band. (`D17:134-149`)
- **D003-D17-E022**: `[E]` — surfaces. D17 adds monitor, integration test and decision doc; modifies SSB, task runner, config and module exports. (`D17:151-160`)
- **D003-D17-E023**: `[E]` — non-goals. Dashboard rendering, per-family band tuning, and long-term trend reporting are excluded; the latter two are explicitly deferred. (`D17:162-166`)
- **D003-D17-E024**: `[E]` — next. D18 completes Wave 2 after D17. (`D17:168-170`)

### D-19: prmbench adversarial gaming

- **Status**: PLANNED — deps D-2, D-3; companion §1.13 / PPM adversarial baseline
- **Superseded:** SUP-018 — D-19 CI enforcement. Target: Informational warn-not-block until production-gating transition. Loser: Summary direct 'gates PPM promotion' without CI nuance.
- **Anchors**: plan `D-19_prmbench_adversarial_gaming.md`; DEEP-D-001..004; aggregate unit (24 declared source atoms); compiled 24
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-19

**Behaviors**

- **D003-D19-E001**: `[E]` — capability. D19 evaluates D2’s PPM on PRMBench Deception Resistance and Circular Logic, reports per category, and frames the result as a conservative promotion gate. (`D19:1-8`)
- **D003-D19-E002**: `[E]` — prerequisites/compute/concurrency. Estimated effort is two sessions; D2 and D3 are prerequisites; it needs the remote sidecar but not RunPod and is parallel-safe with D20–D25. (`D19:9-12`)
- **D003-D19-E003**: `[E]` — baseline context. The context cites 60–65% for top PRMs in the two weak categories and distinguishes discriminative, LLM-judge, and ThinkPRM evidence. (`D19:14-22`)
- **D003-D19-E004**: `[E]` — failure decision. Below-baseline evidence permits either accepting the limitation under D3 consensus or deferring promotion until D28 adversarial farming improves the PPM. (`D19:24-28`)
- **D003-D19-E005**: `[E]` — research gate. `prm-lessons-2501.07301-interface.md` and `thinkprm-2504.16828-interface.md` must be ingested. (`D19:41-47`)
- **D003-D19-E006**: `[E]` — fixture lifecycle/compliance. The public HuggingFace dataset license must be checked; fixtures are filtered to the two initial categories and stored as category/task JSON. (`D19:49-58`)
- **D003-D19-E007**: `[E]` — fixture schema/volume. Each fixture stores problem, step history, candidate step, and correct/incorrect label; bootstrap volume is at least 100 per category, 200 total. (`D19:56-59`)
- **D003-D19-E008**: `[E]` — runner schema. `PRMBenchRunner` owns PPM plus fixture/output paths; `PRMBenchReport` stores category, counts, false positives/negatives, accuracy/precision/recall, baseline, and `GatingVerdict`. (`D19:61-68`)
- **D003-D19-E009**: `[E]` — verdict schema. `GatingVerdict` is `Pass`, `FailBelowBaseline`, or `InsufficientData`. (`D19:69`)
- **D003-D19-E010**: `[E]` — runner workflow. `run_category` walks fixtures, calls `score_step`, compares ground truth, accumulates counters and metrics, then compares against a configured baseline. (`D19:70-75`)
- **D003-D19-E011**: `[E]` — CLI behavior. `elai bench prmbench` evaluates all categories and exits `0` on pass, `1` on below-baseline failure. (`D19:77-80`)
- **D003-D19-E012**: `[E]` — shared benchmark events. Schema `64→65` adds reusable `BenchmarkStarted {name,fixture_count,audit_id}` and `BenchmarkResult {name,accuracy,baseline,verdict,audit_id}` with round-trip tests for D19–D25. (`D19:82-86`)
- **D003-D19-E013**: `[E]` — CI integration. `bench-prmbench.yml` runs on push, PR and weekly cron, boots the remote PPM when credentials allow, invokes the CLI, and publishes a report artifact. (`D19:88-92`)
- **D003-D19-E014**: `[E]` — nonblocking failure. Until PPM becomes production-gating, `FailBelowBaseline` warns but does not block merge because D3 treats PPM as one consensus signal. (`D19:93-94,96-103`)
- **D003-D19-E015**: `[E]` — configurable threshold. The 35% threshold must come from TOML or CLI arguments rather than be hardcoded. (`D19:120-123`)
- **D003-D19-E016**: `[E]` — timeout fallback. A PPM timeout marks the affected task `InsufficientData` and the category run continues. (`D19:120-121`)
- **D003-D19-E017**: `[E]` — evidence integrity. Every report must expose its baseline and verdict; silent pass is forbidden. (`D19:122`)
- **D003-D19-E018**: `[E]` — observable proof. A category CLI run produces one start row, one result row, and `.elai_cc/benchmarks/prmbench/reports/{date}-{category}.json`. (`D19:124-127`)
- **D003-D19-E019**: `[E]` — deterministic done probes. Checks cover schema/events/runner, at least 200 fixtures, CI workflow, build/tests, and both paper interfaces. (`D19:129-142`)
- **D003-D19-E020**: `[E]` — acceptance gate. Done requires schema/event round-trips, compiling runner/CLI/workflow, 200+ fixtures, at least six tests, zero new errors, persisted reports, paper interfaces and the decision doc. (`D19:144-160`)
- **D003-D19-E021**: `[E]` — numeric cloud gates. Deception Resistance passes at `>=35%`; Circular Logic passes at `>=30%`; lower results are reported as informational `FailBelowBaseline`. (`D19:154-158`)
- **D003-D19-E022**: `[E]` — surfaces. D19 adds runner, CLI, 200+ fixtures, CI workflow, decision and paper docs; modifies SSB, CLI command routing and exports. (`D19:162-175`)
- **D003-D19-E023**: `[E]` — non-goals. Other PRMBench categories, ThinkPRM integration, and the D28 adversarial retraining loop are deferred/excluded. (`D19:177-181`)
- **D003-D19-E024**: `[E]` — next. D20 follows. (`D19:183-185`)

### D-25: determinism archive tamper suite

- **Status**: PLANNED — deps D-8, C-27; D-8 + C-27 gating evidence / tamper suite
- **Anchors**: plan `D-25_determinism_archive_tamper_suite.md`; DEEP-D-001..004; aggregate unit (23 declared source atoms); compiled 23
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-25

**Behaviors**

- **D003-D25-E001**: `[E]` — capability/gate. D25 defines ten archive/determinism scenarios; applicable tamper must be caught within five seconds, a clean repository exits `0`, and reverting tamper restores clean verification. (`D25:1-8`)
- **D003-D25-E002**: `[E]` — dependencies/concurrency. D8 archive verification and C27 `DeterminismDrift` are prerequisites; D25 is parallel-safe with D19–D24. (`D25:6-8`)
- **D003-D25-E003**: `[E]` — scenario 1. Deleting a middle `ArchiveEntry` must produce `BrokenParent` on the following entry. (`D25:15-16`)
- **D003-D25-E004**: `[E]` — scenario 2. Replacing `parent_hash` with a nonexistent hash must produce `BrokenParent`. (`D25:17`)
- **D003-D25-E005**: `[E]` — scenario 3. Editing `template_body` without recomputing `self_hash` must produce `HashMismatch`. (`D25:18`)
- **D003-D25-E006**: `[E]` — scenario 4. Reusing one `self_hash` for two entries must produce `DuplicateHash`. (`D25:19`)
- **D003-D25-E007**: `[E]` — scenario 5. A malformed JSON line must produce `MalformedJson`. (`D25:20`)
- **D003-D25-E008**: `[E]` — scenario 6. Two `elai /task` runs with the same `ELAI_SEED` but divergent `action_type` sequences must emit `DeterminismDrift`. (`D25:21-22`)
- **D003-D25-E009**: `[E]` — scenario 7/control. Same-seed identical action sequences must emit zero `DeterminismDrift` rows. (`D25:23`)
- **D003-D25-E010**: `[E]` — scenario 8. Rewriting Blake3 content-hash bytes as SHA-256 on disk must produce `HashMismatch`. (`D25:24-25`)
- **D003-D25-E011**: `[E]` — scenario 9/recovery. Restoring a tampered archive from backup must make verification exit `0` again, proving recovery/idempotency. (`D25:26,48-52`)
- **D003-D25-E012**: `[E]` — scenario 10/full collection. A two-line tamper at lines 3 and 7 must report two errors when verification runs in full-collect mode, exercising continue versus fail-fast semantics. (`D25:27,48-53`)
- **D003-D25-E013**: `[E]` — fixture layout. `.elai_cc/benchmarks/tamper/scenarios/` contains ten directories, each with setup script and expected verdict; scripts generate clean state, alter it, and invoke verification. (`D25:34-38`)
- **D003-D25-E014**: `[E]` — runner workflow. `TamperRunner` executes setup and `elai archive verify`, records exit/error types, and compares them to the expected verdict. (`D25:40-43`)
- **D003-D25-E015**: `[E]` — CLI behavior. `elai bench tamper` runs all ten and exits `0` only when every scenario matches expectation. (`D25:45-46`)
- **D003-D25-E016**: `[E]` — produced/consumed interfaces. D25 produces `TamperRunner`, `TamperReport`, ten scripts and CLI; consumes D8 CLI, C27 drift detection and `SsbWriter`; it is reachable from CLI and CI. (`D25:55-66`)
- **D003-D25-E017**: `[E]` — constraints. The contract forbids missing a scenario, false-positive failure on clean state, or failure to recover after revert. (`D25:67-70`)
- **D003-D25-E018**: `[E]` — observability. One full run expects ten `BenchmarkStarted`, ten `BenchmarkResult`, and a report under `.elai_cc/benchmarks/tamper/reports/`. (`D25:71-74`)
- **D003-D25-E019**: `[E]` — deterministic check. Static checks require exactly ten scenario entries and a clean workspace compile. (`D25:76-82`)
- **D003-D25-E020**: `[E]` — acceptance gate. Done requires all directories, compiling runner, all-match exit semantics, zero build errors, every applicable tamper within five seconds, clean exit `0`, recovery after revert, and the decision doc. (`D25:84-93`)
- **D003-D25-E021**: `[E]` — surfaces. D25 adds runner, CLI, `{1..10}/setup.sh`, `expected.toml`, and decision doc and modifies command routing. (`D25:95-101`)
- **D003-D25-E022**: `[E]` — non-goals. Git cryptographic signing and multiparty signing are deferred. (`D25:103-107`)
- **D003-D25-E023**: `[E]` — lifecycle. D25 declares Wave 3 complete and hands off all-feature dashboard work to D26–D27. (`D25:109-111`)

### D-30: calibration bootstrap corpus

- **Status**: mixed / partial — Rust dispatch claimed wired; corpus/data legs DEFERRED (SUP-006)
- **Superseded:** SUP-006 — D-30 calibration bootstrap status. Target: Partial: Rust dispatch claimed wired/live; corpus/data/D22 legs DEFERRED. Loser: Summary blanket DEFERRED.
- **Anchors**: plan `D-30_calibration_bootstrap_corpus.md`; DEEP-D-001..004; aggregate unit (24 declared source atoms); compiled 24
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-30

**Behaviors**

- **D003-D30-E001**: `[E]` — status/objective. D30 is explicitly `DEFERRED`; it spikes a generic-corpus warm start because D16 needs 500 calibration tuples and early sessions cannot supply them. (`D30:1-10`)
- **D003-D30-E002**: `[E]` — dependencies/concurrency. Unblock requires D16 and a specific public HuggingFace corpus; D30 is parallel-safe with D28 and D29. (`D30:7-11`)
- **D003-D30-E003**: `[E]` — hypothesis/lifecycle. Candidate bootstrap data begins with generic QA/factual calibration, then local per-project data refines the threshold as sessions accumulate. (`D30:13-18`)
- **D003-D30-E004**: `[E]` — risk/fallback. Distribution mismatch between generic QA and ELAI code/reasoning may miscalibrate tau; nightly D16 updates limit bootstrap to starting-point status. (`D30:20-23`)
- **D003-D30-E005**: `[E]` — corpus candidates. Candidate sources are TruthfulQA, verified MMLU subsets, and HaluEval; the chosen source must publicly expose confidence labels and truth-availability flags and then receive paper ingestion. (`D30:32-40`)
- **D003-D30-E006**: `[E]` — selection/data surfaces. Unblocking adds `selection.md`, downloads `.elai_cc/calibration/bootstrap/{corpus}/`, and converts source records to D16 `CalibrationTuple` via a shim. (`D30:42-49`)
- **D003-D30-E007**: `[E]` — bootstrap persistence. D16 `compute_tau` runs on the bootstrap corpus and persists `.elai_cc/calibration/tau_bootstrap.txt`. (`D30:50-52`)
- **D003-D30-E008**: `[E]` — switchover lifecycle. Startup uses bootstrap tau while local corpus size is `<500` and switches to local-corpus tau once size is `>=500`. (`D30:53-54`)
- **D003-D30-E009**: `[E]` — validation workflow. D22’s runner tests bootstrap tau on synthetic ELAI claims and requires held-out empirical refusal behavior within epsilon; outside epsilon produces a distribution-mismatch Path-B report. (`D30:56-60`)
- **D003-D30-E010**: `[E]` — closure alternatives. Path A productizes bootstrap selection when it improves cold start; Path B closes with a concrete shifted-distribution blocker greater than epsilon. (`D30:61-66`)
- **D003-D30-E011**: `[E]` — decision lifecycle. `D-30-decisions.md` remains deferred and is drafted only at unblock. (`D30:68-69`)
- **D003-D30-E012**: `[E]` — implemented symbol inventory. The wiring contract names `warm_start_tau` at `calibration_bootstrap.rs:346`, `load_corpus` at `:292`, `CalibrationBootstrapCorpus` at `:189`, conversion at `:216`, `PublishedCalibrationRow` at `:148`, `BootstrapWarmStartOutcome` at `:239`, resampling at `:431`, CI helper at `:525`, and defaults at `:56/:65/:72`. (`D30:71-80`)
- **D003-D30-E013**: `[E]` — event/runtime inventory. It names `CalibrationBootstrapCompleted` plus payload at `ssb.rs:6746`, and `CalibrationBootstrapMode` at `runtime_config.rs:487`, gated by `ELAI_CALIBRATION_BOOTSTRAP_ENABLE`. (`D30:80-82`)
- **D003-D30-E014**: `[E]` — deferred data legs. Corpus download, `selection.md`, and `tau_bootstrap.txt` remain deferred because no corpus is selected. (`D30:82`)
- **D003-D30-E015**: `[E]` — consumed interfaces. Conversion feeds D16 `CalibrationCorpus`, warm start calls D16 `compute_tau`, mode comes from `CalibrationBootstrapMode::from_env` at runtime config line 499, D22 is deferred validation, and SSB writer journals completion. (`D30:83-88`)
- **D003-D30-E016**: `[E]` — live reachability/default. The environment-enabled path is wired at `task_runner.rs:5980` with an off-mode early-exit at `:16213`; default is wire-dark `Off`. (`D30:89-92`)
- **D003-D30-E017**: `[E]` — safety constraints. Warm start must enforce `DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE`, tolerate empty corpus without panic, always emit completion after a run, and use bootstrap tau below `PRODUCTION_FLOOR_TUPLES=500`. (`D30:93-97`)
- **D003-D30-E018**: `[E]` — observable proof contract. With enable flag and a present corpus, task execution should emit one completion row containing corpus slug and warm-start tau and write `tau_bootstrap.txt`. (`D30:97-100`)
- **D003-D30-E019**: `[E]` — partial implementation status. The plan explicitly says data/artifact/D22 legs are pending but the Rust dispatch path at the two task-runner anchors “IS wired and live.” (`D30:101-103`)
- **D003-D30-E020**: `[E]` — deterministic-check status. The entire deterministic-check section is `DEFERRED`; it supplies no executable check despite the named live Rust path. (`D30:105-107`)
- **D003-D30-E021**: `[E]` — acceptance/unblock gate. All acceptance boxes remain open; completion requires final corpus choice, chosen-corpus paper interface, persisted bootstrap tau, D22 Path A within epsilon or a concrete Path B mismatch, and decision doc. (`D30:109-117`)
- **D003-D30-E022**: `[E]` — anticipated surfaces. Remaining anticipated writes are corpus directory, selection document, bootstrap tau, D16 startup fallback modification, and decision doc. (`D30:119-126`)
- **D003-D30-E023**: `[E]` — non-goals. Custom corpus authoring and multi-corpus blending are deferred beyond D30. (`D30:128-132`)
- **D003-D30-E024**: `[E]` — terminal sequencing. The D plan completes only after D30 unblocks; D28–D30 otherwise operate independently and are prioritized by GPU availability and research value. (`D30:134-138`)

### D-4: tripwire stack

- **Status**: PLANNED — deps D-1, D-3; Phase 4H.VI / multi-signal auto-revert
- **Anchors**: plan `D-4_tripwire_stack.md`; DEEP-D-001..004; aggregate unit (52 declared source atoms); compiled 52
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-002 — **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 p | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.

**Behaviors**

- **D004-D4-001**: [E][capability] = (`Replace the undocumented MSI > 0.3 / 10%-over-3-pipelines rule with five orthogonal tripwires: KL divergence, correlation drop, IQR outlier, perplexity anomaly, and DDM; auto-revert requires at least two concurrent fires.`)
- **D004-D4-002**: [E][status] + (`Estimated effort is one session; the plan claims roughly 80% coverage of observed reward hacks and cites 21.3% prevalence as rationale, not as an acceptance result.`)
- **D004-D4-003**: [E][dependency] = (`Hard prerequisites are D-1 ArchiveChain/ArchiveEntry and D-3 VerifierDisagreement; spec-grounder must verify both DONE.`)
- **D004-D4-004**: [E][concurrency] + (`It is declared parallel-safe with D-5, D-6, and D-8, with explicit dataflow: tripwire fires feed BAI, held-out pass rate feeds Δρ, and rollback appends through ArchiveChain.`)
- **D004-D4-005**: [E][config] Δ (`Paper-derived thresholds are KL τ=0.3; Δρ>0.5 over 50-step windows; high outlier >Q3+3×IQR; PPL anomaly >μ+2σ; DDM warning/drift at 2σ/3σ; concurrent-fire count 2; window 50.`)
- **D004-D4-006**: [E][boundary] + (`Reward-signal checksums belong to D-8 and unauthorized reward-function syscall tracing belongs to D-9 LockedKeysAuditor; D-4 does not implement either.`)
- **D004-D4-007**: [E][lifecycle] Δ (`On concurrence in the same default 50-action window, retire the most recently Promoted template with RetirementReason::Degradation, switch the orchestrator to its parent_id, and retain an auditable append-only revert chain.`)
- **D004-D4-008**: [E][integration] + (`Existing inputs are ArchiveChain retired transitions, D-3 per-step proxy reward plus deterministic compile/test outcome as gold reward, Phase-4G entropy/mean_logprob as a PPL proxy, and 50-step SSB aggregation.`)
- **D004-D4-009**: [E][dependency] + (`Before TESTING_FIRST, paper-ingestor must create three interfaces: reward-hacking-detection-2507.05619, concept-drift-2004.05785, and rm-overoptimization-2210.10760.`)
- **D004-D4-010**: [E][schema] Δ (`Closed, L3-locked TripwireKind has KlDivergence, CorrelationDrop, IqrOutlier, PerplexityAnomaly, and Ddm.`)
- **D004-D4-011**: [E][schema] Δ (`TripwireConfig fields are kl_spec_tau, rho_delta_threshold, iqr_outlier_multiplier, ppl_anomaly_sigma, ddm_warning_threshold, ddm_drift_threshold, consensus_fire_count, and window_size, with the defaults in D004-D4-005.`)
- **D004-D4-012**: [E][schema] + (`TripwireEvent carries kind, UTC fired_at, value, threshold, SignalSource, and audit_id; SignalSource is CurrentSession, HeldOut, or CrossDomain.`)
- **D004-D4-013**: [E][algorithm] + (`kl_divergence applies standard KL; both inputs must sum to 1 within an unspecified epsilon, identical distributions return 0, and a nonzero current mass against zero baseline returns positive infinity.`)
- **D004-D4-014**: [E][workflow] + (`check_kl aggregates StepOutput slices into vocabulary-bin token histograms and emits a TripwireEvent only when KL is strictly greater than tau.`)
- **D004-D4-015**: [E][algorithm] + (`pearson_correlation requires equal lengths and returns 0 on zero variance; check_correlation_drop compares two windows and fires when corr_a−corr_b is strictly greater than the configured delta.`)
- **D004-D4-016**: [E][algorithm] + (`iqr_outlier_check computes Q1/Q3/IQR, fires if any value is strictly above Q3+multiplier×IQR, and places the outlier itself in event.value.`)
- **D004-D4-017**: [E][algorithm] + (`ppl_anomaly_check computes mean and standard deviation over its PPL values and fires if any value is strictly above mean+sigma_multiplier×standard deviation.`)
- **D004-D4-018**: [E][state] Δ (`DDM tracks p_i+s_i, where s_i=sqrt(p_i(1−p_i)/i); levels are Normal, Warning, Drift, with Normal→Warning at p_min+2s_min and Warning→Drift at p_min+3s_min.`)
- **D004-D4-019**: [E][recovery] + (`DdmState.update(success) emits only when entering Drift and is required to reset statistics when recovering from Drift to Normal.`)
- **D004-D4-020**: [E][schema] + (`TripwireOrchestrator owns config, a VecDeque ActionRecord window, DdmState, SsbSender, and Arc<RwLock<ArchiveChain>>.`)
- **D004-D4-021**: [E][determinism] + (`ingest appends every post-action ActionRecord, trims the oldest record deterministically at capacity, invokes all detectors over the current window, and aggregates their events without silent drops.`)
- **D004-D4-022**: [E][event] Δ (`Reaching consensus emits TripwireFiredConcurrent with the fired-kind list and audit ID.`)
- **D004-D4-023**: [E][lifecycle] Δ (`The automatic rollback invokes ArchiveChain.retire on the latest promoted entry through append-only ArchiveChain semantics.`)
- **D004-D4-024**: [E][event] Δ (`TripwireAutoRevert carries template_id, optional parent_template_id, fired_kinds, and audit_id; its intent event must be emitted before archive.retire so a failed retire remains auditable.`)
- **D004-D4-025**: [E][event] Δ (`SCHEMA_VERSION moves 49→50 and adds TripwireFired, TripwireFiredConcurrent, and TripwireAutoRevert with round-trip tests; individual fires include kind/value/threshold/source/audit_id.`)
- **D004-D4-026**: [E][config/security] + (`elai_cc.toml gains the exact [tripwire] defaults; thresholds must load from config rather than constructor constants and the section is operator-tunable but L3-locked against agent edits.`)
- **D004-D4-027**: [E][unit-test] + (`KL tests cover identical=0, disjoint=∞, and a hand-computed positive finite uniform-vs-peaked case.`)
- **D004-D4-028**: [E][unit-test] + (`Correlation tests cover +1/−1 linear cases, zero for uncorrelated data, and a window-to-window drop.`)
- **D004-D4-029**: [E][unit-test] + (`IQR tests require Q3+4IQR to fire and Q3+2.5IQR not to fire at multiplier 3.0.`)
- **D004-D4-030**: [E][unit-test] + (`PPL tests require μ+2.5σ to fire and μ+1.5σ not to fire at sigma 2.0.`)
- **D004-D4-031**: [E][unit-test] + (`DDM test drives 100 successes then 50 samples with 80% failure through Warning and Drift; acceptance additionally requires recovery reset.`)
- **D004-D4-032**: [E][unit-test] + (`Orchestrator test with fire count 2 and three distinct event kinds must emit both concurrent-fire and auto-revert events.`)
- **D004-D4-033**: [E][integration-test] Δ (`A 100-action stream injects regression at action 60; at least 3/5 tripwires and auto-revert must occur by action 65, and the latest template must be Retired for Degradation.`)
- **D004-D4-034**: [E][decision] + (`The decision record must justify DDM as an orthogonal, low-false-alarm concept-drift signal with published defaults.`)
- **D004-D4-035**: [E][decision] + (`The decision record must justify fire-count 2 using the cited 8%-zero-fire / roughly-80%-two-or-more distribution and the false-positive/miss tradeoff.`)
- **D004-D4-036**: [E][compliance] + (`The decision record must justify retire-not-delete because D-1 requires append-only lineage.`)
- **D004-D4-037**: [E][observability] Δ (`Synthetic 80%-failure injection expects at least three distinct TripwireFired rows, at least one concurrent row, at least one auto-revert row, and a degradation-retired archive entry.`)
- **D004-D4-038**: [E][artifact] + (`Evidence artifacts are workflow/reasoning/D-4-decisions.md and config defaults matching cited values; the source calls the latter tripwire.toml although the implementation surface is elai_cc.toml.`)
- **D004-D4-039**: [E][done-gate] + (`Deterministic gates inspect schema version 50, all three SSB variants, core types, all five detector functions, exact four headline thresholds, workspace build, scoped orchestrator/contracts tests, and all three paper interfaces.`)
- **D004-D4-040**: [E][acceptance] Δ (`Done additionally requires SCHEMA_VERSION_PUBLIC=50, payload round trips, all detector/state/config/integration behaviors, zero new workspace errors, at least 15 new orchestrator tripwire tests, contracts tests, decision doc, and project_d4_tripwire_stack.md from memory-writer.`)
- **D004-D4-041**: [E][fallback] + (`Cloud regression proof uses elai archive verify after D-8; before D-8 it falls back to direct inspection of reasoning-templates.jsonl.`)
- **D004-D4-042**: [E][negative-test] + (`A clean 50-action cloud session must emit zero TripwireFiredConcurrent rows.`)
- **D004-D4-043**: [E][surface] + (`Change set: add tripwire.rs, tripwire integration/contract tests, decision doc, and three paper interfaces; modify ssb.rs, task_runner.rs, elai_cc.toml, and orchestrator lib.rs.`)
- **D004-D4-044**: [E][non-goal] + (`D-5 BAI algorithm, D-8 archive-verifier CLI, D-9 enforcement of the documented lock, and D-26 dashboard rendering are explicitly out of scope.`)
- **D004-D4-045**: [E][dependency] + (`Next consumers are D-5, which consumes consensus passes plus fires, and D-6, whose held-out distribution feeds KL/Δρ baselines.`)
- **D004-D4-046**: [C][ordering] ~ (`Task prose lists retire before TripwireAutoRevert emission, while the stub ban mandates emitting TripwireAutoRevert before retire. The latter is the explicit failure-audit rule and the implementation order must be reconciled.`)
- **D004-D4-047**: [A][state] + (`A Drift→Normal transition condition is never defined even though reset on that transition/recovery is required.`)
- **D004-D4-048**: [A][failure] + (`No missing-signal/NaN/short-window policy, quartile/stddev convention, KL epsilon/smoothing rule, or minimum detector sample count is specified.`)
- **D004-D4-049**: [A][idempotency] + (`No cooldown, deduplication, or repeated-window rule prevents the same concurrence from emitting/reverting repeatedly.`)
- **D004-D4-050**: [A][recovery] + (`The source records intent before retire failure but does not define retry, rollback, parent absence, archive-lock poisoning, or orchestrator switch behavior when retire fails.`)
- **D004-D4-051**: [A][schema] + (`“tripwire.toml” in expected_artifact conflicts nominally with the only configured file, .elai_cc/elai_cc.toml [tripwire].`)
- **D004-D4-052**: [C][signal] + (`The prerequisite calls D-3 VerifierDisagreement an input, while the partial-foundation/wiring text instead identifies VerificationResult proxy reward and VerifierConsensusPass count as the correlation source; the exact D-3 signal is not stable within the source.`)

### D-7: cross domain canary

- **Status**: PLANNED — deps D-6; Phase 4H.VIII.2 / cross-family regression block
- **Superseded:** SUP-008 — D-7 canary block rule. Target: Unresolved: objective non-target-only vs implementation ANY-family below threshold. Loser: Neither deleted; both retained.
- **Anchors**: plan `D-7_cross_domain_canary.md`; DEEP-D-001..004; aggregate unit (44 declared source atoms); compiled 44
- **Composes with**: D-001/IX-001 — **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4  | D-001/IX-002 — **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 p | D-001/IX-006 — **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.

**Behaviors**

- **D004-D7-001**: [E][capability] = (`Evaluate every L2 mutation across code-fix, math-proof, prose-reason, planning, and knowledge-QA; intended headline gate blocks when a non-target family regresses by more than 5%.`)
- **D004-D7-002**: [E][status] + (`Estimated effort is one to two sessions.`)
- **D004-D7-003**: [E][dependency] = (`D-6 is the prerequisite because its held-out store supplies per-family tags.`)
- **D004-D7-004**: [E][concurrency] + (`It is declared parallel-safe with D-5 and D-8 but not D-4 because D-4 reads canary signals as a Δρ source.`)
- **D004-D7-005**: [E][dataset] Δ (`code-fix means Rust/Python compile-and-test fixtures from Exercism and public LeetCode.`)
- **D004-D7-006**: [E][dataset] Δ (`math-proof means SymPy-checkable identities and step proofs from the PRMBench math subset.`)
- **D004-D7-007**: [E][dataset] Δ (`prose-reason means natural-language inference/reasoning from HotpotQA and AdversarialQA public splits.`)
- **D004-D7-008**: [E][dataset] Δ (`planning means multi-step decomposition/dependency order from PlanBench and AlfWorld public data.`)
- **D004-D7-009**: [E][dataset] Δ (`knowledge-QA means factual recall plus grounded citation from PopQA, TriviaQA, and FActScore.`)
- **D004-D7-010**: [E][lifecycle] + (`Public HuggingFace fixtures are bootstrapped and tagged into D-6 HeldOutStore on first ingest.`)
- **D004-D7-011**: [E][dependency] + (`Existing contracts are HeldOutStore.query(domain), the D-1 Domain enum, and D-5’s single gate; D-7 produces per-family CrossDomainRegression rows for BAI.`)
- **D004-D7-012**: [E][dependency] + (`paper-ingestor must produce workflow/papers/nanoflux-2509.23252-interface.md.`)
- **D004-D7-013**: [E][schema/lifecycle] Δ (`Domain extends from Code/Math/Prose/Mixed to add Planning and KnowledgeQa; all matchers become exhaustive, old Mixed entries remain valid, and no disk migration is needed.`)
- **D004-D7-014**: [E][schema] Δ (`Closed, L3-locked CanaryFamily is CodeFix, MathProof, ProseReason, Planning, KnowledgeQa, mapped to Domain with Code→CodeFix and Math→MathProof called out.`)
- **D004-D7-015**: [E][schema] + (`FamilyEvalResult carries family, pass_count, total_count, and pass_rate.`)
- **D004-D7-016**: [E][schema] + (`CanaryReport carries template_id, baseline and candidate vectors, per-family deltas, target_family, and CanaryDecision.`)
- **D004-D7-017**: [E][state] Δ (`CanaryDecision is Allow, BlockRegression{family,delta}, or InsufficientData{family}.`)
- **D004-D7-018**: [E][schema] + (`CanaryRunner owns HeldOutStore, Evaluator, regression_threshold, and min_samples_per_family.`)
- **D004-D7-019**: [E][workflow] Δ (`run queries every family, evaluates baseline and candidate templates on the held-out tasks, computes pass rates and deltas, then returns the report.`)
- **D004-D7-020**: [E][decision] Δ (`Implementation text says BlockRegression if any family delta is below negative threshold and InsufficientData if any family sample count is below the minimum.`)
- **D004-D7-021**: [E][dataset] + (`Bootstrap prescription is code-fix 10 Exercism+5 LeetCode; math 10 PRMBench; prose 10 HotpotQA+5 AdversarialQA; planning 10 PlanBench; knowledge 10 PopQA+5 TriviaQA.`)
- **D004-D7-022**: [E][schema/compliance] + (`Every fixture must populate ExpectedOutcome, set domain correctly, and cite benchmark plus version in source.`)
- **D004-D7-023**: [E][event] Δ (`SCHEMA_VERSION moves 52→53; CrossDomainRegression records template/family/baseline/candidate/delta/threshold/audit_id, CanaryFamilyPass records pass rate/sample count, and CanaryInsufficientData records sample/required counts; all round-trip.`)
- **D004-D7-024**: [E][config/security] Δ (`L3-locked [canary] defaults are regression_threshold=0.05, min_samples_per_family=5, and all five enabled family snake-case names; the threshold must load from config.`)
- **D004-D7-025**: [E][workflow] Δ (`BaiPromotionEngine.promote_round runs the canary before every promotion; regression skips promotion and emits SSB, comparing current Promoted target-family template with the waiting Candidate.`)
- **D004-D7-026**: [E][unit-test] + (`Tests cover pass-rate division, at least ten decision scenarios, family/domain mapping, and thresholds where −0.06 blocks while −0.04 does not at 0.05.`)
- **D004-D7-027**: [E][integration-test] Δ (`Mock 50-task scenarios require Allow for +10% code/no other loss, BlockRegression for +10% code/−8% prose, and InsufficientData for only three planning tasks.`)
- **D004-D7-028**: [E][decision] + (`Decision doc must justify five families as the asserted 3-too-few/12-too-many compromise, 5% as noise-vs-change balance, and minimum five as a floor that auto-defers BAI until data accrues.`)
- **D004-D7-029**: [E][failure] + (`Sparse data must not panic or allow promotion: it returns InsufficientData, emits its SSB row, and defers promotion.`)
- **D004-D7-030**: [E][determinism] + (`Existing Domain variants must not be renamed.`)
- **D004-D7-031**: [E][observability] Δ (`The synthetic +10% code/−8% prose case expects one prose_reason CrossDomainRegression at −0.08, four family-pass rows, no TemplatePromoted, unchanged Candidate status, and a persisted report under workflow/sessions/{id}/canary-reports/.`)
- **D004-D7-032**: [E][done-gate] + (`Deterministic checks inspect schema 53, three variants, core types, Planning/KnowledgeQa arms, config, seed fixtures, BAI wiring, workspace/tests, and NanoFlux interface.`)
- **D004-D7-033**: [E][acceptance] Δ (`Done requires public schema 53, event round trips, mapping/enum extension, at least ten decision cases, fixture seeding, BAI block integration, loaded defaults, zero new errors, and at least ten new canary tests.`)
- **D004-D7-034**: [E][cloud-test] Δ (`Cloud A expects +10% code/−8% prose to block; B expects +10% code/+5% math/neutral others to emit five family-pass rows and TemplatePromoted; C expects sparse data to emit CanaryInsufficientData.`)
- **D004-D7-035**: [E][surface] + (`Change set adds canary.rs, at least 50 dated seed TOMLs, integration test, decision doc, and paper interface; modifies Domain, SSB, BAI, config, and lib.rs.`)
- **D004-D7-036**: [E][non-goal] + (`D-24 validation, D-15 novelty, and D-18 observable effects are out of scope; D-8 later surfaces the lineage and D-24 benchmarks 100 mutations.`)
- **D004-D7-037**: [C][decision] ~ (`The objective says only non-target regressions block, but CanaryRunner says ANY family delta below threshold blocks. The target-family rule must be corrected or intentionally broadened.`)
- **D004-D7-038**: [C][schema] + (`The plan calls CanaryFamily↔Domain a bijection, but Domain contains Mixed and six variants after extension while CanaryFamily has five; no mapping for Mixed is specified.`)
- **D004-D7-039**: [C][dataset] + (`The explicit bootstrap recipe totals 65 tasks, while integration/acceptance/files repeatedly say 50 or at least 50. “At least ten per family” is compatible with 65 but the canonical fixture count is not unique.`)
- **D004-D7-040**: [A][decision] + (`Exact −0.05 boundary behavior is untested, and precedence is undefined when one family regresses while another has insufficient data.`)
- **D004-D7-041**: [A][determinism] + (`CanaryDecision can store only one offending family; selection order/tie-breaking for multiple regressions or sparse families is unspecified.`)
- **D004-D7-042**: [A][failure] + (`Evaluator errors, held-out query failures, partial family completion, retries, cancellation, and concurrent promotion races have no contract.`)
- **D004-D7-043**: [A][artifact] + (`Persisting CanaryReport is required only by observable_effect; no serialization schema, write ordering, or file change is specified for canary-reports.`)
- **D004-D7-044**: [A][concurrency] + (`“Parallel-safe with D-5” conflicts operationally with modifying bai.rs and requiring BaiPromotionEngine to exist; branch/landing order is not explained.`)

### D-11: constructive checker registry

- **Status**: PLANNED — deps D-10; RPS.II / Mode-A checkers
- **Anchors**: plan `D-11_constructive_checker_registry.md`; DEEP-D-001..004; aggregate unit (61 declared source atoms); compiled 61
- **Composes with**: D-001/IX-004 — **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lac

**Behaviors**

- **D004-D11-001**: [E][capability] = (`Bootstrap .elai_cc/checkers/{math,code,text,graph,api,perf}, six initial typed checkers, CheckerResult, and orchestrator dispatch so Mode A constructive proof becomes operational and external checkers—not LLM confidence—are ground truth.`)
- **D004-D11-002**: [E][status] + (`Estimated effort is three to four sessions.`)
- **D004-D11-003**: [E][dependency] = (`D-10 mode routing must be DONE; D-12 is declared parallel-safe and consumes the registry as its target.`)
- **D004-D11-004**: [E][surface] Δ (`Full registry blueprint includes math/{sympy,z3,lean}, code/{compile/test for Rust/Python/TS}, text/{regex_assertion,schema_validate}, graph/networkx, api/registry_check, and perf/{timing_harness,asymptotic_curve_fit}.`)
- **D004-D11-005**: [E][contract] Δ (`Conceptual checker contract is deterministic Claim+Candidate+SandboxHandle input and Pass, Fail with evidence, or Inapplicable with reason output.`)
- **D004-D11-006**: [E][scope] Δ (`The initial six are sympy, z3, compile_rust, compile_py, registry_check, and regex_assertion; Lean, TypeScript, performance, schema validation, and networkx are deferred.`)
- **D004-D11-007**: [E][security] + (`Tier-C sandbox from B-18/C-11 supplies SandboxHandle; the existing Python sidecar scaffold is reusable and D-10 already has the constructive dispatch hook.`)
- **D004-D11-008**: [E][dependency] + (`Before testing, paper-ingestor must produce PoT, CodeAct, and AlphaProof interface files with the exact slugs pot-2211.12588, codeact-2402.01030, alphaproof-nature-2025.`)
- **D004-D11-009**: [E][schema] Δ (`CheckerResult is Pass, Fail{evidence: Vec<BoundedString<2048>>}, or Inapplicable{reason:String}.`)
- **D004-D11-010**: [E][schema] + (`CheckerEntry fields are name, claim_types, domain, executable_path, CheckerLanguage, and timeout_secs; CheckerLanguage is closed Python/Shell/Rust/Js.`)
- **D004-D11-011**: [E][schema] + (`CheckerRegistry stores a Vec<CheckerEntry>.`)
- **D004-D11-012**: [E][lifecycle] + (`CheckerRegistry.load walks root/*/*.toml registration files, one CheckerEntry per TOML.`)
- **D004-D11-013**: [E][dispatch] Δ (`applicable requires both claim_type membership and domain match.`)
- **D004-D11-014**: [E][dispatch] Δ (`run chooses the first applicable checker, sandbox-executes it, parses stdout JSON, and returns its CheckerResult.`)
- **D004-D11-015**: [E][fallback] + (`Checker timeout returns Inapplicable with reason “timeout”; malformed checker JSON also returns Inapplicable rather than panicking.`)
- **D004-D11-016**: [E][config] + (`Registration TOML names the checker, accepted claim types, domain, executable path, language, and timeout; the canonical example is SymPy/math/Python/30 seconds.`)
- **D004-D11-017**: [E][checker] Δ (`sympy.py reads a math identity JSON request, parses LHS/RHS, simplifies the difference, passes at zero, fails with the simplified difference, and is Inapplicable on parse error.`)
- **D004-D11-018**: [E][process] + (`sympy.py emits JSON; Pass, Fail, and Inapplicable exit 0, while only script-level failure exits 1.`)
- **D004-D11-019**: [E][supply-chain] + (`SymPy must be present and version-pinned in pyproject.toml or its install requirement documented under the anti-stale rule.`)
- **D004-D11-020**: [E][checker] Δ (`z3.py translates finite-domain SAT/SMT claims, proves by unsatisfiability of the negation, and returns a counterexample on Fail; z3-solver must be declared.`)
- **D004-D11-021**: [E][checker/security] Δ (`compile_rust.sh writes candidate source to a temporary lib.rs and runs cargo check inside Tier C; exit 0 passes and stderr evidence accompanies failure.`)
- **D004-D11-022**: [E][checker/security] Δ (`compile_py.sh analogously runs python -m py_compile plus ruff check in the checker path.`)
- **D004-D11-023**: [E][checker/integration] Δ (`registry_check.py accepts package+registry JSON and performs live npm, PyPI, or crates.io lookup; npm checks version/dist-tags/time, PyPI version/license/recent release, and crates.io its API.`)
- **D004-D11-024**: [E][checker/failure] + (`Registry lookup passes only for a hit with a release in the last 24 months, fails “package not found” or “stale”, and is Inapplicable for an unknown registry.`)
- **D004-D11-025**: [E][checker] Δ (`regex_assertion.py accepts text, regex, and full-or-contains mode; it passes on match, fails with mismatch position, and is Inapplicable for invalid regex.`)
- **D004-D11-026**: [E][workflow/event] Δ (`D-10’s Constructive branch calls CheckerRegistry.run and emits CheckerDispatched plus exactly one CheckerPass, CheckerFail, or CheckerInapplicable result row.`)
- **D004-D11-027**: [E][event] Δ (`SCHEMA_VERSION moves 56→57; CheckerDispatched carries claim/checker/audit, CheckerPass adds latency_ms, CheckerFail carries evidence, and CheckerInapplicable carries reason; all round-trip.`)
- **D004-D11-028**: [E][unit-test] + (`Registry tests require math→sympy+z3 applicability, API→registry_check, mocked pass parsing, and timeout→Inapplicable.`)
- **D004-D11-029**: [E][integration-test] Δ (`Integration cases are true and false SymPy identities, invalid Rust compile failure, live npm “react” pass, and fake malicious package failure.`)
- **D004-D11-030**: [E][decision] + (`Decision record chooses Python for mature SymPy/Z3 bindings.`)
- **D004-D11-031**: [E][isolation] + (`Decision record chooses subprocess JSON I/O over PyO3 to isolate Python state and apply Tier-C constraints uniformly.`)
- **D004-D11-032**: [E][security] + (`Decision record makes live existence checking an anti-slopsquatting gate before an agent imports unknown dependencies.`)
- **D004-D11-033**: [E][integration] + (`Registry consumes GroundingMode::Constructive, Claim/ClaimType, Domain, and Tier C; it is reachable through mode dispatch and optional registration paths in elai_cc.toml [checkers].`)
- **D004-D11-034**: [E][timeout] + (`Every run must honor CheckerEntry.timeout_secs; infinite waits are forbidden.`)
- **D004-D11-035**: [E][integration] + (`Production registry_check must hit the live registry; a cached mock is forbidden in production.`)
- **D004-D11-036**: [E][failure] + (`An empty registry returns an empty Vec without panic.`)
- **D004-D11-037**: [E][observability] Δ (`Six synthetic claims expect six dispatch rows plus six terminal result rows, six executable scripts in the registry, and parseable JSON stdout from each.`)
- **D004-D11-038**: [E][done-gate] + (`Deterministic checks inspect schema 57, four SSB variants, registry types, six files, at least six TOMLs, build/tests, direct SymPy JSON round-trip, live registry smoke, and three paper interfaces.`)
- **D004-D11-039**: [E][acceptance] Δ (`Done requires public schema 57, four event round trips, registry unit tests, executable scripts, valid registrations, per-checker positive/negative behavior, zero new errors, and at least ten new checker tests.`)
- **D004-D11-040**: [E][cloud-test] Δ (`Cloud math must route Mode A→SymPy→CheckerPass; fake npm install must be intercepted, emit CheckerFail, and block installation.`)
- **D004-D11-041**: [E][surface] + (`Change set adds checker_registry.rs, six script+TOML pairs, dispatch integration test, decision doc, and three paper interfaces; modifies SSB, grounding_mode, lib.rs, and Python dependency declarations.`)
- **D004-D11-042**: [E][non-goal] Δ (`Lean, TypeScript check/test, schema_validate, networkx, timing_harness, and multi-checker voting are deferred; v1 dispatch remains first-applicable single checker.`)
- **D004-D11-043**: [E][dependency] + (`D-12 next requires every claim-emitting baby step to reference an applicable checker.`)
- **D004-D11-044**: [E][amendment/contract] + (`J-11 imports a minimal verifier contract: self-contained verifier script, candidate on stdin, deterministic expected stdout, and passed/score/info/metadata output; info transports status/error/traceback evidence with no LLM in the verdict path.`)
- **D004-D11-045**: [E][amendment/lifecycle] + (`At registration time, every checker must run its bundled reference fixture; a failure rejects the checker and fails closed before runtime admission.`)
- **D004-D11-046**: [E][amendment/decision] + (`Registry verdict composition must support a demotion clause: CheckerResult::Fail caps the overall verdict rather than being averaged with other scores; construction failure≤0.3 is the given example.`)
- **D004-D11-047**: [E][amendment/security] + (`ComBench reliability_guard must not be copied as a sandbox; generated code must execute in ELAI Tier C using seccomp, SO_PEERCRED, and capability drop because the Python guard does not prevent network, out-of-limit writes, or process spawn.`)
- **D004-D11-048**: [E][amendment/retry] + (`J-15 makes every first checker Fail deterministically trigger exactly one structured reformulation prompt containing checker X and reason Y, then re-run the same checker.`)
- **D004-D11-049**: [E][amendment/state] + (`If the one reformulation passes, claim state becomes Verified; a second Fail makes it Ungrounded.`)
- **D004-D11-050**: [E][amendment/determinism] + (`No model chooses retry, there is no unbounded loop, the error is the sole feedback, the checker is the sole verdict, and the one-pass bound is explicitly anti-gaming.`)
- **D004-D11-051**: [E][amendment/composition] + (`The demotion clause remains in force: reformulation applies to the failed checker’s claim and does not remove the cap on overall verdict calculation.`)
- **D004-D11-052**: [C][schema] + (`J-15 calls error text the info field “from CheckerResult,” but the declared CheckerResult has evidence on Fail and reason on Inapplicable, no info field.`)
- **D004-D11-053**: [A][determinism] + (`“First applicable” has no stable ordering/priority rule; filesystem TOML walk order and entries ordering can change the selected checker.`)
- **D004-D11-054**: [A][registration] + (`The amendment requires a bundled reference fixture, but CheckerEntry/TOML has no fixture path, expected result, registration state, or rejection evidence schema.`)
- **D004-D11-055**: [A][decision] + (`Demotion cap is mandatory but its value, per-checker configuration, verdict type, aggregation integration point, and event/evidence schema are absent; 0.3 is introduced only as an example.`)
- **D004-D11-056**: [A][state] + (`“Pass after reformulation→Verified” and “original construction failure still caps overall verdict” leave the final claim/overall-verdict relationship unresolved.`)
- **D004-D11-057**: [A][parsing] + (`Natural-language-to-Z3 translation grammar and rejection policy are unspecified, so deterministic soundness cannot be established from the plan.`)
- **D004-D11-058**: [A][failure] + (`Invalid TOML, duplicate names, missing/non-executable paths, subprocess exit 1, oversized output/evidence, temporary-file cleanup, and checker crash handling have no contract.`)
- **D004-D11-059**: [A][security/privacy] + (`Live registry access has no URL/package escaping, allowlist, timeout/retry, rate-limit, provenance, privacy, or network-policy contract; Tier-C network permissions are not reconciled with mandatory live lookup.`)
- **D004-D11-060**: [A][amendment/test] + (`Both amendments postdate the original acceptance/files list but add no updated unit/integration/cloud tests, SSB variants, schema bump, or touched files for registration rejection, demotion, or reformulation.`)
- **D004-D11-061**: [A][concurrency] + (`Registry reload/mutation, concurrent checker runs, subprocess cancellation, and idempotency of repeated dispatch/reformulation are unspecified.`)

### D-15: novelty class labeling

- **Status**: PLANNED — deps D-11, D-12, C-17; RPS.VI / novelty escalation
- **Superseded:** SUP-007 — D-15 NoveltyClass attachment target. Target: ClaimResult metadata (D-15 original). Loser: Summary: field on CheckerResult.
- **Anchors**: plan `D-15_novelty_class_labeling.md`; DEEP-D-001..004; aggregate unit (38 declared source atoms); compiled 38
- **Composes with**: D-001/IX-004 — **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lac

**Behaviors**

- **D004-D15-001**: [E][capability] = (`Every Mode A or Mode D result receives Search, DistantSearch, or DiscoveryCandidate; candidates route to human review and the system never autonomously claims discovery.`)
- **D004-D15-002**: [E][status] + (`Estimated effort is one to two sessions.`)
- **D004-D15-003**: [E][dependency] = (`Prerequisites are D-11 results, D-12 procedures, and C-17 DKG resilience; spec-grounder verifies them, and D-14 is declared parallel-safe.`)
- **D004-D15-004**: [E][policy] + (`The engineering stance permits checker-scored novel-output search but forbids autonomous discovery marketing and distinguishes checker verification from uncheckable search output.`)
- **D004-D15-005**: [E][classification] Δ (`Search means a near-duplicate in DKG, MistakeBook, or prior session archive.`)
- **D004-D15-006**: [E][classification] Δ (`DistantSearch means a novel composition whose referenced primitives are all known to the DKG.`)
- **D004-D15-007**: [E][classification] Δ (`DiscoveryCandidate means at least one referenced primitive or relationship is absent from the DKG and therefore requires human review.`)
- **D004-D15-008**: [E][dependency] + (`The classifier reuses C-17 DKG/embedding resilience, C-24 symbol_impact, MistakeBook, ArchiveChain, and D-11 CheckerResult.`)
- **D004-D15-009**: [E][dependency] + (`Before testing, paper-ingestor must produce FunSearch, AlphaEvolve, and Chollet ARC interface files.`)
- **D004-D15-010**: [E][schema] = (`NoveltyClass is a closed, L3-locked enum with Search, DistantSearch, DiscoveryCandidate.`)
- **D004-D15-011**: [E][schema] Δ (`NoveltyClassification carries class, DKG match count, near-duplicate boolean, known/missing primitive counts, and evidence bounded to 2048 characters.`)
- **D004-D15-012**: [E][algorithm] Δ (`classify first checks cosine similarity≥0.85 in DKG OR MistakeBook OR archive; otherwise all-known primitives produce DistantSearch, and any unknown primitive produces DiscoveryCandidate.`)
- **D004-D15-013**: [E][workflow] Δ (`After Mode A Constructive or Mode D OperationalFalsification produces CheckerResult, classify it, attach NoveltyClass to ClaimResult metadata, and emit NoveltyLabeled.`)
- **D004-D15-014**: [E][workflow/artifact] Δ (`DiscoveryCandidate writes workflow/sessions/{id}/discovery-queue/{claim_id}.toml and emits DiscoveryCandidateQueued; a queue file is mandatory, not SSB-only.`)
- **D004-D15-015**: [E][schema/lifecycle] + (`DiscoveryQueueEntry stores claim_id, UTC classification time, Blake3 result hash, bounded evidence, and ReviewStatus.`)
- **D004-D15-016**: [E][state] Δ (`ReviewStatus transitions from Pending to RatifiedHuman{reviewer,ratified_at} or RejectedHuman{reviewer,reason}.`)
- **D004-D15-017**: [E][CLI/deferred] Δ (`elai discovery review v1 only prints pending entries; an interactive TUI is deferred.`)
- **D004-D15-018**: [E][event] Δ (`SCHEMA_VERSION moves 60→61 and adds NoveltyLabeled, DiscoveryCandidateQueued, DiscoveryRatified, and DiscoveryRejected with the listed label/count/path/reviewer/reason payloads.`)
- **D004-D15-019**: [E][integration/fallback] + (`near_duplicate_check uses the existing knowledge embedding pipeline and top-k DKG lookup at default 0.85; if the crate cannot be called directly, the plan permits a documented stub interface, but forbids reinventing embeddings.`)
- **D004-D15-020**: [E][unit-test] + (`Unit cases require synthetic-embedding Search, known-primitive DistantSearch, unknown-primitive DiscoveryCandidate, and correct threshold behavior.`)
- **D004-D15-021**: [E][integration-test] Δ (`A mocked three-symbol/three-embedding DKG must exercise all classes, with the novel-primitive case creating a queue entry.`)
- **D004-D15-022**: [E][decision] + (`The decision record justifies three buckets as coarse but sufficient human routing, 0.85 as a false-positive/false-negative balance, and human ratification as the no-auto-discovery safety rule.`)
- **D004-D15-023**: [E][config] + (`Reachability names elai_cc.toml [novelty] for cosine threshold and queue path, but no concrete section is otherwise defined.`)
- **D004-D15-024**: [E][failure] + (`Empty DKG must yield DiscoveryCandidate when no references are known, never panic.`)
- **D004-D15-025**: [E][observability] Δ (`Ten mixed-novelty synthetic claims must yield ten NoveltyLabeled rows and queue TOMLs for every DiscoveryCandidate.`)
- **D004-D15-026**: [E][done-gate] + (`Deterministic checks inspect schema 61, four SSB identifiers, enum/classifier/helper, workspace/test results, and all three paper interfaces.`)
- **D004-D15-027**: [E][acceptance] Δ (`Done requires public schema 61, four event round trips, three-bucket and threshold tests, queue writes, zero new errors, at least eight novelty tests, three cloud cases, paper interfaces, and the decision doc.`)
- **D004-D15-028**: [E][surface] + (`Change set adds novelty.rs, integration test, decision doc, and three paper interfaces; modifies SSB, grounding_mode, and lib.rs.`)
- **D004-D15-029**: [E][non-goal] Δ (`Review TUI, multi-reviewer ratification, and N-reviewer auto-publish are deferred; v1 is single-reviewer and conservatively does not auto-publish.`)
- **D004-D15-030**: [E][dependency] + (`D-16 conformal Mode-E refusal is next.`)
- **D004-D15-031**: [C][schema] ~ (`The summary says NoveltyClass becomes a field on CheckerResult; the original instead attaches it to ClaimResult metadata.`)
- **D004-D15-032**: [C][interface] + (`classify accepts CheckerResult, but D-11 CheckerResult contains only verdict/evidence/reason and no candidate text, embedding, primitive list, or relationships needed by the heuristic.`)
- **D004-D15-033**: [C][policy/schema] + (`Context requires marking novel claims with confidence, but neither NoveltyClassification nor queue/event payloads contains a confidence value.`)
- **D004-D15-034**: [A][lifecycle] + (`v1 “prints list,” yet ratified/rejected states and events require a mutation flow; no command syntax, persistence transaction, reviewer authentication, or release/publication rule is defined.`)
- **D004-D15-035**: [A][config] + (`[novelty] is referenced but has no defaults, lock status, loader task, file modification, or acceptance assertion.`)
- **D004-D15-036**: [A][algorithm] + (`Primitive extraction, relationship matching, top-k choice, embedding model/version, treatment of empty result content, and exact 0.85 boundary are unspecified.`)
- **D004-D15-037**: [A][lifecycle/security] + (`Queue creation has no atomic-write/idempotency/hash-verification, duplicate claim, retention, permissions, reviewer identity, privacy, or tamper policy.`)
- **D004-D15-038**: [A][consistency] + (`The helper signature searches only SymbolGraph, while the classification requirement also searches MistakeBook and ArchiveChain; adapters and similarity semantics for those stores are absent.`)

### D-18: observable effect typed

- **Status**: PLANNED — deps C-22, D-11; Phase 4F extension / Mode-D effects
- **Anchors**: plan `D-18_observable_effect_typed.md`; DEEP-D-001..004; aggregate unit (44 declared source atoms); compiled 44
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-18

**Behaviors**

- **D004-D18-001**: [E][capability] = (`Extend C-22 FeasibilityProber with typed ObservableEffect, require at least two orthogonal effects per claim, and distinguish sandbox from production observations.`)
- **D004-D18-002**: [E][status] + (`Estimated effort is two sessions.`)
- **D004-D18-003**: [E][dependency] = (`Prerequisites are completed C-22 FeasibilityProber and D-11 effect-verifying checkers; it is declared parallel-safe with D-17.`)
- **D004-D18-004**: [E][safety] Δ (`Goodhart defense requires at least two effects measuring orthogonal aspects.`)
- **D004-D18-005**: [E][safety] Δ (`SandboxObservation is explicitly not ProductionObservation; production grounding must never be claimed from sandbox evidence alone.`)
- **D004-D18-006**: [E][observability] Δ (`Confounded experiments must log environmental variables and surface known risks through confound_flags.`)
- **D004-D18-007**: [E][dependency] + (`Reused substrate is C-22 prober/tests/UI, D-11 CheckerRegistry, and B-18/C-11 Tier-C sandbox.`)
- **D004-D18-008**: [E][dependency] + (`Before testing, paper-ingestor must produce Voyager, Eureka, and Coscientist interface files.`)
- **D004-D18-009**: [E][schema] Δ (`ObservableEffect fields are TriggerSpec trigger, TypedObservation expected_observation, Predicate success_predicate, Duration bounded_runtime, SandboxTier, and Vec<ConfoundFlag>.`)
- **D004-D18-010**: [E][schema] + (`TriggerSpec holds TriggerKind plus a payload bounded to 4096; TriggerKind is Subprocess, HttpRequest, FileWrite, or SandboxedRun.`)
- **D004-D18-011**: [E][schema] + (`TypedObservation is StringMatch<2048>, FileContent{path<512>, optional expected Blake3}, ExitCode, or MetricValue{name, inclusive/unspecified range tuple}.`)
- **D004-D18-012**: [E][schema] + (`SandboxTier is closed TierA/TierB/TierC matching B-18/C-11.`)
- **D004-D18-013**: [E][schema] Δ (`ConfoundFlag is ColdCache, ColdNetwork, GpuRecentlyIdle, ConcurrentTasks, or Other(String).`)
- **D004-D18-014**: [E][schema] Δ (`EffectSet carries effects, claim_id, and target_orthogonality, whose stated default is at least two.`)
- **D004-D18-015**: [E][validation] Δ (`validate_orthogonality rejects fewer effects than target and requires effects to differ on trigger kind or observation variant; same-kind effects count only once.`)
- **D004-D18-016**: [E][schema] Δ (`ObservationContext is Sandbox(SandboxTier) or Production.`)
- **D004-D18-017**: [E][schema] Δ (`ObservationResult embeds the effect, context, outcome, timing_ms, and EnvironmentSnapshot.`)
- **D004-D18-018**: [E][state] + (`ObservationOutcome is Confirmed, Refuted with evidence bounded to 2048, or Inconclusive with reason.`)
- **D004-D18-019**: [E][schema] Δ (`EnvironmentSnapshot carries hostname, hardware_class, and active_confounds.`)
- **D004-D18-020**: [E][permission] Δ (`Every sandbox observation preserves its tier, and promotion from sandbox evidence to a production claim requires explicit operator ratification; automatic promotion is forbidden.`)
- **D004-D18-021**: [E][integration] Δ (`DefaultFeasibilityProber.probe changes from one result to Vec<ObservationResult>; probe construction includes EffectSet and validates orthogonality at author time.`)
- **D004-D18-022**: [E][failure/event] Δ (`check_goodhart_risk returns a flag with reason “single-effect probe vulnerable to Goodhart” when effect count is below target.`)
- **D004-D18-023**: [E][event] Δ (`SCHEMA_VERSION moves 63→64; ObservableEffectDispatched records count/target/context, GoodhartRiskFlag records count/reason, SandboxObservation records effect ID/outcome/tier, and ProductionObservationProposal records sandbox evidence/operator_required; all round-trip.`)
- **D004-D18-024**: [E][config/security] Δ (`L3-locked [observable_effect] defaults are min_effects_per_claim=2, target_orthogonality=2, and auto_promote_sandbox_to_production=false, with the false default singled out as safety-critical.`)
- **D004-D18-025**: [E][unit-test] + (`Validation tests require one effect and two same-kind effects to error, two different-kind effects to pass, insufficient effects to flag Goodhart, and sandbox context to survive result construction.`)
- **D004-D18-026**: [E][integration-test] Δ (`Integration runs a one-effect flag case, a two-orthogonal-effect dispatch with two sandbox rows, and a sandbox success that emits no production claim.`)
- **D004-D18-027**: [E][decision] + (`Decision record justifies two effects as anti-Goodhart, explicit context as sim-to-real honesty, and operator-only production claims as protection from proxy-reward optimization.`)
- **D004-D18-028**: [E][integration] Δ (`The feature consumes FeasibilityProber, CheckerRegistry for individual effects, Tier C, and Mode D; it is reached from Operational Falsification and the prober entry point.`)
- **D004-D18-029**: [E][invariant] + (`Stub bans require validation against target_orthogonality, false auto-promotion default, a flag for single-effect probes, and persistent context labels.`)
- **D004-D18-030**: [E][observability/artifact] Δ (`A valid two-effect probe expects one dispatch, two sandbox rows, zero Goodhart and zero production proposal until operator action, plus workflow/sessions/{id}/observations/{claim_id}.toml with environment snapshot/confounds.`)
- **D004-D18-031**: [E][done-gate] + (`Deterministic checks inspect schema 64, four event names, core types/functions, config and false default, workspace/tests, and three paper interfaces.`)
- **D004-D18-032**: [E][acceptance] Δ (`Done requires public schema 64, event round trips, validation/context/flag/config behavior, Vec-return integration, loaded config, zero new errors, at least ten tests, three cloud smoke cases, papers, and decision doc.`)
- **D004-D18-033**: [E][surface] + (`Change set adds observable_effect.rs, integration test, decision doc, and three paper interfaces; modifies feasibility lib, SSB, grounding_mode, elai_cc.toml, and orchestrator lib.rs.`)
- **D004-D18-034**: [E][non-goal] Δ (`Operator production-claim CLI, per-domain effect templates, and automated confound detection are deferred; v1 is sandbox-only, ad-hoc authored, and manually flagged.`)
- **D004-D18-035**: [E][dependency] + (`The plan declares Wave 2 complete and Wave 3 responsible for benchmark/adversarial gates.`)
- **D004-D18-036**: [C][workflow] + (`Author-time validation says a one-effect probe cannot be constructed, yet runtime integration/cloud acceptance requires such a probe to execute and emit GoodhartRiskFlag. The reject-vs-observe path is unresolved.`)
- **D004-D18-037**: [A][algorithm] + (`For target_orthogonality above two, “differ on at least one attribute” and “same-kind count as one” do not define equivalence classes or how many independent groups satisfy the target.`)
- **D004-D18-038**: [A][timeout] + (`bounded_runtime is data only: no runner, timeout, cancellation, retry, or terminal Inconclusive behavior is specified.`)
- **D004-D18-039**: [A][security/privacy] + (`HttpRequest and FileWrite triggers have no host/path allowlist, credential/network policy, capability mapping, redaction, or production authorization contract.`)
- **D004-D18-040**: [C][observability] + (`The source says log all environmental variables, but EnvironmentSnapshot specifies only hostname, hardware class, and active confounds.`)
- **D004-D18-041**: [A][operator-flow] + (`ProductionObservationProposal and operator ratification have no callable interface, reviewer identity/audit transaction, or state transition; the required CLI is explicitly deferred.`)
- **D004-D18-042**: [A][config] + (`min_effects_per_claim and target_orthogonality overlap, while EffectSet also carries a target; precedence and whether lower/higher operator values are legal under L3 lock are unspecified.`)
- **D004-D18-043**: [A][schema] + (`effect_id appears in SSB but not in ObservableEffect, and observation TOML serialization, atomicity, retention, and identifier derivation are unspecified.`)
- **D004-D18-044**: [A][integration] + (`The plan says CheckerRegistry runs effects but gives no adapter from TriggerSpec/TypedObservation/Predicate to D-11 Claim/Candidate checker input/output.`)

### D-20: arc agi2 ood probe

- **Status**: PLANNED — deps D-10, D-15; companion §1.16 + real-problem §1 Mode C / ARC OOD
- **Superseded:** SUP-011 — D-20 synthetic ARC fallback. Target: v1: ContaminationSuspected/exit 1 only; synthetic generation deferred. Loser: Objective promised synthetic fallback if leakage flagged.
- **Anchors**: plan `D-20_arc_agi2_ood_probe.md`; DEEP-D-001..004; aggregate unit (36 declared source atoms); compiled 36
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-20

**Behaviors**

- **D004-D20-001**: [E][capability] = (`Run a 50-task public ARC-AGI-2 sample on cloud Sonnet with mode routing, reporting solve and Mode-E refusal rates against a human baseline while forbidding DiscoveryCandidate auto-publication.`)
- **D004-D20-002**: [E][status] + (`Estimated effort is two to three sessions; local 7B execution is explicitly excluded.`)
- **D004-D20-003**: [E][dependency] = (`Declared prerequisites are D-10 routing and D-15 novelty labeling; D-19 and D-21..D-25 are declared parallel-safe.`)
- **D004-D20-004**: [E][expectation] + (`Expected solve performance is single-digit to low-double-digit; any solve rate passes because the harness, not model capability, is gated.`)
- **D004-D20-005**: [E][acceptance-policy] Δ (`Honest-evaluation gates are D-17 healthy refusal rate, no DiscoveryCandidate auto-publish, and negligible D-14 sycophancy flags.`)
- **D004-D20-006**: [E][dependency] + (`No new paper ingest is required because ARC Prize and Chollet ARC interfaces are attributed to D-6/D-15.`)
- **D004-D20-007**: [E][dataset] Δ (`Pull 50 public ARC-AGI-2 evaluation tasks into .elai_cc/benchmarks/arc_agi2/fixtures and record dataset version plus checksum for drift detection.`)
- **D004-D20-008**: [E][schema] + (`ArcTask fields are task_id, training input/output grid pairs, test inputs, and test outputs.`)
- **D004-D20-009**: [E][schema] Δ (`ARCAGI2Runner owns fixture/output paths and Arc<dyn CloudClient>, explicitly cloud Sonnet only.`)
- **D004-D20-010**: [E][schema] Δ (`ARCReport carries totals/solved/refused, mode and novelty distributions, sycophancy count, solve/refusal rates, human baseline, and ARCVerdict.`)
- **D004-D20-011**: [E][state] Δ (`ARCVerdict is HonestlyEvaluated, RefusalRateOutOfBand, or ContaminationSuspected.`)
- **D004-D20-012**: [E][workflow] Δ (`For each task, construct a Claim, route it through the mode predicate, observe ClaimResult, count solved only when Verified and exact output matches expected, and aggregate distributions.`)
- **D004-D20-013**: [E][CLI] + (`elai bench arc-agi2 runs the harness and emits a report.`)
- **D004-D20-014**: [E][exit/failure] + (`CLI exits 0 only for HonestlyEvaluated regardless of solve rate and exits 1 for refusal out of band or contamination suspected.`)
- **D004-D20-015**: [E][detection] Δ (`Contamination heuristic flags only when solve_rate>50% and “model accessed before 6 months ago”; it is intentionally conservative.`)
- **D004-D20-016**: [E][event/dependency] + (`It adds no schema version and reuses BenchmarkStarted and BenchmarkResult from D-19.`)
- **D004-D20-017**: [E][decision] + (`Decision record justifies cloud-only as a routing-infrastructure test, no solve threshold as honest future-proofing, and 50 rather than 400 tasks for cost.`)
- **D004-D20-018**: [E][integration] Δ (`Runner consumes CloudClient, GroundingMode, NoveltyClass, RefusalDecision, SycophancyFlag, and SsbWriter.`)
- **D004-D20-019**: [E][CI] + (`The CLI is the primary reachability path; CI integration is optional because it is expensive.`)
- **D004-D20-020**: [E][invariant] Δ (`Runner must never auto-publish DiscoveryCandidate, must cite the human baseline and framework caveats, and must use a documented stratified 50-task sample.`)
- **D004-D20-021**: [E][observability] Δ (`A run emits BenchmarkStarted, per-task ModeSelected and ClaimDispatched, then BenchmarkResult with novelty/refusal/sycophancy distributions.`)
- **D004-D20-022**: [E][artifact] Δ (`Dated JSON reports live under .elai_cc/benchmarks/arc_agi2/reports and must contain the honest report with no candidate auto-publication.`)
- **D004-D20-023**: [E][done-gate] + (`Deterministic checks only inspect runner/run symbol, at least 50 JSON fixtures, CLI file presence, and workspace build.`)
- **D004-D20-024**: [E][acceptance] Δ (`Done requires compiled runner and synthetic ArcTask unit tests, 50 fixtures, successful cloud CLI, zero new errors, all arc tests, and full-run report with solve/refusal/mode/novelty metrics.`)
- **D004-D20-025**: [E][cloud-test] Δ (`Healthy refusal band is 5%-50%; auto-publishes must be zero; sycophancy count must be zero “or near-zero.”`)
- **D004-D20-026**: [E][surface] + (`Change set adds runner, CLI module, 50 fixtures, and decision doc, and adds SlashCommand::BenchArcAgi2 to commands.rs.`)
- **D004-D20-027**: [E][non-goal] Δ (`Full 400-task run, ARC-AGI-1 baseline, and synthetic ARC-style generation are deferred.`)
- **D004-D20-028**: [C][fallback] ~ (`The objective promises synthetic ARC-style tasks if leakage is flagged, but the implementation only returns ContaminationSuspected/exit 1 and explicitly defers synthetic generation. There is no fallback in v1.`)
- **D004-D20-029**: [C][gate] + (`The stated pass gates include no auto-publication and sycophancy hygiene, but ARCVerdict/CLI failure states cover only refusal and contamination; neither safety violation can make exit 1 as typed.`)
- **D004-D20-030**: [C][dependency] + (`D-14, D-16, D-17, and D-19 are runtime/event dependencies but are absent from declared prerequisites; D-19 is even called parallel-safe although its event variants are consumed.`)
- **D004-D20-031**: [A][detection] + (`“model accessed before 6 months ago” has no subject, timestamp source, comparison date, or meaning; contamination response has no alternate data/retry path.`)
- **D004-D20-032**: [A][metric] + (`human_baseline is one f32 despite a cited 60%-70% range; its chosen value, source/version in the report, and comparison calculation are unspecified.`)
- **D004-D20-033**: [A][evidence] + (`ARCReport has no auto-publication counter or publication audit field, so the no-publish gate cannot be proved from its declared schema alone.`)
- **D004-D20-034**: [A][acceptance] + (`“near-zero” sycophancy is unquantified and conflicts with the same bullet’s sycophancy_flag_count=0 form.`)
- **D004-D20-035**: [A][reliability/privacy] + (`Cloud timeout/retry/rate-limit/cost cap, partial-run resume, deterministic sampling/seed, response retention, secrets, and fixture licensing are unspecified.`)
- **D004-D20-036**: [A][mode] + (`The harness exercises Mode C by implication but does not define its handler; it only routes through the external D-10 predicate.`)

### D-22: conformal calibration validation

- **Status**: PLANNED — deps D-16, D-6; D-16 gating evidence / shifted calibration
- **Superseded:** SUP-012 — D-22 distributions source. Target: Separate 300-file benchmark tree; HeldOutStore not wired. Loser: Summary: distributions sourced from D-6.
- **Anchors**: plan `D-22_conformal_calibration_validation.md`; DEEP-D-001..004; aggregate unit (26 declared source atoms); compiled 26
- **Composes with**: see D-0 wave deps and D-00N/IX records citing D-22

**Behaviors**

- **D004-D22-001**: [E][capability] = (`Validate that D-16 empirical refusal rate remains inside [τ−ε,τ+ε] with ε=0.1 for in-domain, near-OOD, and far-OOD distributions.`)
- **D004-D22-002**: [E][status] + (`Estimated effort is one to two sessions.`)
- **D004-D22-003**: [E][dependency] = (`Prerequisites are D-16 UniCR and D-6 held-out data; it is declared parallel-safe with D-19..D-25.`)
- **D004-D22-004**: [E][dataset] Δ (`In-domain is the calibration family; near-OOD is related but different (code→math example); far-OOD is semantically distant (code→prose/planning example).`)
- **D004-D22-005**: [E][dataset] Δ (`Create in_domain, near_ood, and far_ood directories with 100 hand-labeled claims each, 300 total.`)
- **D004-D22-006**: [E][schema] + (`Fixture labels are claim, ground_truth_available, and hand-labeled, with no further record schema given.`)
- **D004-D22-007**: [E][schema] Δ (`ConformalValidationRunner owns fixtures_dir, tau, and epsilon.`)
- **D004-D22-008**: [E][workflow] Δ (`run calls D-16 refusal_decision on every distribution, computes empirical refusal rate, compares each with [τ−ε,τ+ε], and reports per-distribution shifts.`)
- **D004-D22-009**: [E][CLI] + (`Add elai bench conformal-validate.`)
- **D004-D22-010**: [E][decision] + (`Decision record justifies three tiers from UniCR §6, 100 samples/tier as sufficient for ε=0.1 despite the 500 calibration bound, and ε=0.1 as D-16’s default.`)
- **D004-D22-011**: [E][integration/config] Δ (`Runner consumes refusal_decision, τ from .elai_cc/calibration/tau.txt, and SsbWriter, and is reachable from CLI plus CI.`)
- **D004-D22-012**: [E][failure] Δ (`Every distribution rate must be reported and the verdict must fail if any distribution shifts by more than ε.`)
- **D004-D22-013**: [E][observability] Δ (`CLI run expects BenchmarkStarted, BenchmarkResult, an unnamed per-distribution rate row, and a report in .elai_cc/benchmarks/conformal/reports/.`)
- **D004-D22-014**: [E][done-gate] + (`Deterministic checks inspect runner presence, at least 300 fixture JSON files, and workspace build.`)
- **D004-D22-015**: [E][acceptance] Δ (`Done requires compiled runner, 300 hand labels, CLI exit 0 only when all three are within ε, zero new errors, and a decision doc.`)
- **D004-D22-016**: [E][cloud-test] Δ (`Live test uses a D-16 corpus of at least 500 tuples and requires all three empirical rates within ε=0.1; far-OOD is intentionally shifted yet must still satisfy the theorem-derived band.`)
- **D004-D22-017**: [E][surface] + (`Change set adds runner, CLI module, 300 fixture files, and decision doc, and adds SlashCommand::BenchConformalValidate.`)
- **D004-D22-018**: [E][non-goal] Δ (`Per-family/multi-domain τ is deferred; v1 has one tau.`)
- **D004-D22-019**: [E][dependency] + (`D-23 sycophancy benchmark is next.`)
- **D004-D22-020**: [C][dataflow] ~ (`The summary says the three distributions are sourced from D-6, but the implementation creates a separate benchmark fixture tree and the consumes list never names HeldOutStore. D-6 is a declared prerequisite without specified runtime wiring.`)
- **D004-D22-021**: [A][schema] + (`ConformalValidationReport, its verdict, per-distribution row identifier/payload, report filename/format, and CLI nonzero exit behavior are not defined.`)
- **D004-D22-022**: [A][semantics] + (`“distribution shifts by >ε” is not defined separately from refusal_rate outside the τ band; the statistic called shift and its sign/absolute comparison are ambiguous.`)
- **D004-D22-023**: [A][boundary] + (`The source does not say whether [τ−ε,τ+ε] is clamped to [0,1], whether endpoints are inclusive, or how missing/invalid tau is handled.`)
- **D004-D22-024**: [A][statistics] + (`No confidence level, interval method, seed, sampling strategy, stratification, multiple-comparison rule, or mathematical support is specified for “100 is enough for ε=0.1.”`)
- **D004-D22-025**: [A][data] + (`“hand-labeled” has no annotator/reviewer provenance, adjudication, format, privacy/licensing, ground-truth value, or quality gate.`)
- **D004-D22-026**: [A][reliability] + (`refusal_decision errors, partial runs, retry/resume, concurrent tau updates, snapshotting, and reproducibility under nightly D-16 tau changes are unspecified.`)

### User-facing surfaces (D)

- **CLI**: ``elai template migrate`` — D-002/D1-E25, D-002/D1-E38 — migrate TOML/MD prompts to Frozen archive
- **CLI**: ``elai archive verify`` — D-002/D8-E01, D-002/D8-E10, D-001/D0-043 — flags: path, --json, --fail-fast; exit 0/1
- **CLI**: ``elai ppm health`` — D-003/D003-D2-E023 — three-stage health; exit 0/1
- **CLI**: ``elai calibration status`` — D-003/D003-D16-E015 — corpus size, tau, rates; always exit 0
- **CLI**: ``elai calibration update`` — D-003/D003-D16-E016 — manual tau recompute; CI needs L3_OVERRIDE:calibration_update
- **CLI**: ``elai bench sycophancy`` — D-001/D23-009 — D-14 gate CLI
- **CLI**: ``elai bench mode-routing`` — D-002/D21-E09 — D-10 F1 gate
- **CLI**: ``elai bench canary-regression`` — D-002/D24-E10 — D-7 gate
- **CLI**: ``elai bench prmbench`` — D-003/D003-D19-E011 — PPM adversarial categories
- **CLI**: ``elai bench tamper`` — D-003/D003-D25-E015 — 10 scenarios
- **CLI**: ``elai bench arc-agi2`` — D-004/D004-D20-013 — ARC OOD harness
- **CLI**: ``elai bench conformal-validate`` — D-004/D004-D22-009 — tau+/-eps distributions
- **CLI**: ``elai farm-adv-rm run / verdict`` — D-002/D28-E24 — shipping-pass claimed live
- **CLI**: ``elai discovery review`` — D-004/D004-D15-017 — v1 print pending only; TUI deferred
- **CLI**: ``bn_status_report.py --plan=D`` — D-001/D0-068, D-001/D26-034 — progress overlay hook
- **SlashCommand**: ``ArchiveVerify`` — D-002/D8-E10 — dispatch elai archive verify
- **SlashCommand**: ``BenchArcAgi2`` — D-004/D004-D20-026
- **SlashCommand**: ``BenchConformalValidate`` — D-004/D004-D22-017
- **Panel**: ``MutationArchivePanel.svelte`` — D-001/D26-011, D-001/D0-061 — archive list/filter/colors
- **Panel**: ``LineageGraphView.svelte`` — D-001/D26-013, D-001/D0-061 — Cytoscape parent-hash DAG
- **Panel**: ``TripwireStatusStrip.svelte`` — D-001/D26-014 — last 5 events; 3s red SLA
- **Panel**: ``BaiArmsTable.svelte`` — D-001/D26-015 — posteriors + CI
- **Panel**: ``HeldOutPanel.svelte`` — D-001/D26-016 — active/retired per family
- **Panel**: ``ArchiveVerifyGauge.svelte`` — D-001/D26-017 — Clean/Tampered
- **Window**: ``ReasoningDrawer.svelte (D-26 tabs)`` — D-001/D26-018, D-001/D26-023 — archive sub-window + drawer
- **Window**: ``dedicated archive sub-window`` — D-001/D26-023 — D-26 reachability
- **Command**: ``get_archive_recent(limit)`` — D-001/D26-005
- **Command**: ``get_archive_verify_status`` — D-001/D26-006 — async verify
- **Command**: ``get_tripwire_status(window_minutes)`` — D-001/D26-007
- **Command**: ``get_bai_arms`` — D-001/D26-008
- **Command**: ``get_canary_report(optional template_id)`` — D-001/D26-009
- **Command**: ``get_held_out_status`` — D-001/D26-010
- **Panel**: ``ModeRoutingPanel.svelte`` — D-001/D27-011, D-001/D0-062 — pie/stacked modes
- **Panel**: ``RefusalRateGauge.svelte`` — D-001/D27-012, D-001/D0-052 — healthy [5%,50%]
- **Panel**: ``NoveltyBreakdown.svelte`` — D-001/D27-013
- **Panel**: ``DiscoveryQueueDrawer.svelte`` — D-001/D27-014 — ratify/reject
- **Panel**: ``SycophancyFlagStrip.svelte`` — D-001/D27-015, D-001/D0-062 — quarantine links
- **Panel**: ``CheckerStatusPanel.svelte`` — D-001/D27-016 — registry health
- **Window**: ``MonitorWindow (routing/refusal)`` — D-001/D27-017
- **Window**: ``ChatWindow (sycophancy)`` — D-001/D27-017
- **Window**: ``Reasoning tab (novelty/queue/checkers)`` — D-001/D27-017
- **Command**: ``get_mode_breakdown`` — D-001/D27-004
- **Command**: ``get_refusal_rate`` — D-001/D27-005
- **Command**: ``get_novelty_breakdown`` — D-001/D27-006
- **Command**: ``get_sycophancy_flags`` — D-001/D27-007
- **Command**: ``get_discovery_queue`` — D-001/D27-008
- **Command**: ``ratify_discovery`` — D-001/D27-009 — requires reviewer
- **Command**: ``reject_discovery`` — D-001/D27-010 — requires reviewer+reason
- **Flag**: ``ELAI_PPM_ENABLE=1`` — D-003/D003-D2-E018, D-002/D10-E16 — opt-in PPM
- **Flag**: ``ELAI_CLOUD_MODE`` — D-002/D10-E16 — alternate-family availability
- **Flag**: ``ELAI_SEED`` — D-001/D5-028 — deterministic BAI
- **Flag**: ``ELAI_BAI_MODE=hierarchical`` — D-001/D29-011, D-001/D29-019 — with ELAI_BAI_ENABLE=1
- **Flag**: ``ELAI_BAI_ENABLE=1`` — D-001/D29-019
- **Flag**: ``ELAI_CALIBRATION_BOOTSTRAP_ENABLE`` — D-003/D003-D30-E013 — bootstrap mode
- **Flag**: ``L3_OVERRIDE:* commit tag`` — D-002/D9-E27, D-003/D003-D16-E016 — locked-key / calibration override
- **Flag**: ``[ci skip] commit message`` — D-002/D8-E13 — skips archive-verify CI
- **Config**: ``[bai] L3-locked`` — D-001/D5-027 — eval_budget, ci_alpha, deltas
- **Config**: ``[held_out] L3-locked`` — D-001/D6-025
- **Config**: ``[tripwire] L3-locked`` — D-004/D004-D4-011, D-004/D004-D4-026
- **Config**: ``[canary] L3-locked`` — D-004/D004-D7-024
- **Config**: ``[verification] L3-locked`` — D-003/D003-D3-E024
- **Config**: ``[runtime.ppm]`` — D-003/D003-D2-E017
- **Config**: ``[adversary] L3-locked`` — D-003/D003-D13-E021
- **Config**: ``[conformal] L3-locked`` — D-003/D003-D16-E019
- **Config**: ``[refusal_sla] L3-locked`` — D-003/D003-D17-E011
- **Config**: ``[sycophancy_probe] L3-locked`` — D-002/D14-E20
- **Config**: ``[observable_effect] L3-locked`` — D-004/D004-D18-024
- **Config**: ``[novelty] (referenced)`` — D-004/D004-D15-023 — underspecified defaults
- **Config**: ``reasoning.templates (commented)`` — D-002/D1-E30 — archive_path, bounded_string_max
- **Notification**: ``Bubble-Up RefusalRateOutOfBand`` — D-003/D003-D17-E008 — B13 pattern
- **Notification**: ``LockedKeysTouch / L3OverrideRequested`` — D-002/D9-E25, D-001/D0-044
- **Notification**: ``SYCOPHANCY_FLAG / quarantine`` — D-002/D14-E01, D-002/D14-E17, D-002/D14-E29 — blocks MEMORIZING
- **Notification**: ``TripwireFiredConcurrent + AutoRevert`` — D-004/D004-D4-022, D-004/D004-D4-024
- **Notification**: ``CrossDomainRegression block`` — D-004/D004-D7-023, D-004/D004-D7-025
- **Notification**: ``ClaimWithoutProcedure auditor BLOCK`` — D-001/D12-014, D-001/D12-024
- **Notification**: ``DiscoveryCandidate queue + ratify/reject`` — D-004/D004-D15-014, D-001/D27-009
- **Notification**: ``GoodhartRiskFlag`` — D-004/D004-D18-022, D-001/D0-053
- **StatusDisplay**: ``Archive verify Clean/Tampered`` — D-001/D26-017
- **StatusDisplay**: ``Refusal gauge healthy band`` — D-001/D27-012, D-003/D003-D17-E006
- **StatusDisplay**: ``Tripwire strip yellow/red`` — D-001/D26-014
- **StatusDisplay**: ``Canary report Allow/Block/InsufficientData`` — D-004/D004-D7-017
- **StatusDisplay**: ``PRMBench GatingVerdict`` — D-003/D003-D19-E009
- **StatusDisplay**: ``ARCVerdict HonestlyEvaluated/RefusalOutOfBand/Contamination`` — D-004/D004-D20-011
- **StatusDisplay**: ``ModeSelected distribution`` — D-001/D27-011, D-002/D10-E24
- **Hook**: ``scripts/locked_keys_check.sh pre-commit`` — D-002/D9-E20, D-001/D0-044
- **Hook**: ``CI archive-verify.yml`` — D-002/D8-E12
- **Hook**: ``CI locked-keys-audit.yml`` — D-002/D9-E31
- **Hook**: ``CI bench-prmbench.yml`` — D-003/D003-D19-E013

### Superseded notes index (D)

- **SUP-001** (Baby-step block number for MEASURABLE_PROCEDURE): Target: D-12 block 9 (eight existing + ninth). Superseded: Summary said block 7.
- **SUP-002** (D-1 archive hash formula): Target: Unresolved conflict preserved: D-1 two-part (body+parent) vs D-8 three-part (+mutation_proposal_text). Superseded: Neither fully supersedes; target state requires reconciliation before implement.
- **SUP-003** (D-26 prerequisites): Target: D-26 original: D-1..D-9 shipped. Superseded: D-0 index: D-1..D-8.
- **SUP-004** (D-28 status and SSB events): Target: Shipping-pass: AdversarialFarmRunCompleted; local CLI/SSB legs claimed live; training/deploy deferred. Superseded: Header Status:DEFERRED and two-event design AdversarialCorpusGenerated/PpmFinetuneCompleted.
- **SUP-005** (D-29 hierarchical BAI status): Target: Mixed: Path A host Rust dispatch claimed shipped; Gibbs/RunPod research closure still DEFERRED. Superseded: Summary/D-0 clean DEFERRED only.
- **SUP-006** (D-30 calibration bootstrap status): Target: Partial: Rust dispatch claimed wired/live; corpus/data/D22 legs DEFERRED. Superseded: Summary blanket DEFERRED.
- **SUP-007** (D-15 NoveltyClass attachment target): Target: ClaimResult metadata (D-15 original). Superseded: Summary: field on CheckerResult.
- **SUP-008** (D-7 canary block rule): Target: Unresolved: objective non-target-only vs implementation ANY-family below threshold. Superseded: Neither deleted; both retained.
- **SUP-009** (D-17 SLA event name): Target: RefusalSlaBreach + AlertKind::RefusalRateOutOfBand. Superseded: Summary AUTO_FLAG.
- **SUP-010** (D-14 sycophancy dispatch event): Target: SycophancyProbeDispatched. Superseded: Narrative SYCOPHANCY_PROBE.
- **SUP-011** (D-20 synthetic ARC fallback): Target: v1: ContaminationSuspected/exit 1 only; synthetic generation deferred. Superseded: Objective promised synthetic fallback if leakage flagged.
- **SUP-012** (D-22 distributions source): Target: Separate 300-file benchmark tree; HeldOutStore not wired. Superseded: Summary: distributions sourced from D-6.
- **SUP-013** (D-3 deceptive consensus default): Target: Default AtLeastTwo can pass PPM+intrinsic with deterministic fail; code claims use RequireAll. Superseded: Summary unconditional deterministic-fail blocks promotion.
- **SUP-014** (D-2 PpmClient shape): Target: Concrete struct PpmClient + PpmAdapter RuntimeAdapter. Superseded: Summary trait.
- **SUP-015** (D-1 staged_eval_result field name): Target: staged_eval_result (+ retirement_reason). Superseded: Summary eval_result projection.
- **SUP-016** (D-26/D-27 dashboard surface counts): Target: D-26: 6 commands + 6 components; D-27: 7 commands + 6 components. Superseded: Summary thinner command/component lists.
- **SUP-017** (D-21 Mode C benchmarkability): Target: Conflict retained: D-21 demands Mode C fixtures; D-10 never returns DiscoveryLabelOnly from selector. Superseded: Neither deleted.
- **SUP-018** (D-19 CI enforcement): Target: Informational warn-not-block until production-gating transition. Superseded: Summary direct 'gates PPM promotion' without CI nuance.

## Audit block (SECTION-PHASE-D)

- **PHASE units covered**: 44 — phase-0, phase-1C, phase-3A, phase-3J-BASELINE, phase-4K, phase-4M, 1A, 1D, 3D, 3G, 4D-ORACLE, 4G, phase-1B, phase-1G, phase-3H, phase-4D-SURGICAL, phase-4E, phase-index-3A-4J, 1E, 2B, 2C, 3L, 4C, 4I, 1F, 2A, 3F, 3J, 1H, 3I, 3K, 4A, 4H, pre-phase-0, phase-1I, phase-2D, phase-3B, phase-4D, 3C, 3E, 4B, 4F, 4J, IDX
- **D units covered**: 31 — D-0, D-5, D-6, D-12, D-23, D-26, D-27, D-29, D-1, D-8, D-9, D-10, D-14, D-21, D-24, D-28, D-2, D-3, D-13, D-16, D-17, D-19, D-25, D-30, D-4, D-7, D-11, D-15, D-18, D-20, D-22
- **Entry count (PHASE + D plan units)**: 75
- **Behavior bullets PHASE**: 1677
- **Behavior bullets D**: 1215
- **Behavior bullets total**: 2892
- **User-facing surfaces PHASE**: 243
- **User-facing surfaces D**: 88
- **Surfaces total**: 331
- **Superseded notes count (D SUP-*)**: 18
- **Per-D-unit atom parse**: D-0=68/68, D-5=45/45, D-6=52/52, D-12=38/38, D-23=21/21, D-26=34/34, D-27=33/33, D-29=28/28, D-1=57/57, D-8=41/41, D-9=52/52, D-10=53/53, D-14=47/47, D-21=33/33, D-24=28/28, D-28=47/47, D-2=42/42, D-3=37/37, D-13=32/32, D-16=31/31, D-17=24/24, D-19=24/24, D-25=23/23, D-30=24/24, D-4=52/52, D-7=44/44, D-11=61/61, D-15=38/38, D-18=44/44, D-20=36/36, D-22=26/26
- **Sources**: `docs/thefinalglance/feature-discovery/aggregates/PHASE-AGGREGATE.md`, `docs/thefinalglance/feature-discovery/aggregates/D-AGGREGATE.md`
- **Output**: `docs/thefinalglance/feature-discovery/aggregates/compile/SECTION-PHASE-D.md`

ZERO-OMISSION intent: every plan-unit from both aggregates has an entry; every source atom from unit sections is re-emitted as a behavior bullet with atom ID. Summary-layer SUM-* inventory is folded into D unit statuses and Superseded notes. Relational IX/AC/SD registers are cited under Composes with / Superseded rather than re-listed as pseudo-units.

### Relational registers (PHASE)

Source: `docs/thefinalglance/feature-discovery/aggregates/PHASE-AGGREGATE.md`.
Emits every unique namespaced IX/SD/AC definition row (first occurrence). Format: `ID | kind/action | statement | anchor`.

#### PHASE IX (172)

- **PHASE-001/IX-001** | E | Phase 0 nine-crate split, LlmProvider, minimal FSM, visibility, msgpack, contracts, entropy quarantine are substrate for later phases here. | P0-E002, P0-E046, P0-E049–P0-E065
- **PHASE-001/IX-002** | E | Phase 0 ONNX path + DEGRADED BM25_ONLY + models/** hidden_paths match Phase 1C dual-model resilience. | P0-E015–P0-E017; P1C-E018, P1C-E029
- **PHASE-001/IX-003** | E | Phase 1C ClockProvider breaker ban on std::time consummates Phase 0 entropy quarantine. | P0-E060–P0-E064; P1C-E026
- **PHASE-001/IX-004** | E | Phase 1C SSB file_write invalidation and knowledge_degradation require Phase 3B SSB (outside chunk) but specify production wiring. | P1C-E021, P1C-E032
- **PHASE-001/IX-005** | E | Phase 3A extends Phase 0 IDLE/EXECUTING to nine states, evidence gates, InteractiveGate, risk; AUTONOMOUS to 4G. | P0-E052; P3A-E001–P3A-E029
- **PHASE-001/IX-006** | E | Phase 3A IDLE→PLANNING consumes knowledge readiness; later gates consume tests/DKG. | P3A-E019, P3A-E023; P1C-E029
- **PHASE-001/IX-007** | E | Phase 3A defers SSB phase_transition to 3B; 4K visualizes phase_progress and recent_actions from that journal. | P3A-E033; P4K-E012, P4K-E047
- **PHASE-001/IX-008** | E | Phase 4K interventions operationalize Phase 3A InteractiveGate via INTERVENTION_REQUEST/RESPONSE UI. | P3A-E024–P3A-E025; P4K-E013–P4K-E014, P4K-E049–P4K-E052
- **PHASE-001/IX-009** | E | Phase 4K SsbViewer read-only consumer of 3B SSB; elai --dashboard desktop control surface. | P4K-E011–P4K-E026
- **PHASE-001/IX-010** | E | Phase 3J-BASELINE fills capability_profile.toml for ModelCapabilityProfile without booting 3A/4K. | P3JB-E001–P3JB-E002, P3JB-E007–P3JB-E009
- **PHASE-001/IX-011** | E | 3J-BASELINE JSON-schema gate exercises ConstrainedDecodingConfig; 4M capabilities.json_schema_mode reports related capability. | P3JB-E011; P4M-E006
- **PHASE-001/IX-012** | E | Phase 4M migrates 3J backends and Ollama/vLLM into RuntimeAdapter; requires 4A Tier C for CLI adapters. | P4M-E001, P4M-E014–P4M-E020
- **PHASE-001/IX-013** | E | Phase 4M /runtime and RuntimePanel extend operator surface beyond Phase 0 slash set and 4K workspace. | P0-E055; P4K-E026; P4M-E023–P4M-E028
- **PHASE-001/IX-014** | E | Phase 4M runtime_fallback uses 3A INTERACTIVE ESCALATE vs AUTONOMOUS HALT; orders before 4G model fallback. | P3A-E001; P4M-E022
- **PHASE-001/IX-015** | E | Slash command evolution: Phase 0 /task | /config
- **PHASE-001/IX-016** | I | RuntimePanel health colors can share Ready/Degraded/Unavailable vocabulary with 1C readiness without a shared enum named here. | P1C-E029; P4M-E014, P4M-E028
- **PHASE-001/IX-017** | E | 1C three-ranker RRF stays additive for later SPLADE/cross-encoder; semaphore overflow unranked. | P1C-E007, P1C-E030–P1C-E031
- **PHASE-001/IX-018** | E | Phase 0 NotFound-indistinguishable visibility reused as security pattern; 3A tool filters are a separate state axis. | P0-E053, P0-E056; P3A-E017
- **PHASE-002/IX-001** | E | Phase 1A unblocks 1D: directed outbound/inbound edge API, DkgNodeInfo signature+SymbolKind, and DkgSnapshot::from_engine PRE_GATE. | P1A-E008, P1A-E012, P1A-E027; P1D-E006, P1D-E022
- **PHASE-002/IX-002** | E | 1A failed_files bridges parse-failed files into 1B BM25; PageRank leaf treatment of failures is deferred to 1D (compliance item 4). | P1A-E009, P1A-E010, P1A-E043; P1D-E001
- **PHASE-002/IX-003** | E | 1D density-guard fallback is insertion-order, not BM25, because BM25 rerank is upstream in the query pipeline. | P1D-E011
- **PHASE-002/IX-004** | E | 3D requires 3B SSB + 3C agents; upgrades 3C spawn_parallel stub to true JoinSet parallelism; 3E/3F consume ChannelRegistry. | P3D-E001, P3D-E019, P3D-E013
- **PHASE-002/IX-005** | E | 3D checkpoint+SSB replay composes with 3C agent checkpoints under `.elai_cc/sessions/{sid}/`; UntrackedValue deliberately absent from both. | P3D-E015, P3D-E018, P3D-E011
- **PHASE-002/IX-006** | E | 3G.4 requires 3E Slot 3 ContextBudget; skeleton always loads in PhaseSpec slot; overflow uses 3E ReducedGranularity. | P3G-E001, P3G-E015
- **PHASE-002/IX-007** | E | 3G coherence is invoked by 3C auditor via IPC; findings use action_type finding; plan mutations use plan_update/stub_reference. | P3G-E010, P3G-E011, P3G-E016
- **PHASE-002/IX-008** | E | 3G split_phase max depth 2 matches 4G SPLIT_TASK max_depth 2 and preserve_contracts for L2 phase sizing. | P3G-E014; P4G-E009
- **PHASE-002/IX-009** | E | 4G SPLIT_TASK is the Terminal path consumer of 3E context-budget exhaustion; sequential merge avoids 3H/S10 pick-one branches. | P4G-E009, P4G-E010
- **PHASE-002/IX-010** | E | 4G SUBSTITUTE_MODEL is the autonomous completion of 3C on_max_restarts policy (interactive escalates). | P4G-E011
- **PHASE-002/IX-011** | E | 4G wraps 3A InteractiveGate without new FSM states; evidence gates from orchestrator S13; risk levels from arch-overview S6. | P4G-E001, P4G-E002, P4G-E005
- **PHASE-002/IX-012** | E | 4G tree auto-advance integrates 4E TreeNavigator + BubbleUpHandler; concept-level always human. | P4G-E019, P4G-E020
- **PHASE-002/IX-013** | E | 4G.9 Spinning subsumes prior 3× GRC anti-spiral; personas spawn via 4A Tier-C isolation. | P4G-E021, P4G-E023, P4G-E029
- **PHASE-002/IX-014** | E | 4D-ORACLE cache sits on /research pipeline used by 4D research path; C-30 shipped SCHEMA 62 SSB hit/miss. | P4DOR-E001, P4DOR-E002, P4DOR-E013
- **PHASE-002/IX-015** | E | Hash policy split: 1A/1D knowledge path uses blake2s for file content; 4DOR/4G stagnation require blake3; 4DOR bans blake2/sha2 for cache keys. | P1A-E013, P1A-E035; P4DOR-E014; P4G-E021
- **PHASE-002/IX-016** | E | SSB action-type growth across chunk: 3D memory_monitoring + ephemeral clear in checkpoint; 3G plan_update/stub_reference/finding; 4DOR oracle_cache_*; 4G escalation + stagnation_* + lateral_* + watchdog_timeout + goal_drift_* + cloud_spend_*. | P3D-E017, P3D-E023; P3G-E007, P3G-E011; P4DOR-E012; P4G-E014, P4G-E022, P4G-E032
- **PHASE-002/IX-017** | I | 1A index build observable symbol_graph_updated and 1D repo maps jointly feed later Slot 3 / context pipelines used by 3E and 4G evidence readiness checks. | P1A-E039; P1D-E009; P4G-E005
- **PHASE-002/IX-018** | E | 4G circuit breakers operate at two grains: phase auto-advances (default 10) and tree baby steps per concept (default 50). | P4G-E007, P4G-E019
- **PHASE-002/IX-019** | E | Goal-drift Block and watchdog timeouts both hand off to 4E BubbleUpHandler rather than inventing a third recovery owner. | P4G-E026, P4G-E027
- **PHASE-002/IX-020** | E | 3D straggler timeout reuses 3C heartbeat 15s; 4G GenerationWatchdog adds independent idle/material/safety timeouts per baby step. | P3D-E005; P4G-E026
- **PHASE-003/IX-001** | 1B BM25 feeds 1E/1G pipeline stages; 1G skips in BM25_ONLY/SPARSE_ONLY that 1B+1I establish. | P1B-E007, P1B-E018, P1G-E006 | 
- **PHASE-003/IX-002** | 1G canary requires BM25 candidates from 1B path; shared knowledge degradation SSB with stage discriminators. | P1G-E012, P1G-E014, P1B-E018 | 
- **PHASE-003/IX-003** | 3H trimming target formula depends on 3E allocated slot tokens; index 3E floors feed that budget. | P3H-E010, P3H-A006, PIDX-E006 | 
- **PHASE-003/IX-004** | 3H stub recovery and merge events depend on 3B SSB append-only + seq query; branch_abandon amendment touches 17-type registry. | P3H-E013-E014, PIDX-E003, PIDX-C001 | 
- **PHASE-003/IX-005** | 3H never stubs Slots 1–3; 3E/3G place system/rules/phase (skeleton) in those slots. | P3H-E009, PIDX-E006, PIDX-E008 | 
- **PHASE-003/IX-006** | 4E Phase Sizing uses 3G PlanSkeleton::split_phase depth≤2 matching 4G SPLIT_TASK depth. | P4E-E010, PIDX-E008, PIDX-E020 | 
- **PHASE-003/IX-007** | 4E novel_step_ids consumed by 4F; visual_check adapters by 4I. | P4E-E027, PIDX-E019, PIDX-E022 | 
- **PHASE-003/IX-008** | 4E baby-step cycle reuses 3A FSM states TESTING_FIRST/EXECUTING/AUDITING and 4C T1–T3 hooks. | P4E-E018-E019, PIDX-E002, PIDX-E016 | 
- **PHASE-003/IX-009** | 4E bubble-up FlashResearch and ConceptDecomposition types come from 4D L1. | P4E-E014, P4E-E020, PIDX-E017 | 
- **PHASE-003/IX-010** | 4D-SURGICAL ExecutionMode and EscalationPolicy gates compose 3E context_budget + 4G-style autonomous confirm thresholds. | P4DS-E009, P4DS-E015, PIDX-E006 | 
- **PHASE-003/IX-011** | 4D-SURGICAL DKG blast-radius queries depend on 1A DKG + rules DkgQuerier extension. | P4DS-E006-E007, P1B-E001 | 
- **PHASE-003/IX-012** | Index 3K replaces 1B fixed 50-line windows with AST chunking — future mutation of 1B fallback path. | PIDX-E011, P1B-E010, P1B-E014 | 
- **PHASE-003/IX-013** | Index 3J.WIRE.SESSION replaces raw jsonl with SessionDag + auto-trim (3H integration path). | PIDX-E012, P3H-E015 | 
- **PHASE-003/IX-014** | Surgical mode intercepts before 4D DefaultWebResearchRunner; parallel path to 4E tree execution, not a substitute for L2 builder. | P4DS-E002-E003, P4E-E001, PIDX-E017 | 
- **PHASE-003/IX-015** | RuntimeClient ssb/fsm/clock rule in 4E ties L2 assembly to 3A/3B entropy-quarantine composition. | P4E-E013, PIDX-E002-E003 | 
- **PHASE-003/IX-016** | Shared degradation/observability: 1G knowledge_degradation vs 4DS surgical_* vs 4E replan_request are distinct action families. | P1G-E012, P4DS-E010, P4E-E021 | 
- **PHASE-003/IX-017** | User surfaces chain: `index build` (1B) → knowledge canary (1G) → `/task --surgical` + SurgicalDrawer (4DS) → `/mode`/`/model`/`/plan` (index 3J/4G). | P1B-E019, P4DS-E002, P4DS-E011, PIDX-E013, PIDX-E024 | 
- **PHASE-003/IX-018** | 4E integration time ladder matches 4B IntegrationTestRunner levels in index. | P4E-E022, PIDX-E015 | 
- **PHASE-003/IX-019** | 4H MSI tracks SURGICAL/GREENFIELD modes; surgical CLI path is a related but separate C-30 feature. | PIDX-E021, P4DS-E001 | 
- **PHASE-003/IX-020** | Handoff 85% (3H) aligns with 4C PreCompact checkpoint-at-85% theme in index. | P3H-E002, PIDX-E016 | 
- **PHASE-004/IX-001** | E | 1E composes 1B BM25 + 1C vectors + 1D PageRank/DKG; leaves explicit stubs for 1G rerank and 3E token_fit. | P1E-E001, P1E-E006, P1E-E026
- **PHASE-004/IX-002** | E | 1E density guard and circuit-breaker skips feed 1I resilience/mode cascade and QueryTrace stage list. | P1E-E007, P1E-E023, P1E-E027
- **PHASE-004/IX-003** | E | 2B replaces 2A unimplemented verify_claim/ground_plan stubs with breaking signatures consumed by 2C method policy. | P2B-E012, P2B-E019, P2C-E006
- **PHASE-004/IX-004** | E | 2B plan sweep supplies the ungrounded-zero evidence gate for 3A PLANNING→TESTING_FIRST / EXECUTING path. | P2B-E003, P2B-E020
- **PHASE-004/IX-005** | E | 2C allowed_methods and grounding_level layer on 2B VerificationMethod enum; 4A later dual-enforces via IPC. | P2C-E006, P2C-E015, P2B-E005
- **PHASE-004/IX-006** | E | 2C auditor double-verify + FINDING anti-rubber-stamp is last defense before acceptance; composes with 3A AUDITING evidence. | P2C-E004, P2C-E007
- **PHASE-004/IX-007** | E | 3L Pi hooks reuse plugins HookVerdict/TieredAnalysisResult shapes without forking 4C SSB-write T1/T2/T3 family. | P3L-E003, P4C-E009
- **PHASE-004/IX-008** | E | 3L CONVENTIONS injection via reasoning prompt_builder and /profile CLI; telescope priorities sit upstream of agent SYSTEM context. | P3L-E012, P3L-E015, P3L-E007
- **PHASE-004/IX-009** | E | 3L predecessor is 3K RAG hardening; convention signal is additive prompt input, not a replacement for 1E pipeline. | P3L-E002, P1E-E001
- **PHASE-004/IX-010** | E | 4C PostToolUse ShortCoder placement resolves 4B tools.txt code_simplify + 4A WARN-3 under Tier C before auditor FS. | P4C-E003, P4C-E017
- **PHASE-004/IX-011** | E | 4C T1 SCHEMA_REQUIRED Escalate to ProviderTier couples rules DSL to provider constrained decoding (outside 4C body). | P4C-E005, P4C-E011
- **PHASE-004/IX-012** | E | 4C circular-trust normalization + AUDITOR PROPOSED freeze composes with rules.txt self-discovery and 3F-style promotion gates. | P4C-E010
- **PHASE-004/IX-013** | E | 4I lazy media path depends on 4D MediaSpec / 3G visual_check+output_type and 4E adapter structural check #8. | P4I-E001, P4I-E012, P4I-E023
- **PHASE-004/IX-014** | E | 4I VLM process uses 4A Tier C isolation + same postcard IPC as agents; ToolCapabilityFlags degrade without panicking. | P4I-E012, P4I-E020, P4I-E004
- **PHASE-004/IX-015** | E | 4I VisualFeedbackLoop writes SSB visual_feedback so 3B journal and 3C/4G liveness observe long RCE iterations. | P4I-E018, P4I-E017
- **PHASE-004/IX-016** | E | Hash split in chunk: 4C violation_patterns blake2s vs knowledge blake2 elsewhere; 4I does not introduce new content hash for media. | P4C-E010
- **PHASE-004/IX-017** | E | Budget authority handoff 2C→4A mirrors hook authority: enforcement is orchestrator-side under Tier C (4C hooks never in agent binary). | P2C-E015, P4C-E001
- **PHASE-004/IX-018** | I | 1E measure binary and 2B/2C/3L/4C/4I phase_*.toml metrics share the implementation-guide "every phase measures" pattern for later dashboards. | P1E-E028, P2B-E021, P2C-E016, P3L-E017, P4C-E015, P4I-E014
- **PHASE-004/IX-019** | E | 4C cross-session write locks and 2B FILE_CHECK both depend on tools/path reality but enforce different layers (policy hooks vs claim truth). | P4C-E013, P2B-E017
- **PHASE-004/IX-020** | E | Designer AgentGrcConfig (2C) produces PipelineSpec that later may include media steps triggering 4I lazy factory. | P2C-E018, P4I-E001
- **PHASE-005/IX-001** | E | 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite. | P1F-E004, P1F-E015, P1F-E016; P3J-E001, P3J-E009
- **PHASE-005/IX-002** | E | 1F requires 1C ModelPool VRAM accounting for draft overhead measurement. | P1F-E007, P1F-E010
- **PHASE-005/IX-003** | E | 2A freezes GrcEngine six-method trait so 2B can implement verify_claim/ground_plan without breaking BasicGrcEngine. | P2A-E016, P2A-E017
- **PHASE-005/IX-004** | E | 2A scope enumeration WARNs feed agent context; orchestrator (outside 2A) enforces plan_action coverage. | P2A-E013
- **PHASE-005/IX-005** | E | 3F depends on 3D complete and 1A tree-sitter via AstEvaluator; self-discovery needs 3C agent/auditor IPC. | P3F-E001, P3F-E006, P3F-E032
- **PHASE-005/IX-006** | E | 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM). | P3F-E018, P3F-E019
- **PHASE-005/IX-007** | E | 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7). | P3F-E032, P3F-E035; P3J-E028, P3J-E029
- **PHASE-005/IX-008** | E | 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters. | P3J-E048, P3J-E054; P3F-E036
- **PHASE-005/IX-009** | E | 3J workload_profile + advanced SSB types extend 3B core actions to ≥17 including phase_complete atomic boundary used later by 4A. | P3J-E023, P3J-E033
- **PHASE-005/IX-010** | E | 3J.WIRE.SESSION composes 3H SessionDag trim 85% + stub recovery from SSB. | P3J-E050, P3J-E051
- **PHASE-005/IX-011** | E | 3J.WIRE.PLAN composes 3G PlanSkeleton into CLI Slot 2 and /plan surfaces. | P3J-E052
- **PHASE-005/IX-012** | E | 3J.WIRE.FOUNDATION InteractiveGate is the 3A/4G-shared human gate surface (Approve/Reject/Modify/Redirect). | P3J-E045
- **PHASE-005/IX-013** | E | 3J.14 retrieval injects 1B/1E/1G pipeline pieces (BM25, CatRAG, ARAG, optional cross-encoder) into prompt Slot 4. | P3J-E060
- **PHASE-005/IX-014** | E | 3J.15 SUBSTITUTE_MODEL 100B path anticipates 4G escalation; anti-spiral 3× same error swap. | P3J-E064
- **PHASE-005/IX-015** | E | Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception. | P3F-E032; P3J-E029
- **PHASE-005/IX-016** | E | 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers. | P2A-E007, P2A-E027; P3J-E037, P3J-E047
- **PHASE-005/IX-017** | I | 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals. | P1F-E021, P1F-E032; P3J-E002
- **PHASE-005/IX-018** | E | 3J MCP tools join native tools under uniform ToolRequest/ToolResult after 3C agents exist. | P3J-E025, P3J-E027
- **PHASE-005/IX-019** | E | AppContainer self-mod locks prefigure 4A BLOCK-8 realpath+FS+git filters. | P3J-E031
- **PHASE-005/IX-020** | E | DoLa disabled for prompt-engineer aligns with constrained-decoding-only role (addendum A-10). | P3J-E013, P3J-E066
- **PHASE-006/IX-001** | E | Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this). | P1H-E027, P1H-E032; P3K-E003
- **PHASE-006/IX-002** | E | 3K hardening composes 1E A-RAG/CatRAG stubs, 1G reranker, 1C embeddings, 1A DKG grammars, and 3J sidecar only for HyDE. | P3K-E001–P3K-E003, P3K-E010, P3K-E015–P3K-E018
- **PHASE-006/IX-003** | E | 3K.4 QueryTier intentional quality selection is explicitly distinct from 1I SearchMode fault-degradation cascade. | P3K-E012
- **PHASE-006/IX-004** | E | 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence. | P3I-E001–P3I-E002; P4A-E036; P4H-E001
- **PHASE-006/IX-005** | E | 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary. | P3I-E018; P4A-E013–P4A-E014
- **PHASE-006/IX-006** | E | 3I entropy quarantine (zero raw time/rng in orchestrator) is a hard PRE_GATE for BSP determinism and for any later seed-replay of isolation tests. | P3I-E016
- **PHASE-006/IX-007** | E | 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch. | P4A-E001; P4H-E001, P4H-E027, P4H-E038
- **PHASE-006/IX-008** | E | 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging. | P4A-E016; P4H-E027, P4H-E038
- **PHASE-006/IX-009** | E | 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern. | P4A-E008; P4H-E026, P4H-E032
- **PHASE-006/IX-010** | E | 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list. | P4A-E010, P4A-E034; P4H-E027, P4H-E038
- **PHASE-006/IX-011** | E | 4A ShortCoder deferral to 4B orchestrator placement is a cross-phase WARN-3 handoff outside this chunk's implementation. | P4A-E035
- **PHASE-006/IX-012** | E | 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria. | P4A-E018, P4A-E038; P4H-E048
- **PHASE-006/IX-013** | E | 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance. | P4H-E009, P4H-E031, P4H-E043
- **PHASE-006/IX-014** | E | 4H FailureTaxonomy expansion closes reasoning-architecture Layer1 with fix templates from higher layers' successful resolutions. | P4H-E010, P4H-E032, P4H-E044
- **PHASE-006/IX-015** | E | 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop. | P4H-E034–P4H-E036; P3K-E014; P1H-E024–P1H-E026
- **PHASE-006/IX-016** | E | 4H OSC/REG/EVO ouroboros ports operate per PipelineMode in parallel with MSI and feed SSB action types. | P4H-E040–P4H-E042, P4H-E045
- **PHASE-006/IX-017** | E | 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc. | P3K-E017; P4A-E021
- **PHASE-006/IX-018** | E | Budget authority handoff: 2C in-process budgets become advisory under 4A orchestrator ThinkStep counting. | P4A-E006, P4A-E023
- **PHASE-006/IX-019** | I | 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation. | P3K-E003; P1H-E027
- **PHASE-006/IX-020** | E | Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT. | P4A-E032; P4H-E041
- **PHASE-006/IX-021** | E | 3I VCR uses blake2s for prompt hashing while much of ELAI elsewhere bans blake2; this phase-local choice is intentional for cassette keys. | P3I-I001
- **PHASE-006/IX-022** | E | Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network. | P4A-E003–P4A-E004; P3K-E010
- **PHASE-007/IX-001** | E | Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D. | PP0-E001–PP0-E022; P1I-E012; P2D-E013; P3B-E028; P4D-E019
- **PHASE-007/IX-002** | E | Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path. | P1I-E016; P3B-E010, P3B-E019–P3B-E020
- **PHASE-007/IX-003** | E | Phase 1I ReadinessStatus READY|DEGRADED|UNAVAILABLE gates orchestrator FSM IDLE→PLANNING; UNAVAILABLE blocks all transitions (3A evidence gate consumer). | P1I-E022–P1I-E024
- **PHASE-007/IX-004** | E | Phase 1I SearchMode cascade and BM25_ONLY degraded mode compose with 1B/1C isolation verified under degradation. | P1I-E013–P1I-E018, P1I-E036
- **PHASE-007/IX-005** | E | Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate. | P1I-E006; P3B-E019
- **PHASE-007/IX-006** | E | Phase 2D Blocked outcome signals ReplanRequest(MAJOR) for orchestrator 3A+ human-approval EXECUTING→PLANNING. | P2D-E007, P2D-E022; P2D-E015
- **PHASE-007/IX-007** | E | Phase 2D CSQ depends on 2C AgentGrcConfig.csq_required and per-role CSQ mandates. | P2D-E001, P2D-E021
- **PHASE-007/IX-008** | E | Phase 2D inline feasibility SourceMatch is temporary; 4F adds WebSearch/Grep/ApiRegistryCheck variants and deeper contradiction analysis. | P2D-E009, P2D-E022; P2D-C001
- **PHASE-007/IX-009** | E | Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events. | P3B-E002–P3B-E004; P4D-E018, P4D-E037, P4D-E054
- **PHASE-007/IX-010** | E | Phase 3B WARN-6 write-before-ToolResult contract binds agent tool IPC to durable journal order. | P3B-E020
- **PHASE-007/IX-011** | E | Phase 3B PhaseCompletePayload fsync-before-unload durability rule composes with agent lifecycle unload paths. | P3B-E024
- **PHASE-007/IX-012** | E | Phase 4D Research Agent Tier C isolation reuses Phase 4A process walls and IPC proxy patterns. | P4D-E001, P4D-E014, P4D-E053
- **PHASE-007/IX-013** | E | Phase 4D Designer GRC config reuses 2C role specialization pattern (distinct from Executor). | P4D-E005
- **PHASE-007/IX-014** | E | Phase 4D SystemConstraints sizing defaults feed later L2 sizing/feasibility consumers. | P4D-E006
- **PHASE-007/IX-015** | E | Phase 4D package API_REGISTRY_CHECK reuses 2B verification method family. | P4D-E032, P4D-E035
- **PHASE-007/IX-016** | E | Phase 4D Interactive present_options is a human-gate surface sibling to 3A InteractiveGate / later dashboard interventions. | P4D-E034, P4D-I001
- **PHASE-007/IX-017** | C | Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension. | P4D-C001; P3B-E010; P4D-A001
- **PHASE-007/IX-018** | E | Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer. | P1I-E034; P4D-E057
- **PHASE-007/IX-019** | I | Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness. | PP0-E025; P3B-E010, P3B-E012
- **PHASE-007/IX-020** | E | Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml. | PP0-E021; P2D-E019; P4D-E025, P4D-E049, P4D-E056
- **PHASE-008/IX-001** | E | 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection. | P3C-E009; P3E-E015
- **PHASE-008/IX-002** | E | 3C spawn_parallel sequential stub is upgraded to true parallel only by 3D. | P3C-E008, P3C-E024
- **PHASE-008/IX-003** | E | 3C communicate in-process stub becomes IPC in 4A; same AgentHandle abstraction. | P3C-E011, P3C-E024
- **PHASE-008/IX-004** | E | 3C ReplanRequest MAJOR human gate and MINOR PLANNING fast-track compose with 3A FSM and auditor. | P3C-E012, P3C-E031
- **PHASE-008/IX-005** | E | 3C ReasoningTraceEntry and checkpoint events depend on 3B SSB single-writer append path. | P3C-E005, P3C-E032, P3C-E037
- **PHASE-008/IX-006** | E | 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400. | P3C-E007, P3C-E026; IDX-E019
- **PHASE-008/IX-007** | E | 3C Designer role produces PipelineSpec later consumed by 4D/4E/4F chain. | P3C-E023
- **PHASE-008/IX-008** | E | 3E budget is recomputed inside 3D BSP superstep between prepare_next_tasks and execute_tasks; consumes channel kinds from 3D. | P3E-E016, P3E-E017
- **PHASE-008/IX-009** | E | 3E Terminal AUTONOMOUS SPLIT_TASK is owned by 4G; INTERACTIVE escalate is immediate. | P3E-E022
- **PHASE-008/IX-010** | E | 3E OutputReserve and floors protect model generation headroom used by all later agent invocations. | P3E-E013, P3E-E014, P3E-E025
- **PHASE-008/IX-011** | E | 4B tools run only through 4A IPC ToolRequest/ToolResult; isolation/sanitization owned by 4A, execution by 4B. | P4B-E001, P4B-E004, P4B-E021
- **PHASE-008/IX-012** | E | 4B IntegrationTestRunner blocked on 4E DecompositionTree types and matches 4E integration time budgets. | P4B-E003, P4B-E028
- **PHASE-008/IX-013** | E | 4B ShortCoder post-processes executor output before auditor; placement also referenced from 4C hooks. | P4B-E017
- **PHASE-008/IX-014** | E | 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral. | P4B-E018; P3C-E052
- **PHASE-008/IX-015** | E | 4B write tools honor SSB file locks (orchestrator concurrency invariant). | P4B-E008
- **PHASE-008/IX-016** | E | 4F reuses 2D FeasibilityOutcome and 4A Tier-C agent process with restricted AgentDispatchConfig. | P4F-E010, P4F-E016, P4F-E018
- **PHASE-008/IX-017** | E | 4F skip path PLANNING→TESTING_FIRST and novel_steps marking come from 4E structural validation. | P4F-E011, P4F-E012
- **PHASE-008/IX-018** | E | 4F decision Escalate vs Replan depends on execution mode introduced by 3A/4G. | P4F-E009
- **PHASE-008/IX-019** | E | 4F SSB branch tags compose with 3B SSB and 3D branch semantics (tag=branch:…). | P4F-E020
- **PHASE-008/IX-020** | E | 4F baby-step feasibility requires 4E DecompositionTree/BabyStepSpec and runs before concept execution in 4G. | P4F-E029–P4F-E032
- **PHASE-008/IX-021** | E | 4J event bridge writes Empire telemetry into 3B SSB for GRC grounding. | P4J-E016, P4J-E032
- **PHASE-008/IX-022** | E | 4J ElaiNeuron capability path reuses 4A IPC proxy + ToolFilter trust boundary. | P4J-E017, P4J-E019
- **PHASE-008/IX-023** | E | 4J repair pipelines are generated by 4D pipeline factory and validated with 4H TestOutcomeReport. | P4J-E018, P4J-E025
- **PHASE-008/IX-024** | E | 4J autonomous unattended Empire operation depends on 4G autonomous mode. | P4J-E001
- **PHASE-008/IX-025** | E | Index 2C role budgets are the same numbers 3C adversarial expects at spawn. | IDX-E019; P3C-E026
- **PHASE-008/IX-026** | E | Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase. | IDX-E023; P3C-E012
- **PHASE-008/IX-027** | E | Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs). | IDX-E011; P3E-E003
- **PHASE-008/IX-028** | E | Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts. | P3C-E014; P3E-E034; P4B-E034; P4F-E024
- **PHASE-008/IX-029** | E | RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J. | P4B-E032; P4F-E035; P4J-E037
- **PHASE-008/IX-030** | E | blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A). | P3C-E053; P4F-E005
- **PHASE-008/IX-031** | I | 4J Tauri/Svelte Empire panel will eventually sit beside 4K dashboard surfaces but 4K is out of this chunk. | P4J-E033
- **PHASE-008/IX-032** | E | 3C SubstituteModel is type-ready but gated until 4G; 4F/4B do not enable it. | P3C-E047

#### PHASE SD (256)

- **PHASE-001/SD-P0-001** | **ADD** full risk register R17–R25 with LOC/import facts and stale comparison-doc warning. | 01-phase:L24-L42 | P0-E004–P0-E012, P0-E068
- **PHASE-001/SD-P0-002** | **ADD** workspace path migration (rust/→crates/), archive semantics, strip-before-restructure rationale. | 01-phase:L24-L33 | P0-E013–P0-E014
- **PHASE-001/SD-P0-003** | **ADD** ONNX path, blake2 setup, DEGRADED BM25_ONLY, models/** hidden_paths. | 01-phase:L24-L42 | P0-E015–P0-E017
- **PHASE-001/SD-P0-004** | **ADD** ordered strip sequence, KEEP/STRIP slash lists, TokenUsage shim vs core fields. | 01-phase:L29-L32 | P0-E024–P0-E029, P0-C001
- **PHASE-001/SD-P0-005** | **ADD** module mapping LOCs, tools 2000-LOC cap, cli <500 soft, nine crates, contracts TDD red phase. | 01-phase:L31-L39 | P0-E032–P0-E048, P0-E066
- **PHASE-001/SD-P0-006** | **ADD** LlmProvider streaming complete, Ollama endpoints, allowlist sanitization, hidden_paths inventory, smoke skip. | 01-phase:L36-L44 | P0-E049–P0-E059
- **PHASE-001/SD-P0-007** | **ADD** ClockProvider/EntropySource shapes, RuntimeContext, CI exclusions, ELAI_SEED=42 dual-run proof. | 01-phase:L40-L41 | P0-E060–P0-E065
- **PHASE-001/SD-P0-008** | **FLAG** open prompt/remote/sandbox extraction choices omitted from summary. | 01-phase:L24-L42 | P0-A001–P0-A004
- **PHASE-001/SD-P1C-001** | **ADD** R22/R26/R27/R29 mitigations and HYBRID→SPARSE_ONLY→BM25_ONLY cascade. | 01-phase:L69-L83 | P1C-E003–P1C-E006, P1C-E012–P1C-E013
- **PHASE-001/SD-P1C-002** | **CORRECT** SPLADE deferral: original 1G vs summary 1H. | 01-phase:L74-L74 | P1C-E007, P1C-C002
- **PHASE-001/SD-P1C-003** | **ADD** TurboQuant grounded params and ban on 1C quantization. | 01-phase:L83-L83 | P1C-E015, P1C-E034
- **PHASE-001/SD-P1C-004** | **ADD** DualVector/OrtSession/UsearchIndex APIs, 512 truncation, filtered_search rebuild, RRF k=60 fixed. | 01-phase:L75-L81 | P1C-E017–P1C-E022, P1C-E030, P1C-E035
- **PHASE-001/SD-P1C-005** | **ADD** ≥22 tests, physical DB isolation, QueryTrace, knowledge_degradation triggers, write-before-query race rule. | 01-phase:L79-L81 | P1C-E024–P1C-E025, P1C-E032–P1C-E036
- **PHASE-001/SD-P1C-006** | **ADD** Step 3.5 tensor gate, Step 5 fields, cache_hit BLOCK <90%, path naming tension. | 01-phase:L80-L82 | P1C-E027, P1C-E037–P1C-E039, P1C-C001
- **PHASE-001/SD-P1C-007** | **QUALIFY** summary fastembed pin: original forbids adding fastembed with ort. | 01-phase:L74-L74 | P1C-E014
- **PHASE-001/SD-P3A-001** | **ADD** full transition matrix including CONFIGURING, FEASIBILITY revise, ReplanRequest, SKILL.md AUDITING arms + SPEC_UPDATE. | 01-phase:L233-L244 | P3A-E005–P3A-E008, P3A-C001
- **PHASE-001/SD-P3A-002** | **ADD** transition payload, token_budget_remaining, msgpack path, TransitionConflict, typed errors. | 01-phase:L237-L237 | P3A-E009–P3A-E014
- **PHASE-001/SD-P3A-003** | **ADD** full tool-filter table including FEASIBILITY stub-write and UNGROUNDED COMPLETE/IDLE. | 01-phase:L238-L239 | P3A-E015–P3A-E017
- **PHASE-001/SD-P3A-004** | **ADD** DAIRA max-3, EvidenceError shape, DEGRADED-allows-with-warning, FEASIBILITY ungated. | 01-phase:L240-L242 | P3A-E019–P3A-E023
- **PHASE-001/SD-P3A-005** | **ADD** structured EvidenceSummary, trait mockability, no-timeout default, risk examples + overrides. | 01-phase:L243-L244 | P3A-E024–P3A-E030
- **PHASE-001/SD-P3A-006** | **ADD** 81-pair tests, latency SLOs, SSB phase_transition non-deliverable note. | 01-phase:L233-L244 | P3A-E031–P3A-E033
- **PHASE-001/SD-P3JB-001** | **ADD entire file**: MASTER does not index phase-3J-BASELINE.txt. | 01-phase:L360-L375, L655 | P3JB-E001–P3JB-E019
- **PHASE-001/SD-P3JB-002** | **ADD** standalone stdlib harness, 8 tiers, 7 metrics, REPAIR_CONTEXT ±15%, JSON-schema 20-prompt, exit 0/1/2. | 01-phase:L655 | P3JB-E002–P3JB-E015
- **PHASE-001/SD-P3JB-003** | **ADD** non-overwrite run_benchmark.py, optional CI bench-capability, ShortCoder ban + output_token_ratio. | 01-phase:L655 | P3JB-E004, P3JB-E014, P3JB-E017
- **PHASE-001/SD-P4K-001** | **CORRECT/EXPAND** summary phase ribbon/commit history: MVP is project selector + task + plan tree + activity feed + diff viewer. | 01-phase:L528-L528 | P4K-E029–P4K-E053
- **PHASE-001/SD-P4K-002** | **ADD** full visual tokens, window geometry, identifier, Vite :1420, elai --dashboard single-instance focus. | 01-phase:L522-L528 | P4K-E006, P4K-E018, P4K-E026–P4K-E027
- **PHASE-001/SD-P4K-003** | **ADD** distinct Tauri commands including projects/templates/submit_task/get_action_detail. | 01-phase:L527-L527 | P4K-E019–P4K-E024, P4K-E033–P4K-E035, P4K-E053
- **PHASE-001/SD-P4K-004** | **ADD** intervention options, per-hunk controls, multi-file nav, preview open, 500-entry cap, in-flight poll skip, 200-char truncate. | 01-phase:L525-L526 | P4K-E013–P4K-E014, P4K-E040–P4K-E055
- **PHASE-001/SD-P4K-005** | **ADD** project name validation, min 5-char task, projects.json corrupt empty-list, post-MVP deferred list. | 01-phase:L528-L529 | P4K-E033, P4K-E036, P4K-E057
- **PHASE-001/SD-P4K-006** | **ADD** WebView2 bootstrapper, <100KB JS, typed schema drift compile-fail, graceful open-fail dialog. | 01-phase:L529-L529 | P4K-E007–P4K-E010, P4K-E028
- **PHASE-001/SD-P4M-001** | **ADD** summary stops at 4M.1; original continues 4M.2–4M.6 adapters/hybrid/CLI/dashboard/metrics/migration. | 01-phase:L531-L541, L653 | P4M-E014–P4M-E031
- **PHASE-001/SD-P4M-002** | **ADD** PermissionMode defaults, ResolveBackendError, capabilities fields, factory timeouts. | 01-phase:L537-L539 | P4M-E006–P4M-E009
- **PHASE-001/SD-P4M-003** | **ADD** default per-role map, runtime vs model fallback order, ESCALATE/HALT, per-session state. | 01-phase:L539-L539 | P4M-E021–P4M-E027
- **PHASE-001/SD-P4M-004** | **ADD** /runtime show | set | fallback and RuntimePanel Test-backend canary.
- **PHASE-001/SD-P4M-005** | **ADD** Tier C mandate, stdout sanitization, missing-binary Unavailable, exit_code fail-closed, RuntimeClient wiring. | 01-phase:L531-L541 | P4M-E018–P4M-E019, P4M-E031
- **PHASE-001/SD-P4M-006** | **ADD** [backend]→[runtime] deprecation, phase_4M.toml metrics, ouroboros grounding distinctions, R1–R4. | 01-phase:L533-L541 | P4M-E015, P4M-E029–P4M-E033
- **PHASE-002/SD-1A-001** | **ADD** full COMPLIANCE SCOPE split of knowledge.txt Sec 8 items 1+5 vs deferred 2–4 and orphan-invariant block until 1B. | MASTER:L44-L55 | P1A-E009, P1A-E010, P1A-E043
- **PHASE-002/SD-1A-002** | **ADD** readiness split dkg_readiness vs full KnowledgeEngine.readiness_check (Phase 1I). | MASTER:L48-L51 | P1A-E016, P1A-E017
- **PHASE-002/SD-1A-003** | **ADD** complete schema tables dkg_files/dkg_meta, directed edge semantics, WIRING_CONTRACT symbols, index build SSB symbol_graph_updated, and 12 metric keys. | MASTER:L50-L55 | P1A-E022–P1A-E027, P1A-E038–P1A-E041
- **PHASE-002/SD-1A-004** | **ADD** TDD red gate (≥20 tests, ≥10 fail), object-safety LanguageGrammar, and dynamic KNOW_PATH resolution. | MASTER:L48-L53 | P1A-E002, P1A-E011, P1A-E034
- **PHASE-002/SD-1A-005** | **FLAG** ungrounded <15s build claim and designed-not-spec SQLite schema. | MASTER:L44-L55 | P1A-E006, P1A-E007
- **PHASE-002/SD-1D-001** | **ADD** full T1–T18/P1–P3/A1–A3 catalog and make_snapshot helper requirements. | MASTER:L93-L93 | P1D-E010–P1D-E014
- **PHASE-002/SD-1D-002** | **ADD** exact 13-item special-file list and config-driven deferral; qualify MASTER “such as” wording. | MASTER:L96-L96 | P1D-E016, P1D-A001
- **PHASE-002/SD-1D-003** | **ADD** R20 ID collision note and Layer 3.5 vs Layer 4 correction. | MASTER:L85-L97 | P1D-E003, P1D-C001
- **PHASE-002/SD-1D-004** | **ADD** aider not-adopted items (specificity/private/generic penalties) and 7-metric phase_1D.toml fields. | MASTER:L88-L97 | P1D-E019, P1D-E020
- **PHASE-002/SD-1D-005** | **ADD** zero-budget T13, NaN/Inf clamp, and initial binary-search guess max_tokens/25. | MASTER:L94-L96 | P1D-E015, P1D-E017
- **PHASE-002/SD-3D-001** | **ADD** four pre-implement orchestrator.txt amendments and corrected loop rationale (checkpoint after gate). | MASTER:L284-L288 | P3D-E007, P3D-E014
- **PHASE-002/SD-3D-002** | **ADD** ephemeral clear SSB replay algorithm and Topic(false) two-phase snapshot+clear definition (spec gap). | MASTER:L281-L282, L287 | P3D-E009, P3D-E010, P3D-E018
- **PHASE-002/SD-3D-003** | **ADD** DeepSizeOf/size_of_val ban, 10MB/2× force-clear, and isolation_metric separation from memory_monitoring. | MASTER:L283-L287 | P3D-E004, P3D-E017, P3D-E022
- **PHASE-002/SD-3D-004** | **ADD** wall-time concurrency proof, VRAM max_concurrent formula, and simulation-only byte-identical determinism caveat for live LLMs. | MASTER:L277-L288 | P3D-E003, P3D-E019
- **PHASE-002/SD-3D-005** | **ADD** baseline measurement section as non-exit UNGROUNDED estimates. | MASTER:L276-L288 | P3D-E020
- **PHASE-002/SD-3G-001** | **ADD** 3E dependency correction for 3G.4 and partial-parallel recommendation. | MASTER:L321-L332 | P3G-E001, P3G-E002
- **PHASE-002/SD-3G-002** | **ADD** visual_check/output_type media adapter rules and ClockProvider timestamp ungrounded fields. | MASTER:L325-L325 | P3G-E004, P3G-E005
- **PHASE-002/SD-3G-003** | **ADD** 3600 safety margin, SkeletonBudgetExceeded, ContractConflict, and empty declared_interfaces fail-closed. | MASTER:L327-L330 | P3G-E008, P3G-E017, P3G-E023
- **PHASE-002/SD-3G-004** | **ADD** split_phase preserve_contracts algorithm details and u32 checked_add overflow Err. | MASTER:L331-L331 | P3G-E014, P3G-E018
- **PHASE-002/SD-3G-005** | **ADD** 100-run determinism with EntropyProvider pipeline_id and ban wall clocks in plan_skeleton. | MASTER:L332-L332 | P3G-E019
- **PHASE-002/SD-4DOR-001** | **ADD** entire Oracle cache feature set: MASTER lists phase-4D-ORACLE only as unindexed file to inspect, without extracting atoms. | MASTER:L655-L655, L714-L714 | P4DOR-E001–P4DOR-E017
- **PHASE-002/SD-4DOR-002** | **ADD** TTL/confidence mapping, LRU default 128, /research+/ask reachability, blake3 ban, C-30 SCHEMA 62 events. | MASTER:L655 | P4DOR-E006–P4DOR-E015
- **PHASE-002/SD-4DOR-003** | **ADD** LiveCli lazy Option init and T4Rejected early-return stub_ban. | MASTER:L655 | P4DOR-E011, P4DOR-E014
- **PHASE-002/SD-4DOR-004** | **FLAG** query normalization and expired-entry lazy eviction ambiguities absent from any summary. | MASTER:L655 | P4DOR-A001, P4DOR-A002
- **PHASE-002/SD-4G-001** | **ADD** full EvidenceCheck/EvidenceVerdict types and TESTING_FIRST cargo heuristic caveats. | MASTER:L471-L471 | P4G-E005, P4G-E006, P4G-A002
- **PHASE-002/SD-4G-002** | **ADD** 28+8 test catalog, InterventionCard exhaustion UI, and mode-switch TOCTOU RwLock rules. | MASTER:L476-L476 | P4G-E015, P4G-E028, P4G-E035
- **PHASE-002/SD-4G-003** | **ADD** complete 4G.9 stagnation/persona/watchdog/goal-drift port (thresholds, SSB types, affinity, fingerprint, Jaccard bands). | MASTER:L477-L477 (compressed tree only) | P4G-E021–P4G-E027, P4G-E034
- **PHASE-002/SD-4G-004** | **ADD** shipped addendum: reasoning-exhaustion 7th auto-advance check and CloudSpendTracker budget policy (default $5). | MASTER:L467-L478 | P4G-E031, P4G-E032
- **PHASE-002/SD-4G-005** | **ADD** required new APIs can_fit_llm, get_role, SSB escalation type and SessionState open. | MASTER:L475-L475 | P4G-E011, P4G-E013, P4G-E014, P4G-A001
- **PHASE-002/SD-4G-006** | **QUALIFY** MASTER runtime UX (--local/--hybrid, /model, backend config): more explicit in summary than phase body; keep as orientation/I. | MASTER:L478-L478 | P4G-I001, AC-014
- **PHASE-002/SD-4G-007** | **ADD** anti-spiral pin L180 and HIGH-risk dual-path clarification vs auditor tool review. | MASTER:L472-L472 | P4G-C001, P4G-C002
- **PHASE-003/SD-1B-001** | **ADD** full API surface IndexStats/BM25Hit snippet policy, single-writer batch lock, and explicit project_id rule. | MASTER:L57-L67 | P1B-E006-E013
- **PHASE-003/SD-1B-002** | **ADD** WIRING_CONTRACT CLI `index build` + SSB `index_checkpoint` + call sites. | MASTER:L57-L67 | P1B-E018-E019
- **PHASE-003/SD-1B-003** | **ADD** environment-aware CI 50 ms vs dev 10 ms gates and warm-up measurement. | MASTER:L67 | P1B-E017
- **PHASE-003/SD-1B-004** | **ADD/CORRECT** tantivy_v21 vs INDEX_FORMAT_VERSION and DkgNode-vs-BM25Chunk ownership. | MASTER:L61-L63 | P1B-E004-E006, P1B-C001
- **PHASE-003/SD-1B-005** | **ADD** 1I scoping-out of readiness/breaker/versioning from 1B. | MASTER:L67 | P1B-E020
- **PHASE-003/SD-1G-001** | **ADD** full RerankerConfig field set, mpsc batching design, and oneshot offset split. | MASTER:L123-L133 | P1G-E004-E008
- **PHASE-003/SD-1G-002** | **ADD** L-6 amendment path, cold-start SLA exception, and canary DEGRADED semantics. | MASTER:L129-L132 | P1G-E010-E014, P1G-C001
- **PHASE-003/SD-1G-003** | **ADD** INT8 trigger p50>30 ms / NDCG delta <0.5 and quantization script path. | MASTER:L127, L133 | P1G-E015
- **PHASE-003/SD-1G-004** | **ADD** complete metrics list including queue wait p99 and amendment-on-fail-50ms rule. | MASTER:L133 | P1G-E017-E018
- **PHASE-003/SD-1G-005** | **ADD** 1F DROP independence note (MASTER has 1F section; 1G body reconfirms). | MASTER:L114-L121 | P1G-E001
- **PHASE-003/SD-3H-001** | **ADD** full SQL column/index set, types, and DELETE-journal/FK/integrity_check safety. | MASTER:L334-L344 | P3H-E003-E008
- **PHASE-003/SD-3H-002** | **EXPAND** three amendments (schema/event/cleanup) and Conflict/blake3 open items beyond MASTER [Q] one-liner. | MASTER:L337 | P3H-A001-A005, P3H-C001
- **PHASE-003/SD-3H-003** | **ADD** recovery APIs, typed StubRecoveryFailed, and 100-run determinism/measure targets. | MASTER:L343-L344 | P3H-E014-E017
- **PHASE-003/SD-3H-004** | **ADD** branch naming rules, close_branch, and immutable trunk-extension merge semantics detail. | MASTER:L342 | P3H-E012
- **PHASE-003/SD-4DS-001** | **ADD** entire C-30 surgical path: MASTER only flags file as unindexed; full CLI/SSB/drawer/LOC/complexity atoms are deep-only. | MASTER:L655 | P4DS-E001-E015
- **PHASE-003/SD-4DS-002** | **ADD** intercept-before-research runner semantics and DkgQuerierAdapter stub ban. | MASTER:L655 | P4DS-E003, P4DS-E007, P4DS-E013
- **PHASE-003/SD-4E-001** | **ADD** full type inventory, nine builder stubs, and 8 ValidationErrorKind list. | MASTER:L442-L455 | P4E-E003-E008
- **PHASE-003/SD-4E-002** | **ADD** Phase Sizing formula constants, fixpoint hash escalate, REORDER-before-SUBSTITUTE, RuntimeClient wiring rule. | MASTER:L445-L449 | P4E-E007, P4E-E009-E010, P4E-E013
- **PHASE-003/SD-4E-003** | **ADD** complete baby-step 8-stage cycle, BubbleUpPolicy defaults, SSB replan_request, msgpack recovery. | MASTER:L451-L455 | P4E-E014-E023
- **PHASE-003/SD-4E-004** | **ADD** test catalog counts (≥30+10+33) and 4E.12 context p95/overhead gates. | MASTER:L450-L455 | P4E-E011, P4E-E024-E025
- **PHASE-003/SD-4E-005** | **FLAG** enum name CycleDetected vs CyclicDependency and weakest-dep open. | MASTER:L446-L448 | P4E-A001, P4E-A003
- **PHASE-003/SD-IDX-001** | **ADD** this file is a second index surface parallel to MASTER; captures 3J wire substeps, `/model`, backend modes, 4H MSI, Empire caps not always expanded in MASTER bullets. | MASTER:L334-L478 region | PIDX-E012-E013, PIDX-E021-E024
- **PHASE-003/SD-IDX-002** | **QUALIFY** index is not authoritative over deep phase files when they conflict (e.g. surgical absent here). | MASTER:L1-L8 | PIDX-I001, PIDX-A001, AC-014
- **PHASE-003/SD-IDX-003** | **ADD** 3K FAST/STANDARD/DEEP tier routing and 4B IntegrationTestRunner ladder as index-compressed features. | MASTER:L399, retrieval ladder | PIDX-E011, PIDX-E015
- **PHASE-004/SD-1E-001** | **ADD** full risk set R30–R34 with mitigation numbers (N_seed, K_edge, density 0.01, max_loops, ±2 lines, 2s budget). | MASTER:L99-L112 | P1E-E007
- **PHASE-004/SD-1E-002** | **ADD** sparse CatRagOverlay type fields, QueryPipeline/PipelineStage schema, and measure_1e five-query catalog Q1–Q5 + phase_1E.toml keys. | MASTER:L103-L112 | P1E-E009, P1E-E015, P1E-E028
- **PHASE-004/SD-1E-003** | **ADD** deterministic 1E A-RAG fixed tool order vs deferred ReAct Phase 2+; stage failure fall-through; RRF rank-from-1. | MASTER:L111 | P1E-E025, P1E-E027
- **PHASE-004/SD-1E-004** | **ADD** match-cap 2*N_seed and >50% DKG match warning; optional LLM edge scoring via breaker only. | MASTER:L107 | P1E-E020, P1E-E021
- **PHASE-004/SD-1E-005** | **FLAG** sub-second vs 2s acceptance tension absent from summary compression. | MASTER:L100, L112 | P1E-C001
- **PHASE-004/SD-2B-001** | **ADD** PlanGroundingResult verify_needed semantics (training-only subset of ungrounded) and empty-plan trivially grounded. | MASTER:L194 | P2B-E008, P2B-E020
- **PHASE-004/SD-2B-002** | **ADD** complete prefix dispatch table including grep→FileCheck and tool→SourceCheck; 30s cargo test timeout; BashSandbox API. | MASTER:L198-L201 | P2B-E017-P2B-E019
- **PHASE-004/SD-2B-003** | **ADD** full T1–T16 catalog, network/timing bans, tempfile cleanup, and dependency gate (no knowledge in cargo tree). | MASTER:L197, L196 | P2B-E015, P2B-E016, P2B-E013
- **PHASE-004/SD-2B-004** | **ADD** Uncertain auto-resolution during think() enhancement of 2A; 4A wrap-without-signature-change constraint. | MASTER:L191, L200 | P2B-E004, P2B-E005
- **PHASE-004/SD-2B-005** | **QUALIFY** summary network stub string vs source "not implemented in phase 2B" — behavioral typed error is the requirement. | MASTER:L200 | P2B-C001
- **PHASE-004/SD-2C-001** | **ADD** complete T1–T15 test catalog and role budgets (executor 8/1200, auditor 10/1200, memory_writer 2/400, prompt_engineer 5/800, designer distinct). | MASTER:L209, L215 | P2C-E011, P2C-E018
- **PHASE-004/SD-2C-002** | **ADD** whitelist formula `(base ∪ overrides) ∩ (required_steps ∪ {Status,Converge})` and with_agent_config topic reset. | MASTER:L216 | P2C-E012, P2C-E014
- **PHASE-004/SD-2C-003** | **ADD** FINDING_PREFIX contract, scope enumeration WARN path, and five validation rules D2 including csq_required. | MASTER:L211-L212 | P2C-E005, P2C-E007, P2C-E008
- **PHASE-004/SD-2C-004** | **ADD** full SPEC GAPS list (8-field type, GrcError variants, PartialOrd order, rubber-stamp invariant, methods vs quality complementarity). | MASTER:L207-L218 | P2C-E020, P2C-C001
- **PHASE-004/SD-2C-005** | **ADD** metrics memory <2KB and malformed-table fail vs missing-table default distinction. | MASTER:L218 | P2C-E016, P2C-E014
- **PHASE-004/SD-3L-001** | **ADD** entire Phase 3L feature set: MASTER only notes phase-3L.txt as unindexed file to inspect (Q-line), without atom extraction. | MASTER:L655, L713 | P3L-E001–P3L-E021
- **PHASE-004/SD-3L-002** | **ADD** Pi-prefixed hook naming, LazyLock D2, conventions_migration_v1 flat path, seed_rules API, profile.toml + /profile. | MASTER:L655 | P3L-E003–P3L-E008
- **PHASE-004/SD-3L-003** | **ADD** telescope priorities 100/80/60, TELESCOPE_MAX_CHARS 1200, pi_telescope_contract five invariants, calibration gates 50/100 ms. | MASTER:L655 | P3L-E015–P3L-E017
- **PHASE-004/SD-3L-004** | **ADD** four-language determinism matrix and CONVENTIONS prompt block budget 200 tokens. | MASTER:L655 | P3L-E010, P3L-E012
- **PHASE-004/SD-3L-005** | **ADD** 3K predecessor correction (not Tier0 scaffold) and reuse of plugins HookVerdict without forking 4C tiers. | MASTER:L655 | P3L-E002, P3L-E003
- **PHASE-004/SD-4C-001** | **ADD** complete 30+10 test catalogs, cross-session locks, SCHEMA_REQUIRED Escalate, ShortCoder AST-equivalent tests. | MASTER:L421 | P4C-E011, P4C-E012
- **PHASE-004/SD-4C-002** | **ADD** PreCompact 0.85 handoff threshold, ruff >10000 breaker, T2 >5000 breaker, BSP cache invalidation timing. | MASTER:L413-L418 | P4C-E003, P4C-E007, P4C-E005
- **PHASE-004/SD-4C-003** | **ADD** exact sanitization string "Tool call rejected by policy", plugins dependency direction, RuntimeClient ssb/fsm/clock rule. | MASTER:L419 | P4C-E004, P4C-E014, P4C-E016
- **PHASE-004/SD-4C-004** | **ADD** 4A WARN-1/WARN-6 mitigations and metrics nine categories with coverage >80%. | MASTER:L422 | P4C-E017, P4C-E015
- **PHASE-004/SD-4C-005** | **ADD** deferred skills/MCP rationale (no 4D–4G dependency on 4C for those). | MASTER:L423 | P4C-E002
- **PHASE-004/SD-4I-001** | **ADD** full veto matrix including schema-invalid Degraded and VLM-unavailable Degraded; Mode A post-validation field checks + retry. | MASTER:L506-L507 | P4I-E007, P4I-E011
- **PHASE-004/SD-4I-002** | **ADD** complete adversarial set (injection, 500MB, headers, path/symlink, sandbox, queue 65, ComfyUI 600s, CLAP noise). | MASTER:L496-L507 | P4I-E009
- **PHASE-004/SD-4I-003** | **ADD** full RCE type system (MediaRenderAdapter, five adapters, VisualFeedbackLoop, VisualIssue, convergence by severity/category/location). | MASTER:L505 | P4I-E015–P4I-E017
- **PHASE-004/SD-4I-004** | **ADD** SSB visual_feedback liveness, lazy register_lazy factory, ToolCapabilityFlags, MockVlm Phase0, ollama ≥0.12.7. | MASTER:L498-L502 | P4I-E012, P4I-E004, P4I-E005, P4I-E018
- **PHASE-004/SD-4I-005** | **ADD** RCE latency targets (web <3s, Blender <10s, loop web <10s/iter), convergence ≥70% measure, total ≥43 tests, Phase 4 overall exit block. | MASTER:L506-L507 | P4I-E014, P4I-E008, P4I-E022
- **PHASE-004/SD-4I-006** | **ADD** stale architecture-overview LLaVA note and MoE VRAM clarification conflict with media-validator L60. | MASTER:L501-L502 | P4I-E021, P4I-C001
- **PHASE-004/SD-4I-007** | **QUALIFY** summary "bounded N" iterations: source default max_iterations=3 with early stop on unchanged issues. | MASTER:L505 | P4I-E017
- **PHASE-005/SD-1F-001** | **ADD** full RESOLVED timeline: ollama 0.20.2 NOT_AVAILABLE, PR #8134 never merged, consolidated phase_1F.toml decision=UNAVAILABLE, EXIT satisfied via DROP. | MASTER:L114-L117 | P1F-E002–P1F-E006, P1F-E015
- **PHASE-005/SD-1F-002** | **ADD** complete decision gate order, boundary DROP at ==1.3x/3000MB, prompt-lookup preference, acceptance early kill, k-sweep, GPU temp rules, marginal retest. | MASTER:L120-L120 | P1F-E020, P1F-E032–P1F-E037, P1F-E041
- **PHASE-005/SD-1F-003** | **ADD** hardware scenarios A/B remaining VRAM, nvidia-smi polling, Phase-0-hardware-only ADOPT caveat, TIDE conditional later opt. | MASTER:L121-L121 | P1F-E010-P1F-E011, P1F-E040, P1F-E043
- **PHASE-005/SD-1F-004** | **ADD** adversarial serde/num_draft_tokens=0, silent-ignore false positive, cache uniqueness, and all metrics file paths. | MASTER:L118-L120 | P1F-E022-P1F-E031, P1F-E039
- **PHASE-005/SD-1F-005** | **QUALIFY** "Phase 2+" revisit as post-Phase-1/vLLM (3J), not a Phase-2 numbered dependency. | MASTER:L117, L659 | P1F-I001, P1F-E004
- **PHASE-005/SD-2A-001** | **ADD** design D1-D5 paper grounding (BudgetThinker, RARR/SAFE, DSPy Assert/Suggest) and check-then-increment rationale. | MASTER:L172-L186 | P2A-E005-P2A-E009, P2A-E027
- **PHASE-005/SD-2A-002** | **ADD** full T1-T12 catalog notes including max_calls=3 trick, free Status, default 5/800, scope three-way tests. | MASTER:L182-L182 | P2A-E020-P2A-E023
- **PHASE-005/SD-2A-003** | **ADD** valid grounding type list and level mapping including tool→High and search→Verified. | MASTER:L184-L184 | P2A-E025, P2A-E026
- **PHASE-005/SD-2A-004** | **ADD** metrics methodology (1000-iter median, no criterion without approval) and reset_topic full removal. | MASTER:L186-L186 | P2A-E028, P2A-E030
- **PHASE-005/SD-2A-005** | **FLAG** T11 Err-vs-warn, incomplete allowed_next table, and "every step grounding" wording conflict. | MASTER:L172-L186 | P2A-A001-P2A-A002, P2A-C001
- **PHASE-005/SD-3F-001** | **ADD** WARN preservation with BLOCK, severity DESC within type, and evaluation-order rationale. | MASTER:L305-L306 | P3F-E007, P3F-E008
- **PHASE-005/SD-3F-002** | **ADD** full transition thresholds, mutable=0 dual invariants, transition log fields, insufficient-shadow-data error. | MASTER:L311-L313 | P3F-E018-P3F-E023
- **PHASE-005/SD-3F-003** | **ADD** PostToolUse seed GREP surfaces (img/table/TODO strings), merge override rules, seed contract-test minimums. | MASTER:L317-L317 | P3F-E036-P3F-E038
- **PHASE-005/SD-3F-004** | **ADD** proptest De Morgan/associativity, READ-LOCK on contract_tests, COMPOSITE depth 8 UNGROUNDED note, counterfactual runtime deferral. | MASTER:L318-L319 | P3F-E039-P3F-E042, P3F-E044
- **PHASE-005/SD-3F-005** | **CORRECT/QUALIFY** MASTER SCHEMA_REQUIRED "escalates provider tier" and NaN/Inf guard: not stated in phase-3F body (schema validate + metric ops only). | MASTER:L306, L314 | P3F-I001, AC-014, AC-015
- **PHASE-005/SD-3F-006** | **ADD** AstEvaluator injection option (b) recommended, domain_mapping config, scoring [0,15], blake2s discovery. | MASTER:L303-L316 | P3F-E006, P3F-E029, P3F-E032, P3F-E034
- **PHASE-005/SD-3J-001** | **CORRECT** step numbering: multiplex+preload are 3J.6; profiles 3J.7; MILP 3J.8; MCP 3J.9; hooks 3J.10; AppContainer 3J.11; SSB 3J.12; verify 3J.13; E2E 3J.14; 31B 3J.15. | MASTER:L370-L374 | P3J-C001, P3J-C002, P3J-E020-P3J-E063
- **PHASE-005/SD-3J-002** | **CORRECT** 3J.6 API is can_colocate + MultiplexingPolicy fields, not can_fit_onnx in this file. | MASTER:L370 | P3J-E020, P3J-I002
- **PHASE-005/SD-3J-003** | **ADD** complete user-facing surface set: --local, elai.bat, /mode,/model,/plan,/resume,/session fork,/compact, InteractiveGate four choices, degraded prefix, startup banner. | MASTER:L675-L681 | P3J-E038, P3J-E042, P3J-E045, P3J-E050, P3J-E052, P3J-E058, P3J-E061, P3J-E068
- **PHASE-005/SD-3J-004** | **ADD** full WIRE.* chain FOUNDATION→BUDGET→RULES→SESSION→PLAN→STUBS and UNIFIED RuntimeClient constraints. | MASTER:L675-L681 | P3J-E040-P3J-E055
- **PHASE-005/SD-3J-005** | **ADD** MCP security/retry, hook sanitization string, AppContainer job limits/env allowlist, isolation_metric formula, SSB ≥17 write-time validation. | MASTER:L373-L373 | P3J-E025-P3J-E034
- **PHASE-005/SD-3J-006** | **ADD** 3J.14 Ollama fallback re-probe, QueryPipeline Slot 4, BM25 update-on-edit, 60s REPL SLO, release binary checks. | MASTER:L675-L681 | P3J-E057-P3J-E062
- **PHASE-005/SD-3J-007** | **ADD** 3J.15 31B buckets, EAGLE-3 skip reason, optional 100B SUBSTITUTE_MODEL gate, metrics file names. | MASTER:L374 | P3J-E063, P3J-E064
- **PHASE-005/SD-3J-008** | **ADD** entire 3J.ADDENDUM shipped extensions A-8/9/10/17/25/26/27/28/31 with file paths. | MASTER:L360-L375 (absent) | P3J-E066, P3J-E067
- **PHASE-005/SD-3J-009** | **ADD** hardware/model install snapshot (vLLM 0.19.0 WSL2, gemma4 sizes, native vision) and R1-R11 risk register. | MASTER:L363-L364 | P3J-E003-P3J-E008
- **PHASE-005/SD-3J-010** | **FLAG** undefined 3J.COMPOSE/LOCAL steps, TBD contrast pairs, NF4 detail, hybrid mode history. | MASTER:L675-L681 | P3J-I001, P3J-A002-P3J-A004, AC-011-AC-012
- **PHASE-006/SD-1H-001** | **ADD** full risk register R24–R30 including air-gap ort download, tokenizers separate crate, and statistical power of 10 vs 50 queries. | MASTER:L135-L143 | P1H-E007–P1H-E009
- **PHASE-006/SD-1H-002** | **ADD** PrePooled vs RawMLM ONNX output shapes, RRF k=60 0.5/0.5 weights, circuit-breaker empty-expansion fallback, and SPLADE_MODEL_PATH default. | MASTER:L141 | P1H-E016–P1H-E021
- **PHASE-006/SD-1H-003** | **ADD** early-drop 0.2–0.4 calibration note, quant Jaccard≥0.95 prefer int8, cross-model within-1% pick-faster, Wilcoxon paired protocol, and ≥20 non-zero pairs power gate. | MASTER:L142-L143 | P1H-E022–P1H-E026, P1H-A001
- **PHASE-006/SD-1H-004** | **ADD** SPARSE_ONLY→BM25_ONLY cascade verification when splade disabled and full measurement field list including binary_size_delta_mb. | MASTER:L143 | P1H-E028–P1H-E029
- **PHASE-006/SD-1H-005** | **ADD** knowledge.txt typo/lock note and UNGROUNDED code-search recall labeling. | MASTER:L135-L143 | P1H-E006, P1H-E010
- **PHASE-006/SD-3I-001** | **ADD** full VCR cassette design (blake2s HTTP body hash, msgpack, prompt-drift fail) and TraceOps trajectory layer. | MASTER:L349 | P3I-E004–P3I-E006
- **PHASE-006/SD-3I-002** | **ADD** custom SQLite VFS details, madsim timer coordination, and spike-to-possibly-skip VFS. | MASTER:L349 | P3I-E007–P3I-E008, P3I-A001
- **PHASE-006/SD-3I-003** | **ADD** complete FsmEvent ten-name list, SuperstepBoundary atomic substeps, AgentCleanup lifecycle, and by-value SUT apply semantics. | MASTER:L351 | P3I-E012–P3I-E014
- **PHASE-006/SD-3I-004** | **ADD** R1–R5 risk block, PROPTEST_CASES 10000/256 split, bare-fn check_determinism constraint, and proptest-regressions synthetic seed. | MASTER:L350-L354 | P3I-E009, P3I-E017
- **PHASE-006/SD-3I-005** | **ADD** entropy-quarantine BLOCK PRE_GATE, burst-loss 5×100%, whitespace anti-spiral, zero-byte evidence rejection, and ops/sec ungrounded 100K target. | MASTER:L355-L358 | P3I-E016, P3I-E018–P3I-E021
- **PHASE-006/SD-3I-006** | **ADD** SPEC_GAP that orchestrator.txt has no DST section; design lives in external-analysis. | MASTER:L346-L358 | P3I-E023
- **PHASE-006/SD-3I-007** | **ADD** exit note AUTONOMOUS not enabled (Phase 4G) and HITL walking-skeleton is multi-phase. | MASTER:L358 | P3I-E022
- **PHASE-006/SD-3K-001** | **ADD** current-stack inventory (1A–1I) and explicit missing list before steps. | MASTER:L159-L161 | P3K-E003
- **PHASE-006/SD-3K-002** | **ADD** chunk metadata fields, Tantivy schema extension, format-version reindex migration, and 1000-token hard cap check. | MASTER:L162 | P3K-E005–P3K-E007
- **PHASE-006/SD-3K-003** | **ADD** exact HyDE sidecar prompt/max_tokens/temperature, 3s timeout, 5-minute cache, and rerank-against-original adversarial mitigation. | MASTER:L164 | P3K-E010–P3K-E011
- **PHASE-006/SD-3K-004** | **ADD** per-tier latency budgets FAST<20ms STANDARD<200ms DEEP<5s and AUTO word-count rules. | MASTER:L165 | P3K-E012–P3K-E013
- **PHASE-006/SD-3K-005** | **ADD** A-RAG Score_kw formula, 3× oversample MAX grouping, progressive disclosure ≥50% tokens, and length-domination adversarial. | MASTER:L167 | P3K-E015–P3K-E018
- **PHASE-006/SD-3K-006** | **ADD** epsilon/beta formulas, entity-extraction 0-entity adversarial, beta dominance adversarial, and post-improvement tokens ≥30% reduction criterion. | MASTER:L168-L170 | P3K-E019–P3K-E022
- **PHASE-006/SD-3K-007** | **QUALIFY** paper list and independence from 4D web research. | MASTER:L159-L170 | P3K-E015
- **PHASE-006/SD-4A-001** | **ADD** per-type frame size caps, postcard version framing details, and exactly-15 const_assert. | MASTER:L380-L381 | P4A-E013–P4A-E015
- **PHASE-006/SD-4A-002** | **ADD** full listen-before-spawn sequence, UUID v4 agent_id, and version-handshake kill. | MASTER:L385 | P4A-E019–P4A-E020
- **PHASE-006/SD-4A-003** | **ADD** complete functional isolation test battery including env sanitization, /proc/sys masks, RLIMIT behaviors, concurrent agent isolation, and role-method grounding matrix. | MASTER:L392 | P4A-E021–P4A-E025
- **PHASE-006/SD-4A-004** | **ADD** full adversarial IPC/replan/self-mod/crash/Windows test inventory and WARN/BLOCK coverage requirement. | MASTER:L389 | P4A-E026–P4A-E031
- **PHASE-006/SD-4A-005** | **ADD** Linux clone flags, rlimits defaults, heartbeat 5s/15s, and Windows Job object numbers. | MASTER:L382-L388 | P4A-E032–P4A-E034
- **PHASE-006/SD-4A-006** | **ADD** phase_complete SSB fields and RuntimeClient wiring exit rule. | MASTER:L391 | P4A-E018, P4A-E038
- **PHASE-006/SD-4A-007** | **ADD** separation-cost planning numbers (0.1ms/50ms/20MB) vs SLO targets. | MASTER:L392 | P4A-E036–P4A-E037, P4A-A001
- **PHASE-006/SD-4A-008** | **QUALIFY** feasibility GRC 3/600 stub-write/bash timeouts appear in MASTER dispatch bullet but are not restated as 4A step numbers in phase-4A.txt (they live in later 4F); do not invent them as 4A.x steps. | MASTER:L390 | P4A-E017
- **PHASE-006/SD-4H-001** | **ADD** revision history 2026-04-15 audit resolution (4 BLOCK/8 WARN), paper refs DGM/DarwinNet/SWE-World, and code-vs-config safety divergence. | MASTER:L480-L482 | P4H-E001–P4H-E003, P4H-C001
- **PHASE-006/SD-4H-002** | **ADD** complete PipelineOutcome field set including ETC/staleness/truncation/miss vectors and CatchLayer enum. | MASTER:L486-L488 | P4H-E007–P4H-E010
- **PHASE-006/SD-4H-003** | **ADD** full AllowedConfigKey Parameter/DataTable/LOCKED lists including cloud spend locks and Freshness* keys. | MASTER:L493 | P4H-E015–P4H-E017
- **PHASE-006/SD-4H-004** | **ADD** DataTableValidation FPR defaults, ProjectScope, ReasoningTraceRef, ProposedChange variants, and StructuralChange type set. | MASTER:L479-L493 | P4H-E018–P4H-E026
- **PHASE-006/SD-4H-005** | **ADD** ouroboros OSC/REG/EVO algorithms, SSB action types, exploration boost, key freeze, and MSI force-STABILIZING on Withhold. | MASTER:L480-L494 | P4H-E022–P4H-E025, P4H-E040–P4H-E042, P4H-E045
- **PHASE-006/SD-4H-006** | **ADD** staged-eval archive handling differences Stage1 discard vs Stage2/3 retain, trigger_task.toml schema, and mode-aware evaluation. | MASTER:L484 | P4H-E034–P4H-E036
- **PHASE-006/SD-4H-007** | **ADD** Layer1A four growth paths with numeric thresholds (3 catches, FPR 0.05, domain 5+, verified 0.8) and no-staged-eval path. | MASTER:L483 | P4H-E031–P4H-E033
- **PHASE-006/SD-4H-008** | **ADD** MSI frequency schedules (1/3 and 1/5 pipelines), per-mode pause, anti-fragility N=10 recovery bound, and exponential weight series. | MASTER:L491-L492 | P4H-E039
- **PHASE-006/SD-4H-009** | **ADD** anti-staleness `elai dep-audit`, reasoning fix-template loop, token-efficiency slot proposals, and optional FM cross-transfer. | MASTER:L494 | P4H-E043–P4H-E046
- **PHASE-006/SD-4H-010** | **ADD** conditional learned-reward EVALUATE section with domain-mismatch warnings and R53. | MASTER:L480-L494 | P4H-E047
- **PHASE-006/SD-4H-011** | **ADD** four-layer enforcement detail, lock-list unreadability, objective-hacking trigger, and FPR threshold lock. | MASTER:L494 | P4H-E027, P4H-E038
- **PHASE-007/SD-PP0-001** | **ADD** exact file/line sites for PermissionsExt and set_mode (7 sites / 3 files) and Windows no-op chmod pattern. | 01-phase:L13-L14 | PP0-E006–PP0-E008
- **PHASE-007/SD-PP0-002** | **ADD** named failing plugin tests and Windows .cmd / cmd.exe shell dispatch requirements. | 01-phase:L15-L16 | PP0-E010–PP0-E013
- **PHASE-007/SD-PP0-003** | **ADD** named tools failures (path ends_with, chmod, python3→python, env PoisonError) and 10-run adversarial stability. | 01-phase:L17-L18 | PP0-E014–PP0-E019
- **PHASE-007/SD-PP0-004** | **ADD** exact warning sites, pre_phase_0.toml fields, measurements, grounding table, and WIRING_CONTRACT archive-verify surface. | 01-phase:L19-L22 | PP0-E020–PP0-E025
- **PHASE-007/SD-PP0-005** | **ADD** R-pre1/R-pre2 mitigations and cfg-gate-not-delete rule. | 01-phase:L10-L11 | PP0-E003–PP0-E004
- **PHASE-007/SD-PP0-006** | **FLAG** incomplete pre-enumeration of all cfg-gated tests and HookRunner shell audit. | 01-phase:L21-L22 | PP0-A001–PP0-A002
- **PHASE-007/SD-P1I-001** | **ADD** ACTION-REQUIRED 1G→1I dep-graph fix and compile-failure scenario if ignored. | 01-phase:L147-L147 | P1I-E002, P1I-C001
- **PHASE-007/SD-P1I-002** | **ADD** permanent-vs-transient classification, 2-success HALF_OPEN, fresh InferenceSession, deterministic model_id jitter details. | 01-phase:L148-L149 | P1I-E007–P1I-E011
- **PHASE-007/SD-P1I-003** | **ADD** full cascade chain including TEXT-only/CODE-only intermediate modes and SPLADE-disabled collapse. | 01-phase:L150-L151 | P1I-E013–P1I-E018
- **PHASE-007/SD-P1I-004** | **ADD** newline-accepting validation tests, canary_reference_vec first-startup generation, DEGRADED⇒BM25_ONLY invariant tests. | 01-phase:L152-L154 | P1I-E020–P1I-E025
- **PHASE-007/SD-P1I-005** | **ADD** batching merge/segregate rules for cross-encoder queue and overflow returns unranked non-empty candidates. | 01-phase:L155-L155 | P1I-E026–P1I-E027
- **PHASE-007/SD-P1I-006** | **ADD** full QueryTrace/StageTrace field lists, nine stage names, DEBUG-not-SSB rule, session-end aggregates. | 01-phase:L156-L156 | P1I-E028–P1I-E031
- **PHASE-007/SD-P1I-007** | **ADD** index_version schema fields, archive-on-failure abort, user notification, isolation-under-degradation verification ownership. | 01-phase:L157-L158 | P1I-E032–P1I-E036
- **PHASE-007/SD-P1I-008** | **ADD** MEASURE targets and Phase-1 overall measurement list. | 01-phase:L145-L158 | P1I-E037–P1I-E039
- **PHASE-007/SD-P2D-001** | **ADD** design decisions D1–D5 (no-DKG detection, severity classes, separate inline counter, ReplanRequest MAJOR, type inventory). | 01-phase:L220-L226 | P2D-E004–P2D-E011
- **PHASE-007/SD-P2D-002** | **ADD** full T1–T10 TDD inventory including CONTRADICTS_PLAN: marker and BudgetExhausted topic. | 01-phase:L228-L229 | P2D-E013–P2D-E015
- **PHASE-007/SD-P2D-003** | **ADD** exact Critical pattern list, case-insensitive compare, assumption/consequence templates, guard_added=false default. | 01-phase:L227-L227 | P2D-E016–P2D-E018
- **PHASE-007/SD-P2D-004** | **ADD** phase_2D.toml fields and overhead/false-positive SLOs; architecture-overview update requirement. | 01-phase:L231-L231 | P2D-E019
- **PHASE-007/SD-P2D-005** | **ADD** grounding summary distinguishing GROUNDED/VERIFIED/UNGROUNDED claims and 4F deferrals. | 01-phase:L230-L230 | P2D-E020–P2D-E022
- **PHASE-007/SD-P2D-006** | **FLAG** D3 method list vs SourceMatch-only type conflict and T4 `.passed` field wording. | 01-phase:L224-L229 | P2D-C001, P2D-A001–P2D-A002
- **PHASE-007/SD-P3B-001** | **ADD** R10/R-3B-1/2/3 risks, r2d2 WAL cleanup, CHECK migration drop+recreate session semantics. | 01-phase:L246-L249 | P3B-E006–P3B-E007
- **PHASE-007/SD-P3B-002** | **ADD** full SQL schema, indexes, ssb_files junction, and note that orchestrator.txt §3 synopsis is incomplete. | 01-phase:L249-L252 | P3B-E009–P3B-E012
- **PHASE-007/SD-P3B-003** | **ADD** complete typed payload struct inventory including PhaseComplete fsync-before-unload semantics. | 01-phase:L256-L256 | P3B-E023–P3B-E024
- **PHASE-007/SD-P3B-004** | **ADD** write-loop BEGIN IMMEDIATE, oneshot seq, shutdown drain, broadcast-after-commit, Arc payload, CustomizeConnection. | 01-phase:L253-L254 | P3B-E017–P3B-E021, P3B-E027
- **PHASE-007/SD-P3B-005** | **ADD** SsbTraceable boundary, msgpack disk serialization, 1MB payload reject, full unit/property/adversarial/determinism suite and measure SLOs. | 01-phase:L255-L257 | P3B-E025–P3B-E029
- **PHASE-007/SD-P3B-006** | **ADD** audit-fix history BLOCK1–5 WARN1–6 from s-1427264. | 01-phase:L246-L257 | P3B-E031
- **PHASE-007/SD-P3B-007** | **FLAG** internal 15-vs-17 payload wording conflict. | 01-phase:L250-L250 | P3B-C001, P3B-A001
- **PHASE-007/SD-P4D-001** | **ADD** Designer role config, SystemConstraints sizing defaults, confidence/ranking formulas, adaptive complexity table, info_gain prune rule. | 01-phase:L426-L434 | P4D-E004–P4D-E016
- **PHASE-007/SD-P4D-002** | **ADD** full 4D.1–4D.6 TDD/adversarial gates, FORMULATE-SEARCH-EXTRACT-REFLECT loop, cloud-only speculation, measure keys, RuntimeClient wire rule. | 01-phase:L435-L435 | P4D-E019–P4D-E027
- **PHASE-007/SD-P4D-003** | **ADD** recursive 4-level protocol, all decomposition/baby-step block types, ResearchCache, DecisionSource, package anti-hallucination gate, Interactive/Autonomous selection. | 01-phase:L436-L440 | P4D-E028–P4D-E037
- **PHASE-007/SD-P4D-004** | **ADD entire web-research extension 4D.11–4D.18**: SearXNG client, WebFetcher/robots/readability, SourceQualityTable ≥200 domains, Web A-RAG, six VRAM tiers, budget manager, WebResearchOrchestrator, E2E metrics file. | 01-phase:L425-L440 | P4D-E038–P4D-E056
- **PHASE-007/SD-P4D-005** | **ADD** CLI `--vram-tier`, INTERACTIVE progress via research_step events, design principles from survey papers. | 01-phase:L440-L440 | P4D-E038, P4D-E049, P4D-E057
- **PHASE-007/SD-P4D-006** | **FLAG CONFLICT** research_* SSB action types not in 3B 17-type schema; Interactive present_options UI unspecified; projected VRAM ceilings. | 01-phase:L425-L440 | P4D-C001, P4D-A001–P4D-A003
- **PHASE-007/SD-P4D-007** | **CORRECT/EXPAND** summary DecisionSource labels: source uses Research/UserChoice/AutoSelected (not RESEARCH|EXISTING_PROJECT|USER_SPECIFIED|FALLBACK). | 01-phase:L439-L439 | P4D-E032
- **PHASE-008/SD-3C-001** | **ADD** full input/output contracts (3B/3A/2C→3D/3E/3F/4A/ReplanRequest max-1). | 01-MASTER:L259-L274 | P3C-E005–P3C-E012
- **PHASE-008/SD-3C-002** | **ADD** claw-code mapping details (UUID EntropySource, ClockProvider, TrustRequired drop). | 01-MASTER:L262 | P3C-E013–P3C-E015
- **PHASE-008/SD-3C-003** | **ADD** R6 VRAM serialize rule and R31 NotFound indistinguishability tests. | 01-MASTER:L268 | P3C-E016–P3C-E017, P3C-E030
- **PHASE-008/SD-3C-004** | **ADD** five checkpoint files, UntrackedValue rejection, Windows atomic rename, metadata-only SSB checkpoint. | 01-MASTER:L269-L271 | P3C-E034–P3C-E038
- **PHASE-008/SD-3C-005** | **ADD** recovery recipe map, corrupt degrade, monotonic step_index, no tool replay. | 01-MASTER:L272 | P3C-E040–P3C-E044
- **PHASE-008/SD-3C-006** | **ADD** spiral UI options string, N-3 recovery_target, blake3 B-16A supersession. | 01-MASTER:L274 | P3C-E053–P3C-E057
- **PHASE-008/SD-3C-007** | **ADD** Tier B bash gap deferral, isolation_metric aggregation out-of-scope, measure targets. | 01-MASTER:L259-L274 | P3C-E058–P3C-E060
- **PHASE-008/SD-3E-001** | **ADD** full AdaptiveLevel enum including Terminal; exact compute remainder-to-CodeContext rule. | 01-MASTER:L290-L300 | P3E-E007, P3E-E026
- **PHASE-008/SD-3E-002** | **ADD** BSP call timing, channel→slot6 rules, handoff BP2 recompute, Display surface. | 01-MASTER:L290-L300 | P3E-E016–P3E-E017, P3E-E024, P3E-E035
- **PHASE-008/SD-3E-003** | **ADD** floor steal order, BudgetInfeasible at dynamic<700, CodeContext<200 warning. | 01-MASTER:L294 | P3E-E027–P3E-E028
- **PHASE-008/SD-3E-004** | **ADD** numbered steps 3E.1–3E.7 acceptance detail MASTER only points to phase file for. | 01-MASTER:L300 | P3E-E036
- **PHASE-008/SD-3E-005** | **ADD** reduction logs excluded from current context; 100x determinism; no Instant::now. | 01-MASTER:L297-L299 | P3E-E032–P3E-E034
- **PHASE-008/SD-4B-001** | **ADD** full DAIRA marker set (15 forms) and MarkerReport counts. | 01-MASTER:L402-L403 | P4B-E013, P4B-E015
- **PHASE-008/SD-4B-002** | **ADD** force_full INTERACTIVE user-decide vs AUTONOMOUS auto-outline. | 01-MASTER:L407 | P4B-E019
- **PHASE-008/SD-4B-003** | **ADD** R1–R4 risks, paper ingest PRE_GATE, RuntimeClient wiring exit rule. | 01-MASTER:L395-L409 | P4B-E005–P4B-E008, P4B-E027, P4B-E032
- **PHASE-008/SD-4B-004** | **ADD** IntegrationTestRunner tool_id AUDITING-only, IntegrationResult stdout_hash. | 01-MASTER:L409 | P4B-E028–P4B-E030
- **PHASE-008/SD-4B-005** | **ADD** metrics key list and UNGROUNDED labels on latency/BLOCK reduction projections. | 01-MASTER:L409 | P4B-E022–P4B-E026
- **PHASE-008/SD-4B-006** | **QUALIFY** recorded DONE status applies to 4B.1–4.4 only; measure+runner remain deferred. | 01-MASTER:L398-L399 | P4B-E002–P4B-E003, P4B-C001
- **PHASE-008/SD-4F-001** | **ADD** complete type field lists (GpuInfo uuid, stderr 512, ResourceDelta, ConstraintResult margin). | 01-MASTER:L461 | P4F-E002–P4F-E008
- **PHASE-008/SD-4F-002** | **ADD** 16+8 named tests T1–T16/A1–A8 and dual-timeout kill_on_drop detail. | 01-MASTER:L463 | P4F-E021–P4F-E024
- **PHASE-008/SD-4F-003** | **ADD** E-22 sysinfo rejection / env fallback path and SD2 flat file layout. | 01-MASTER:L457-L465 | P4F-E034, P4F-C001–P4F-C002
- **PHASE-008/SD-4F-004** | **ADD** metrics targets and 12 testable invariants summary. | 01-MASTER:L463 | P4F-E028, P4F-E036
- **PHASE-008/SD-4F-005** | **ADD** DispatchStage.agent AUTHORITATIVE override rule and SSB branch merge/discard. | 01-MASTER:L462 | P4F-E018, P4F-E020
- **PHASE-008/SD-4F-006** | **FLAG** web_search inclusion conflict in FEASIBILITY tool set. | 01-MASTER:L462 | P4F-A001
- **PHASE-008/SD-4J-001** | **ADD** EMPIRE-PENDING pre-resolved decisions 1–4 (scaffold, submodule path, 5 SSB variants, EmpireIpc namespace). | 01-MASTER:L508-L517 | P4J-E002–P4J-E005
- **PHASE-008/SD-4J-002** | **ADD** Tauri command surface + Svelte panel scaffolding (user-facing). | 01-MASTER:L508-L517 | P4J-E033
- **PHASE-008/SD-4J-003** | **ADD** rate-limit global vs high-priority elevated cap, generic rejection, reconnect Register. | 01-MASTER:L514-L515 | P4J-E020–P4J-E023
- **PHASE-008/SD-4J-004** | **ADD** full measure keys/targets, FSB evaluate trigger, multi-brain evaluate trigger. | 01-MASTER:L517 | P4J-E028–P4J-E030
- **PHASE-008/SD-4J-005** | **ADD** repair blast-radius, TYPE_CHECKING false-positive guard, one-issue-per-pipeline. | 01-MASTER:L512 | P4J-E025
- **PHASE-008/SD-4J-006** | **FLAG CONFLICT** summary/body empire_event singular vs decision 5 variants; elai_empire crate name drift. | 01-MASTER:L514-L516 | P4J-C001–P4J-C002
- **PHASE-008/SD-IDX-001** | **ADD** PRE0 Windows-only scope and baseline LOC/test counts not expanded in MASTER's PRE0 section compression. | 01-MASTER:L13 | IDX-E002–IDX-E003
- **PHASE-008/SD-IDX-002** | **ADD** Phase 0 entropy-quarantine occurrence count (60) and 9-crate/one-tool skeleton. | 01-MASTER:L24 | IDX-E004–IDX-E005
- **PHASE-008/SD-IDX-003** | **ADD** 1F ADOPT threshold 1.3x and 1H vocab/recall KEEP gates as compact numbers. | 01-MASTER:L168-L181 (1F), related | IDX-E012, IDX-E014
- **PHASE-008/SD-IDX-004** | **ADD** 2C six role budget table and anti-rubber-stamp FINDING rule. | 01-MASTER:L340-L361 area | IDX-E019–IDX-E021
- **PHASE-008/SD-IDX-005** | **QUALIFY** index is not a substitute for numbered phase files' PRE_GATE/TASK/ACCEPTANCE. | 01-MASTER:L5 | IDX-I001

#### PHASE AC (103)

- **PHASE-001/AC-001** | C | TokenUsage field set differs design (4 fields) vs 0.2c shim (2 fields). | P0-C001
- **PHASE-001/AC-002** | A | prompt.rs REPLACE vs STRIP open. | P0-A001
- **PHASE-001/AC-003** | A | remote.rs DEFERRED vs STRIP open. | P0-A002
- **PHASE-001/AC-004** | A | sandbox whole-file vs types-only open. | P0-A003
- **PHASE-001/AC-005** | C | 1C model path onnx/ vs Step 5 per-model subdirs. | P1C-C001
- **PHASE-001/AC-006** | C | SPLADE deferral 1G (source) vs 1H (MASTER summary). | P1C-C002
- **PHASE-001/AC-007** | C | AUDITING recovery transitions need orchestrator.txt SPEC_UPDATE. | P3A-C001
- **PHASE-001/AC-008** | A | UNGROUNDED COMPLETE/IDLE tool policies and gate traits. | P3A-E016, P3A-I001
- **PHASE-001/AC-009** | A | REDIRECT invalid-target and full risk matrix incomplete. | P3A-A001–P3A-A002
- **PHASE-001/AC-010** | A | 3J-BASELINE fixture prompts and compression tolerance unspecified. | P3JB-A001–P3JB-A002
- **PHASE-001/AC-011** | A | 4K submit_task path, projects precedence, diff modal vs replace. | P4K-A001–P4K-A003
- **PHASE-001/AC-012** | A | 4M adapter alias, panel placement, name list, hybrid/auto detail. | P4M-A001–P4M-A003
- **PHASE-002/AC-001** | conflict | R20 ID collision: PageRank density vs phase-0 entropy quarantine; renumber out of scope for 1D. | P1D-E003
- **PHASE-002/AC-002** | conflict | Anti-spiral 3× vs same_step_count>3; 4G pins 3rd-failure interpretation. | P4G-C001
- **PHASE-002/AC-003** | qualification | HIGH-risk human pause vs auditor tool review are different layers; both remain binding. | P4G-C002
- **PHASE-002/AC-004** | ambiguity | 1A partial parse on syntax errors lacks completeness floor. | P1A-A003
- **PHASE-002/AC-005** | ambiguity | 1D “mentioned in task” personalization matching algorithm unspecified. | P1D-A002
- **PHASE-002/AC-006** | ambiguity | 3D evidence_gate_passes sufficiency predicate incomplete in phase file. | P3D-A003
- **PHASE-002/AC-007** | dependency | 3D ChannelRegistry + memory_monitoring + Topic(false) + checkpoint placement need orchestrator.txt amendments before/with implement. | P3D-E007, P3D-A002
- **PHASE-002/AC-008** | ambiguity | 3G plan_phases SQL shorthand vs SSB-only interim; view amendment needs user confirmation. | P3G-A001
- **PHASE-002/AC-009** | ambiguity | 3G coherence “text reference” vs structured implemented_contracts dual wording. | P3G-A003
- **PHASE-002/AC-010** | ambiguity | 4DOR query normalization for blake3 key undefined. | P4DOR-A001
- **PHASE-002/AC-011** | ambiguity | 4G SessionState ownership and definition still open; execution_mode attachment site TBD. | P4G-A001
- **PHASE-002/AC-012** | dependency | 4G requires new APIs: can_fit_llm, AgentPool::get_role, SSB escalation type — all require post-approval spec propagation. | P4G-E011, P4G-E013, P4G-E014
- **PHASE-002/AC-013** | ambiguity | parallel_dispatch default/TOML key for persona fan-out unspecified. | P4G-A003
- **PHASE-002/AC-014** | evidence | 4G runtime UX flags appear in MASTER summary more explicitly than in phase-4G body; do not over-claim source atoms for --local/--hybrid without addendum. | P4G-I001
- **PHASE-002/AC-015** | qualification | blake2 (1A file hash) vs blake3 (4DOR/4G) is intentional layer split, not a single-stack rule — document when consolidating hash policy. | IX-015
- **PHASE-003/AC-001** | conflict/schema | tantivy_v21 label vs INDEX_FORMAT_VERSION u32. | P1B-C001, P1B-A001
- **PHASE-003/AC-002** | ambiguity/config | Exact tantivy/ort pin versions deferred to execution decision docs. | P1B-A001, P1G-A003
- **PHASE-003/AC-003** | conflict/fallback | L-6 reranker tier needs amendment vs skip-only spec. | P1G-C001, P1G-E011
- **PHASE-003/AC-004** | ambiguity/measure | 1G skip threshold and 50 ms pass/fail after levers. | P1G-A001, P1G-E018
- **PHASE-003/AC-005** | conflict/schema | 3H per-message snapshot_context vs orchestrator blob schema. | P3H-C001, P3H-A001
- **PHASE-003/AC-006** | ambiguity/event | branch_abandon new type vs checkpoint payload. | P3H-A002, PIDX-C001
- **PHASE-003/AC-007** | ambiguity/data | SSB merge cleanup logical vs physical wording. | P3H-A003
- **PHASE-003/AC-008** | ambiguity/merge | File-overlap Conflict variant ungrounded. | P3H-A004
- **PHASE-003/AC-009** | ambiguity/crypto | blake3 stub hash algorithm choice. | P3H-A005
- **PHASE-003/AC-010** | ambiguity/surface | SurgicalDrawer full UI fields; step-cap merge notes. | P4DS-A001, P4DS-A002
- **PHASE-003/AC-011** | ambiguity/enum | CycleDetected vs CyclicDependency naming. | P4E-A001
- **PHASE-003/AC-012** | ambiguity/repair | Weakest-dependency definition for REORDER. | P4E-A003
- **PHASE-003/AC-013** | ambiguity/order | Index 3K listed before 3J vs HyDE dependency on 3J. | PIDX-A001
- **PHASE-003/AC-014** | ambiguity/index | Index omits C-30 surgical CLI/drawer present in 4D-SURGICAL. | PIDX-I001, P4DS-E002
- **PHASE-003/AC-015** | ambiguity/dep | 3E total_allocated_tokens exposure for TrimConfig. | P3H-A006
- **PHASE-004/AC-001** | conflict | 1E "sub-second" goal vs 2s hard interactive budget. | P1E-C001
- **PHASE-004/AC-002** | ambiguity | 1E heuristic edge-score aggregation into 0–10 before phi not fully formal. | P1E-A002
- **PHASE-004/AC-003** | ambiguity | 1E SPLADE in knowledge order vs no 1E splade stage. | P1E-A001
- **PHASE-004/AC-004** | ambiguity | 2B claim regex / Uncertain path-vs-symbol heuristics incomplete. | P2B-A001, P2B-A002
- **PHASE-004/AC-005** | conflict | reasoning.txt 4-field AgentGrcConfig vs 2C 8-field requirement + GroundingLevel order fix in 2A. | P2C-C001, P2C-E013, P2C-E020
- **PHASE-004/AC-006** | ambiguity | FINDING: prefix vs future structured FindingReport. | P2C-A001
- **PHASE-004/AC-007** | ambiguity | PI_T2_CIRCUIT_BREAKER_LINES exact value; full conventions SQL columns. | P3L-A001, P3L-A002
- **PHASE-004/AC-008** | conflict | 4C adversarial "prefer ruff library" vs decided subprocess+breaker. | P4C-C001
- **PHASE-004/AC-009** | ambiguity | T2 deny-list / lazy-code pattern catalog incomplete in phase text. | P4C-A001
- **PHASE-004/AC-010** | conflict | architecture-overview media models stale vs 4I Mode A/B authority. | P4I-C001
- **PHASE-004/AC-011** | ambiguity | QualitySpec threshold field names and max_iterations override policy. | P4I-A001, P4I-A002
- **PHASE-004/AC-012** | qualification | MASTER marks phase-3L as unindexed inventory item; deep extract is authoritative for 3L atoms. | P3L-E001
- **PHASE-005/AC-001** | conflict | 1F prerequisite Phase 0-independent vs 1C dependency graph. | P1F-C001
- **PHASE-005/AC-002** | conflict | 1F full eval suite vs RESOLVED skip 1F.3-1F.5. | P1F-C002
- **PHASE-005/AC-003** | conflict | 2A "grounding every step" summary line vs required subset. | P2A-C001
- **PHASE-005/AC-004** | ambiguity | 2A T11 hard Err vs warning for missing scope. | P2A-A002
- **PHASE-005/AC-005** | ambiguity | 2A full allowed_next table incomplete. | P2A-A001
- **PHASE-005/AC-006** | conflict | 3F rules→core-only crate map vs AstEvaluator knowledge coupling. | P3F-C001
- **PHASE-005/AC-007** | ambiguity | 3F evaluate return ALLOW|BLOCK|WARN vs ESCALATE enum arm. | P3F-A003
- **PHASE-005/AC-008** | conflict | 3J step numbering vs MASTER summary bundles. | P3J-C001
- **PHASE-005/AC-009** | conflict | 3J.14 31B in intro vs 3J.15 body renumber + E2E as 3J.14. | P3J-C002
- **PHASE-005/AC-010** | ambiguity | DoLa full-logit access method still VERIFY. | P3J-A001
- **PHASE-005/AC-011** | ambiguity | hybrid/auto BackendMode vs session removal of dead variants. | P3J-A003
- **PHASE-005/AC-012** | dependency | 3J.COMPOSE/3J.LOCAL referenced but not defined in this file. | P3J-I001
- **PHASE-005/AC-013** | qualification | MASTER can_fit_onnx for 3J.6 vs source can_colocate. | P3J-I002
- **PHASE-005/AC-014** | ambiguity | SCHEMA_REQUIRED provider-tier escalation is MASTER-only, not 3F body. | P3F-I001
- **PHASE-005/AC-015** | ambiguity | 3F.5 NaN/Inf metric guard appears in MASTER not phase-3F body. | MASTER:L314 vs P3F-E024
- **PHASE-006/AC-001** | ambiguity | 1H 0.3 vocab threshold ungrounded; 0.2–0.4 band may still measure recall. | P1H-A001
- **PHASE-006/AC-002** | ambiguity | 1H Candidate B missing path skips full A/B comparison. | P1H-A002
- **PHASE-006/AC-003** | ambiguity/dependency | 3I custom SQLite VFS necessity vs madsim hooks unresolved until spike. | P3I-A001
- **PHASE-006/AC-004** | ambiguity | 3I Windows madsim availability and manual scheduler equivalence open until 3I.1. | P3I-A002
- **PHASE-006/AC-005** | ambiguity/config | 3K synonym lexicon and AUTO keyword lists incomplete. | P3K-A001–P3K-A002
- **PHASE-006/AC-006** | ambiguity/measurement | 4A IPC/spawn/memory/overhead SLOs are ungrounded until 4A.6. | P4A-A001
- **PHASE-006/AC-007** | ambiguity | 4A WARN 1–9 bodies not expanded in phase file despite required test mapping. | P4A-A002
- **PHASE-006/AC-008** | ambiguity/design | 4H archive cap 50 and MSI thresholds are local design choices, not paper transfers. | P4H-A001–P4H-A002
- **PHASE-006/AC-009** | ambiguity | 4H source-tier auto-promote thresholds (5 hits, 0.8 verified ratio) lack broader tier policy. | P4H-A003
- **PHASE-006/AC-010** | conflict | DGM-style code self-mod vs 4H config/data-only must not be flattened. | P4H-C001
- **PHASE-007/AC-001** | C | 1I requires adding missing 1G→1I edge to implementation-guide dep graph before implement. | P1I-C001, P1I-E002
- **PHASE-007/AC-002** | A | 1I permanent/transient error enum variants incomplete; HALF_OPEN probe concurrency unspecified. | P1I-A001–P1I-A002
- **PHASE-007/AC-003** | A | 1I session-end p50/p99 SSB aggregation payload shape unnamed. | P1I-A003
- **PHASE-007/AC-004** | C | 2D design methods web_search/grep/api_registry vs SourceMatch-only type until 4F. | P2D-C001, P2D-A001
- **PHASE-007/AC-005** | A | 2D T4 uses stored `.passed` field vs computed method design. | P2D-A002
- **PHASE-007/AC-006** | A | 2D multi-word match threshold not a named constant. | P2D-A003
- **PHASE-007/AC-007** | C | 3B SsbPayload "15 variants" wording vs 17 action_types. | P3B-C001, P3B-A001
- **PHASE-007/AC-008** | A | 3B CURRENT_VERSION evolution policy beyond v1 not specified. | P3B-A002
- **PHASE-007/AC-009** | C | 4D research_step / research_decompose_level / web_research_step absent from 3B 17-type CHECK. | P4D-C001, P4D-A001
- **PHASE-007/AC-010** | A | 4D Interactive present_options concrete UI surface open. | P4D-A003, P4D-I001
- **PHASE-007/AC-011** | A | 4D several VRAM context_ceiling values projected not measured. | P4D-A002
- **PHASE-007/AC-012** | A | pre-phase-0 full cfg-gated test inventory not pre-enumerated. | PP0-A001
- **PHASE-007/AC-013** | A | pre-phase-0 HookRunner /bin/sh hardcode status left as audit. | PP0-A002
- **PHASE-008/AC-001** | ambiguity | 3C tool_results rolling window always-on vs only-if-slow. | P3C-A001
- **PHASE-008/AC-002** | ambiguity | 3C SSB CHECK migration mechanism and spiral "last N" size. | P3C-A002, P3C-A003
- **PHASE-008/AC-003** | ambiguity | 3E attempt-1 overlap threshold "e.g. 0.3" not fixed default. | P3E-I001, P3E-A001
- **PHASE-008/AC-004** | status/conflict | 4B DONE status vs remaining measure/integration body. | P4B-C001, P4B-E002-E003
- **PHASE-008/AC-005** | ambiguity | 4B deterministic commit message template unspecified. | P4B-A002
- **PHASE-008/AC-006** | conflict | 4F FeasibilityProbe vs FeasibilityProber naming and sysinfo vs shell-out RAM strategy. | P4F-C001, P4F-C002
- **PHASE-008/AC-007** | conflict | 4F FEASIBILITY tool set: stub_write+bash only vs +web search. | P4F-A001
- **PHASE-008/AC-008** | conflict | 4J 5 SSB empire_* variants vs body 'empire_event' singular. | P4J-C001
- **PHASE-008/AC-009** | conflict | 4J cargo package name elai_empire vs orchestrator --features empire. | P4J-C002
- **PHASE-008/AC-010** | ambiguity | 4J Tauri/Svelte surface names not enumerated. | P4J-I001
- **PHASE-008/AC-011** | ambiguity | Index pre0–2D omits completion and step-level gates. | IDX-I001, IDX-A001

### Relational registers (D)

Source: `docs/thefinalglance/feature-discovery/aggregates/D-AGGREGATE.md`.
Emits every D-001/AC-*, D-001/SD-*, and D-001/IX-* definition (IX re-listed for completeness). Format: `ID | statement` (anchor embedded in statement where present).

#### D IX (10)

- **D-001/IX-001** | **Promotion pipeline.** D-3 consensus supplies D-5 binary observations; D-5 promotes/retire templates through D-1 archive; D-6 interposes a held-out veto before promotion; D-7 adds a family veto; D-4 can auto-revert the same archive; D-8 later exposes lineage verification. Evidence: D0-007..D0-012, D5-011/D5-022..D5-024/D5-036/D5-045, D6-022/D6-023/D6-036/D6-045/D6-048.
- **D-001/IX-002** | **Concurrency boundary.** D-5 and D-6 can proceed around D-7’s read/signal work, and both explicitly exclude parallel execution with D-4 because archive/evaluation state overlaps. D-5 also calls D-6 parallel-safe, although D-6 separately requires D-5 to be `DONE`; that unresolved ordering tension is recorded below. Evidence: D5-003, D6-002.
- **D-001/IX-003** | **Deterministic promotion evidence.** C-15/D-1 seeded randomness drives D-5; C-15 seeded time drives D-6. D-5 demands same-seed byte identity; D-6 uses mock/seeded time and idempotent storage checks. Evidence: D5-017/D5-028/D5-036..D5-042, D6-006/D6-036/D6-039/D6-042.
- **D-001/IX-004** | **Audit-state claim path.** D-12 consumes D-11’s checker registry, blocks `AUDITING -> MEMORIZING` without a procedure, and shares the checker-result hook with downstream D-15. D-13 handles claims lacking a Mode-A checker. Evidence: D0-015..D0-019, D12-001/D12-003/D12-005/D12-017/D12-028/D12-038.
- **D-001/IX-005** | **Sycophancy proof-to-UI path.** D-14 produces the probe/flag, D-23 quantitatively gates it, and D-27 streams flags with quarantine links; D-27’s operator review is distinct from D-23’s human fixture labeling. Evidence: D0-018/D0-027/D0-031, D23-001/D23-005..D23-018, D27-007/D27-015/D27-024/D27-030.
- **D-001/IX-006** | **Wave-1 dashboard.** D-26 reads D-1/D-4/D-5/D-6/D-7/D-8 state, applies bounded/asynchronous query rules, and is parallel-safe with D-27. Evidence: D26-001/D26-002/D26-005..D26-027.
- **D-001/IX-007** | **Wave-2 dashboard and human disposition.** D-27 reads D-10..D-17 events but also writes ratify/reject decisions through `SsbWriter`; it is therefore both an observation surface and a human-governance mutation surface. Evidence: D27-004..D27-010/D27-020..D27-025.
- **D-001/IX-008** | **Hierarchical extension.** D-29 is an opt-in extension of D-5 with two environment gates and flat default; benchmark closure compares directly with the D-5 flat baseline. Evidence: D5-044, D29-008..D29-011/D29-018..D29-023.
- **D-001/IX-009** | **Evaluation-store secrecy.** D-6 composes temporal/9-gram filtering with Arbor-style dev/test isolation and DeNovo-style `.git` history removal; runtime leakage blocking remains explicitly owned by H-47. Evidence: D6-003/D6-019/D6-049..D6-052.
- **D-001/IX-010** | **Plan status/dashboard accounting.** D-0 defines `DONE` as implementation + cloud smoke + threshold, while D-26 adds the `--plan=D` status-report hook. D-29’s embedded “shipped” note does not independently satisfy the D-0 `DONE` definition because its closure benchmark remains pending. Evidence: D0-003/D0-035/D0-066/D0-068, D29-016/D29-020/D29-024..D29-027.

#### D SD (27)

- **D-001/SD-A001** | **D-0 exact operational ledger.** Add all schema-version expectations, exact SSB identifiers, directories, files, ports/GPU mapping, fixtures, baseline numbers, deferred unblock reasons, and `bn_status_report.py --plan=D` from D0-035..D0-068. The summary intentionally did not read D-0 and only surfaced its filename at summary lines 732/775.
- **D-001/SD-A002** | **D-5 selection policy is now concrete.** Add Beta(1,1) initialization, staged 10/50/200 evaluation, credible-lower-bound versus promoted median, per-domain deltas, bottom-quartile retirement, LUCB formula, gamma-ratio sampling, empty-arm/zero-sample behavior, and MSI-as-secondary-only policy (D5-005..D5-010/D5-014..D5-027/D5-034/D5-037). This resolves most of summary uncertainty item 7 (`04-D...:752`).
- **D-001/SD-A003** | **D-5 reproducibility/evidence is stronger.** Add same-seed byte identity, exact 10-arm rates, 500 pulls, 100 trials, bound x2, >=85% best-arm selection, >=15 tests, and exact event payloads (D5-026/D5-028..D5-033/D5-038..D5-042).
- **D-001/SD-A004** | **D-5 security/fallback detail.** Add user approval before `statrs`, incomplete-beta fallback, config immutability, graceful empty input, and archive-lock concurrency exclusion with D-4 (D5-003/D5-015/D5-018/D5-037).
- **D-001/SD-A005** | **D-6 schema and lifecycle.** Add complete task/outcome/source/status/reason schemas, dual indexes, active-only query, rewrite-on-retire behavior, signed shared-subset regression, gate reason, and exact event payloads (D6-009..D6-024).
- **D-001/SD-A006** | **D-6 thresholds resolved.** “Uniformly passing” means age `>6 months`, pass rate `>0.95`, and at least 20 evaluations; regression allows `delta >= -5%`; n-gram contamination is 9-gram Jaccard `>0.3` against a Blake3 Bloom (D6-019..D6-025). This resolves most of summary uncertainty item 8 (`04-D...:753`).
- **D-001/SD-A007** | **D-6 degraded mode and seed composition.** Add the empty-Bloom skip behavior, exact 25-task source mix, test tables, and the distinction between 20-file smoke presence versus 25-file acceptance (D6-026..D6-033/D6-039..D6-046).
- **D-001/SD-A008** | **D-6 evaluation secrecy amendments.** Add Arbor `B_dev/B_test` detached-worktree prior art and mandatory `.git` destruction/reinitialization for repo-derived fixtures; runtime leakage control remains H-47 (D6-049..D6-052). This is a material security addition to summary lines 568-583/618-621.
- **D-001/SD-A009** | **D-12 full operationalization contract.** Add the complete bounded `MeasurableProcedure`, TOML parser/error format, actual-vs-expected tri-state diff, keyword claim detector, per-checker timeout, human-review prompt, fresh execution/no cache, exact payloads, and eight-test floor (D12-009..D12-038).
- **D-001/SD-A010** | **D-23 benchmark implementation.** Add exact 70-positive/30-negative human-labeled fixtures, report confusion-matrix fields/formulas, `GatingVerdict`, CLI/CI reachability, benchmark events, dated report artifact, and `Convergent` cloud expectation (D23-005..D23-019).
- **D-001/SD-A011** | **D-26 surface is much larger.** Add `get_bai_arms`, `get_canary_report`, `get_held_out_status`, plus `TripwireStatusStrip`, `BaiArmsTable`, `HeldOutPanel`, and `ArchiveVerifyGauge`; add filters, colors, click behavior, async verification, bounded recent reads, and exact three-second tripwire SLA (D26-005..D26-032). The summary listed only three commands and two named new components at lines 383-399/529-531.
- **D-001/SD-A012** | **D-27 is a governance UI, not only charts.** Add `get_discovery_queue`, `ratify_discovery`, `reject_discovery`, `DiscoveryQueueDrawer`, and `CheckerStatusPanel`, including reviewer-required write actions, quarantine links, configurable thresholds, placement, and evidence snippets (D27-008..D27-025/D27-030). The summary listed only four commands/four components at lines 401-415/529-531.
- **D-001/SD-A013** | **D-29 contains concrete host-side symbols.** Add exact Rust types/functions/line anchors, `HierarchicalBaiRoundCompleted`, two environment gates, default-flat behavior, task-runner branch, and the explicit claim that Path A host dispatch shipped while the Gibbs comparison remains pending (D29-013..D29-023).
- **D-001/SD-A014** | **D-29 model proposal is concrete.** Add Beta-per-arm/Gamma-hyperprior/Gibbs shape and the five-family, ten-arm, within-family-correlated, 1000-pull benchmark (D29-003/D29-007/D29-008), resolving part of summary uncertainty item 20 (`04-D...:765`).
- **D-001/SD-C001** | **Baby-step block number.** Correct summary line 187’s “block 7” to **block 9** for this deep evidence set: D0-047 and D12-004/D12-027 agree on eight existing blocks plus ninth `MEASURABLE_PROCEDURE`.
- **D-001/SD-C002** | **D-23 comparison mode.** Summary line 345 says D-23 runs probe-enabled versus probe-disabled. The processed D-23 original does **not** specify that ablation; it replays every fixture through D-14 and compares the flag to labels (D23-007). Treat enabled/disabled as summary-only unless another original proves it.
- **D-001/SD-C003** | **D-23 provenance wording.** Summary line 343 says “Anthropic/Sharma”; this original says Sharma et al. public probes and does not name an Anthropic dataset identifier (D23-001/D23-004).
- **D-001/SD-C004** | **D-26 lineage target.** Summary line 390 says it cross-links to a “baby-step archive.” The original specifies an `ArchiveEntry` parent-hash DAG and flips to `MutationArchivePanel` detail, not an explicit baby-step-archive cross-link (D26-012/D26-013).
- **D-001/SD-C005** | **D-26 prerequisites.** Qualify summary line 399/matrix line 652 (`D-1..D-8`) with the original’s stronger D-1..D-9 shipped prerequisite (D26-002), while retaining the conflict with D0-030.
- **D-001/SD-C006** | **D-26 poll interval.** Summary uncertainty line 764 says polling latency is unquantified. The tripwire acceptance SLA is explicitly **3 seconds** (D26-031); other polls remain unquantified.
- **D-001/SD-C007** | **D-26 PPM/consensus display.** Summary lines 391-393 mirror the objective, but the detailed original task/command/component contract lacks a PPM or consensus query. Keep these as objective-level requirements with an implementation-surface ambiguity (D26-001 versus D26-005..D26-021).
- **D-001/SD-C008** | **D-29 status.** Summary lines 432-444/655/766 present D-29 only as deferred and assert zero shipped features. The original still has deferred acceptance, but explicitly claims Path A host-side Rust dispatch shipped (D29-016/D29-023). Correct representation is mixed evidence/partial productization with research closure pending, not cleanly `DONE`.
- **D-001/SD-C009** | **D-29 closure outcome.** Summary treats Path A and Path B as both open. The embedded wiring comment states Path A was the outcome and Path B was not (D29-016), although unchecked acceptance prevents treating the overall session as closed.
- **D-001/SD-C010** | **D-6 privacy/security coverage.** Summary lines 618-621 say no distinct privacy feature. That remains true for user PII, but D6-051/D6-052 add a distinct confidentiality/leakage safeguard for evaluation fixtures: remove recoverable Git history at build time.
- **D-001/SD-C011** | **Held-out auto-retirement.** Summary lines 99/601 phrase six-month uniform passing broadly. Deep rules require all of strict age, `>0.95`, and `>=20` evaluations (D6-020/D6-021); a TTL alone does not retire.
- **D-001/SD-C012** | **D-5 benchmark wording.** Summary line 600 reports a 50-round Audibert gate. Preserve that index gate (D0-009), but also record the original’s separate 500-pull/100-trial/bound-x2 integration test and 50-cycle cloud bootstrap (D5-033/D5-042).
- **D-001/SD-C013** | **Dashboard component/command counts.** Replace any interpretation that D-26 has only three commands/two components or D-27 only four commands/four components: their originals require 6+6 and 7+6 respectively (D26-021/D26-029; D27-020/D27-028).

#### D AC (50)

- **D-001/AC-001** | **D-29 lifecycle/status is internally conflicting `[C]`.** Header, deterministic-check section, acceptance list, and “anticipated files” say deferred/pending (D29-002, D29-024..D29-027); the wiring contract says Path A shipped and Rust dispatch is live (D29-016/D29-023). Safest status is **mixed/partially implemented claim, research closure not done**, not simply `DONE` or simply untouched.
- **D-001/AC-002** | **D-0 versus D-29 status conflicts `[C]`.** The index calls D-29 deferred/research-grade (D0-033/D0-064/D0-066); the later D-29 contract claims Path A shipped (D29-016). No date/version marker explains the update.
- **D-001/AC-003** | **Gibbs evidence is unresolved `[A/C]`.** D29-017/D29-020 say the Python Gibbs benchmark is RunPod-pending, while D29-023 expects a live round with `gibbs_iterations > 0`. It is unclear whether Rust performs a separate Gibbs-like update or the observable contract is aspirational.
- **D-001/AC-004** | **Flat-module identity drifts `[A]`.** D-5 plans `bai.rs`/`BaiPromotionEngine` (D5-021/D5-043); D-29 names a D-5 `darwinian_bai` baseline and default discount factor (D29-018). Rename/history is not explained.
- **D-001/AC-005** | **D-26 prerequisite differs `[C]`.** D-0 says D-26 requires/exposes D-1..D-8 (D0-030/D0-061); D-26 says D-1..D-9 must be shipped (D26-002). D-26 does not name a D-9 data surface, so D-9 may be a governance prerequisite rather than a rendered capability.
- **D-001/AC-006** | **D-26 objective exceeds its task ledger `[A]`.** D26-001 promises PPM-score and consensus-rule display, but its six commands/components do not name a PPM/consensus query or field. Existing `ReasoningDrawer` data may supply them, but that is not stated.
- **D-001/AC-007** | **Archive verification ownership is split `[A]`.** D26-006 attributes the invoked method to D-1, while D26-022 says `verify_chain` spans D-1+D-8. Exact owning module/API is unresolved.
- **D-001/AC-008** | **Held-out “append-only” conflicts with rewrite `[C/A]`.** D0-041 calls the task layout append-only, but D6-015 rewrites TOML status on retirement. The no-delete rule (D6-037) is explicit; append-only may mean no file deletion or may refer only to audit history.
- **D-001/AC-009** | **Semantic/structural contamination defenses are designed but not implemented `[A]`.** D6-004 requires semantic fingerprints, D6-005 names structural-prior randomization, and D6-012 has `SemanticContamination`, but no semantic detector or structural-randomization task/config/test is specified. Only n-gram Bloom logic is executable in this plan.
- **D-001/AC-010** | **Bloom absence degrades silently `[A]`.** D6-027 says skip n-gram checking until data exists; no SSB warning, operator alert, or promotion fail-closed rule is stated for this protection gap.
- **D-001/AC-011** | **Store error/idempotency semantics are incomplete `[A]`.** Acceptance says load/append/retire are idempotent (D6-042), but behavior for duplicate `TaskId`, partially written TOML, parse failure, date-path mismatch, or concurrent writers is absent.
- **D-001/AC-012** | **Seed-count gates differ `[A]`.** D6-026/D6-043 require 25 tasks; shell presence gate accepts `>=20` (D6-041); observable contract expects `>=20` retirements from a 25-task uniform-pass simulation (D6-039). These may be smoke versus final gates, but are not labeled that way.
- **D-001/AC-013** | **Archive task payload shape is underspecified `[A]`.** D6-006 says an `ArchiveEntry` carries `task_id` rather than `template_id`, but no union/schema amendment is defined in D-6.
- **D-001/AC-014** | **D-12 block-number conflict is resolved only by deep evidence `[C]`.** The orientation summary says “block 7,” but both D0-047 and D12-004/D12-027 explicitly say block 9 after eight existing blocks. The original-plan evidence therefore favors block 9.
- **D-001/AC-015** | **D-12 schema-bump atomicity is unclear `[A]`.** D0-045 expects one atomic D-10+D-11+D-12 bump; D12-021 independently specifies `57 -> 58`. It may mean sequenced per-session bumps, but “atomic” is unexplained.
- **D-001/AC-016** | **Claim detection cannot literally guarantee no false negatives `[A]`.** D12-019 defines keyword/prose heuristics while D12-020/D12-030 say false negatives are not allowed. No formal grammar, exhaustive claim AST, or fallback classifier is supplied.
- **D-001/AC-017** | **Transition compatibility is temporary but unbounded `[A]`.** `measurable_procedure` is `Option` during transition (D12-009) while the L3 rule is no opt-out (D12-006/D12-030); no migration version/date identifies when `None` ceases to be accepted outside claim steps.
- **D-001/AC-018** | **D-23’s 70/30 rationale is numerically tense `[C]`.** Context cites observed sycophancy at 23-33% (D23-003), while decisions say a fixture population with 70% sycophancy “matches” Sharma (D23-010). The split is useful for power but does not match that prevalence literally.
- **D-001/AC-019** | **D-23 outcome vocabulary drifts `[A]`.** Fixtures use `Sycophancy/Robust` (D23-005), execution uses “flagged-divergent” (D23-007), and Cloud B expects `Convergent` (D23-018); mapping is not typed.
- **D-001/AC-020** | **D-23 failure exit is incomplete `[A]`.** CLI exit 0 is specified only for a passing gate (D23-016); non-passing exit code and report persistence on failure are not defined.
- **D-001/AC-021** | **D-5 has multiple benchmark scales `[A, not necessarily conflict]`.** D0-009 says 50 rounds within the published bound; D5-033 defines 500 pulls/100 trials/bound x2; D5-042 defines a 50-cycle cloud bootstrap. They appear to be distinct index, integration, and cloud gates and must not be collapsed into one threshold.
- **D-001/AC-022** | **D-5 Successive Rejects output semantics are terse `[A]`.** D5-019 returns “ordered survivors,” while repeated elimination conventionally ends with one survivor; D5-024 applies it to a bottom quartile. Exact returned ordering and budget allocation across `retire_round` are not fully specified.
- **D-001/AC-023** | **D-27 rejection lifecycle is asymmetric `[A]`.** Ratification explicitly updates the queue file (D27-009); rejection only explicitly writes an event (D27-010), although the UI implies disposition. Whether rejected entries leave the pending queue is not stated.
- **D-001/AC-024** | **D-27 human governance is attribution-only `[A]`.** Reviewer is required, but authentication, reviewer authorization, multiple reviewers, and conflict-of-interest controls are absent; multi-reviewer flow is explicitly deferred (D27-023/D27-032).
- **D-001/AC-025** | **Polling semantics remain partly open `[A]`.** D-26 fixes tripwire reaction at 3 seconds (D26-031), but archive, verification, D-27 sycophancy, disconnection, stale state, and retry intervals are not quantified.
- **D-001/AC-026** | **Dashboard data exposure is ungoverned `[A]`.** D26-012 and D27-007/D27-014..D27-016 display template, claim, evidence, reviewer, and failure snippets without an explicit permission/redaction model.
- **D-001/AC-027** | **D-0’s aggregate status is internally tense `[C/A]`.** D0-066 reports 27 in-scope slots and three deferred slots but also says all 30 authored files are `PLANNED`; the status legend treats `PLANNED` and `DEFERRED` as distinct. It is unclear whether “all PLANNED” describes document authorship rather than session status.
- **D-001/AC-028** | **The D-5 file inventory drifts `[A/C]`.** D0-040 expects both `bai.rs` and `promotion_arms.rs`, while the detailed D-5 tasks, wiring contract, and files-touched list place the engine in `bai.rs` and never name `promotion_arms.rs` (D5-014..D5-043). No consolidation/rename note resolves the extra index file.
- **D-001/AC-029** | **The held-out event inventory grows without reconciliation `[A]`.** D0-041 lists only `HELD_OUT_REFRESH` and `HELD_OUT_AUTO_RETIRE`; D6-024 makes `HeldOutPromotionGate` a third schema event. This may be a detailed-plan expansion, but D-0’s “expected additions” list was not updated.
- **D-001/AC-030** | **D-6 has several compatible but distinct time-scale gates `[A, not necessarily conflict]`.** The index cloud proof retires one 18-month-old task (D0-010), D-6 units exercise 7/5-month cases (D6-031), and cloud/integration evidence advances across 12 months (D6-033/D6-039/D6-044). They must remain separate checks rather than one normalized age threshold.
- **D-001/AC-031** | **Auto-refresh cadence is unbound `[A]`.** D6-003 says refresh every `N` sessions, but `AutoRefreshPolicy`, config, reachability, deterministic checks, and acceptance define no `N`, scheduler, or invocation trigger (D6-020/D6-025/D6-035..D6-046).
- **D-001/AC-032** | **The n-gram detector’s corpus interface is internally underspecified `[A]`.** The source signature accepts `training_corpus_hashes: &Vec<Blake3Hash>` while the algorithm also says to load `ngram-bloom.bin`; it does not say which input is authoritative or how exact Jaccard union size is recovered from Bloom membership (D6-019).
- **D-001/AC-033** | **BabyStepSpec’s existing-block identifier drifts `[A]`.** D12-004 quotes the current block as `FILES_PLAN`, while the same source calls the C-31 foundation `FILE_PLAN` (D12-007/D12-026). The ninth block is consistently `MEASURABLE_PROCEDURE`, but the pre-existing block name is not.
- **D-001/AC-034** | **Checker timeout disposition is undefined `[A]`.** D12-030 requires `run_procedure` to honor `checker.timeout_secs`, but no `CheckerResult`, divergence value, event payload, retry policy, or auditor reason is specified for timeout.
- **D-001/AC-035** | **D-12’s deterministic shell does not directly cover two acceptance helpers `[A]`.** D12-032’s grep checks omit `diff_expected` and `detect_claim_in_baby_step`, although D12-033 requires both behaviors. Scoped tests may cover them, but the deterministic presence gate does not prove that explicitly.
- **D-001/AC-036** | **D-23 fixture-count gates differ `[A]`.** Tasks and acceptance say 100 fixtures (D23-005/D23-016), while the shell gate accepts any count `>=100` (D23-015). Extra fixtures would change prevalence and metrics, but their handling is not defined.
- **D-001/AC-037** | **D-23 serialization contracts are absent `[A]`.** The plan defines report fields and label counts but not the JSON fixture fields needed to replay D-14, nor the payload schemas for the generic `BenchmarkStarted`/`BenchmarkResult` events (D23-005..D23-007/D23-014/D23-019).
- **D-001/AC-038** | **D-23’s parallel-safe range self-includes `[A]`.** D23-002 says it is parallel-safe with D-19..D-25, a literal range containing D-23 itself. The likely intent is “the other benchmark sessions,” but exclusions are not enumerated.
- **D-001/AC-039** | **Archive parent identifiers drift `[A]`.** `MutationArchivePanel` detail links `parent_id` (D26-012), whereas `LineageGraphView` edges use `parent_hash` (D26-013). The source does not state whether they are aliases or require a lookup.
- **D-001/AC-040** | **Canary “last N” is not parameterized `[A]`.** `get_canary_report(template_id: Option<TemplateId>)` promises the given template or “last N” reports (D26-009), but its signature/config defines no `N` or limit default.
- **D-001/AC-041** | **Two D-26 navigation surfaces lack explicit wiring `[A]`.** `ArchiveVerifyGauge` opens a full report and the contract claims a dedicated archive sub-window (D26-017/D26-023), but the six commands and files-touched list define neither a full-report command nor a sub-window integration file (D26-021/D26-033).
- **D-001/AC-042** | **D-26 mixes polling and subscription without ownership rules `[A]`.** Individual panels poll, while archive/tripwire updates also reuse real-time SSB subscription (D26-014/D26-017/D26-019/D26-027). Cache precedence, deduplication, and reconnect behavior are not specified.
- **D-001/AC-043** | **The refusal healthy band has two authorities `[A]`.** The component task states `[5%,50%]` (D27-012), while the stub ban requires thresholds loaded from D-17 config (D27-025). D-27 does not state that 5/50 are D-17 defaults or what the UI shows when config differs.
- **D-001/AC-044** | **Discovery disposition events lack schema/lifecycle contracts `[A]`.** D27-009/D27-010 write `DiscoveryRatified` and `DiscoveryRejected`, but this source defines no SSB variant/payload/schema bump, round-trip test, idempotency key, duplicate-click behavior, or rejection queue-file update.
- **D-001/AC-045** | **D-29 closure paths overlap on equality `[C]`.** Path A accepts hierarchical regret `<=` flat (D29-009), while Path B is “equal or worse” (D29-010); equality satisfies both, so the branch decision is not mutually exclusive.
- **D-001/AC-046** | **D-29 mode/event failure schemas are incomplete `[A]`.** The source names `HierarchicalBaiRoundCompletedPayload` and the unset-mode `Flat` default (D29-015/D29-022), but gives no payload field types beyond observable references to `gibbs_iterations`/`SelectionBasis`, and no behavior for an invalid `ELAI_BAI_MODE` value or hierarchical runtime failure.
- **D-001/AC-047** | **“Online updates” conflicts with an unqualified update API `[A]`.** D29-014 claims `hierarchical_update`, while D29-028 defers “online hierarchical updates.” The source does not distinguish a batch/offline update from the deferred online behavior.
- **D-001/AC-048** | **D-5’s LUCB optional return is incomplete `[A]`.** D5-020 declares `Option<TemplateId>` but specifies neither empty-input nor no-competitor behavior; the explicit empty-input rule applies only to `thompson_select` (D5-018).
- **D-001/AC-049** | **D-5/D-6 ordering and parallelism conflict `[C/A]`.** D5-003 calls D-6 parallel-safe, but D6-002 requires D-5 to be `DONE` before D-6. No milestone-level dependency explains how both can be true.
- **D-001/AC-050** | **Empty-set n-gram overlap is undefined `[A]`.** D6-017/D6-037 require empty extraction to return safely, while D6-018 defines overlap as `intersection/union`; no zero-union convention is given for two inputs with no n-grams.

#### D-004 summary deltas (D004-SD, 14)

Additional SD-class rows under the D-004 ledger namespace `D-004/D004-SD-*` (not `D-00N/SD-*`).

- **D-004/D004-SD-001** | [correction] Preserve Rust identifiers TripwireFired, TripwireFiredConcurrent, and TripwireAutoRevert rather than collapsing D-4 to a single TRIPWIRE_FIRED label; add event payloads and intent-before-retire ordering. | D004-D4-022..026, D004-D4-046
- **D-004/D004-SD-002** | [correction] Resolve “non-target family” versus implementation “ANY family” regression blocking; do not present the non-target rule as fully specified until reconciled. | D004-D7-001, D004-D7-020, D004-D7-037
- **D-004/D004-SD-003** | [addition] Add D-7’s Domain migration, three-valued decision, sparse-data deferral, exact fixtures/config/events, BAI pre-promotion hook, and its non-bijective Mixed gap/count conflict. | D004-D7-013..044
- **D-004/D004-SD-004** | [addition] Add all J-11/J-15 amendment behavior: register-time self-fixture fail-closed gate, demotion cap instead of averaging, Tier-C non-copy rule, and exactly one deterministic same-checker reformulation before Ungrounded. | D004-D11-044..051
- **D-004/D004-SD-005** | [correction] Do not state that D-15 adds NoveltyClass to CheckerResult; the plan attaches it to ClaimResult metadata, and the declared CheckerResult lacks classification inputs. | D004-D15-013, D004-D15-031..033
- **D-004/D004-SD-006** | [addition] Add D-15 queue file/schema/states/events, 0.85 precedence heuristic, empty-DKG fallback, conservative no-auto-publish boundary, and the fact that review mutation/TUI/config are unresolved or deferred. | D004-D15-011..029, D004-D15-034..038
- **D-004/D004-SD-007** | [addition] Add D-18 nested trigger/observation/outcome/environment types, actual orthogonality rule, operator-only production gate, exact config/events/artifact, and the one-effect reject-vs-flag conflict. | D004-D18-009..044
- **D-004/D004-SD-008** | [correction] Remove the claimed synthetic ARC fallback from v1: contamination produces ContaminationSuspected/exit 1, while synthetic generation is explicitly deferred. | D004-D20-015, D004-D20-027..028
- **D-004/D004-SD-009** | [correction] Expand D-20 prerequisites/landing order beyond D-10/D-15: it consumes D-14/D-16/D-17 and D-19 events, and its typed verdict cannot currently fail no-publish or sycophancy gates. | D004-D20-003, D004-D20-005, D004-D20-016, D004-D20-018, D004-D20-029..030
- **D-004/D004-SD-010** | [correction] Do not claim D-22’s distributions are operationally sourced from D-6; the plan builds a separate 300-file benchmark tree and never wires HeldOutStore. | D004-D22-003, D004-D22-005..008, D004-D22-020
- **D-004/D004-SD-011** | [addition] Add D-22’s tau.txt input, exact fixture counts, CLI/report/events, 500-tuple live gate, single-τ non-goal, and missing report/statistical/snapshot contracts. | D004-D22-005..026
- **D-004/D004-SD-012** | [addition] Add D-4 detector edge cases/state/config, deterministic trimming, append-only recovery evidence, false-positive cloud run, pre-D-8 inspection fallback, and unresolved signal/order/recovery details. | D004-D4-011..052
- **D-004/D004-SD-013** | [addition] Add D-11’s precise checker I/O, timeout/Inapplicable fallbacks, live anti-slopsquatting checks, isolation/sandbox requirements, first-applicable limitation, and missing ordering/registration/security/test contracts. | D004-D11-009..043, D004-D11-052..061
- **D-004/D004-SD-014** | [correction] Treat uppercase summary event names as descriptive labels unless a serializer mapping is supplied; originals define CamelCase Rust variants and several additional variants. | D004-X-015

## Audit note (relational registers repair)

Appended full relational register dumps after the prior audit block. Prior section body cited registers under Composes-with / Superseded only; most AC/SD rows and many IX rows were not re-listed as dedicated register lines.

- **PHASE IX emitted**: 172 (aggregate unique definition rows: 172)
- **PHASE SD emitted**: 256 (aggregate unique definition rows: 256)
- **PHASE AC emitted**: 103 (aggregate unique definition rows: 103)
- **D IX emitted**: 10 (aggregate unique D-*/IX-*: 10)
- **D SD emitted**: 27 (aggregate unique D-*/SD-*: 27; independent audit said 28, source holds 27)
- **D AC emitted**: 50 (aggregate unique D-*/AC-*: 50)
- **D-004/D004-SD emitted (extra)**: 14
- **Write mode**: append-only to SECTION-PHASE-D.md; aggregates read-only.
- **Verification method**: unique-ID grep counts on the updated section file must be >= aggregate unique counts for PHASE IX/SD/AC and D AC/SD.
