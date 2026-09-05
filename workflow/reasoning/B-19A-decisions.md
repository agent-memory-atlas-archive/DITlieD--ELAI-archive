B-19A DECISIONS — Stop hook + ruff PostToolUse + hook latency budgets
=====================================================================

Phase: B-19 Hook System Unification (subset A — testable subset).
Session: s-28972.
FSM journey: TESTING_FIRST -> EXECUTING.

DECISION-A — Task 5 PreCompact wire = DEFERRED
----------------------------------------------
Status: deferred to B-19B/C, NOT implemented in this cycle.
Rationale (verified by context-compiler at session start):
- task_runner.rs:4379 (PreCompact fire site) holds only writer/seq/RUNNER_AGENT_ID.
- It does NOT hold TrimmedMessage history, SsbReader, EntropySource, ClockProvider,
  or a SessionDag handle (those live in LiveCli, not in TaskRunnerResolved).
- The "fires when token_used > 0.85" trigger is not measured anywhere at the
  conversation level (`SESSION_HANDOFF_THRESHOLD` const not present in tree;
  `context_budget.rs` measures per-baby-step slot allocations, not token_used).
- A correct wire requires a substantial TaskRunnerResolved refactor that is
  out of scope for this testable-subset phase.
The PreCompact stub stays a no-op for now; SESSION_HANDOFF_THRESHOLD const
from B-9 is the future trigger source. Leaving the dispatch path intact
preserves the existing trace! breadcrumb the dashboard already filters on.

DECISION-B — Task 4 Stop hook double-Ctrl+C-within-3s gate = IMPLEMENTED
-----------------------------------------------------------------------
What changed:
- New `StopHookGate` in `crates/orchestrator/src/hooks.rs` carries an
  `AtomicBool` (aborted) + `AtomicI64` (first-press timestamp ms).
- `observe_signal(fsm_state, now_ms) -> StopHookDecision` is the canonical
  decision function — purely synchronous, thread-safe, zero-allocation per
  signal. No tokio types in the hot path.
- `STOP_HOOK_DOUBLE_PRESS_WINDOW_MS = 3_000` constant.
- HookAbortMonitor::spawn (cli/src/main.rs:3528) now takes `(abort_signal,
  fsm: Arc<Fsm>, ssb: Option<Arc<Ssb>>, clock: Arc<dyn ClockProvider>)` and
  loops on Ctrl+C. Each press: read FSM state via `fsm.get_state().await`,
  feed `(state, now_ms)` to the gate, dispatch on result.
- LiveCli call site at cli/main.rs:4368 passes `fsm.clone()`, `None` for ssb
  (intentional — see "Constraint" below), and `clock.clone()`.

Constraint: SSB intervention_request emission is currently surfaced via
`tracing::warn` instead of an SSB row. LiveCli stores `Option<Ssb>` (owned),
not `Option<Arc<Ssb>>`. Promoting it to an Arc requires touching every
build_runtime call site — out of scope for this cycle. The HookAbortMonitor
SIGNATURE accepts an `Option<Arc<Ssb>>`, so when LiveCli is later refactored
to hold an Arc the wire is one-line.

Tests: 6 cases in `crates/orchestrator/tests/stop_hook_double_ctrl_c.rs`:
- IDLE -> abort first press
- EXECUTING -> warn -> abort within 3s
- TESTING_FIRST -> warn -> abort
- AUDITING -> warn -> abort
- 5s gap resets cycle
- Aborted gate stays aborted (idempotent).
All 6 pass.

DECISION-C — Task 6 ruff PostToolUse subprocess = IMPLEMENTED
-------------------------------------------------------------
What changed:
- New module `crates/plugins/src/post_tool_use.rs` (~430 lines).
- `RuffPostToolUseRunner::new()` probes for `ruff` on PATH at construction
  via `std::process::Command::new("ruff").arg("--version")`.
  - Cross-platform: on Windows, tries `ruff.exe` first then `ruff` (falls
    through `ErrorKind::NotFound`).
- `run(file_path, content) -> Result<RuffOutcome, RuffRunnerError>` async:
  - skip-with-NoOp on empty path / non-Python file / >10000 lines.
  - blake3(content) cache lookup; cache hit returns cached outcome.
  - subprocess spawn via `tokio::process::Command` with 5s timeout
    (`tokio::time::timeout`); timeout returns `RuffOutcome::Timeout`.
  - parse stdout/stderr text format (path:line:col: CODE message) into
    `Vec<RuffFinding>`.
- Cache: `Mutex<BTreeMap<[u8;32], CacheEntry>>` capped at 256 entries with
  FIFO eviction by insertion_seq. NO new dependencies (lru crate not in
  tree per Cargo.toml inspection; BTreeMap satisfies "no std HashMap").
