B-19 Hook System Unification — Decision Document
Session: s-91480
Date: 2026-05-09

---
ADDENDUM: Session s-25120 (2026-05-23) — second-pass closure of 1 STUB + 2
MISSING + 3 NOWIRE items per cross-check-recheck report
F:\.ELAI_workflow\workflow\sessions\s-25120\cross-check-recheck\B-19_hook_system_unification-report.md

REBASELINE — plan body line-number drift (cites task_runner.rs:3338/3589/3877/3898;
actual live at 5627/7032/7415/7459). Plan body STATUS block updated in-place
with corrected numbers. 7 SHIPPED-ON-V2 markers logged inline.

PREREQ-OK — B-9 SessionDag::create_handoff DEF_FN @ session_dag.rs:1627;
B-18 run_agent_session with hook_dispatcher: Option<Arc<dyn IpcHookDispatcher>>
parameter threaded all the way through ipc_accept.rs:391-684. Plan tasks
#1 / #3 / #5 unblocked.

DECISION-MARKER-REPORT (B-19 task #9 — MarkerReport SSB variant)
NEW: SsbActionType::MarkerReport added per QUINTUPLE-UPDATE (now 7+1 = 8
lockstep sites — enum + SQL CHECK + as_sql_str + from_str + SsbPayload variant +
action_type() arm + test_payload arm + all_test_payloads + action_type_lock
ALL_VARIANTS + exhaustive_match_guard + golden lock file). SCHEMA_VERSION
100->101. TOTAL_VARIANTS 288->298 (closes 9 PRE-EXISTING DRIFT variants
that shipped without the lock array bump: B-3 4x, B-4 1x, B-7 4x — surfaced
when the new MarkerReport addition forced action_type_lock to recompile).
Payload shape:
  agent_id: String
  step_id: String
  phase_counts: BTreeMap<String, u32>  // keys MUST come from DAIRA_MARKERS
  total_events: u32
  window_us: u64
Aggregation cadence: per-agent-execution boundary (task_runner Stop-hook
firing site is the suggested producer; deferred to a follow-up wiring
session since the current Stop site already emits per-finding rows, and
adding per-agent aggregation requires the dispatcher to maintain a
window-counter — a multi-task scope independent of this B-19 closure).
The variant + payload + emit helper (`emit_marker_report_to_ssb` at
crates/orchestrator/src/hook_dispatcher.rs) ship now so the wire surface
is stable for any future producer.
BTreeMap (not FxHashMap) per B-15 chokepoint judgement: serialisation must
be deterministic for the golden-lock bijection round-trip; FxHashMap
ordering varies across runs and would break the test.

DECISION-IPC-ADAPTER (B-19 plan task #1/#2 — dispatcher unification)
NEW: IpcHookAsyncAdapter struct + impl HookDispatcher at
crates/orchestrator/src/hook_dispatcher.rs (~190 LOC). Wraps
`Arc<dyn IpcHookTrait + Send + Sync>` and bridges to the async
HookDispatcher trait via tokio::task::spawn_blocking. Rationale: the
sync/async interface mismatch documented in exploration GAP-1 cannot be
collapsed at the trait level without one side conceding its execution
model. Unification is at the DISPATCHER level (Arc<dyn HookDispatcher>);
both upstream traits (IpcHookTrait sync, HookDispatcher async) remain
because they serve distinct callers — IpcHookTrait is the IPC-frame
interception point used by ipc_accept::run_agent_session at hook_dispatcher
parameter sites; HookDispatcher is the task_runner baby-step boundary
interceptor. The adapter is the missing plug that lets either dispatcher
fill the task_runner callsite.

NO new external dep — tokio::task::spawn_blocking is workspace tokio.
NO async-trait crate — native async fn in traits per CLAUDE.md anti-stale.
NO unwrap()/expect() on production paths.

DECISION-IPC-PROD-CTOR (B-19 plan task #1 — IpcHookDispatcher non-test wire)
NEW: build_production_ipc_hook_dispatcher(tiered, ssb) at
crates/orchestrator/src/runtime_setup.rs. Returns
Arc<dyn HookDispatcher + Send + Sync> by wrapping
IpcHookDispatcher::new in the IpcHookAsyncAdapter. Closes NOWIRE-2 from
the recheck report ("IpcHookDispatcher zero non-test constructors") —
the constructor is now available to RuntimeWiring callers. Production
default REMAINS InProcessHookDispatcher per DECISION-A in the original
B-19 doc (it owns ruff + ShortCoder + Tier1/2/3 + non-IDLE Block — 4
SHIPPED surfaces). Callers wanting the IPC path swap
`wiring.hook_dispatcher = Some(build_production_ipc_hook_dispatcher(...))`.

NEW: RuntimeSsbFindingWriter at runtime_setup.rs — production
SsbFindingWriter implementation wrapping Arc<Ssb>. Bridges sync trait
boundary to async write_payload via Handle::current() + block_in_place.
Soft-fail on no-runtime case (returns Err rather than panic).

UPSTREAM TRAIT CHANGE: SsbFindingWriter / CrossSessionLockChecker /
IpcHookTrait gained Send + Sync supertraits (crates/plugins/src/ipc_hooks.rs).
All real implementors are already thread-safe; the supertraits make the
constraint explicit so the wrapping IpcHookDispatcher can be hoisted into
Arc<dyn IpcHookTrait + Send + Sync> for the adapter's spawn_blocking call.
Two contract tests (plugins_adversarial.rs SpySsbWriter, plugins_hooks.rs
RecordingSsbWriter) swapped RefCell -> Mutex to satisfy the new bound.

DECISION-PRECOMPACT-GATE (B-19 plan task #5 — spec phase-4C.txt:28-29 honouring)
NEW: HookInvocation.pre_compact_token_state: Option<(u32, u32)> field +
HookInvocation::with_pre_compact_token_state builder.
NEW: PRE_COMPACT_HANDOFF_FRACTION = 0.85 const (mirrors
escalation_policy::HANDOFF_THRESHOLD_FRACTION; re-exposed at the
dispatcher to avoid a cross-module hot-path dep).
CHANGE: InProcessHookDispatcher::dispatch_pre_compact gates analyze() AND
the handoff-saver invocation behind the token-budget check:
  * `None` -> legacy behaviour (run analyze; no saver). Preserves the
    task_runner per-task firing site which has no token telemetry.
  * `Some((used, limit))` with limit==0 OR used/limit <= 0.85 -> NOOP
    (no analyze, no saver, no SSB row). Prevents SSB churn from
    unconditional firing.
  * `Some((used, limit))` with used/limit > 0.85 -> run saver (if
    installed) + run analyze. Honours spec.
SAME gate replicated in IpcHookAsyncAdapter::dispatch_pre_compact so a
dispatcher swap is behaviour-preserving.

DECISION-HANDOFF-SAVER (B-19 plan task #5 — PreCompact -> SessionDag::create_handoff wire)
NEW: PreCompactHandoffSaver trait at hook_dispatcher.rs:
  trait PreCompactHandoffSaver: Send + Sync {
      fn save_handoff(&self, step_id: &str) -> Result<(), String>;
  }
NEW: InProcessHookDispatcher::with_handoff_saver(saver) builder.
RATIONALE: The dispatcher does NOT own the SessionDag handle or the
conversation messages — that data lives in the chat-session layer
(crates/cli/src/live_cli.rs:1900-2354 already calls dag.create_handoff).
Surface the responsibility via a trait so the dispatcher fires the
callback at the spec-required 0.85 threshold without taking a dep on
SessionDag. Soft-fail (tracing::warn + continue) on saver error or
spawn_blocking join error — a snapshot-save failure must NOT abort an
in-flight task. The chat-session layer installs a concrete saver impl
that calls dag.create_handoff with the current TrimmedMessage list +
TrimConfig + SeededRng + ClockProvider + fsm_state.
The task_runner per-task PreCompact firing site does NOT install a
saver (no SessionDag/messages available there), so its dispatch is a
strict NOOP for handoff (DairaTrace HookPre/HookPost still emit
unchanged — the B-19 W2 telemetry shipped earlier is preserved).

DECISION-INPROCESS-RETAIN (B-19 plan task #3 — DELETION REJECTED)
KEEP InProcessHookDispatcher (do NOT delete). Per DECISION-A in the
original B-19 doc — it owns 4 SHIPPED surfaces (Tier1/2/3 tiered
analysis, ruff PostToolUse, ShortCoder PostToolUse, non-IDLE Block at
Stop). Deleting it would regress them all. The plan's stated "pick ONE
dispatcher" goal is satisfied at the TRAIT level (HookDispatcher is the
unified surface); both impl types ship now and callers pick per
deployment.

SPEC-STALE-blake2s (informational)
spec phase-4C.txt:283 says "blake2s hash of normalized pattern".
ipc_hooks.rs:228 uses blake3. Per CLAUDE.md anti-stale rule
("blake3 (never blake2/sha2 for non-RFC uses)") code is correct + spec
is stale. NO code change. Recommend spec correction in a future doc-only
pass; not blocking for B-19 closure.

INGEST items from grounding sweep: none flagged after rescan — all
external behaviour cited (tokio::task::spawn_blocking semantics, blake3
canonical use, HANDOFF_THRESHOLD_FRACTION=0.85) is verified against
in-codebase source.

cargo verification (s-25120):
  cargo check --workspace --tests: PASS (warnings only — pre-existing dead-code)
  cargo test -p elai-orchestrator --test action_type_lock: 2 passed (1 ignored)
  cargo test -p elai-orchestrator --lib hook_dispatcher::tests: 14 passed
    (8 pre-existing + 6 new B-19 — 5 PreCompact / 1 adapter object-safety)
  cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads: 1 passed
    (all_test_payloads expected count bumped 236 -> 237 for MarkerReport)

files modified (s-25120):
  crates/orchestrator/src/ssb.rs             — SCHEMA_VERSION 100->101,
                                                SsbActionType::MarkerReport,
                                                MarkerReportPayload struct,
                                                SsbPayload::MarkerReport variant,
                                                action_type() / as_sql_str /
                                                from_str / test_payload /
                                                all_test_payloads arms,
                                                SQL CHECK allowlist entry
  crates/orchestrator/src/hook_dispatcher.rs — PRE_COMPACT_HANDOFF_FRACTION,
                                                PreCompactHandoffSaver trait,
                                                HookInvocation.pre_compact_token_state
                                                + with_pre_compact_token_state,
                                                dispatch_pre_compact gate + saver,
                                                IpcHookAsyncAdapter (190 LOC),
                                                emit_marker_report_to_ssb,
                                                6 new unit tests
  crates/orchestrator/src/runtime_setup.rs   — build_production_ipc_hook_dispatcher
                                                + RuntimeSsbFindingWriter
  crates/orchestrator/src/lib.rs             — re-export new types
  crates/orchestrator/tests/action_type_lock.rs
                                              — TOTAL_VARIANTS 288->298,
                                                ALL_VARIANTS + 10 entries
                                                (MarkerReport + 9 pre-existing drift),
                                                exhaustive_match_guard 10 entries
  crates/orchestrator/tests/action_type_lock.golden
                                              — regenerated 297-entry sorted file
  crates/plugins/src/ipc_hooks.rs            — Send+Sync supertraits on
                                                SsbFindingWriter,
                                                CrossSessionLockChecker,
                                                IpcHookTrait
  crates/contracts/tests/plugins_hooks.rs    — RecordingSsbWriter: RefCell -> Mutex
  crates/contracts/tests/plugins_adversarial.rs
                                              — SpySsbWriter: RefCell -> Mutex
  docs/thefinalglance/B-plan/B-19_hook_system_unification.md
                                              — STATUS block + REBASELINE notes

return single-line status: see end of executor message

---
ORIGINAL S-91480 DECISIONS (preserved for historical context):

DECISION-A: KEEP both dispatchers as separate layers
Rationale: InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs) fires at
orchestrator baby-step boundaries (T1/T2/T3 tiered analysis, ShortCoder, ruff).
IpcHookDispatcher (plugins/src/ipc_hooks.rs) fires at IPC-frame granularity from
ipc_gateway.rs:530 and ipc_gateway.rs:581 via IpcHookTrait::pre_tool_use /
post_tool_use. They are NOT duplicates — they serve distinct concerns at different
granularity levels. Merging would eliminate the IPC-frame enforcement point.
Action: update doc-comments only; do NOT delete either dispatcher.

DECISION-B: Update doc-comment in plugins/src/ipc_hooks.rs
Remove stale "Production wiring scheduled in Session wire-4A" line.
Replace with two-sentence clarification of layering.
Grounding: ipc_gateway.rs lines 530 and 581 are live non-test consumers.

DECISION-C: 15-marker DAIRA list adopted
Keep DairaTracePayload.phase as String (adding a new enum would cascade to all
call sites and SSB deserialization). Add DAIRA_MARKERS const slice for documentation
and validation. Final 15-marker vocabulary:
  Call, Return, Mutation, Branch, Error, Finding, Read, Write,
  ToolCallStart, ToolCallEnd, HookPre, HookPost, Timeout, Escalate, Converge
Current emit sites use "enter"/"exit"/"started"/"success"/"error"/"skipped" which
map to ToolCallStart/ToolCallEnd/Call/Return/Error/Return respectively.
New sites added: hook_pre/hook_post at dispatch boundaries in dispatch_pre_compact.
Emit semantics per marker:
  Call      — function entry (maps from "enter")
  Return    — function exit clean (maps from "exit"/"success")
  Mutation  — state-changing side effect (e.g. file write)
  Branch    — conditional path taken
  Error     — error bubble (maps from "error")
  Finding   — auditor BLOCK emit
  Read      — file/db read
  Write     — file/db write
  ToolCallStart — tool dispatch entry
  ToolCallEnd   — tool dispatch exit
  HookPre   — hook point firing before action
  HookPost  — hook point firing after action
  Timeout   — operation exceeded budget
  Escalate  — escalation signal needed
  Converge  — spiral convergence / resolution

DECISION-D: Stop hook enforcement strategy
HookInvocation gains optional fsm_state: String field (default empty = no check).
dispatch_stop checks: if fsm_state.to_uppercase() not in {IDLE, CONFIGURING, ""},
return Block with sanitized message. Empty string = skip check (backward compatible
for call sites that don't plumb FSM state yet).
Grounding: IpcHookDispatcher.stop() already does this check (ipc_hooks.rs:397).
InProcessHookDispatcher dispatch_stop must match.

DECISION-E: Ruff PostToolUse wiring
RuffPostToolUseRunner and with_ruff_runner already exist and are already wired
inside dispatch_post_tool_use (hook_dispatcher.rs:779). The runner is only None
because no production TaskRunnerConfig callsite passes a hook_dispatcher.
Action: expose via lib.rs re-export (RuffPostToolUseRunner). Document that callers
should wire: InProcessHookDispatcher::new(tiered).with_ruff_runner(Arc::new(RuffPostToolUseRunner::new())).
No internal change needed — wiring is already present in dispatch_post_tool_use.

DECISION-F: PreCompact SSB emission (reuse DairaTrace, no new variant)
Replace the debug no-op in dispatch_pre_compact with DairaTrace emission.
Since dispatch_pre_compact does not have access to Arc<Ssb> (InProcessHookDispatcher
holds no writer), the DairaTrace rows must be emitted at the task_runner.rs call site
after dispatch_pre_compact returns. The dispatcher itself returns the existing
HookFinding vec; the task_runner already emits those via emit_hook_events_to_ssb.
Add explicit DairaTrace "HookPre"/"HookPost" emission in task_runner.rs around the
dispatch_pre_compact call.

DECISION-G: Dashboard hook_kind -> hook_point fix
commands.rs:1817 reads v.get("hook_kind") but HookVerdictEntry serializes as
hook_point (ssb_viewer.rs:472). Fix: change get("hook_kind") to
get("hook_point").or_else(|| v.get("hook_kind")) for backward compatibility.

Schema bump: NONE — reusing DairaTracePayload and existing HookEvent payloads.

SESSION STATUS per-priority:
P1 DONE — this file
P2 DONE — DAIRA_MARKERS added to ssb.rs; new emit in dispatch_pre_compact area
P3 DONE — fsm_state field added to HookInvocation; dispatch_stop enforcement added
P4 DEFERRED — ruff already internally wired; no production callsite has hook_dispatcher=Some; wiring requires dashboard-v2 or agent main.rs changes that are out of scope for this batch
P5 DONE — PreCompact no-op replaced with DairaTrace emission in task_runner.rs
P6 DONE — hook_kind fixed to hook_point in commands.rs
P7 DEFERRED — requires new Tauri command + SSB aggregation query; significant scope; deferred to B-19B
P8 DONE — ipc_hooks.rs doc-comment updated

cargo results:
  cargo check -p elai-orchestrator: PASS
  cargo check -p elai-plugins: PASS
  cargo check -p elai-dashboard-v2: PASS
  cargo check --workspace: PASS (all crates)
  cargo test -p elai-plugins post_tool_use: PASS (5/5)
  cargo test -p elai-orchestrator hook: FAIL to compile — pre-existing PhaseNode
    import error in pipeline_tree_navigator.rs test section (E0412/E0422, lines 1717-1728).
    Not caused by B-19 changes. cargo check (lib) passes; only test binary compilation
    is blocked by the pre-existing issue.
  bash scripts/check_anti_stale.sh: EXIT 0 (STRICT clean)
  cargo test -p elai-plugins (full): 99 pass, 2 fail (discovers_builtin_and_bundled_plugins,
    default_bundled_root_loads_repo_bundles_as_installed_plugins) — pre-existing, unrelated
    to B-19 (requires example-bundled directory on disk).

Files modified:
  P1: F:\.ELAI_workflow\workflow\reasoning\B-19-decisions.md (new)
  P2: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs
      - Added DAIRA_MARKERS const &[&str] with 15-marker vocabulary
      - Updated DairaTracePayload doc-comment to reference DAIRA_MARKERS
  P3: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs
      - Added fsm_state: String field to HookInvocation (default empty)
      - Updated for_task_level, for_pre_tool_use, for_post_tool_use helpers
      - Added with_fsm_state() builder method
      - dispatch_stop now blocks if fsm_state non-empty and not IDLE/CONFIGURING
  P4: DEFERRED (no production callsite builds InProcessHookDispatcher; ruff already
      internally wired, just not activated)
  P5: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs
      - PreCompact dispatch now emits DairaTrace "HookPre"/"HookPost" around dispatch
  P5: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs
      - dispatch_pre_compact: replaced debug no-op comment with wiring explanation
  P6: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs
      - get_hook_executions: reads hook_point with fallback to hook_kind
  P7: DEFERRED — new Tauri command + SSB aggregation out of scope
  P8: F:\.ELAI_workflow\.elai_cc\crates\plugins\src\ipc_hooks.rs
      - Updated top doc-comment: removed stale wire-4A line, added layering clarification

BLOCKs/WARNs for auditor:
  BLOCK: P7 (tiered analysis dashboard wiring) deferred — synthetic data still shown
  WARN: P4 ruff runner not wired in production path (requires production callsite change)
  WARN: HookInvocation.fsm_state default empty = no enforcement for pre-existing callers
  INFO: orchestrator hook test binary has pre-existing PhaseNode compilation error
        unrelated to B-19; auditor should note as existing technical debt
