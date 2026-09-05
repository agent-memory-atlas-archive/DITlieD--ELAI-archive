# C-10: Phase 4G.9 Anti-Spiral Primitives — StagnationDetection + Personas + GoalDriftCanary + GenerationWatchdog
Objective: implement all 4G.9 anti-spiral / lateral-thinking types absent from
the codebase — StagnationPattern enum, StagnationDetection, ExecutionHistory
ring buffer, ThinkingPersona, PersonaStrategy, GoalDriftCanary (Jaccard),
GenerationWatchdog (blake3 fingerprint), AdaptiveFutilityTracker — wire into
autonomous_gate.rs decide(), emit stagnation SSB rows, create 5 persona agent
files, add OrchestrationDrawer StagnationSection.
Estimated effort: 2 sessions.
Pre-requisites: C-9 (AutoAdvanceEvaluator + EscalationPolicy ship first; retry
counts feed StagnationDetection history); C-2 (stagnation_* and
lateral_thinking_* SsbActionType variants defined in ssb.rs).
Parallel-safe with: C-11..C-15 (non-overlapping file scope).

## Context

phase-4G.report.md confirms all 4G.9 symbols are entirely absent — zero matches
in elai.map: StagnationPattern, ExecutionHistory, StagnationDetection,
ThinkingPersona, PersonaStrategy, GenerationWatchdog, WatchdogTimeoutKind,
GoalDriftCanary. The 5 persona agent files do not exist in .claude/agents/.
C-2 added SSB variant definitions (StagnationSpinningDetected,
StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning,
WatchdogTimeout) but zero emit sites exist. autonomous_gate.rs carries
EscalationTrigger::AntiSpiral (~line 374) with no detection logic; the gap is
the detection machinery itself. .claude/skills/anti-spiral.md documents the
same 4-pattern taxonomy and 5-persona switch at the Claude Code level; C-10
ports that logic into Rust for the autonomous /task runtime loop.

## Reads

- .elai_cc/crates/orchestrator/src/autonomous_gate.rs (EscalationTrigger,
  GateContext, AutonomyDecision, GateMode, decide())
- .elai_cc/crates/orchestrator/src/ssb.rs (C-2 stagnation variants + emit helper)
- .elai_cc/crates/orchestrator/src/task_runner.rs (decide() call site, step hooks)
- .elai_cc/crates/contracts/tests/autonomous_gate_contract.rs (14 tests — no regress)
- .elai_cc/crates/contracts/tests/phase_4g_extended_contract.rs (18 tests)
- docs/specs/guide/steps/phase-4G.txt §4G.9
- verification/reports/steps/phase-4G.report.md §Deliverables MISSING
- .claude/skills/anti-spiral.md (pattern taxonomy, affinity table)

## Tasks

