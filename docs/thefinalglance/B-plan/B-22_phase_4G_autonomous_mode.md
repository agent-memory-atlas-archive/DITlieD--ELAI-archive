# B-22: Phase 4G — Autonomous Mode Enablement
Objective: implement Phase 4G per spec — AutonomousGate wraps
InteractiveGate; AutoAdvanceEvaluator with 6 evidence checks;
SUBSTITUTE_MODEL via ModelPool swap chain; SPLIT_TASK on context
budget Terminal; circuit breaker (>10 auto-advances).
Estimated effort: 2-3 sessions.
Pre-requisites: B-2 (FSM live), B-6 (evidence gates surfaced),
B-21 (FeasibilityProbe), B-15..B-20 (production hardening),
B-36 Session 2+ (V2 dashboard MonitorWindow wired — for AutoAdvanceStats
surfacing).
Parallel-safe with: B-23..B-28.

## V2 REBASELINE (2026-05-07) — dashboard surfaces target V2

The orchestrator-side AutonomousGate / AutoAdvanceEvaluator changes are
V2-agnostic — land them as written. Dashboard surface changes target V2:

  V1                                       V2
  ---------------------------------------------------------------------
  crates/dashboard/src/commands.rs       -> crates/dashboard-v2/src-tauri/
                                             src/lib.rs (B-36 wires
                                             get_auto_advance_stats —
                                             stub already exists in
                                             stub.ts at line 220-225;
                                             swap to real invoke after
                                             auto_advance.rs lands)
  V1 MonitorWindow.svelte                -> crates/dashboard-v2/src/lib/
                                             windows/MonitorWindow.svelte
                                             (already scaffolded —
                                             stub-backed via stub.ts;
                                             B-22 wires the live data
                                             once the orchestrator
                                             producer ships)
  V1 AutoAdvanceStats panel              -> Render inside V2's
                                             MonitorWindow.svelte using
                                             stub.get_auto_advance_stats()
                                             until B-22 + B-36 swap to
                                             live invoke

V2 settings UI for `--local|--api|--hybrid` and `/model` lives in
`crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte` (NOT a
separate Settings.svelte). Persistence config goes through V2's stub
`set_model_routing(role, target)` (stub.ts:241) which becomes a Tauri
invoke as part of B-22 (or earlier if B-36 covers it first).

Coordinate `model_swap` SSB writes with B-36 — the stub
`get_inference_metrics` (stub.ts:526) is the V2 surface for tokens/sec
+ swap history.

## Context

Per spec phase-4G.txt and phase-4-summary.md PHASE 4G (8 sub-steps):
- AutonomousGate wraps InteractiveGate using DispatchPipeline /
  PlanDispatchTable types: checks evidence artifacts first, delegates
  to human only on HIGH-risk gate, missing evidence, or escalation
  trigger
- 6 evidence checks (one per gate from S13):
  1. knowledge readiness
  2. zero UNGROUNDED claims
  3. tests fail correctly
  4. test flipped
  5. zero BLOCKs
  6. DKG+changelog present
- Escalation triggers: anti-spiral (3x), HIGH risk, ReplanRequest(MAJOR),
  circuit breaker (>10 auto-advances)
- SPLIT_TASK on context budget Terminal: max 4 sub-tasks, max depth 2
  (ties to Phase Sizing), sequential merge
- SUBSTITUTE_MODEL on max_restarts: iterate fallback_chain via ModelPool
  swap protocol
- Mode switch (/mode autonomous|interactive) takes effect at next gate.
  Per-session, not global.
- 4G.7-4G.8 EXTENSION (Tree-Level Autonomous Advance):
  - Auto-advances at baby step granularity within DecompositionTree
  - Risk levels per decomposition level: BabyStep=LOW, Step=LOW,
    Phase=MEDIUM, Concept=HIGH
  - Concept-level bubble-up always requires human (HIGH risk)
  - Circuit breaker: max 50 auto baby steps per concept before forced
    human check
  - TreeAutoAdvancePolicy type
- 36 tests (28 contract + 8 adversarial)
- MODEL SELECTION UI (from 3J.LOCAL):
  - --local: sidecar single model
  - --hybrid: per-role routing
  - --api: cloud only
  - /model command at runtime
  - elai_cc.toml [agents.{role}.backend] = "local" | "api" | "auto"

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4G.txt
- summary/phase-4-summary.md PHASE 4G
- crates/orchestrator/src/gate.rs (existing InteractiveGate)
- crates/orchestrator/src/evidence.rs (7 EvidenceGate impls — wire to checks)

