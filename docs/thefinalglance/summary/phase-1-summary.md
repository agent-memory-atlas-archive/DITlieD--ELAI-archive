PHASE 1 LOSSLESS SUMMARY (Pre-Phase 0 through Phase 1I)
Source files: F:\.ELAI_workflow\docs\specs\guide\steps\
Project root for code: F:\.ELAI_workflow\.elai_cc\crates\knowledge\

================================================================================
## PHASE PRE-0: WINDOWS PLATFORM FIXES (clean base for elai_cc)
================================================================================

Prerequisites: claw-code-rust-port cloned into .elai_cc/
Platform: Windows 11 (primary dev environment)

GOAL: Make claw-code-rust-port compile AND pass all tests on Windows. Zero compile errors, zero test failures, zero warnings. No feature changes. Pure platform compatibility.

RISKS:
- R-pre1: some tests inherently Unix-only (shell script execution). Mitigation: cfg-gate (not delete).
- R-pre2: fixing path separators may break Linux. Mitigation: std::path::MAIN_SEPARATOR or Path methods.

BASELINE (measured 2026-04-06):
- cargo check: 40.15s, 0 errors, 2 warnings
- cargo test: 10 compile errors (2 root causes), 8 runtime failures (4 root causes)
- .rs files: 69, LOC: 61,317, test annotations: 684

