# C-9: Autonomous Mode Core Infrastructure

Objective: implement the named type gaps and production wiring identified in
phase-4G.report.md and B-22.report.md — AutoAdvanceEvaluator, EscalationPolicy,
SplitTaskPlan/SubTask/MergeStrategy, EvidenceCheck/EvidenceVerdict, two missing
EscalationTrigger variants, ModelPool::can_fit_llm(), production wiring of
TreeAutoAdvancePolicy in cli/main.rs, [agents.{role}.backend] toml stanza,
phase_4G.toml runtime measurement population, and AutonomyDecision::SplitTask
calling ResearchTree::split_step().
Estimated effort: 2-3 sessions.
Pre-requisites: C-1 (SSB emission baseline — gate_decision rows must exist so
acceptance smoke test finds real SSB rows); C-5 (cloud mode + --mode autonomous
flag, since the smoke test drives cloud Sonnet 4.6 in autonomous mode).
Parallel-safe with: C-10 (anti-spiral primitives — StagnationPattern,
ThinkingPersona, GenerationWatchdog, GoalDriftCanary live in a new
orchestrator module not touched here).

## Context

phase-4G.report.md §Deliverables PARTIAL + MISSING (verified 2026-05-17)
documents seven structural gaps in autonomous_gate.rs that each have a direct
spec reference in phase-4G.txt §4G.1:

1. AutoAdvanceEvaluator struct with gate_checks: HashMap<FsmTransition,
   EvidenceCheck> — missing; Evidence struct + all_passed() covers the
   semantics but not the named type (phase-4G.report:49-51).
2. EvidenceCheck + EvidenceVerdict — missing named types (phase-4G.report:80-81).
3. EscalationPolicy struct — missing; circuit_breaker_limit in AutonomousGateConfig
   covers max_auto_advances_per_phase but the other three fields are absent
   (phase-4G.report:44-47).
4. SplitTaskPlan/SubTask/MergeStrategy — missing; AutonomyDecision::SplitTask
   carries only two u32/u8 fields (phase-4G.report:37-40).
5. SubstituteModelFailed + SplitTaskFailed EscalationTrigger variants — missing;
   only 4 of 6 spec variants implemented (phase-4G.report:41-43, autonomous_gate.rs:374-384).
6. ModelPool::can_fit_llm() — missing; ModelPool trait only exposes
   available_vram() (agent_pool.rs:48-51, phase-4G.report:57-59).
7. TreeAutoAdvancePolicy wired in cli/main.rs — phase_4G.toml [notes]:165
   explicitly records "cli/main.rs still passes autonomous_gate: None".

B-22.report.md §Claims MISSING confirms [agents.{role}.backend] stanza
absent from elai_cc.toml (B-22.report:128-130).

phase-4G.report.md §Deliverables PARTIAL (metrics) confirms phase_4G.toml
contains only design-time metadata, not the 4G.6 runtime measurement keys
auto_advance_accuracy, false_escalation_rate, mode_switch_latency_ms, etc.
(phase-4G.report:65-67).

task_runner.rs:7345-7353 shows AutonomyDecision::SplitTask dispatches to
an optional splitter stub; ResearchTree::split_step() does not exist —
the dispatch is a TODO path, not a live call.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4G.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-22_phase_4G_autonomous_mode.report.md
- .elai_cc/crates/orchestrator/src/autonomous_gate.rs (Evidence:189-298, EscalationTrigger:374-397, AutonomyDecision:408-439, AutonomousGateConfig:484-503)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (ModelPool trait:48-51)
- .elai_cc/crates/orchestrator/src/task_runner.rs (SplitTask dispatch:7345-7409; derive_autonomous_evidence:7096)
- .elai_cc/crates/orchestrator/src/research_tree.rs (ResearchTree struct:994 — verify split_step absence)
- .elai_cc/crates/cli/src/main.rs (TaskRunnerConfig construction — confirm autonomous_gate: None)
- .elai_cc/crates/cli/src/runtime_config.rs ([agents.auditor] stanza at :135; confirm no backend key)
- .elai_cc/elai_cc.toml (confirm agents.* section structure)
- .elai_cc/metrics/phase_4G.toml (design-time snapshot; keys to add in task 10)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4G.txt (§§4G.1, 4G.5, 4G.6, 4G.7-4G.8)

