I-8 decisions: aletheia voluntary abort
task_id: A85-I8-EXECUTE
session: s-107b022a
date: 2026-06-12

WHAT
Ship the voluntary-abort mechanism: acknowledge_infeasible tool (3-arm pattern), dead_end_cache.rs
(bounded VecDeque/HashSet, blake3 fingerprint, stdlib-only), TrajectoryAborted SSB variant
(3-arm Kind: Voluntary/DeadEndHash/HaltingBound), abort handler (OutcomeTriple->AbstentionScored
emission + conditional timetravel rollback), I-17 halting-bound seam (config field + registered
stub), and i8 contract test suite (5 arms).

KEY DECISIONS

D-1 Dead-end cache backing store: VecDeque<String> + HashSet<String> (stdlib only, no lru crate).
  Rationale: lru crate unverified in Cargo.lock (exploration gap confirmed). stdlib combo gives O(1)
  amortised lookup + O(n) eviction at capacity, which is fine for DEFAULT_DEAD_END_CACHE_CAPACITY=256.
  Rejected: adding new crate dependency (anti-slopsquat HARD RULE).

D-2 blake3_hex reuse: import from crate::architecture::timetravel::blake3_hex rather than
  re-implementing. Grounded in context-package + exploration (timetravel.rs:203 confirmed live).

D-3 Halt watchdog seam: add halting_bound_ms: Option<u64> to TaskRunnerConfig (verified live at
  task_runner.rs:2619) as I-17's injection point. Wave-1 ships the config field + HaltingBound
  variant + named constant DEFAULT_HALTING_BOUND_MS_FALLBACK; the actual watchdog timer is a
  registered TODO[I-17] stub. No hardcoded 15s timer in this wave.

D-4 AcknowledgeInfeasibleRequest location: ipc-types/src/lib.rs (same file as all other tool
  request structs). Follows EditRequest/FileReadRequest precedent.

D-5 SSB TrajectoryAborted baseline: live values at edit time are SCHEMA_VERSION=237, TOTAL_VARIANTS=546.
  I-8 bumps to 238/547. Verified by grep immediately before editing.

D-6 Dispatcher arm Wave-1: translate_tool_request arm emits TrajectoryAborted(Voluntary) SSB row
  then returns Err(reason) — matching the established Err-stub pattern at :2407-2410. The Err
  propagates as ToolOutcome::Failed per handle_tool_call :1529-1530. This is NOT pure-Err-no-emit
  (that would be wire-dark per the exploration warning).

D-7 Rollback Wave-1: abort handler calls timetravel::materialize() only if a pre-abort manifest is
  passed (Option<FileManifest>). If None, logs "rollback skipped: no pre-abort snapshot" and
  continues. Full trunk-restore via SessionDag HEAD is Wave-2 per plan.

D-8 Contract test structure: follows h39_abstention_score_contract.rs fixture pattern exactly
  (TempDir + Ssb::new with pool_size Some(2)). No bench trio. 5 test functions per plan.

EXPECTED TEST OUTCOME
cargo check -p elai-orchestrator -p elai-contracts --tests: exit 0
contract test i8 voluntary/dead-end/halting/rollback/schema arms: all pass
action_type_lock tests: 2/2 (238/547 pin)

UNKNOWNS
H-49 SessionDag HEAD linkage not available Wave-1 — acknowledged, skip+log path implemented.
I-17 watchdog timer not available Wave-1 — registered stub TODO[I-17] with seam.

---

FIX DECISIONS (task_id: A89-I8-FIX, session: s-107b022a, date: 2026-06-12)

WHAT
Three BLOCK-1 / WARN-1 / F-3 findings from I-8-audit.md remediated.

D-FIX-1 Dead-end cache pre-dispatch hook — no kill-switch flag.
  BLOCK-1 required wiring check_and_insert before model dispatch. Remedy placed in task_runner.rs
  inside the `else` branch of the focus_tunnel / hook_pre_blocked / executor chain (line ~9228),
  initialised once per drive_task invocation as `let mut dead_end_cache` (stack-local, not in
  ResolvedConfig). Default-ON per plan spec. No ELAI_*_ENABLE kill-switch added (the plan
  explicitly says "Default-ON, no kill-switch flag" and the cache is a pure in-memory prune with no
  persistent side effects that would require a bypass).

D-FIX-2 Fail-closed allow-list chosen over block-list for calibration suite filter.
  WARN-1 asked for a filter so voluntary_abort AbstentionScored rows cannot bias the conformal
  corpus. Chose an allow-list (CALIBRATION_ALLOWED_SUITES = ["seccodeplt"]) over a block-list
  because: (a) unknown new suites are automatically excluded until explicitly promoted — this is the
  stricter posture; (b) a block-list would silently admit any novel suite string. Constants and
  is_suite_excluded_from_calibration() placed in abstention_score.rs and re-exported via
  calibration/mod.rs so I-23 has a stable import path.

D-FIX-3 Stack-local dead_end_cache vs ResolvedConfig field.
  Considered adding a `dead_end_cache: DeadEndCache` field to ResolvedConfig. Rejected because:
  (a) DeadEndCache is not Sync (VecDeque + HashSet with &mut API), adding it to a shared config
      would require Arc<Mutex<>> with lock contention across concurrent baby steps;
  (b) dead-end detection is per-task (within one drive_task call), not cross-task — a per-invocation
      local is the correct scope;
  (c) initialising stack-local before the loop adds zero cost vs a field.

D-FIX-4 Source-scan wiring arm rather than full smoke test.
  F-3 asked for a wiring arm proving production pre-dispatch path consults the cache. Chose a
  source-scan static test (reads task_runner.rs + asserts contains call sites) rather than a full
  start_task_runner integration smoke because: producing two baby steps with identical
  task_summary bytes within one drive_task run requires controlling the decompose tree output,
  which is outside the hermetic contract boundary (decompose calls LLM or a stub that generates
  different summaries per step). The source-scan + existing payload round-trip (F3) together
  constitute the Layer-2 + Layer-3 evidence chain; Layer-4 fires in the production binary.

EXPECTED TEST OUTCOME (post-fix)
i8_voluntary_abort_contract: 9/9 PASS (7 original + 2 new arms)
cargo check --workspace --tests: exit 0 (0 errors, pre-existing warnings only)
