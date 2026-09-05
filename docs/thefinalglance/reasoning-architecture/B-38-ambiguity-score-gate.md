# Ambiguity-Score Continuous Metric (B-38)

- **ELAI ids:** B-38 / B38-001..B38-022
- **Spec status (as designed):** WIRED (`ELAI-FEATURE-MAP.md:613-614`)
- **Crate / module (spec):** `crates/reasoning/src/ambiguity.rs` (`AmbiguityScorer`, `AmbiguityScore`, `ScoreBreakdown`, `ComponentScore`); `crates/orchestrator` `GroundingSweepGate` extension + `SessionState` streak; dashboard `GatesPanel` + `get_ambiguity_state`
- **Purpose of this file:** enough architecture to rebuild the continuous ambiguity gate that sits **on top of** binary `ground_plan`, with ouroboros weights/floors/streak. Not a status ledger.

## 1. What it is

The **ambiguity-score gate** retrofits ouroboros’s formula `ambiguity = 1 − Σ(clarity_i × weight_i)` (threshold `≤ 0.2`) onto ELAI’s existing `GroundingSweepGate` so the gate produces a **graded** score instead of binary all-grounded/blocked (`B-38_ambiguity_score_gate.md:1-6`, `:16-18`, B38-001). It adds per-dimension floors (Goal ≥ 0.75, Constraint ≥ 0.65, Success ≥ 0.70, brownfield Context ≥ 0.60) and an auto-complete streak of **2** consecutive ready snapshots (`:4-6`, `:41-46`). Binary UNGROUNDED remains first: this organ **augments** B-8/B-17 `ground_plan`, it does not replace it (`:16-18`, `:132-137`). Paper port: `github.com/Q00/ouroboros` `src/ouroboros/bigbang/ambiguity.py` (MIT) (`:29-30`, `:56`).

## 2. Why it exists

Binary sweep treats “one perfect citation” the same as “thirty thorough citations”, conflates missing vs intentionally deferred claims, and lets a single noisy snapshot flip the gate (`B-38_ambiguity_score_gate.md:20-27`, B38-002). If the organ is missing, PLANNING→TESTING_FIRST can advance on thin-but-zero-UNGROUNDED plans. If it is faked (LLM overall vibe 0–1 without weights/floors/streak), aggregate-pass / dimension-fail cases leak (`:43-44`). Ambiguity readiness does **not** substitute for B-41 zero-BLOCK tier evidence (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-006).

## 3. How it works (runtime)

1. **Who calls it.** `GroundingSweepGate` on PLANNING → TESTING_FIRST, after `pipeline_grc.ground_plan` (`B-38_ambiguity_score_gate.md:132-134`, `:171-172`, B38-013, B38-017).
2. **Hard binary first (deterministic).** `ungrounded_count > 0` → **BLOCK** (existing 2B/B-8 behavior). Scorer is not a pardon (`:137`).
3. **Input to scorer.** `AmbiguityScorer::score(plan: &PlanSkeleton, grounding_report: &GroundingReport, brownfield: bool, deferrals: &[String]) -> Result<AmbiguityScore, GrcError>` (`:114-121`, B38-010). Deferrals are intentional “decide later” items and **must not penalize** (`:119`, `:129-130`, `:175-176`).
4. **Model may generate** per-dimension `clarity_score` and `justification` at `SCORING_TEMPERATURE = 0.1` (`:48-49`, `:99`). Adaptive tokens: start `2048`, double on truncation up to **5** retries; independent max **5** format-error retries (`:123-125`, B38-011). JSON parse: markdown code-block wrap **or** raw JSON (`:126-127`).
5. **Deterministic post-parse.** Clamp scores to `[0.0, 1.0]`. Missing **justification** → fallback string. Missing **score** → **error** (not 0.0) (`:127-128`, B38-012). Weights are constants, not model output (`:92-108`).
6. **Compute** `overall_score = 1.0 - Σ(clarity_i × weight_i)` (`:79`, `:1-2`).
7. **Verdict (ordered, fail-closed)** (`:135-142`, B38-013):
   ```
   ungrounded_count > 0              -> BLOCK
   overall_score > 0.2               -> BLOCK with breakdown
   any floor_failures(brownfield)    -> BLOCK with floor list
   qualifies AND streak < 2          -> WAIT (need one more snapshot)
   qualifies AND streak >= 2         -> ALLOW
   ```
   `qualifies_for_advance = is_ready() && floor_failures(brownfield).is_empty()` (`:83-87`). `is_ready` iff `overall_score <= 0.2` (`:83`, `:208`).
8. **Streak (deterministic).** Persisted in `SessionState`, **per-session**, **resets on plan revision**. Must not carry across revisions (`:143-144`, `:174`, B38-014).
9. **Journal.** `ambiguity_scored`, `ambiguity_floor_failed`, `ambiguity_streak_advanced` (`:154-155`).
10. **Artifact.** GatesPanel: score, milestone label, per-dimension bars, floor flags, streak `1/2` or `2/2` (`:146-152`).

