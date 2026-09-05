C-10 Phase 4G.9 Anti-Spiral Primitives — Decisions

Session: s-32952
Date: 2026-05-18
Plan: docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md
Context: workflow/sessions/s-32952/context-package-C10.md

This file pins the orchestrator-confirmed decisions that resolve the 7
DRIFT items captured in the context package. Each decision is
load-bearing for either the compile path, the SSB schema, or the
dashboard wiring — downstream changes that revisit any of these MUST
update this file in lockstep.

==========================================================================
DECISION D1 — Persona agent file front-matter uses CSV `tools:` form
==========================================================================

Source of disagreement: plan §Task 6 specifies
`tools: [Read, Grep, Glob, Bash, WebSearch]` (JSON-array form).
Existing repository agents (auditor.md, memory-writer.md,
spec-grounder.md, sidecar-health.md) use the CSV form:
`tools: Read, Grep, Glob, Bash, WebSearch`.

Shipped: CSV form (matches repository convention; the harness loader
parses the CSV variant). All 5 persona files
(persona-{hacker,researcher,simplifier,architect,contrarian}.md) use
the same CSV header layout.

Why: the plan's JSON-array wording is an editorial mistake — every
agent in .claude/agents/ uses CSV. Switching the parser to accept
JSON would be a larger scoped change than fixing the 5 files. CSV is
the de facto standard.

==========================================================================
DECISION D2 — Stagnation SSB variant coverage
==========================================================================

Spec §4G.9 task A enumerates 7 SSB action_types for the full
stagnation/lateral-thinking taxonomy. C-2 (already shipped) defined 5:
  StagnationSpinningDetected, StagnationOscillationDetected,
  LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout

C-10 plan §Task 2 (DRIFT-2 resolution) proposed piggybacking NoDrift /
DiminishingReturns onto the 2 existing stagnation variants via the
`pattern` field already on StagnationSpinningDetectedPayload +
StagnationOscillationDetectedPayload.

Shipped: SCHEMA_VERSION bumped 51 -> 52 atomically with **3 NEW
variants**:
  - LateralThinkingSucceeded — persona-switch broke the loop.
  - LateralThinkingExhausted — all 5 personas tried without break.
  - GoalDriftBlock — companion to GoalDriftWarning for the Red band.

Existing 5 C-2 variants kept verbatim. The Spinning/Oscillation row
type carries NoDrift/DiminishingReturns via the payload `pattern`
field per plan §Task 2:
  - Spinning + NoDrift -> StagnationSpinningDetected (pattern
    discriminates).
  - Oscillation + DiminishingReturns -> StagnationOscillationDetected
    (pattern discriminates).

Atomic count update: all_test_payloads grows 103 -> 106 (3 new C-10
payloads). 9-tuple lockstep across:
  1. SsbActionType enum (3 new arms)
  2. SQL CHECK constraint in CREATE_SSB_TABLE (3 new strings)
  3. SsbActionType::as_sql_str() match (3 new arms)
  4. ssb_action_type_from_str() match (3 new arms)
  5. SsbPayload enum (3 new variants)
  6. SsbPayload::action_type() match (3 new arms)
  7. Payload structs (LateralThinkingSucceededPayload,
     LateralThinkingExhaustedPayload, GoalDriftBlockPayload)
  8. test_builder_dispatch test-helper match (3 new arms)
  9. all_test_payloads() vec (3 new entries) + SCHEMA bump 51 -> 52
     + count 103 -> 106 in assert_eq! + descriptive panic string.

==========================================================================
DECISION D3 — GoalDriftCanary verdict semantics (Jaccard direction)
==========================================================================

Source of disagreement: spec §4G.9 task E defines `drift = 1.0 -
jaccard` (divergence direction; higher = more drift; threshold
inversion: Block ≥ 0.7).

Plan §Task 1 specifies `Jaccard direct` (similarity direction; higher
= aligned; thresholds Green ≥ 0.7, Yellow ≥ 0.4).