1. **New module: anti_spiral_extended.rs**
   File: .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs.
   Define all 4G.9 types:
   - StagnationPattern enum: Spinning | Oscillation | NoDrift |
     DiminishingReturns. Derive Debug, Clone, Copy, PartialEq, Eq,
     Serialize, Deserialize.
   - AttemptResult: {attempt_index: u32, error_class: String,
     output_hash: u64, error_count: u32, timestamp_ms: u64}.
   - ExecutionHistory ring buffer (capacity 32): VecDeque<AttemptResult>
     with push(), is_full(), last_n(), clear(). Stdlib only.
   - StagnationDetection: {spin_threshold=3, oscillation_window=4,
     diminishing_min_window=4, diminishing_min_improvement_pct=5.0f32}.
     detect_pattern(history) -> Option<StagnationPattern>. Rules in order:
     NoDrift (last 2 hashes equal), Spinning (last N error_class identical),
     Oscillation (last 4 alternate between 2 strings), DiminishingReturns
     (error_count improvement < 5% per step over 4-attempt window).
     Returns None if history len < 2.
   - ThinkingPersona enum: Hacker | Researcher | Simplifier | Architect |
     Contrarian. Derive same as StagnationPattern.
   - PersonaStrategy: {persona: ThinkingPersona, prompt_prefix: &'static str}.
     PERSONA_STRATEGIES: [PersonaStrategy; 5] static table with short
     imperative prefix per persona. suggest_persona_for_pattern(pattern) ->
     ThinkingPersona — deterministic affinity: Spinning->Contrarian,
     Oscillation->Architect, NoDrift->Hacker, DiminishingReturns->Simplifier.
     Researcher is fallback rotation (index % 5 == 4).
   - GoalDriftCanary: {green_threshold=0.7f32, yellow_threshold=0.4f32}.
     tokenize(text) -> Vec<String> (whitespace-split, strip non-alphanumeric,
     lowercase). compute(original, current) -> f32 Jaccard via BTreeSet (not
     HashSet — determinism). verdict(score) -> DriftVerdict.
     DriftVerdict enum: Green | Yellow | Red.
   - WatchdogTimeoutKind enum: Idle | NoMaterialProgress | Safety.
   - GenerationWatchdog: {idle_timeout_ms=120_000, no_progress_timeout_ms=
     600_000, safety_timeout_ms=1_800_000, workflow_fingerprint: Option<u64>}.
     compute_fingerprint(ssb_row_count: u64, output_hash: u64) -> u64 — blake3
     of 16-byte concat, return first 8 bytes as u64. Verify blake3 crate
     present in Cargo.toml before writing; hard-error if absent.
     check(last_event_ms, now_ms, current_fingerprint, start_ms) ->
     Option<WatchdogTimeoutKind>. Order: Safety > NoMaterialProgress > Idle.
     Updates self.workflow_fingerprint after check.
   - AdaptiveFutilityTracker: {success_count: HashMap<String,u32>,
     attempt_count: HashMap<String,u32>}. record(error_class, succeeded),
     success_rate(error_class) -> f32 (0.5 prior if no data),
     is_futile(error_class, threshold) -> bool (rate < threshold AND
     attempts >= 4). Serialize payload to SSB on trigger.

2. **Wire StagnationDetection into autonomous_gate.rs decide()**
   Add GateContext fields: execution_history: ExecutionHistory (default empty),
   attempt_index: u32 (default 0), suggested_prompt_prefix: Option<&'static str>
   (default None). Existing callers unaffected via struct update syntax.
   In decide(): after EscalationTrigger::AntiSpiral check, call
   StagnationDetection::detect_pattern(&ctx.execution_history). On Some(pattern):
   emit stagnation SSB row (SpinningDetected or OscillationDetected variant;
   pattern field in payload disambiguates NoDrift / DiminishingReturns);
   emit LateralThinkingActivated SSB row with new_persona field; set
   ctx.suggested_prompt_prefix = Some(PERSONA_STRATEGIES[persona].prompt_prefix).
   File: .elai_cc/crates/orchestrator/src/autonomous_gate.rs

3. **Wire GoalDriftCanary at step boundaries in task_runner.rs**
   Store original_task_tokens: Vec<String> in task session state at task start.
   After each baby_step completion: compute Jaccard vs current subtask text.
   If DriftVerdict::Red: emit GoalDriftWarning SSB row with jaccard_score,
   original_snippet (first 80 chars), current_snippet (first 80 chars).
   File: .elai_cc/crates/orchestrator/src/task_runner.rs

4. **Wire GenerationWatchdog in task_runner.rs**
   Instantiate GenerationWatchdog::default() per task. On each step heartbeat:
   call watchdog.check(last_event_ms, now_ms, fingerprint, start_ms). On Some:
   emit WatchdogTimeout SSB row (kind, elapsed_ms, fingerprint_hex); set
   AutonomyDecision::Escalate(EscalationTrigger::AntiSpiral). Use
   tokio::time::Instant. File: .elai_cc/crates/orchestrator/src/task_runner.rs

5. **AdaptiveFutilityTracker in task_runner.rs**
   Per-task instance in TaskRunnerState. After each baby_step attempt: call
   tracker.record(error_class, succeeded). Before each repair-loop iteration:
   call tracker.is_futile(error_class, 0.15). If true: skip remaining retries;
   escalate Escalate(EscalationTrigger::CircuitBreaker) with serialized tracker
   payload in SSB row detail. File: .elai_cc/crates/orchestrator/src/task_runner.rs

6. **5 persona agent files**
   Each file: ~30 lines, front-matter: name, description, model: sonnet,
   tools: [Read, Grep, Glob, Bash, WebSearch], instructions block.
   - .claude/agents/persona-hacker.md — ignores elegance; proposes 3 shortest
     implementation paths (DIRTY/CLEAN/HYBRID); outputs one concrete baby_step.
   - .claude/agents/persona-researcher.md — 3 web searches for prior art;
     outputs the most relevant reference + derived next action.
   - .claude/agents/persona-simplifier.md — strips implementation to minimal
     skeleton; adds layers back until failure reappears; names the bad layer.
   - .claude/agents/persona-architect.md — maps dependency graph of oscillating
     error pair; identifies coupling root; outputs decoupling baby_step.
   - .claude/agents/persona-contrarian.md — states opposite assumption; derives
     alternative baby_step; no hedging.

7. **OrchestrationDrawer.svelte — StagnationSection**
   Add collapsed StagnationSection querying new Tauri command
   get_stagnation_events (returns Vec<SsbRow> for action_types:
   stagnation_spinning_detected, stagnation_oscillation_detected,
   lateral_thinking_activated, goal_drift_warning, watchdog_timeout ordered
   by timestamp DESC). Render: timestamp, color-coded badge (amber=stagnation,
   blue=lateral, orange=goal_drift, red=watchdog), pattern/persona field,
   detail string. Follow GatesPanel.svelte color token pattern; no new tokens.
   Files: .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte,
          .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs

8. **Contract tests: anti_spiral_extended_contract.rs** (17 functions)
   File: .elai_cc/crates/contracts/tests/anti_spiral_extended_contract.rs
   - test_spinning_detected_after_threshold_identical_errors
   - test_no_drift_detected_on_identical_hash
   - test_oscillation_detected_on_alternating_error_pair
   - test_diminishing_returns_detected_below_5pct
   - test_detect_returns_none_with_single_entry
   - test_persona_affinity_spinning_yields_contrarian
   - test_persona_affinity_oscillation_yields_architect
   - test_persona_affinity_no_drift_yields_hacker
   - test_persona_affinity_diminishing_yields_simplifier
   - test_goal_drift_canary_green_above_07
   - test_goal_drift_canary_red_below_04
   - test_goal_drift_tokenize_strips_punctuation
   - test_watchdog_safety_fires_at_ceiling
   - test_watchdog_no_progress_fires_on_same_fingerprint
   - test_watchdog_idle_fires_on_no_events
   - test_adaptive_futility_is_futile_below_threshold
   - test_execution_history_ring_evicts_at_capacity

9. **Contract tests: anti_spiral_gate_integration_contract.rs** (5 functions)
   File: .elai_cc/crates/contracts/tests/anti_spiral_gate_integration_contract.rs
   Uses in-process SSB (SQLite temp-db per autonomous_gate_contract.rs pattern).
   - test_decide_emits_stagnation_spinning_after_3_identical_errors
   - test_decide_emits_lateral_thinking_activated_with_persona
   - test_decide_attaches_prompt_prefix_to_gate_ctx
   - test_goal_drift_warning_emitted_when_jaccard_red
   - test_watchdog_timeout_emits_ssb_and_sets_escalate

10. **Metrics update**
    Add [stagnation] section to .elai_cc/metrics/phase_4G.toml:
    stagnation_detection_enabled = true, persona_switch_count = 0,
    goal_drift_red_count = 0, watchdog_timeout_count = 0,
    spin_threshold = 3, adaptive_futility_threshold = 0.15.

## WIRING_CONTRACT

produces_symbols:
  - StagnationPattern (4 variants), AttemptResult, ExecutionHistory
  - StagnationDetection::detect_pattern()
  - ThinkingPersona (5 variants), PersonaStrategy, PERSONA_STRATEGIES,
    suggest_persona_for_pattern()
  - GoalDriftCanary::compute() Jaccard + verdict() + tokenize()
  - DriftVerdict (Green | Yellow | Red)
  - WatchdogTimeoutKind (3 variants), GenerationWatchdog::check() +
    compute_fingerprint()
  - AdaptiveFutilityTracker::record() + success_rate() + is_futile()
  - GateContext fields: execution_history, attempt_index, suggested_prompt_prefix
  - task_runner.rs: GoalDriftCanary emit, GenerationWatchdog tick,
    AdaptiveFutilityTracker per-task + CircuitBreaker escalation
  - get_stagnation_events Tauri command + OrchestrationDrawer StagnationSection
  - 5 persona agent files; 22 new contract tests (2 files)

consumes_symbols:
  - C-2: SsbActionType::{StagnationSpinningDetected, StagnationOscillationDetected,
    LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout} + emit helper
  - C-9: EscalationPolicy.anti_spiral_threshold (feeds spin_threshold default)
  - autonomous_gate.rs: EscalationTrigger::AntiSpiral, GateContext, decide()
  - task_runner.rs: step-boundary hooks, ssb_tx, TaskRunnerState

reachable_from:
  - elai /task --mode autonomous (all detection paths active)
  - autonomous_gate.rs decide() every gate eval
  - task_runner.rs step heartbeat
  - OrchestrationDrawer.svelte StagnationSection

stub_ban:
  - No .unwrap() in anti_spiral_extended.rs or new task_runner.rs paths
  - GoalDriftCanary MUST use BTreeSet (not HashSet) for determinism
  - GenerationWatchdog MUST use blake3; hard-error if crate absent
  - AdaptiveFutilityTracker MUST serialize to SSB payload (not silently dropped)
  - Persona agent files MUST follow existing agent front-matter format exactly

observable_effect:
  trigger: elai /task --mode autonomous "fix the impossible bug in src/broken.rs"
    (broken.rs = syntactically valid, no actual bug, or assert_eq!(1,2) test)
  expect_ssb: after 3 attempts: stagnation_spinning_detected row (pattern=spinning,
    iteration=3); lateral_thinking_activated row (new_persona=<one of 5>);
    next prompt row contains persona prompt_prefix; if subtask text drifts,
    goal_drift_warning row (jaccard_score < 0.4); at 10-min no-progress ceiling,
    watchdog_timeout row (kind=no_material_progress) terminates task
  expect_dashboard: OrchestrationDrawer StagnationSection shows all events with
    timestamps and color-coded badges (amber/blue/orange/red per type)

## DETERMINISTIC_CHECK

```bash
# 1. All 4G.9 types in anti_spiral_extended.rs (>= 9 hits)
rg "StagnationPattern|ExecutionHistory|StagnationDetection|ThinkingPersona|\
PersonaStrategy|GoalDriftCanary|WatchdogTimeoutKind|GenerationWatchdog|\
AdaptiveFutilityTracker" .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs \
  | wc -l

# 2. detect_pattern wired in decide()
rg "detect_pattern|StagnationDetection" \
  .elai_cc/crates/orchestrator/src/autonomous_gate.rs | wc -l
# Expected: >= 2

# 3. GoalDriftCanary emit in task_runner (>= 3)
rg "GoalDriftCanary|goal_drift_warning|DriftVerdict::Red" \
  .elai_cc/crates/orchestrator/src/task_runner.rs | wc -l

# 4. GenerationWatchdog in task_runner (>= 3)
rg "GenerationWatchdog|watchdog_timeout|WatchdogTimeoutKind" \
  .elai_cc/crates/orchestrator/src/task_runner.rs | wc -l

# 5. All 5 persona files present
ls .claude/agents/persona-hacker.md .claude/agents/persona-researcher.md \
  .claude/agents/persona-simplifier.md .claude/agents/persona-architect.md \
  .claude/agents/persona-contrarian.md 2>&1 | grep -c ".md"
# Expected: 5

# 6. Contract tests pass (>= 22)
cargo test -p elai_contracts -- anti_spiral 2>&1 | grep "test result"

# 7. No regression in existing 4G tests
cargo test -p elai_contracts -- autonomous_gate phase_4g 2>&1 | grep "test result"
# Expected: >= 14 + >= 18 passed

# 8. get_stagnation_events Tauri command present
rg "get_stagnation_events" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | wc -l
# Expected: >= 1

# 9. Cargo check clean
cargo check -p elai_orchestrator -p elai_contracts 2>&1 | tail -2
```

## ACCEPTANCE

- [ ] anti_spiral_extended.rs compiles: all 11 types defined (StagnationPattern, AttemptResult,
      ExecutionHistory, StagnationDetection, ThinkingPersona, PersonaStrategy, DriftVerdict,
      GoalDriftCanary, WatchdogTimeoutKind, GenerationWatchdog, AdaptiveFutilityTracker)
- [ ] detect_pattern correct for all 4 variants (contract-tested)
- [ ] GoalDriftCanary::compute: Jaccard in [0,1]; BTreeSet confirmed (not HashSet)
- [ ] GenerationWatchdog::compute_fingerprint: blake3 grep-confirmed in Cargo.toml
- [ ] GateContext fields added (execution_history, attempt_index, suggested_prompt_prefix); zero regression on existing callers
- [ ] decide() emits stagnation + LateralThinkingActivated SSB rows on pattern detection
- [ ] task_runner emits GoalDriftWarning (Red) at step boundary; ticks GenerationWatchdog
- [ ] AdaptiveFutilityTracker triggers CircuitBreaker at rate < 0.15 / >= 4 attempts; SSB payload serialized
- [ ] 5 persona agent files exist with valid front-matter; each ~30 lines
- [ ] 17 anti_spiral_extended + 5 gate_integration contract tests pass
- [ ] Zero regression: autonomous_gate_contract (>=14) + phase_4g_extended (>=18)
- [ ] phase_4G.toml [stagnation] section added
- [ ] get_stagnation_events returns correct SSB rows; StagnationSection renders
- [ ] cargo check -p elai_orchestrator -p elai_contracts: 0 errors
- [ ] Cloud-Sonnet smoke test PROVEN:
      Run: elai /task --mode autonomous "fix the impossible bug in src/broken.rs"
        (broken.rs = valid Rust file with no real bug, or assert_eq!(1,2) test)
      (a) After 3 identical failures: stagnation_spinning_detected SSB row with
          pattern="spinning", iteration=3
      (b) lateral_thinking_activated SSB row with new_persona in {hacker,
          researcher, simplifier, architect, contrarian}
      (c) Next baby_step prompt SSB row contains persona's prompt_prefix string
      (d) If subtask drifts: goal_drift_warning row with jaccard_score < 0.4
      (e) At 10-min no-progress ceiling: watchdog_timeout row (kind=no_material_progress) terminates the task
      (f) OrchestrationDrawer StagnationSection shows all events; amber badge on
          stagnation rows; blue badge on lateral_thinking rows

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs
- MODIFY: .elai_cc/crates/orchestrator/src/autonomous_gate.rs
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs
- ADD: .elai_cc/crates/contracts/tests/anti_spiral_extended_contract.rs
- ADD: .elai_cc/crates/contracts/tests/anti_spiral_gate_integration_contract.rs
- MODIFY: .elai_cc/metrics/phase_4G.toml
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte
- ADD: .claude/agents/persona-hacker.md
- ADD: .claude/agents/persona-researcher.md
- ADD: .claude/agents/persona-simplifier.md
- ADD: .claude/agents/persona-architect.md
- ADD: .claude/agents/persona-contrarian.md
- CREATE: workflow/reasoning/C-10-decisions.md

## Next

C-11 (next verification gap). Confirm C-9 shipped before opening this session.