### P0.1 FIX COMPILE ERRORS: PermissionsExt / std::os::unix (10 errors, 3 files)
PRE_GATE:
- cargo check passes (lib code compiles, only test code fails)
- cargo test 2>&1 | grep "^error\[" | wc -l returns 10
TASK files:
- crates/runtime/src/mcp_stdio.rs: Line 1413 use std::os::unix::fs::PermissionsExt (wrap in #[cfg(unix)]); Lines 1456, 1502, 1636, 1761, 2680: permissions.set_mode(0o755) wrap each in #[cfg(unix)] block; add #[cfg(windows)] no-op.
- crates/runtime/src/mcp_tool_bridge.rs: Line 317 use std::os::unix::fs::PermissionsExt; Line 434 set_mode(0o755).
- crates/rusty-claude-cli/tests/mock_parity_harness.rs: Line 4 import; Line 425 set_mode(0o755).
Pattern: `#[cfg(unix)] { permissions.set_mode(0o755); fs::set_permissions(&path, permissions).expect("chmod"); }` // Windows: no chmod equivalent
DETERMINISTIC_CHECK:
- cargo test --no-run 2>&1 | grep "^error" | wc -l returns 0
- grep -rn "use std::os::unix" crates/ --include="*.rs" | grep -v "#\[cfg(unix)\]" returns 0
ADVERSARIAL: cfg gates break Unix path -> Linux CI catches. set_mode outside cfg block -> compile error.
ACCEPTANCE: cargo test --no-run: 0 compile errors across all crates.

### P0.2 FIX RUNTIME FAILURES: PLUGIN HOOK TESTS (4 failures)
PRE_GATE: P0.1 complete; cargo test --no-run passes
TASK file: crates/plugins/src/hooks.rs (test module, lines 346-498)
Failing tests:
- collects_and_runs_hooks_from_enabled_plugins (line 395) — write_hook_plugin creates hooks/pre.sh with #!/bin/sh; Fix: write .cmd on Windows (echo message)
- pre_tool_use_denies_when_plugin_hook_exits_two (line 456) — "printf 'blocked by plugin'; exit 2"; Fix Windows: "cmd /c \"echo blocked by plugin & exit /b 2\""
- propagates_plugin_hook_failures (line 473) — "printf 'broken plugin hook'; exit 1"; Fix Windows: "cmd /c \"echo broken plugin hook & exit /b 1\""
Also fix write_hook_plugin (line 362): #[cfg(unix)] keep .sh; #[cfg(windows)] write .cmd with @echo off + echo message.
HookRunner::run_hook_command (~line 326) may hardcode /bin/sh — if so add #[cfg(windows)] cmd.exe.
DETERMINISTIC_CHECK: cargo test -p plugins --lib reports "33 passed; 0 failed".
ACCEPTANCE: cargo test -p plugins --lib: 0 failures on Windows.

### P0.3 FIX RUNTIME FAILURES: TOOLS TESTS (4 failures)
PRE_GATE: P0.2 complete: plugin tests pass
TASK file: crates/tools/src/lib.rs (test module)
Failing tests:
- glob_and_grep_tools_cover_success_and_errors (line 6672) — asserts ends_with("nested/lib.rs") but Windows uses backslash. Fix: normalize separators or Path::ends_with.
- powershell_runs_via_stub_shell (line 7069) — calls chmod on stub shell script that doesn't exist on Windows. Fix: cfg-gate Unix chmod.
- repl_executes_python_code (line 7007) — exit 9009 = "python3 not recognized" on Windows. Fix: "python" on Windows, "python3" on Unix.
- skill_loads_local_skill_prompt (line 5745) — PoisonError on env lock. Fix: dedicated mutex per test or catch_unwind, or #[serial] using serial_test crate.
DETERMINISTIC_CHECK: cargo test -p tools --lib reports "61 passed; 0 failed".
ACCEPTANCE: 0 failures on Windows.

### P0.4 FIX WARNINGS (2 warnings)
PRE_GATE: P0.3 complete
TASK:
- crates/plugins/src/hooks.rs line 2: unused import std::path::Path -> remove or #[allow]
- crates/runtime/src/hooks.rs line 636: unused mut on command_builder -> remove mut
DETERMINISTIC_CHECK: cargo check 2>&1 | grep "^warning:" | grep -v "generated\|Finished\|build" | wc -l returns 0
ACCEPTANCE: 0 errors, 0 warnings.

### P0.5 VERIFY CLEAN BASELINE
PRE_GATE: P0.4 complete
TASK: cargo check (0/0), cargo test all pass; record metrics to .elai_cc/metrics/pre_phase_0.toml: compile_time, test_total, test_pass, test_fail, loc_total, warnings.
DETERMINISTIC_CHECK: .elai_cc/metrics/pre_phase_0.toml exists and test_fail = 0
ACCEPTANCE: 0 errors, 0 warnings, 0 failures, all cfg-gated tests documented.

EXIT: cargo check 0/0 on Windows; cargo test 0 failures; no behavior changes; Unix-only tests cfg-gated.

GROUNDING: 10 compile errors VERIFIED; set_mode 7 sites VERIFIED; std::os::unix 3 sites VERIFIED; 4 plugin failures VERIFIED; 4 tools failures VERIFIED; 2 warnings VERIFIED; exit code 9009 VERIFIED; PoisonError from env lock VERIFIED at lib.rs:5745; 61,317 LOC, 69 files, 684 tests VERIFIED 2026-04-06.


================================================================================
## PHASE 0: FORK AND STRIP (the walking skeleton)
================================================================================

Prerequisites: pre-phase-0 complete (Windows fixes)

GOAL: claw-code-rust-port compiles as elai_cc with all unnecessary code removed. One model (ollama), one FSM loop (IDLE->EXECUTING->IDLE), one tool (file read).

RISKS: R17 claw-code compilation verified (26.24s, 0 errors); R18 module quality unknown; R19 runtime crate is 25,035 LOC across 38 files; R20 entropy quarantine 60 occurrences across 31 files; R21 base-fork-comparison.txt LOC numbers stale (green_contract.rs listed as 4,061, actual 152); R22 config.rs imports sandbox::{SandboxConfig, FilesystemIsolationMode}; R23 TokenUsage/UsageTracker embedded in Session/conversation.rs/main.rs (61 occurrences across 5 files); R24 tools/lib.rs imports api::*; R25 commands/lib.rs has 28+ claw-code slash commands.

DESIGN DECISIONS:
- STRIP BEFORE RESTRUCTURE: Strip ~4K+ LOC of known-bad code first.
- WORKSPACE PATH MIGRATION: Steps 0.1-0.3 in .elai_cc/rust/; Step 0.4 creates .elai_cc/crates/ with .elai_cc/Cargo.toml workspace root.
- ONNX MODEL CANONICAL PATH: .elai_cc/models/onnx/ (pre-staged, not auto-downloaded). Setup: download + blake2 hash verify + write index_version.toml. Missing models -> DEGRADED (BM25_ONLY), not crash. Hidden via visibility filter.
- TOKENUSAGE HANDLING (R23): usage.rs (313 LOC) STRIPPED; minimal elai_cc_core::TokenUsage struct with input_tokens/output_tokens/total_tokens/estimated_cost_usd. Strip UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing.
- PROVIDER TRAIT MIGRATION (WARN-7): keep old Provider trait during Phase 0. New OllamaProvider implements LlmProvider directly. Mark old with #[deprecated].

### 0.1 BASELINE
PRE_GATE: pre-phase-0 done; cargo --version OK; .elai_cc/rust/Cargo.toml exists.
TASK: cargo check --workspace (baseline 26.24s, 0 errors), cargo test --workspace, record metrics, create .elai_cc/migration_manifest.toml classifying every .rs file as KEEP/STRIP/REPLACE/EXTEND.
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_0_baseline.toml with compile_time_s, test_pass, test_fail, test_total, loc_total, loc_by_crate; manifest grep KEEP|STRIP|REPLACE|EXTEND = 69.
ACCEPTANCE: 100% .rs files (69) classified.

### 0.2 STRIP (a-h sub-tasks)
PRE_GATE: 0.1 complete; manifest has STRIP entries.
TASK (each followed by cargo check --workspace):
a. telemetry/ crate (526 LOC): remove dir + workspace Cargo.toml + dep entries.
b. compat-harness/ crate (~357 LOC): STRIP (TypeScript-bridge, dead code). Remove call sites in main.rs (extract_manifest, UpstreamPaths at ~L1274).
c. runtime/src/usage.rs (313 LOC) + compact.rs (696 LOC) + summary_compression.rs (300 LOC): STRUCTURAL STRIP. Define minimal TokenUsage in runtime/src/lib.rs (input_tokens: u64, output_tokens: u64) — temp shim replaced by core::TokenUsage in 0.4a. Update Session, remove UsageTracker/CompactionConfig/format_usd/pricing_for_model/ModelPricing. main.rs: remove cost reporting, compaction config, billing displays. conversation.rs: remove compaction triggers. Delete the 3 files.
d. runtime/src/lsp_client.rs (747 LOC): STRIP.
e. runtime/src/oauth.rs (596 LOC): STRIP — DEFERRED.
f. Audit runtime/src/prompt.rs (803 LOC): remove SYSTEM_PROMPT_DYNAMIC_BOUNDARY (L39), "You are an interactive agent" (L451). REPLACE with minimal builder or STRIP.
g. Classify commands/lib.rs slash commands (28+): KEEP candidates: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens. STRIP: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes. Record in manifest. Actual removal at 0.4c.
h. runtime/src/remote.rs (401 LOC): evaluate. Remote execution: DEFERRED. Cloud telemetry: STRIP.
NOTE: "sentinel"/"anti_distill"/"bias_inject" patterns DO NOT exist in codebase (verified 2026-04-06).
DETERMINISTIC_CHECK: stripped files don't exist; no dangling refs (compact::, summary_compression::, usage::, lsp_client::); no "You are an interactive agent"; cargo check 0 errors.

### 0.3 VERIFY STRIP
PRE_GATE: 0.2 complete; cargo check 0 errors.
TASK: cargo test --workspace, classify failures EXPECTED vs BUG, fix BUGs, mark STRIPPED in manifest, compute strip delta.
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_0_strip_delta.toml with loc_removed, files_removed, tests_removed, tests_broken_expected, tests_broken_bug.
ACCEPTANCE: 0 errors; all BUGs fixed; strip delta recorded.

### 0.4a RESTRUCTURE: LEAF CRATES (core, rules, providers)
PRE_GATE: 0.3 complete.
TASK: Create .elai_cc/Cargo.toml workspace root, crate dirs under .elai_cc/crates/.
1. core/: extract config.rs (1648 LOC), session_control.rs (459 LOC), json.rs (358 LOC), bootstrap.rs (111 LOC). Move sandbox.rs (385 LOC) — option (a) move entire file or (b) extract SandboxConfig + FilesystemIsolationMode to core/src/sandbox_types.rs. Define core::TokenUsage. Split session_control.rs: identity -> core/, state -> orchestrator/.
2. rules/: policy_engine.rs (581 LOC), green_contract.rs (152 LOC), trust_resolver.rs (299 LOC).
3. providers/: absorb api/src/ (4377 LOC), move runtime/src/sse.rs (158 LOC). Update all api:: imports to providers:: (R24: tools/lib.rs critical). Mark old Provider trait #[deprecated].
DETERMINISTIC_CHECK: 3 crate dirs with Cargo.toml; cargo check 0 errors; no "use api::"; core/ no deps on orchestrator/plugins/cli.
ACCEPTANCE: leaf crates compile; core/ true leaf.

### 0.4b RESTRUCTURE: MID-LAYER CRATES (tools, plugins)
PRE_GATE: 0.4a complete.
TASK:
4. tools/: extract file_ops.rs (762 LOC), bash.rs (336 LOC), bash_validation.rs (1004 LOC), stale_branch.rs (417 LOC), branch_lock.rs (144 LOC). Cargo.toml depends on core/ (not runtime/). tools/lib.rs (7345 LOC): split if >3 distinct tools.
5. plugins/: extract hooks.rs (987 LOC), plugin_lifecycle.rs (533 LOC). Depends on core/.
6. knowledge/: empty skeleton (Phase 1 fills). Cargo.toml only, depends on core/.
7. reasoning/: empty skeleton (Phase 2 fills). Cargo.toml only, depends on core/.
DETERMINISTIC_CHECK: 7 crate dirs; tools/plugins no longer depend on runtime/; no .rs file in tools/ exceeds 2000 LOC after restructure.
ACCEPTANCE: cargo check 0 errors.

### 0.4c RESTRUCTURE: TOP-LAYER (orchestrator, cli) + CLEANUP
PRE_GATE: 0.4b complete.
TASK:
8. orchestrator/: conversation.rs (1690), session.rs (1246), worker_boot.rs (1083), mcp_stdio.rs (2928), mcp_tool_bridge.rs (920), mcp_lifecycle_hardened.rs (843), mcp.rs (304), mcp_client.rs (248), permissions.rs (683), permission_enforcer.rs (551), sandbox.rs (385), recovery_recipes.rs (631). EVALUATE lane_events.rs (383), task_packet.rs (158), task_registry.rs (503), team_cron_registry.rs (509). Depends on core/, providers/, tools/, rules/.
9. cli/: absorb rusty-claude-cli/ + commands/. main.rs (8615 LOC) -> cli/src/main.rs. Strip claw-code commands. render.rs (796), init.rs (434), input.rs (330) -> cli/src/. mock-anthropic-service/ -> dev-dep. Target: cli/src/main.rs <500 LOC.
10. Cleanup: archive .elai_cc/rust/ (-> .elai_cc/rust_archived/); remove old crate dirs (api/, commands/, rusty-claude-cli/, runtime/); workspace lists exactly 9 crates.
DETERMINISTIC_CHECK: 9 members; each crate has Cargo.toml (core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli); cargo check 0 errors; no "use api::|use runtime::|use commands::|use rusty_claude".
ACCEPTANCE: 9 crate directories; manifest has correct target crate per file.

### 0.5 VERIFY RESTRUCTURE
PRE_GATE: 0.4c complete.
TASK: cargo test --no-run --workspace; cargo test --workspace; classify RESTRUCTURE_BUG vs PATH_UPDATE; fix all; record metrics.
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_0_restructure.toml with compile_time_s, test_pass, test_fail, loc_per_crate, total_crates=9.
ACCEPTANCE: 0 errors, all non-stripped tests pass, per-crate LOC recorded.

### 0.6 CONTRACT TESTS (skeleton)
PRE_GATE: 0.5 complete.
TASK: Create .elai_cc/crates/contracts/ test-only crate (no lib.rs, integration tests in tests/), [package] name = "elai_cc_contracts", [dev-dependencies] on all 9 crates. Add to workspace.
Contract test files (5 — all import elai_cc_* names):
- crates/contracts/tests/provider_trait.rs: LlmProvider trait in elai_cc_providers, has complete/embed/token_count
- crates/contracts/tests/fsm_basic.rs: FSM IDLE -> EXECUTING -> IDLE
- crates/contracts/tests/tool_basic.rs: ToolRegistry register + execute file_read tool
- crates/contracts/tests/config_basic.rs: elai_cc.toml parses; round-trips via msgpack
- crates/contracts/tests/visibility_filter.rs: hidden path returns NotFound (identical to missing); glob on hidden returns empty; .elai_cc/** + .elai_cc/models/** covered by hidden_paths
All FAIL at runtime (TDD red phase).
DETERMINISTIC_CHECK: crate exists; 5 test files exist; cargo test -p elai_cc_contracts --no-run returns 0; runtime grep -c "FAILED" returns 5.
ACCEPTANCE: 5 test files, all compile + fail at runtime, all imports use elai_cc_* names.

### 0.7a IMPLEMENT PROVIDER + CONFIG
PRE_GATE: 0.6 complete (5 contract tests fail).
TASK: Extend LlmProvider trait in providers/:
- async fn complete(&self, prompt: &str, config: &CompletionConfig) -> Result<impl Stream<Item=Token>>;
- async fn embed(&self, text: &str) -> Result<Vec<f32>>;
- fn token_count(&self, text: &str) -> usize;
- fn token_count_estimate(&self, text: &str) -> usize;
All take &self.
OllamaProvider implementing LlmProvider: HTTP wrapper (POST /api/generate, POST /api/embed); configurable base_url default http://localhost:11434; streaming response (newline-delimited JSON).
elai_cc.toml schema in core/: provider config + project settings + agent role configs. toml crate. Round-trip via msgpack (rmp-serde).
DETERMINISTIC_CHECK: provider_trait + config_basic pass; OllamaProvider base_url configurable.
ACCEPTANCE: tests pass; configurable base_url; msgpack round-trips without loss.

### 0.7b IMPLEMENT FSM + TOOL
PRE_GATE: 0.7a complete.
TASK: Minimal FSM in orchestrator/: IDLE/EXECUTING only; transitions IDLE->EXECUTING (task start), EXECUTING->IDLE (complete/error); state as enum; invalid -> Error not panic; methods get_state(), transition(), get_active_tools().
file_read tool in tools/: read contents with metadata header (total_lines, estimated_tokens via token_count_estimate); respect visibility filter (hidden -> NotFound).
Replace JSON session storage with msgpack in orchestrator/: rmp-serde; round-trip test verifies file is valid msgpack.
DETERMINISTIC_CHECK: fsm_basic + tool_basic pass; session file is msgpack (verify via python msgpack.unpack).
ACCEPTANCE: FSM returns Error on invalid; file_read respects visibility; msgpack persistence verified.

### 0.7c IMPLEMENT INTERFACE HARDENING
PRE_GATE: 0.7b complete (4 tests pass).
TASK:
- Slash-command routing in cli/: /task, /config, /continue, /explore, /status; fallback classifier (default /task); each command -> handler; unknown -> error. Build on KEEP from 0.2g; strip others.
- Visibility filter in tools/: per-agent allowed_paths + hidden_paths; path canonicalization (block ../); hidden returns NotFound. Default hidden_paths: .elai_cc/models/**, rules.db, harness source, other agent checkpoints, SSB database, elai_cc binary internals.
- Error message sanitization: strip internal IDs from model-facing output via allowlist (not denylist).
- Tool schema sanitization: strip internal-only parameters.
- Tracing-based logging (replace telemetry): tracing + tracing-subscriber; log every tool call, FSM transition, file operation.
DETERMINISTIC_CHECK: visibility_filter passes; slash-command tests pass; no internal IDs leak (rule_id, predicate_type, hook_name).
ACCEPTANCE: all 5 contract tests pass; visibility blocks traversal; allowlist sanitization; tracing captures.

### 0.8 VERIFY IMPLEMENTATION
PRE_GATE: 0.7c complete.
TASK: All contract tests pass; cargo check clean; smoke test ollama (WARN if unavailable); record metrics.
DETERMINISTIC_CHECK: .elai_cc/scripts/smoke_test_ollama.sh; .elai_cc/metrics/phase_0_implement.toml with compile_time_s, test_total, test_pass, loc_delta.
ACCEPTANCE: tests pass; ollama OK or env-skip documented.

### 0.9 ENTROPY QUARANTINE (deterministic simulation prerequisite)
PRE_GATE: 0.8 complete.
TASK: Define ClockProvider and EntropySource traits in core/:
  trait ClockProvider { fn now(&self) -> LogicalInstant; fn elapsed_since(&self, mark: LogicalInstant) -> Duration; fn system_now(&self) -> SystemTime; }
  trait EntropySource { fn next_u64(&mut self) -> u64; fn fill_bytes(&mut self, buf: &mut [u8]); }
&self on ClockProvider; &mut self on EntropySource.
Production impls in core/: OsClock (std::time::Instant), OsRng (rand::thread_rng()).
Test impls in core/: SeededClock (logical clock, advances via tick(), monotonic), SeededRng (ChaCha8Rng::seed_from_u64).
Replace ALL raw time/entropy: 60 occurrences / 31 files.
Known high-density (post-restructure):
- providers/prompt_cache.rs: 4 SystemTime::now()
- tools/lib.rs: 14 Instant::now()
- cli/main.rs: 5 Instant::now()
- providers/anthropic.rs: 2 SystemTime::now()
- orchestrator/session_control.rs: 3 Instant::now()
Bundle as RuntimeContext { clock: Arc<dyn ClockProvider>, entropy: Arc<Mutex<dyn EntropySource>> }.
Legitimate exceptions: core/src/clock.rs (OsClock), core/src/entropy.rs (OsRng).
CI script: .elai_cc/scripts/check_entropy_quarantine.sh — greps banned patterns, excludes those two files + TEST_KEEP files.
Audit transitive deps: cargo tree -e features | grep "time\|rand\|getrandom".
DETERMINISTIC_CHECK: greps for Instant::now()/SystemTime::now() return only core/src/clock.rs (+TEST_KEEP); thread_rng/OsRng::new/getrandom only in core/src/entropy.rs; trait ClockProvider exists; trait EntropySource exists; CI script returns 0; struct RuntimeContext exists.
ACCEPTANCE: raw access only in core/ impls (+TEST_KEEP); CI catches violations; SeededClock+SeededRng pass contract tests; RuntimeContext bundles both.

### 0.10 VERIFY ENTROPY QUARANTINE + DETERMINISM PROOF
PRE_GATE: 0.9 complete.
TASK: Run CI quarantine check; ELAI_SEED=42 cargo test --workspace passes; same seed twice single-threaded -> diff = 0 (determinism proof for Phase 3I DST).
DETERMINISTIC_CHECK: script returns 0; ELAI_SEED=42 passes; --test-threads=1 ran twice diff=0.
ACCEPTANCE: zero raw entropy access outside core/; tests pass with seeded providers; same seed = identical output.

EXIT CRITERIA:
- All 5 contract tests pass
- Ollama round-trip works (or environmental skip)
- No stripped code remains
- Zero raw time/entropy outside core/ (entropy quarantine + determinism proven)
- 9-crate structure with no circular deps
- claw-code slash commands stripped per classification

MODULE MAPPING (runtime/ -> target crates):
TO core/: config.rs (1648), session_control.rs (459, SPLIT identity/state), json.rs (358), bootstrap.rs (111), sandbox.rs (385).
TO rules/: policy_engine.rs (581, REPLACE in 3F), green_contract.rs (152), trust_resolver.rs (299).
TO providers/: sse.rs (158), api/src/* (4377). NOTE api/src/sse.rs (279) duplicates runtime/src/sse.rs (158) — keep api/ version.
TO tools/: file_ops.rs (762), bash.rs (336), bash_validation.rs (1004), stale_branch.rs (417), branch_lock.rs (144), tools/src/* (7526).
TO plugins/: hooks.rs (987), plugin_lifecycle.rs (533), plugins/src/* (3860).
TO orchestrator/: conversation.rs (1690), session.rs (1246), worker_boot.rs (1083), mcp_stdio.rs (2928), mcp_tool_bridge.rs (920), mcp_lifecycle_hardened.rs (843), mcp.rs (304), mcp_client.rs (248), permissions.rs (683), permission_enforcer.rs (551), recovery_recipes.rs (631), lane_events.rs (383, EVAL), task_packet.rs (158, EVAL), task_registry.rs (503, EVAL), team_cron_registry.rs (509, EVAL).
TO cli/: rusty-claude-cli/src/main.rs (8615, target <500), render.rs (796), init.rs (434), input.rs (330), commands/src/lib.rs (4841).
STRIP: compact.rs (696), summary_compression.rs (300), usage.rs (313), lsp_client.rs (747), oauth.rs (596 DEFERRED), prompt.rs (803), remote.rs (401 EVAL), telemetry/src/lib.rs (526), compat-harness/src/lib.rs (~357).
TEST INFRA: mock-anthropic-service/ (~180) -> dev-dep.

================================================================================
## PHASE 1A: DKG (tree-sitter -> SQLite graph)
================================================================================

Prerequisites: Phase 0 complete (9-crate layout, cargo check passes).
Canonical path: .elai_cc/crates/knowledge/. Resolve dynamically: KNOW_PATH=$(find .elai_cc -path "*/knowledge/Cargo.toml" | head -1 | xargs dirname).

GOAL: Parse source code into dependency knowledge graph stored in SQLite. Layer 1. Downstream: Phase 1D PageRank, Phase 1E CatRAG, Phase 1B BM25 (independent but shares readiness gate).

NOTE: knowledge.txt header corrected to "5 major + 2 sub-layers" (architecture-overview.txt L100 says 7 counting sub-layers 3.25 and 3.5).

RISKS: R7 grammar gaps (start Rust/Python/TypeScript/Go); R-1A-1 SQLite schema not in specs; R-1A-2 no build time baseline (<15s claim UNGROUNDED); R-1A-3 Phase 1D PRE_GATE requires edge/adjacency iteration API; R-1A-4 parse failure fallback DKG-only (BM25/vector deferred); R-1A-5 LanguageGrammar trait must be object-safe (&self); R-1A-6 DkgNodeInfo must include signature+SymbolKind for Phase 1D contract.

COMPLIANCE SCOPE (knowledge.txt Sec 8 5 items):
1. File excluded from DKG -> IMPLEMENTED in 1A
2. File STILL indexed by BM25 -> DEFERRED to 1B
3. File STILL embedded by vectors -> DEFERRED to 1C
4. PageRank treats file as leaf -> DEFERRED to 1D
5. Parse failure logged, retried on next file change -> IMPLEMENTED in 1A

### 1A.1 SKELETON
PRE_GATE: Phase 0 layout exists; Phase 0 compiles clean; no prior DKG code.
TASK:
- Resolve KNOW_PATH.
- Add deps to Cargo.toml (verified crates.io 2026-04-06):
  tree-sitter = "0.26"  (0.26.8 latest)
  tree-sitter-rust = "0.24"  (0.24.2 latest)
  rusqlite = { version = "0.39", features = ["bundled"] }
  blake2 = "0.10"
  [dev-dependencies] proptest = "1"
  Future: tree-sitter-python "0.25", tree-sitter-go "0.25", tree-sitter-typescript "0.23".
- Define KnowledgeEngine trait in lib.rs (matching architecture-overview.txt L94-104):
  fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph>
  fn query(&self, query: &str, strategy: QueryStrategy) -> Result<Vec<CodeChunk>>
  fn rerank(&self, query: &str, candidates: Vec<CodeChunk>) -> Result<Vec<CodeChunk>>
  fn update(&mut self, file_changed: &Path) -> Result<()>
  fn readiness_check(&self) -> ReadinessStatus
- Define DependencyGraph { node_count: usize, edge_count: usize, files_parsed: usize }.
- ReadinessStatus enum: READY, DEGRADED, UNAVAILABLE.
- KnowledgeEngine.readiness_check() is FULL ENGINE aggregate (Phase 1I). DkgEngine has its own dkg_readiness() -> DkgReadiness method.
- LanguageGrammar trait (object-safe): fn language_id(&self) -> &'static str; fn tree_sitter_language(&self) -> tree_sitter::Language; fn extract_nodes(&self, tree: &Tree, source: &[u8]) -> Vec<DkgNode>. All methods take &self.
- SymbolKind enum: Function, Struct, Impl, Trait, Enum, Module, Use, TypeAlias, Other.
- DkgNodeInfo struct (matching Phase 1D contract):
  pub struct DkgNodeInfo { pub node_id: i64, pub symbol_name: String, pub file_path: PathBuf, pub signature: String, pub kind: SymbolKind, pub start_line: u32, pub end_line: u32 }
  Signature reconstruction: function_item "fn name(params) -> return_type"; struct_item "struct Name { fields... }"; trait_item "trait Name"; enum_item "enum Name".
- DkgEngine struct in src/dkg/mod.rs. DkgEngine::new(db_path: &Path) — explicit path for project-scoped isolation. db_path: .elai_cc/projects/{project_id}/knowledge.db (caller provides).
- SQLite schema:
  Table dkg_nodes: id INTEGER PK, file_path TEXT NOT NULL, name TEXT NOT NULL, kind TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL, end_line INTEGER NOT NULL, parent_id INTEGER REFERENCES dkg_nodes(id), concept TEXT DEFAULT NULL, concept_stale INTEGER NOT NULL DEFAULT 1, file_hash BLOB NOT NULL.
  Table dkg_edges: source_id INTEGER REFERENCES dkg_nodes(id) ON DELETE CASCADE, target_id INTEGER REFERENCES dkg_nodes(id) ON DELETE CASCADE, edge_type TEXT NOT NULL, PRIMARY KEY (source_id, target_id, edge_type).
  Edge types: "imports" (use_declaration), "contains" (parent->child), "implements" (impl->trait).
  Table dkg_files: file_path TEXT PK, file_hash BLOB NOT NULL, parse_status TEXT NOT NULL DEFAULT 'pending' ('ok'|'failed'|'pending'), parse_error TEXT DEFAULT NULL, last_indexed INTEGER NOT NULL.
  Table dkg_meta: key TEXT PK, value TEXT. Initial: ('schema_version', '1'), ('created_at', unix_timestamp).
  Indexes: idx_nodes_name ON dkg_nodes(name), idx_nodes_file ON dkg_nodes(file_path), idx_nodes_kind ON dkg_nodes(kind), idx_files_status ON dkg_files(parse_status).
- index_version.toml at project-scoped path: [index] version=1, dkg_schema_version=1, vector_dimensions={code=0,text=0}, bm25_format="", created_at=<unix>. All 5 fields per knowledge.txt L195-201.
- Public API on DkgEngine:
  pub fn nodes(&self) -> Result<Vec<DkgNodeInfo>>
  pub fn edges(&self) -> Result<Vec<(i64, i64, String)>>
  pub fn node_by_id(&self, id: i64) -> Result<Option<DkgNodeInfo>>
  pub fn outbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>>
  pub fn inbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>>
  pub fn failed_files(&self) -> Result<Vec<String>>
  pub fn dkg_readiness(&self) -> DkgReadiness
- DkgReadiness enum: Ready, Empty (0 nodes), Unavailable (not built).
DETERMINISTIC_CHECK: cargo check 0; trait/struct/enum greps return expected hits; ON DELETE CASCADE present; proptest in dev-deps; index_version.toml has 5 fields.
ACCEPTANCE: cargo check 0; Box<dyn LanguageGrammar> compiles (object-safety verified).

### 1A.2 VERIFY
PRE_GATE: 1A.1 checks pass.
TASK: cargo check workspace; cargo test --workspace no regressions.
DETERMINISTIC_CHECK: workspace check 0 errors; all existing tests pass (0 failed).
ACCEPTANCE: workspace compiles, pre-existing tests pass.

### 1A.3 TEST (TDD red — at least 20 tests, >=10 fail)
PRE_GATE: 1A.2 passes; schema defined; proptest available.
TASK (test categories):
Functional: parse Rust file extracting function_item/struct_item/impl_item/use_declaration; query by function name; query by struct name; signature populated correctly; incremental update (modified file -> updated lines + new file_hash + concept_stale=1); concept column NULL by default; file_hash matches blake2s round-trip.
Edge type: impl block -> "contains" edge; use declaration -> "imports" edge; impl Trait for Struct -> "implements" edge; outbound_edges direction (returns methods not reverse).
Downstream API: edges() returns all edges; outbound_edges only outbound; inbound_edges only inbound; nodes() iteration covers all DkgNodeInfo fields; failed_files() empty when all OK.
Parse failure (DKG only): empty file -> 0 nodes no error; syntax errors -> partial extraction, log, parse_status='failed'; binary file -> 0 nodes, parse_status='failed', appears in failed_files().
Readiness: dkg_readiness() Unavailable before build_graph; Ready after success >0 nodes; Empty after build on empty dir.
Project isolation: two DkgEngine instances with different db_path do not share state.
Property: random valid Rust snippets -> node count >=0, line numbers >0, names non-empty, file_hashes non-empty, signatures non-empty for function_item/struct_item.
DETERMINISTIC_CHECK: test files exist; cargo test -p knowledge --no-run = 0; >=10 FAILED in TDD red; #[test] count >=20.
ACCEPTANCE: >=20 tests, >=10 fail.

### 1A.4 IMPLEMENT
PRE_GATE: 1A.3 passes; tree-sitter grammars verified.
TASK: RustGrammar (impl LanguageGrammar — &self): walk AST, extract function_item/struct_item/impl_item/use_declaration/mod_item/trait_item/enum_item/type_item; reconstruct signatures.
HashMap<&str, Box<dyn LanguageGrammar>> dispatch; only ".rs" -> RustGrammar.
SQLite: CREATE TABLE IF NOT EXISTS for all + indexes; PRAGMA journal_mode=WAL; node insertion + concept_stale=1 + file_hash; typed directed edges insertion ("contains"/"imports"/"implements"); dkg_files tracking; query-by-name.
Directed query API: nodes(), edges(), outbound_edges (WHERE source_id=?), inbound_edges (WHERE target_id=?), node_by_id, failed_files().
Incremental: blake2s hash compare; if unchanged skip; if changed DELETE WHERE file_path=? then re-parse re-insert; CASCADE deletes edges; update dkg_files; concept_stale=1 for re-inserted.
build_graph: walk dir, parse each, populate SQLite, return DependencyGraph.
Parse failure: log::warn (or tracing::warn) with path+reason; insert dkg_files row parse_status='failed'; no dkg_nodes; no panic; available via failed_files() for Phase 1B.
dkg_readiness(): Unavailable before build, Ready after >0 nodes, Empty after 0.
Index versioning: write index_version.toml after build_graph (5 fields).
DkgEngine::new(db_path) takes explicit Path.
DETERMINISTIC_CHECK: tests pass (0 failed); WAL mode set; log::warn/tracing::warn present; DkgReadiness:: present >=3; edge type strings >=3; blake2 present; concept_stale set >=2; index_version 5 fields >=4; project-scoped db_path; signature reconstruction >=2 hits.
ACCEPTANCE: 0 failures; DKG builds from .elai_cc/; query returns correct file+line+signature; edge count >0; directed queries work; readiness Ready after build; non-.rs no panic appears in failed_files; isolation per path.

### 1A.5 MEASURE
PRE_GATE: 1A.4 passes; DKG indexes .elai_cc/.
TASK: Measure index build time; query latency p50/p99 (10 lookups); SQLite db size; count nodes, edges by type (imports/contains/implements), files parsed/skipped, signatures populated. Record .elai_cc/metrics/phase_1A.toml. Compare vs <15s target (knowledge.txt L20).
DETERMINISTIC_CHECK: phase_1A.toml exists with 12 keys: build_time_ms, query_p50_ms, query_p99_ms, db_size_bytes, total_nodes, total_edges, files_parsed, files_skipped, edges_imports, edges_contains, edges_implements, signatures_populated.
ACCEPTANCE: all 12 metrics populated; build time recorded; edge counts by type recorded; signature population recorded.

EXIT CRITERIA: DKG builds, queries return file+line+signature; all tests pass; directed edge API exposed; parse failure DKG-only items 1+5; failed_files() available; dkg_readiness correct; index_version.toml 5 fields; DkgNodeInfo with signature+SymbolKind; concept_stale tracked; project-scoped path; metrics baseline; no Phase 0 regression.

GROUNDING NOTES: <15s build UNGROUNDED, query latency UNGROUNDED, schema DESIGNED, edge types DERIVED.

AUDIT FINDINGS ADDRESSED v3: Path resolution dynamic; signature+SymbolKind contract; readiness split (DKG vs full); index_version.toml 5 fields; failed_files() bridge to Phase 1B; LanguageGrammar object safety; concept_stale column; project isolation test; proptest dep; outbound/inbound edges; DependencyGraph defined; file_hash round-trip test; dkg_files table.

================================================================================
## PHASE 1B: BM25 (tantivy full-text search)
================================================================================

Canonical path: .elai_cc/crates/knowledge/. Prereq: Phase 1A.

GOAL: Full-text search via tantivy BM25 index. Per-project isolation. Sub-10ms queries for indexes <100K docs. Zero orphan files: every text file appears in BM25 index, even if tree-sitter cannot parse it. Binary files (null bytes in first 8KB) sole exception.

RISKS: R1 tantivy index format version (INDEX_FORMAT_VERSION u32 in source, NOT spec label "tantivy_v21"); R2 index corruption on crash (need atomic commit); R3 Windows path handling; R4 memory on large repos; R5 binary file panic on non-UTF8 (skip+warn); R6 chunking strategy alignment; R7 path field MUST be STRING (untokenized) not TEXT.

### 1B.1 ADD TANTIVY DEPENDENCY
PRE_GATE: knowledge Cargo.toml exists; Phase 1A tests pass; no existing tantivy dep.
TASK: Verify tantivy on crates.io; add tantivy with pinned version; record INDEX_FORMAT_VERSION (u32) of pinned version in decision doc (spec label "tantivy_v21" is project identifier not tantivy API).
DETERMINISTIC_CHECK: cargo check 0 errors; tantivy in Cargo.toml.
ACCEPTANCE: cargo check 0 errors with tantivy.

### 1B.2 BM25 MODULE SKELETON WITH CORRECT API SHAPE
PRE_GATE: 1B.1 done; cargo check 0.
TASK: Create bm25.rs in src/.
Types:
  pub struct BM25Chunk { pub path: PathBuf, pub content: String, pub line_start: u32, pub line_end: u32, pub kind: ChunkKind }
  pub enum ChunkKind { Function, LineBlock, File }
NOTE: DKG exports DkgNode (AST metadata, no content). BM25Chunk includes content. Conversion is BM25's responsibility.
Bm25Index struct: new(project_root: &Path, project_id: &str) -> Result<Self>. project_id explicitly passed (knowledge.txt L274).
Index path: {project_root}/.elai_cc/projects/{project_id}/bm25/.
Tantivy schema:
  path: STRING | STORED  (untokenized — required for delete_term exact match)
  content: TEXT | STORED  (tokenized — BM25 search target)
  line_start: u64 STORED
  line_end: u64 STORED
  chunk_kind: STRING | STORED  (serialized from ChunkKind enum)
CRITICAL: path MUST be STRING. TEXT tokenizes (splits / . -). delete_term on TEXT silently fails.
Methods:
  index_file(path: &Path, chunks: Option<Vec<BM25Chunk>>) -> Result<usize>
  index_directory(root: &Path, dkg: Option<&DkgIndex>) -> Result<IndexStats>
    IndexStats { files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms }
  query(query_text: &str, limit: usize) -> Result<Vec<BM25Hit>>
    BM25Hit { path: String, line_start: u32, line_end: u32, score: f32, chunk_kind: ChunkKind, snippet: String }
    snippet: first 200 chars (simple truncation, not SnippetGenerator)
  delete_file(path: &Path) -> Result<usize>  (uses delete_term on STRING path field)
  update_file(path: &Path, chunks: Option<Vec<BM25Chunk>>) -> Result<usize>  (atomic: delete+index in single commit; <5s)
Writer serialization: Bm25Index holds single IndexWriter. index_directory holds writer for entire batch — concurrent update_file blocks (single-writer model).
Register in lib.rs.
Contract test stub: .elai_cc/tests/contracts/bm25_contract.rs with one test per public method.
DETERMINISTIC_CHECK: file exists; mod bm25 registered; cargo check 0; existing tests pass; BM25Hit has score+chunk_kind; IndexStats exists; BM25Chunk defined; STRING in schema; contract test file exists.
ACCEPTANCE: bm25.rs compiles; 5 methods present; index path follows spec; ChunkKind enum (not strings); STRING path field; contract test file exists.

### 1B.3 WRITE FAILING TESTS (TDD)
PRE_GATE: 1B.2 done; stubs compile.
TASK: Tests in bm25.rs or tests/bm25_tests.rs. ALL synthetic files in tempdir. No real codebase paths.
Core: (a) index Rust file with "fn hello_world", query, get correct file+lines; (b) tempdir with 3 files including "fn complete(input: &str)", query "fn complete"; (c) delete file, query unique symbol returns empty; (d) non-existent term returns empty Vec.
Zero orphan: (e) .xyz file (unsupported), index chunks=None, query keyword, get result (line-level fallback); (f) tempdir with 1 text + 1 binary, assert files_indexed=1, files_skipped_binary=1.
Incremental: (g) index "fn old_function", update_file with "fn new_function", query "new_function" succeeds, "old_function" empty.
Per-project isolation: (h) two Bm25Index with different project_id+root; A no results from B.
Chunking: (i) BM25Chunks provided (kind=Function) -> separate docs; chunks=None on 120-line file -> 50-line windows; assert chunk_kind correct.
Path field correctness: (j) index "src/deep/nested/module.rs", delete exactly that path, query unique content -> empty (guards against TEXT tokenization).
DETERMINISTIC_CHECK: >=10 BM25 tests; tests fail (FAILED in output).
ACCEPTANCE: >=10 tests fail (TDD red).

### 1B.4 IMPLEMENT BM25 INDEX
PRE_GATE: 1B.3 done.
TASK:
Core index: tantivy index with schema (path STRING, content TEXT, line_start/end u64, chunk_kind STRING). Storage at {project_root}/.elai_cc/projects/{project_id}/bm25/.
index_file two modes: with BM25Chunks each chunk one tantivy doc (chunk_kind=Function); without (line-level fallback) read content split into 50-line sliding windows with 10-line overlap (chunk_kind=LineBlock); small files <50 lines single doc (chunk_kind=File).
NOTE: 50-line windows match vector embedding chunk size (knowledge.txt L173); 10-line overlap UNGROUNDED for BM25 specifically (BM25 scores terms independently) — carried over for cross-layer alignment.
Binary detection: read first 8KB, if null byte (\0) classify binary, skip, log warning, count in IndexStats.
index_directory: walk recursively, skip hidden dirs (.*), skip .elai_cc/. For each file query DKG ranges, read content for BM25Chunks. If DKG no nodes, line-level fallback. Atomic commit (one writer.commit() after all). Single-writer constraint documented.
Query: BM25 via tantivy QueryParser on content; top N sorted by score DESC; BM25Hit with snippet (first 200 chars).
delete_file: delete_term on STRING path field; single commit.
update_file: atomic delete+add in single commit; <5s.
DETERMINISTIC_CHECK: BM25 tests pass; no warnings on bm25.rs; existing DKG tests still pass.
ACCEPTANCE: 10+ tests pass; binary file skipped (no panic); .xyz file IS indexed via fallback (zero orphan); delete_file exact-match (test j); update_file atomic; no DKG regression; per-project isolation.

### 1B.5 MEASURE AND VALIDATE INVARIANTS
PRE_GATE: 1B.4 done.
TASK:
PERFORMANCE: benchmark test (separate binary or #[bench]):
  (a) index .elai_cc/rust/ (69 .rs files, ~61K LOC)
  (b) measure index build time
  (c) 10 diverse queries: "fn main", "struct Config", "impl Display", "error handling", "async fn", "pub mod", "#[test]", "use std::", "trait", "Result<"
  (d) p50/p99 query latency
  (e) single-file update_file latency
INVARIANT VALIDATION:
  (f) count text files vs files in index — assert every non-binary indexed (zero orphan)
  (g) index path at .elai_cc/projects/{id}/bm25/
  (h) update_file <5s
RECORD .elai_cc/metrics/phase_1B.toml fields: index_build_time_ms, file_count, chunk_count, binary_skipped, query_p50_ms, query_p99_ms, update_file_p99_ms, orphan_files (=0), index_size_bytes, tantivy_version, index_format_version.
DETERMINISTIC_CHECK: file exists; query_p99_ms+orphan_files present; latency env-aware (CI=true: <50, else <10); update_file <5s.
ACCEPTANCE: query p99 <10ms dev / <50ms CI; update_file <5s; orphan_files=0; metrics file with all fields; index build <5s for 69 files.

EXIT CRITERIA: BM25 query <10ms p99 dev; 10+ tests pass; per-project isolation; zero orphan text files; binary skipped (no panic); update_file atomic <5s; chunking DKG-derived or 50-line fallback; index at spec path; STRING path; contract tests exist; metrics with tantivy_version + index_format_version.

DECISIONS (workflow/reasoning/phase-1B-decisions.md):
D1: 50-line sliding windows + 10-line overlap (UNGROUNDED for BM25, cross-layer consistency).
D2: Binary detection via null byte in first 8KB (not MIME) — git autocrlf heuristic.
D3: BM25Hit includes chunk_kind for downstream filtering.
D4: Index path .elai_cc/projects/{project_id}/bm25/ (knowledge.txt L195).
D5: update_file atomic (single-writer constraint).
D6: path STRING (not TEXT) — verified tantivy deleting_updating_documents.rs example.
D7: BM25Chunk distinct from DkgNode (DKG decoupled, BM25 owns content).
D8: project_id explicit input (not derived) — knowledge.txt L274.
D9: snippet simple 200-char truncation (not SnippetGenerator) — adequate for Phase 1B.

================================================================================
## PHASE 1C: VECTOR SEARCH (dual-model ONNX)
================================================================================

Prereq: Phase 1B.

GOAL: Semantic code search via dual embedding models (GraphCodeBERT 768d for code, Granite-embedding-small 384d for text) using ONNX Runtime. Vector cache in SQLite keyed by content hash. Per-model circuit breakers with degradation cascade. Startup readiness gate with canary query.

RISKS: R22 ONNX GPU contention with main model (embedding GPU1 only when executor/auditor on GPU0; fallback all CPU); R26 single ONNX failure cascading (per-model circuit breaker, knowledge.txt §6, HYBRID->SPARSE_ONLY->BM25_ONLY); R27 ONNX startup delay blocking FSM (canary embed("fn main()") within 5s); R29 cross-encoder concurrency (tokio::Semaphore(1) + queue depth 4).

SCOPE DECISIONS:
- SPLADE: DEFERRED to Phase 1G/1H. Phase 1C implements core dual-vector pipeline. RRF fusion in Step 4 merges BM25 + code_vec + text_vec (3 rankers). SPLADE if added becomes 4th. Circuit breakers Phase 1C: GraphCodeBERT + Granite only.

GROUNDING (verified):
[VERIFIED-SPEC] CODE: GraphCodeBERT 768d (knowledge.txt §2 L3)
[VERIFIED-SPEC] TEXT: Granite-embedding-small 384d (knowledge.txt §2 L3)
[VERIFIED-SPEC] embed() -> Vec<f32> trait (providers.txt §2)
[VERIFIED-SPEC] vector_cache schema (content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INTEGER)
[VERIFIED-SPEC] ort listed as ADD dep (architecture-overview §10)
[VERIFIED-SPEC] Circuit breaker: 3 failures/60s window, half-open after 30s
[VERIFIED-SPEC] Canary embed("fn main()") within 5s
[VERIFIED-SPEC] Cross-encoder Semaphore(1) max queue depth 4
[VERIFIED-SPEC] Cache hit rate >90% on incremental re-index
[VERIFIED-SPEC] Cache invalidation: DELETE on SSB action_type='file_write'
[VERIFIED-SPEC] GraphCodeBERT fail -> TEXT-only; Granite fail -> CODE-only; both -> SPARSE_ONLY
[VERIFIED-SPEC] ONNX+LLM GPU multiplexing only on dual-GPU; single GPU = CPU-only ONNX

CRATES verified 2026-04-06: ort (pykeio/ort) 166 reverse deps; usearch v2.24.0 HNSW SIMD; fastembed v5.12.0 (wraps ort — DO NOT add); blake2 (NOT blake2s, use Blake2s256 hasher).

GROUNDED PAPERS: TurboQuant (arXiv:2504.19874, ICLR 2026) — interface map workflow/papers/turboquant-interface.md. Distortion (unit-norm): b=2 MSE=0.117, b=3 MSE=0.030, b=4 MSE=0.009. Inner product distortion: b=2 0.56/d, b=3 0.18/d, b=4 0.047/d. 768d at b=3: ~10.7x compression vs f32, InProd error ~0.00023. 384d at b=3: ~10.7x, error ~0.00047. Indexing ~0.001s vs PQ ~494s at 100k vectors. Requirements: unit-norm input (store norms externally), QR rotation (NOT Hadamard), Lloyd-Max codebook precomputed per (b,d). O(d^2) space. Status: GROUNDED but DEFERRED to Phase 1G or dedicated phase. Phase 1C stores raw f32.

UNGROUNDED: GraphCodeBERT/Granite ONNX tensor format (must verify); embedding throughput; dual-vector vs BM25 accuracy.

### STEP 1: DEPENDENCY VERIFICATION AND CARGO SETUP
PRE_GATE: Phase 1B exit; rusqlite present; no prior ort entry.
ACTIONS: cargo add ort --package knowledge (pinned, GPU feature flag if dual-GPU); cargo add usearch; cargo add blake2; do NOT add fastembed (wraps ort); record HNSW decision (usearch) in workflow/reasoning/phase-1C-decisions.md.
DETERMINISTIC_CHECK: cargo check 0; ort version-pinned in Cargo.toml; phase-1C-decisions.md contains HNSW line.
ACCEPTANCE: cargo check exits 0 with ort entry; HNSW decision recorded; FAIL if ort version not pinned.

### STEP 2: SKELETON
PRE_GATE: Step 1 done; HNSW decision recorded.
ACTIONS:
- knowledge/src/vector/mod.rs: VectorModule trait; DualVector struct (code_vec: Vec<f32> 768d, text_vec: Vec<f32> 384d). Stub returns Err(NotReady), unimplemented!() only behind feature flag.
- knowledge/src/vector/cache.rs: SQLite vector cache CRUD (insert, lookup by blake2s, invalidate by file_path). Schema: content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INTEGER.
- knowledge/src/vector/ort_session.rs: OrtSession wrapping Arc<Mutex<ort::Session>>. ort::Session::run() takes &mut self — Mutex required. Single Session per model. Loads from canonical .elai_cc/models/onnx/ (configurable .toml [knowledge.onnx_model_path]). Missing model -> Err(ModelNotFound) -> DEGRADED (BM25_ONLY), not crash. embed(text: &str) -> Result<Vec<f32>, OrtError>. Two instances: code_model (GraphCodeBERT 768d), text_model (Granite 384d). max_seq_len=512 for GraphCodeBERT (huggingface.co/microsoft/graphcodebert-base). Tokenizer truncates BEFORE tensor construction; log WARNING on truncation. Store max_seq_len as field.
- knowledge/src/vector/circuit_breaker.rs: CircuitBreaker. States: CLOSED, OPEN, HALF_OPEN. Threshold 3/60s. Half-open after 30s.
- vector_cache migration extends Phase 1B schema runner (M3). SSB file_write handler: DELETE FROM vector_cache WHERE file_path=?.
- knowledge/src/vector/hnsw_index.rs: VectorIndex trait + UsearchIndex impl wrapping usearch::Index. Constructor: dimension + MetricKind::Cos. add(key, vec), search(query, k) -> Vec<(Key, f32)>, filtered_search(query, k, predicate) for project isolation. save(path)/load(path) for persistence (NOT view() — read-only). Two: code_index 768d, text_index 384d. Stored alongside SQLite DB.
- INPUT CHUNKING DEPENDENCY noted: chunking strategy (AST via tree-sitter vs line) is prereq. Reference code-sage (github.com/faxioman/code-sage).
DETERMINISTIC_CHECK: cargo check 0; DualVector exists; CircuitBreaker exists; vector_cache table; Arc<Mutex<.*Session>> present; max_seq_len/512 present; VectorIndex/UsearchIndex present; filtered_search present; save/load present; no TODO/placeholder.
ACCEPTANCE: cargo check 0; SQLite migration runs (integration test); DualVector::new validates dims; OrtSession stores max_seq_len + Arc<Session>; VectorIndex with save/load/filtered_search compile; FAIL on TODO; FAIL if OrtSession::new bad path panics.

### STEP 3: TEST (write failing tests, no real ONNX, no network)
PRE_GATE: Step 2 done; tests/contracts/ exists; KnowledgeEngine contract test present.
ACTIONS: Unit tests with mock OrtSession:
- test_dual_vector_dimensions (768/384)
- test_vector_cache_insert_lookup (round-trip)
- test_vector_cache_invalidation (file_write event removes)
- test_circuit_breaker_opens_after_3_failures
- test_circuit_breaker_half_open_probe (advance 30s -> HALF_OPEN)
- test_graceful_degradation_code_model_fail (text_vec only)
- test_graceful_degradation_both_models_fail (SPARSE_ONLY)
- test_query_validation_empty -> Err(ValidationError::EmptyQuery)
- test_query_validation_oversized (10001 chars -> Err::TooLong)
- test_query_validation_control_chars (null/control -> Err::InvalidChars)
- test_startup_canary_success (<5s -> Ready)
- test_startup_canary_timeout (>5s -> Degraded(BM25_ONLY))
- test_onnx_canonical_path (.elai_cc/models/onnx/{name}.onnx)
- test_missing_model_degraded (nonexistent .onnx -> Err(ModelNotFound) -> DEGRADED)
- test_semaphore_queue_overflow (>4 concurrent -> 5th unranked)
- test_cache_hit_skips_embed
- test_truncation_warning_on_long_input (>512 tokens, tracing-test asserts WARNING; output dim correct)
- test_hnsw_index_persistence_roundtrip (10 vectors save/load search same)
- test_hnsw_filtered_search_project_isolation
- test_concurrent_embed_mutex_session (4 tasks, no deadlock)
- test_rrf_fusion_formula: BM25 ranks [1,3,5], code_vec [2,1,4], text_vec [3,2,1] for 3 docs; sum 1/(60+rank) across 3 rankers (Cormack et al. SIGIR'09 k=60)
Contract tests in tests/contracts/knowledge_vector_contract.rs:
- test_query_returns_dual_vector_results (code_score+text_score)
- test_query_project_isolation (knowledge.txt §13 requires physical file isolation — separate knowledge.db per project, not query filtering)
DETERMINISTIC_CHECK: >=20 #[test] in vector/; named tests present; cargo test --no-run 0.
ACCEPTANCE: cargo test --no-run 0; >=22 unit tests (13+7+2); contract tests exist; FAIL if std::time in circuit_breaker (entropy quarantine); FAIL on real ONNX dep in unit tests.

### STEP 3.5: TENSOR FORMAT VERIFICATION (gate for Step 4)
PRE_GATE: Step 3 done; ONNX model files OR model cards accessible.
ACTIONS: Verify GraphCodeBERT ONNX tensor format (microsoft/graphcodebert-base): input_ids (batch, seq_len), attention_mask (batch, seq_len), max seq_len=512. ort metadata inspection if file available. Verify Granite tensor format. Record in phase-1C-decisions.md: tensor_format_verified: true with shapes. Add tokenizer crate dep if missing (tokenizers or huggingface-hub for BPE).
DETERMINISTIC_CHECK: tensor_format_verified: true present; input_ids/attention_mask present.
ACCEPTANCE: decisions file with tensor_format_verified: true + shapes.

### STEP 4: IMPLEMENT (ort ONNX loading, dual-vector, HNSW, cache)
PRE_GATE: Step 3 done; Step 3.5 done; TurboQuant paper INGESTED but implementation deferred.
ACTIONS:
- OrtSession::embed(): build ort tensor from tokenized input, run session, extract output, return Vec<f32>. Tokenizer truncates at max_seq_len=512 emit tracing::warn! on truncation.
- VectorCache (cache.rs): insert_dual, lookup_by_hash, invalidate_by_file. Single-writer SSB channel for all SQLite writes.
- CircuitBreaker: record_failure/success, is_open, probe — using ClockProvider (NOT std::time::Instant).
- HNSW via usearch v2.24.0. MetricKind::Cos, dim 768 code / 384 text. Wrap in VectorIndex trait. save() after bulk add; load() at startup. If load fails: rebuild from vector_cache (cache is source of truth). filtered_search for project isolation.
- KnowledgeEngine::readiness_check(): load both models, canary embed("fn main()"), assert <5s. Return Ready or Degraded(mode). WARMUP: canary serves as ONNX warm-up (first inference known-slow due to graph optimization, microsoft/onnxruntime#19177).
- Query pipeline RRF: BM25 + code_vector + text_vector. rrf_score(d) = sum 1/(k+rank_r(d)) where k=60 (Cormack/Clarke/Buettcher SIGIR'09). Return Vec<CodeChunk> with code_score, text_score, bm25_score, rrf_score.
- SSB file_write handler: DELETE FROM vector_cache WHERE file_path=?. Registered at startup.
- Cross-encoder tokio::Semaphore(1), queue depth 4, overflow returns unranked.
- SSB action_type='knowledge_degradation' on every SearchMode change (knowledge.txt §5).
- QueryTrace at DEBUG via tracing: query_id, stages Vec<StageTrace>, total_latency_ms, mode, result_count.
DETERMINISTIC_CHECK: cargo check 0; no std::time in vector/; no unwrap(); no TODO; QueryTrace/StageTrace present; file_write/invalidate_by_file present; rrf_score/1.0/60 present; max_seq_len/truncat present; .save/.load present; filtered_search present.
ACCEPTANCE: cargo check 0; no std::time/unwrap; SSB handler registered; QueryTrace emitted; RRF k=60; truncation warning; persistence; filtered_search; canary serves warmup; FAIL on tensor format mismatch; FAIL on TurboQuant code (deferred).

### STEP 5: MEASURE (operational, requires real ONNX models, does not block phase exit)
PRE_GATE: Step 4 done; audit passed; ONNX models at .elai_cc/models/graphcodebert/ and .elai_cc/models/granite-embedding-small/.
ACTIONS:
- Embed 100 real chunks from .elai_cc/src/ with both models. Record throughput, p50/p99 latency.
- 5 representative semantic queries — relevant in top-5? CODE/TEXT/dual?
- Compare dual-vector vs BM25 alone (manual eval).
- Cache hit rate (run twice, expect 100% on second). Assert >=90% (knowledge.txt §13).
- Log to workflow/reasoning/phase-1C-measurements.md.
- If embedding throughput <10 chunks/sec on CPU: WARN.
- If cache hit rate <90%: BLOCK.
DETERMINISTIC_CHECK: measurements file with throughput_code_model, throughput_text_model, cache_hit_rate, dual_vector_vs_bm25; cache_hit_rate >=90; no TODO/N/A.
ACCEPTANCE: file with 4 entries; cache hit rate >=90%; FAIL if missing or N/A; FAIL if cache_hit_rate <90.

EXIT CRITERIA:
1. cargo check 0 with ort
2. >=20 unit tests pass
3. Contract test for dual-vector format passes
4. Canary <5s (or DEGRADED correctly), serves warmup
5. Vector cache insert/lookup/invalidation tests pass
6. Circuit breaker opens after 3/60s
7. std::time = 0 in vector/
8. Measurements file with cache_hit_rate >=90%
9. HNSW via usearch v2.24.0; persistence via save/load
10. TurboQuant NOT implemented (deferred Phase 1G)
11. RRF k=60 implemented + tested
12. Tokenizer truncation at 512 with warning
13. usearch filtered_search for project isolation
14. Arc<Mutex<Session>> verified (concurrent embed no deadlock)

MEASUREMENTS: throughput chunks/sec per model; p50/p99 ms; RRF hybrid recall vs BM25 vs vector-only; first-query latency post-warmup vs not (cold-start); cache hit rate; canary <5s; cross-encoder p99 rerank for 50 candidates <50ms CPU.

================================================================================
## PHASE 1D: PAGERANK REPO MAP
================================================================================

Prereq: Phase 1C (cargo check 0 with ort).

GOAL: Rank code symbols by importance via PageRank on DKG graph. Produce token-budget-fitted repo maps for context loading.

SOURCE REFERENCE: github.com/Aider-AI/aider/blob/main/aider/repomap.py (design 31.7 origin). Diverges from aider 3 ways (symbol-level vs file-level, density guard, insertion-order fallback); adopts 5 improvements.

RISKS: R20 PageRank density degradation (guard at 0.3 threshold). R-NEW Dangling nodes (zero rank in naive iteration) — pass dangling=personalization. R-NEW Orphan symbols (defined-not-referenced) — add self-edges weight=0.1.

### 1D.1 SKELETON
PRE_GATE: Phase 1C exit; cargo check 0; DkgEngine has pub struct with dkg/node/symbol; providers has token_count_estimate; DkgEngine has edge/neighbor/adjacen API (else 1D.1 must add edge-iteration API querying dkg_edges).
TASK:
- Verify proptest exists; add to dev-deps.
- DkgSnapshot { nodes: Vec<DkgNodeInfo>, edges: Vec<(usize, usize, f64)> }.
  DkgNodeInfo { node_id: i64, symbol_name: String, file_path: PathBuf, signature: String, kind: SymbolKind, ref_count: usize }
  pub fn DkgSnapshot::from_engine(engine: &DkgEngine) -> Result<DkgSnapshot>
  Self-edges: from_engine adds self-edge (weight=0.1) for any node with zero incoming edges (aider/repomap.py).
- pagerank module:
  PageRankConfig { damping_factor: f64=0.85, max_iterations: usize=100, tolerance: f64=1e-6, density_threshold: f64=0.3, max_map_tokens: usize=2048 }
  RepoMap { ranked_symbols: Vec<RankedSymbol>, token_count: usize, density: f64, used_pagerank: bool, fit_iterations: usize }
  RankedSymbol { node_id: i64, symbol_name: String, file_path: PathBuf, rank: f64, signature: String }
- pub fn compute_repo_map(snapshot: &DkgSnapshot, task: &str, active_files: &[PathBuf], config: &PageRankConfig, token_estimator: &dyn Fn(&str)->usize) -> RepoMap
- All bodies: todo!()
DETERMINISTIC_CHECK: cargo check 0; compute_repo_map=1; PageRankConfig=1; RepoMap=1; DkgSnapshot=1; proptest in Cargo.toml.
ACCEPTANCE: cargo check 0; proptest in dev-deps.

### 1D.2 VERIFY
PRE_GATE: 1D.1 done.
TASK: cargo check workspace; cargo test --package knowledge --lib (no regressions).
DETERMINISTIC_CHECK: workspace check 0; existing tests pass.
ACCEPTANCE: workspace 0 errors; zero regressions.

### 1D.3 TEST (TDD red, >=21 tests)
PRE_GATE: 1D.2 done; proptest compiles.
TASK Unit tests:
T1: 5-node linear graph, task mentions node 3 -> node 3 highest
T2: Edge-weight boost — edges touching active_files weight *= 50, boosted ranks > unboosted
T3: Personalization — task-mentioned 10x, rank higher
T4: Density guard — density >0.3 -> used_pagerank=false, insertion-order (NOT BM25-ranked, no BM25 dep in signature)
T5: Density <0.3 -> used_pagerank=true
T6: Token budget — token_count <= max_map_tokens * 1.15 (+15%)
T7: Token undershoot — 200 nodes, mock estimator 10/symbol, max_map_tokens=1000 -> token_count >= 850 (85%)
T8: Special files always included — Cargo.toml, README
T9: Personalization sums to 1.0 (f64 epsilon)
T10: Binary search convergence — 1000 nodes, fit_iterations <= 10
T11: Empty graph (0 nodes) -> empty RepoMap, token_count=0, fit_iterations=0
T12: Single-node graph
T13: Zero budget (max_map_tokens=0) -> empty, no panic
T14: All nodes mentioned -> uniform personalization, valid ranking (no NaN/Inf)
T15: Dangling node redistribution — A->B->C, C is leaf. Without dangling: C accumulates never distributes. With dangling=personalization: C.rank > 0 AND sum ranks ~= 1.0.
T16: Orphan self-edges — node 4 with no edges. With self-edge weight=0.1: rank > 0. Without: rank = 0.
T17: Sqrt reference scaling — A 100 refs, B 4 refs. With sqrt: weight ratio sqrt(100)/sqrt(4)=5x. A.rank/B.rank < 8 (not 25x linear).
T18: Line truncation — 500-char signature. Estimator len/4. Without truncation 125 tokens; with 100-char: 25 tokens. Verify truncated estimate.
Property tests (proptest):
P1: random graph 1-500 nodes -> all ranks finite >= 0.0
P2: used_pagerank=true -> sum ranks ~= 1.0 (within 1e-4)
P3: random max_map_tokens 100-10000 -> token_count <= max_map_tokens * 1.15
Adversarial:
A1: Near-complete graph (density ~0.95) -> guard activates used_pagerank=false
A2: Star graph (1 hub, 100 spokes) -> hub highest
A3: Disconnected components — nodes without active files get base rank only
Helper: fn make_snapshot(nodes: usize, edges: Vec<(usize,usize)>) -> DkgSnapshot
DETERMINISTIC_CHECK: test files exist; #[test] count >= 21 (T1-T18 + A1-A3); proptest! >=1; tests fail (todo!).
ACCEPTANCE: >=21 tests; compile (cargo test --no-run 0); fail at runtime.

### 1D.4 IMPLEMENT
PRE_GATE: 1D.3 done.
TASK:
- DkgSnapshot::from_engine: query dkg_nodes + dkg_edges, build vectors.
- compute_repo_map:
  1. Build adjacency from edges. Self-edges (0.1) already from from_engine. Apply sqrt(ref_count) edge weight scaling (aider).
  2. Edge-weight boost (*= 50) for edges touching active_files BEFORE PageRank.
  3. Personalization vector: 10x for task-mentioned nodes, normalize sum=1.0. All mentioned -> uniform 1/N.
  4. Density = edges.len() / (nodes.len() * (nodes.len()-1)). If >0.3 OR nodes.len()<=1: skip PageRank, insertion order, used_pagerank=false. (Density guard is OUR addition; aider has no guard, uses try/except.)
  5. Power iteration: damping/max_iter/tolerance from config. Dangling nodes redistribute per personalization (NetworkX behavior: "dangling nodes are given outedges according to the personalization vector"). Without: leaf nodes accumulate but never distribute.
  6. Special files (13 Rust items): Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile, docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml. Config-driven extensibility deferred.
  7. Binary search to fit ranked + special to max_map_tokens. If max_map_tokens=0: empty, fit_iterations=0. Tolerance ±15%. Max 10 iter. Initial guess: min(max_map_tokens/25, num_ranked_nodes). Reduces iterations.
  7b. Line truncation: each signature truncated to 100 chars before estimation (aider — prevents minified code blowing budget).
  8. Return RepoMap.
- Helpers: build_adjacency_matrix (sqrt scaling, includes self-edges); build_personalization (sums to 1.0); build_dangling_vector; power_iteration (NaN/Inf clamp + log warning); truncate_signature (default 100); is_special_file; binary_search_fit (returns symbols + iter count).
DETERMINISTIC_CHECK: cargo test pagerank: 0 failures; T1-T18 pass; P1-P3 (PROPTEST_CASES=100); A1-A3 pass.
ACCEPTANCE: 0 failures; clippy 0 errors; no todo!/unimplemented!/panic!("not implemented").

### 1D.5 MEASURE
PRE_GATE: 1D.4 done.
TASK: .elai_cc/metrics/phase_1D.toml: test_count, compile_time_seconds (cargo build --timings), pagerank_loc (wc -l, tokei optional), ranking_quality (star hub #1? task-mentioned top-3? boolean), token_budget_accuracy (max % deviation), density_guard_correct (boolean), binary_search_max_iterations.
DETERMINISTIC_CHECK: file exists; >=5 lines.
ACCEPTANCE: file with all 7 metrics.

AIDER ADOPTED (5):
1. Dangling node redistribution via personalization (T15)
2. Self-edges weight=0.1 for orphans (T16)
3. sqrt(num_refs) edge scaling (T17)
4. Binary search initial guess max_map_tokens/25 (T10)
5. Line truncation 100 chars (T18)

DIVERGENCES (3): symbol-level (not file-level); density guard 0.3 (aider has none); insertion-order fallback (aider returns empty).

NOT ADOPTED: identifier specificity boost; private symbol penalty (Rust uses pub/pub(crate)); generic name penalty (sqrt already dampens).

GROUNDING UNGROUNDED: damping=0.85/max_iter=100/tol=1e-6 (Brin/Page 1998 defaults, not in our specs — overridable in config); special files list 13 items (knowledge.txt L75 says ~100 — minimal Rust list, extensibility deferred).

SPEC CONFLICTS: phase originally said "Section 2 Layer 4" (CatRAG) — corrected to Layer 3.5; R20 ID collision (risk-register.txt:L88 PageRank density vs phase-0.txt:L22 entropy quarantine) — cross-file fix needed.

EXIT CRITERIA: 0 failures, >=21 tests; density guard activates; token budget within ±15%; special files always present; no todo!/unimplemented!; clippy 0 errors; phase_1D.toml exists; T13 zero-budget no panic; P1-P3 pass with PROPTEST_CASES=100; T15 dangling rank>0; T16 orphan self-edge rank>0; T17 sqrt prevents domination; T18 truncation limits tokens.


================================================================================
## PHASE 1E: CatRAG + A-RAG (query pipeline assembly)
================================================================================

Prereqs: Phase 1D (PageRank), Phase 1B (BM25), Phase 1C (vectors).

GOAL: Assemble retrieval pipeline: query -> BM25 -> vector -> PageRank -> CatRAG -> A-RAG. CatRAG provides query-aware graph traversal over DKG. A-RAG provides hierarchical retrieval with snippet-based progressive disclosure. Target: 55-87% token savings, sub-second latency.

GROUNDING SOURCES:
- Paper CatRAG (arXiv 2602.01965) — workflow/papers/catrag-interface.md. Symbolic Anchoring, Query-Aware Dynamic Edge Weighting, Key-Fact Enhancement. Applied as query-time overlays on PPR (Phase 1D).
- Paper A-RAG (arXiv 2602.03442) — workflow/papers/arag-interface.md. Three-tool hierarchy, ReAct loop, snippet-based progressive disclosure, context tracker.
- knowledge.txt Layers 4-5, §3 Query Pipeline.

ADAPTATION (paper -> ELAI):
- Entity nodes V_E -> DkgNode (functions, structs, traits, modules)
- Passage nodes V_P -> source file chunks (BM25)
- Relation edges E_rel -> DKG edges
- Synonym edges E_syn -> DKG nodes with high embedding similarity (>0.8 cosine)
- Context edges E_ctx -> DKG node-to-file mapping
- OpenIE triple extraction NOT NEEDED (tree-sitter extracts structured relations)
- NER -> symbol name extraction from query (regex + fuzzy match)
- A-RAG: keyword_search -> BM25 + snippets; semantic_search -> dual-vector + snippets; chunk_read -> file content/lines from disk; Agent loop -> ReAct via local LLM (ollama) or external; no parallel tool calling (paper design)

Pipeline order (knowledge.txt L104-105): query -> BM25+SPLADE(RRF) -> dual-vector rerank -> PageRank importance filter -> CatRAG expand -> A-RAG filter -> cross-encoder rerank -> token-budget fit -> context. Cross-encoder is Phase 1G. Token-budget fit is Phase 3E. This phase: CatRAG + A-RAG + pipeline wiring; stubs for 1G + 3E.

RISKS: R30 CatRAG LLM at query time (limit N_seed=5, K_edge=15, cache); R31 sparse code graphs (synonym edges if similarity >0.8; if density <0.01 skip CatRAG); R32 A-RAG no convergence (max_loops=15, max_token_budget, ContextTracker dedupes); R33 snippet loses context (+/- 2 lines); R34 latency budget <2s; per-layer measure.

### 1E.1 SKELETON — CatRAG MODULE + A-RAG ORCHESTRATOR STUBS
PRE_GATE: Phase 1D done; DkgSnapshot/Bm25Index/pagerank present.
TASK: Create catrag.rs in src/.
CatRagConfig { damping: f64=0.5, epsilon: f64=0.2 (symbolic anchor), beta: f64=2.5 (key-fact boost), n_seed: usize=5, k_edge: usize=15, min_graph_density: f64=0.01 }
CatRagOverlay { reset_distribution: Vec<(usize, f64)>, edge_weight_deltas: Vec<(usize, usize, f64)> }
CatRag::new(config) -> Self
CatRag::compute_overlay(&self, query: &str, snapshot: &DkgSnapshot, pagerank_scores: &[(usize, f64)]) -> Result<CatRagOverlay>
Apply symbolic anchoring + dynamic edge weighting + key-fact enhancement. Returns overlay modifying PPR inputs without mutating base graph.
Create arag.rs:
AragConfig { max_loops: usize=15, max_token_budget: usize=128000, top_k: usize=5 }
RetrievalTool { KeywordSearch{keywords, top_k}, SemanticSearch{query, top_k}, ChunkRead{chunk_ids} }
RetrievalResult { chunk_id: String, snippet: String (matched sentences only), score: f64, tool_used: String ("keyword"/"semantic"/"chunk_read") }
AragOrchestrator (holds BM25, vector index, file reader)
ContextTracker { read_chunks: HashSet<String> }  (per-query, not shared)
AragOrchestrator::new(config) -> Self
AragOrchestrator::execute_tool(&self, tool: &RetrievalTool, tracker: &mut ContextTracker) -> Result<Vec<RetrievalResult>>
AragOrchestrator::query_pipeline(&self, query: &str, snapshot: &DkgSnapshot) -> Result<Vec<RetrievalResult>>
Create pipeline.rs:
QueryPipeline { bm25, catrag, arag } (// vector_index Phase 1C, // pagerank Phase 1D commented)
PipelineStage { stage: String (matches knowledge.txt L305), results: Vec<RetrievalResult>, latency_ms: u64 }
QueryPipeline::query(&self, q: &str) -> Result<Vec<PipelineStage>>
lib.rs: pub mod catrag; pub mod arag; pub mod pipeline;
All bodies todo!() stubs.
DETERMINISTIC_CHECK: cargo check 0; mod catrag/arag/pipeline registered.
ACCEPTANCE: cargo check 0.

### 1E.2 VERIFY — COMPILATION CHECK
PRE_GATE: 1E.1 done.
TASK: cargo check -p knowledge; cargo check --workspace; no new warnings.
DETERMINISTIC_CHECK: both cargo check 0.
ACCEPTANCE: workspace clean.

### 1E.3 TEST — WRITE FAILING TESTS (TDD)
PRE_GATE: 1E.2 done.
TASK Tests fail at todo!() stubs.
CatRAG (catrag.rs or tests/catrag_tests.rs):
a. Symbolic anchoring: query with known symbols -> reset_distribution contains matching DKG nodes weight = epsilon * |files_containing|^(-1)
b. Dynamic edge weighting: snapshot with known edges -> overlay has modified weights for seed nodes; non-seeds unchanged
c. Key-fact enhancement: verified code references (T_seed) -> context edges to files containing them get weight *= (1+beta)
d. Density guard: density < min_graph_density -> compute_overlay returns empty overlay (passthrough to A-RAG)
e. Score-to-weight phi() tiers: 0-3->0.0, 4->0.2, 5->0.25, 6->0.3, 7->2.0, 8->2.5, 9->3.0, 10->5.0
A-RAG (arag.rs or tests/arag_tests.rs):
f. Keyword search: BM25 with known content -> snippets (matched sentences only); score = sum(count * |keyword|)
g. Semantic search: vector index -> top-k chunks with snippets; oversample 3x at sentence, aggregate to chunks by max score
h. Chunk read: first read full content; second read same chunk -> "(previously read)" via ContextTracker
i. Context tracker: read_chunks grows monotonically, is_read returns correct
j. Token budget enforcement: accumulated > max_token_budget -> stops, returns partial
Pipeline (pipeline.rs or tests/pipeline_tests.rs):
k. Full pipeline smoke: query_pipeline returns stages in order ["bm25", "vector", "pagerank", "catrag", "arag"]
l. Pipeline latency: each PipelineStage latency_ms > 0
m. Empty query: returns empty without panic
Contract (tests/contracts/):
n. CatRagOverlay round-trip: applied + reverted (base unchanged)
o. RetrievalResult populated (no empty chunk_id/snippet)
DETERMINISTIC_CHECK: new tests fail (todo!), existing tests pass.
ACCEPTANCE: all new tests fail with todo!; existing tests pass.

### 1E.4 IMPLEMENT CatRAG — QUERY-AWARE GRAPH TRAVERSAL
PRE_GATE: 1E.3 done; CatRAG paper ingested (workflow/papers/catrag-interface.md).
TASK Implement compute_overlay (catrag-interface.md §8):
a. Symbolic Anchoring: extract symbol names (split whitespace/camelCase/snake_case); fuzzy-match against DKG names (case-insensitive substring); reset_weight = epsilon * |files_containing(e_i)|^(-1); combine with PageRank seeds (higher base); normalize sum=1.0. NO LLM (deterministic).
b. Query-Aware Dynamic Edge Weighting (simplified for code):
- Top-N_seed nodes by reset prob
- For each seed get outgoing DKG edges
- |neighbors| > K_edge: coarse prune by embedding similarity (query vs edge label/target name); keep top-K_edge
- Fine-grained scoring: heuristic (NOT LLM in hot path):
  (1) edge type weight: calls=8, implements=7, references=5, imports=3
  (2) target node type relevance to query
  (3) file proximity to query-mentioned files
  Map to phi() tiers (0-10)
- LLM scoring (paper) optional via circuit breaker
- Apply phi() (paper Table 7); store as edge_weight_deltas (sparse)
c. Key-Fact Passage Weight Enhancement: for each DKG edge symbol->file: if symbol in query context (T_seed = mentioned symbols + direct callees) boost context edge *= (1+beta). Pure algorithmic.
d. Density Guard: density = |edges|/(|nodes|*(|nodes|-1)). If <min_graph_density (0.01): empty overlay; log "CatRAG skipped: graph density {d} below threshold {t}".
e. Apply overlay to PPR: modify reset vector + transition matrix; run PPR with modified inputs (reuse Phase 1D); return ranked results.
DETERMINISTIC_CHECK: CatRAG tests (a-e) pass; clippy 0 (-D warnings).
ACCEPTANCE: tests pass; overlay sparse + query-local; base graph never mutated.

### 1E.5 IMPLEMENT A-RAG — HIERARCHICAL RETRIEVAL ORCHESTRATOR
PRE_GATE: 1E.4 done.
TASK Implement (arag-interface.md §4):
a. Keyword Search (wraps BM25): keywords Vec<String>, top_k. Bm25Index::query each, merge. Score = sum(count(keyword, chunk_text) * keyword.len()). Snippets: sentences with at least one keyword (+/- 2 lines for code). Top_k results with snippets (NOT full chunks).
b. Semantic Search (wraps vector index): query, top_k. Vector index for query embedding. Oversample 3x at sentence/chunk, aggregate by max score per file. Snippets with surrounding context. Top_k with snippets.
c. Chunk Read: chunk_ids. If ContextTracker.is_read(id) -> "(previously read)". Else read full file (or line range), mark read. Full content for new, placeholder for already-read.
d. Context Tracker: HashSet<String>. Per-query (not shared). is_read, mark_read, token_count.
e. Execute Tool dispatch: match enum variant, dispatch a/b/c. Track tokens. Enforce max_token_budget: exceeded -> return partial.
f. Query Pipeline orchestration (simplified, no LLM agent loop): for Phase 1E: deterministic pipeline keyword_search(query_keywords) -> semantic_search(query) -> chunk_read(top hits). Full ReAct (LLM decides) is Phase 2+ (requires reasoning crate). Phase 1E implements TOOLS + PIPELINE WIRING, not agent decision-making.
DETERMINISTIC_CHECK: A-RAG tests (f-j) pass; clippy 0.
ACCEPTANCE: tests pass; snippet works for code; budget enforcement triggers.

### 1E.6 IMPLEMENT PIPELINE
PRE_GATE: 1E.5 done; CatRAG operational.
TASK Wire stages in QueryPipeline::query():
a. Stage 1 BM25: extract keywords (split, dedup, filter stopwords); Bm25Index::query(query, top_k=20); record PipelineStage{stage:"bm25"}.
b. Stage 2 Vector: vector index with query embedding (Phase 1C); RRF merge (k=60); record. If vector unavailable (circuit breaker): skip use BM25 only.
c. Stage 3 PageRank: build DkgSnapshot, run PageRank; filter/rerank by PageRank (boost files containing high-PageRank symbols); record.
d. Stage 4 CatRAG: compute CatRagOverlay; apply to PageRank, re-run PPR; expand by reachable via high-weight overlay edges; record. Density guard -> passthrough.
e. Stage 5 A-RAG: AragOrchestrator filter/refine; snippets not full files; ContextTracker dedup; record.
f. Stage 6-7 Stubs: PipelineStage{stage:"rerank"} (Phase 1G), PipelineStage{stage:"token_fit"} (Phase 3E). Passthrough.
g. Return Vec<PipelineStage> with full trace.
DETERMINISTIC_CHECK: pipeline tests (k-m) pass; existing tests pass; clippy 0.
ACCEPTANCE: stages in correct order; latency_ms+results populated; graceful degradation.

### 1E.7 MEASURE — END-TO-END LATENCY AND TOKEN SAVINGS
PRE_GATE: 1E.6 done; .elai_cc/crates/ available.
TASK: src/bin/measure_1e.rs. Index .elai_cc codebase. 5 queries:
Q1: "How does the FSM handle state transitions?" (multi-hop)
Q2: "BM25 query latency" (keyword)
Q3: "What implements the LlmProvider trait?" (symbol search, PageRank)
Q4: "Error handling in the orchestrator" (broad semantic)
Q5: "How are tools registered and dispatched?" (relationship)
Per query measure: total_pipeline_latency_ms (target <2000ms); per_stage_latency_ms (bottleneck); tokens_retrieved (full vs snippets); token_savings_pct = 1 - (snippet_tokens / full_chunk_tokens); stages_active.
.elai_cc/metrics/phase_1E.toml: pipeline_p50_ms, pipeline_p99_ms, token_savings_mean_pct (target >50%), catrag_overlay_size, arag_tools_per_query, stages_skipped.
DETERMINISTIC_CHECK: file exists; pipeline_p50_ms+token_savings_mean_pct present.
ACCEPTANCE: 5 queries non-empty; pipeline_p50_ms <2000ms; token_savings_mean_pct >50% (paper claims 55-87%); metrics populated; tests pass (21+).

EXIT CRITERIA: full retrieval pipeline operational; token savings >50%; latency <2s; graceful degradation; tests green; no regression.

================================================================================
## PHASE 1F: SPECULATIVE DECODING EVALUATION (EVALUATION PHASE)
================================================================================

Prereq: Phase 1C (vector search; ModelPool VRAM accounting). Original said "Phase 0 complete (independent of 1A-1E)" — corrected.

GOAL: Evaluate whether speculative decoding improves throughput enough to justify VRAM cost. Decision gate: ADOPT if >1.3x throughput on target hardware, DROP if less. EVALUATION phase, not feature phase. Output is measured decision, not shipped code.

DECISION OUTCOME: DROP (resolved 2026-04-07, auto-pilot P1F.2 commit eeb9a63). Ollama v0.20.2 CONFIRMED no speculative decoding support. PR #8134 closed never merged. Live probe verified: draft_model parameter silently ignored, accepted_tokens always null. Steps 1F.3-1F.5 SKIPPED. Prompt Lookup Decoding (1F.3b) ALSO SKIPPED as part of DROP. Speculative decoding REVISITED in Phase 2+ when vLLM provider implemented (vLLM natively supports spec decode, EAGLE-3, layer-level intervention; ollama is "black box"). The OllamaProvider + LlmProvider trait abstraction built in 1F.1 means vLLM integration is new provider impl, not rewrite.

SPEC CORRECTIONS APPLIED (session s-937656, 2026-04-06): providers.txt L46-47 corrected to 2x-6.5x and ~4.8GB headroom; external-analysis.txt L312,456 to 2x-6.5x; implementation-plan.txt L184 still has stale "1.8-2.3x" (INFO not blocking).

RISKS: R4 VRAM contention RTX 3060 12GB (Gemma 4 E2B ~7.2GB Q4_K_M, headroom ~4.8GB NOT ~10GB as providers.txt L47 incorrectly stated); Scenario A ~2GB draft = 2.8GB OS/CUDA; Scenario B ~3GB delta = 1.8GB tight. Monitor nvidia-smi --query-gpu=memory.used polling.
R-NEW-1 Ollama spec decode NOT confirmed merged (PR #8134 open Apr 2026). Fallback: (a) llama.cpp direct (only viable Phase 1 — vLLM doesn't exist until Phase 2+, providers.txt L55), (b) defer.
R-NEW-2 EAGLE-3 published 2x-6.5x measured on 70B+ models. Gains on 2.3B on consumer GPU substantially lower.
R-NEW-3 Gemma 4 EAGLE-3 draft model may not exist (vLLM #38893 unresolved). Standard speculative (same-family smaller) used if EAGLE-3 unavailable.

GROUNDING UPDATES: EAGLE-3 1.8-2.3x CORRECTED to 2x-6.5x (NeurIPS'25, RedHat/Vertex). Headroom ~10GB WRONG: 12-7.2=4.8GB. TIDE 1.15x VERIFIED (arxiv 2602.05145). Ollama UNVERIFIED. >1.3x adoption threshold UNGROUNDED. Draft model size critical (llama.cpp #10466: 3B=1.08x@k=1, 1.5B=1.17x@k=4, 0.5B=1.4x@k=4 — smaller drafts outperform larger ones on consumer GPUs). num_draft_tokens=5 SUBOPTIMAL — sweep instead. Acceptance rate: below 0.5 spec decode HURTS (vLLM blog 2024-10-17). Prompt lookup decoding: zero-VRAM, 2.8x on summarization.

### 1F.1 SPECULATIVE DECODING CONFIG SKELETON
PRE_GATE: Phase 1C tests pass; providers compiles; OllamaConfig exists.
TASK: Add speculative_decoding: bool (default false), draft_model: Option<String>, num_draft_tokens: u8 (default 5) to OllamaConfig or nested SpeculativeConfig. Serde support (de+serialize).
DETERMINISTIC_CHECK: cargo check 0; 3+ field hits; existing tests pass.
ACCEPTANCE: 0 errors; defaults match (false, None, 5); serde round-trip preserves all fields.

### 1F.2 OLLAMA SPECULATIVE DECODING API AVAILABILITY CHECK
PRE_GATE: 1F.1 done; ollama running; main model loaded.
TASK: Probe ollama for draft_model parameter in /api/generate or /api/chat. If NOT supported: document, evaluate fallback (llama.cpp only). If neither viable: DROP. If supported: identify exact API params. Draft model selection: prefer 0.5B-1.5B over 2B-3B (llama.cpp #10466 evidence). Record draft model name, params, quantization, VRAM.
DETERMINISTIC_CHECK: probe exits 0 with "SUPPORTED: [params]" or "NOT_SUPPORTED: [reason]"; if NOT_SUPPORTED phase-1F-decisions.md documents fallback.
ACCEPTANCE: AVAILABLE or NOT_AVAILABLE binary determination.
RESULT: NOT_AVAILABLE -> proceed to 1F.5 DROP.

### 1F.3 CORRECTNESS VERIFICATION (SKIPPED)
PRE_GATE: 1F.2 result is AVAILABLE (skip to 1F.5 DROP otherwise).
TASK: Generate output WITHOUT spec (baseline), then WITH (same prompt+seed). Spec decoding algorithmically lossless (draft tokens verified by target). 10 prompts minimum. Compare decoded text (soft match: identical text); token ID divergence WARN not fail.
DETERMINISTIC_CHECK: phase_1F_correctness.toml with 10/10 text matches.
ACCEPTANCE: 10/10 identical decoded text; seed=fixed, temp=0; token ID divergence WARN.

### 1F.3b PROMPT LOOKUP DECODING BASELINE (SKIPPED — part of DROP)
PRE_GATE: 1F.3 passes OR 1F.2 NOT_AVAILABLE.
TASK: Benchmark prompt lookup (n-gram speculation from input). Zero VRAM, no draft. Up to 2.8x on summarization (vLLM blog). Same 5 prompts as 1F.4. 3 runs each (15 measurements). Record tokens/sec vs baseline. If ollama doesn't support: check llama.cpp --lookup-ngram-min. If neither: skip, "UNAVAILABLE".
DETERMINISTIC_CHECK: phase_1F_prompt_lookup.toml with 15 measurements + ratio OR "UNAVAILABLE".
ACCEPTANCE: ratio measured or unavailable; feeds into 1F.5.

### 1F.4 THROUGHPUT BENCHMARK (SKIPPED)
PRE_GATE: 1F.3 passes (10/10).
TASK: Tokens/sec WITH/WITHOUT spec. 5 diverse prompts (short/medium/long; code/prose/structured). PARAMETER SWEEP k={1,2,3,4,5,7}. 3 runs per prompt per k per mode. Record best k. Per run: prompt length, output length, wall time, tokens/sec, GPU util, ACCEPTANCE RATE. EARLY KILL: if acceptance <0.5 across all k for 3+ prompts -> STOP, DROP. Mean+stddev per mode at best-k.
DETERMINISTIC_CHECK: phase_1F_throughput.toml; best k per prompt; mean acceptance; ratio at best-k; stddev <20% per mode.
ACCEPTANCE: all measurements; stddev <20% per mode; ratio single number; acceptance measured/inferred; best k per prompt; GPU temp <85C; early kill documented if triggered.

### 1F.5 VRAM MEASUREMENT AND DECISION GATE
PRE_GATE: 1F.4 done OR 1F.2 NOT_AVAILABLE.
TASK: Measure baseline_vram_mb, speculative_vram_mb, delta_vram_mb (peak via nvidia-smi --query-gpu=memory.used -l 1 polling).
DECISION GATE (top-to-bottom, stop at first match):
- If NOT_AVAILABLE: DROP (current case). Document "spec decode not available on current stack as of [date]"
- If acceptance <0.5: DROP. "acceptance rate too low — spec decode hurts"
- If prompt_lookup >1.3x AND spec_decode <= prompt_lookup: DROP spec decode, ADOPT prompt lookup (zero VRAM)
- If spec_decode >1.3x AND delta_vram_mb <3000 (1.8GB margin on 3060): ADOPT. Enable for executor. Record best-k. If prompt_lookup also >1.3x: prefer spec only if measurably better
- If spec_decode <=1.3x OR delta_vram_mb >=3000: DROP. Reclaim VRAM for embedding models
- Boundary ratio==1.3x or delta==3000MB: DROP (conservative tie-break)
Write to workflow/reasoning/phase-1F-decisions.md.
DETERMINISTIC_CHECK: phase_1F_vram.toml peak baseline+spec VRAM; phase-1F-decisions.md ADOPT or DROP with justification; if ADOPT config default updated.
ACCEPTANCE: VRAM delta in MB with nvidia-smi evidence; binary ADOPT/DROP; doc with throughput ratio, VRAM delta, hardware, date, rationale; if ADOPT config compiles; if DROP no config changes.

EXIT CRITERIA: ADOPT or DROP documented. If ADOPT: which method (spec or prompt lookup), at which config (best-k, draft model). All measurements in metrics/phase_1F_*.toml. Decision in workflow/reasoning/phase-1F-decisions.md. If ADOPT spec: defaults updated, best-k set, draft specified. If ADOPT prompt lookup: defaults updated no draft. If DROP both: spec disabled, VRAM reclaimed for embedding. Phase 0 + 1A-1C tests still pass.

MEASUREMENTS: spec at best-k target >1.3x (UNGROUNDED); prompt lookup zero-VRAM baseline; acceptance rate >0.5 (vLLM blog evidence); optimal k (sweep {1,2,3,4,5,7}); VRAM overhead <3000MB; draft size 0.5B-1.5B optimal; EAGLE-3 ref 2x-6.5x (NeurIPS'25); TIDE 1.15x ref (arxiv 2602.05145). If <1.3x: disable, reclaim VRAM for embedding models (Phase 1C).


================================================================================
## PHASE 1G: CROSS-ENCODER RERANKER
================================================================================

Prereq: Phase 1E (CatRAG + A-RAG operational).

NOTE (2026-04-07): Phase 1F steps 1F.3-1F.5 DROPPED. Phase 1F deliverables: OllamaProvider skeleton (1F.1) + probe infrastructure (1F.2). No impact on 1G — 1G depends on 1E (pipeline) not 1F (inference speed).

GOAL: Add cross-encoder reranking (ms-marco-MiniLM-L12-v2) after A-RAG, before token-budget fitting. CPU-only, target <50ms latency for 50 candidates. Concurrency-safe with bounded queue, request batching, circuit breaker integration. ONNX-optimized (graph optimizations + optional INT8 quantization).

GROUNDING:
"cross-encoder/ms-marco-MiniLM-L-12-v2" VERIFIED 33.4M params 12 layers hidden_size=384 base=microsoft/MiniLM-L12-H384-uncased; WordPiece tokenizer (BertTokenizer); requires tokenizer.json bundled.
"~120MB ONNX fp32" VERIFIED Xenova/ms-marco-MiniLM-L-12-v2 ~120MB (0.12GB). Spec said ~130MB — corrected.
"~30MB ONNX int8" VERIFIED dynamic int8 quantization ~4x reduction (Microsoft Azure: BERT-base). 120MB/4 ~ 30MB. No calibration dataset.
"ort crate" CONDITIONAL pykeio/ort v2.0.0-rc.12 (RC, not stable; API changed rc.8->rc.10).
"tokenizers crate" VERIFIED huggingface/tokenizers required for WordPiece -> input_ids/attention_mask/token_type_ids.
"<50ms CPU for 50 candidates" CONDITIONAL no published L-12 CPU benchmarks. V100 GPU 960 docs/sec. FlashRank claims sub-20ms with optimized ONNX. Empirical gate with 3 levers: graph opt, int8, threads. If still >50ms: spec amendment.
"ORT_ENABLE_ALL graph optimization" VERIFIED CPU layout opts (NCHWc); BERT SQuAD F1 87.05 vs 87.03.
"INT8 dynamic quantization up to 6x" VERIFIED VNNI ~6x; AVX2 ~2.9x.
"intra-op thread scaling" CONDITIONAL Milvus AI cites 1->8 = 4.2x (AI-generated source); empirical in 1G.5.B.
"L-6-v2 fallback" VERIFIED 22.7M params 6 layers MRR@10=36.81 vs L-12 39.02. Same WordPiece. SPEC AMENDMENT REQUIRED: workflow/reasoning/phase-1G-spec-amendment.md (current spec L149 only "skip" as fallback).
"skip when candidates <=10" GROUNDED knowledge.txt L96 ("below that ordering noise > signal"). Configurable.
"tokio::Semaphore(1) + bounded queue depth 4" GROUNDED L219-220.
"batching queued" GROUNDED L221.
"circuit breaker for cross-encoder" GROUNDED L135, L149.
"SSB degradation" GROUNDED L124 action_type='knowledge_degradation' separate from tracing.
"startup canary includes rerank" GROUNDED L159: embed+BM25+rerank within 5s.
"injectable clock" GROUNDED entropy quarantine.
"max sequence length 512" VERIFIED standard BERT/MiniLM.
"enable_deterministic_compute multi-thread" VERIFIED ORT issue #4611 — CPU MATMUL with threads non-deterministic float reduction unless enable_deterministic_compute=true.

DECISION ort vs fastembed-rs: raw ort + tokenizers (other ONNX models already use ort directly; raw allows ORT_ENABLE_ALL/enable_deterministic_compute/int8 directly; fastembed limits control). Revisitable Phase 1I.
DECISION fp32 vs int8: start fp32. INT8 trigger if baseline p50 >30ms (60% of 50ms budget, 20ms headroom for queue wait under <4 agents per L222). Accept int8 only if NDCG@10 delta <0.5 points.
DECISION skip threshold: default 10. Sweep 5,10,15,20,30 in 1G.5 with NDCG@10. Update if different optimum for code search.
DECISION batching: tokio::sync::mpsc bounded channel (capacity 4) over Notify (atomic drain-and-batch via try_recv loop, avoids lost-wakeup races). Reranker task drains channel after acquiring semaphore, merges candidate sets, runs single inference, splits via per-caller oneshot channels.

### 1G.1 RERANKER MODULE SKELETON + DEPENDENCIES
PRE_GATE: Phase 1E tests pass; KnowledgeEngine has rerank method (record exact signature: async? Result type? param types?); ort compiles after add; ONNX model + tokenizer.json present.
TASK: Add ort (pinned RC), tokenizers to knowledge Cargo.toml. Create reranker module stub.
CrossEncoderReranker struct fields: session (ort::Session), tokenizer (tokenizers::Tokenizer), semaphore (tokio::Semaphore), request_channel (mpsc::Sender for batch aggregation), circuit_breaker (CircuitBreaker), clock (Arc<dyn ClockProvider>).
RerankerConfig:
  model_path: PathBuf (primary L-12)
  tokenizer_path: PathBuf (shared L-12 + L-6 same BERT base)
  fallback_model_path: Option<PathBuf> (L-6 SPEC AMENDMENT)
  quantized_model_path: Option<PathBuf> (int8 from 1G.5)
  skip_threshold: usize (default 10, tuned 1G.5)
  max_sequence_length: usize (512)
  max_queue_depth: usize (4)
  intra_op_threads: usize (1, tuned 1G.5)
  graph_optimization_level: GraphOptimizationLevel (ORT_ENABLE_ALL)
  enable_deterministic_compute: bool (true)
Implement KnowledgeEngine::rerank stub (returns candidates unchanged) matching trait signature.
Wire call site between A-RAG output and token-budget fitting. SPARSE_ONLY/BM25_ONLY skip reranker (L118-119).
DETERMINISTIC_CHECK: cargo check 0; CrossEncoderReranker exists; fn rerank matches trait; tokenizers + ort in Cargo.toml; graph_optimization_level/ORT_ENABLE_ALL/enable_deterministic_compute hits.
ADVERSARIAL: ort RC yanked -> compile fail; missing default config -> runtime panic; trait signature mismatch -> compile fail.

### 1G.2 ONNX MODEL LOADING, TOKENIZER, AND GRAPH OPTIMIZATION
PRE_GATE: 1G.1 done; ONNX model + tokenizer.json present; verify ort Session builder API against actual rc.12 (graph opt enum, enable_deterministic_compute option).
TASK: ONNX loading via ort session builder (verified API). graph_optimization_level=ORT_ENABLE_ALL (CPU layout opts, no accuracy loss). enable_deterministic_compute=true (reproducible float reduction across threads). CPU execution provider only (no CUDA/TensorRT). Configurable intra-op thread count (default 1 for deterministic tests). Tokenizer from tokenizer.json via tokenizers::Tokenizer::from_file(). Truncation: max_length=512, truncation=true. Path resolution: quantized first (if Some) then model_path then local cache. L-6 lazy load via fallback_model_path: NOT pre-loaded. Created on first L-12 breaker open. Cold-start 5-15s acceptable (no SLA on fallback). Error handling: Result, no unwrap/panic, map ort errors to KnowledgeError.
DETERMINISTIC_CHECK: unit tests load model+tokenizer Ok; invalid path returns Err (not panic); missing tokenizer returns Err; tokenizer.encode("hello world") non-empty Vec<u32>; truncates >512 to 512; ORT_ENABLE_ALL produces identical scores to ORT_DISABLE_ALL (score identity); enable_deterministic_compute=true + 4 threads produces identical scores 5 runs.

### 1G.3 PAIR SCORING, RERANKING, AND CONCURRENCY
PRE_GATE: 1G.2 done.
TASK: Tokenize (query, chunk) pairs via tokenizer.encode_batch() with pair encoding -> input_ids, attention_mask, token_type_ids. Batch inference: collect tensors, single ort session.run(). Extract relevance scores (single logit per pair). Re-sort DESC. Skip when count <= skip_threshold (default 10) — return unchanged.
Concurrency: tokio::Semaphore(1) guards session (L219). Bounded mpsc (capacity 4): each caller sends (candidates, oneshot::Sender<Result>). Channel full (try_send Full) -> return candidates unranked immediately (L220).
Request batching: reranker task drains channel via try_recv after acquiring semaphore. Merges drained candidate sets into single batch. One ort session.run(). Splits results by tracked offsets. Returns via per-caller oneshot. (L221)
DETERMINISTIC_CHECK unit tests: 20 candidates -> output order differs; 5 candidates passed unchanged; 10 candidates passed unchanged (boundary <=10); 11 reranked (>10 boundary); skip_threshold=5 config 7 reranked; empty list returns empty (no panic); 1000-token chunk valid score (truncation); tokenizer produces correct shapes; 5 concurrent only 1 executes; 6th when 4 queued returns unranked; 2 queued requests batched (batch count=1 not 2); 2 concurrent distinct sets each gets own (no cross-contamination via offsets).

### 1G.4 CIRCUIT BREAKER, DEGRADATION, SSB EVENTS, AND MODEL TIERING
PRE_GATE: 1G.3 done; CircuitBreaker exists (Phase 1C/1E); ClockProvider exists; SSB write mechanism exists; spec amendment filed.
TASK: Independent breaker for L-12 (L135) using injectable ClockProvider (no std::time::Instant). Fallback cascade on L-12 failure:
1. If fallback_model_path configured: lazy-load L-6 (first failure only, cached). L-6 own breaker.
2. If L-6 also fails (or not configured): skip reranking, return A-RAG order (L149 spec-defined final fallback).
NOTE: L-6 intermediate tier requires spec amendment approval (workflow/reasoning/phase-1G-spec-amendment.md). If rejected: remove L-6 path.
SPARSE_ONLY/BM25_ONLY skip reranker entirely (1G.1 wired).
SSB event on degradation: action_type='knowledge_degradation' with stage='rerank' (L124). SEPARATE from tracing. SSB write failure non-fatal (log warning continue with fallback).
Tracing DEBUG: emit StageTrace for rerank in QueryTrace (L238).
Breaker config: failure_threshold=3, window=60s, half_open_timeout=30s (L139-142), durations via ClockProvider.
DETERMINISTIC_CHECK unit tests: 3 consecutive L-12 failures opens breaker; with breaker open + no L-6 -> rerank returns unchanged; mock clock advances past timeout -> probe succeeds, closes (<1s test); breaker open -> SSB event emitted; breaker close (recovery) -> SSB event; SSB write failure does not crash; "rerank" in StageTrace; SPARSE_ONLY rerank not called; L-12 open + L-6 configured -> L-6 lazy-loaded; both open -> skip; L-6 fallback path nonexistent -> graceful skip not panic.

### 1G.5 STARTUP CANARY UPDATE
PRE_GATE: 1G.4 done; readiness_check exists; canary invokes embed+BM25.
TASK: Update canary to include rerank (L159: embed + BM25 + rerank). Canary rerank: 3 candidates from BM25 canary output through reranker. All within 5s (L160). If reranker fails during canary: enter degraded mode with reranker skipped. Log canary result including reranker latency.
DETERMINISTIC_CHECK: healthy reranker -> READY includes rerank; broken reranker -> DEGRADED (not UNAVAILABLE — embed+BM25 work); canary completes within 5s.

### 1G.6 OPTIMIZATION SWEEP AND PRECISION VALIDATION
PRE_GATE: 1G.1-1G.5 done; existing tests pass; tests/fixtures/reranker_benchmark.json committed (1 query, 50 candidates, binary relevance, >=10 relevant in top 50, ACTUAL code chunks); INT8 quantization toolchain available if baseline >30ms (scripts/quantize_cross_encoder.py committed).
TASK execute in order:
A. BASELINE (fp32, ORT_ENABLE_ALL, deterministic, 1 thread): rerank 50 candidates warm session p50/p99 (10 runs); NDCG@10 with/without on fixture; record baseline.
B. THREAD SCALING SWEEP (1/2/4/8 threads): 10 runs each p50/p99; determinism check (5 runs identical ranking order or top-5 identical set+order); optimal_threads = lowest p50 passing determinism.
C. INT8 QUANTIZATION (only if baseline p50 >30ms): run scripts/quantize_cross_encoder.py; validate quantized loads in ort; benchmark at optimal_threads with int8; NDCG@10 delta vs fp32; accept only if delta <0.5 points; if accepted set quantized_model_path in default config.
D. SKIP THRESHOLD SWEEP: skip_threshold = 5,10,15,20,30; NDCG@10 at threshold boundary (N+1 candidates) and at 50; optimal = lowest where NDCG@10 at N+1 shows measurable improvement (delta >0.02) over unranked.
E. FINAL MEASUREMENT: full benchmark all optimizations (optimal threads, best variant, optimal threshold). Record final_latency_p50/p99, final_ndcg_at_10, model_variant. If final_p50 >50ms: spec amendment filed with data. Model load time (cold) recorded separately. Queue wait p99 under 4-agent load recorded for future session pool eval (L224).
DETERMINISTIC_CHECK: fixture exists with code chunks; baseline_latency_p50, thread_sweep, int8_latency/int8_ndcg_delta (if attempted), threshold_sweep, final_latency_p50 in metrics; NDCG@10 improvement >0; if final p50 >50ms: spec amendment updated.
ACCEPTANCE: tests pass; reranking changes order for >threshold; skip logic correct + configurable + tuned; breaker opens after 3 failures; L-6 fallback if configured; final fallback skips (L149); SSB event on state changes (non-fatal if SSB unavailable); queue overflow returns unranked; batching merges via channel drain + offset-tracked split; ORT_ENABLE_ALL + enable_deterministic_compute applied; threads optimized + determinism verified; INT8 evaluated if baseline >30ms accepted only if NDCG delta <0.5; canary includes rerank; latency measured all opts; if p50 >50ms spec amendment with data; NDCG@10 measurable on fixture; StageTrace emitted; semaphore prevents concurrent ONNX; ClockProvider injected (no Instant::now); tokenizer correct shapes; no unwrap/panic.

EXIT: Reranker measurably improves NDCG@10 on committed fixture. Latency optimized via graph opt + deterministic compute + thread scaling + optional int8. Final latency documented. If >50ms after all opts: spec amendment filed with data. Skip threshold tuned. Circuit breaker, degradation (with L-6 if amendment approved), SSB events, startup canary all wired. Concurrency-safe: semaphore + bounded mpsc 4 + overflow unranked + batching via channel drain. Tokenization via tokenizers crate. Injectable ClockProvider. All Phase 1G tests + pre-existing knowledge tests pass.

MEASUREMENTS: baseline_latency p50/p99 (fp32, ORT_ENABLE_ALL, deterministic, 1 thread); thread scaling sweep (1,2,4,8 — latency + ranking determinism); INT8 latency p50/p99 + NDCG@10 delta vs fp32; skip threshold sweep (5,10,15,20,30 NDCG@10); final optimized latency p50/p99; model variant (fp32/int8); NDCG@10 delta on committed fixture; model load cold (L-12 + L-6); queue wait p99 under 4 agents; circuit breaker recovery time (mock clock); startup canary total + per-stage breakdown.

================================================================================
## PHASE 1H: SPLADE KEYWORD EXPANSION (CONDITIONAL — EVALUATION PHASE)
================================================================================

Prereq: Phase 1G.

GOAL: Evaluate SPLADE sparse keyword expansion. Feature-gated behind cfg(feature="splade"). Compare two model candidates: prithivida/Splade_PP_en_v1 (established, ONNX-ready) and naver/splade-v3 (newer, ensemble distillation, outperforms PP_en_v1 on TREC/BEIR per arxiv 2403.06789). Pick better-performing candidate on code corpus, then keep/drop gate. DROP if <5% recall improvement over BM25-only on test corpus.

DECISION OUTCOME: Three possible outcomes:
1. EARLY DROP (vocab coverage <0.3) — phase complete without recall measurement. SPLADE fundamentally mismatched for code corpus.
2. DROP (delta <5% OR p >=0.05) — improvement absent or statistically insignificant.
3. KEEP (delta >=5% AND p <0.05) — statistically significant improvement confirmed.

CRITICAL RISK — BERT vocabulary mismatch on code: SPLADE operates in BERT WordPiece (30522 tokens). Code tokens like "fn", "impl", "async", "Vec<T>", "unwrap()", "println!" may be aggressively subword-split. HuggingFace research confirms "vocabulary mismatch can destroy the representation entirely". MUST measure vocab coverage before concluding.

NOTE: knowledge.txt L58 has typo "prithvida" (correct: "prithivida"). File correction.

RISKS: R24 SPLADE ONNX dep weight ~532MB main vs marginal recall gain on code; R25 SPLADE benchmarks 29% nDCG@10 ESCI are NL — code recall may differ; R26 ort downloads prebuilt from Microsoft (may fail air-gapped); R27 SPLADE requires BERT tokenization (input_ids/attention_mask/token_type_ids) — ort doesn't include tokenization; R28 ort unconditional dep (GraphCodeBERT/Granite) — SPLADE feature gates only ort::Session init; R29 BERT WordPiece ~30522 trained Wikipedia/BookCorpus — programming tokens NOT in vocab, split into char n-grams; R30 10-query eval underpowered (TREC 50 minimum, Sakai 2016).

GROUNDING:
"prithivida/Splade_PP_en_v1" VERIFIED HF model exists, ONNX 532MB main, 436MB quantized.
"~500MB ONNX" VERIFIED 532MB actual.
"ort crate" VERIFIED pyke/ort.
"RRF fusion with BM25" GROUNDED knowledge.txt L61, L88.
"5% recall threshold" GROUNDED knowledge.txt L62.
"SPLADE improves recall over BM25" VERIFIED 29% nDCG@10 ESCI (Qdrant).
"error handling matches try/catch" GROUNDED L60 SPLADE design intent.
SPLADE recall on CODE SEARCH UNGROUNDED — all benchmarks NL (ESCI, TREC DL).
fastembed-rs alternative GROUNDED architecture-overview.txt L280 EVALUATE.
"naver/splade-v3" VERIFIED HF model. Ensemble distillation + fused losses. Outperforms SPLADE++ on TREC DL + BEIR (arxiv 2403.06789). ONNX export via sentence-transformers.
BERT vocab on code tokens VERIFIED HF blog confirms vocab mismatch destroys SPLADE in specialized domains. 30522 tokens, none programming-specific.
10-query power VERIFIED TREC=50 minimum. Sakai 2016 systematic review: 10 insufficient for <~15% effect sizes. Wilcoxon + Benjamini-Hochberg recommended.
Quantized ONNX VERIFIED onnx-models/Splade_PP_en_v1-onnx 436MB int8. ~3x CPU speedup. Quality loss typically <1% nDCG.

### 1H.1 SKELETON — splade module stub (feature-gated)
PRE_GATE: Phase 1G done; cargo check workspace 0; ort resolvable.
TASK:
- Cargo.toml: [dependencies.ort] version="..." optional=true (with feature gate)
- .elai_cc/src/knowledge/splade.rs with cfg(feature="splade") on entire module
- Stub: pub struct SpladeExpander (no fields), pub fn expand_query(&self, query: &str) -> Vec<(String, f32)> returning empty vec
- Re-export from knowledge/mod.rs behind #[cfg(feature = "splade")]
DETERMINISTIC_CHECK: cargo check (no features) 0 — splade NOT included; cargo check --features splade 0 — splade included; without splade no SpladeExpander symbols (ort itself unconditional for GraphCodeBERT/Granite — do NOT test for ort absence).
ACCEPTANCE: cargo check no features 0 errors no SpladeExpander; cargo check --features splade 0 errors SpladeExpander compiles; ort remains regardless of splade feature.

### 1H.2 TEST — write failing tests before implementation
PRE_GATE: 1H.1 done.
TASK: tests/knowledge/splade_tests.rs (feature-gated #[cfg(feature = "splade")]):
- expand_query("error handling") returns sparse vector with "error", "exception", "try", "catch", "handling" non-zero weights
- expand_query("") returns empty (edge case)
- expand_query >10000 chars no panic, returns result or graceful Err
- RRF fusion of SPLADE + BM25 produces combined ranking different from BM25-only
- Property: any non-empty query string -> all weights finite and >=0.0
- Integration test (marked): recall measurement on synthetic corpus BM25-only vs BM25+SPLADE
DETERMINISTIC_CHECK: tests compile with --features splade; tests fail (TDD).
ACCEPTANCE: file compiles; >=5 unit + 1 property + 1 integration; tests FAIL (red phase).

### 1H.3 IMPLEMENT — SPLADE inference + RRF fusion
PRE_GATE: 1H.2 tests compile (--no-run); tokenizers crate present (may already be from Phase 1G); ONNX model accessible (default .elai_cc/models/splade/model.onnx, override SPLADE_MODEL_PATH env).
TASK: Ensure tokenizers in Cargo.toml. SpladeExpander::new(model_path) loads ONNX via ort::Session, BERT tokenizer via tokenizers (vocab.txt + tokenizer.json). Model path configurable for swapping candidates.
SpladeExpander::expand_query(): tokenize to (input_ids, attention_mask, token_type_ids); run inference; apply SPLADE pooling; return top-k (token, weight) pairs.
IMPORTANT — ONNX output varies by source:
- PP_en_v1 (prithivida ONNX): outputs MLM logits directly. Apply max-pooling over token positions, then ReLU + log(1+x) for sparse vector.
- splade-v3 (sentence-transformers optimum export): outputs raw MLM logits only — SPLADE pooling NOT baked in. expand_query() MUST apply pooling in Rust. model_variant enum (PrePooled, RawMLM) dispatch. Verify output shape: [1, seq_len, 30522] RawMLM vs [1, 30522] PrePooled.
SpladeExpander::vocab_coverage(corpus_tokens: &[String]) -> f64: fraction of unique code tokens that map to single WordPiece (not split). Returns 0.0-1.0. Vocabulary fitness metric — if <0.3 SPLADE unlikely to help regardless of model.
Weighted RRF fusion: rrf_fuse(bm25_results, splade_results, k=60, weight_bm25=0.5, weight_splade=0.5) -> merged ranking. Ranks 1-based. Guard k=max(k,1).
Circuit breaker integration (knowledge.txt §6).
Fallback on SPLADE failure: empty expansion (BM25-only continues).
Both candidates via config:
- A: prithivida/Splade_PP_en_v1 (532MB full, 436MB int8 quantized)
- B: naver/splade-v3 (export via sentence-transformers optimum)
Same SpladeExpander interface, only model_path differs.
DETERMINISTIC_CHECK: unit tests pass with --features splade; property test passes; non-feature build still 0; vocab_coverage returns valid ratio.
ACCEPTANCE: tests pass; weights finite and >=0.0; corrupted model returns Err (not panic); non-feature build 0 errors no SpladeExpander; single query <2s on CPU (100-char, p50 of N=10 excluding warm-up); missing tokenizer files returns Err; vocab_coverage 0.0-1.0; both candidates load + produce output via same interface.

### 1H.4 MEASURE — vocabulary fitness, model comparison, recall, keep/drop decision
PRE_GATE: 1H.3 done; integration test infra exists.
TASK execute in order — each gates next:

4a. VOCABULARY FITNESS CHECK (early-exit gate):
- Extract unique code identifiers from synthetic corpus via tree-sitter (function/type/var names + keywords — NOT string literals or comments)
- Run vocab_coverage() on extracted identifiers
- If code-specific coverage <0.3: EARLY DROP. Log "SPLADE vocab coverage on code is [X]% — below 30% threshold. BERT WordPiece fundamentally mismatched. SPLADE will not meaningfully expand. Decision: DROP without recall measurement." Skip 4b-4d.
- If >=0.3: log + proceed to 4b
- UNGROUNDED: 0.3 threshold is conservative heuristic (NOT published). Reasoning: <30% means >70% identifiers split into 2+ subwords diluting BERT contextualized expansion. Should be revisited with production data. If coverage 0.2-0.4: consider proceeding to recall and using result to calibrate data-driven threshold.

4b. MODEL VARIANT SELECTION:
- PRE_GATE: model files pre-staged (download is prereq OUTSIDE session, 500MB+ each). Verify .elai_cc/models/splade/model.onnx + .elai_cc/models/splade-v3/model.onnx. If splade-v3 unavailable: Candidate A only, log "B unavailable — skipped".
- Quantization (Jaccard valid for same-model): A-full (532MB) and A-quant (436MB int8) on 5 sample queries. Top-50 expanded terms Jaccard. If >=0.95 prefer A-quant (saves 96MB + ~3x speed). Jaccard ONLY valid for same-model precision regression, NOT cross-model.
- Cross-model (A vs B requires actual recall): if B available run quick recall@10 on 10 of 50 queries for A-winner and B. Pick higher mean recall@10. If within 1%: pick faster.
- Log table (candidate, p50_latency_ms, quantization_jaccard, quick_recall) to workflow/reasoning/phase-1H-measurements.md.

4c. RECALL MEASUREMENT (statistically powered):
- Synthetic corpus: 100 code files with known relevant matches for 50 queries. TREC standard 50 topics minimum (Sakai 2016). 50 queries + Wilcoxon enables ~5% differences at p<0.05 with ~80% power.
- Query design: >=15 vocabulary-mismatch ("error handling" -> try/catch, "concurrent access" -> mutex/rwlock/arc); >=15 with distractors (sharing keywords but not relevant); >=10 straightforward keyword (sanity baseline).
- For winning candidate measure BM25-only recall@10 per query AND BM25+SPLADE (RRF fused) recall@10 per query.
- Per-query delta: if bm25_recall>0: (splade_recall - bm25_recall)/bm25_recall * 100. If bm25_recall==0: exclude + log warning.
- Wilcoxon signed-rank on paired (bm25_recall, splade_recall). Report p-value.
- Log: mean delta, std, p-value, per-query breakdown.

4d. KEEP/DROP DECISION:
- KEEP if: mean delta >=5% AND p<0.05 (statistically significant)
- DROP if: mean delta <5% OR p>=0.05 (absent or noise)
- EDGE CASE: delta >=5% but p>=0.05 -> "promising but inconclusive, need larger corpus". Default DROP, preserve module for future re-eval with production data.
- Log full rationale.

DETERMINISTIC_CHECK: vocab coverage produces numeric output; comparison table logged (if vocab passed); recall produces per-query (if vocab passed); decision documented with statistical test (wilcoxon|p_value|EARLY DROP).
ADVERSARIAL: synthetic corpus too easy -> recall 100% both, delta 0% (>=15 queries with distractors); all queries NL-style -> SPLADE looks good on training but fails on real code (>=15 code-specific syntax queries); subset measurement -> bias (assert all 50 run; failing queries log error don't drop); SPLADE recall after BM25 caching -> invalid (random order or clear caches); too few non-zero deltas -> insufficient power (assert >=20 non-zero pairs; if fewer log "insufficient" + DROP); quantized dramatically different (Jaccard <0.80 vs full) -> quality loss exceeds <1% nDCG estimate (assert Jaccard >=0.80 sample queries).
ACCEPTANCE: vocab coverage on code-specific tokens (not all). If <0.3: EARLY DROP doc + rationale + phase complete. If >=0.3: comparison table latency+overlap. Recall on 50 queries 100 corpus files (or EARLY DROP). Wilcoxon p-value (or EARLY DROP). Decision (KEEP if delta>=5% AND p<0.05 else DROP). If DROP: feature flag remains but not enabled by default; module preserved.

EXIT CRITERIA: Three outcomes (EARLY DROP / DROP / KEEP). All: feature flag works, module preserved behind cfg(feature="splade"). If KEEP: circuit breaker integrated, RRF fusion operational, winning candidate documented. If any DROP: feature flag disabled by default, rationale + measurements logged. SPARSE_ONLY cascade: with splade disabled SPARSE_ONLY must degrade to BM25_ONLY (no dead code path).

MEASUREMENTS: vocab_coverage_code (0.0-1.0, threshold 0.3); model_comparison table (candidate, p50_latency_ms, top50_term_jaccard); recall_delta_percent per query (excluding bm25=0, mean+std across 50); wilcoxon_p_value (threshold p<0.05); splade_query_latency_ms p50/p95 (N=10 excluding warm-up); model_load_time_s; binary_size_delta_mb (--features splade vs without).


================================================================================
## PHASE 1I: RESILIENCE LAYER (circuit breaker, degradation, startup, tracing)
================================================================================

Prereqs: Phase 1E (query pipeline) AND Phase 1G (cross-encoder for semaphore).

BLOCKING NOTE: implementation-guide.txt L141 dep graph shows 1E -> 1I only. INCOMPLETE. Phase 1I also requires 1G because step 1I.5 adds concurrency around cross-encoder delivered in 1G. Dep graph MUST be updated to show both 1E -> 1I AND 1G -> 1I before this phase. Without fix, implementer following dep graph starts 1I after 1E but before 1G -> compile failures at 1I.5. ACTION: update implementation-guide.txt L141.

GOAL: Make pipeline survive individual component failures gracefully. Circuit breakers, degradation cascade, startup readiness, query validation, cross-encoder concurrency control, pipeline tracing, index versioning.

### 1I.1 CIRCUIT BREAKER
PRE_GATE: cargo test --lib passes (Phase 1G baseline); knowledge compiles; cross-encoder reranker exists (1G); ClockProvider trait exists (entropy quarantine).
TASK: EmbeddingCircuitBreaker struct with states CLOSED, OPEN, HALF_OPEN. Independent instance per ONNX model (GraphCodeBERT, Granite, cross-encoder, SPLADE).
Config: failure_threshold=3 (transient), failure_window=60s, half_open_timeout_base=30s.
All time access via ClockProvider (entropy quarantine, deterministic).
Failure classification (Azure Architecture Center pattern):
- TRANSIENT (timeout, temp OOM): count toward threshold (3 to open)
- PERMANENT (segfault, missing file, invalid ONNX): open immediately after 1 failure
- Classification via error type enum, NOT string matching
Exponential backoff for successive OPEN (AWS Builders Library):
- First OPEN: 30s. Second: 60s. Third: 120s. Cap: 120s.
- Deterministic jitter per model (hash of model_id, NOT random) — prevents thundering herd
State NOT checkpointed (resets to CLOSED on restart).
On OPEN: skip model, return fallback immediately.
On HALF_OPEN: allow limited probe calls. Require 2 CONSECUTIVE successes before CLOSED (Azure pattern: success counter prevents premature recovery on flaky models). Any failure -> OPEN (reset timer, increase backoff).
Fresh ONNX InferenceSession for HALF_OPEN probes (don't reuse potentially-corrupted — ONNX issues #20043/#20354 show sessions can crash/hang).
DETERMINISTIC_CHECK: cargo test circuit_breaker passes; EmbeddingCircuitBreaker definition present; CLOSED/OPEN/HALF_OPEN variants present.
ACCEPTANCE: 3 transient failures within 60s opens (state==OPEN); 1 permanent opens immediately; failures spread >60s stay CLOSED (3 at 0s/61s/122s); HALF_OPEN requires 2 consecutive successes for CLOSED (1 success still HALF_OPEN); HALF_OPEN failure after 1 success resets counter (success+failure -> OPEN); exponential backoff first=30s, second=60s, third=120s (mock clock); jitter deterministic (same model_id same offset); 4 models distinct HALF_OPEN times; fresh session on probe (mock verifies InferenceSession::new during HALF_OPEN); each model independent.
GROUNDING: knowledge.txt L139 "3 failures within 60s"; L141-142 HALF_OPEN after 30s timeout (base + backoff); L151 state NOT checkpointed; L135 Independent per ONNX model; Azure success counter; AWS exponential backoff with jitter; Azure failure classification; ONNX issues #20043 #20354 fresh session.

### 1I.2 SEARCHMODE CASCADE AND DEGRADATION
PRE_GATE: 1I.1 compiles + tests pass; BM25 (Phase 1B) exists; Vector search (Phase 1C) exists.
TASK: SearchMode enum: HYBRID, SPARSE_ONLY, BM25_ONLY. Cascade trigger: any layer failure drops to next mode.
Fallback chain per model:
- GraphCodeBERT fail -> TEXT-vector-only (Granite)
- Granite fail -> CODE-vector-only (GraphCodeBERT)
- Both fail -> SPARSE_ONLY (BM25 + SPLADE)
- SPLADE fail -> BM25_ONLY
- Cross-encoder fail -> skip reranking, use vector similarity scores
Mode restored automatically when failed component passes health check. Health check mechanism: circuit breaker HALF_OPEN probe (1I.1). Probe succeeds -> breaker CLOSED -> cascade re-evaluates available layers. Polling driven by half_open_timeout (30s), not separate timer.
Every mode change logged to SSB as action_type='knowledge_degradation'.
DETERMINISTIC_CHECK: search_mode + degradation tests pass; SearchMode 3 variants; knowledge_degradation in SSB logging.
ACCEPTANCE: kill embedding -> BM25 within 100ms of failure detection; auto-restores when health passes; SSB has event for every change; never total outage (BM25_ONLY always reachable, tantivy no ONNX dep).
GROUNDING: L117-120 HYBRID -> SPARSE_ONLY -> BM25_ONLY; L123 mode restored; L124 SSB knowledge_degradation; L145-149 fallback chain.

### 1I.3 QUERY VALIDATOR
PRE_GATE: knowledge compiles.
TASK: QueryValidator runs BEFORE any embedding call. Reject:
- Empty (zero tokens after whitespace split)
- > 10,000 chars
- null bytes or control characters
Validation latency: <1ms (no model invocation).
DETERMINISTIC_CHECK: query_valid tests pass; QueryValidator struct; 10_000 or MAX_QUERY constant.
ACCEPTANCE: empty rejected; 10,001 rejected; null byte rejected; control char rejected; valid accepted ("fn main()" Ok); newline accepted ("fn\nmain()" Ok); <1ms.
GROUNDING: L127 max 10,000; L128 reject null+control; L129 reject empty; L130 <1ms.

### 1I.4 STARTUP READINESS
PRE_GATE: 1I.1 + 1I.2 exist.
TASK: Startup readiness sequence:
1. Load ONNX models (GraphCodeBERT, Granite, cross-encoder, optionally SPLADE)
2. Open SQLite (DKG graph, vector cache, BM25 index)
3. Canary query: embed("fn main()") + BM25 search("main") + rerank
4. All return non-error within 5s
5. Canary embedding drift check: cosine similarity of embed("fn main()") vs reference vector at build time. If <0.85: weights corrupted/wrong version. Fail canary -> DEGRADED. Reference generated once during first successful startup, persisted in index_version.toml as canary_reference_vec. Evidence: embedding audit best practices — model loads but produces garbage silently corrupts retrieval.
6. Report ReadinessStatus to orchestrator.
ReadinessStatus enum: READY, DEGRADED, UNAVAILABLE.
- READY: all layers passed canary
- DEGRADED: ONNX failed but BM25 operational -> BM25_ONLY SearchMode
- UNAVAILABLE: BM25 also failed (SQLite corrupt, disk error) -> total outage
Invariant: DEGRADED implies SearchMode==BM25_ONLY (NOT HYBRID or SPARSE_ONLY).
If startup fails after 3 retries: DEGRADED (BM25_ONLY), log WARNING.
If BM25 also fails: UNAVAILABLE, log ERROR, orchestrator blocks all transitions.
Orchestrator gates FSM: IDLE -> PLANNING blocked until READY or DEGRADED. UNAVAILABLE blocks all FSM transitions.
DETERMINISTIC_CHECK: startup_readiness + canary_query tests pass; ReadinessStatus 3 variants; readiness_check fn exists.
ACCEPTANCE: healthy READY <5s; all ONNX fail -> DEGRADED after 3 retries BM25_ONLY active; BM25 also fails -> UNAVAILABLE; UNAVAILABLE blocks FSM (IDLE -> PLANNING rejected); canary timeout -> DEGRADED <5s; drift detection cos_sim <0.85 -> DEGRADED; first startup (no reference) skip drift, persist after success; DEGRADED implies BM25_ONLY; ReadinessStatus exactly 3.
GROUNDING: L159 canary; L160 within 5s; L164 3 retries -> BM25_ONLY; architecture-overview.txt L99 ReadinessStatus enum; L165 startup 5-15s; embedding audit best practices.

### 1I.5 CROSS-ENCODER SEMAPHORE
PRE_GATE: cross-encoder reranker compiles (1G); cargo test rerank passes.
TASK: tokio::Semaphore(1) guards cross-encoder ONNX session. One reranking at a time. Queued requests in bounded channel (capacity 4). Concurrency model: Semaphore(1) + bounded_channel(4) = max 5 in-flight (1 active + 4 queued). 6th overflows -> returns unranked immediately. Batching: multiple requests queued -> merge candidate sets, rerank once.
DETERMINISTIC_CHECK: semaphore + rerank_queue tests pass; Semaphore(1) usage.
ACCEPTANCE: serializes (2nd starts after 1st); 6th overflows (1 active + 4 queued + 6th unranked); overflow returns candidates not empty (result.len()==input.len()); batching merges (2 requests 10 candidates each -> 1 call 20 candidates); batching segregates results per caller.
GROUNDING: L219 Semaphore(1); L220 max queue 4 + overflow unranked; L221 merge + rerank once; L222 <50ms.

### 1I.6 QUERY TRACE
PRE_GATE: 1I.1-1I.5 compile; tracing crate in Cargo.toml.
TASK: QueryTrace + StageTrace per knowledge.txt L229-244.
QueryTrace fields: query_id (UUID), stages (Vec<StageTrace>), total_latency_ms, mode (SearchMode), result_count.
StageTrace fields: stage (String), latency_ms, input_count, output_count, error (Option<String>), circuit_breaker_state (Option<String>).
Stages (9): "bm25", "splade", "code_vector", "text_vector", "pagerank", "catrag", "arag", "rerank", "token_fit".
Emit via tracing at DEBUG.
Aggregated per-session statistics (p50/p99 per stage) stored in SSB at session end.
NOT stored in SSB per-query (too verbose).
DETERMINISTIC_CHECK: query_trace tests pass; QueryTrace + StageTrace structs; tracing::debug or tracing::event present.
ACCEPTANCE: every query() emits one QueryTrace (3 queries -> 3 trace events); HYBRID trace has bm25/code_vector/text_vector/rerank min; BM25_ONLY trace has mode==BM25_ONLY + only "bm25" stage; p50/p99 aggregation correct.
GROUNDING: L230-236 QueryTrace; L237-244 StageTrace; L246 DEBUG not SSB; L247 aggregated SSB session end; L238 9 stages.

### 1I.7 INDEX VERSION SCHEMA
PRE_GATE: knowledge compiles; .elai_cc/projects/ structure exists.
TASK: index_version.toml read/write at .elai_cc/projects/{id}/index_version.toml.
Schema: [index] version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format, created_at.
On upgrade: compare versions -> full rebuild OR BM25-only rebuild OR reuse.
Archive old index before rebuild to index_backup_{version}/.
User notified before rebuild (NOT silent).
DETERMINISTIC_CHECK: index_version tests pass; index_version schema handling; round-trip test.
ACCEPTANCE: round-trip (write/read/match); missing file creates default version=1 (no panic); version mismatch -> rebuild notification; archive created (index_backup_{version}/ exists).
GROUNDING: L195-201 schema; L203-207 full rebuild on schema change BM25 only on format change; L208 archived; L211 user notified.

VERIFY (post-1I.7):
- cargo check -p elai_knowledge compiles all new types
- cargo test --lib passes (no Phase 1G regression)
- Project isolation under degradation: query project_id=A in BM25_ONLY never returns project B results (1A/1B/1C own implementation; this verifies under degraded mode)
- Vector cache invalidation under degradation: file_write event invalidates cache even in SPARSE_ONLY/BM25_ONLY mode (no stale embeddings on recovery)
NOTE: project isolation invariants (knowledge.txt L274-279) owned by phases 1A (DKG), 1B (BM25), 1C (vectors). This phase verifies they hold under degradation.

MEASURE: degradation latency (failure injection -> SearchMode switch <100ms target); canary on healthy (<5s); circuit breaker recovery (OPEN -> HALF_OPEN -> CLOSED <35s typical); cross-encoder semaphore p99 queue wait under load; QueryTrace overhead <1ms per query; index version check <10ms.

EXIT: integration test on elai_cc's own codebase returns results; pipeline survives ONNX failure without total outage; circuit breaker transitions all 3 states correctly; cascade reaches BM25_ONLY + auto-recovers; startup gates FSM until READY/DEGRADED; cross-encoder semaphore prevents concurrent corruption; QueryTrace emitted at DEBUG; index version migration preserves backup.

PHASE 1 OVERALL MEASUREMENTS: index build time, query latency per layer, token savings, dual-vector vs single-vector accuracy, reranker precision gain, SPLADE recall delta, degradation cascade correctness, startup readiness latency.

================================================================================
## PHASE 1 INTERFACE MAP
================================================================================

### Phase 1A: DKG (tree-sitter -> SQLite graph)
Public Rust API surface (in .elai_cc/crates/knowledge/src/):
- trait KnowledgeEngine { fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph>; fn query(&self, query: &str, strategy: QueryStrategy) -> Result<Vec<CodeChunk>>; fn rerank(&self, query: &str, candidates: Vec<CodeChunk>) -> Result<Vec<CodeChunk>>; fn update(&mut self, file_changed: &Path) -> Result<()>; fn readiness_check(&self) -> ReadinessStatus; }
- enum ReadinessStatus { READY, DEGRADED, UNAVAILABLE }
- enum DkgReadiness { Ready, Empty, Unavailable }
- trait LanguageGrammar { fn language_id(&self) -> &'static str; fn tree_sitter_language(&self) -> tree_sitter::Language; fn extract_nodes(&self, tree: &Tree, source: &[u8]) -> Vec<DkgNode>; }  (object-safe, all &self)
- enum SymbolKind { Function, Struct, Impl, Trait, Enum, Module, Use, TypeAlias, Other }
- pub struct DkgNodeInfo { node_id: i64, symbol_name: String, file_path: PathBuf, signature: String, kind: SymbolKind, start_line: u32, end_line: u32 }
- pub struct DependencyGraph { node_count: usize, edge_count: usize, files_parsed: usize }
- pub struct DkgEngine + DkgEngine::new(db_path: &Path)
- DkgEngine methods: nodes() -> Result<Vec<DkgNodeInfo>>, edges() -> Result<Vec<(i64, i64, String)>>, node_by_id(id) -> Result<Option<DkgNodeInfo>>, outbound_edges(node_id) -> Result<Vec<(i64, String)>>, inbound_edges(node_id) -> Result<Vec<(i64, String)>>, failed_files() -> Result<Vec<String>>, dkg_readiness() -> DkgReadiness
- struct RustGrammar (impl LanguageGrammar)

SQLite tables created (in {project_root}/.elai_cc/projects/{project_id}/knowledge.db):
- dkg_nodes (id PK, file_path, name, kind, signature, start_line, end_line, parent_id REFERENCES dkg_nodes(id), concept, concept_stale, file_hash BLOB)
- dkg_edges (source_id REFERENCES dkg_nodes(id) ON DELETE CASCADE, target_id REFERENCES dkg_nodes(id) ON DELETE CASCADE, edge_type, PRIMARY KEY (source_id, target_id, edge_type))
- dkg_files (file_path PK, file_hash BLOB, parse_status DEFAULT 'pending', parse_error, last_indexed)
- dkg_meta (key PK, value)
Indexes: idx_nodes_name, idx_nodes_file, idx_nodes_kind, idx_files_status
Edge type strings: "imports", "contains", "implements"
PRAGMA journal_mode=WAL

Wire/file formats produced:
- index_version.toml at project-scoped path: [index] version=1, dkg_schema_version=1, vector_dimensions={code=0, text=0}, bm25_format="", created_at=<unix>
- File hashes: blake2s of file content stored as BLOB

Integration points (downstream consumers):
- Phase 1B BM25 consumes failed_files() to BM25-index parse-failed files
- Phase 1D PageRank consumes nodes() + outbound_edges()/inbound_edges() to build adjacency matrix; uses signature field for repo map output; uses SymbolKind for filtering
- Phase 1E CatRAG traverses directed edges via outbound_edges() for evidence chains; uses node names for symbolic anchoring
- Phase 1I uses dkg_readiness() in startup readiness aggregate

### Phase 1B: BM25 (tantivy full-text search)
Public Rust API surface:
- pub struct BM25Chunk { path: PathBuf, content: String, line_start: u32, line_end: u32, kind: ChunkKind }
- pub enum ChunkKind { Function, LineBlock, File }
- pub struct Bm25Index + Bm25Index::new(project_root: &Path, project_id: &str) -> Result<Self>
- pub struct BM25Hit { path: String, line_start: u32, line_end: u32, score: f32, chunk_kind: ChunkKind, snippet: String }
- pub struct IndexStats { files_indexed, chunks_indexed, files_skipped_binary, elapsed_ms }
- Bm25Index methods: index_file(path: &Path, chunks: Option<Vec<BM25Chunk>>) -> Result<usize>, index_directory(root: &Path, dkg: Option<&DkgIndex>) -> Result<IndexStats>, query(query_text: &str, limit: usize) -> Result<Vec<BM25Hit>>, delete_file(path: &Path) -> Result<usize>, update_file(path: &Path, chunks: Option<Vec<BM25Chunk>>) -> Result<usize>

SQLite tables created: NONE (tantivy uses directory-based segment files, not SQLite)

Wire/file formats produced:
- Tantivy directory at {project_root}/.elai_cc/projects/{project_id}/bm25/
- Tantivy schema: path STRING|STORED (untokenized — required for delete_term exact match), content TEXT|STORED (tokenized BM25 search), line_start u64 STORED, line_end u64 STORED, chunk_kind STRING|STORED
- INDEX_FORMAT_VERSION (u32) recorded in metrics

Integration points:
- Consumes Phase 1A failed_files() to ensure parse-failed files still BM25-indexed (zero orphan)
- Consumes Phase 1A DkgEngine for chunk derivation (if DKG has nodes for file)
- Phase 1C uses BM25 results in RRF fusion (3-ranker: BM25 + code_vec + text_vec)
- Phase 1E A-RAG keyword_search wraps Bm25Index::query
- Phase 1G cross-encoder reranker reranks BM25Hit results
- Phase 1H SPLADE RRF-fuses with BM25 results
- Phase 1I QueryTrace records "bm25" stage; SearchMode BM25_ONLY uses BM25 alone

### Phase 1C: VECTOR SEARCH (dual-model ONNX)
Public Rust API surface (in src/vector/):
- VectorModule trait (mod.rs)
- pub struct DualVector { code_vec: Vec<f32> (768d), text_vec: Vec<f32> (384d) }
- pub struct OrtSession (ort_session.rs) — Arc<Mutex<ort::Session>>; loads from .elai_cc/models/onnx/; embed(text: &str) -> Result<Vec<f32>, OrtError>; max_seq_len field (512 for GraphCodeBERT)
- pub struct VectorCache (cache.rs) — insert_dual, lookup_by_hash, invalidate_by_file
- pub struct CircuitBreaker (circuit_breaker.rs) — states CLOSED/OPEN/HALF_OPEN; record_failure/record_success/is_open/probe; uses ClockProvider
- VectorIndex trait + UsearchIndex impl (hnsw_index.rs) — wraps usearch::Index; add(key, vec), search(query, k), filtered_search(query, k, predicate), save(path), load(path); two instances code_index 768d, text_index 384d
- KnowledgeEngine::readiness_check() — loads models, canary embed("fn main()") within 5s, returns Ready or Degraded(mode); canary serves ONNX warmup

SQLite tables created:
- vector_cache (content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INTEGER)
- Migration M3 extends Phase 1B schema runner

Wire/file formats produced:
- usearch HNSW index files alongside SQLite DB (saved via Index::save, loaded via Index::load; if load fails rebuild from vector_cache)
- ONNX model files canonical at .elai_cc/models/onnx/{model_name}.onnx (pre-staged, not auto-downloaded)

Integration points:
- Consumes BM25 results (Phase 1B) for RRF fusion
- Provides QueryTrace stages "code_vector" and "text_vector" (Phase 1I)
- Provides circuit breaker per-model (Phase 1I §1I.1) for GraphCodeBERT and Granite
- SSB file_write event handler: DELETE FROM vector_cache WHERE file_path=? (registered at startup)
- SSB action_type='knowledge_degradation' on every SearchMode change (Phase 1I §1I.2)
- Cache invalidation under degradation verified by Phase 1I VERIFY step

### Phase 1D: PAGERANK REPO MAP
Public Rust API surface (in src/pagerank.rs):
- pub struct PageRankConfig { damping_factor: f64=0.85, max_iterations: usize=100, tolerance: f64=1e-6, density_threshold: f64=0.3, max_map_tokens: usize=2048 }
- pub struct RepoMap { ranked_symbols: Vec<RankedSymbol>, token_count: usize, density: f64, used_pagerank: bool, fit_iterations: usize }
- pub struct RankedSymbol { node_id: i64, symbol_name: String, file_path: PathBuf, rank: f64, signature: String }
- pub struct DkgSnapshot { nodes: Vec<DkgNodeInfo>, edges: Vec<(usize, usize, f64)> } (extends DkgNodeInfo with ref_count: usize for sqrt scaling)
- DkgSnapshot::from_engine(engine: &DkgEngine) -> Result<DkgSnapshot> (adds self-edges weight=0.1 for orphans)
- pub fn compute_repo_map(snapshot: &DkgSnapshot, task: &str, active_files: &[PathBuf], config: &PageRankConfig, token_estimator: &dyn Fn(&str)->usize) -> RepoMap
- Internal helpers: build_adjacency_matrix, build_personalization, build_dangling_vector, power_iteration, truncate_signature, is_special_file, binary_search_fit

SQLite tables created: NONE (in-memory adjacency matrix from DkgSnapshot)

Wire/file formats produced:
- RepoMap output structure (token-budget-fitted symbol list with ranks)

Integration points:
- Consumes Phase 1A DkgEngine via DkgSnapshot::from_engine (queries dkg_nodes + dkg_edges)
- Consumes providers/ token_count_estimate via token_estimator closure parameter
- Phase 1E CatRAG receives PageRank scores via compute_overlay(query, snapshot, pagerank_scores)
- Phase 1E pipeline Stage 3 invokes PageRank for importance filtering
- Special files (13 Rust): Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile, docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml

### Phase 1E: CatRAG + A-RAG (query pipeline assembly)
Public Rust API surface (in src/catrag.rs, src/arag.rs, src/pipeline.rs):
- pub struct CatRagConfig { damping=0.5, epsilon=0.2, beta=2.5, n_seed=5, k_edge=15, min_graph_density=0.01 }
- pub struct CatRagOverlay { reset_distribution: Vec<(usize, f64)>, edge_weight_deltas: Vec<(usize, usize, f64)> }
- pub struct CatRag with new(config) and compute_overlay(&self, query, snapshot, pagerank_scores) -> Result<CatRagOverlay>
- pub struct AragConfig { max_loops=15, max_token_budget=128000, top_k=5 }
- pub enum RetrievalTool { KeywordSearch{keywords, top_k}, SemanticSearch{query, top_k}, ChunkRead{chunk_ids} }
- pub struct RetrievalResult { chunk_id: String, snippet: String, score: f64, tool_used: String }
- pub struct AragOrchestrator with new(config), execute_tool(&self, tool, tracker), query_pipeline(&self, query, snapshot)
- pub struct ContextTracker { read_chunks: HashSet<String> } (per-query, NOT shared)
- pub struct QueryPipeline { bm25, catrag, arag, ... }
- pub struct PipelineStage { stage: String, results: Vec<RetrievalResult>, latency_ms: u64 }
- QueryPipeline::query(&self, q: &str) -> Result<Vec<PipelineStage>>
- phi() function with discrete tiers: 0-3->0.0, 4->0.2, 5->0.25, 6->0.3, 7->2.0, 8->2.5, 9->3.0, 10->5.0

SQLite tables created: NONE

Wire/file formats produced:
- PipelineStage trace (stage names: "bm25", "vector", "pagerank", "catrag", "arag", "rerank" stub, "token_fit" stub)
- RetrievalResult snippets (matched sentences only, +/- 2 lines context for code)
- workflow/papers/catrag-interface.md (paper interface ingested)
- workflow/papers/arag-interface.md (paper interface ingested)

Integration points:
- Consumes Phase 1B Bm25Index for keyword search
- Consumes Phase 1C vector index for semantic search
- Consumes Phase 1D PageRank (compute_overlay applies overlay to PPR — modifies PageRank's reset vector + transition matrix, re-runs PPR)
- RRF fusion (k=60) merges BM25 + vector results
- Phase 1G cross-encoder reranks A-RAG output (between A-RAG and token-budget fitting)
- Phase 3E token-budget fit consumes A-RAG snippets
- ContextTracker prevents redundant reads (snippet-based progressive disclosure)

### Phase 1F: SPECULATIVE DECODING EVALUATION (DROPPED — see Phase Outcome)
Public Rust API surface introduced (only 1F.1 deliverable):
- SpeculativeConfig added to providers (or nested OllamaConfig): speculative_decoding: bool (default false), draft_model: Option<String>, num_draft_tokens: u8 (default 5)
- OllamaProvider + LlmProvider trait abstraction enables future vLLM provider impl

SQLite tables created: NONE

Wire/file formats produced:
- workflow/reasoning/phase-1F-decisions.md (DROP decision documented)
- .elai_cc/metrics/phase_1F_*.toml (correctness, throughput, vram, prompt_lookup) — most empty due to DROP
- Probe infrastructure for ollama API availability (1F.2 deliverable)

Integration points (after DROP):
- No runtime integration — config defaults remain disabled
- Speculative decoding REVISITED in Phase 2+ when vLLM provider implemented
- vLLM natively supports speculative decoding, EAGLE-3, layer-level intervention (ollama is "black box")

### Phase 1G: CROSS-ENCODER RERANKER
Public Rust API surface (in knowledge crate):
- pub struct CrossEncoderReranker { session: ort::Session, tokenizer: tokenizers::Tokenizer, semaphore: tokio::Semaphore, request_channel: mpsc::Sender, circuit_breaker: CircuitBreaker, clock: Arc<dyn ClockProvider> }
- pub struct RerankerConfig { model_path, tokenizer_path, fallback_model_path: Option<PathBuf>, quantized_model_path: Option<PathBuf>, skip_threshold: usize=10, max_sequence_length: usize=512, max_queue_depth: usize=4, intra_op_threads: usize=1, graph_optimization_level: GraphOptimizationLevel=ORT_ENABLE_ALL, enable_deterministic_compute: bool=true }
- KnowledgeEngine::rerank trait method implementation (matches exact trait signature recorded in 1G.1 PRE_GATE)
- Wired into pipeline between A-RAG output and token-budget fitting

SQLite tables created: NONE

Wire/file formats produced:
- ONNX model files: cross-encoder/ms-marco-MiniLM-L-12-v2 (~120MB fp32) at configured path
- Optional INT8 quantized variant (~30MB): generated by scripts/quantize_cross_encoder.py
- Optional L-6 fallback model (cross-encoder/ms-marco-MiniLM-L-6-v2, 22.7M params)
- tokenizer.json (WordPiece, shared between L-12 and L-6 — same BERT vocab base)
- tests/fixtures/reranker_benchmark.json (1 query, 50 candidates, binary relevance, >=10 relevant in top 50, real code chunks)
- workflow/reasoning/phase-1G-spec-amendment.md (L-6 tier amendment + final latency if >50ms)
- .elai_cc/metrics/phase_1G.toml (baseline_latency_p50, thread_sweep, int8_latency, int8_ndcg_delta, threshold_sweep, final_latency_p50, final_latency_p99, final_ndcg_at_10, model_variant)

Integration points:
- Consumes A-RAG output (Phase 1E) as candidates Vec
- SPARSE_ONLY/BM25_ONLY skip reranker entirely (knowledge.txt L118-119)
- SSB action_type='knowledge_degradation' with stage='rerank' (L124, separate from tracing)
- Phase 1I §1I.4 startup canary includes rerank in 5s window
- Phase 1I §1I.5 wraps with tokio::Semaphore(1) + bounded mpsc(4) + batching
- Phase 1I §1I.6 emits StageTrace for "rerank" in QueryTrace at DEBUG
- Phase 1I §1I.1 cross-encoder gets independent circuit breaker (TRANSIENT vs PERMANENT classification, exponential backoff)
- ClockProvider injected for all time-dependent logic (entropy quarantine)
- L-6 lazy-loaded on first L-12 breaker open (cold-start 5-15s acceptable, no SLA on fallback)

### Phase 1H: SPLADE KEYWORD EXPANSION (CONDITIONAL — likely DROP per vocab fitness)
Public Rust API surface (feature-gated cfg(feature="splade")):
- pub struct SpladeExpander
- SpladeExpander::new(model_path) -> loads ONNX via ort::Session + BERT tokenizer
- SpladeExpander::expand_query(&self, query: &str) -> Vec<(String, f32)> (sparse vector top-k token+weight pairs)
- SpladeExpander::vocab_coverage(corpus_tokens: &[String]) -> f64 (0.0-1.0 fraction of unique code tokens mapping to single WordPiece)
- model_variant enum (PrePooled, RawMLM) for dispatch (PP_en_v1 has pooling baked in; splade-v3 raw MLM logits, must apply max-pool + ReLU + log1p in Rust)
- rrf_fuse(bm25_results, splade_results, k=60, weight_bm25=0.5, weight_splade=0.5) -> merged ranking (k clamped to max(k,1))

SQLite tables created: NONE

Wire/file formats produced (if KEEP):
- ONNX model file at .elai_cc/models/splade/model.onnx (default; SPLADE_MODEL_PATH override)
- Candidate A: prithivida/Splade_PP_en_v1 (532MB full or 436MB int8)
- Candidate B: naver/splade-v3 (export via sentence-transformers optimum)
- Shared tokenizer.json (BERT WordPiece, 30522 tokens)
- workflow/reasoning/phase-1H-measurements.md (vocab_coverage_code, model_comparison table, recall_delta_percent, wilcoxon_p_value, splade_query_latency_ms, model_load_time_s, binary_size_delta_mb)

Integration points (only if KEEP):
- RRF fusion with BM25 (Phase 1B) — weighted RRF with k=60
- Circuit breaker integration (Phase 1I §1I.1)
- SearchMode SPARSE_ONLY uses BM25 + SPLADE; SPLADE fail -> BM25_ONLY (Phase 1I §1I.2)
- QueryTrace stage "splade" (Phase 1I §1I.6)
If DROP: feature flag remains but disabled by default; module preserved for future re-evaluation. SPARSE_ONLY cascade degrades to BM25_ONLY.

### Phase 1I: RESILIENCE LAYER
Public Rust API surface:
- pub struct EmbeddingCircuitBreaker (states CLOSED/OPEN/HALF_OPEN; per ONNX model: GraphCodeBERT, Granite, cross-encoder, SPLADE; failure_threshold=3 transient, failure_window=60s, half_open_timeout_base=30s; exponential backoff 30s/60s/120s with deterministic jitter from hash of model_id; TRANSIENT vs PERMANENT failure classification via error type enum; HALF_OPEN requires 2 consecutive successes; fresh ONNX InferenceSession on probe)
- pub enum SearchMode { HYBRID, SPARSE_ONLY, BM25_ONLY }
- pub struct QueryValidator (rejects empty, >10000 chars, null bytes, control chars; <1ms)
- pub enum ReadinessStatus { READY, DEGRADED, UNAVAILABLE } (DEGRADED implies SearchMode==BM25_ONLY)
- KnowledgeEngine::readiness_check() (canary embed("fn main()") + BM25 search("main") + rerank within 5s; embedding drift check cosine_sim against canary_reference_vec, <0.85 -> DEGRADED)
- pub struct QueryTrace { query_id: UUID, stages: Vec<StageTrace>, total_latency_ms, mode: SearchMode, result_count }
- pub struct StageTrace { stage: String, latency_ms, input_count, output_count, error: Option<String>, circuit_breaker_state: Option<String> }
- 9 trace stages: "bm25", "splade", "code_vector", "text_vector", "pagerank", "catrag", "arag", "rerank", "token_fit"
- index_version.toml read/write
- Cross-encoder concurrency: tokio::Semaphore(1) + bounded_channel(4) = max 5 in-flight (1 active + 4 queued); 6th overflows -> unranked

SQLite tables created: NONE in this phase (uses tables from prior phases). SSB events written via existing SSB write mechanism. Aggregated per-session statistics (p50/p99 per stage) stored in SSB at session end.

Wire/file formats produced:
- index_version.toml at .elai_cc/projects/{id}/index_version.toml: [index] version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format, created_at, canary_reference_vec
- Archived old indexes: index_backup_{version}/ before rebuild (user notified, NOT silent)
- SSB action_type='knowledge_degradation' on every SearchMode change
- tracing DEBUG events with QueryTrace per query

Integration points:
- Aggregates dkg_readiness() (Phase 1A) into KnowledgeEngine::readiness_check()
- Wraps cross-encoder (Phase 1G) with concurrency control
- Manages circuit breakers for all ONNX models (GraphCodeBERT, Granite, cross-encoder, SPLADE if present)
- Orchestrator gates FSM: IDLE -> PLANNING blocked until READY or DEGRADED; UNAVAILABLE blocks ALL FSM transitions
- Cache invalidation under degradation: file_write event invalidates Phase 1C vector cache even in SPARSE_ONLY/BM25_ONLY (no stale embeddings on recovery)
- Project isolation under degradation verified (knowledge.txt L274-279 invariants owned by 1A/1B/1C, this phase verifies hold under degraded SearchMode)

================================================================================
END OF PHASE 1 LOSSLESS SUMMARY
================================================================================

DEFERRED / TODO / STUB MARKERS across Phase 1:
- Phase 1A: tree-sitter Python (0.25), Go (0.25), TypeScript (0.23) added in future phases (Rust only initially)
- Phase 1A: knowledge.txt Sec 8 items 2-4 (BM25 indexing parse-failed files, vector embedding parse-failed, PageRank treats parse-failed as leaf) DEFERRED to Phases 1B-1D
- Phase 1A: <15s build target UNGROUNDED, 1A.5 establishes baseline
- Phase 1B: 10-line overlap UNGROUNDED for BM25 specifically (carried over from vector layer)
- Phase 1B: SnippetGenerator with keyword highlighting deferred (simple 200-char truncation in Phase 1B)
- Phase 1C: SPLADE DEFERRED to Phase 1G/1H measurement cycle
- Phase 1C: TurboQuant quantization DEFERRED to Phase 1G or dedicated phase (paper INGESTED, raw f32 stored in 1C)
- Phase 1C: GraphCodeBERT/Granite ONNX tensor format UNGROUNDED until Step 3.5 verification
- Phase 1D: damping=0.85, max_iter=100, tolerance=1e-6 UNGROUNDED (Brin/Page 1998 defaults, overridable)
- Phase 1D: special files list 13 Rust items (knowledge.txt L75 says ~100, config-driven extensibility deferred)
- Phase 1E: Full ReAct agent loop (LLM decides which tool) DEFERRED to Phase 2+ (requires reasoning crate); Phase 1E implements deterministic pipeline
- Phase 1E: PipelineStage stubs for "rerank" (Phase 1G) and "token_fit" (Phase 3E)
- Phase 1F: ALL spec decoding implementation DEFERRED to Phase 2+ when vLLM provider implemented (DROP decision)
- Phase 1F: Prompt Lookup Decoding (1F.3b) ALSO SKIPPED as part of DROP
- Phase 1G: L-6 intermediate tier requires SPEC AMENDMENT approval (workflow/reasoning/phase-1G-spec-amendment.md); if rejected, only "skip reranking" final fallback (knowledge.txt L149)
- Phase 1G: INT8 quantization conditional (only if baseline p50 >30ms)
- Phase 1H: SPLADE module is feature-gated cfg(feature="splade"), disabled by default, preserved for future re-evaluation if DROP
- Phase 1H: 0.3 vocab coverage threshold UNGROUNDED conservative heuristic, should be revisited with production data
- Phase 1I: implementation-guide.txt L141 dep graph needs update (1G -> 1I edge missing) — ACTION REQUIRED before implementation
- Phase 1I: KnowledgeEngine.readiness_check() FULL ENGINE aggregate built in Phase 1I (DkgEngine has separate dkg_readiness() in Phase 1A)

INTERNAL CONTRADICTIONS / STALE REFERENCES IN SOURCE SPECS:
1. knowledge.txt L16 said "FIVE LAYERS" — corrected to "5 major + 2 sub-layers" (architecture-overview.txt L100 says 7 counting sub-layers 3.25 and 3.5).
2. base-fork-comparison.txt LOC numbers stale (green_contract.rs listed as 4,061, actual 152) — R21.
3. knowledge.txt L200 says bm25_format = "tantivy_v21" but this is project-level label, NOT tantivy internal API (INDEX_FORMAT_VERSION u32). Phase 1B records actual tantivy crate version + INDEX_FORMAT_VERSION instead.
4. providers.txt L46-47 had "1.8-2.3x" and "~10GB headroom" — CORRECTED to 2x-6.5x and ~4.8GB headroom. external-analysis.txt L312, L456 also corrected to 2x-6.5x.
5. implementation-plan.txt L184 still has stale "1.8-2.3x" (INFO not blocking).
6. R20 ID collision: risk-register.txt:L88 = "PageRank density degradation" vs phase-0.txt:L22 = "entropy quarantine scope" — cross-file fix needed (out of scope for Phase 1D).
7. Phase 1D originally referenced "Section 2 Layer 4" — knowledge.txt has PageRank at Layer 3.5 (L64). Layer 4 is CatRAG (L78). FIXED in phase-1D.txt.
8. knowledge.txt L58 typo "prithvida" — correct "prithivida". File correction pending (BLOCKED owned by s-702732 at time of writing).
9. Phase 1B SPEC_GAP: "Section 4 (KnowledgeEngine trait)" in architecture-overview.txt — VERIFIED Section 4 exists at line ~80, KnowledgeEngine at L94. Earlier finding incorrect.
10. Phase 1B: "Chunk struct from Phase 1A" — CORRECTED: Phase 1A exports DkgNode (no content), not Chunk. BM25Chunk defined as BM25-owned with content field; conversion is BM25's responsibility.
11. Phase 1B: "Crate path .elai_cc/rust/crates/knowledge/" — CORRECTED to .elai_cc/crates/knowledge/ (phase-1A.txt L3).
12. Phase 1F PRE_GATE originally said "Phase 0 complete (independent of 1A-1E)" — CORRECTED to "Phase 1C complete" per implementation-guide.txt L141 dep graph.
13. Phase 1I dep graph incomplete: needs 1G -> 1I edge added to implementation-guide.txt L141 (ACTION REQUIRED).
14. Phase 1A: Auditor caught original LanguageGrammar design had associated functions without &self — FIXED (object safety required).
15. Phase 1G: spec L149 defines only "skip reranking" as fallback — L-6 intermediate tier is SPEC AMENDMENT awaiting approval.

SUB-STEPS LACKING DETERMINISTIC_CHECK OR ACCEPTANCE BLOCKS:
After scanning all 11 source files, every numbered sub-step DOES include both DETERMINISTIC_CHECK and ACCEPTANCE blocks. The following sub-steps deserve special note:
- Phase 1E.1 (SKELETON): has DETERMINISTIC_CHECK + ACCEPTANCE (single ACCEPTANCE bullet "knowledge crate compiles ...").
- Phase 1E.2 (VERIFY): has DETERMINISTIC_CHECK + ACCEPTANCE; lacks ADVERSARIAL block (only this verify step skips it — others have it).
- Phase 1E.7 (MEASURE): has DETERMINISTIC_CHECK + ACCEPTANCE.
- Phase 1H sub-steps (1, 2, 3, 4): use checkbox list "[ ]" format for ACCEPTANCE rather than bullet list — content equivalent.
- Phase 1I sub-steps (1I.1-1I.7): ALL include PRE_GATE + TASK + DETERMINISTIC_CHECK + ADVERSARIAL + ACCEPTANCE + GROUNDING blocks (most complete structure).
- Phase 1F sub-steps 1F.3-1F.5: marked SKIPPED due to DROP decision but ACCEPTANCE blocks remain in source for completeness.

No sub-step is missing both DETERMINISTIC_CHECK AND ACCEPTANCE. Source spec coverage is complete.