Shipped: BOTH directions are isomorphic — picking one is a labeling
choice. Following the **plan's similarity-direction reading** for the
in-Rust verdict bands:
  - score >= 0.7 -> DriftVerdict::Green
  - 0.4 <= score < 0.7 -> DriftVerdict::Yellow (emits
    `goal_drift_warning`)
  - score < 0.4 -> DriftVerdict::Red (emits `goal_drift_block`)

Field-name drift: GoalDriftWarningPayload.drift_score is named for the
divergence direction but carries the Jaccard *similarity*. We do NOT
rename — the C-2 SCHEMA is frozen. Doc-comment on
GoalDriftCanary::compute() spells this out so downstream consumers
read the value through the verdict bands rather than the field name.

If a future patch wants to migrate to the divergence direction, both
the SsbPayload struct field AND every consumer (Tauri command + Svelte
StagnationSection + this code) must flip in lockstep — non-trivial
ripple.

==========================================================================
DECISION D4 — Persona affinity table (3 conflicting sources)
==========================================================================

Source A (plan §Task 1):
  Spinning -> Contrarian, Oscillation -> Architect, NoDrift -> Hacker,
  DiminishingReturns -> Simplifier. Researcher fallback rotation.

Source B (phase-4G.txt §4G.9 task B): multi-affinity priority table
that resolves to:
  Spinning -> Hacker, Oscillation -> Simplifier,
  NoDrift -> Researcher, DiminishingReturns -> Researcher.

Source C (.claude/skills/anti-spiral/SKILL.md):
  Spinning -> Simplifier, Oscillation -> Architect,
  NoDrift -> Researcher, DiminishingReturns -> Contrarian.

Shipped: **Source A (plan)**. The plan is the executable spec for
C-10. suggest_persona_for_pattern() implements Source A verbatim. Test
assertions test_persona_affinity_* pin Source A.

Why: Source B is the legacy ouroboros multi-affinity table with
priority arbitration — over-engineered for the runtime; Source A is
single-affinity, deterministic, no priority arbitration. Source C is
the Claude-Code human-operator skill protocol — it does not bind the
orchestrator runtime; can be re-aligned in a future doc-only patch.

==========================================================================
DECISION D5 — AdaptiveFutilityTracker placement (TaskRunnerState absent)
==========================================================================

Plan §Task 5 writes "Per-task instance in TaskRunnerState." but no
struct named TaskRunnerState exists in task_runner.rs (DRIFT-5).
Closest analogues: TaskRunnerConfig (line 2261) + ResolvedConfig (line
2618). The active `spiral_tracker: Option<SpiralTracker>` (line 4967)
is a STACK-LOCAL inside drive_task — not a config field.

Shipped: stack-local placement at task_runner.rs:4967 area —
`let mut adaptive_futility = AdaptiveFutilityTracker::new();` alongside
spiral_tracker. No new ResolvedConfig field; no breaking change to
TaskRunnerConfig.

Why: lifetime is scoped to a single task — matches the design intent
("Per-task instance"). The tracker doesn't need to be visible to the
gate (futile classes short-circuit at the caller site via local
escalation, not via GateContext).

==========================================================================
DECISION D6 — (informational, no shipping change)
==========================================================================

Plan §Task 8 lists 17 functions; §Task 9 lists 5; total = 22. C-10
ACCEPTANCE bullet "17 anti_spiral_extended + 5 gate_integration
contract tests pass" confirms. DETERMINISTIC_CHECK bullet 6 "Contract
tests pass (>= 22)" confirms. Shipped as a single file
c10_anti_spiral_contract.rs containing all 22 tests (file count
combined for atomic compile + run).

==========================================================================
DECISION D7 / DECISION-PLACEMENT — Detector wired at task_runner, not in gate
==========================================================================

Source of conflict: plan §Task 2 says "Wire StagnationDetection into
autonomous_gate.rs decide()". But decide()'s signature is
`async fn decide(&self, ctx: &GateContext) -> AutonomyDecision`
— immutable `&GateContext`. Setting suggested_prompt_prefix inside
decide() requires either:
  (a) signature change to `&mut GateContext` — ripples through 15+
      contract test callers + every production call site.
  (b) interior mutability on AutonomousGate — heavyweight for a
      single-call-per-step state.

