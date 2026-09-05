Source: ouroboros bigbang/ambiguity.py (github.com/Q00/ouroboros MIT)
Reliability: MEDIUM
Files examined:
  https://raw.githubusercontent.com/Q00/ouroboros/main/src/ouroboros/bigbang/ambiguity.py (raw fetch)
  https://github.com/Q00/ouroboros/blob/main/src/ouroboros/bigbang/ambiguity.py (HTML view, constants + signatures)
  https://github.com/Q00/ouroboros/blob/main/src/ouroboros/bigbang/interview.py (streak counter field)
  Local cache: F:\.ELAI_workflow\workflow\papers\ouroboros-ambiguity-src\ambiguity.py

Consumer: B-38 Ambiguity-score continuous metric port to ELAI reasoning crate


==============================================================================
CONSTANTS (all verified verbatim)
==============================================================================

AMBIGUITY_THRESHOLD               = 0.2      (seed readiness gate; score <= this -> READY)
SEED_CLOSER_ACTIVATION_THRESHOLD  = 0.25     (triggers specialized interview mode before READY)
AUTO_COMPLETE_STREAK_REQUIRED     = 2        (consecutive READY scores before auto-complete fires)

GOAL_CLARITY_WEIGHT               = 0.40     (greenfield)
CONSTRAINT_CLARITY_WEIGHT         = 0.30     (greenfield)
SUCCESS_CRITERIA_CLARITY_WEIGHT   = 0.30     (greenfield)
BROWNFIELD_GOAL_CLARITY_WEIGHT    = 0.35
BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT        = 0.25
BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT  = 0.25
BROWNFIELD_CONTEXT_CLARITY_WEIGHT           = 0.15

GOAL_CLARITY_FLOOR                = 0.75     (minimum component score before auto-complete)
CONSTRAINT_CLARITY_FLOOR          = 0.65
SUCCESS_CRITERIA_CLARITY_FLOOR    = 0.70
BROWNFIELD_CONTEXT_CLARITY_FLOOR  = 0.60

SCORING_TEMPERATURE               = 0.1
MAX_TOKEN_LIMIT: int | None       = None     (no hard cap; uses model context window)


==============================================================================
FORMULA
==============================================================================

Core ambiguity formula (verified from _calculate_overall_score()):
    weighted_clarity = sum(component.clarity_score * component.weight
                           for component in breakdown.components)
    overall_score = round(1.0 - weighted_clarity, 4)

Equivalently:
    ambiguity = 1.0 - sum(clarity_i * weight_i for i in components)

Invariant: overall_score in [0.0, 1.0] (0=no ambiguity, 1=total ambiguity)
Invariant: weights must sum to 1.0 within each mode
    greenfield:   0.40 + 0.30 + 0.30 = 1.00
    brownfield:   0.35 + 0.25 + 0.25 + 0.15 = 1.00

Rounding: round(..., 4) — preserve 4 decimal places in Rust port.


==============================================================================
MILESTONE_DEFINITIONS
==============================================================================

Lookup: iterate in order; first entry where score <= upper_bound is the milestone.
Milestones are checked from smallest bound upward (READY first, then REFINED, etc.),
or equivalently from score descending:
  score <= 0.20 -> READY    "All criteria concrete and testable. Ready for Seed generation."
  score <= 0.30 -> REFINED  "Success criteria partially defined. Edge cases and non-goals remain."
  score <= 0.40 -> PROGRESS "Most requirements captured. Some details and edge cases missing."
  score <= 1.00 -> INITIAL  "Core requirements identified. Major gaps in constraints and success criteria."

Python representation:
MILESTONE_DEFINITIONS: tuple[tuple[float, AmbiguityMilestone, str], ...] = (
    (1.0,                AmbiguityMilestone.INITIAL,  "Core requirements identified..."),
    (0.4,                AmbiguityMilestone.PROGRESS, "Most requirements captured..."),
    (0.3,                AmbiguityMilestone.REFINED,  "Success criteria partially defined..."),
    (AMBIGUITY_THRESHOLD, AmbiguityMilestone.READY,   "All criteria concrete and testable..."),
)
Note: the tuples are ordered largest-bound first; get_milestone() finds first where score <= bound.


==============================================================================
Public API
==============================================================================

AmbiguityScore | frozen dataclass(frozen=True, slots=True)
  overall_score: float                        # [0.0, 1.0]
  breakdown: ScoreBreakdown
  is_ready_for_seed -> bool                   # property: overall_score <= AMBIGUITY_THRESHOLD

