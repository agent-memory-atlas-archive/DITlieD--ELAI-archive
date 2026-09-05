# Anti-Spiral Lateral Thinking (C-10)

- **ELAI ids:** C-10 / phase-4G.9 / B-22b / C-003/E-C10-001..E-C10-045 / P4G-E029 / B0-068
- **Spec status (as designed):** PARTIAL (feature-map 1.3: 5 action_types fire in `drive_task`; `phase_4G.toml` `[stagnation]` missing); original plan PLANNED after C-9/C-2
- **Crate / module (spec):** `crates/orchestrator/src/anti_spiral_extended.rs`; wire `autonomous_gate.rs::decide()`, `task_runner.rs` step boundaries; 5 files `.claude/agents/persona-*.md`; `OrchestrationDrawer` `StagnationSection`
- **Purpose of this file:** enough architecture to rebuild 4G.9 stagnation detection, five-persona lateral switch, Jaccard goal-drift canary, generation watchdog, futility tracker. Not a status ledger.

## 1. What it is

**Anti-spiral primitives** are the Phase-4G.9 detection machinery for **REPETITION** in the autonomous `/task` loop: a 4-pattern `StagnationDetection` over an `ExecutionHistory` ring, a deterministic persona switch among 5 `ThinkingPersona`s, a Jaccard `GoalDriftCanary`, a blake3 `GenerationWatchdog`, and an `AdaptiveFutilityTracker`. C-2 already defined SSB variants with **zero emit sites**; `EscalationTrigger::AntiSpiral` existed with no detector. C-10 **is** that detector (`C-plan/C-10_anti_spiral_primitives.md:1-26`, `ELAI-SYSTEMS-OVERVIEW.md:107-111`).

**Neighbor, do not author:** GRC-v3 **overthinking guard (J-85)** catches chain **DRIFT** off a correct answer (oscillation/hesitation/declining confidence). Anti-spiral catches **REPETITION** (same hash, same error class, alternating error pair, diminishing error counts). Feature-map: J-85 is “DISTINCT from anti-spiral (which catches REPETITION; grep `oscillation` hit only `anti_spiral_extended.rs` + telemetry)” (`ELAI-FEATURE-MAP.md:668-677`). Do not implement J-85 in this organ.

## 2. Why it exists