## Tasks

1. **AutonomousGate** wraps InteractiveGate via DispatchPipeline /
   PlanDispatchTable. Checks evidence FIRST; only delegates to human if
   HIGH risk / missing evidence / escalation trigger.

2. **6 Evidence checks** mirror gates S13. Implement check functions
   per gate; AutoAdvanceEvaluator runs all 6 before auto-advancing.

3. **Escalation triggers** wired:
   - anti-spiral (3x same step) — couples with B-6 SpiralTracker
   - HIGH risk — per-transition risk classification
   - ReplanRequest(MAJOR) — per B-8 inline feasibility
   - circuit breaker (>10 auto-advances within session)

4. **SPLIT_TASK** on context budget Terminal: max 4 sub-tasks, depth 2
   (ties to Phase Sizing). Sequential merge.

5. **SUBSTITUTE_MODEL** on max_restarts: iterate fallback_chain via
   ModelPool swap protocol. Emit `model_swap` SSB row (B-3 dead UI #5
   becomes live).

6. **Mode switch** (/mode autonomous|interactive) per-session metadata.
   Effect at next gate.

7. **TreeAutoAdvancePolicy** for 4G.7-4G.8: per-baby-step auto-advance
   thresholds. Risk levels per level. 50-baby-steps-per-concept circuit
   breaker.

8. **Model Selection UI**: dashboard Settings panel exposes
   --local|--api|--hybrid; /model command parses runtime swap. Reads
   elai_cc.toml [agents.{role}.backend].

9. **AutonomousGate dashboard surface**: MonitorWindow (B-26) shows live
   auto-advance count, last escalation, circuit breaker remaining,
   per-tree-level advance counts.

## WIRING_CONTRACT

produces_symbols:
  - AutonomousGate
  - AutoAdvanceEvaluator
  - TreeAutoAdvancePolicy
  - SplitTaskPlan
  - SubTask
  - AutonomyDecision
  - GateDecisionPayload
  - ModelSwapPayload
  - TaskSplitPayload
  - crates/cli/src/commands.rs::SlashCommand::Mode
  - crates/cli/src/commands.rs::SlashCommand::Model
consumes_symbols:
  - ModelPool
  - crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte
reachable_from:
  - crates/cli/src/run_subcommand.rs::run_async
  - crates/dashboard-v2/src-tauri/src/lib.rs::setup
stub_ban:
  - AutoAdvance running indefinitely (no circuit breaker) disallowed
  - HIGH-risk auto-advance without human disallowed
  - Concept-level auto-bubble-up without human disallowed
frontend_pair: dashboard MonitorWindow + AutoAdvanceStats panel
                (already built per audit; activate via feature flag)
observable_effect:
  trigger: cargo run --manifest-path F:\.ELAI_workflow\.elai_cc\Cargo.toml -p elai-cli -- run --mode autonomous --project-root F:\.ELAI_workflow\.elai_cc
  expect_ssb: gate_decision
  expect_artifact: MonitorWindow shows live auto-advance count + circuit
                   breaker gauge

## DETERMINISTIC_CHECK

```bash
# 1. AutonomousGate type exists
rg "struct AutonomousGate" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit.

# 2. 36 tests
cargo test --package elai-orchestrator autonomous 2>&1 | grep "test result"
# Expected: ≥36 tests.

# 3. /mode command
rg "SlashCommand::Mode" .elai_cc/crates/cli/src/commands.rs
```

## ACCEPTANCE

- [ ] AutonomousGate wraps InteractiveGate
- [ ] 6 evidence checks per gate
- [ ] All 4 escalation triggers wired
- [ ] SPLIT_TASK works (max 4 / depth 2)
- [ ] SUBSTITUTE_MODEL works via ModelPool swap chain
- [ ] /mode + /model CLI commands functional
- [ ] dashboard model selection UI
- [ ] TreeAutoAdvancePolicy enforces per-level risk
- [ ] Circuit breaker (>10 auto-advance / 50 baby steps per concept)
- [ ] MonitorWindow surfaces AutoAdvanceStats
- [ ] 36+ tests pass

## Files touched

- ADD: crates/orchestrator/src/autonomous_gate.rs
- ADD: crates/orchestrator/src/auto_advance.rs
- MODIFY: crates/orchestrator/src/gate.rs (wraps with autonomous)
- MODIFY: crates/orchestrator/src/task_runner.rs (auto-advance loop)
- MODIFY: crates/cli/src/commands.rs (/mode, /model)
- MODIFY: elai_cc.toml schema [agents.{role}.backend]
- MODIFY: crates/dashboard/src/commands.rs + Settings/MonitorWindow
- CREATE: workflow/reasoning/B-22-decisions.md