- `warn_once: std::sync::Once` ensures BinaryMissing warning fires exactly
  once per process (mirrored to AtomicBool for tests).

Tests: 4 cases in `crates/plugins/tests/ruff_post_tool_use.rs`:
- missing binary returns NoOp + warns once (3 pass)
- timeout constant = 5_000ms
- present binary runs check + caches (#[ignore], runs with --ignored when
  ruff is installed in the test env)
- cache size cap evicts oldest (256 cap honored)
3 pass + 1 ignored.

DECISION-D — Task 10 T1/T2/T3 hook latency budgets = IMPLEMENTED
----------------------------------------------------------------
What changed:
- 3 new public consts at `crates/orchestrator/src/hook_dispatcher.rs`:
  - `HOOK_T1_BUDGET_MS = 1`
  - `HOOK_T2_BUDGET_MS = 10`
  - `HOOK_T3_BUDGET_MS = 5`
  Plus `hook_tier_budget_ms(tier) -> u64` lookup helper.
- Two new SSB emission helpers:
  - `emit_hook_latency_violation_to_ssb(...)` writes a single
    `hook_latency_violation` row with payload {hook_name, tier, budget_ms,
    actual_ms}. Mirrors the `emit_hook_events_to_ssb` pattern.
  - `report_hook_latency(writer, agent_id, superstep, hook_name, tier,
    actual_ms)` is the single SSB-aware overrun gate: returns false when
    within budget, true when over. Logs warn and writes the row when over.
- Dispatcher instrumentation: `InProcessHookDispatcher::analyze` now times
  the tiered analysis with `tokio::time::Instant`, emits warn on overrun
  for both T1 (using TieredAnalysisResult.duration_ms) and Composite (the
  whole pass elapsed). The SSB-side emit lives on the free function
  `report_hook_latency` so the task_runner (which holds the Ssb writer)
  can wire it without changing the dispatcher trait signature.

SSB schema bump 22 -> 23 — coordinated 14-site edit:
- ssb.rs:28           SCHEMA_VERSION 22 -> 23 + comment.
- ssb.rs:74           CHECK constraint adds 'hook_latency_violation'.
- ssb.rs:100          variant-count comment 74 -> 75.
- ssb.rs:312          enum SsbActionType::HookLatencyViolation variant.
- ssb.rs:394          as_sql_str arm.
- ssb.rs:483          ssb_action_type_from_str arm.
- ssb.rs:773          HookLatencyViolationPayload struct.
- ssb.rs:1614         doc comment 76 -> 77 total variants.
- ssb.rs:1712         SsbPayload::HookLatencyViolation variant.
- ssb.rs:1796         action_type() match arm.
- ssb.rs:3082         test_payload arm for HookLatencyViolation.
- ssb.rs:3479         test_all_action_types_insert types vec extension.
- ssb.rs:3501         test count assertion 35 -> 36.
- ssb.rs:4434         arb_payload prop_oneof! arm (proptest round-trip).
Migration block at ssb.rs:2218 stays as-is (DROP TABLE on schema mismatch
is the correct path for session-scoped SSB, no data preserved across
schema bumps — same precedent as B-10 21->22).

OUT OF SCOPE — explicitly deferred
-----------------------------------
- Tasks 1-3 (dispatcher dedup): blocked on B-18B IPC accept-loop, both
  InProcess + IPC dispatchers stay live this phase per scope override.
- Tasks 7-9 (DAIRA framework): substantial work; deferred to B-19B.
- Task 11 (auditor independent rules approval): deferred.
- Frontend Svelte/TS files: dashboard abandoned per user direction.

VERIFICATION
------------
cargo check -p elai-orchestrator -p elai-plugins -p elai-cli: PASS (2 pre-existing warnings unchanged).
cargo test -p elai-orchestrator --test stop_hook_double_ctrl_c: 6/6 pass.
cargo test -p elai-orchestrator --test hook_latency_budgets: 4/4 pass.
cargo test -p elai-plugins --test ruff_post_tool_use: 3/4 pass + 1 ignored (ruff binary missing on test env).
cargo test -p elai-orchestrator --lib ssb: 39/39 pass (SCHEMA_VERSION migration + proptest round-trip on new variant).

Deterministic checks:
- HOOK_T*_BUDGET_MS hits across crates: 19 (>=3).
- hook_latency_violation/HookLatencyViolation hits in ssb.rs: 16 (>=4).
- ruff hits in post_tool_use.rs: 43 (>=1).
- 3s window comments in cli/main.rs: 4 (>=1).

=====================================================================
CYCLE 2 — auditor BLOCK 1/2/3 + WARN-2 fixes (s-28972)
=====================================================================

