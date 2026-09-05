# C-26: Constrained Decoding + Retry/Futility Primitives
Objective: implement ConstrainedDecodingConfig/DecodingMode in providers crate,
RetryStrategy with temperature_schedule + futility_same_error_threshold, wire
RetryStrategy into BubbleUpPolicy in pipeline_tree_navigator.rs, implement
futility detection + persona switch in NavigatorTree (consumes C-10
GoalDriftCanary), wire ConstrainedDecodingConfig into SidecarClient and
CloudGroundingClient (Anthropic Tool API json_schema), fix benchmark fixture
path bug so measurements 3/4/5 produce non-zero deltas, re-run bench and update
model_baseline.toml, add three EBNF grammar files, add CRANE adaptive flag, add
per-role decoding defaults.
Estimated effort: 2-3 sessions.
Pre-requisites: C-10 (GoalDriftCanary + ThinkingPersona + AdaptiveFutilityTracker
ship first — output-similarity futility and persona switch consume those types
directly); C-25 (CumulativeTokenTracker for measurement wiring in smoke test).
Parallel-safe with: C-21, C-22, C-23, C-24.

## Context

token-efficiency-design.report.md §MISSING (2026-05-17): ConstrainedDecodingConfig
and DecodingMode have zero DEF/SCIP_DEF. json_schema_mode exists only as a bare bool
(providers/src/runtime/types.rs:157); ModelFeatures.supports_grammar always false
(context_budget.rs:2737). CRANE and EBNF grammar files have no evidence. BubbleUpPolicy
(pipeline_tree_navigator.rs:169-184) has only max_retries counts — no RetryStrategy,
no temperature, no futility. Benchmark measurements 3/4/5 report 0% (fixture path bug).
update-implementation-order.report.md §MISSING: B-1 + B-2 (constrained decoding) are
the top two outstanding action items.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\token-efficiency-design.report.md
  (§MISSING + §PARTIAL for exact gap boundaries)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\update-implementation-order.report.md
  (B-1/B-2 action items)
- .elai_cc/crates/providers/src/runtime/types.rs (:157 json_schema_mode bool)
- .elai_cc/crates/providers/src/sidecar_client.rs (:640 ETC threshold — struct layout)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (:169-184 BubbleUpPolicy)
- .elai_cc/crates/orchestrator/src/context_budget.rs (:241 supports_grammar, :2737 profiles)
- .elai_cc/benches/model_capability/run_benchmark.py (fixture path causing 0% outputs)
- .elai_cc/metrics/model_baseline.toml (measurements 1+2 present; 3/4/5 zero)
- .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs (C-10 — confirm types exist)

## Tasks

1. **New file: crates/providers/src/decoding.rs**
   - DecodingMode enum: JsonSchema { schema_json: String } | Grammar { ebnf_path:
     PathBuf, crane_adaptive: bool } | Regex { pattern: String } | FreeText.
     Derive Debug, Clone, PartialEq, Serialize, Deserialize.
   - ConstrainedDecodingConfig: { mode: DecodingMode, max_retries_on_invalid: u8=3,
     strict: bool=true }. Default produces FreeText. Derive Debug, Clone, Default,
     Serialize, Deserialize.
   - crane_adaptive_eligible(mode: &DecodingMode) -> bool: true only for Grammar {
     crane_adaptive: true, .. }.
   - PER_ROLE_DECODING_DEFAULTS: [(RoleDecoding, ConstrainedDecodingConfig); 5]:
     designer=FreeText, executor=Grammar(function_signature.ebnf, crane_adaptive=true),
     critic=JsonSchema, auditor=JsonSchema, researcher=FreeText. Use local RoleDecoding
     enum (not cross-crate AgentRole import) to avoid circular dep; document in
     decisions file. No new crate deps; no .unwrap().
   File: .elai_cc/crates/providers/src/decoding.rs (ADD)

2. **RetryStrategy + BubbleUpPolicy extension (pipeline_tree_navigator.rs)**
   - RetryStrategy { temperature_schedule: Vec<f32> (default [0.0, 0.2, 0.5]),
     futility_same_error_threshold: u8=3, decompose_on_failure: bool=false }.
     Derive Debug, Clone, PartialEq, Serialize, Deserialize.
   - Add retry_strategy: RetryStrategy field to BubbleUpPolicy (Default::default()).
     Update all ~4 literal sites (:1757, :1772, :1882, :2009) with struct-update syntax;
     verify callers via elai.map before editing.
   - temperature_for_attempt(attempt: u32) -> f32: saturating index into schedule;
     returns last entry if attempt >= schedule.len(). No panic on empty — default 0.0.
   - No change to existing retry count logic.
   File: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (MODIFY)

