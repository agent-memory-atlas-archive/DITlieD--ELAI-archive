# B-38: Ambiguity-Score Continuous Metric (ouroboros port)
Objective: retrofit ouroboros's continuous ambiguity formula
(`ambiguity = 1 − Σ clarity_i × weight_i`, threshold ≤ 0.2) onto ELAI's
existing GroundingSweepGate so the gate produces a graded score instead
of binary all-grounded/blocked. Adds per-dimension floors (Goal ≥ 0.75,
Constraint ≥ 0.65, Success ≥ 0.70, Context ≥ 0.60 brownfield) and
auto-complete-streak requirement so a single noisy snapshot can't pass.
Estimated effort: 1 session.
Pre-requisites: **B-17 (already done)** — GroundingSweepGate fires in
production. B-8 (PipelineGrc activation) — `ground_plan` callable.
Parallel-safe with: any non-grounding-related B-N.

## Context

ELAI has a GroundingSweepGate that fires (per B-17) and PipelineGrc's
`ground_plan` (per B-8) classifies claims as GROUNDED / VERIFY_NEEDED /
UNGROUNDED. The current gate verdict is binary: zero UNGROUNDED → pass,
else block.

This works but is brittle:
- It treats "everything verified" the same as "verified-but-thin" —
  a plan with 1 perfect citation ≠ a plan with 30 thorough citations
  even though both pass.
- It conflates "missing claim" with "unverifiable claim" — sometimes
  a claim is INTENTIONALLY deferred ("decide later when we know X")
  and shouldn't penalize the score.
- A single noisy snapshot can flip the gate. No streak requirement.

Ouroboros (github.com/Q00/ouroboros, MIT) solves this with a continuous
formula and per-dimension floors. Source:
`src/ouroboros/bigbang/ambiguity.py`. The four dimensions map cleanly
onto ELAI's grounding sweep:

| Ouroboros dimension      | Weight (greenfield / brownfield) | ELAI mapping                                 |
|--------------------------|----------------------------------|----------------------------------------------|
| Goal Clarity             | 40% / 35%                        | Plan goal verifiable from spec               |
| Constraint Clarity       | 30% / 25%                        | CLAUDE.md hard rules + project boundaries    |
| Success Criteria Clarity | 30% / 25%                        | ACCEPTANCE blocks measurable + DETERMINISTIC |
| Context Clarity          | — / 15%                          | DKG + RepoMap + CLAUDE.md auxiliary context  |

Threshold: `ambiguity ≤ 0.2` to advance.
Per-dimension floors (additional safety): Goal ≥ 0.75, Constraint ≥ 0.65,
Success ≥ 0.70, Context ≥ 0.60. Aggregate score can pass while one
dimension is dangerously low; floors block that.
Auto-complete streak: 2 consecutive ready snapshots required. Single
noisy snapshot does not advance.

LLM at `temperature=0.1` for reproducibility (matches ouroboros
SCORING_TEMPERATURE).

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-17_ground_vacuous_gates.md (DONE)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-8_pipelinegrc_activation.md
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2B.txt (grounding methods)
- ouroboros src/ouroboros/bigbang/ambiguity.py (full file, MIT)

## Tasks

1. **Spawn paper-ingestor (mode=source)** against
   `github.com/Q00/ouroboros src/ouroboros/bigbang/ambiguity.py` →
   `workflow/papers/ouroboros-ambiguity-interface.md` reliability MEDIUM.

2. **Add `AmbiguityScore` types** to `crates/reasoning/src/`:
   ```rust
   pub struct ComponentScore {
       pub name: String,
       pub clarity_score: f32,    // [0.0, 1.0]
       pub weight: f32,           // [0.0, 1.0]
       pub justification: String,
   }
   pub struct ScoreBreakdown {
       pub goal_clarity: ComponentScore,
       pub constraint_clarity: ComponentScore,
       pub success_criteria_clarity: ComponentScore,
       pub context_clarity: Option<ComponentScore>,  // brownfield only
   }
   pub struct AmbiguityScore {
       pub overall_score: f32,    // 1.0 - Σ(clarity_i × weight_i)
       pub breakdown: ScoreBreakdown,
   }
   impl AmbiguityScore {
       pub fn is_ready(&self) -> bool { self.overall_score <= 0.2 }
       pub fn floor_failures(&self, brownfield: bool) -> Vec<String>;
       pub fn qualifies_for_advance(&self, brownfield: bool) -> bool {
           self.is_ready() && self.floor_failures(brownfield).is_empty()
       }
   }
   ```