CYCLE-2 DECISION-A — BLOCK-1: ruff invocation now feeds content via stdin
------------------------------------------------------------------------
Problem (auditor cycle 1): post_tool_use.rs run() invoked
`ruff check <file_path>` with `Stdio::null()` stdin. The cache key was
blake3(content) but the linted source was whatever happened to be on
disk — racy with the writer, missing-file produced E902 caches as valid
Findings.

Fix:
- Build the command with `--stdin-filename <file_path> -` so ruff treats
  stdin as the source while still reporting findings under the real path.
- Set `Stdio::piped()` for stdin, take `child.stdin`, write content via
  `tokio::io::AsyncWriteExt::write_all`, then `shutdown()` to deliver EOF.
- Switch from `cmd.output()` to `child.wait_with_output()` (the
  spawn-and-write split requires the explicit child handle).
- All write/shutdown/wait failures map to `RuffRunnerError::Spawn` to
  preserve the existing error-shape contract.
- Doc comment block at L226-229 rewritten to describe the stdin-fed
  invocation. Verified by:
    grep -n "stdin-filename" crates/plugins/src/post_tool_use.rs (3 hits)
    grep -n "Stdio::null" crates/plugins/src/post_tool_use.rs (0 hits)

Test impact: existing `ruff_post_tool_use.rs` fixtures still hold — the
fixtures use empty path / non-Python path / cache test seam, none invoke
the subprocess path. Added `with_clock_constructor_accepts_seeded_clock`
to verify ClockProvider injection (BLOCK-3).

CYCLE-2 DECISION-B — BLOCK-3: ClockProvider injection for production timing
---------------------------------------------------------------------------
Problem: `tokio::time::Instant::now()` at hook_dispatcher.rs:480 and
post_tool_use.rs:230 violated the entropy quarantine — production timing
must flow through `ClockProvider`.

Fix (applied before BLOCK-2 so timing replacement is consistent):
- `RuffPostToolUseRunner` gains a `clock: Arc<dyn ClockProvider>` field.
  Constructors:
    * `new()` defaults to `OsClock` (production).
    * `with_clock(clock)` for test injection (SeededClock).
  All `started.elapsed()` sites replaced with
  `self.clock.elapsed_since(started)`. Hand-rolled `Debug` impl because
  `Arc<dyn ClockProvider>` is not Debug.
- `InProcessHookDispatcher` gains a `clock` field. Default constructor
  installs `OsClock` so existing call sites in dashboard/dashboard-v2
  /task_runner/contract tests stay backward-compatible. New `with_clock`
  builder for tests.
- `analyze()` body now reads `self.clock.now()` and
  `self.clock.elapsed_since(started)`. Removes both
  `tokio::time::Instant::now()` call sites. Verified:
    grep -n "tokio::time::Instant::now" crates/plugins/src/post_tool_use.rs (only doc comment mentions, 0 actual calls)
    grep -n "tokio::time::Instant::now" crates/orchestrator/src/hook_dispatcher.rs (only doc comment, 0 actual calls)
- `tokio::time::timeout` retained intentionally — that's the runtime's
  scheduling primitive, not entropy. Comment in the runner explains.

CYCLE-2 DECISION-C — BLOCK-2: T3 budget = partial fix (per spec option 2)
-------------------------------------------------------------------------
Problem: only T1+T2 were checked; the "T1" check actually fired on the
COMPOSITE timing returned by `TieredAnalysis::analyze` (which includes
T1+T2+T3). Misleading log label.

Constraint: `TieredAnalysis::analyze` returns ONE `duration_ms: f64`
covering the whole pass. Splitting per tier requires a cross-crate
refactor of `elai_plugins::tiered_analysis` (changes the public
`TieredAnalysisResult` struct used by 5 contract tests). Spec
authorised the partial-fix path (option 2 documentation) when the
cross-crate refactor is too disruptive; we take that path here.

Fix:
- Removed the misleading `t1_elapsed_ms` derivation and the
  "hook_dispatcher: T1 exceeded latency budget" log line.
- Single composite-budget warning fires when
  `composite_elapsed_ms > HOOK_T2_BUDGET_MS` (10ms — matches
  `hook_tier_budget_ms(HookTier::Composite)`). Message explicitly cites
  the deferred per-tier enforcement: "(per-tier enforcement deferred to
  B-19B; TieredAnalysis returns composite timing only)".
- Added test `per_tier_overruns_emit_independent_rows` to
  hook_latency_budgets.rs proving the SSB layer already supports
  per-tier rows independently — when the cross-crate refactor lands,
  no SSB-side change is required.

Deferred to B-19B: Make `TieredAnalysisResult` carry
`(t1_ms, t2_ms, t3_ms)` and switch the dispatcher to per-tier budget
enforcement. Update hook_latency_budgets.rs to assert dispatcher-side
violation per tier instead of just SSB-helper-side.