## Follow-up: B-22b (4G.9 stagnation+personas+watchdog, ouroboros port)

NOT IN ORIGINAL B-22 SCOPE. B-22 shipped per s-26716 (2026-05-09) with
the lone "3x same GRC step -> escalate" anti-spiral. Phase 4G has been
extended with sub-step 4G.9 to subsume that rule under a 4-pattern
detector + 5-persona unstuck path + GenerationWatchdog (ouroboros port,
MIT-licensed). Work to do in B-22b:

1. **4 stagnation patterns** (replace lone "3x same step" rule):
   - `Spinning` (3+ identical phase_outputs OR error_signatures, blake3-hashed)
   - `Oscillation` (period-2 alternation, 4+ outputs needed)
   - `NoDrift` (drift score |delta| < 0.01 for 3 iterations)
   - `DiminishingReturns` (improvements < 0.01 AND monotonically decreasing)
   Stateless `StagnationDetector` over `ExecutionHistory`.
   Existing "3x same step" rule becomes the `Spinning` case.

2. **5 lateral-thinking personas + affinity table**:
   - Hacker → [Spinning]
   - Researcher → [NoDrift, DiminishingReturns]
   - Simplifier → [DiminishingReturns, Oscillation]
   - Architect → [Oscillation, NoDrift]
   - Contrarian → [all]
   `suggest_persona_for_pattern(pattern, exclude)` returns first match by
   priority. On exhaustion: emit `lateral_thinking_exhausted` and
   escalate per existing EscalationPolicy.

3. **Parallel multi-persona fan-out** (optional flag): when configured,
   spawn ALL 5 personas in parallel via Tier C with FRESH context per
   persona (prevents anchoring bias). First-success wins.

4. **GenerationWatchdog** per baby step:
   - `idle_timeout` (60s default — no SSB events at all)
   - `no_material_progress_timeout` (180s default — events flowing but
     no material event)
   - `safety_timeout` (600s default — wall-clock cap)
   Material event types whitelisted: `phase_transition`,
   `evidence_gate_evaluated`, `integration_checkpoint_success`,
   `bubble_up_typed`, `agent_lifecycle` (state change only). Heartbeats
   and status echoes excluded. `workflow_fingerprint` dedup prevents
   fingerprint-replay gaming the watchdog.

5. **5 persona agent files** at `.claude/agents/persona-*.md`:
   persona-hacker.md, persona-researcher.md, persona-simplifier.md,
   persona-architect.md, persona-contrarian.md. Format consistent with
   existing agents (auditor.md, context-compiler.md, etc.).

6. **Goal-drift canary** (partial port of ouroboros drift.py — `goal_drift`
   only). Cheap pre-audit sanity check on baby step output:
   - `drift = 1 - jaccard(tokens(output), tokens(baby_step.goal))`
     using lowercase alphanumeric tokenization on whitespace splits.
   - 3 verdict bands: `< 0.5` Pass · `0.5–0.7` Warning (advisory SSB row,
     continue to audit) · `>= 0.7` Block (fail before audit, BubbleUp
     RetryBabyStep, max 1 retry).
   - Skip when output token count `< 20` or goal is empty.
   - Saves at least one audit LLM call per Block trigger (measured in
     4G.6 metrics).
   - Constraint_drift and ontology_drift components from ouroboros are
     NOT ported (redundant with T1/T2/T3 hooks and 4H.7B OSC respectively).
   - Source: ouroboros `src/ouroboros/observability/drift.py:125-156`
     (calculate_goal_drift) + `:217-229` (_tokenize). MIT.

7. **Source**: github.com/Q00/ouroboros, MIT-licensed. Ports:
   `src/ouroboros/resilience/stagnation.py`,
   `src/ouroboros/resilience/lateral.py`,
   `src/ouroboros/evolution/watchdog.py`,
   `src/ouroboros/observability/drift.py` (goal_drift component only),
   CHANGELOG "Parallel multi-persona dispatch" entry (2026-05-08).

Estimated effort: 1 session for the patterns + watchdog, +1 for the
parallel fan-out + persona files + goal-drift canary. Total ~2 sessions.

## Next

B-23 (Phase 4H Self-Modification — now extended with OSC/REG/EVO).