## Tasks

1. **EvidenceVerdict enum** in autonomous_gate.rs. Three variants:
   Pass, Fail { reason: String }, Missing. Derive Debug/Clone/PartialEq/
   Serialize/Deserialize. Add as_str() for SSB row labelling.

2. **EvidenceCheck struct** in autonomous_gate.rs. Fields:
   artifact_path: Option<PathBuf>, description: &'static str,
   check_fn: fn(&Evidence) -> EvidenceVerdict.
   No async: check_fn reads from the Evidence snapshot already computed by
   derive_autonomous_evidence.

3. **AutoAdvanceEvaluator struct** in autonomous_gate.rs. Fields:
   gate_checks: HashMap<FsmTransition, EvidenceCheck>. Methods:
   evaluate(&self, transition: &FsmTransition, evidence: &Evidence)
   -> EvidenceVerdict (calls check_fn for the matching key, returns Missing if
   no entry for that transition).
   Impl Default: pre-populate gate_checks with one entry per EscalationTrigger
   variant (AntiSpiral, HighRisk, ReplanMajor, CircuitBreaker) mapped to the
   relevant Evidence field check. The existing Evidence::all_passed() path in
   AutonomousGate::decide() calls AutoAdvanceEvaluator::evaluate_all() which
   iterates gate_checks and calls check_fn per entry.

4. **EscalationPolicy struct** in autonomous_gate.rs. Fields:
   anti_spiral_threshold: u32, high_risk_pauses: bool,
   max_auto_advances_per_phase: u32. Wire into AutonomousGateConfig as a
   new field escalation_policy: EscalationPolicy (alongside existing
   circuit_breaker_limit — keep circuit_breaker_limit for backward compat;
   max_auto_advances_per_phase mirrors it via Default). Add
   EscalationPolicy::default() (anti_spiral_threshold=3, high_risk_pauses=true,
   max_auto_advances_per_phase=10).

5. **SubstituteModelFailed + SplitTaskFailed** EscalationTrigger variants.
   Add after CircuitBreaker in the enum. Update as_str() match arm:
   SubstituteModelFailed => "substitute_model_failed",
   SplitTaskFailed => "split_task_failed".
   These fire in task_runner when SubstituteModel returns an empty fallback
   chain or when SplitTask collapses back to sub_task_count=0.

6. **MergeStrategy enum** in autonomous_gate.rs. Single variant for now:
   Sequential. Add doc note: "Parallel variant deferred until C-10
   anti-spiral fan-out is stable." Derive Debug/Clone/PartialEq/Serialize/
   Deserialize.

7. **SubTask struct** in autonomous_gate.rs. Fields: task_id: String,
   scope: String, context_budget: u32, depends_on: Vec<String>.
   Derive Debug/Clone/PartialEq/Serialize/Deserialize.

8. **SplitTaskPlan struct** in autonomous_gate.rs. Fields:
   parent_phase: String, sub_tasks: Vec<SubTask>,
   merge_strategy: MergeStrategy, max_depth: u8, current_depth: u8.
   Update AutonomyDecision::SplitTask to carry split_plan: SplitTaskPlan
   instead of two bare fields sub_task_count/depth. Update all match arms
   in task_runner.rs that destructure SplitTask.