AmbiguityMilestone | StrEnum
  INITIAL = "initial"
  PROGRESS = "progress"
  REFINED = "refined"
  READY = "ready"

ComponentScore | pydantic BaseModel
  name: str
  clarity_score: float  ge=0.0 le=1.0
  weight: float  ge=0.0 le=1.0
  justification: str

ScoreBreakdown | pydantic BaseModel
  goal_clarity: ComponentScore
  constraint_clarity: ComponentScore
  success_criteria_clarity: ComponentScore
  context_clarity: ComponentScore | None      # None for greenfield

AmbiguityScorer | @dataclass
  llm_adapter: LLMAdapter                    (required)
  model: str | None = None
  model_is_explicit: bool = False  (not in __init__)
  temperature: float = 0.1
  initial_max_tokens: int = 2048
  max_retries: int | None = 10
  max_format_error_retries: int = 5

  async def score(
      self,
      state: InterviewState,
      is_brownfield: bool = False,
      additional_context: str = "",
  ) -> Result[AmbiguityScore, ProviderError]

Helper functions:
  get_milestone(score: float) -> tuple[AmbiguityMilestone, str]
  get_next_milestone(score: float) -> tuple[float, AmbiguityMilestone, str] | None
  get_completion_floor_failures(score: AmbiguityScore, *, is_brownfield: bool) -> list[str]
  qualifies_for_seed_completion(score: AmbiguityScore, *, is_brownfield: bool) -> bool
  is_ready_for_seed(score: AmbiguityScore) -> bool
  format_score_display(score: AmbiguityScore) -> str


==============================================================================
Behavioral contracts
==============================================================================

score() -> adaptive token allocation:
  - starts with current_max_tokens = initial_max_tokens (2048)
  - if LLM response finish_reason == "length": current_max_tokens *= 2
  - if MAX_TOKEN_LIMIT is not None: current_max_tokens = min(current_max_tokens, MAX_TOKEN_LIMIT)
  - MAX_TOKEN_LIMIT is None by default (no cap; effective ceiling = model context window)
  - truncation does NOT increment format_error_count
  - max 5 format-error retries (max_format_error_retries=5); hard break when exceeded
  - max_retries=10 for provider errors; None = unlimited provider-error retries
  - missing required JSON score key -> raises ValueError (not retried; hard failure)

score() -> JSON parse:
  Step 1: try regex r"```(?:json)?\s*(\{.*?\})\s*```" (DOTALL) -> group(1)
  Step 2: fallback regex r"\{.*\}" (DOTALL) -> group(0)
  Step 3: clamp each score: max(0.0, min(1.0, float(value)))
  Step 4: missing justification -> f"{component_name} justification not provided by model."
  Step 5: missing required score field -> ValueError (propagates, not retried as format error)

score() -> deferral instruction (system prompt invariant):
  Both greenfield and brownfield system prompts include:
  "If the additional context lists 'decide-later' or 'deferred' items, these are INTENTIONAL
  deferrals — do NOT penalise the clarity score for them."
  The additional_context parameter is appended to user prompt with label
  "Additional context (intentional deferrals — do not penalise):"

score() -> InterviewState fields consumed:
  state.interview_id       (logging only)
  state.rounds             (iterable; each .question + .user_response)
  prompt_safe_initial_context(state)  (context summary prepended)
  initial_context_summary_missing(state) check

Streak counter (InterviewState.completion_candidate_streak):
  Type: int, ge=0, default=0, field on InterviewState (not on AmbiguityScorer)
  Increment: by authoring_handlers when score.overall_score <= AMBIGUITY_THRESHOLD
  Auto-complete trigger: streak >= AUTO_COMPLETE_STREAK_REQUIRED (2)
  Reset: in InterviewState.record_response() when user reopens completed interview
         (state.completion_candidate_streak = 0)
  Note: AUTO_COMPLETE_STREAK_REQUIRED is defined in ambiguity.py but consumed in interview.py/authoring_handlers


==============================================================================
4-Component score breakdown
==============================================================================

Greenfield (3 components):
  goal_clarity        weight=0.40
  constraint_clarity  weight=0.30
  success_criteria_clarity  weight=0.30

Brownfield (4 components):
  goal_clarity        weight=0.35
  constraint_clarity  weight=0.25
  success_criteria_clarity  weight=0.25
  context_clarity     weight=0.15


==============================================================================
Platform constraints
==============================================================================