CYCLE-2 DECISION-D — WARN-2: RuffPostToolUseRunner wired into dispatcher
-----------------------------------------------------------------------
Problem: `RuffPostToolUseRunner` existed but was never invoked from
`InProcessHookDispatcher::dispatch_post_tool_use`. The spec
observable_effect ("agent runs Python file write -> ruff_check_passed
within 5s") was unreachable.

Fix:
- `InProcessHookDispatcher` gains an optional `ruff:
  Option<Arc<RuffPostToolUseRunner>>` field + `with_ruff_runner` builder
  + `has_ruff_runner` accessor (mirrors existing shortcoder pattern).
- `dispatch_post_tool_use` now: (1) runs the tiered pass first so
  T1/T2/T3 BLOCK still aborts; (2) if a ruff runner is installed and
  the dispatcher is not env-disabled, calls
  `run_ruff_post_tool_use(&invocation)`; (3) appends ruff findings to
  the returned `Vec<HookFinding>`.
- New helper `run_ruff_post_tool_use` short-circuits on missing source,
  missing path, or non-`.py`/`.pyi` extension before invoking the
  runner.
- New free functions `ruff_outcome_to_findings` +
  `ruff_finding_to_hook_finding` map `RuffOutcome` -> `HookFinding`:
    * NoOp(BinaryMissing) -> single Composite Warn (so dashboard can
      tell user to install ruff).
    * NoOp(other reasons) -> empty (expected skips stay silent).
    * Timeout -> Composite Warn with category "ruff_timeout".
    * Findings(empty, exit 0) -> empty (clean run).
    * Findings(non-empty) -> one Composite Warn per ruff diagnostic;
      `category = "ruff_<rule_code>"`, `rule_id = rule_code`.

Deferred WARNs (per spec "DO NOT touch" list — all logged here for B-19B)
------------------------------------------------------------------------
- WARN-1: detect_ruff_binary swallows non-NotFound errors silently. Fix
  in B-19B by distinguishing PermissionDenied / unknown io errors and
  surfacing them as a separate `RuffSkipReason::DetectionError`.
- WARN-3: T1 budget constant duplicated in ipc_hooks.rs:34. Fix in
  B-19B by importing the canonical const from `hook_dispatcher`.
- WARN-4: BTreeMap O(n) eviction in `RuffPostToolUseRunner::insert_cache`
  (linear scan to find min insertion_seq). Fix in B-19B by adding a
  VecDeque<[u8;32]> insertion-order index for O(1) eviction or vendoring
  `lru` (needs dep approval).
- WARN-5: StopHookGate has no contract test in `crates/contracts/tests/`.
  Fix in B-19B by adding `stop_hook_gate_contract.rs` covering all
  state x signal x window combinations.
- WARN-6: One `InterventionType::ApproveChange` site uses the wrong
  variant name (per cycle 1 audit). Fix in B-19B once the IPC schema
  is reviewed for variant usage consistency.
- INFO-4: Windows path colon parse in `parse_ruff_output` (e.g.
  `C:\foo\bar.py:5:1: ...` would split incorrectly on the drive letter
  colon). Currently masked because we pass `--stdin-filename` with the
  caller-provided path; fix in B-19B by switching to JSON output once
  ruff version pin is bumped.

CYCLE-2 VERIFICATION
--------------------
cargo check -p elai-orchestrator -p elai-plugins -p elai-cli: PASS.
cargo check --workspace: PASS (2 pre-existing warnings unchanged; no new warnings).
cargo test -p elai-plugins --test ruff_post_tool_use: 4 passed, 1 ignored.
cargo test -p elai-orchestrator --test hook_latency_budgets: 5 passed.
cargo test -p elai-orchestrator --test stop_hook_double_ctrl_c: 6 passed.
cargo test -p elai_cc_contracts --test hook_dispatcher_contract --test shortcoder_post_tool_use_invocation_contract: 15 passed (10 + 5).
cargo test -p elai_cc_contracts --test tiered_analysis_contract -- --test-threads=1: 18 passed.
  (Parallel run shows pre-existing env-var race in
   contract_env_disable_shortcoder_returns_skipped — unrelated to this
   cycle; same flake as cycle 1 baseline. Single-threaded run is the
   established workaround for env-mutating tests in this binary.)

Cycle-2 deterministic checks:
- `ruff check --stdin-filename` invocation present + content piped to
  child stdin: VERIFIED (post_tool_use.rs L279-307).
- `tokio::time::Instant::now()` removed from hook_dispatcher.rs:480 and
  post_tool_use.rs:230: VERIFIED (grep returns only doc comments).
- T3 budget: partial fix documented; one composite-budget check, no
  false per-tier labels: VERIFIED (hook_dispatcher.rs analyze() body).
- RuffPostToolUseRunner invoked from dispatch_post_tool_use when
  installed: VERIFIED (hook_dispatcher.rs dispatch_post_tool_use body).