9. **ModelPool::can_fit_llm()** in agent_pool.rs. Add to the ModelPool trait:
   fn can_fit_llm(&self, model_size_bytes: u64) -> bool {
       self.available_vram() >= model_size_bytes
   }
   Default impl on the trait so existing MockModelPool impls in bsp/engine.rs,
   and agent_pool.rs test helpers do not require changes. Wire in
   AutonomousGate::decide() before emitting SubstituteModel: if the target
   model's estimated size exceeds available VRAM, fire Escalate {
   trigger: EscalationTrigger::SubstituteModelFailed } instead.

10. **Wire TreeAutoAdvancePolicy in cli/main.rs**. Locate the TaskRunnerConfig
    construction that passes autonomous_gate: None (confirmed in phase_4G.toml
    notes:165). Replace None with Some(Arc::new(AutonomousGate::new(
    AutonomousGateConfig { mode: GateMode::from_cli_args(&args), ..Default::default() },
    None))). Add GateMode::from_cli_args() helper that reads the --mode flag
    value parsed in C-5 (returns Interactive when flag absent).

11. **[agents.{role}.backend] stanza** in elai_cc.toml.example. Add a new
    section after the existing [agents.auditor] block:

    [agents.executor.backend]
    backend = "auto"   # local | api | hybrid | auto

    [agents.auditor.backend]
    backend = "anthropic_api"

    [agents.designer.backend]
    backend = "anthropic_api"

    Add BackendStanza { backend: String } to the Agents config struct in
    runtime_config.rs. Each role gets an optional BackendStanza defaulting
    to "auto". This closes the B-22.report MISSING claim.

12. **Wire AutonomyDecision::SplitTask to ResearchTree::split_step()**.
    In task_runner.rs at the SplitTask dispatch (line 7399):
    The optional splitter path already exists; add a fallback that calls a new
    ResearchTree::split_step(plan: &SplitTaskPlan) -> Vec<SubTask>. The
    ResearchTree method uses the plan's parent_phase to locate the active node
    and returns plan.sub_tasks.clone() with corrected task_id sequencing. If
    ResearchTree is unavailable (task_runner has no tree handle at that call
    site), emit EscalationTrigger::SplitTaskFailed and escalate. Document the
    wiring path in task 15 decisions file.

13. **Populate metrics/phase_4G.toml** with 4G.6 runtime measurement keys.
    After smoke test (task 14) completes, write measured values into a new
    [runtime_measurements] section:
    auto_advance_accuracy = <measured — true positive auto-advances / total>
    false_escalation_rate = <measured>
    mode_switch_latency_ms = <measured from /mode command to gate mode flip>
    split_task_overhead_ms = <measured>
    substitute_model_swap_ms = <measured>
    end_to_end_autonomous_vs_interactive = <ratio>
    gate_evaluation_overhead_ms = <measured via tracing span>
    escalation_count_per_session = <count from smoke test SSB>
    completion = true

14. **Smoke test (cloud Sonnet 4.6, autonomous mode)**. Requires C-1 and C-5.
    Run: elai /task --mode autonomous "implement a 4-method calculator class
    with tests, in Rust"
    Assert conditions enumerated in ACCEPTANCE section below. Capture SSB
    rows and write measured latencies into task 13 fields.

15. **Decision doc**: write workflow/reasoning/C-9-decisions.md covering:
    (a) why can_fit_llm uses a trait default rather than requiring each impl;
    (b) SplitTaskPlan carries MergeStrategy::Sequential only — Parallel
    deferred to C-10; (c) cli/main.rs wiring passes None for interactive_gate
    because headless cloud tasks do not route through InterventionCard.

## WIRING_CONTRACT

produces_symbols:
  - AutoAdvanceEvaluator struct + evaluate() + evaluate_all()
  - EvidenceCheck struct + EvidenceVerdict enum
  - EscalationPolicy struct (anti_spiral_threshold, high_risk_pauses, max_auto_advances_per_phase)
  - SplitTaskPlan struct + SubTask struct + MergeStrategy enum
  - EscalationTrigger::SubstituteModelFailed + EscalationTrigger::SplitTaskFailed
  - ModelPool::can_fit_llm() (trait default method)
  - GateMode::from_cli_args()
  - ResearchTree::split_step()
  - [agents.{role}.backend] TOML section + BackendStanza struct in runtime_config.rs
  - metrics/phase_4G.toml [runtime_measurements] section