3. **Futility detection in NavigatorTree**
   Two paths after each BabyStepResult:
   a) Same-error-class: track last N FailureRecord.reason strings per baby_step in
      VecDeque<String> (capacity = futility_same_error_threshold). All N equal ->
      emit futility_detected SSB { error_class, consecutive_count, level="baby_step" };
      escalate via BubbleUpAction::Escalate.
   b) Output-similarity: call GoalDriftCanary::compute(prev_output, curr_output) -> f32
      (Jaccard, from C-10). Score > 0.90 -> emit futility_output_stagnation SSB
      { similarity, threshold=0.90, level }; set navigator.suggested_persona via
      suggest_persona_for_pattern(StagnationPattern::NoDrift). Both paths compile
      to no-op if C-10 anti_spiral_extended.rs is absent (conditional use block;
      document strategy in decisions file). No .unwrap().
   File: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (MODIFY)

4. **Wire ConstrainedDecodingConfig into SidecarClient**
   Read sidecar_client.rs around :640 ETC threshold before editing. Add
   decoding_config: Option<ConstrainedDecodingConfig> to outbound request builder.
   JsonSchema -> request_body["response_format"] = {"type":"json_object","schema":...}.
   Grammar -> request_body["grammar_path"] = ebnf_path string.
   FreeText -> no extra fields. On constrained_decoding_unavailable (server.py:117):
   tracing::warn! + FreeText fallback (no panic). serde_json already in workspace;
   verify before use.
   File: .elai_cc/crates/providers/src/sidecar_client.rs (MODIFY)

5. **Wire ConstrainedDecodingConfig into CloudGroundingClient**
   Grep elai.map for CloudGroundingClient DEF before editing (path unverified;
   may be crates/orchestrator/src/ or providers/src/). Add decoding_config to
   Anthropic API request builder. JsonSchema -> Anthropic Tool API encoding:
   tools=[{name:"structured_output", input_schema: schema_json}] +
   tool_choice={type:"tool", name:"structured_output"} (Sonnet 4.6 supports this
   per update-implementation-order.report.md). Grammar/FreeText -> no extra fields
   (Grammar is sidecar-only). No new deps.
   File: .elai_cc/crates/orchestrator/src/grounding_client.rs (confirm path) (MODIFY)

6. **Fix benchmark fixture path bug + re-run + update model_baseline.toml**
   Read benchmark script (benches/model_capability/run_benchmark.py) to locate the
   fixture path reference producing 0% for measurements 3/4/5. Report §PARTIAL
   identifies fixtures/templates.json and fixtures/padding.txt as missing.
   a) Create fixtures/templates.json (3 template entries, valid JSON) and
      fixtures/padding.txt (100 lines) to unblock harness.
   b) Fix fixture path reference in the script (verify current value vs actual path).
   c) Run harness; record actual non-zero measurement values. Update model_baseline.toml.
      Do NOT fabricate values.
   Files: benches/model_capability/run_benchmark.py (MODIFY),
          benches/model_capability/fixtures/templates.json (ADD),
          benches/model_capability/fixtures/padding.txt (ADD),
          .elai_cc/metrics/model_baseline.toml (MODIFY)

7. **EBNF grammar files**
   Add under .elai_cc/crates/orchestrator/src/grammars/:
   - import_block.ebnf (~20 lines): Rust `use crate::...;` + Python `import X` /
     `from X import Y` common subset.
   - function_signature.ebnf (~25 lines): Rust `fn name(args) -> R` + Python
     `def name(args) -> R:` signature lines.
   - error_type.ebnf (~15 lines): CamelCase identifiers ending in Error/Err/Exception;
     qualified paths like `std::io::Error`.
   Grammar mode ebnf_path resolution strategy (cwd-relative vs binary-embedded)
   documented in decisions file. Files are non-empty; each covers its stated pattern.
   Files: .elai_cc/crates/orchestrator/src/grammars/import_block.ebnf (ADD)
          .elai_cc/crates/orchestrator/src/grammars/function_signature.ebnf (ADD)
          .elai_cc/crates/orchestrator/src/grammars/error_type.ebnf (ADD)