Phase-3C anti-spiral is a **3 consecutive identical blake3 step-hash** stop (`ELAI-SPEC-OVERVIEW.md:82-83`, `ELAI-SYSTEMS-OVERVIEW.md:107-108`). 4G.9 **subsumes** that 3× rule as `Spinning` (threshold=3), it does not remove it (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` P4G-E029, PHASE-002/IX-013). B-22b is the ouroboros port brief: 4-pattern detector + 5-persona lateral thinking + watchdog + Jaccard canary (`B-plan/B-PLAN-SUMMARY.md:424-431`). Without C-10, autonomous mode escalates on a named trigger with **no pattern, no persona, no journal**. If faked (prompt “try a different approach”), the loop keeps editing the same file. Skill-level `.claude/skills/anti-spiral.md` is Claude-Code; C-10 ports the logic into **Rust** for `/task` (`C-plan/C-10_anti_spiral_primitives.md:25-26`).

## 3. How it works (runtime)

1. **Who calls it.** `elai /task --mode autonomous` → every `autonomous_gate.rs::decide()`; `task_runner.rs` baby-step completion + heartbeat (`C-plan/C-10_anti_spiral_primitives.md:205-208`).
2. **History.** `GateContext.execution_history: ExecutionHistory` (default empty), `attempt_index: u32` (default 0), `suggested_prompt_prefix: Option<&'static str>` (default None). Existing callers stay valid via struct-update (`:88-90`).
3. **Detect (deterministic, ordered).** After the existing `EscalationTrigger::AntiSpiral` check, `StagnationDetection::detect_pattern(&history) -> Option<StagnationPattern>`. If `len < 2` → `None`. Else **in order**:
   1. **NoDrift** — last 2 `output_hash` equal
   2. **Spinning** — last `N=spin_threshold` `error_class` identical
   3. **Oscillation** — last 4 entries alternate between 2 strings
   4. **DiminishingReturns** — `error_count` improvement `< 5%` per step over a 4-attempt window
   (`:54-58`, exhaustive C-003/E-C10-006..010)
4. **Persona (deterministic affinity).** `suggest_persona_for_pattern`:
   - Spinning → **Contrarian**
   - Oscillation → **Architect**
   - NoDrift → **Hacker**
   - DiminishingReturns → **Simplifier**
   - **Researcher** = fallback rotation (`index % 5 == 4`)
   (`:64-66`, C-003/E-C10-013..014)
5. **Emit + attach.** On `Some(pattern)`: emit stagnation SSB (`StagnationSpinningDetected` or `StagnationOscillationDetected`; payload `pattern` disambiguates NoDrift / DiminishingReturns); emit `LateralThinkingActivated` with `new_persona`; set `suggested_prompt_prefix` from `PERSONA_STRATEGIES` (`:92-96`). Next baby-step prompt **contains** that prefix (`:223`).
6. **GoalDriftCanary (step boundary).** Store `original_task_tokens` at task start. After each baby-step: Jaccard vs current subtask text. `DriftVerdict::Red` → `GoalDriftWarning` with `jaccard_score`, original/current snippets (first 80 chars) (`:99-104`, C-003/E-C10-026). This is **instruction-text** drift vs the original task, **not** J-85 answer-drift.
7. **GenerationWatchdog (heartbeat).** Per-task `GenerationWatchdog::default()`. `check(last_event_ms, now_ms, current_fingerprint, start_ms)`. Precedence **Safety > NoMaterialProgress > Idle**. On Some: emit `WatchdogTimeout` `{kind, elapsed_ms, fingerprint_hex}`; set `AutonomyDecision::Escalate(EscalationTrigger::AntiSpiral)` (`:106-111`, C-003/E-C10-021, E-C10-027). Fingerprint: blake3 of 16-byte concat `(ssb_row_count:u64, output_hash:u64)`, first 8 bytes as `u64`; crate absence = hard-error (`:75-77`).
8. **AdaptiveFutilityTracker.** After each baby-step `record(error_class, succeeded)`. Before repair-loop iteration: `is_futile(error_class, 0.15)` → skip remaining retries; escalate `CircuitBreaker` with **serialized tracker payload** on an SSB row (must not silently drop) (`:113-118`, `:85`, C-003/E-C10-028).
9. **Dashboard.** `get_stagnation_events` newest-first for the five action_types; `StagnationSection` badges: amber stagnation, blue lateral, orange goal_drift, red watchdog (`:136-141`, C-003/E-C10-033..034).
10. **Fail-closed.** No `.unwrap()` on new paths. Jaccard **must** use `BTreeSet` (not `HashSet`). Watchdog **must** use blake3. Persona files must match existing agent front-matter (`:211-216`).

The model may execute the **persona prompt**. It may **not** classify the pattern, pick the persona, or suppress emits.

## 4. Types, enums, objects

Copy from `C-plan/C-10_anti_spiral_primitives.md:44-85`. Do not invent fields.

```
StagnationPattern { Spinning, Oscillation, NoDrift, DiminishingReturns }
  // Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize

AttemptResult {
  attempt_index: u32,
  error_class: String,
  output_hash: u64,
  error_count: u32,
  timestamp_ms: u64,
}

ExecutionHistory                        // VecDeque<AttemptResult>, capacity 32
  push(), is_full(), last_n(), clear()  // stdlib only; push-past-capacity evicts

StagnationDetection {
  spin_threshold: 3,
  oscillation_window: 4,
  diminishing_min_window: 4,
  diminishing_min_improvement_pct: 5.0f32,
}
  detect_pattern(history) -> Option<StagnationPattern>

ThinkingPersona { Hacker, Researcher, Simplifier, Architect, Contrarian }

PersonaStrategy { persona: ThinkingPersona, prompt_prefix: &'static str }
PERSONA_STRATEGIES: [PersonaStrategy; 5]
suggest_persona_for_pattern(pattern) -> ThinkingPersona

GoalDriftCanary { green_threshold: 0.7f32, yellow_threshold: 0.4f32 }
  tokenize: whitespace-split, strip non-alphanumeric, lowercase
  compute: Jaccard via BTreeSet (NOT HashSet)
  verdict -> DriftVerdict { Green, Yellow, Red }

WatchdogTimeoutKind { Idle, NoMaterialProgress, Safety }
GenerationWatchdog {
  idle_timeout_ms: 120_000,             // 2 min
  no_progress_timeout_ms: 600_000,      // 10 min
  safety_timeout_ms: 1_800_000,         // 30 min
  workflow_fingerprint: Option<u64>,
}

AdaptiveFutilityTracker {
  success_count: HashMap<String,u32>,
  attempt_count: HashMap<String,u32>,
}
  unseen class prior 0.5
  is_futile: rate < threshold AND attempts >= 4
```

**Five personas (instructions, ~30 lines each, `model: sonnet`, tools Read/Grep/Glob/Bash/WebSearch)** (`:120-132`, C-003/E-C10-029..032):

| Persona | Job |
|---|---|
| Hacker | 3 shortest paths DIRTY/CLEAN/HYBRID; one concrete baby_step; ignore elegance |
| Researcher | 3 web searches for prior art; best reference + next action |
| Simplifier | strip to minimal skeleton; add layers until failure reappears; name the bad layer |
| Architect | map dependency graph of oscillating error pair; decoupling baby_step |
| Contrarian | state opposite assumption; alternative baby_step; **no hedging** |

## 5. Control flow / state machine

**Owner of the loop:** `decide()` + task_runner hooks. Detection is **code**, not the model.

**Pattern → persona (rebuild this table, not the skill-capability table):**

| Pattern | Meaning (REPETITION) | Persona |
|---|---|---|
| NoDrift | last 2 hashes equal | Hacker |
| Spinning | last 3 error_class identical | Contrarian |
| Oscillation | last 4 alternate 2 strings | Architect |
| DiminishingReturns | error_count improvement < 5% / step over 4 | Simplifier |
| (fallback) | `index % 5 == 4` | Researcher |

**Watchdog order:** Safety > NoMaterialProgress > Idle (`:79`).

**Escalation split:** watchdog → `Escalate(AntiSpiral)`; futility → `Escalate(CircuitBreaker)` (C-003/DELTA!-C10-2).

**Forced:** 3 identical errors → spinning row `iteration=3` + lateral-thinking row (`:221-222`).

## 6. Grounding, receipts, gates

**Receipts:** `AttemptResult` hashes and error classes in the ring; Jaccard over token sets; blake3 fingerprint; SSB rows listed in §11. Pattern is **computed**, never self-reported.

**Never allowed as stagnation / persona evidence:** LLM “I am looping”; prompt-only persona; `HashSet` Jaccard; blake2/sha2 fingerprint.

**J-85 is not a receipt here.** Do not use overthinking-guard oscillation `r=0.78` as C-10 `Oscillation` (that is answer-drift, different organ).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| history ring | 32 | per gate context |
| spin_threshold | 3 (feeds C-9 `anti_spiral_threshold`) | detect |
| oscillation_window | 4 | detect |
| diminishing window / pct | 4 / 5.0 | detect |
| Jaccard green / yellow | `0.7` / `0.4` (Red below yellow) | per step |
| idle / no-progress / safety | 120s / 600s / 1800s | per task |
| futility | rate `< 0.15` and attempts `≥ 4` | per error_class |
| persona files | 5 × ~30 lines | `.claude/agents/` |
| tests | 17 extended + 5 integration; no regress ≥14 + ≥18 | contracts |
| max persona switches | **UNCERTAIN** in C-10 (skill says max 2/phase — neighbor skill, not this module) | |

**B-22b conflict (do not blend):** B-22b lists idle 60s / no-progress 180s / safety 600s and Jaccard bands 0.5/0.7 (`B-plan/B-PLAN-SUMMARY.md:427-430`). **Rebuild C-10 numbers** (`C-plan/C-10_anti_spiral_primitives.md:67-74`).

## 8. Roles and neighbors

**Consumes:** C-2 `SsbActionType::{StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout}` + emit helper; C-9 `EscalationPolicy.anti_spiral_threshold`; `EscalationTrigger::AntiSpiral`; task_runner hooks (`:198-203`). **Produces:** the 11 types in acceptance (`:275-277`) plus PERSONA_STRATEGIES + functions (exhaustive DELTA!-C10-3).

**B-22b:** predecessor brief / ouroboros port; C-10 is the implementation plan (`ELAI-FEATURE-MAP.md:758-765`).

**E-46:** **widens** the canary to consecutive **Phase** pairs with Jaccard **and** cosine vs Idea embedding. C-10 canary is original-task vs subtask at baby-step. Do not merge the two (`E-plan/E-46_planning_decomposition.md:4`, `:43`).

**I-32:** drop narrative personas on executor/researcher **input**; **KEEP** for auditor/critic/**anti-spiral** (`ELAI-FEATURE-MAP.md:871-872`).

**Must not fork:** a second 3× hash stopper that ignores 4-pattern detection; authoring J-85.

## 9. Config / flags / defaults

- `[stagnation]` in `.elai_cc/metrics/phase_4G.toml`: `stagnation_detection_enabled=true`, counters 0, `spin_threshold=3`, `adaptive_futility_threshold=0.15` (`:175-178`). Feature-map: this section **missing** while action_types already fire (`ELAI-FEATURE-MAP.md:761-765`).
- Autonomous gate flags remain C-9 (`ELAI_DISABLE_AUTONOMOUS_GATE`, `ELAI_GATE_MODE`) (`ELAI-FEATURE-MAP.md:754-760`).
- Decision doc: `workflow/reasoning/C-10-decisions.md` (`:319`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `anti_spiral_extended.rs` | `C-plan/C-10_anti_spiral_primitives.md:42-85` |
| MODIFY `autonomous_gate.rs` | `:97` |
| MODIFY `task_runner.rs` | `:104`, `:111`, `:118` |
| `anti_spiral_extended_contract.rs` (17 tests) | `:145-163` |
| `anti_spiral_gate_integration_contract.rs` (5 tests) | `:165-172` |
| `get_stagnation_events` + `OrchestrationDrawer.svelte` | `:136-143` |
| `persona-{hacker,researcher,simplifier,architect,contrarian}.md` | `:120-132` |
| `phase_4G.toml` `[stagnation]` | `:175-178` |

C-9 note that “C-10 does not touch `autonomous_gate.rs` / `task_runner.rs`” is **false against this plan** (exhaustive C-005/C9-E35 vs C-10 tasks 2–5). Rebuild **C-10**: it **does** wire those files.

## 11. Journal / SSB events

C-2 variants; C-10 **emits**:

| Event | When |
|---|---|
| `stagnation_spinning_detected` / `stagnation_oscillation_detected` | pattern hit; payload disambiguates NoDrift / DiminishingReturns |
| `lateral_thinking_activated` | `new_persona` |
| `goal_drift_warning` | Jaccard Red |
| `watchdog_timeout` | kind + elapsed + fingerprint_hex |

Futility SSB `action_type` is **unnamed** (C-003/A-C10-05) — **UNCERTAIN**.

Feature-map: five types fire unconditionally in `task_runner.rs:7143-7330` (`ELAI-FEATURE-MAP.md:762-764`).

## 12. Tests and observables

17 + 5 named tests (`C-plan/C-10_anti_spiral_primitives.md:147-172`): spinning after threshold; NoDrift identical hash; oscillation alternating pair; diminishing `<5%`; `None` on one entry; four affinities; Jaccard green `>0.7` / red `<0.4`; tokenize strips punctuation; watchdog safety / no-progress / idle; futility below threshold; ring evicts at 32; decide emits spinning + lateral + prefix; red drift emit; watchdog emit + escalate.

**Smoke** (`:218-227`, `:291-302`): autonomous “fix the impossible bug in `src/broken.rs`” → iteration-3 spinning, lateral persona ∈ {5}, next prompt has prefix, drift warning if Jaccard `<0.4`, 10-min no-progress watchdog terminates, drawer badges.

Zero regression: `autonomous_gate_contract` ≥14, `phase_4g_extended` ≥18 (`:261-262`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Four patterns `Spinning | Oscillation | NoDrift | DiminishingReturns` with the **ordered** rules and numeric defaults above.
- Five personas `Hacker | Researcher | Simplifier | Architect | Contrarian` with **C-10 affinity** (Spinning→Contrarian, Oscillation→Architect, NoDrift→Hacker, DiminishingReturns→Simplifier; Researcher fallback).
- Ring 32; Jaccard `BTreeSet`; watchdog blake3 + 120s/600s/1800s + Safety>NoProgress>Idle.
- Futility 0.15 / ≥4 attempts serializes to SSB; CircuitBreaker vs AntiSpiral split as specified.
- Five agent files + StagnationSection + 22 contract tests.
- Organ detects **REPETITION**; does not implement J-85 DRIFT.

A prompt “switch persona,” or the 3× hash stopper alone, is a cousin.

## 14. Non-goals / banned designs

- Authoring J-85 overthinking guard (`ELAI-FEATURE-MAP.md:668-677`).
- Replacing 3× Spinning subsumption with deleting 3C anti-spiral (P4G-E029).
- `HashSet` Jaccard; missing blake3 (`:213-214`).
- Silent drop of futility payload (`:215`).
- Merging C-10 baby-step Jaccard with E-46 Phase/Idea cosine canary.
- Using the **skill-capability** mapping (Spinning=SIMPLIFIER, NoDrift=RESEARCHER, Diminishing=CONTRARIAN) instead of C-10’s table — that mapping is **not** in `C-10_anti_spiral_primitives.md`. Rebuild C-10 (`:64-66`). Document the conflict in §16; do not invent a winner beyond “this organ copies C-10.”

## 15. Source citations

Primary: `docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md` (full file). Index: `C-plan/C-SUMMARY_INDEX.md:126-135`. B-22b: `B-plan/B-PLAN-SUMMARY.md:424-431`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:91-111`. Spec overview: `ELAI-SPEC-OVERVIEW.md:82-83`. Feature map 1.2/1.3: `ELAI-FEATURE-MAP.md:668-677`, `:738-740`, `:754-765`. Exhaustive C-10: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md:9446-9518` and C-003/E-C10-*. J-85 neighbor only: `ELAI-FEATURE-MAP.md:668-677`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Researcher fallback: what `index` is, when fallback runs; no pattern maps to Researcher (C-003/A-C10-01).
- Boundary at exactly 0.7/0.4; empty-set Jaccard; diminishing zero denominator; which string field oscillates; equal-hash vs spinning precedence beyond “NoDrift first” (C-003/A-C10-02).
- Fingerprint endianness; no-progress comparison; `timestamp_ms` vs `tokio::time::Instant` (C-003/A-C10-03).
- Which of two stagnation variants carries NoDrift vs DiminishingReturns (C-003/A-C10-04).
- Futility `action_type` name; timed-out task fate beyond Escalate/terminate (C-003/A-C10-05).
- `AdaptiveFutilityTracker` uses `HashMap` (iteration order unconstrained) while Jaccard bans `HashSet` (C-003/C-C10-01).
- B-22b timeout/Jaccard numbers vs C-10 (`B-PLAN-SUMMARY.md:427-430` vs C-10 `:67-74`).
- Skill-capability persona table vs C-10 affinity (this organ: **C-10**).
- Skill “max 2 switches per phase” — not in C-10 module.
