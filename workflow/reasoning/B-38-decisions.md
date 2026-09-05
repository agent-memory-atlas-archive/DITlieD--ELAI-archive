B-38 Ambiguity-Score Continuous Metric — Decisions

Session: s-18544
Phase: B-38 Ambiguity scorer

WHAT
- Port ouroboros bigbang/ambiguity.py to ELAI reasoning crate.
- Continuous ambiguity score (0..1) replacing binary grounding-sweep verdict.
- Extend GroundingSweepGate with score-threshold + per-dimension floors + 2-consecutive streak requirement.
- New SSB action types (ambiguity_scored, ambiguity_floor_failed, ambiguity_streak_advanced); SCHEMA_VERSION 42 -> 43.
- Dashboard GatesPanel.svelte + get_ambiguity_state Tauri command stub.

KEY DECISIONS

1. JSON parse strategy.
   Step 1: regex r"```(?:json)?\s*(\{[\s\S]*?\})\s*```" — fenced block.
   Step 2: fallback regex r"\{[\s\S]*\}" — greedy first-to-last brace.
   Step 3: serde_json::from_str on the captured slice.
   Step 4: clamp each clarity_score to [0.0, 1.0]; missing justification -> fallback string; missing score -> hard AmbiguityError::MissingField (no retry, matches ouroboros).
   Grounding: workflow/papers/ouroboros-ambiguity-interface.md lines 144-150 (parse pipeline).

2. Error handling.
   New `AmbiguityError` enum with thiserror::Error (per CLAUDE.md anti-stale rule).
   Variants: LlmError(String), ParseError{snippet, reason}, MissingField{name}, BudgetExhausted{retries}.
   Distinct from GrcError to avoid coupling reasoning::types to LLM concerns.

3. Truncation handling — adapter limitation.
   The LlmProvider trait (providers/src/providers/ollama.rs:352) only exposes temperature/top_p — no max_tokens. ELAI ollama path can't enforce a token budget from the caller side.
   Therefore the adaptive-token-allocation logic is implemented as a structural retry loop with `current_max_tokens` tracked INTERNALLY but NOT passed to the provider. Truncation is INFERRED when the response fails JSON parse AND the response ends abruptly (no closing brace). This is a documented degradation vs ouroboros semantics — provider-side max_tokens plumb-through is OUT OF SCOPE for B-38 and tracked as a follow-up.
   max_truncation_doublings still bounds the loop (5).
   max_format_error_retries also bounded (5).
   Grounding: file:.elai_cc/crates/providers/src/providers/ollama.rs:288-303 (OllamaConfig fields).

4. Streak persistence.
   For B-38 minimal contract: streak is a `u32` field threaded through TransitionContext (`streak_count`).
   Caller (task_runner.rs in a follow-up) is responsible for incrementing on each gate evaluation, resetting on plan revision, and persisting between runs.
   AmbiguityStreakAdvanced SSB row is emitted by the gate when the score qualifies but streak hasn't hit 2 yet.
   No new session-state machinery in B-38.

5. Brownfield detection.
   `brownfield: bool` field on TransitionContext, default false.
   When true: 4-component breakdown + context_clarity floor (0.60) applies.
   Detection logic (e.g. "does the project root have prior code?") lives in the caller; the gate trusts the flag.

6. Backward compatibility.
   When `TransitionContext.ambiguity_score == None`, the gate behaves exactly like B-17 (zero-ungrounded -> ALLOW). This preserves all existing call sites that haven't been migrated to populate ambiguity_score yet. Real scorer wiring into task_runner.rs is OUT OF SCOPE per "minimal B-38 contract".

7. Verdict priority (preserved order in `check()`).
   (a) ungrounded_count > 0           -> BLOCK (existing reason; ambiguity NEVER overrides this).
   (b) ambiguity_score is None        -> ALLOW (B-17 path).
   (c) overall_score > 0.2            -> BLOCK with breakdown reason.
   (d) any floor_failures             -> BLOCK with floor-list reason.
   (e) qualifies AND streak+1 < 2     -> BLOCK reason "streak 1/2; needs one more snapshot" (emit streak_advanced SSB).
   (f) qualifies AND streak+1 >= 2    -> ALLOW.

8. Threat model — LLM scorer can be wrong.
   The LLM is adversarial; we cannot trust an LLM-supplied "everything is 1.0" output.
   Mitigations: low temperature (0.1) reduces variance; per-dimension floors are MANDATORY (cannot be bypassed even when overall_score is tiny — see assertion in test_gate_blocks_when_floor_failed_even_below_threshold); 2-streak requirement prevents single-snapshot noise.
   We DO trust the deterministic formula (1 - Σ clarity_i * weight_i) and the deterministic milestone bands.

9. Rounding.
   `(x * 10000.0).round() / 10000.0` per ouroboros gotcha #2 (apply round to final result, not to sum). Tested.

10. No new external dependencies.
    `regex` is already in elai-reasoning Cargo.toml.
    `serde_json` is workspace-default for reasoning; available transitively. Verified via Cargo.toml.
    `async-trait` NOT used — native `async fn` only (anti-stale rule).

DEFERRED (explicit, out of B-38 scope)
- Wire AmbiguityScorer into task_runner.rs (real plan-text + grounding-report assembly + gate invocation pipeline). Tracked as B-38 follow-up.
- Provider-side max_tokens plumb-through (LlmProvider trait extension). Tracked as separate refactor.
- Real GatesPanel data wiring (B-38 ships mock data; producer arrives with the task_runner wiring).
- SessionState streak persistence across process restarts.

GROUNDING
- B-38 plan: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-38_ambiguity_score_gate.md
- Paper-ingestor output: F:\.ELAI_workflow\workflow\papers\ouroboros-ambiguity-interface.md
- LlmProvider trait: file:.elai_cc/crates/providers/src/providers/ollama.rs:352-368
- SCHEMA_VERSION head: file:.elai_cc/crates/orchestrator/src/ssb.rs:147 (42, bump to 43)
- B-34 SSB pattern: file:.elai_cc/crates/orchestrator/src/ssb.rs:818-834,982-987,1141-1146,2360-2421,3503-3507,3654-3658,5505-5538
- Existing GroundingSweepGate: file:.elai_cc/crates/orchestrator/src/evidence.rs:196-225

EXPECTED TEST OUTCOME
- Pre-implementation: contract tests compile-fail (types/methods don't exist).
- Post-implementation: 18 contract tests pass + reasoning unit tests for ambiguity module.