Shipped: **Option-1 (caller-site detection)**. The task_runner
caller (drive_task at task_runner.rs:4980-5660 area) runs:
  1. push AttemptResult onto ExecutionHistory
  2. GenerationWatchdog::check()
  3. StagnationDetection::detect_pattern()
  4. emit stagnation + LateralThinkingActivated SSB rows
  5. emit GoalDriftWarning / GoalDriftBlock
  6. populate gctx.execution_history / attempt_index /
     suggested_prompt_prefix
  7. OR pattern.is_some() and watchdog_fired into
     gctx.anti_spiral_fired so the existing AntiSpiral short-circuit
     in decide() still fires.

decide()'s signature is unchanged (zero existing-caller breakage).
The new GateContext fields default to empty/None values so the 32+
existing contract-test ctx_at(...) helpers compile with a 3-line
mechanical addition.

==========================================================================
DECISION D8 — Existing anti_spiral.rs is NOT replaced (two-tier coexistence)
==========================================================================

The existing anti_spiral.rs operates at the GRC reasoning-step level,
detects only Spinning, ring=4, threshold=3, and is invoked from
task_runner via SpiralTracker. C-10's new module
anti_spiral_extended.rs operates at the task-runner step level
(per-baby-step), 4 patterns, with persona + canary + watchdog.

Shipped: both coexist. anti_spiral.rs is unchanged. The new module is
added as a sibling. GateContext.anti_spiral_fired carries the OR of:
  - SpiralTracker.hard_stop() (GRC step level)
  - StagnationDetection::detect_pattern returns Some (baby-step level)
  - GenerationWatchdog timeout

==========================================================================
Determinism guarantees
==========================================================================

- GoalDriftCanary uses BTreeSet (NOT HashSet) for token set
  comparison so Jaccard scores are reproducible.
- GenerationWatchdog fingerprints via blake3 (workspace dep confirmed
  in orchestrator Cargo.toml line 64).
- Persona affinity is a static match (no rand, no hash iteration).
- StagnationDetection rule precedence is total-ordered (NoDrift first,
  then Spinning, Oscillation, DiminishingReturns) — first match wins.
- No .unwrap() / .expect() in anti_spiral_extended.rs or any C-10
  task_runner addition.

==========================================================================
Files touched (final list)
==========================================================================

ADDED:
  .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs
  .elai_cc/crates/contracts/tests/c10_anti_spiral_contract.rs
  .claude/agents/persona-hacker.md
  .claude/agents/persona-researcher.md
  .claude/agents/persona-simplifier.md
  .claude/agents/persona-architect.md
  .claude/agents/persona-contrarian.md
  workflow/reasoning/C-10-decisions.md (this file)

MODIFIED:
  .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA 51 -> 52, +3 variants, +3 payloads, atomic 9-tuple lockstep)
  .elai_cc/crates/orchestrator/src/lib.rs
    (+anti_spiral_extended module, +re-export block, +3 payload exports)
  .elai_cc/crates/orchestrator/src/autonomous_gate.rs
    (+3 GateContext fields, updated Debug impl, in-module ctx_with)
  .elai_cc/crates/orchestrator/src/task_runner.rs
    (stack-local detector instances at line 4967 area; pre-gate
     detection block; gate-context build site populates the 3 new fields)
  .elai_cc/crates/contracts/tests/autonomous_gate_contract.rs (ctx_at)
  .elai_cc/crates/contracts/tests/phase_4g_extended_contract.rs (ctx_at)
  .elai_cc/crates/contracts/tests/c9_autonomous_infra_contract.rs (ctx_at)
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
    (get_stagnation_events Tauri command + StagnationEventDto)
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (invoke_handler entry)
  .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte
    (StagnationSection card, mirror of AMBIGUITY pattern)
  .elai_cc/metrics/phase_4G.toml ([stagnation] section)