Python version: requires 3.11+ (StrEnum stdlib, slots=True on dataclass, X | Y union syntax)
Pydantic v2 required (Field(ge=..., le=...) Pydantic v2 signature)
Async: score() is async; requires asyncio event loop
No multiprocessing usage; no pickling concerns
No file I/O in this module
Depends on: LLMAdapter protocol (ouroboros.llm), InterviewState (ouroboros.bigbang.interview),
            Result type (likely result/returns library or ouroboros-internal)


==============================================================================
Gotchas
==============================================================================

1. MILESTONE_DEFINITIONS ordered largest-bound first (1.0 -> 0.4 -> 0.3 -> 0.2).
   get_milestone() finds first entry where score <= bound — not last. Port must iterate
   in reverse order of bound magnitude (i.e., check READY first then REFINED then PROGRESS).
   Wrong iteration order returns INITIAL for scores that are READY.

2. overall_score = round(1 - sum, 4), NOT 1 - round(sum, 4). Apply round to the final result.

3. Missing JSON score key raises ValueError (hard failure). Missing justification returns
   fallback string (soft). These are NOT symmetric — do not unify them as "soft".

4. MAX_TOKEN_LIMIT = None means the doubling is unbounded in theory. ELAI port should add
   a practical cap (e.g. model context window size) to avoid infinite growth.

5. AUTO_COMPLETE_STREAK_REQUIRED is defined in ambiguity.py but streak state lives on
   InterviewState.completion_candidate_streak. The Rust port needs both: the constant
   and the per-session state field.

6. Truncation (finish_reason=="length") does NOT count toward format_error_retries.
   A sequence of 5 truncations followed by a parse error is 1 format error, not 6.

7. SEED_CLOSER_ACTIVATION_THRESHOLD (0.25) and AMBIGUITY_THRESHOLD (0.2) are distinct.
   0.25 activates special interview mode; 0.2 gates seed generation. Do not conflate.

8. Brownfield weight sum must equal 1.0: 0.35+0.25+0.25+0.15 = 1.00.
   Greenfield weight sum: 0.40+0.30+0.30 = 1.00. Both exact — no floating point slack intended.

9. qualifies_for_seed_completion() checks BOTH overall_score <= AMBIGUITY_THRESHOLD AND
   all per-component floors. is_ready_for_seed() checks ONLY overall_score. These are
   different predicates.

10. SCORING_TEMPERATURE = 0.1 is intentionally low for deterministic scoring. Do not raise
    it for "diversity" — low variance is by design.


==============================================================================
Critical assertions (testable, Python syntax)
==============================================================================

# Weight sums
assert abs((0.40 + 0.30 + 0.30) - 1.0) < 1e-9, "greenfield weights must sum to 1.0"
assert abs((0.35 + 0.25 + 0.25 + 0.15) - 1.0) < 1e-9, "brownfield weights must sum to 1.0"

# Formula: ambiguity = 1 - sum(clarity_i * weight_i)
def _ambiguity(scores, weights):
    return round(1.0 - sum(s * w for s, w in zip(scores, weights)), 4)
assert _ambiguity([1.0, 1.0, 1.0], [0.40, 0.30, 0.30]) == 0.0
assert _ambiguity([0.0, 0.0, 0.0], [0.40, 0.30, 0.30]) == 1.0
assert 0.0 <= _ambiguity([0.5, 0.5, 0.5], [0.40, 0.30, 0.30]) <= 1.0

# Milestone boundary: score=0.20 -> READY
assert _ambiguity([0.9, 0.9, 0.9], [0.40, 0.30, 0.30]) <= 0.20  # 1 - 0.9 = 0.1 -> READY

# Threshold
assert 0.2 == 0.20  # AMBIGUITY_THRESHOLD exactly
assert 0.2 < 0.25   # AMBIGUITY_THRESHOLD < SEED_CLOSER_ACTIVATION_THRESHOLD

# Streak
assert 2 == 2  # AUTO_COMPLETE_STREAK_REQUIRED

# Score clamping
assert max(0.0, min(1.0, 1.5)) == 1.0
assert max(0.0, min(1.0, -0.1)) == 0.0

# Milestone lookup order (READY must not be shadowed by INITIAL/PROGRESS/REFINED)
# score=0.15 should be READY, not INITIAL
bounds = [1.0, 0.4, 0.3, 0.2]
score_val = 0.15
milestone_idx = next(i for i, b in enumerate(reversed(bounds)) if score_val <= b)
assert milestone_idx == 0  # first match in reversed = tightest bound = READY