consumes_symbols:
  - Evidence::all_passed() (autonomous_gate.rs:284-298) — wrapped by evaluate_all()
  - EscalationTrigger enum (autonomous_gate.rs:374-397) — extended with 2 variants
  - AutonomyDecision::SplitTask (autonomous_gate.rs:421-424) — struct replaced
  - ModelPool::available_vram() (agent_pool.rs:50) — called by default can_fit_llm()
  - AutonomousGateConfig (autonomous_gate.rs:484-503) — gains escalation_policy field
  - TreeAutoAdvancePolicy (autonomous_gate.rs:125-173) — wired in cli/main.rs
  - task_runner.rs SplitTask dispatch (7345-7409) — updated to SplitTaskPlan
  - gate_decision SSB action_type (ssb.rs:1016) — consumed in smoke test assertion
  - escalation_fired SSB action_type (ssb.rs:544) — conditional smoke test check
reachable_from:
  - cli/main.rs TaskRunnerConfig construction
  - elai_cc.toml [agents.{role}.backend]
  - AutonomousGate::decide() internal dispatch
  - task_runner.rs SplitTask arm
  - MonitorWindow AutoAdvanceStats panel (reads gate_decision SSB rows — no Svelte change)
stub_ban:
  - AutoAdvanceEvaluator::evaluate() MUST NOT return EvidenceVerdict::Pass when
    Evidence::all_passed() would return false
  - can_fit_llm() MUST NOT always return true (defeats SUBSTITUTE_MODEL guard)
  - ResearchTree::split_step() MUST NOT return an empty Vec when SplitTaskPlan
    contains sub_tasks (escalate instead)
observable_effect:
  trigger: elai /task --mode autonomous "implement a 4-method calculator class
           with tests, in Rust" (cloud Sonnet 4.6)
  expect_ssb: >=1 gate_decision row with decision=AutoAdvance and
              source=AutoAdvanceEvaluator; escalation_fired row present only
              if a stagnation trigger occurred
  expect_artifact: metrics/phase_4G.toml [runtime_measurements] populated;
                   completion=true; MonitorWindow AutoAdvanceStats shows
                   non-zero last_5min_count

## DETERMINISTIC_CHECK

```bash
# 1. Named types present
rg "struct AutoAdvanceEvaluator|struct EvidenceCheck|enum EvidenceVerdict" \
  .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# 2. EscalationPolicy struct
rg "struct EscalationPolicy" .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# 3. SplitTaskPlan + SubTask + MergeStrategy
rg "struct SplitTaskPlan|struct SubTask|enum MergeStrategy" \
  .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# 4. SubstituteModelFailed + SplitTaskFailed variants
rg "SubstituteModelFailed|SplitTaskFailed" \
  .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# 5. can_fit_llm default method
rg "fn can_fit_llm" .elai_cc/crates/orchestrator/src/agent_pool.rs

# 6. cli/main.rs no longer passes None for autonomous_gate
rg "autonomous_gate: None" .elai_cc/crates/cli/src/main.rs
# Expected: zero matches (was deferred in phase_4G.toml notes line 165)

# 7. [agents.{role}.backend] in toml example
rg "agents\.executor\.backend|BackendStanza" \
  .elai_cc/elai_cc.toml.example .elai_cc/crates/cli/src/runtime_config.rs

# 8. ResearchTree::split_step exists
rg "fn split_step" .elai_cc/crates/orchestrator/src/research_tree.rs

# 9. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- autonomous 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- autonomous 2>&1 | grep "test result"

# 10. phase_4G.toml has runtime_measurements
rg "runtime_measurements|auto_advance_accuracy|escalation_count_per_session" \
  .elai_cc/metrics/phase_4G.toml
```