3. **Constants** matching ouroboros (port verbatim):
   ```rust
   pub const AMBIGUITY_THRESHOLD: f32 = 0.2;
   pub const AUTO_COMPLETE_STREAK_REQUIRED: usize = 2;
   pub const GOAL_CLARITY_FLOOR: f32 = 0.75;
   pub const CONSTRAINT_CLARITY_FLOOR: f32 = 0.65;
   pub const SUCCESS_CRITERIA_CLARITY_FLOOR: f32 = 0.70;
   pub const BROWNFIELD_CONTEXT_CLARITY_FLOOR: f32 = 0.60;
   pub const SCORING_TEMPERATURE: f32 = 0.1;

   pub const GOAL_CLARITY_WEIGHT: f32 = 0.40;
   pub const CONSTRAINT_CLARITY_WEIGHT: f32 = 0.30;
   pub const SUCCESS_CRITERIA_CLARITY_WEIGHT: f32 = 0.30;
   pub const BROWNFIELD_GOAL_CLARITY_WEIGHT: f32 = 0.35;
   pub const BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT: f32 = 0.25;
   pub const BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT: f32 = 0.25;
   pub const BROWNFIELD_CONTEXT_CLARITY_WEIGHT: f32 = 0.15;
   ```

4. **`AmbiguityScorer`** at `crates/reasoning/src/ambiguity.rs`:
   ```rust
   pub struct AmbiguityScorer { /* llm_adapter, model, temperature, ... */ }
   impl AmbiguityScorer {
       pub async fn score(
           &self,
           plan: &PlanSkeleton,
           grounding_report: &GroundingReport,
           brownfield: bool,
           deferrals: &[String],   // intentional deferrals (do not penalize)
       ) -> Result<AmbiguityScore, GrcError>;
   }
   ```
   - Adaptive token allocation: starts with 2048, doubles on truncation
     up to 5 retries, max 5 format-error retries (matches ouroboros
     `initial_max_tokens=2048`, `max_format_error_retries=5`).
   - JSON parse: handle markdown code-block wrap and raw JSON.
   - Clamp scores to [0.0, 1.0]; missing justifications get a fallback
     string but missing scores are an error (matches ouroboros).
   - Deferral instruction in system prompt: "decide-later items don't
     penalize the score."

5. **Wire `AmbiguityScorer` into GroundingSweepGate** (existing, B-17):
   - Gate now invokes scorer in addition to the existing
     `ground_plan` UNGROUNDED check.
   - Verdict logic:
     ```
     ungrounded_count > 0           -> BLOCK (existing behavior)
     ambiguity > 0.2                -> BLOCK with breakdown
     any floor_failures             -> BLOCK with floor list
     score qualifies AND streak < 2 -> WAIT (one more snapshot)
     score qualifies AND streak >= 2 -> ALLOW
     ```
   - Streak counter persisted in SessionState (per-session, resets on
     plan revision).

6. **Dashboard surface**: extend B-17's GatesPanel to show:
   - Current ambiguity score with milestone label (INITIAL > 0.4,
     PROGRESS > 0.3, REFINED > 0.2, READY ≤ 0.2 — port from ouroboros
     `MILESTONE_DEFINITIONS`)
   - Per-dimension breakdown bars
   - Floor-failure flags
   - Streak progress (1/2 or 2/2)

7. **SSB action_types**: `ambiguity_scored`, `ambiguity_floor_failed`,
   `ambiguity_streak_advanced`. Surfaced in OrchestrationDrawer.

## WIRING_CONTRACT

