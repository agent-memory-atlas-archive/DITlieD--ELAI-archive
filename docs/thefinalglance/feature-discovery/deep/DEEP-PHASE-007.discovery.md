# DEEP-PHASE-007 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1406-L1416`, heading `### DEEP-PHASE-007`; the next chunk starts at L1418.
- Declared chunk: family PHASE, **5 files**, **2,723 logical lines**, **166,521 bytes**, all five classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all five listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `pre-phase-0:Lx-Ly`, `phase-1I:Lx-Ly`, `phase-2D:Lx-Ly`, `phase-3B:Lx-Ly`, and `phase-4D:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 343 | 24,838 | e6b45aa69b8027736c2ce27fef682579be0eb087aa5892f4b6ac3225be9ee429 | docs/specs/guide/steps/phase-1I.txt | exact |
| numbered/index original | 428 | 28,356 | c9a3a7dff05032f0274ad689099e76919b3960ecb129aca23f152b6b669560c7 | docs/specs/guide/steps/phase-2D.txt | exact |
| numbered/index original | 492 | 30,997 | d8a018ef04d8868e5e21a7768cef3c007f87a3a4f67574ffe788f4b28307a034 | docs/specs/guide/steps/phase-3B.txt | exact |
| numbered/index original | 1,260 | 72,544 | 96bfd91863d5f645bdace4d3c4f38acac92824baa37d74744a73d70ffb675fdc | docs/specs/guide/steps/phase-4D.txt | exact |
| numbered/index original | 200 | 9,786 | 470a805e489fa0d12e71287178405dffb9ec571beb18cb85ea6508b06903b4eb | docs/specs/guide/steps/pre-phase-0.txt | exact |
| **Aggregate** | **2,723** | **166,521** | — | **5 exact paths** | **exact** |

## Per-file atomic ledgers

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

## Cross-source interactions and lifecycle

| Interaction ID | Class | Explicit discovery or bounded interpretation | Source atoms |
|---|---|---|---|
| IX-001 | E | Pre-phase-0 clean Windows baseline is substrate for all later cargo gates in 1I/2D/3B/4D. | PP0-E001–PP0-E022; P1I-E012; P2D-E013; P3B-E028; P4D-E019 |
| IX-002 | E | Phase 1I knowledge_degradation SSB events require Phase 3B typed action_type and single-writer path. | P1I-E016; P3B-E010, P3B-E019–P3B-E020 |
| IX-003 | E | Phase 1I ReadinessStatus READY\|DEGRADED\|UNAVAILABLE gates orchestrator FSM IDLE→PLANNING; UNAVAILABLE blocks all transitions (3A evidence gate consumer). | P1I-E022–P1I-E024 |
| IX-004 | E | Phase 1I SearchMode cascade and BM25_ONLY degraded mode compose with 1B/1C isolation verified under degradation. | P1I-E013–P1I-E018, P1I-E036 |
| IX-005 | E | Phase 1I ClockProvider breaker and 3B created_at ClockProvider both require Phase 0 entropy quarantine substrate. | P1I-E006; P3B-E019 |
| IX-006 | E | Phase 2D Blocked outcome signals ReplanRequest(MAJOR) for orchestrator 3A+ human-approval EXECUTING→PLANNING. | P2D-E007, P2D-E022; P2D-E015 |
| IX-007 | E | Phase 2D CSQ depends on 2C AgentGrcConfig.csq_required and per-role CSQ mandates. | P2D-E001, P2D-E021 |
| IX-008 | E | Phase 2D inline feasibility SourceMatch is temporary; 4F adds WebSearch/Grep/ApiRegistryCheck variants and deeper contradiction analysis. | P2D-E009, P2D-E022; P2D-C001 |
| IX-009 | E | Phase 3B is backbone for 1C file_write invalidation, 3C traces, 3D supersteps, 3E SSB slot, and 4D research progress events. | P3B-E002–P3B-E004; P4D-E018, P4D-E037, P4D-E054 |
| IX-010 | E | Phase 3B WARN-6 write-before-ToolResult contract binds agent tool IPC to durable journal order. | P3B-E020 |
| IX-011 | E | Phase 3B PhaseCompletePayload fsync-before-unload durability rule composes with agent lifecycle unload paths. | P3B-E024 |
| IX-012 | E | Phase 4D Research Agent Tier C isolation reuses Phase 4A process walls and IPC proxy patterns. | P4D-E001, P4D-E014, P4D-E053 |
| IX-013 | E | Phase 4D Designer GRC config reuses 2C role specialization pattern (distinct from Executor). | P4D-E005 |
| IX-014 | E | Phase 4D SystemConstraints sizing defaults feed later L2 sizing/feasibility consumers. | P4D-E006 |
| IX-015 | E | Phase 4D package API_REGISTRY_CHECK reuses 2B verification method family. | P4D-E032, P4D-E035 |
| IX-016 | E | Phase 4D Interactive present_options is a human-gate surface sibling to 3A InteractiveGate / later dashboard interventions. | P4D-E034, P4D-I001 |
| IX-017 | C | Phase 4D research_* SSB action types are not registered in Phase 3B's 17-type CHECK schema; consumers cannot journal them without a schema extension. | P4D-C001; P3B-E010; P4D-A001 |
| IX-018 | E | Phase 1I index rebuild user notification is the knowledge-side operator surface; 4D research_step events are the research-side progress surface; both need a later dashboard consumer. | P1I-E034; P4D-E057 |
| IX-019 | I | Pre-phase-0 WIRING_CONTRACT archive verify → archive_verify_result suggests SSB action vocabulary predates formal 3B enum completeness. | PP0-E025; P3B-E010, P3B-E012 |
| IX-020 | E | Phase 4D VRAM-tier --vram-tier CLI flag and 3B metrics files continue the operator observability artifact pattern from pre_phase_0.toml / phase_2D.toml / phase_4D.toml. | PP0-E021; P2D-E019; P4D-E025, P4D-E049, P4D-E056 |

## Ambiguity and conflict register

| Register ID | Class | Issue | Source atoms |
|---|---|---|---|
| AC-001 | C | 1I requires adding missing 1G→1I edge to implementation-guide dep graph before implement. | P1I-C001, P1I-E002 |
| AC-002 | A | 1I permanent/transient error enum variants incomplete; HALF_OPEN probe concurrency unspecified. | P1I-A001–P1I-A002 |
| AC-003 | A | 1I session-end p50/p99 SSB aggregation payload shape unnamed. | P1I-A003 |
| AC-004 | C | 2D design methods web_search/grep/api_registry vs SourceMatch-only type until 4F. | P2D-C001, P2D-A001 |
| AC-005 | A | 2D T4 uses stored `.passed` field vs computed method design. | P2D-A002 |
| AC-006 | A | 2D multi-word match threshold not a named constant. | P2D-A003 |
| AC-007 | C | 3B SsbPayload "15 variants" wording vs 17 action_types. | P3B-C001, P3B-A001 |
| AC-008 | A | 3B CURRENT_VERSION evolution policy beyond v1 not specified. | P3B-A002 |
| AC-009 | C | 4D research_step / research_decompose_level / web_research_step absent from 3B 17-type CHECK. | P4D-C001, P4D-A001 |
| AC-010 | A | 4D Interactive present_options concrete UI surface open. | P4D-A003, P4D-I001 |
| AC-011 | A | 4D several VRAM context_ceiling values projected not measured. | P4D-A002 |
| AC-012 | A | pre-phase-0 full cfg-gated test inventory not pre-enumerated. | PP0-A001 |
| AC-013 | A | pre-phase-0 HookRunner /bin/sh hardcode status left as audit. | PP0-A002 |

## Deltas against `01-phase-index-MASTER.discovery.md`

Summary artifact: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. Anchors use `01-phase:` line numbers.

### pre-phase-0 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-PP0-001 | **ADD** exact file/line sites for PermissionsExt and set_mode (7 sites / 3 files) and Windows no-op chmod pattern. | 01-phase:L13-L14 | PP0-E006–PP0-E008 |
| SD-PP0-002 | **ADD** named failing plugin tests and Windows .cmd / cmd.exe shell dispatch requirements. | 01-phase:L15-L16 | PP0-E010–PP0-E013 |
| SD-PP0-003 | **ADD** named tools failures (path ends_with, chmod, python3→python, env PoisonError) and 10-run adversarial stability. | 01-phase:L17-L18 | PP0-E014–PP0-E019 |
| SD-PP0-004 | **ADD** exact warning sites, pre_phase_0.toml fields, measurements, grounding table, and WIRING_CONTRACT archive-verify surface. | 01-phase:L19-L22 | PP0-E020–PP0-E025 |
| SD-PP0-005 | **ADD** R-pre1/R-pre2 mitigations and cfg-gate-not-delete rule. | 01-phase:L10-L11 | PP0-E003–PP0-E004 |
| SD-PP0-006 | **FLAG** incomplete pre-enumeration of all cfg-gated tests and HookRunner shell audit. | 01-phase:L21-L22 | PP0-A001–PP0-A002 |

### phase-1I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P1I-001 | **ADD** ACTION-REQUIRED 1G→1I dep-graph fix and compile-failure scenario if ignored. | 01-phase:L147-L147 | P1I-E002, P1I-C001 |
| SD-P1I-002 | **ADD** permanent-vs-transient classification, 2-success HALF_OPEN, fresh InferenceSession, deterministic model_id jitter details. | 01-phase:L148-L149 | P1I-E007–P1I-E011 |
| SD-P1I-003 | **ADD** full cascade chain including TEXT-only/CODE-only intermediate modes and SPLADE-disabled collapse. | 01-phase:L150-L151 | P1I-E013–P1I-E018 |
| SD-P1I-004 | **ADD** newline-accepting validation tests, canary_reference_vec first-startup generation, DEGRADED⇒BM25_ONLY invariant tests. | 01-phase:L152-L154 | P1I-E020–P1I-E025 |
| SD-P1I-005 | **ADD** batching merge/segregate rules for cross-encoder queue and overflow returns unranked non-empty candidates. | 01-phase:L155-L155 | P1I-E026–P1I-E027 |
| SD-P1I-006 | **ADD** full QueryTrace/StageTrace field lists, nine stage names, DEBUG-not-SSB rule, session-end aggregates. | 01-phase:L156-L156 | P1I-E028–P1I-E031 |
| SD-P1I-007 | **ADD** index_version schema fields, archive-on-failure abort, user notification, isolation-under-degradation verification ownership. | 01-phase:L157-L158 | P1I-E032–P1I-E036 |
| SD-P1I-008 | **ADD** MEASURE targets and Phase-1 overall measurement list. | 01-phase:L145-L158 | P1I-E037–P1I-E039 |

### phase-2D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P2D-001 | **ADD** design decisions D1–D5 (no-DKG detection, severity classes, separate inline counter, ReplanRequest MAJOR, type inventory). | 01-phase:L220-L226 | P2D-E004–P2D-E011 |
| SD-P2D-002 | **ADD** full T1–T10 TDD inventory including CONTRADICTS_PLAN: marker and BudgetExhausted topic. | 01-phase:L228-L229 | P2D-E013–P2D-E015 |
| SD-P2D-003 | **ADD** exact Critical pattern list, case-insensitive compare, assumption/consequence templates, guard_added=false default. | 01-phase:L227-L227 | P2D-E016–P2D-E018 |
| SD-P2D-004 | **ADD** phase_2D.toml fields and overhead/false-positive SLOs; architecture-overview update requirement. | 01-phase:L231-L231 | P2D-E019 |
| SD-P2D-005 | **ADD** grounding summary distinguishing GROUNDED/VERIFIED/UNGROUNDED claims and 4F deferrals. | 01-phase:L230-L230 | P2D-E020–P2D-E022 |
| SD-P2D-006 | **FLAG** D3 method list vs SourceMatch-only type conflict and T4 `.passed` field wording. | 01-phase:L224-L229 | P2D-C001, P2D-A001–P2D-A002 |

### phase-3B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P3B-001 | **ADD** R10/R-3B-1/2/3 risks, r2d2 WAL cleanup, CHECK migration drop+recreate session semantics. | 01-phase:L246-L249 | P3B-E006–P3B-E007 |
| SD-P3B-002 | **ADD** full SQL schema, indexes, ssb_files junction, and note that orchestrator.txt §3 synopsis is incomplete. | 01-phase:L249-L252 | P3B-E009–P3B-E012 |
| SD-P3B-003 | **ADD** complete typed payload struct inventory including PhaseComplete fsync-before-unload semantics. | 01-phase:L256-L256 | P3B-E023–P3B-E024 |
| SD-P3B-004 | **ADD** write-loop BEGIN IMMEDIATE, oneshot seq, shutdown drain, broadcast-after-commit, Arc payload, CustomizeConnection. | 01-phase:L253-L254 | P3B-E017–P3B-E021, P3B-E027 |
| SD-P3B-005 | **ADD** SsbTraceable boundary, msgpack disk serialization, 1MB payload reject, full unit/property/adversarial/determinism suite and measure SLOs. | 01-phase:L255-L257 | P3B-E025–P3B-E029 |
| SD-P3B-006 | **ADD** audit-fix history BLOCK1–5 WARN1–6 from s-1427264. | 01-phase:L246-L257 | P3B-E031 |
| SD-P3B-007 | **FLAG** internal 15-vs-17 payload wording conflict. | 01-phase:L250-L250 | P3B-C001, P3B-A001 |

### phase-4D deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P4D-001 | **ADD** Designer role config, SystemConstraints sizing defaults, confidence/ranking formulas, adaptive complexity table, info_gain prune rule. | 01-phase:L426-L434 | P4D-E004–P4D-E016 |
| SD-P4D-002 | **ADD** full 4D.1–4D.6 TDD/adversarial gates, FORMULATE-SEARCH-EXTRACT-REFLECT loop, cloud-only speculation, measure keys, RuntimeClient wire rule. | 01-phase:L435-L435 | P4D-E019–P4D-E027 |
| SD-P4D-003 | **ADD** recursive 4-level protocol, all decomposition/baby-step block types, ResearchCache, DecisionSource, package anti-hallucination gate, Interactive/Autonomous selection. | 01-phase:L436-L440 | P4D-E028–P4D-E037 |
| SD-P4D-004 | **ADD entire web-research extension 4D.11–4D.18**: SearXNG client, WebFetcher/robots/readability, SourceQualityTable ≥200 domains, Web A-RAG, six VRAM tiers, budget manager, WebResearchOrchestrator, E2E metrics file. | 01-phase:L425-L440 | P4D-E038–P4D-E056 |
| SD-P4D-005 | **ADD** CLI `--vram-tier`, INTERACTIVE progress via research_step events, design principles from survey papers. | 01-phase:L440-L440 | P4D-E038, P4D-E049, P4D-E057 |
| SD-P4D-006 | **FLAG CONFLICT** research_* SSB action types not in 3B 17-type schema; Interactive present_options UI unspecified; projected VRAM ceilings. | 01-phase:L425-L440 | P4D-C001, P4D-A001–P4D-A003 |
| SD-P4D-007 | **CORRECT/EXPAND** summary DecisionSource labels: source uses Research/UserChoice/AutoSelected (not RESEARCH\|EXISTING_PROJECT\|USER_SPECIFIED\|FALLBACK). | 01-phase:L439-L439 | P4D-E032 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **5/5 files**, **2,723/2,723 logical lines**, **166,521/166,521 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **198 unique atoms** = **175 explicit**, **005 inferred**, **014 ambiguous**, **004 conflicting/corrective**.
- Per-file atom counts: pre-phase-0 **028**; phase-1I **044**; phase-2D **028**; phase-3B **035**; phase-4D **063**. Sum **198**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **34** (`SD-*`). Ambiguity/conflict register: **13** (`AC-001`–`AC-013`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `PP0/P1I/P2D/P3B/P4D-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: every distinct CLI verb/flag (`elai archive verify`, `--vram-tier`), metrics artifact path, user-notification (index rebuild), Interactive present_options selection, and INTERACTIVE research progress event family named in these sources was given its own SURFACE/EVENT atom rather than folded into backend bullets. No Tauri window/panel is defined in this chunk's sources.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the five exact DEEP-PHASE-007 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0389`; `output_bytes=059242`.
- Canonical normalized output SHA-256: 204cb23e3c051ffb12650911f3ab2a5649f0b528a5bc3768cafe65d3d8c9d075.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