The model may not choose weights, floors, streak length, or override UNGROUNDED.

## 4. Types, enums, objects

Copy from `B-38_ambiguity_score_gate.md:64-108`. Do not invent fields.

```
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

pub struct AmbiguityScorer { /* llm_adapter, model, temperature, ... */ }
impl AmbiguityScorer {
    pub async fn score(
        &self,
        plan: &PlanSkeleton,
        grounding_report: &GroundingReport,
        brownfield: bool,
        deferrals: &[String],
    ) -> Result<AmbiguityScore, GrcError>;
}

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

**Dimension map** (`:34-39`, B38-006):

| Ouroboros dimension | Greenfield / brownfield weight | ELAI mapping |
|---|---|---|
| Goal Clarity | 40% / 35% | Plan goal verifiable from spec |
| Constraint Clarity | 30% / 25% | CLAUDE.md hard rules + project boundaries |
| Success Criteria Clarity | 30% / 25% | ACCEPTANCE + DETERMINISTIC measurable |
| Context Clarity | — / 15% | DKG + RepoMap + CLAUDE.md auxiliary |

`context_clarity` is `None` in greenfield (no fourth term in the sum).

**Milestones** (display only, ouroboros `MILESTONE_DEFINITIONS`) (`:147-149`, B38-015): `INITIAL` if score `> 0.4`; `PROGRESS` `> 0.3`; `REFINED` `> 0.2`; `READY` `≤ 0.2`.

## 5. Control flow / state machine

**Owner:** `GroundingSweepGate` + `SessionState` streak. Scorer is a service; the gate owns the verdict.

```
PLANNING --ground_plan--> ungrounded? --yes--> BLOCK
                              |
                             no
                              v
                       AmbiguityScorer.score
                              |
                    overall > 0.2? --yes--> BLOCK (breakdown)
                              |
                             no
                              v
                    floor_failures? --yes--> BLOCK (floor list) + ambiguity_floor_failed
                              |
                             no
                              v
                    streak < 2? --yes--> WAIT + ambiguity_streak_advanced
                              |
                             no
                              v
                            ALLOW