## ACCEPTANCE

- [ ] AutoAdvanceEvaluator struct with gate_checks: HashMap<FsmTransition,
      EvidenceCheck> present in autonomous_gate.rs; evaluate() returns
      EvidenceVerdict; evaluate_all() wraps Evidence::all_passed()
- [ ] EvidenceCheck struct (artifact_path, description, check_fn) and
      EvidenceVerdict enum (Pass, Fail { reason }, Missing) defined
- [ ] EscalationPolicy struct (4 fields) wired into AutonomousGateConfig
- [ ] SplitTaskPlan, SubTask, MergeStrategy defined; AutonomyDecision::SplitTask
      carries split_plan: SplitTaskPlan instead of bare fields
- [ ] EscalationTrigger has 6 variants (SubstituteModelFailed + SplitTaskFailed added)
- [ ] ModelPool trait has can_fit_llm() default method; SUBSTITUTE_MODEL path
      calls it before emitting SubstituteModel decision
- [ ] cli/main.rs passes a real AutonomousGate (not None) in TaskRunnerConfig
- [ ] [agents.executor.backend] + [agents.auditor.backend] stanzas in
      elai_cc.toml.example; BackendStanza parsed by runtime_config.rs
- [ ] ResearchTree::split_step() implemented; task_runner SplitTask arm calls it
- [ ] Smoke test: elai /task --mode autonomous "implement a 4-method calculator
      class with tests, in Rust" (cloud Sonnet 4.6) completes without prompting
      the user; >=1 SSB row has action_type=gate_decision, decision=AutoAdvance,
      source=AutoAdvanceEvaluator
- [ ] Smoke test: MonitorWindow AutoAdvanceStats.last_5min_count > 0
- [ ] Smoke test: mode toggle in Sidebar reflects AUTONOMOUS during run
- [ ] metrics/phase_4G.toml [runtime_measurements] populated with all 8 keys
      including escalation_count_per_session and completion=true
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- autonomous: all pass, count >= prior
- [ ] cargo test -p elai_cc_contracts -- autonomous: all pass
- [ ] Decision doc workflow/reasoning/C-9-decisions.md present (3 decisions)

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/autonomous_gate.rs
    (add AutoAdvanceEvaluator, EvidenceCheck, EvidenceVerdict, EscalationPolicy,
     SplitTaskPlan, SubTask, MergeStrategy; extend EscalationTrigger; update
     AutonomousGateConfig; refactor SplitTask variant; wire can_fit_llm check)
- MODIFY: .elai_cc/crates/orchestrator/src/agent_pool.rs
    (add can_fit_llm default method to ModelPool trait)
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs
    (add split_step() method)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs
    (update SplitTask match arm to use SplitTaskPlan; call ResearchTree::split_step;
     add SubstituteModelFailed + SplitTaskFailed escalation paths)
- MODIFY: .elai_cc/crates/cli/src/main.rs
    (wire AutonomousGate into TaskRunnerConfig; add GateMode::from_cli_args)
- MODIFY: .elai_cc/crates/cli/src/runtime_config.rs
    (add BackendStanza; extend Agents struct with per-role backend field)
- MODIFY: .elai_cc/elai_cc.toml.example
    (add [agents.executor.backend], [agents.auditor.backend],
     [agents.designer.backend] stanzas)
- MODIFY: .elai_cc/metrics/phase_4G.toml
    (add [runtime_measurements] section after smoke test)
- ADD: workflow/reasoning/C-9-decisions.md

## Next

C-10 (anti-spiral primitives — StagnationPattern, ThinkingPersona,
GenerationWatchdog, GoalDriftCanary) or C-11 depending on prioritization.
C-10 is the direct continuation of phase 4G.9 work; it is parallel-safe with
this session because it does not touch autonomous_gate.rs or task_runner.rs.