produces_symbols:
  - `AmbiguityScore`, `ScoreBreakdown`, `ComponentScore` types
  - `AmbiguityScorer` service in reasoning crate
  - `is_ready()`, `floor_failures()`, `qualifies_for_advance()` methods
  - GroundingSweepGate verdict extension (continuous + streak)
  - SSB action_types: ambiguity_scored, ambiguity_floor_failed,
    ambiguity_streak_advanced
  - GatesPanel ambiguity section
consumes_symbols:
  - existing GroundingSweepGate (B-17 done)
  - existing PipelineGrc.ground_plan (B-8 done)
  - existing LlmProvider for scoring
reachable_from:
  - PLANNING -> TESTING_FIRST FSM transition
stub_ban:
  - Streak counter must NOT carry across plan revisions
  - Floor failures MUST block even when overall score is ≤ 0.2
  - Deferrals MUST NOT penalize score
frontend_pair: GatesPanel ambiguity section
observable_effect:
  trigger: submit task with deliberately vague goal
  expect_ssb: ambiguity_scored row with overall_score > 0.2
  expect_artifact: GatesPanel shows INITIAL/PROGRESS/REFINED label,
                   per-dimension bars, floor-failure flags

## DETERMINISTIC_CHECK

```bash
# 1. AmbiguityScorer + types exist
rg "struct AmbiguityScorer|struct AmbiguityScore" .elai_cc/crates/reasoning/src

# 2. Constants match ouroboros (8 weights + 4 floors + threshold + streak)
rg "AMBIGUITY_THRESHOLD|AUTO_COMPLETE_STREAK_REQUIRED|GOAL_CLARITY_FLOOR|CONSTRAINT_CLARITY_FLOOR|SUCCESS_CRITERIA_CLARITY_FLOOR" .elai_cc/crates/reasoning/src
# Expected: ≥ 5 hits

# 3. Tests
cargo test -p elai_reasoning -- ambiguity 2>&1 | grep "test result"
# Expected: ≥ 12 tests (greenfield score, brownfield score, floor failures,
# deferral non-penalty, streak counter, JSON parse edge cases, etc.)

# 4. Gate integration
rg "AmbiguityScore" .elai_cc/crates/orchestrator/src/gate.rs
```

## ACCEPTANCE

- [ ] Paper-ingestor produced ouroboros-ambiguity-interface.md
- [ ] AmbiguityScore + ScoreBreakdown + ComponentScore types
- [ ] All 11 weight + floor constants ported verbatim from ouroboros
- [ ] `is_ready()` returns true iff overall_score ≤ 0.2
- [ ] `floor_failures()` returns the per-dimension failures list
- [ ] `qualifies_for_advance()` requires BOTH overall ≤ 0.2 AND no floor
      failures
- [ ] AmbiguityScorer.score() supports adaptive token allocation
      (doubles on truncation, max 5 retries; max 5 format-error retries)
- [ ] Deferral instruction in system prompt prevents penalty for
      decide-later items
- [ ] GroundingSweepGate verdict extended with continuous + streak logic
- [ ] Streak counter resets on plan revision
- [ ] GatesPanel shows ambiguity score + milestone + breakdown +
      floor flags + streak
- [ ] 3 new SSB action_types live
- [ ] ≥ 12 tests pass

## Files touched

- ADD: crates/reasoning/src/ambiguity.rs (~300 LOC port)
- MODIFY: crates/reasoning/src/lib.rs (re-export)
- MODIFY: crates/orchestrator/src/gate.rs (extend GroundingSweepGate)
- MODIFY: crates/orchestrator/src/ssb.rs (3 new action_types)
- MODIFY: crates/dashboard-v2/src/lib/components/GatesPanel.svelte
  (or whichever B-17 created)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (get_ambiguity_state cmd)
- ADD: workflow/papers/ouroboros-ambiguity-interface.md (paper-ingestor)
- CREATE: workflow/reasoning/B-38-decisions.md

## Next

Whatever the next outstanding B-N is per B-0_INDEX prioritization.