```

**Forced reset:** plan revision zeros streak (`:174`). **WAIT** is not ALLOW and not a silent pass.

Greenfield vs brownfield selects the weight set **and** whether Context floor applies (`:85-86`, `:39`).

## 6. Grounding, receipts, gates

Binary 2B receipts remain authoritative for UNGROUNDED. Ambiguity is an **additional** gate on PLANNING→TESTING_FIRST (`:171-172`).

Deferral instruction in the **system prompt**: “decide-later items don’t penalize the score” (`:129-130`). Deferrals are also an explicit `&[String]` input — prompt-only exemption is insufficient (`:119`, stub_ban `:176`).

**Never allowed as a confidence signal:** overall LLM 0–1 with no breakdown; using window fill; treating streak as optional; using greenfield weights on brownfield (or omitting Context floor when `brownfield=true`); passing because `is_ready()` while a floor fails.

Source of the formula is ouroboros MIT code, ingested to `workflow/papers/ouroboros-ambiguity-interface.md` reliability MEDIUM (`:60-62`, B38-007) — not an ELAI-invented scalar.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `AMBIGUITY_THRESHOLD` | 0.2 | overall_score ≤ to be ready |
| streak | 2 consecutive qualifying snapshots | per session, reset on revision |
| Goal / Constraint / Success / Context floors | 0.75 / 0.65 / 0.70 / 0.60 | Context brownfield only |
| scoring temperature | 0.1 | scorer calls |
| initial_max_tokens | 2048, double on truncation | per score() |
| truncation retries | max 5 | per score() |
| format-error retries | max 5 | independent |
| tests | ≥ 12 | reasoning crate |

Acceptance text says “all **11** weight + floor constants” while the block lists **15** named consts (threshold, streak, 4 floors, temperature, 3 greenfield weights, 4 brownfield weights) (`B-38_ambiguity_score_gate.md:206-207` vs `:92-108`, B38-022). Rebuild **the named consts verbatim**; do not invent a subset of 11.

## 8. Roles and neighbors

**Consumes:** `GroundingSweepGate` (B-17), `PipelineGrc.ground_plan` (B-8), existing `LlmProvider` (`:167-170`).

**Produces:** `AmbiguityScore` types + methods; gate verdict extension; 3 SSB types; GatesPanel section; `get_ambiguity_state`.

**Must not fork:** a second ambiguity formula in the dashboard; replacing UNGROUNDED with the continuous score; carrying streak across plan revisions.

**Surfaced in** OrchestrationDrawer **and** GatesPanel (`:155`, `:146`). B-11 visual tokens apply to the new bars (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-003).

## 9. Config / flags / defaults

- No default-OFF experiment flag in B-38. Gate extension is the production path once B-17/B-8 exist (`:8-10`).
- `brownfield: bool` is an argument to `score` / `floor_failures` / `qualifies_for_advance` — how the orchestrator **detects** brownfield is not specified in B-38 (§16).
- Paper artifact + `workflow/reasoning/B-38-decisions.md` required (`:232-233`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `crates/reasoning/src/ambiguity.rs` (~300 LOC port) | `:225` |
| re-export `lib.rs` | `:226` |
| `orchestrator/src/gate.rs` extend GroundingSweepGate | `:227` |
| `orchestrator/src/ssb.rs` 3 action_types | `:228` |
| `GatesPanel.svelte` (or whichever B-17 created) | `:229-230` |
| `get_ambiguity_state` Tauri | `:231` |
| `workflow/papers/ouroboros-ambiguity-interface.md` | `:232` |

Deterministic `rg`: `struct AmbiguityScorer|struct AmbiguityScore`; consts `AMBIGUITY_THRESHOLD|AUTO_COMPLETE_STREAK_REQUIRED|GOAL_CLARITY_FLOOR|...`; `AmbiguityScore` in `gate.rs` (`:184-200`).

## 11. Journal / SSB events

| action_type | when |
|---|---|
| `ambiguity_scored` | after `score()`; payload includes `overall_score` (smoke expects `> 0.2` on vague goal) |
| `ambiguity_floor_failed` | floor list nonempty |
| `ambiguity_streak_advanced` | streak increment on qualifying snapshot |

Schema bump recorded later as 42 → 43 (`B-PLAN-SUMMARY.md:990`, B0-028). B-38 body does not pin the integer — rebuild the **three strings**.

## 12. Tests and observables

≥ **12** tests (`:195-197`, `:221`): greenfield score, brownfield score, floor failures, deferral non-penalty, streak counter, JSON parse edge cases, plus constants/retries/dashboard/3 events (`:203-221`).

Observable: submit task with deliberately vague goal → `ambiguity_scored` with `overall_score > 0.2`; GatesPanel shows INITIAL/PROGRESS/REFINED (or READY), bars, floor flags (`:178-182`).

Acceptance also: paper-ingestor artifact; `is_ready` iff ≤ 0.2; `qualifies_for_advance` conjunction; adaptive tokens; deferral prompt; streak reset on revision; 3 SSB types live (`:203-221`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Formula `1 − Σ(clarity_i × weight_i)` with the **verbatim** greenfield 0.40/0.30/0.30 and brownfield 0.35/0.25/0.25/0.15 weights.
- Floors 0.75 / 0.65 / 0.70 / 0.60 (Context only if brownfield); floors BLOCK even when overall ≤ 0.2.
- Threshold 0.2; streak 2; temperature 0.1; 2048×2 token retries (max 5) and max 5 format retries.
- Types `ComponentScore`, `ScoreBreakdown`, `AmbiguityScore`, `AmbiguityScorer::score` including `deferrals`.
- Gate order: UNGROUNDED → overall → floors → streak WAIT → ALLOW.
- Streak in SessionState resets on plan revision.
- Missing score is an error; missing justification is a fallback string; clamp [0,1].
- Three SSB types + GatesPanel milestone/bars/floors/streak.
- ≥ 12 tests covering the cases in §12.

A 0–1 LLM “clarity” with no floors/streak/UNGROUNDED-first is a cousin.

## 14. Non-goals / banned designs

- Replacing `ground_plan` / deleting binary UNGROUNDED (`:16-18`, `:137`).
- LLM-assigned **weights** (constants only).
- Penalizing listed deferrals (`:176`).
- Streak shared across plan revisions (`:174`).
- Using ambiguity ALLOW to skip B-41 per-level BLOCK checks.
- Inventing a 11-constant subset that drops named floors or brownfield weights (B38-022).
- Scoring temperature other than 0.1 “because local models differ”.

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-38_ambiguity_score_gate.md` (full file). Index: `B-PLAN-SUMMARY.md:790-811`, `:990`. Feature map: `ELAI-FEATURE-MAP.md:613-614`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:198-199`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B38-001..B38-022. Neighbors: `B-8_pipelinegrc_activation.md`, `02B-grounding-verification.md`. Paper: ouroboros `ambiguity.py` (MIT) as ingested.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- How `brownfield: bool` is detected (index size? EXISTING_PROJECT? config?).
- `PlanSkeleton` / `GroundingReport` field-level mapping onto 2B `PlanGroundingResult` (`:115-117` names them; 2B uses `PlanGroundingResult`).
- Exact `AmbiguityScorer` struct fields (`llm_adapter, model, temperature, ...` ellipsis) (`:112`).
- `floor_failures` string format / dimension names in the `Vec<String>`.
- WAIT vs BLOCK FSM encoding (does WAIT hold PLANNING? retry scorer? user-visible only?).
- `ambiguity_scored` full payload besides `overall_score`.
- Schema integer 42→43 vs B-38 body silence.
- “11 weight + floor constants” vs 15 named consts (B38-022) — copy names, do not invent the count.
- Whether OrchestrationDrawer **and** GatesPanel are both required or one is the V2 home (`:155` vs `:146`).