8. **Contract tests: constrained_decoding_contract.rs** (12 functions)
   File: .elai_cc/crates/contracts/tests/constrained_decoding_contract.rs
   - test_decoding_mode_default_is_free_text
   - test_json_schema_mode_serializes_schema_json_field
   - test_grammar_mode_crane_adaptive_true_eligible
   - test_grammar_mode_crane_adaptive_false_not_eligible
   - test_retry_strategy_temperature_at_attempt_0_is_zero
   - test_retry_strategy_temperature_clamps_at_last_entry
   - test_bubble_up_policy_default_includes_retry_strategy
   - test_futility_same_error_3x_triggers_escalation
   - test_futility_output_similarity_90pct_triggers_persona_switch
     (#[ignore] if C-10 not yet shipped)
   - test_per_role_defaults_executor_is_grammar
   - test_per_role_defaults_auditor_is_json_schema
   - test_per_role_defaults_designer_is_free_text

9. **Contract tests: benchmark_fixture_contract.rs** (3 functions)
   File: .elai_cc/crates/contracts/tests/benchmark_fixture_contract.rs
   - test_templates_json_exists_and_is_valid_json
   - test_padding_txt_exists_and_has_content
   - test_measurements_345_nonzero_in_model_baseline_toml

10. **Decision doc**
    workflow/reasoning/C-26-decisions.md: (a) RoleDecoding local enum vs AgentRole
    cross-crate import; (b) ebnf_path resolution strategy; (c) temperature_schedule
    [0.0, 0.2, 0.5] staged-warming rationale; (d) silent FreeText fallback vs error
    on constrained_decoding_unavailable; (e) fixture placeholder justification.

## WIRING_CONTRACT

produces_symbols:
  - DecodingMode enum (4 variants: JsonSchema, Grammar, Regex, FreeText)
  - ConstrainedDecodingConfig struct (mode, max_retries_on_invalid, strict)
  - crane_adaptive_eligible() fn
  - PER_ROLE_DECODING_DEFAULTS constant table
  - RetryStrategy struct (temperature_schedule, futility_same_error_threshold,
    decompose_on_failure) + temperature_for_attempt() method
  - BubbleUpPolicy.retry_strategy field (extends existing struct)
  - NavigatorTree futility detection (same-error + output-similarity paths)
  - SSB action types: futility_detected, futility_output_stagnation
  - SidecarClient decoding_config request field + FreeText fallback on error
  - CloudGroundingClient json_schema tool-use request encoding
  - grammars/import_block.ebnf, grammars/function_signature.ebnf,
    grammars/error_type.ebnf
  - Benchmark fixture files + model_baseline.toml measurements 3/4/5 (non-zero)
  - 15 contract tests (2 files)

consumes_symbols:
  - C-10: GoalDriftCanary::compute(), ThinkingPersona, suggest_persona_for_pattern()
  - C-25: CumulativeTokenTracker (smoke test token_count comparison)
  - pipeline_tree_navigator.rs: BubbleUpPolicy, BabyStepResult, FailureRecord,
    NavigatorTree, BubbleUpAction::Escalate
  - ssb.rs: SsbActionType enum (add futility_detected + futility_output_stagnation)
  - context_budget.rs: ModelFeatures.supports_grammar (patch profiles for Grammar mode)

reachable_from:
  - elai /task (executor=Grammar, critic/auditor=JsonSchema defaults)
  - elai /task --mode cloud (CloudGroundingClient json_schema path)
  - elai bench token-efficiency-baseline
  - NavigatorTree retry loop at each BabyStepResult

stub_ban:
  - No .unwrap() in decoding.rs or new pipeline_tree_navigator.rs paths
  - ConstrainedDecodingConfig::Default MUST produce FreeText (no accidental grammar)
  - temperature_for_attempt() MUST NOT panic on empty schedule; default to 0.0
  - SidecarClient fallback on constrained_decoding_unavailable MUST log tracing::warn!
    (not silently swallow)
  - model_baseline.toml MUST contain actual measured values not fabricated floats
  - EBNF grammar files MUST NOT be empty stubs; each must cover the stated pattern

observable_effect:
  trigger: elai /task --mode cloud "produce a JSON config matching schema X"
    with ConstrainedDecodingConfig::JsonSchema active for critic role
  expect_ssb: SSB row constrained_decoding_used with mode=JsonSchema,
    validation=Pass on first attempt (no retry); zero futility_detected rows
  expect_ssb (adversarial): run task with a Regex pattern that always rejects;
    after 3x same error: futility_detected row (error_class=constrained_decode_reject,
    consecutive_count=3); persona switch row (ThinkingPersona variant)
  expect_bench: elai bench token-efficiency-baseline produces non-zero delta for
    measurements 3/4/5; model_baseline.toml updated with results

## DETERMINISTIC_CHECK

```bash
# 1. DecodingMode + ConstrainedDecodingConfig in decoding.rs
rg "enum DecodingMode|struct ConstrainedDecodingConfig" \
  .elai_cc/crates/providers/src/decoding.rs

# 2. RetryStrategy in pipeline_tree_navigator.rs
rg "struct RetryStrategy|temperature_schedule|futility_same_error_threshold" \
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 3. BubbleUpPolicy gains retry_strategy field
rg "retry_strategy" \
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 4. Futility SSB types in ssb.rs
rg "futility_detected|futility_output_stagnation" \
  .elai_cc/crates/orchestrator/src/ssb.rs

# 5. SidecarClient decoding_config wired
rg "decoding_config|ConstrainedDecodingConfig" \
  .elai_cc/crates/providers/src/sidecar_client.rs

# 6. EBNF grammar files present
powershell -c "Test-Path .elai_cc/crates/orchestrator/src/grammars/import_block.ebnf; \
  Test-Path .elai_cc/crates/orchestrator/src/grammars/function_signature.ebnf; \
  Test-Path .elai_cc/crates/orchestrator/src/grammars/error_type.ebnf"
# Expected: True True True

# 7. Fixture files present
powershell -c "Test-Path .elai_cc/benches/model_capability/fixtures/templates.json; \
  Test-Path .elai_cc/benches/model_capability/fixtures/padding.txt"
# Expected: True True

# 8. model_baseline.toml measurements 3/4/5 non-zero
rg "measurement_3|measurement_4|measurement_5" .elai_cc/metrics/model_baseline.toml
# Expected: 3 lines, all values != 0.0

# 9. Per-role defaults table
rg "PER_ROLE_DECODING_DEFAULTS" .elai_cc/crates/providers/src/decoding.rs

# 10. Contract tests pass
cargo test -p elai_contracts -- constrained_decoding benchmark_fixture \
  2>&1 | grep "test result"
# Expected: >= 15 passed

# 11. Build clean
cargo check -p elai_providers -p elai_orchestrator -p elai_contracts \
  2>&1 | tail -3
```

## ACCEPTANCE

- [ ] decoding.rs: DecodingMode (4 variants), ConstrainedDecodingConfig, crane_adaptive_eligible(),
      PER_ROLE_DECODING_DEFAULTS (5 entries); Default = FreeText
- [ ] RetryStrategy: temperature_schedule=[0.0,0.2,0.5] default; temperature_for_attempt()
      clamps to last entry; no panic on empty schedule
- [ ] BubbleUpPolicy.retry_strategy field added; all ~4 literal sites compile (struct-update)
- [ ] NavigatorTree: same-error 3x -> futility_detected SSB + Escalate (contract-tested)
- [ ] NavigatorTree: similarity > 0.90 -> futility_output_stagnation SSB + suggested_persona
      (#[ignore] if C-10 absent)
- [ ] SidecarClient: JsonSchema -> response_format; Grammar -> grammar_path; FreeText -> no-op;
      constrained_decoding_unavailable -> tracing::warn! + fallback
- [ ] CloudGroundingClient: JsonSchema -> Anthropic Tool API (tools + tool_choice); others unchanged
- [ ] Three EBNF grammar files non-empty; each covers stated pattern
- [ ] Fixture path fixed; templates.json + padding.txt created; harness runs clean
- [ ] model_baseline.toml measurements 3/4/5 non-zero (actual measured values)
- [ ] futility_detected + futility_output_stagnation in SsbActionType
- [ ] 12 constrained_decoding_contract + 3 benchmark_fixture_contract pass
- [ ] cargo check -p elai_providers -p elai_orchestrator -p elai_contracts: 0 errors
- [ ] Cloud-Sonnet smoke test PROVEN:
      (a) elai /task --mode cloud "produce a JSON config matching schema X":
          SSB constrained_decoding_used mode=JsonSchema, validation=Pass, retry_count=0
      (b) Adversarial (always-failing Regex, 3x): futility_detected SSB
          (error_class=constrained_decode_reject, consecutive_count=3); persona switch SSB
          (new_persona from ThinkingPersona)
      (c) elai bench token-efficiency-baseline: measurements 3/4/5 non-zero in output;
          model_baseline.toml updated on disk
- [ ] Decision doc workflow/reasoning/C-26-decisions.md (5 decisions)

## Files touched

- ADD: .elai_cc/crates/providers/src/decoding.rs
- MODIFY: .elai_cc/crates/providers/src/sidecar_client.rs
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_client.rs (confirm path via elai.map)
- MODIFY: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
- ADD: .elai_cc/crates/orchestrator/src/grammars/import_block.ebnf
- ADD: .elai_cc/crates/orchestrator/src/grammars/function_signature.ebnf
- ADD: .elai_cc/crates/orchestrator/src/grammars/error_type.ebnf
- MODIFY: .elai_cc/benches/model_capability/run_benchmark.py
- ADD: .elai_cc/benches/model_capability/fixtures/templates.json
- ADD: .elai_cc/benches/model_capability/fixtures/padding.txt
- MODIFY: .elai_cc/metrics/model_baseline.toml
- ADD: .elai_cc/crates/contracts/tests/constrained_decoding_contract.rs
- ADD: .elai_cc/crates/contracts/tests/benchmark_fixture_contract.rs
- CREATE: workflow/reasoning/C-26-decisions.md

## Next

C-27 or remaining C-2x. Confirm C-10 shipped before opening the output-similarity
futility path; it cfg-gates to a no-op if absent but smoke test (b) requires
ThinkingPersona to fire.
